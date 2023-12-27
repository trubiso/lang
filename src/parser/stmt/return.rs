use crate::{
	lexer::Keyword,
	parser::{
		core::expr::expr,
		types::{ParserStmt, ScopeRecursive},
	},
};
use chumsky::prelude::*;

pub fn stmt(s: ScopeRecursive) -> token_parser_no_span!(ParserStmt : '_) {
	jkeyword!(Return)
		.or(jkeyword!(Yield))
		.then(expr(s))
		.map(|(token, expr)| ParserStmt::Return {
			value: expr,
			is_yield: token.is_keyword(Keyword::Yield),
		})
}
