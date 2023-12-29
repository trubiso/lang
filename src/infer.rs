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
		span::{Span, Spanned},
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
	var_tys: HashMap<Id, TypeId>,
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

	fn unify_inner(&mut self, a: TypeId, b: TypeId) -> Result<(), String> {
		use TypeInfo::*;

		match (self.tys[&a].clone(), self.tys[&b].clone()) {
			(a, b) if a == b => Ok(()),

			(SameAs(a), _) => self.unify_inner(a, b),
			(_, SameAs(b)) => self.unify_inner(a, b),

			(Bottom, _) => Ok(()),
			(_, Bottom) => Ok(()),

			(Unknown, _) => {
				self.tys.insert(a, TypeInfo::SameAs(b));
				Ok(())
			}
			(_, Unknown) => {
				self.tys.insert(b, TypeInfo::SameAs(a));
				Ok(())
			}

			(a, b) => Err(format!(
				"could not unify {} and {}",
				a.display(self),
				b.display(self)
			)),
		}
	}

	/// Returns TypeInfo::Bottom if unification failed, otherwise returns
	/// TypeInfo corresponding to the unified type of both sides. Allows
	/// changing the name and notes of the error.
	pub fn unify_custom_error(
		&mut self,
		a: TypeId,
		b: TypeId,
		span: Span,
		title: &str,
		notes: Vec<&str>,
	) -> TypeInfo {
		let unified = self.unify_inner(a, b);
		if let Err(ref err) = unified {
			let mut notes: Vec<String> = notes.iter().map(|x| x.to_string()).collect();
			notes.push(err.clone());
			add_diagnostic(
				Diagnostic::error()
					.with_message(title)
					.with_labels(vec![Label::primary(span.file_id, span.range())])
					.with_notes(notes),
			)
		}
		unified.map_or(TypeInfo::Bottom, |_| self.tys.get(&a).unwrap().clone())
	}

	/// Returns TypeInfo::Bottom if unification failed, otherwise returns
	/// TypeInfo corresponding to the unified type of both sides.
	pub fn unify(&mut self, a: TypeId, b: TypeId, span: Span) -> TypeInfo {
		self.unify_custom_error(a, b, span, "type conflict", vec![])
	}

	pub fn dump(&self) {
		for k in self.tys.keys().sorted() {
			let v = &self.tys[k];
			println!(
				"@{k} -> {}",
				v.display_custom(|x| format!("[@{x} ({})]", self.tys[x].display(self)))
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

impl ToInfo for Type {
	fn to_info(&self, mappings: &mut Mappings) -> TypeInfo {
		match self {
			Type::User(x) => engine()
				.tys
				.get(&mappings.named_tys.get(&x.id()).expect(&format!(
					"type {} doesn't exist (options: {:?})",
					x.id(),
					mappings.named_tys
				)))
				.expect("??")
				.clone(),
			Type::BuiltIn(x) => TypeInfo::BuiltIn(x.clone()),
			Type::Generic(..) => todo!("(generic type parsing is not even implemented yet)"),
			Type::Inferred => TypeInfo::Unknown,
		}
	}
}

impl ToInfo for FuncSignature {
	fn to_info(&self, mappings: &mut Mappings) -> TypeInfo {
		// 1. add all generics as UnknownGeneric for later unification/inference
		let mut generics = Vec::new();
		for generic in &self.generics.value {
			let ty = engine().add_ty(TypeInfo::UnknownGeneric(generic.value.id()));
			generics.push(ty);
			mappings.named_tys.insert(generic.value.id(), ty);
		}
		// 2. add all arg types for later unification/inference (NOT the idents)
		let mut args = Vec::new();
		for arg in &self.args.value {
			let ty = arg.ty().convert_and_add(mappings);
			args.push(ty);
		}
		// 3. create the return ty once more for later unification/inference
		let return_ty = self.return_ty.convert_and_add(mappings);

		TypeInfo::FuncSignature {
			return_ty,
			args,
			generics,
		}
	}
}

impl ToInfo for Spanned<HoistedExpr> {
	fn to_info(&self, mappings: &mut Mappings) -> TypeInfo {
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
			}
			Expr::Identifier(x) => match mappings.var_tys.get(&x.id()) {
				Some(x) => TypeInfo::SameAs(*x),
				None => {
					println!("we couldn't get {}", x.id());
					panic!("??")
				}
			},
			Expr::BinaryOp(lhs, _op, rhs) => {
				// TODO: allow ops between different tys with custom return tys
				let lhs = lhs.convert_and_add(mappings);
				let rhs = rhs.convert_and_add(mappings);
				engine().unify(lhs, rhs, self.span.clone())
			}
			Expr::UnaryOp(_op, value) => {
				// TODO: allow ops to have custom return tys
				value.to_info(mappings)
			}
			Expr::Scope(inner) => inner.to_info(mappings),
			Expr::Call {
				callee: _,
				generics: _,
				args: _,
			} => TypeInfo::Unknown, // TODO: function calls
		}
	}
}

impl ToInfo for HoistedScope {
	fn to_info(&self, mappings: &mut Mappings) -> TypeInfo {
		for (ident, var) in &self.data.vars {
			let ty = var.value.ty.convert_and_add(mappings);
			mappings.var_tys.insert(ident.id(), ty);
		}
		for (ident, func) in &self.data.funcs {
			let ty = func.value.convert_and_add(mappings);
			mappings.var_tys.insert(ident.id(), ty);
		}
		let mut has_yielded_or_returned = false;
		let mut return_type = TypeInfo::BuiltIn(BuiltInType::Void);
		for stmt in &self.stmts {
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
						engine().unify(var_ty, value_ty, stmt.span.clone());
					}
				}
				Stmt::Set { id, value } => {
					let var_ty = mappings.var_tys.get(&id.value.id()).expect("eugh").clone();
					let value_ty = value.convert_and_add(mappings);
					engine().unify(var_ty, value_ty, stmt.span.clone());
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
						let ty = arg.ty().convert_and_add(mappings);
						mappings.var_tys.insert(arg.ident().id(), ty);
					}
					let return_ty = signature.return_ty.convert_and_add(mappings);
					if let Some(inner) = body {
						let actual_return = inner.convert_and_add(mappings);
						let mut engine = engine();
						let return_ty_ty = engine.tys[&return_ty].clone();
						let return_ty_ty = return_ty_ty.display(&engine);
						let actual_return_ty = engine.tys[&actual_return].clone();
						let actual_return_ty_display = actual_return_ty.display(&engine);
						engine.unify_custom_error(
							return_ty,
							actual_return,
							signature.return_ty.span.clone(),
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

pub fn infer(scope: &HoistedScope) -> Result<(), Vec<Diagnostic<usize>>> {
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
