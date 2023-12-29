use super::{join::Join, scope::Scope, span::Spanned};
use crate::common::{ident::Ident, r#type::Type, typed_ident::TypedIdent};
use derive_more::Display;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FuncAttribs {
	pub is_pure: bool,
	pub is_unsafe: bool,
}

#[derive(Debug, Default, Display, Clone, PartialEq, Eq)]
pub enum FuncLinkage {
	#[display(fmt = "extern ")]
	External,
	#[default]
	#[display(fmt = "")]
	Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl std::fmt::Display for FuncAttribs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_pure {
			f.write_str("pure ")?;
		}
		if self.is_unsafe {
			f.write_str("unsafe ")?;
		}
		Ok(())
	}
}

impl std::fmt::Display for FuncSignature {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(
			"{}{}{}{} -> {}",
			self.linkage,
			(&self.generics.value).join_comma_wrapped("<", ">"),
			(&self.args.value).join_comma_wrapped("(", ")"),
			self.attribs,
			self.return_ty
		))
	}
}
