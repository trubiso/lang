use crate::parser::{expr::expr, types::ParserStmt};
use chumsky::prelude::*;

pub fn return_stmt() -> token_parser!(ParserStmt) {
	jkeyword!(Return)
		.ignore_then(expr())
		.map(|expr| ParserStmt::Return { value: expr })
}
