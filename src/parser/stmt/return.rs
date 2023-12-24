use crate::parser::{
	core::expr::expr,
	types::{ParserStmt, ScopeRecursive},
};
use chumsky::prelude::*;

pub fn return_stmt(s: ScopeRecursive) -> token_parser!(ParserStmt : '_) {
	jkeyword!(Return)
		.ignore_then(expr(s))
		.map(|expr| ParserStmt::Return { value: expr })
}
