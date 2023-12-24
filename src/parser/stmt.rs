use self::{
	create::create_stmt, func::func_stmt, r#return::return_stmt,
	set::set_stmt,
};
use super::types::{ParserStmt, ScopeRecursive};
use chumsky::prelude::*;

mod create;
mod func;
mod r#return;
mod set;

pub fn stmt(scope: ScopeRecursive) -> token_parser!(ParserStmt : '_) {
	macro_rules! semi {
		(Y $stmt:expr) => {
			$stmt.then_ignore(jpunct!(Semicolon).repeated().at_least(1))
		};
		(N $stmt:expr) => {
			$stmt.then_ignore(jpunct!(Semicolon).repeated())
		};
	}
	choice((
		semi!(Y return_stmt()),
		semi!(Y create_stmt()),
		semi!(Y set_stmt()),
		semi!(N func_stmt(scope)), // TODO: declare funcs
	))
}
