use super::type_info::TypeId;
use crate::common::{ident::Id, span::Spanned};
use std::collections::HashMap;

/// Maps names to types, disambiguating variable names and type names.
///
/// Holds mappings from variable/type names (stored as `Id`) to their types in
/// the inference engine (`Spanned<TypeId>`), differentiating between
/// associations made for types pertaining to variables ("variable types") and
/// types actually associated to a type in code, such as a `Type::User` or a
/// generic ("named types").
#[derive(Debug, Default)]
pub struct Mappings {
	named_tys: HashMap<Id, Spanned<TypeId>>,
	var_tys: HashMap<Id, Spanned<TypeId>>,
}

impl Mappings {
	/// Gets the named type associated to the provided `Id`.
	///
	/// # Panics
	///
	/// The function will panic if there is no named type associated to the
	/// provided `Id`. In the panic message, it will list all available `Id`s
	/// and their respective `TypeId`s to help fix whichever bug caused this
	/// invalid access.
	#[must_use]
	pub fn get_named_ty(&self, id: Id) -> &Spanned<TypeId> {
		self.named_tys.get(&id).unwrap_or_else(|| {
			panic!(
				"tried to access nonexistent named type {} (stored named types: {:?})",
				id, self.named_tys
			)
		})
	}

	/// Gets the variable type associated to the provided `Id`.
	///
	/// # Panics
	///
	/// The function will panic if there is no variable type associated to the
	/// provided `Id`. In the panic message, it will list all available `Id`s
	/// and their respective `TypeId`s to help fix whichever bug caused this
	/// invalid access.
	#[must_use]
	pub fn get_var_ty(&self, id: Id) -> &Spanned<TypeId> {
		self.var_tys.get(&id).unwrap_or_else(|| {
			panic!(
				"tried to access nonexistent var type {} (stored var types: {:?})",
				id, self.var_tys
			)
		})
	}

	/// Registers a mapping between the provided type name and the provided
	/// named type.
	pub fn insert_named_ty(&mut self, id: Id, ty: Spanned<TypeId>) {
		self.named_tys.insert(id, ty);
	}

	/// Registers a mapping between the provided variable name and the provided
	/// variable type.
	pub fn insert_var_ty(&mut self, id: Id, ty: Spanned<TypeId>) {
		self.var_tys.insert(id, ty);
	}

	/// Gets the `Id` associated to the provided `TypeId`.
	///
	/// This function is really inefficient and should only be called for
	/// debugging purposes.
	pub fn get_id_from_ty(&self, ty: TypeId) -> Option<Id> {
		for (id, t) in &self.var_tys {
			if t.value == ty {
				return Some(*id);
			}
		}
		for (id, t) in &self.named_tys {
			if t.value == ty {
				return Some(*id);
			}
		}
		None
	}
}
