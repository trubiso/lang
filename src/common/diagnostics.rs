use super::{
	ident::Ident,
	scope::Scope,
	span::{Span, Spanned},
	stmt::Stmt,
};
use crate::{checker::context::Context, resolver::mappings::MapRepr};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
	static ref DIAGNOSTICS: Mutex<Vec<Diagnostic<usize>>> = Mutex::new(vec![]);
}

pub fn add_diagnostic(diagnostic: Diagnostic<usize>) {
	DIAGNOSTICS.lock().unwrap().push(diagnostic);
}

pub fn add_diagnostics(mut diagnostics: &mut Vec<Diagnostic<usize>>) {
	DIAGNOSTICS.lock().unwrap().append(&mut diagnostics);
}

pub fn diagnostics_size() -> usize {
	DIAGNOSTICS.lock().unwrap().len()
}

pub fn own_diagnostics() -> Vec<Diagnostic<usize>> {
	DIAGNOSTICS.lock().unwrap().clone()
}

pub fn type_mismatch(span: Span, used: MapRepr, desired: MapRepr) {
	add_diagnostic(
		Diagnostic::error()
			.with_message("type mismatch")
			.with_labels(vec![Label::primary(span.file_id, span.range())
				.with_message(format!("used {used} as {desired}"))]),
	);
}

pub fn nonexistent_item(span: Span, ident: &Ident) {
	add_diagnostic(
		Diagnostic::error()
			.with_message("referenced nonexistent item")
			.with_labels(vec![Label::primary(span.file_id, span.range())
				.with_message(format!(
					"'{ident}' is not defined in the current scope"
				))]),
	);
}

pub fn discarded_ident(span: Span) {
	add_diagnostic(
		Diagnostic::error()
			.with_message("referenced discarded item where value is required")
			.with_labels(vec![Label::primary(span.file_id, span.range())
				.with_message("the operation you are trying to perform requires a value, but you passed in a discarded item")
			])
	);
}

pub fn invalid_stmt<T: Scope>(stmt: &Spanned<Stmt<T>>, context: Context) {
	add_diagnostic(
		Diagnostic::error()
			.with_message(format!(
				"invalid {} statement in {context} context",
				stmt.value.variant()
			))
			.with_labels(vec![Label::primary(stmt.span.file_id, stmt.span.range())]),
	);
}

pub fn invalid_case<W: std::fmt::Display, F: std::fmt::Display>(span: Span, wanted: W, found: F) {
	add_diagnostic(
		Diagnostic::warning()
			.with_message("wrong case system used")
			.with_labels(vec![Label::primary(span.file_id, span.range())
				.with_message(format!("expected {wanted}, found {found}",))]),
	);
}
