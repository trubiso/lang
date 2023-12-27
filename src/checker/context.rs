use super::add_diagnostic;
use crate::{common::span::Spanned, parser::types::ParserStmt};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use derive_more::Display;

#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum Context {
	#[display(fmt = "top level")]
	TopLevel,
	#[display(fmt = "function")]
	Func,
}

macro_rules! check_stmt {
	($($v:ident => $($ctx:ident)*;)*) => {
		pub fn check_stmt(stmt: &Spanned<ParserStmt>, context: &Context) {
			match stmt.value {
				$(
					ParserStmt::$v{..} => {
						if $(*context != Context::$ctx)&&* {
							add_diagnostic(
								Diagnostic::error()
									.with_message(format!("invalid {} statement in {context} context", stmt.value.variant()))
									.with_labels(vec![Label::primary(stmt.span.file_id, stmt.span.range())]),
							);
						}
					}
				)*
			}
		}
	};
}

check_stmt!(
	Create => Func;
	Set => Func;
	Func => TopLevel Func;
	Return => Func;
);
