use crate::{
	lexer::Keyword,
	parser::{
		core::expr::expr,
		types::{ParserStmt, ScopeRecursive},
	},
};
use chumsky::prelude::*;

pub fn return_stmt(s: ScopeRecursive) -> token_parser!(ParserStmt : '_) {
	jkeyword!(Return)
		.or(jkeyword!(Yield))
		.then(expr(s))
		.map(|(token, expr)| ParserStmt::Return {
			value: expr,
			is_yield: token.is_keyword(Keyword::Yield),
		})
}
