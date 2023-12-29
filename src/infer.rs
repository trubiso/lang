// sincere thanks to https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=174ca95a8b938168764846e97d5e9a2c

use self::{engine::Engine, mappings::Mappings, to_info::ToInfo, type_info::TypeInfo};
use crate::{
	common::{
		expr::Expr,
		func::Signature,
		r#type::{BuiltIn, Type},
		span::{Add, Spanned},
		stmt::Stmt,
	},
	hoister::{HoistedExpr, HoistedScope},
	lexer::NumberLiteralType,
};
use lazy_static::lazy_static;
use std::sync::{Mutex, MutexGuard};

pub mod engine;
pub mod mappings;
pub mod to_info;
pub mod type_info;

lazy_static! {
	static ref ENGINE: Mutex<Engine> = Mutex::new(Engine::default());
}

pub fn engine<'a>() -> MutexGuard<'a, Engine> {
	ENGINE.lock().unwrap()
}

impl ToInfo for Spanned<Type> {
	fn to_info(&self, mappings: &mut Mappings) -> Spanned<TypeInfo> {
		match &self.value {
			Type::User(x) => engine()
				.tys
				.get(&mappings.get_named_ty(x.id()).value)
				.expect("??")
				.clone()
				.add_span(self.span),
			Type::BuiltIn(x) => TypeInfo::BuiltIn(x.clone()).add_span(self.span),
			Type::Generic(..) => todo!("(generic type parsing is not even implemented yet)"),
			Type::Inferred => TypeInfo::Unknown.add_span(self.span),
		}
	}
}

impl ToInfo for Spanned<Signature> {
	fn to_info(&self, mappings: &mut Mappings) -> Spanned<TypeInfo> {
		// 1. add all generics as UnknownGeneric for later unification/inference
		let mut generics = Vec::new();
		for generic in &self.value.generics.value {
			let ty = engine()
				.add_ty(TypeInfo::UnknownGeneric(generic.value.id()))
				.add_span(generic.span);
			generics.push(ty);
			mappings.insert_named_ty(generic.value.id(), ty);
		}
		// 2. add all arg types for later unification/inference (NOT the idents)
		let mut args = Vec::new();
		for arg in &self.value.args.value {
			let ty = arg.value.ty.convert_and_add(mappings);
			args.push(ty);
		}
		// 3. create the return ty once more for later unification/inference
		let return_ty = self.value.return_ty.convert_and_add(mappings);

		TypeInfo::FuncSignature {
			return_ty,
			args,
			generics,
		}
		.add_span(self.span)
	}
}

impl ToInfo for Spanned<HoistedExpr> {
	fn to_info(&self, mappings: &mut Mappings) -> Spanned<TypeInfo> {
		match &self.value {
			Expr::NumberLiteral(x) => {
				match x.ty.clone() {
					Some(ty) => {
						if ty.has_bits() {
							// convert to BuiltIn
							TypeInfo::BuiltIn(match ty {
								NumberLiteralType::Integer { bits, signed } => {
									// (unwrap is safe because we cleared that it has bits above)
									BuiltIn::Integer {
										bits: bits.unwrap(),
										signed,
									}
								}
								NumberLiteralType::Float { bits } => BuiltIn::Float {
									// (unwrap is safe because we cleared that it has bits above)
									bits: bits.unwrap(),
								},
							})
						} else {
							TypeInfo::Number(Some(ty))
						}
					}
					None => TypeInfo::Number(None),
				}
				.add_span(self.span)
			}
			Expr::Identifier(x) => {
				TypeInfo::SameAs(*mappings.get_var_ty(x.id())).add_span(self.span)
			}
			Expr::BinaryOp(lhs, _op, rhs) => {
				// TODO: allow ops between different tys with custom return tys
				let lhs = lhs.convert_and_add(mappings);
				let rhs = rhs.convert_and_add(mappings);
				engine().unify(lhs, rhs).add_span(self.span)
			}
			Expr::UnaryOp(_op, value) => {
				// TODO: allow ops to have custom return tys
				value.to_info(mappings)
			}
			// FIXME: why do we need this clone???
			Expr::Scope(inner) => inner.clone().add_span(self.span).to_info(mappings),
			Expr::Call {
				callee: _,
				generics: _,
				args: _,
			} => TypeInfo::Unknown.add_span(self.span), // TODO: function calls
		}
	}
}

impl ToInfo for Spanned<HoistedScope> {
	fn to_info(&self, mappings: &mut Mappings) -> Spanned<TypeInfo> {
		for (ident, var) in &self.value.data.vars {
			// FIXME: this span seems weird
			let ty = var
				.value
				.ty
				.clone()
				.add_span(var.span)
				.convert_and_add(mappings);
			mappings.insert_var_ty(ident.id(), ty);
		}
		for (ident, func) in &self.value.data.funcs {
			let ty = func.convert_and_add(mappings);
			mappings.insert_var_ty(ident.id(), ty);
		}
		let mut has_yielded_or_returned = false;
		let mut return_type = TypeInfo::BuiltIn(BuiltIn::Void).add_span(self.span);
		for stmt in &self.value.stmts {
			if has_yielded_or_returned {
				todo!("warning (unnecessary stmt)");
			}
			match &stmt.value {
				Stmt::Create {
					ty_id,
					mutable: _,
					value,
				} => {
					let var_ty = *mappings.get_var_ty(ty_id.ident().id());
					if let Some(value) = value {
						let value_ty = value.convert_and_add(mappings);
						engine().unify(var_ty, value_ty);
					}
				}
				Stmt::Set { id, value } => {
					let var_ty = *mappings.get_var_ty(id.value.id());
					let value_ty = value.convert_and_add(mappings);
					engine().unify(var_ty, value_ty);
				}
				Stmt::Func {
					id: _,
					signature,
					body,
				} => {
					for generic in &signature.generics.value {
						let ty = engine().add_ty(TypeInfo::Generic(generic.value.id()));
						mappings.insert_named_ty(generic.value.id(), ty.add_span(generic.span));
					}
					for arg in &signature.args.value {
						let ty = arg.value.ty.convert_and_add(mappings);
						mappings.insert_var_ty(arg.ident().id(), ty);
					}
					let return_ty = signature.return_ty.convert_and_add(mappings);
					if let Some(inner) = body {
						let actual_return = inner.convert_and_add(mappings);
						let mut engine = engine();
						let return_ty_ty = engine.tys[&return_ty.value].clone();
						let return_ty_ty = return_ty_ty.display(&engine);
						let actual_return_ty = engine.tys[&actual_return.value].clone();
						let actual_return_ty_display = actual_return_ty.display(&engine);
						engine.unify_custom_error(
							return_ty,
							actual_return,
							"type conflict: incorrect return type",
							&[&format!(
								"return type was declared to be {} but a value of type {} was returned instead{}",
								return_ty_ty,
								actual_return_ty_display,
								if actual_return_ty == TypeInfo::BuiltIn(BuiltIn::Void) {
									" (or no return statement exists)"
								} else {
									""
								}
							)],
						);
					}
				}
				// TODO: do something with is_yield
				Stmt::Return { value, is_yield: _ } => {
					has_yielded_or_returned = true;
					return_type = value.to_info(mappings);
				}
			}
		}
		return_type
	}
}

pub fn infer(scope: &Spanned<HoistedScope>) {
	let mut mappings = Mappings::default();
	scope.to_info(&mut mappings);
	engine().dump();
}
