use self::{
	case::{check_case_ident, Case},
	context::{check_stmt, Context},
};
use crate::{common::stmt::Stmt, parser::types::ParserScope};
use codespan_reporting::diagnostic::Diagnostic;
use lazy_static::lazy_static;
use std::sync::Mutex;

pub mod case;
mod context;

lazy_static! {
	static ref DIAGNOSTICS: Mutex<Vec<Diagnostic<usize>>> = Mutex::new(vec![]);
}

pub fn add_diagnostic(diagnostic: Diagnostic<usize>) {
	DIAGNOSTICS.lock().unwrap().push(diagnostic);
}

// Case systems:
// variables/functions -> snake_case
// types/generics -> PascalCase
// pure consts -> UPPER_SNAKE_CASE
// never use camelCase

fn check_inner(scope: &ParserScope, context: Context) {
	for stmt in &scope.stmts {
		check_stmt(stmt, &context);
		match &stmt.value {
			Stmt::Create { ty_id, .. } => check_case_ident(&ty_id.value.ident, Case::SnakeCase),
			Stmt::Set { .. } => {}
			Stmt::Func {
				id,
				signature,
				body,
			} => {
				check_case_ident(id, Case::SnakeCase);
				for generic in &signature.generics.value {
					check_case_ident(generic, Case::PascalCase);
				}
				for arg in &signature.args.value {
					check_case_ident(&arg.value.ident, Case::SnakeCase);
				}
				match body {
					Some(body) => check_inner(&body.value, Context::Func),
					None => {}
				}
			}
			Stmt::Return { .. } => {}
		}
	}
}

// TODO: check no yields in funcs
pub fn check(scope: &ParserScope) -> Vec<Diagnostic<usize>> {
	check_inner(scope, Context::TopLevel);
	if DIAGNOSTICS.lock().unwrap().is_empty() {
		Vec::new()
	} else {
		DIAGNOSTICS.lock().unwrap().clone()
	}
}
