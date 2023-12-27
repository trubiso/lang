use self::context::{check_stmt, Context};
use crate::{common::stmt::Stmt, parser::types::ParserScope};
use codespan_reporting::diagnostic::Diagnostic;
use lazy_static::lazy_static;
use std::sync::Mutex;

// pub mod case;
mod context;

lazy_static! {
	static ref DIAGNOSTICS: Mutex<Vec<Diagnostic<usize>>> = Mutex::new(vec![]);
}

pub fn add_diagnostic(diagnostic: Diagnostic<usize>) {
	DIAGNOSTICS.lock().unwrap().push(diagnostic);
}

fn check_inner(scope: &ParserScope, context: Context) {
	for stmt in &scope.stmts {
		check_stmt(stmt, &context);
		match &stmt.value {
			Stmt::Func { body, .. } => {
				body.as_ref().map(|body| check_inner(&body.value, Context::Func));
			}
			_ => {}
		}
	}
}

pub fn check(scope: &ParserScope) -> Vec<Diagnostic<usize>> {
	check_inner(scope, Context::TopLevel);
	if DIAGNOSTICS.lock().unwrap().is_empty() {
		Vec::new()
	} else {
		DIAGNOSTICS.lock().unwrap().clone()
	}
}
