use crate::{
	common::{diagnostics::add_diagnostic, span::Spanned},
	infer::type_info::TypeInfo,
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
		use TypeInfo::*;

		match (self.tys[&a.value].clone(), self.tys[&b.value].clone()) {
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
