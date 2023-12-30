use super::{join::Join, scope::Scope, span::Spanned};
use crate::common::{ident::Ident, r#type::Type, typed_ident::TypedIdent};
use derive_more::Display;

#[derive(Debug, Default, Clone)]
pub struct Attribs {
	pub is_pure: bool,
	pub is_unsafe: bool,
}

#[derive(Debug, Default, Display, Clone)]
pub enum Linkage {
	#[display(fmt = "extern ")]
	External,
	#[default]
	#[display(fmt = "")]
	Default,
}

#[derive(Debug, Clone)]
pub struct Signature {
	pub attribs: Spanned<Attribs>,
	pub linkage: Spanned<Linkage>,
	pub return_ty: Spanned<Type>,
	pub args: Spanned<Vec<Spanned<TypedIdent>>>,
	pub generics: Spanned<Vec<Spanned<Ident>>>,
}

#[derive(Debug, Clone)]
pub struct Func<Sc: Scope> {
	pub id: Spanned<Ident>,
	pub signature: Signature,
	pub body: Option<Spanned<Sc>>,
}

impl std::fmt::Display for Attribs {
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

impl std::fmt::Display for Signature {
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

impl<Sc: Scope + std::fmt::Display> std::fmt::Display for Func<Sc> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(
			"{} [{}] {}",
			self.id,
			self.signature,
			self.body.as_ref().map_or("".into(), |x| format!("{x}"))
		))
	}
}
