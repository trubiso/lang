use super::span::Spanned;
use crate::common::stmt::Stmt;

pub trait Scope: Sized {
	fn stmts(&self) -> &Vec<Spanned<Stmt<Self>>>;
}
