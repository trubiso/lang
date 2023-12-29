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

impl Spanned<TypedIdent> {
	#[must_use]
	pub fn ident(&self) -> &Ident {
		&self.value.ident.value
	}

	#[must_use]
	pub fn ty(&self) -> &Type {
		&self.value.ty.value
	}
}
