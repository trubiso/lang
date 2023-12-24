use super::scope::Scope;
use crate::{common::ident::Ident, lexer::Operator};

#[derive(Debug, Clone)]
pub enum Expr<Sc: Scope> {
	NumberLiteral(String),
	Identifier(Ident),
	BinaryOp(Box<Expr<Sc>>, Operator, Box<Expr<Sc>>),
	UnaryOp(Operator, Box<Expr<Sc>>),
	Scope(Sc),
}
