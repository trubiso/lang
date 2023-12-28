use super::ident::ident_nospan;
use super::ty::ty;
use crate::common::expr::Expr;
use crate::common::span::{AddSpan, Span, Spanned};
use crate::lexer::Token;
use crate::parser::types::ScopeRecursive;
use crate::parser::types::{ExprRecursive, ParserExpr};
use chumsky::prelude::*;

macro_rules! binop_parser {
	($($op:ident)* => $next:ident) => {
		|| $next()
			.then(
				choice(($(span!(jop!($op)),)*))
				.then($next()).repeated())
			.foldl(|lhs, (op, rhs)| {
				let span = lhs.span.clone() + rhs.span.clone();
				Expr::BinaryOp(Box::new(lhs), op.map(|x| force_token!(x => Operator)), Box::new(rhs)).add_span(span)
			})
	};
}

macro_rules! unop_parser {
	($($op:ident)* => $next:ident$(($($x:expr),*))?) => {
		|| choice(($(span!(jop!($op)),)*)).repeated()
			.then($next($($($x),*)?))
			.foldr(|op, rhs| {
				let span = op.span.clone() + rhs.span.clone();
				Expr::UnaryOp(op.map(|x| force_token!(x => Operator)), Box::new(rhs)).add_span(span)
			})
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
) -> impl Parser<Token, Spanned<ParserExpr>, Error = Simple<Token, Span>> + 'a {
	choice((
		parened!(e),
		span!(literal_parser!(NumberLiteral)),
		// TODO: potentially_qualified_ident
		span!(ident_nospan().map(Expr::Identifier)),
		span!(braced!(s).map(Expr::Scope)),
	))
}

fn call<'a>(
	e: ExprRecursive<'a>,
	s: ScopeRecursive<'a>,
) -> impl Parser<Token, Spanned<ParserExpr>, Error = Simple<Token, Span>> + 'a {
	atom(e.clone(), s)
		.then(angled!(ty(),).or_not().then(parened!(e,)).repeated())
		.foldl(|lhs, (generics, args)| {
			let lhs_span = lhs.span.clone();
			let last_span = args.last().map_or(lhs_span.clone(), |x| x.span.clone());
			Expr::Call {
				callee: Box::new(lhs),
				generics,
				args,
			}
			.add_span(lhs_span + last_span)
		})
}

/// Atoms:
/// - `(<expr>)`
/// - `<number literal>`
/// - `<ident>` (TODO: should be potentially qualified)
/// - `<scope>` (ideally with `yield` stmt)
///
/// Parses:
/// - addition/subtraction (`<expr> +|- <expr>`)
/// - multiplication/division (`<expr> *|/ <expr>`)
/// - negation (`-<expr>`)
/// - function calls (`<expr><<ty>, ...>(<expr>, ...)`)
///
/// Want (basic):
/// - logical operators (`<expr> ||, && <expr>`)
/// - ord/eq operators (`<expr> ==, !=, <, >, <=, >= <expr>`)
/// - ref (`&<expr>`)
/// - deref (`*<expr>`)
/// - dot (`<expr>.<ident>`)
/// - construct (`<ty> { <ident>: <expr>, <ident>: <expr> }`)
///
/// Want (sugar):
/// - deref dot (`<expr>*.<ident>` (`== (*<expr>.<ident>)`))
/// - curry (`<expr>-><ident>()` (`== <ident>(<expr>)`),
///   `<expr1>-><ident>(<expr2>, ...)` (`== <ident>(<expr1>, <expr2>, ...)`))
///
/// Want (thinking about it):
/// - array literals? (`[<expr>, ...]`)
/// - tuples? (`(<expr>, ...)`)
/// - set? (`<ident> = <expr>`, same as doing this outside expr, returning the
///   rhs)
/// - ub producers? (`<ident>++, ++<ident>, <ident>--, --<ident>`, try not to ub
///   :D)
/// - ignore lhs? (`<expr1>, <expr2>, <expr3>` (`== <expr3>`); but do it with
///   another symbol)
/// - bitwise ops? (`~<expr>`, `<expr> ^ <expr>`, ...; maybe should do with
///   functions)
///
/// Want (more scope-y things):
/// - match/switch
/// - if (`if (<expr>) <expr> [else <expr>]`, wouldn't even need to make it a
///   statement, but it would be cool to also allow for `if <expr> { ... } else
///   <expr>`)
/// - for? while? how would these return, being loops
pub fn expr(s: ScopeRecursive) -> token_parser!(ParserExpr : '_) {
	recursive(|e| {
		let neg_parser = unop_parser!(Neg => call(e.clone(), s.clone()));
		let sd_parser = binop_parser!(Star Div => neg_parser);
		let pn_parser = binop_parser!(Plus Neg => sd_parser);
		pn_parser()
	})
}
