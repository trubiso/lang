use super::span::{Add, Spanned};
use crate::common::{ident::Ident, join::Join, typed_ident::TypedIdent};

// TODO: figure out Refs and Ptrs, eg:
//     Ref(Box<Self>, bool), // bool = mut
//     Ptr(Box<Self>, bool), // bool = mut
// Ref and Ptr are really similar so they might be merged into a Ptr type

// TODO: figure out optional types (i32?), eg:
//     Optional(Box<Self>),
// Optional would have to interop directly with the std, also would be cool to
// have Result too in some kind of operator

// TODO: function signature types (i32 -> i32), eg:
//     Function(Box<FuncSignature>),
// C-style function signature types also work ( void(int, int) ), but are quite
// bulky and something like i32 -> i32 would be clearer, although it would break
// the type-then-name rule of the language.

// TODO: we will also have tuples and structs in here at some point

/// A `Type` is the representation of a type in code.
#[derive(Debug, Clone)]
pub enum Type {
	/// A `Type` created by the user, identified with an Ident.
	User(Ident),
	/// A `Type` that comes inherently with the language.
	BuiltIn(BuiltIn),
	/// A `Type` with generics filled in, such as Vec<i32>.
	Generic(Box<Self>, Vec<Self>),
	/// A `Type` not specified by the user which the inferring algorithm must
	/// turn into a proper Type.
	Inferred,
}

impl Spanned<Type> {
	#[must_use]
	pub fn add_discarded_ident(self) -> TypedIdent {
		let span = self.span;
		TypedIdent {
			ty: self,
			ident: Ident::Discarded.add_span(span),
		}
	}
}

// TODO: figure out strings and chars

/// A `BuiltIn` is a kind of `Type` that comes with the language itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltIn {
	/// A numeric type represented by i<num> or u<num>, depending on whether
	/// it's signed or unsigned. This number represents the width of the
	/// integer, which can range from 1 to 2^23. The type itself can hold any
	/// non-decimal numeric value, unless it's unsigned, limiting its range to
	/// only positive integers and zero.
	///
	/// If the `bits` field is `None`, this means the width of the integer is
	/// the same as the pointer width for the target architecture.
	Integer { bits: Option<u32>, signed: bool },
	/// A numeric type represented by f<num>, where the number represents the
	/// width of the float, which may only be 16, 32, 64 or 128 bits. The type
	/// can hold any numeric value.
	Float { bits: u8 },
	/// A type whose purpose is to denote the absence of value, represented by
	/// `void`.
	Void,
}

impl BuiltIn {
	#[must_use]
	pub fn is_valid(&self) -> bool {
		match self {
			Self::Integer { bits, signed: _ } => {
				bits.map(|x| x < 2u32.pow(23) && x > 0).unwrap_or(true)
			}
			Self::Float { bits } => *bits == 16 || *bits == 32 || *bits == 64 || *bits == 128,
			Self::Void => true,
		}
	}

	#[must_use]
	pub fn width(&self) -> Option<u32> {
		match self {
			Self::Integer { bits, signed: _ } => *bits,
			Self::Float { bits } => Some(u32::from(*bits)),
			Self::Void => Some(0),
		}
	}

	#[must_use]
	pub fn from_name(name: &str) -> Option<BuiltIn> {
		let result = match name {
			"void" => Some(BuiltIn::Void),
			"bool" => Some(BuiltIn::Integer {
				bits: Some(1),
				signed: false,
			}),
			name => match name[1..].parse::<u32>() {
				Ok(bits) => {
					if name.starts_with('u') {
						Some(BuiltIn::Integer {
							bits: Some(bits),
							signed: false,
						})
					} else if name.starts_with('i') {
						Some(BuiltIn::Integer {
							bits: Some(bits),
							signed: true,
						})
					} else if name.starts_with('f') {
						if bits > 128 {
							None
						} else {
							// we know bits <= 128 < u8::MAX, so no truncation can happen
							#[allow(clippy::cast_possible_truncation)]
							Some(BuiltIn::Float { bits: bits as u8 })
						}
					} else {
						None
					}
				}
				Err(_) => {
					if name[1..] == *"size" {
						if name.starts_with('u') {
							Some(BuiltIn::Integer {
								bits: None,
								signed: false,
							})
						} else if name.starts_with('i') {
							Some(BuiltIn::Integer {
								bits: None,
								signed: true,
							})
						} else {
							unreachable!()
						}
					} else {
						None
					}
				}
			},
		};
		result.filter(BuiltIn::is_valid)
	}
}

impl std::fmt::Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::User(x) => f.write_fmt(format_args!("{x}")),
			Self::BuiltIn(x) => f.write_fmt(format_args!("{x}")),
			Self::Generic(x, g) => {
				f.write_fmt(format_args!("{x}{}", g.join_comma_wrapped("<", ">")))
			}
			Self::Inferred => f.write_str("_"),
		}
	}
}

impl std::fmt::Display for BuiltIn {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Integer { bits, signed } => f.write_fmt(format_args!(
				"{}{}",
				if *signed { "i" } else { "u" },
				bits.map(|x| x.to_string()).unwrap_or("size".to_string())
			)),
			Self::Float { bits } => f.write_fmt(format_args!("f{bits}")),
			Self::Void => f.write_str("void"),
		}
	}
}
