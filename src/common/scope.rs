use super::{expr::Expr, span::Spanned};
use crate::common::stmt::Stmt;

pub trait Scope: Sized + std::fmt::Display
where
	Expr<Self>: std::fmt::Display,
{
	fn stmts(&self) -> &Vec<Spanned<Stmt<Self>>>;
	fn my_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("{\n")?;
		for stmt in self.stmts() {
			let stmt = format!("{stmt}")
				.split('\n')
				.map(|x| "\t".to_string() + x + "\n")
				.reduce(|acc, b| acc + &b)
				.unwrap_or_default();
			f.write_fmt(format_args!("{stmt}"))?;
		}
		f.write_str("}")?;
		Ok(())
	}
}
