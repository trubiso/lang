use super::{scope::Scope, r#type::Type};
use crate::{common::ident::Ident, lexer::Operator};

// TODO: dot access (a.b), deref, ref, construct (Struct {a: 3, b: 5}), array literal?, tuple??
#[derive(Debug, Clone)]
pub enum Expr<Sc: Scope> {
	NumberLiteral(String),
	Identifier(Ident),
	BinaryOp(Box<Expr<Sc>>, Operator, Box<Expr<Sc>>),
	UnaryOp(Operator, Box<Expr<Sc>>),
	Scope(Sc),
	Call { callee: Box<Expr<Sc>>, generics: Option<Vec<Type>>, args: Vec<Expr<Sc>> }
}
