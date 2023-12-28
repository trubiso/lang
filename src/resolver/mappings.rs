use super::{add_diagnostic, count};
use crate::common::{
	ident::{Id, Ident},
	span::Span,
};
use bimap::BiMap;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use derive_more::Display;
use std::collections::HashMap;

#[derive(Clone, Display, PartialEq, Eq)]
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
	pub fn get_or_add_id(&mut self, id: &Ident) -> Id {
		match self.mappings.get_by_right(id) {
			Some(x) => *x,
			None => {
				let new_id = count();
				self.mappings.insert(new_id, id.clone());
				new_id
			}
		}
	}

	pub fn get_by_id(&self, id: &Id) -> Option<&Ident> {
		self.mappings.get_by_left(id)
	}

	pub fn get_by_ident(&self, ident: &Ident) -> Option<&Id> {
		self.mappings.get_by_right(ident)
	}

	pub fn get_repr(&self, id: &Id) -> Option<MapRepr> {
		self.reprs.get(id).cloned()
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
					type_mismatch_diagnostic(span, x, want);
				}
			}
			None => {
				self.set_repr(&id, want);
			}
		}
	}
}

fn type_mismatch_diagnostic(span: Span, used: MapRepr, desired: MapRepr) {
	add_diagnostic(
		Diagnostic::error()
			.with_message("type mismatch")
			.with_labels(vec![Label::primary(span.file_id, span.range())
				.with_message(format!("used {} as {}", used, desired))]),
	)
}
