use crate::{
	common::ident::Ident,
	lexer::{Keyword, Token},
};
use chumsky::prelude::*;

pub fn ident() -> token_parser!(Ident) {
	filter(|token| matches!(token, Token::Identifier(_))).map(|token| {
		if token.is_keyword(Keyword::DontCare) {
			Ident::Discarded
		} else {
			force_token!(token => Identifier)
		}
	})
}

pub fn ident_nodiscard() -> token_parser!(Ident) {
	ident().validate(|ident, span, emit| {
		if ident.is_discarded() {
			emit(Simple::custom(
				span,
				"discarded ident was used where it is disallowed",
			))
		}
		ident
	})
}

// TODO: potentially_qualified_ident
