use crate::{
	common::span::Add,
	parser::{
		core::{expr::expr, ident::ident},
		types::{ParserExpr, ParserStmt, ScopeRecursive},
	},
};
use chumsky::prelude::*;

macro_rules! set_stmt {
	($s:expr, $ident:ident) => {
		assg!($s, $ident).map(|((lhs, _), rhs)| ParserStmt::Set {id: lhs, value: rhs})
	};
	($s:expr, operator $op:ident) => {
		ident()
		.then(span!(jop!($op)))
		.then(jassg_op!(Set))
		.then(expr($s))
		.map(|(((lhs, op), _), rhs)| {
			let lhs_span = lhs.span.clone();
			let rhs_span = rhs.span.clone();
			ParserStmt::Set {
				id: lhs.clone(),
				value: ParserExpr::BinaryOp(
					Box::new(lhs.map(ParserExpr::Identifier)),
					op.map(|x| force_token!(x => Operator)),
					Box::new(rhs),
				).add_span(lhs_span + rhs_span),
			}
		})
	};
}

pub fn stmt(s: ScopeRecursive) -> token_parser_no_span!(ParserStmt : '_) {
	choice((
		set_stmt!(s.clone(), Set),
		set_stmt!(s.clone(), operator Neg),
		set_stmt!(s.clone(), operator Star),
		set_stmt!(s.clone(), operator Plus),
		set_stmt!(s, operator Div),
	))
}
