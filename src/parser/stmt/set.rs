use crate::parser::types::{ParserExpr, ParserStmt};
use chumsky::prelude::*;

macro_rules! set_stmt {
	($ident:ident) => {
		assg!($ident).map(|((lhs, _), rhs)| ParserStmt::Set {id: lhs, value: rhs})
	};
	(operator $op:ident) => {
		assg!($op -> Set).map(|(((lhs, op), _), rhs)| {
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

pub fn set_stmt() -> token_parser!(ParserStmt) {
	choice((
		set_stmt!(Set),
		set_stmt!(operator Neg),
		set_stmt!(operator Star),
		set_stmt!(operator Plus),
		set_stmt!(operator Div),
	))
}
