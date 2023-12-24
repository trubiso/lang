use crate::parser::types::{ParserExpr, ParserStmt, ScopeRecursive};
use chumsky::prelude::*;

macro_rules! set_stmt {
	($s:expr, $ident:ident) => {
		assg!($s, $ident).map(|((lhs, _), rhs)| ParserStmt::Set {id: lhs, value: rhs})
	};
	($s:expr, operator $op:ident) => {
		assg!($s, $op -> Set).map(|(((lhs, op), _), rhs)| {
			ParserStmt::Set {
				id: lhs.clone(),
				value: ParserExpr::BinaryOp(
					Box::new(ParserExpr::Identifier(lhs)),
					force_token!(op => Operator),
					Box::new(rhs),
				),
			}
		})
	};
}

pub fn stmt(s: ScopeRecursive) -> token_parser!(ParserStmt : '_) {
	choice((
		set_stmt!(s.clone(), Set),
		set_stmt!(s.clone(), operator Neg),
		set_stmt!(s.clone(), operator Star),
		set_stmt!(s.clone(), operator Plus),
		set_stmt!(s, operator Div),
	))
}
