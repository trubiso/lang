use super::{join::Join, r#type::Type, scope::Scope, span::Spanned};
use crate::{
	common::ident::Ident,
	lexer::{NumberLiteral, Operator},
};

// TODO: dot access (a.b), deref, ref, construct (Struct {a: 3, b: 5}), array
// literal?, tuple??
#[derive(Debug, Clone)]
pub enum Expr<Sc: Scope> {
	NumberLiteral(NumberLiteral),
	Identifier(Ident),
	BinaryOp(
		Box<Spanned<Expr<Sc>>>,
		Spanned<Operator>,
		Box<Spanned<Expr<Sc>>>,
	),
	UnaryOp(Spanned<Operator>, Box<Spanned<Expr<Sc>>>),
	Scope(Sc),
	Call {
		callee: Box<Spanned<Expr<Sc>>>,
		generics: Option<Vec<Spanned<Type>>>,
		args: Vec<Spanned<Expr<Sc>>>,
	},
}

impl<Sc: Scope + std::fmt::Display> std::fmt::Display for Expr<Sc> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Expr::NumberLiteral(num) => f.write_fmt(format_args!("{num}")),
			Expr::Identifier(ident) => f.write_fmt(format_args!("{ident}")),
			Expr::BinaryOp(lhs, op, rhs) => f.write_fmt(format_args!("({lhs} {op} {rhs})")),
			Expr::UnaryOp(op, value) => f.write_fmt(format_args!("({op}{value})")),
			Expr::Scope(scope) => f.write_fmt(format_args!("{scope}")),
			Expr::Call {
				callee,
				generics,
				args,
			} => f.write_fmt(format_args!(
				"({callee}){}{}",
				generics
					.as_ref()
					.map_or(String::new(), |x| (&x).join_comma_wrapped("<", ">")),
				args.join_comma_wrapped("(", ")")
			)),
		}
	}
}
