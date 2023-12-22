use crate::{
	common::{
		expr::Expr,
		func::Func,
		scope::Scope,
		span::{Span, Spanned},
		stmt::Stmt,
	},
	lexer::Token,
};
use chumsky::Stream;
use std::vec::IntoIter;

pub type CodeStream<'a> = Stream<'a, Token, Span, IntoIter<Spanned<Token>>>;

pub type ParserExpr = Expr;
pub type ParserFunc = Func<ParserScope>;
pub type ParserStmt = Stmt<ParserScope>;

#[derive(Debug, Clone)]
pub struct ParserScope {
	pub span: Span,
	pub stmts: Vec<ParserStmt>,
}

impl Scope for ParserScope {
	fn stmts(&self) -> &Vec<crate::common::stmt::Stmt<Self>> {
		&self.stmts
	}
}
