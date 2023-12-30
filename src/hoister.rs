use crate::{
	common::{
		expr::Expr,
		func::Func,
		ident::Ident,
		r#type::Type,
		scope::Scope,
		span::{Add, Spanned},
		stmt::Stmt,
	},
	parser::types::{ParserExpr, ParserScope, ParserStmt},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Var {
	pub ty: Type,
	pub mutable: bool,
}

// NOTE: the Spans that these Spanned<T> hold are the declaration spans
#[derive(Default, Debug, Clone)]
pub struct HoistedScopeData {
	pub vars: HashMap<Ident, Spanned<Var>>,
	pub funcs: HashMap<Ident, Spanned<HoistedFunc>>,
}

impl std::ops::Add for HoistedScopeData {
	type Output = HoistedScopeData;

	fn add(self, mut rhs: Self) -> Self::Output {
		for (k, v) in self.vars {
			rhs.vars.insert(k, v);
		}
		for (k, v) in self.funcs {
			rhs.funcs.insert(k, v);
		}
		rhs
	}
}

#[derive(Clone, Default, Debug)]
pub struct HoistedScope {
	pub stmts: Vec<Spanned<Stmt<Self>>>,
	pub data: HoistedScopeData,
}

impl HoistedScope {
	pub fn add_var(&mut self, ident: Ident, var: Spanned<Var>) {
		self.data.vars.insert(ident, var);
	}

	pub fn add_func(&mut self, ident: Ident, func: Spanned<HoistedFunc>) {
		self.data.funcs.insert(ident, func);
	}
}

impl Scope for HoistedScope {
	fn stmts(&self) -> &Vec<Spanned<Stmt<Self>>> {
		&self.stmts
	}
}

impl std::fmt::Display for HoistedScope {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.my_fmt(f)?;
		f.write_str("\n")?;
		for (id, func) in &self.data.funcs {
			let func = format!("{id} => {func}")
				.split('\n')
				.map(|x| "\t".to_string() + x + "\n")
				.reduce(|acc, b| acc + &b)
				.unwrap_or_default();
			f.write_fmt(format_args!("{func}"))?;
		}
		Ok(())
	}
}

pub type HoistedExpr = Expr<HoistedScope>;
pub type HoistedStmt = Stmt<HoistedScope>;
pub type HoistedFunc = Func<HoistedScope>;

trait Hoist {
	type Output;

	fn hoist(&self) -> Self::Output;
}

trait HoistWithScope {
	type Output;

	fn hoist(self, scope: &mut HoistedScope) -> Self::Output;
}

impl<T: Hoist> Hoist for Spanned<T> {
	type Output = Spanned<T::Output>;

	fn hoist(&self) -> Self::Output {
		self.map_ref(Hoist::hoist)
	}
}

impl<T: Hoist> Hoist for Box<T> {
	type Output = Box<T::Output>;

	fn hoist(&self) -> Self::Output {
		Box::new(self.as_ref().hoist())
	}
}

impl<T: Hoist> Hoist for Option<T> {
	type Output = Option<T::Output>;

	fn hoist(&self) -> Self::Output {
		self.as_ref().map(Hoist::hoist)
	}
}

impl<T: Hoist> Hoist for Vec<T> {
	type Output = Vec<T::Output>;

	fn hoist(&self) -> Self::Output {
		self.iter().map(Hoist::hoist).collect()
	}
}

impl Hoist for ParserExpr {
	type Output = HoistedExpr;

	fn hoist(&self) -> Self::Output {
		match self {
			Expr::NumberLiteral(x) => Expr::NumberLiteral(x.clone()),
			Expr::Identifier(x) => Expr::Identifier(x.clone()),
			Expr::BinaryOp(lhs, op, rhs) => Expr::BinaryOp(lhs.hoist(), *op, rhs.hoist()),
			Expr::UnaryOp(op, value) => Expr::UnaryOp(*op, value.hoist()),
			Expr::Scope(scope) => Expr::Scope(scope.hoist()),
			Expr::Call {
				callee,
				generics,
				args,
			} => Expr::Call {
				callee: callee.hoist(),
				generics: generics.clone(),
				args: args.hoist(),
			},
		}
	}
}

impl HoistWithScope for Spanned<ParserStmt> {
	type Output = Option<Spanned<HoistedStmt>>;

	fn hoist(self, scope: &mut HoistedScope) -> Self::Output {
		match self.value {
			Stmt::Create {
				ty_id,
				mutable,
				value,
			} => {
				let ident = ty_id.ident().clone();
				let ty = ty_id.ty().clone();
				scope.add_var(ident, Var { ty, mutable }.add_span(self.span));
				Some(Stmt::Create {
					ty_id,
					mutable,
					value: value.hoist(),
				})
			}
			Stmt::Set { id, value } => Some(Stmt::Set {
				id,
				value: value.hoist(),
			}),
			Stmt::Func {
				id,
				signature,
				body,
			} => {
				scope.add_func(
					id.value.clone(),
					Func {
						id: id.clone(),
						signature: signature.clone(),
						body: body.hoist(),
					}
					.add_span(id.span),
				); // TODO: add the correct span
				None
			}
			Stmt::Return { value, is_yield } => Some(Stmt::Return {
				value: value.hoist(),
				is_yield,
			}),
		}
		.map(|x| x.add_span(self.span))
	}
}

impl Hoist for ParserScope {
	type Output = HoistedScope;

	fn hoist(&self) -> Self::Output {
		let mut scope = HoistedScope::default();
		for stmt in self.stmts.clone() {
			let stmt = stmt.hoist(&mut scope);
			if let Some(stmt) = stmt {
				scope.stmts.push(stmt);
			}
		}
		scope
	}
}

#[must_use]
pub fn hoist(scope: &ParserScope) -> HoistedScope {
	scope.hoist()
}
