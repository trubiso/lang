use crate::common::{ident::Ident, r#type::Type, typed_ident::TypedIdent};

use super::scope::Scope;

#[derive(Debug, Clone)]
pub struct FuncSignature {
	pub return_ty: Type,
	pub args: Vec<TypedIdent>,
	pub generics: Vec<Ident>,
}

pub struct Func<Sc: Scope> {
	pub signature: FuncSignature,
	pub body: Sc,
}
