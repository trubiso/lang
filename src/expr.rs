use crate::{ident::Ident, lexer::Operator};

pub enum Expr {
	NumberLiteral(String),
	Identifier(Ident),
	BinaryOp(Box<Expr>, Operator, Box<Expr>),
	UnaryOp(Operator, Box<Expr>),
}
