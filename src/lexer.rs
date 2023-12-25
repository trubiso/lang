use crate::common::span::{Span, SpannedRaw};
use chumsky::Span as _;
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
			#[must_use]
			pub fn is_keyword(&self, keyword: Keyword) -> bool {
				match self.as_keyword() {
					Some(x) => x == keyword,
					None => false,
				}
			}

			#[must_use]
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
			#[must_use]
			pub fn is_keyword(&self, keyword: Keyword) -> bool {
				match self.as_keyword() {
					Some(x) => x == keyword,
					None => false,
				}
			}

			#[must_use]
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
			/// Represents a number literal. They must follow the following syntax:
			/// * **Literal:** Must be one of the following (in every case, _ can be added to the number everywhere but the very first character):
			///     - a regular integer, e.g. `3_141_592`, `19`, ...
			///     - a float value with an optional left hand side, e.g. `3.141_592`, `.5`, `123_456.789_012`, ...
			///     - a binary value preceded by `0b`, e.g. `0b01101010`, `0b1010_0101`, ...
			///     - an octal value preceded by `0o`, e.g. `0o12345670`, `0o1212_3737`, ...
			///     - a hexadecimal value preceded by `0x`, e.g. `0xdad`, `0xdead_beef`, ...
			/// * **Suffix:** Represents the type of the integer in code. This part of the literal is optional. Must be one of the following:
			///     - `i`, `u`, `f` - these alone represent signed integers, unsigned integers and floats respectively, with no specific bit width
			///     - `i<num>`, `u<num>`, `f<num>` - these represent the above types but with a specific width (floats are restricted to 16, 32, 64 or 128 bits, whereas signed and unsigned integers may range from 1 to 2^23 bits)
			///     - `iz`, `uz` - these `z` suffixes can only go on integers and represent the pointer width for the target architecture (just like `isize` or `usize`)
			/// 
			/// Note: in hexadecimal values, the `f` suffix is changed to `p`, since `f` already represents a hexadecimal value.
			#[regex(r"(?:([0-9][0-9_]*|(?:[0-9][0-9_]*)?\.[0-9][0-9_]*|0b[01][01_]*|0o[0-7][0-7_]*)(i(?:z|[0-9]*)|u(?:z|[0-9]*)?|f(?:16|32|64|128)?)?|(0x[0-9a-fA-F][0-9a-fA-F_]*)(i(?:z|[0-9]*)|u(?:z|[0-9]*)?|p(?:16|32|64|128)?)?)", lex_to_str)]
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
			"return" => Return,
			"yield" => Yield,
			"extern" => Extern,
			"pure" => Pure,
			"unsafe" => Unsafe,
			"let" => Let,
			"mut" => Mut,
			"_" => DontCare,
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
		"<" => Lt,
		">" => Gt,
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
		"{" => LBrace,
		"}" => RBrace,
		// "." => Dot,
		// ".." => DotDot,
		"," => Comma,
		// ":" => Colon,
		// "::" => ColonColon,
		// "->" => Arrow,
		"=>" => FatArrow,
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
			);
		}
	}
	let tokens = tokens
		.iter()
		.cloned() // TODO: ewww
		.filter(|(token, _)| token.is_ok())
		.map(|(token, range)| (token.unwrap(), range))
		.collect();
	if diagnostics.is_empty() {
		Ok(tokens)
	} else {
		Err((tokens, diagnostics))
	}
}
