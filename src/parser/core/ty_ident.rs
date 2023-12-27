use super::ident::{ident, ident_nodiscard};
use super::ty::ty;
use crate::common::span::AddSpan;
use crate::common::typed_ident::TypedIdent;
use chumsky::Parser;

pub fn ty_ident() -> token_parser!(TypedIdent) {
	ty().then(ident())
		.map_with_span(|(ty, ident), span| TypedIdent { ty, ident }.add_span(span))
}

pub fn ty_ident_nodiscard() -> token_parser!(TypedIdent) {
	ty().then(ident_nodiscard())
		.map_with_span(|(ty, ident), span| TypedIdent { ty, ident }.add_span(span))
}
