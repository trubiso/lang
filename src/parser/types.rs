use crate::{
	common::{
		expr::Expr,
		scope::Scope,
		span::{Span, Spanned, SpannedRaw},
		stmt::Stmt,
	},
	lexer::Token,
};
use chumsky::{error::Simple, recursive::Recursive, Stream};
use std::vec::IntoIter;

pub type CodeStream<'a> = Stream<'a, Token, Span, IntoIter<SpannedRaw<Token>>>;

pub type TokenRecursive<'a, T> = Recursive<'a, Token, T, Simple<Token, Span>>;
pub type ScopeRecursive<'a> = TokenRecursive<'a, ParserScope>;
pub type ExprRecursive<'a> = TokenRecursive<'a, Spanned<ParserExpr>>;

pub type ParserExpr = Expr<ParserScope>;
pub type ParserStmt = Stmt<ParserScope>;

#[derive(Debug, Clone)]
pub struct ParserScope {
	pub stmts: Vec<Spanned<ParserStmt>>,
}

impl Scope for ParserScope {
	fn stmts(&self) -> &Vec<Spanned<Stmt<Self>>> {
		&self.stmts
	}
}

impl std::fmt::Display for ParserScope {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.my_fmt(f)
	}
}
