use crate::common::{ident::Ident, r#type::Type, typed_ident::TypedIdent};

#[derive(Debug, Clone)]
pub struct Func<Sc> {
	pub return_ty: Type,
	pub args: Vec<TypedIdent>,
	pub generics: Vec<Ident>,
	pub body: Sc,
}
