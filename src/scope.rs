use crate::stmt::Stmt;

pub trait Scope: Sized {
	fn stmts(&self) -> &Vec<Stmt<Self>>;
}
