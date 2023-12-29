// TODO: (global) write more /// and //! comments

// #![warn(clippy::all, clippy::pedantic)]

use crate::{checker::check, common::span::AddSpan, hoister::hoist, resolver::resolve};
use chumsky::{Span as _, Stream};
use codespan_reporting::{
	diagnostic::Severity,
	files::SimpleFiles,
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use common::{diagnostics::own_diagnostics, span::Span};
use std::fs;

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
pub mod infer;
pub mod lexer;
pub mod parser;
pub mod resolver;

fn main() {
	let sources = vec!["code"];
	let mut files = SimpleFiles::new();
	let mut source_ids = Vec::new();
	for file in sources {
		let code = fs::read_to_string(file).unwrap();
		let file_id = files.add(file, code);
		source_ids.push(file_id);
	}

	for id in source_ids {
		let file = files.get(id).unwrap();
		let code_len = file.source().len();
		let tokens = lexer::lex(file.source(), id);
		let lex_iter = Stream::from_iter(Span::new(id, code_len..code_len), tokens.into_iter());
		let parsed = parser::parse(lex_iter);
		check(&parsed);
		let hoisted = hoist(&parsed);
		let resolved = resolve(&hoisted, &hoister::HoistedScopeData::default());

		println!("{resolved}");

		infer::infer(&resolved.add_span(Span::new(id, 0..code_len)));
	}

	let diagnostics = own_diagnostics();
	if !diagnostics.is_empty() {
		// Print errors and/or warnings
		let writer = StandardStream::stderr(ColorChoice::Always);
		let config = term::Config::default();
		let amount = diagnostics.len();
		let warnings = diagnostics
			.iter()
			.filter(|x| x.severity == Severity::Warning)
			.count();
		for diagnostic in &diagnostics {
			term::emit(&mut writer.lock(), &config, &files, diagnostic).unwrap();
		}
		println!(
			"{amount} diagnostic{} total ({warnings} warning{}, {} error{})",
			if amount == 1 { "" } else { "s" },
			if warnings == 1 { "" } else { "s" },
			amount - warnings,
			if amount - warnings == 1 { "" } else { "s" },
		);

		if diagnostics.iter().any(|x| x.severity == Severity::Error) {
			println!("one or more errors present, cannot compile :(");
		}
	}
}
