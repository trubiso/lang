use self::{create::create_stmt, declare::declare_stmt, set::set_stmt};
use super::types::{ParserStmt, ScopeRecursive};
use chumsky::prelude::*;

mod create;
mod declare;
mod set;

pub fn stmt(_scope: ScopeRecursive) -> token_parser!(ParserStmt) {
	choice((create_stmt(), declare_stmt(), set_stmt()))
		.then_ignore(jpunct!(Semicolon).repeated().at_least(1))
}
