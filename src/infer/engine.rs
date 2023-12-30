use crate::{
	common::{diagnostics::add_diagnostic, r#type, span::Spanned},
	infer::type_info::TypeInfo,
	lexer::NumberLiteralType,
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use itertools::Itertools;
use std::collections::HashMap;

use super::type_info::TypeId;

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
		fn disallowed_implicit_num_cast(
			a: String,
			b: String,
		) -> Result<(), (String, String, String)> {
			Err((
				format!("disallowed implicit cast between numeric types {a} and {b}"),
				a,
				b,
			))
		}

		use TypeInfo::*;
		let c = self.tys[&a.value].clone();
		let d = self.tys[&b.value].clone();

		let mut unify_num_and_builtin = |x: Option<NumberLiteralType>, y: r#type::BuiltIn| {
			let mut accept_specific_type = || {
				self.tys.insert(a.value, TypeInfo::SameAs(b));
				Ok(())
			};
			// we can safely assume both of these are numeric types, we
			// caught voids earlier
			match x {
				// if no numeric type was specified
				None => {
					// the rhs will determine the lhs's numeric type
					accept_specific_type()
				}
				// if a numeric type was specified
				Some(x) => match (x, y) {
					(
						NumberLiteralType::Float { bits },
						r#type::BuiltIn::Float { bits: desired_bits },
					) => match bits {
						None => accept_specific_type(),
						Some(bits) if bits == desired_bits => accept_specific_type(),
						_ => disallowed_implicit_num_cast(c.display(self), d.display(self)),
					},
					(
						NumberLiteralType::Integer { bits, signed },
						r#type::BuiltIn::Integer {
							bits: desired_bits,
							signed: desired_signed,
						},
					) => {
						if signed == desired_signed {
							match bits {
								None => accept_specific_type(),
								Some(bits) if bits == desired_bits => accept_specific_type(),
								_ => disallowed_implicit_num_cast(c.display(self), d.display(self)),
							}
						} else {
							disallowed_implicit_num_cast(c.display(self), d.display(self))
						}
					}
					_ => disallowed_implicit_num_cast(c.display(self), d.display(self)),
				},
			}
		};

		match (c.clone(), d.clone()) {
			(a, b) if a == b => Ok(()),

			(SameAs(a), _) => self.unify_inner(a, b),
			(_, SameAs(b)) => self.unify_inner(a, b),

			(Bottom, _) | (_, Bottom) => Ok(()),

			(Unknown, _) => {
				self.tys.insert(a.value, TypeInfo::SameAs(b));
				Ok(())
			}
			(_, Unknown) => {
				self.tys.insert(b.value, TypeInfo::SameAs(a));
				Ok(())
			}

			// void doesn't unify with anything but void itself
			// don't need to check whether lhs is void, earlier match arm would've caught
			(_, BuiltIn(y)) if y == r#type::BuiltIn::Void => {
				let a = c.display(self);
				let b = d.display(self);
				Err((format!("({a} is a non-void type)"), a, b))
			}
			(BuiltIn(x), _) if x == r#type::BuiltIn::Void => {
				let a = c.display(self);
				let b = d.display(self);
				Err((format!("({b} is a non-void type)"), a, b))
			}

			(BuiltIn(x), BuiltIn(y)) => {
				if x == y {
					Ok(())
				} else {
					let a = c.display(self);
					let b = d.display(self);
					// this is a numeric cast which must be done explicitly (no side can be void, we
					// caught that earlier)
					disallowed_implicit_num_cast(a, b)
				}
			}

			(Number(x), BuiltIn(y)) => unify_num_and_builtin(x, y),
			(BuiltIn(y), Number(x)) => unify_num_and_builtin(x, y),

			(a, b) => Err({
				let a = a.display(self);
				let b = b.display(self);
				(format!("could not unify {a} and {b}"), a, b)
			}),
		}
	}

	/// Returns `TypeInfo::Bottom` if unification failed, otherwise returns
	/// `TypeInfo` corresponding to the unified type of both sides. Allows
	/// changing the name and notes of the error.
	pub fn unify_custom_error(
		&mut self,
		a: Spanned<TypeId>,
		b: Spanned<TypeId>,
		title: &str,
		notes: &[&str],
	) -> TypeInfo {
		let unified = self.unify_inner(a, b);
		if let Err(ref err) = unified {
			let mut notes: Vec<String> = notes.iter().map(|x| (*x).to_string()).collect();
			notes.push(err.0.clone());
			add_diagnostic(
				Diagnostic::error()
					.with_message(title)
					.with_labels(vec![
						Label::primary(a.span.file_id, a.span.range())
							.with_message(format!("({})", err.1)),
						Label::primary(b.span.file_id, b.span.range())
							.with_message(format!("({})", err.2)),
					])
					.with_notes(notes),
			);
		}
		unified.map_or(TypeInfo::Bottom, |_| {
			self.tys.get(&a.value).unwrap().clone()
		})
	}

	/// Returns `TypeInfo::Bottom` if unification failed, otherwise returns
	/// `TypeInfo` corresponding to the unified type of both sides.
	pub fn unify(&mut self, a: Spanned<TypeId>, b: Spanned<TypeId>) -> TypeInfo {
		self.unify_custom_error(a, b, "type conflict", &[])
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
