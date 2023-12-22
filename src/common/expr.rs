use crate::{common::ident::Ident, lexer::Operator};

#[derive(Debug, Clone)]
pub enum Expr {
	NumberLiteral(String),
	Identifier(Ident),
	BinaryOp(Box<Expr>, Operator, Box<Expr>),
	UnaryOp(Operator, Box<Expr>),
}
