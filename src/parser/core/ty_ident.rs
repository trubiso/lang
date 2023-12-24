use super::ident::{ident, ident_nodiscard};
use super::ty::ty;
use crate::common::typed_ident::TypedIdent;
use chumsky::Parser;

pub fn ty_ident() -> token_parser!(TypedIdent) {
	ty().then(ident())
		.map(|(ty, ident)| TypedIdent { ty, ident })
}

pub fn ty_ident_nodiscard() -> token_parser!(TypedIdent) {
	ty().then(ident_nodiscard())
		.map(|(ty, ident)| TypedIdent { ty, ident })
}
