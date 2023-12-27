use super::{scope::Scope, r#type::Type, span::Spanned};
use crate::{common::ident::Ident, lexer::Operator};

// TODO: dot access (a.b), deref, ref, construct (Struct {a: 3, b: 5}), array literal?, tuple??
#[derive(Debug, Clone)]
pub enum Expr<Sc: Scope> {
	NumberLiteral(String),
	Identifier(Ident),
	BinaryOp(Box<Spanned<Expr<Sc>>>, Spanned<Operator>, Box<Spanned<Expr<Sc>>>),
	UnaryOp(Spanned<Operator>, Box<Spanned<Expr<Sc>>>),
	Scope(Sc),
	Call { callee: Box<Spanned<Expr<Sc>>>, generics: Option<Vec<Spanned<Type>>>, args: Vec<Spanned<Expr<Sc>>> }
}
