use chumsky::Stream;
use codespan_reporting::{
	diagnostic::Severity,
	files::SimpleFiles,
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
// use parser::types::CodeStream;
use span::Span;
use std::fs;

pub mod expr;
pub mod func;
pub mod ident;
pub mod join;
pub mod lexer;
pub mod parser;
pub mod scope;
pub mod span;
pub mod stmt;
pub mod r#type;
pub mod typed_ident;

fn main() {
	let sources = vec!["code"];
	let mut files = SimpleFiles::new();
	let mut source_ids = Vec::new();
	for file in sources {
		let code = fs::read_to_string(file).unwrap();
		let file_id = files.add(file, code);
		source_ids.push(file_id);
	}

	let mut all_diagnostics = Vec::new();
	let mut have_errors = false;

	for id in source_ids {
		let file = files.get(id).unwrap();
		let code_len = file.source().len();
		let lex_result = lexer::lex(file.source(), id);
		let tokens = match lex_result {
			Ok(tokens) => tokens,
			Err((tokens, diagnostics)) => {
				for diagnostic in diagnostics {
					if !have_errors && diagnostic.severity == Severity::Error {
						have_errors = true;
					}
					all_diagnostics.push(diagnostic);
				}
				tokens
			}
		};
		// let lexed_iter: CodeStream =
		Stream::from_iter(Span::new(id, code_len..code_len), tokens.into_iter());
	}

	if !all_diagnostics.is_empty() {
		// Print errors and/or warnings
		let writer = StandardStream::stderr(ColorChoice::Always);
		let config = term::Config::default();
		let amount = all_diagnostics.len();
		let warnings = all_diagnostics
			.iter()
			.filter(|x| x.severity == Severity::Warning)
			.count();
		for diagnostic in all_diagnostics {
			term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
		}
		println!(
			"{amount} diagnostic{} total ({warnings} warning{}, {} error{})",
			if amount != 1 { "s" } else { "" },
			if warnings != 1 { "s" } else { "" },
			amount - warnings,
			if amount - warnings != 1 { "s" } else { "" },
		);

		if have_errors {
			println!("one or more errors present, cannot compile :(");
		}
	}
}
