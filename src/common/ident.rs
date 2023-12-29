use crate::common::{r#type::Type, typed_ident::TypedIdent};

use super::span::{Add, Spanned};

// FIXME: PartialEq/Eq will break when you have two ways to call something, eg
// "::types::Type" vs directly importing "Type"

pub type Id = usize;

/// An Ident is a name given to a variable, a function or a type.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Ident {
	/// A named Ident is the most basic form of an Ident: it gives a name to an
	/// object in code.
	Named(String),
	/// A qualified Ident works through imports to get a specific object from a
	/// namespace or a different module altogether.
	Qualified(Vec<Ident>),
	/// A discarded Ident is a name given to a variable or type specifically to
	/// mark it as unimportant/inferrable by the compiler.
	Discarded,
	Resolved(Id),
}

impl std::fmt::Display for Ident {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Named(x) => f.write_str(x),
			Self::Qualified(x) => f.write_str(
				&x.iter()
					.map(|x| format!("{x}"))
					.reduce(|acc, b| acc + "::" + &b)
					.unwrap(),
			),
			Self::Discarded => f.write_str("_"),
			Self::Resolved(x) => f.write_fmt(format_args!("@{x}")),
		}
	}
}

impl Ident {
	#[must_use]
	pub fn is_discarded(&self) -> bool {
		matches!(self, Self::Discarded)
	}

	#[must_use]
	pub fn id(&self) -> Id {
		match self {
			Self::Resolved(x) => *x,
			_ => panic!("tried to get id of unresolved ident"),
		}
	}
}

impl Spanned<Ident> {
	#[must_use]
	pub fn infer_type(self) -> Spanned<TypedIdent> {
		let span = self.span;
		TypedIdent {
			ty: Type::Inferred.add_span(span),
			ident: self,
		}
		.add_span(span)
	}
}
