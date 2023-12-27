// TODO: (global) write more /// and //! comments

#![warn(clippy::all, clippy::pedantic)]

use chumsky::{Span as _, Stream};
use codespan_reporting::{
	diagnostic::Severity,
	files::SimpleFiles,
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use common::span::Span;
use parser::types::CodeStream;
use std::fs;

use crate::{checker::check, hoister::hoist};

// Compilation steps:
// X - Lexing (into Token)
// X - Parsing (into AST)
//   - CHR
//   - some intermediate step (maybe even done in CHR) where the files being
//     compiled are merged to resolve imports and etc
//   - Type inference
//   - another checking pass that checks, now that we know types, whether
//     mutability is being violated
//   - ??? (perhaps passes to detect ptr dereference outside of unsafe and stuff
//     like that, ensuring code is safe)
//   - Codegen (generate LLVM IR)
//   - Compile (compile LLVM IR down to an actual .o file or executable file)

pub mod checker;
pub mod common;
pub mod hoister;
pub mod lexer;
pub mod parser;

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
		let lexed_iter: CodeStream =
			Stream::from_iter(Span::new(id, code_len..code_len), tokens.into_iter());

		let parsed = match parser::parse(lexed_iter) {
			Ok(scope) => scope,
			Err((scope, diagnostics)) => {
				for diagnostic in diagnostics {
					if !have_errors && diagnostic.severity == Severity::Error {
						have_errors = true;
					}
					all_diagnostics.push(diagnostic);
				}
				scope
			}
		};

		all_diagnostics.append(&mut check(&parsed));

		let hoisted = hoist(&parsed);

		dbg!(hoisted);
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
			if amount == 1 { "" } else { "s" },
			if warnings == 1 { "" } else { "s" },
			amount - warnings,
			if amount - warnings == 1 { "" } else { "s" },
		);

		if have_errors {
			println!("one or more errors present, cannot compile :(");
		}
	}
}
