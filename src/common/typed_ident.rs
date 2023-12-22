use crate::common::{ident::Ident, r#type::Type};
use derive_more::Display;

#[derive(Debug, Display, Clone)]
#[display(fmt = "{ty} {ident}")]
pub struct TypedIdent {
	pub ty: Type,
	pub ident: Ident,
}

impl TypedIdent {
	pub fn ident_str(&self) -> String {
		self.ident.to_string()
	}
}
