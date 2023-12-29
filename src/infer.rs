// sincere thanks to https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=174ca95a8b938168764846e97d5e9a2c

use self::{
	to_info::ToInfo,
	type_info::{TypeId, TypeInfo},
};
use crate::{
	common::{
		expr::Expr,
		func::FuncSignature,
		ident::Id,
		r#type::{BuiltInType, Type},
		span::{AddSpan, Span, Spanned},
		stmt::Stmt,
	},
	hoister::{HoistedExpr, HoistedScope},
	lexer::NumberLiteralType,
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::{
	collections::HashMap,
	sync::{Mutex, MutexGuard},
};

pub mod to_info;
pub mod type_info;

#[derive(Debug, Default)]
pub struct Mappings {
	named_tys: HashMap<Id, TypeId>,
	var_tys: HashMap<Id, Spanned<TypeId>>,
}

#[derive(Default)]
pub struct Engine {
	id_counter: TypeId,
	pub tys: HashMap<TypeId, TypeInfo>,
}

impl Engine {
	pub fn add_ty(&mut self, info: TypeInfo) -> TypeId {
		self.id_counter += 1;
		self.tys.insert(self.id_counter, info);
		self.id_counter
	}

	fn unify_inner(
		&mut self,
		a: Spanned<TypeId>,
		b: Spanned<TypeId>,
	) -> Result<(), (String, String, String)> {
		use TypeInfo::*;

		match (self.tys[&a.value].clone(), self.tys[&b.value].clone()) {
			(a, b) if a == b => Ok(()),

			(SameAs(a), _) => self.unify_inner(a, b),
			(_, SameAs(b)) => self.unify_inner(a, b),

			(Bottom, _) => Ok(()),
			(_, Bottom) => Ok(()),

			(Unknown, _) => {
				self.tys.insert(a.value, TypeInfo::SameAs(b));
				Ok(())
			}
			(_, Unknown) => {
				self.tys.insert(b.value, TypeInfo::SameAs(a));
				Ok(())
			}

			(a, b) => Err({
				let a = a.display(self);
				let b = b.display(self);
				(format!("could not unify {} and {}", a, b), a, b)
			}),
		}
	}

	/// Returns TypeInfo::Bottom if unification failed, otherwise returns
	/// TypeInfo corresponding to the unified type of both sides. Allows
	/// changing the name and notes of the error.
	pub fn unify_custom_error(
		&mut self,
		a: Spanned<TypeId>,
		b: Spanned<TypeId>,
		title: &str,
		notes: Vec<&str>,
	) -> TypeInfo {
		let unified = self.unify_inner(a.clone(), b.clone());
		if let Err(ref err) = unified {
			let mut notes: Vec<String> = notes.iter().map(|x| x.to_string()).collect();
			notes.push(err.0.clone());
			add_diagnostic(
				Diagnostic::error()
					.with_message(title)
					.with_labels(vec![
						Label::primary(a.span.file_id, a.span.range()).with_message(format!("({})", err.1)),
						Label::primary(b.span.file_id, b.span.range()).with_message(format!("({})", err.2)),
					])
					.with_notes(notes),
			)
		}
		unified.map_or(TypeInfo::Bottom, |_| {
			self.tys.get(&a.value).unwrap().clone()
		})
	}

	/// Returns TypeInfo::Bottom if unification failed, otherwise returns
	/// TypeInfo corresponding to the unified type of both sides.
	pub fn unify(&mut self, a: Spanned<TypeId>, b: Spanned<TypeId>) -> TypeInfo {
		self.unify_custom_error(a, b, "type conflict", vec![])
	}

	pub fn dump(&self) {
		for k in self.tys.keys().sorted() {
			let v = &self.tys[k];
			println!(
				"@{k} -> {}",
				v.display_custom(|x| format!("[@{x} ({})]", self.tys[&x.value].display(self)))
			);
		}
	}
}

lazy_static! {
	static ref DIAGNOSTICS: Mutex<Vec<Diagnostic<usize>>> = Mutex::new(vec![]);
	static ref ENGINE: Mutex<Engine> = Mutex::new(Engine::default());
}

pub fn engine<'a>() -> MutexGuard<'a, Engine> {
	ENGINE.lock().unwrap()
}

fn add_diagnostic(diagnostic: Diagnostic<usize>) {
	DIAGNOSTICS.lock().unwrap().push(diagnostic);
}

impl ToInfo for Spanned<Type> {
	fn to_info(&self, mappings: &mut Mappings) -> Spanned<TypeInfo> {
		match &self.value {
			Type::User(x) => engine()
				.tys
				.get(&mappings.named_tys.get(&x.id()).expect(&format!(
					"type {} doesn't exist (options: {:?})",
					x.id(),
					mappings.named_tys
				)))
				.expect("??")
				.clone()
				.add_span(self.span.clone()),
			Type::BuiltIn(x) => TypeInfo::BuiltIn(x.clone()).add_span(self.span.clone()),
			Type::Generic(..) => todo!("(generic type parsing is not even implemented yet)"),
			Type::Inferred => TypeInfo::Unknown.add_span(self.span.clone()),
		}
	}
}

impl ToInfo for Spanned<FuncSignature> {
	fn to_info(&self, mappings: &mut Mappings) -> Spanned<TypeInfo> {
		// 1. add all generics as UnknownGeneric for later unification/inference
		let mut generics = Vec::new();
		for generic in &self.value.generics.value {
			let ty = engine().add_ty(TypeInfo::UnknownGeneric(generic.value.id()));
			generics.push(ty.add_span(generic.span.clone()));
			mappings.named_tys.insert(generic.value.id(), ty);
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
		.add_span(self.span.clone())
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
									BuiltInType::Integer {
										bits: bits.unwrap(),
										signed,
									}
								}
								NumberLiteralType::Float { bits } => BuiltInType::Float {
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
				.add_span(self.span.clone())
			}
			Expr::Identifier(x) => match mappings.var_tys.get(&x.id()) {
				Some(x) => TypeInfo::SameAs(x.clone()).add_span(self.span.clone()),
				None => {
					println!("we couldn't get {}", x.id());
					panic!("??")
				}
			},
			Expr::BinaryOp(lhs, _op, rhs) => {
				// TODO: allow ops between different tys with custom return tys
				let lhs = lhs.convert_and_add(mappings);
				let rhs = rhs.convert_and_add(mappings);
				engine().unify(lhs, rhs).add_span(self.span.clone())
			}
			Expr::UnaryOp(_op, value) => {
				// TODO: allow ops to have custom return tys
				value.to_info(mappings)
			}
			// FIXME: why do we need this clone???
			Expr::Scope(inner) => inner.clone().add_span(self.span.clone()).to_info(mappings),
			Expr::Call {
				callee: _,
				generics: _,
				args: _,
			} => TypeInfo::Unknown.add_span(self.span.clone()), // TODO: function calls
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
				.add_span(var.span.clone())
				.convert_and_add(mappings);
			mappings.var_tys.insert(ident.id(), ty);
		}
		for (ident, func) in &self.value.data.funcs {
			let ty = func.convert_and_add(mappings);
			mappings.var_tys.insert(ident.id(), ty);
		}
		let mut has_yielded_or_returned = false;
		let mut return_type = TypeInfo::BuiltIn(BuiltInType::Void).add_span(self.span.clone());
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
					let var_ty = mappings
						.var_tys
						.get(&ty_id.ident().id())
						.expect(&format!(
							"couldn't get {} from {:?}",
							ty_id.ident().id(),
							mappings
						))
						.clone();
					if let Some(value) = value {
						let value_ty = value.convert_and_add(mappings);
						engine().unify(var_ty, value_ty);
					}
				}
				Stmt::Set { id, value } => {
					let var_ty = mappings.var_tys.get(&id.value.id()).expect("eugh").clone();
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
						mappings.named_tys.insert(generic.value.id(), ty);
					}
					for arg in &signature.args.value {
						let ty = arg.value.ty.convert_and_add(mappings);
						mappings.var_tys.insert(arg.ident().id(), ty);
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
							vec![&format!(
								"return type was declared to be {} but a value of type {} was returned instead{}",
								return_ty_ty,
								actual_return_ty_display,
								if actual_return_ty == TypeInfo::BuiltIn(BuiltInType::Void) {
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

pub fn infer(scope: &Spanned<HoistedScope>) -> Result<(), Vec<Diagnostic<usize>>> {
	let mut mappings = Mappings::default();
	scope.to_info(&mut mappings);
	engine().dump();
	let diagnostics = DIAGNOSTICS.lock().unwrap();
	if diagnostics.is_empty() {
		Ok(())
	} else {
		Err(diagnostics.clone())
	}
}
