use crate::common::{span::{Span, SpannedRaw}, diagnostics::add_diagnostics};
use chumsky::Span as _;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use derive_more::Display;
use logos::{Lexer, Logos};
use regex::{Match, Regex};
use std::fmt::Display;

fn lex_to_str(lexer: &Lexer<'_, Token>) -> String {
	lexer.slice().trim().to_string()
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
pub enum NumberLiteralKind {
	Decimal,
	Binary,
	Octal,
	Hex,
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
pub enum NumberLiteralType {
	/// outer option = has width specified, inner option: None = pointer width,
	/// Some = specific width
	#[display(fmt = "{}", r#"match bits {
		Some(x) => format!("{}{}", if *signed {"i"} else {"u"}, match x {Some(x) => format!("{x}"), None => "size".into()}),
		None => if *signed { "int" } else { "uint" }.into()
	}"#)]
	Integer {
		bits: Option<Option<u32>>,
		signed: bool,
	},
	/// option: None = no width specified, Some = width specified
	#[display(
		fmt = "f{}",
		r#"match bits { Some(x) => format!("{x}"), None => "loat".into() }"#
	)]
	Float { bits: Option<u8> },
}

impl NumberLiteralType {
	#[must_use]
	pub fn has_bits(&self) -> bool {
		match self {
			Self::Integer { bits, signed: _ } => bits.is_some(),
			Self::Float { bits } => bits.is_some(),
		}
	}
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
#[display(
	fmt = "{value}{}",
	r#"match ty { Some(x) => format!("({x})"), None => String::new() }"#
)]
pub struct NumberLiteral {
	pub value: String,
	pub kind: NumberLiteralKind,
	pub ty: Option<NumberLiteralType>,
}

fn skip_first(s: &str) -> Option<&str> {
	let s = s.chars().next().map(|c| &s[c.len_utf8()..]);
	s.filter(|x| !x.is_empty())
}

// TODO: force numbers with . to have NumberLiteralType::Float, and if another
// type is specified, throw an error
fn parse_number_literal(lexer: &Lexer<'_, Token>) -> NumberLiteral {
	fn parse_ty(ty: Match<'_>) -> NumberLiteralType {
		let ty = ty.as_str();
		let class = ty.chars().next().expect("what");
		match class {
			'i' | 'u' => NumberLiteralType::Integer {
				bits: skip_first(ty).map(|x| {
					if x == "z" {
						None
					} else {
						Some(x.parse().unwrap())
					}
				}),
				signed: class == 'i',
			},
			'f' | 'p' => NumberLiteralType::Float {
				bits: skip_first(ty).map(|x| x.parse().unwrap()),
			},
			_ => unreachable!(),
		}
	}

	let data = lex_to_str(lexer);
	let re = Regex::new(r"^(?:([0-9][0-9_]*|(?:[0-9][0-9_]*)?\.[0-9][0-9_]*|0b[01][01_]*|0o[0-7][0-7_]*)(i(?:z|[0-9]*)|u(?:z|[0-9]*)?|f(?:16|32|64|128)?)?|(0x[0-9a-fA-F][0-9a-fA-F_]*)(i(?:z|[0-9]*)|u(?:z|[0-9]*)?|p(?:16|32|64|128)?)?)$").unwrap();
	let Some(captures) = re.captures(&data) else { panic!("how did you get here?") };
	let literal_part;
	let kind;
	let ty;
	if let Some(hex) = captures.get(3) {
		literal_part = hex.as_str();
		kind = NumberLiteralKind::Hex;
		ty = captures.get(4).map(parse_ty);
	} else {
		literal_part = captures
			.get(1)
			.expect("you broke my regex, how dare you?")
			.as_str();
		kind = if literal_part.starts_with("0b") {
			NumberLiteralKind::Binary
		} else if literal_part.starts_with("0o") {
			NumberLiteralKind::Octal
		} else {
			NumberLiteralKind::Decimal
		};
		ty = captures.get(2).map(parse_ty);
	}
	NumberLiteral {
		value: literal_part.to_string(),
		kind,
		ty,
	}
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
			#[regex(r"(?:([0-9][0-9_]*|(?:[0-9][0-9_]*)?\.[0-9][0-9_]*|0b[01][01_]*|0o[0-7][0-7_]*)(i(?:z|[0-9]*)|u(?:z|[0-9]*)?|f(?:16|32|64|128)?)?|(0x[0-9a-fA-F][0-9a-fA-F_]*)(i(?:z|[0-9]*)|u(?:z|[0-9]*)?|p(?:16|32|64|128)?)?)", parse_number_literal)]
			NumberLiteral(NumberLiteral),
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

pub fn lex(code: &str, file_id: usize) -> Vec<SpannedRaw<Token>> {
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
	add_diagnostics(&mut diagnostics);
	tokens
}
