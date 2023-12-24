use self::{
	stmt::stmt,
	ty_ident::ty_ident,
	types::{CodeStream, ParserScope},
};
use chumsky::{error::SimpleReason, prelude::*};
use codespan_reporting::diagnostic::{Diagnostic, Label};

#[macro_use]
pub mod macros;
pub mod types;

pub mod expr;
pub mod ident;
mod stmt;
pub mod ty;
pub mod ty_ident;

pub fn bare_scope() -> token_parser!(ParserScope) {
	recursive(|scope| stmt(scope).repeated().map(|stmts| ParserScope { stmts }))
}

pub fn parser() -> token_parser!(ParserScope) {
	bare_scope().then_ignore(end())
}

pub fn parse(
	code_stream: CodeStream,
) -> Result<ParserScope, (ParserScope, Vec<Diagnostic<usize>>)> {
	let (parsed, errors) = parser().parse_recovery(code_stream);
	let mut diagnostics = vec![];
	if errors.is_empty() {
		return Ok(parsed.expect("what"));
	}
	// try not to duplicate diagnostics challenge
	let mut add_diagnostic = |diagnostic: Diagnostic<_>| {
		if !diagnostics.contains(&diagnostic) {
			diagnostics.push(diagnostic);
		}
	};
	for err in errors {
		match err.reason() {
			SimpleReason::Unclosed { span, delimiter } => add_diagnostic(
				Diagnostic::error()
					.with_message(format!("unclosed delimiter {delimiter}"))
					.with_labels(vec![
						Label::primary(err.span().file_id, err.span().range())
							.with_message("invalid delimiter"),
						Label::secondary(span.file_id, span.range())
							.with_message("opening delimiter here"),
					]),
			),
			SimpleReason::Unexpected => add_diagnostic(
				Diagnostic::error()
					.with_message("unexpected token")
					.with_labels(vec![Label::primary(err.span().file_id, err.span().range())
						.with_message("this token is invalid")])
					.with_notes(vec![format!(
						"expected one of {}",
						err.expected()
							.map(|x| x
								.as_ref()
								.map(|x| format!("'{}'", x))
								.unwrap_or("[?]".to_string()))
							.reduce(|acc, b| acc + ", " + &b)
							.unwrap_or("".into())
					)]),
			),
			SimpleReason::Custom(label) => add_diagnostic(
				Diagnostic::error()
					.with_message(label)
					.with_labels(vec![Label::primary(err.span().file_id, err.span().range())]),
			),
		}
	}
	Err((
		parsed.unwrap_or_else(|| ParserScope { stmts: Vec::new() }),
		diagnostics,
	))
}
