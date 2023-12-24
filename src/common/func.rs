use crate::common::{ident::Ident, r#type::Type, typed_ident::TypedIdent};

use super::scope::Scope;

#[derive(Debug, Default, Clone)]
pub struct FuncAttribs {
	pub is_pure: bool,
	pub is_unsafe: bool,
}

#[derive(Debug, Default, Clone)]
pub enum FuncLinkage {
	External,
	#[default]
	Default,
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
	pub attribs: FuncAttribs,
	pub linkage: FuncLinkage,
	pub return_ty: Type,
	pub args: Vec<TypedIdent>,
	pub generics: Vec<Ident>,
}

pub struct Func<Sc: Scope> {
	pub signature: FuncSignature,
	pub body: Sc,
}
