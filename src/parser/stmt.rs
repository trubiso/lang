use super::types::{ParserStmt, ScopeRecursive};
use crate::common::span::AddSpan;
use chumsky::prelude::*;

mod create;
mod func;
mod r#return;
mod set;

pub fn stmt(s: ScopeRecursive) -> token_parser!(ParserStmt : '_) {
	macro_rules! semi {
		(Y $stmt:expr) => {
			$stmt.then_ignore(jpunct!(Semicolon).repeated().at_least(1))
		};
		(N $stmt:expr) => {
			$stmt.then_ignore(jpunct!(Semicolon).repeated())
		};
	}
	choice((
		semi!(Y r#return::stmt(s.clone())),
		semi!(Y create::stmt(s.clone())),
		semi!(Y set::stmt(s.clone())),
		semi!(N func::stmt(s)),
	))
	.map_with_span(|stmt, span| stmt.add_span(span))
}
