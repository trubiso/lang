use super::span::Spanned;
use crate::common::{ident::Ident, r#type::Type};
use derive_more::Display;

#[derive(Debug, Display, Clone)]
#[display(fmt = "{ty} {ident}")]
pub struct TypedIdent {
	pub ty: Spanned<Type>,
	pub ident: Spanned<Ident>,
}

impl TypedIdent {
	#[must_use]
	pub fn ident_str(&self) -> String {
		self.ident.value.to_string()
	}
}
