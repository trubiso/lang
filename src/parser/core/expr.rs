use crate::common::expr::Expr;
use crate::parser::types::ScopeRecursive;
use crate::parser::{
	core::ident::ident,
	types::{ExprRecursive, ParserExpr},
};
use chumsky::prelude::*;

macro_rules! binop_parser {
	($($op:ident)* => $next:ident) => {
		|| $next()
			.then(
				choice(($(jop!($op),)*))
				.then($next()).repeated())
			.foldl(|lhs, (op, rhs)| Expr::BinaryOp(Box::new(lhs), force_token!(op => Operator), Box::new(rhs)))
	};
}

macro_rules! unop_parser {
	($($op:ident)* => $next:ident$(($($x:expr),*))?) => {
		|| choice(($(jop!($op),)*)).repeated()
			.then($next($($($x),*)?))
			.foldr(|op, rhs| Expr::UnaryOp(force_token!(op => Operator), Box::new(rhs)))
	};
}

macro_rules! literal_parser {
	($kind:ident) => {
		filter(|x| matches!(x, $crate::lexer::Token::$kind(_))).map(|x| {
			Expr::$kind(force_token!(x => $kind))
		})
	};
}

fn atom(e: ExprRecursive) -> token_parser!(ParserExpr : '_) {
	choice((
		parened!(e),
		literal_parser!(NumberLiteral),
		// TODO: potentially_qualified_ident
		ident().map(|x| Expr::Identifier(x)),
	))
}

pub fn expr(scope: ScopeRecursive) -> token_parser!(ParserExpr : '_) {
	recursive(|e| {
		let neg_parser = unop_parser!(Neg => atom(e.clone()));
		let sd_parser = binop_parser!(Star Div => neg_parser);
		let pn_parser = binop_parser!(Plus Neg => sd_parser);
		choice((braced!(scope).map(|x| Expr::Scope(x)), pn_parser()))
	})
}
