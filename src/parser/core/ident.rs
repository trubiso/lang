use crate::{
	common::{ident::Ident, span::AddSpan},
	lexer::{Keyword, Token},
};
use chumsky::prelude::*;

pub fn ident() -> token_parser!(Ident) {
	filter(|token| matches!(token, Token::Identifier(_))).map_with_span(|token, span| {
		if token.is_keyword(Keyword::DontCare) {
			Ident::Discarded
		} else {
			force_token!(token => Identifier)
		}
		.add_span(span)
	})
}

pub fn ident_nospan() -> token_parser_no_span!(Ident) {
	ident().map(|x| x.value)
}

pub fn ident_nodiscard() -> token_parser!(Ident) {
	ident().validate(|ident, span, emit| {
		if ident.value.is_discarded() {
			emit(Simple::custom(
				span,
				"discarded ident was used where it is disallowed",
			));
		}
		ident
	})
}

// TODO: potentially_qualified_ident
