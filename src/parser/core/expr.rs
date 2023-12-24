use crate::common::expr::Expr;
use crate::common::span::Span;
use crate::lexer::Token;
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

fn atom<'a>(
	e: ExprRecursive<'a>,
	s: ScopeRecursive<'a>,
) -> impl Parser<Token, ParserExpr, Error = Simple<Token, Span>> + 'a {
	choice((
		parened!(e),
		literal_parser!(NumberLiteral),
		// TODO: potentially_qualified_ident
		ident().map(|x| Expr::Identifier(x)),
		braced!(s).map(|x| Expr::Scope(x)),
	))
}

pub fn expr(s: ScopeRecursive) -> token_parser!(ParserExpr : '_) {
	recursive(|e| {
		let neg_parser = unop_parser!(Neg => atom(e.clone(), s.clone()));
		let sd_parser = binop_parser!(Star Div => neg_parser);
		let pn_parser = binop_parser!(Plus Neg => sd_parser);
		pn_parser()
	})
}
