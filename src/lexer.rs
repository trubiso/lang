use crate::common::span::{Span, SpannedRaw};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use derive_more::Display;
use logos::{Lexer, Logos};
use std::fmt::Display;

fn lex_to_str(lexer: &Lexer<'_, Token>) -> String {
	lexer.slice().trim().to_string()
}

macro_rules! tok_venum {
	($vid:ident { $($match:expr => $to:ident,)* }) => {
		#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
		pub enum $vid { $($to,)* }
		impl Display for $vid {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.write_str(match *self { $(Self::$to => $match,)* })
			}
		}
	}
}

macro_rules! token_keyword {
	($($match:expr => $to:ident,)*) => {
		tok_venum!{Keyword {$($match => $to,)*}}

		impl $crate::common::ident::Ident {
			pub fn is_keyword(&self, keyword: Keyword) -> bool {
				match self.as_keyword() {
					Some(x) => x == keyword,
					None => false,
				}
			}

			pub fn as_keyword(&self) -> Option<Keyword> {
				match self {
					Self::Named(x) => {
						$(if x == $match { return Some(Keyword::$to); })*
						return None;
					}
					#[allow(unreachable_patterns)] // TODO: add more types of Ident and phase this out
					_ => return None,
				}
			}
		}

		impl Token {
			pub fn is_keyword(&self, keyword: Keyword) -> bool {
				match self.as_keyword() {
					Some(x) => x == keyword,
					None => false,
				}
			}

			pub fn as_keyword(&self) -> Option<Keyword> {
				match self {
					Token::Identifier(data) => {
						$(if data == $match { return Some(Keyword::$to); })*
						return None;
					}
					_ => return None,
				}
			}
		}
	}
}

macro_rules! def_token {
	($($vid:ident { $($match:expr => $to:ident,)* })*) => {
		$(tok_venum!{$vid {$($match => $to,)*}})*
		#[derive(Logos, Debug, Display, PartialEq, Eq, Clone, Hash)]
		#[logos(skip r"\s+")] // whitespace
		#[logos(skip r"//[^\n]*")] // line comment
		#[logos(skip r"/\*(?:[^*]|\*[^/])*\*/")] // block comment
		pub enum Token {
			#[regex(r"(?:([0-9][0-9_]*|(?:[0-9][0-9_]*)?\.[0-9][0-9_]*|0b[01][01_]*|0o[0-7][0-7_]*)(i(?:z|8|16|32|64|128)|u(?:z|8|16|32|64|128)?|f(?:16|32|64|128)?)?|(0x[0-9a-fA-F][0-9a-fA-F_]*)(i(?:z|8|16|32|64|128)|u(?:z|8|16|32|64|128)?|p(?:16|32|64|128)?)?)", lex_to_str)]
			NumberLiteral(String),
			#[regex(r"'.'", lex_to_str)]
			CharLiteral(String),
			#[regex(r#""(?:[^"]|\\")*""#, lex_to_str)]
			StringLiteral(String),
			$(
				$(#[token($match, |_| $vid::$to)])*
				$vid($vid),
			)*
			#[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", lex_to_str)]
			Identifier(String),
		}

		token_keyword!{
			"func" => Function,
			"let" => Let,
		}
	};
}

def_token!(
	Operator {
		// unary
		// "?" => Question,
		// "!" => Bang,
		// "&" => Amp,
		// unary/binary
		"-" => Neg,
		"*" => Star,
		// binary
		"+" => Plus,
		"/" => Div,
		// "==" => Eq,
		// "!=" => Ne,
		// "<" => Lt,
		// ">" => Gt,
		// "<=" => Le,
		// ">=" => Ge,
		// "&&" => And,
		// "||" => Or,
	}

	AssignmentOp {
		"=" => Set,
	}

	Punctuation {
		"(" => LParen,
		")" => RParen,
		// "[" => LBracket,
		// "]" => RBracket,
		// "{" => LBrace,
		// "}" => RBrace,
		// "." => Dot,
		// ".." => DotDot,
		// "," => Comma,
		// ":" => Colon,
		// "::" => ColonColon,
		// "->" => Arrow,
		// "=>" => FatArrow,
		";" => Semicolon,
	}
);

type Output = Vec<SpannedRaw<Token>>;
pub fn lex(code: &str, file_id: usize) -> Result<Output, (Output, Vec<Diagnostic<usize>>)> {
	let lex = Token::lexer(code).spanned();
	let tokens = lex
		.map(|(token, range)| (token, Span::new(file_id, range)))
		.collect::<Vec<SpannedRaw<Result<Token, ()>>>>();
	let mut diagnostics = vec![];
	for token in tokens.clone() {
		if token.0.is_err() {
			diagnostics.push(
				Diagnostic::error()
					.with_message("could not parse token")
					.with_labels(vec![Label::primary(token.1.file_id, token.1.range())
						.with_message("invalid token")]),
			)
		}
	}
	let tokens = tokens
		.iter()
		.cloned() // TODO: ewww
		.filter(|(token, _)| token.is_ok())
		.map(|(token, range)| (token.unwrap(), range))
		.collect();
	if !diagnostics.is_empty() {
		Err((tokens, diagnostics))
	} else {
		Ok(tokens)
	}
}
