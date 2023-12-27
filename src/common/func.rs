use super::{scope::Scope, span::Spanned};
use crate::common::{ident::Ident, r#type::Type, typed_ident::TypedIdent};

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
	pub attribs: Spanned<FuncAttribs>,
	pub linkage: Spanned<FuncLinkage>,
	pub return_ty: Spanned<Type>,
	pub args: Spanned<Vec<Spanned<TypedIdent>>>,
	pub generics: Spanned<Vec<Spanned<Ident>>>,
}

pub struct Func<Sc: Scope> {
	pub signature: FuncSignature,
	pub body: Sc,
}
