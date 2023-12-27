use crate::{
	common::{
		expr::Expr,
		func::FuncSignature,
		ident::Ident,
		r#type::Type,
		scope::Scope,
		span::{AddSpan, Spanned},
		stmt::Stmt,
	},
	parser::types::{ParserExpr, ParserScope, ParserStmt},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Var {
	pub ty: Type,
	pub mutable: bool,
}

// NOTE: the Spans that these Spanned<T> hold are the declaration spans
#[derive(Default, Debug)]
pub struct HoistedScopeData {
	pub vars: HashMap<Ident, Spanned<Var>>,
	pub funcs: HashMap<Ident, Spanned<FuncSignature>>,
}

#[derive(Default, Debug)]
pub struct HoistedScope {
	pub stmts: Vec<Spanned<Stmt<Self>>>,
	pub data: HoistedScopeData,
}

impl HoistedScope {
	pub fn add_var(&mut self, ident: Ident, var: Spanned<Var>) {
		self.data.vars.insert(ident, var);
	}

	pub fn add_func(&mut self, ident: Ident, func: Spanned<FuncSignature>) {
		self.data.funcs.insert(ident, func);
	}
}

impl Scope for HoistedScope {
	fn stmts(&self) -> &Vec<Spanned<Stmt<Self>>> {
		&self.stmts
	}
}

pub type HoistedExpr = Expr<HoistedScope>;
pub type HoistedStmt = Stmt<HoistedScope>;

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
		self.as_ref().map(|x| x.hoist())
	}
}

impl<T: Hoist> Hoist for Vec<T> {
	type Output = Vec<T::Output>;

	fn hoist(&self) -> Self::Output {
		self.iter().map(|x| x.hoist()).collect()
	}
}

impl Hoist for ParserExpr {
	type Output = HoistedExpr;

	fn hoist(&self) -> Self::Output {
		match self {
			Expr::NumberLiteral(x) => Expr::NumberLiteral(x.clone()),
			Expr::Identifier(x) => Expr::Identifier(x.clone()),
			Expr::BinaryOp(lhs, op, rhs) => Expr::BinaryOp(lhs.hoist(), op.clone(), rhs.hoist()),
			Expr::UnaryOp(op, value) => Expr::UnaryOp(op.clone(), value.hoist()),
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
	type Output = Spanned<HoistedStmt>;

	fn hoist(self, scope: &mut HoistedScope) -> Self::Output {
		match self.value {
			Stmt::Create {
				ty_id,
				mutable,
				value,
			} => {
				let ident = ty_id.ident().clone();
				let ty = ty_id.ty().clone();
				scope.add_var(ident, Var { ty, mutable }.add_span(self.span.clone()));
				Stmt::Create {
					ty_id,
					mutable,
					value: value.hoist(),
				}
			}
			Stmt::Set { id, value } => Stmt::Set {
				id,
				value: value.hoist(),
			},
			Stmt::Func {
				id,
				signature,
				body,
			} => {
				scope.add_func(
					id.value.clone(),
					signature.clone().add_span(self.span.clone()),
				);
				Stmt::Func {
					id,
					signature,
					body: body.hoist(),
				}
			}
			Stmt::Return { value, is_yield } => Stmt::Return {
				value: value.hoist(),
				is_yield,
			},
		}
		.add_span(self.span)
	}
}

impl Hoist for ParserScope {
	type Output = HoistedScope;

	fn hoist(&self) -> Self::Output {
		let mut scope = HoistedScope::default();
		for stmt in self.stmts.clone() {
			let stmt = stmt.hoist(&mut scope);
			scope.stmts.push(stmt);
		}
		scope
	}
}

pub fn hoist(scope: &ParserScope) -> HoistedScope {
	scope.hoist()
}
