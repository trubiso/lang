use super::count;
use crate::common::{
	diagnostics::type_mismatch,
	ident::{Id, Ident},
	span::Span,
};
use bimap::BiMap;
use derive_more::Display;
use std::collections::HashMap;

#[derive(Clone, Copy, Display, PartialEq, Eq)]
pub enum MapRepr {
	#[display(fmt = "variable")]
	Var,
	#[display(fmt = "function")]
	Func,
	#[display(fmt = "type")]
	Type,
}

#[derive(Clone, Default)]
pub struct Mappings {
	pub mappings: BiMap<Id, Ident>,
	pub reprs: HashMap<Id, MapRepr>,
}

impl Mappings {
	#[must_use]
	pub fn get_or_add_id(&mut self, id: &Ident) -> Id {
		if let Some(x) = self.mappings.get_by_right(id) {
			*x
		} else {
			let new_id = count();
			self.mappings.insert(new_id, id.clone());
			new_id
		}
	}

	#[must_use]
	pub fn get_by_id(&self, id: &Id) -> Option<&Ident> {
		self.mappings.get_by_left(id)
	}

	#[must_use]
	pub fn get_by_ident(&self, ident: &Ident) -> Option<&Id> {
		self.mappings.get_by_right(ident)
	}

	#[must_use]
	pub fn get_repr(&self, id: &Id) -> Option<MapRepr> {
		self.reprs.get(id).copied()
	}

	pub fn set_repr(&mut self, id: &Id, repr: MapRepr) {
		self.reprs.insert(*id, repr);
	}

	pub fn insert(&mut self, id: Id, ident: Ident, repr: MapRepr) {
		self.mappings.insert(id, ident);
		self.reprs.insert(id, repr);
	}

	pub fn insert_var(&mut self, id: Id, ident: Ident) {
		self.insert(id, ident, MapRepr::Var);
	}

	pub fn insert_func(&mut self, id: Id, ident: Ident) {
		self.insert(id, ident, MapRepr::Func);
	}

	pub fn insert_ty(&mut self, id: Id, ident: Ident) {
		self.insert(id, ident, MapRepr::Type);
	}

	pub fn ensure_repr(&mut self, id: Id, want: MapRepr, span: Span) {
		match self.get_repr(&id) {
			Some(x) => {
				if x != want {
					type_mismatch(span, x, want);
				}
			}
			None => {
				self.set_repr(&id, want);
			}
		}
	}
}
