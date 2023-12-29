use self::{
	case::{check_ident, Case},
	context::{check_stmt, Context},
};
use crate::{common::stmt::Stmt, parser::types::ParserScope};

pub mod case;
pub mod context;

// Case systems:
// variables/functions -> snake_case
// types/generics -> PascalCase
// pure consts -> UPPER_SNAKE_CASE
// never use camelCase

fn check_inner(scope: &ParserScope, context: Context) {
	for stmt in &scope.stmts {
		check_stmt(stmt, context);
		match &stmt.value {
			Stmt::Create { ty_id, .. } => check_ident(&ty_id.value.ident, Case::SnakeCase),
			Stmt::Set { .. } | Stmt::Return { .. } => {}
			Stmt::Func {
				id,
				signature,
				body,
			} => {
				check_ident(id, Case::SnakeCase);
				for generic in &signature.generics.value {
					check_ident(generic, Case::PascalCase);
				}
				for arg in &signature.args.value {
					check_ident(&arg.value.ident, Case::SnakeCase);
				}
				match body {
					Some(body) => check_inner(&body.value, Context::Func),
					None => {}
				}
			}
		}
	}
}

// TODO: check no yields in funcs
pub fn check(scope: &ParserScope) {
	check_inner(scope, Context::TopLevel);
}
