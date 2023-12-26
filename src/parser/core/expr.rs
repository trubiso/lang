use super::ty::ty;
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

// NOTE: can't make a type to encapsulate the return below here, because the
// feature "type alias impl trait" SEGFAULTS!!! ahh, rust's so safe...
fn atom<'a>(
	e: ExprRecursive<'a>,
	s: ScopeRecursive<'a>,
) -> impl Parser<Token, ParserExpr, Error = Simple<Token, Span>> + 'a {
	choice((
		parened!(e),
		literal_parser!(NumberLiteral),
		// TODO: potentially_qualified_ident
		ident().map(Expr::Identifier),
		braced!(s).map(Expr::Scope),
	))
}

fn call<'a>(
	e: ExprRecursive<'a>,
	s: ScopeRecursive<'a>,
) -> impl Parser<Token, ParserExpr, Error = Simple<Token, Span>> + 'a {
	atom(e.clone(), s)
		.then(angled!(ty(),).or_not().then(parened!(e,)).repeated())
		.foldl(|lhs, (generics, args)| Expr::Call {
			callee: Box::new(lhs),
			generics,
			args,
		})
}

pub fn expr(s: ScopeRecursive) -> token_parser!(ParserExpr : '_) {
	recursive(|e| {
		let neg_parser = unop_parser!(Neg => call(e.clone(), s.clone()));
		let sd_parser = binop_parser!(Star Div => neg_parser);
		let pn_parser = binop_parser!(Plus Neg => sd_parser);
		pn_parser()
	})
}
