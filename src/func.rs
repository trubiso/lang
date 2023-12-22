use crate::{r#type::Type, typed_ident::TypedIdent, ident::Ident};

pub struct Func<Sc> {
	pub return_ty: Type,
	pub args: Vec<TypedIdent>,
	pub generics: Vec<Ident>,
	pub body: Sc,
}