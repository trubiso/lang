use super::ident::{self, ident};
use super::ty::ty;
use crate::common::span::Add;
use crate::common::typed_ident::TypedIdent;
use chumsky::Parser;

pub fn ty_ident() -> token_parser!(TypedIdent) {
	ty().then(ident())
		.map_with_span(|(ty, ident), span| TypedIdent { ty, ident }.add_span(span))
}

pub fn nodiscard() -> token_parser!(TypedIdent) {
	ty().then(ident::nodiscard())
		.map_with_span(|(ty, ident), span| TypedIdent { ty, ident }.add_span(span))
}
