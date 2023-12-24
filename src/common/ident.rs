use crate::common::{r#type::Type, typed_ident::TypedIdent};

/// An Ident is a name given to a variable, a function or a type.
#[derive(Debug, Clone)]
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
		}
	}
}

impl PartialEq for Ident {
	fn eq(&self, other: &Self) -> bool {
		match self {
			Self::Named(x) => {
				if let Self::Named(y) = other {
					x == y
				} else {
					false
				}
			}
			Self::Qualified(x) => {
				if let Self::Qualified(y) = other {
					x == y
				} else {
					false
				}
			}
			Self::Discarded => matches!(other, Self::Discarded),
		}
	}
}

impl Eq for Ident {}

impl Ident {
	#[must_use]
	pub fn infer_type(&self) -> TypedIdent {
		TypedIdent {
			ty: Type::Inferred,
			ident: self.clone(),
		}
	}

	#[must_use]
	pub fn is_discarded(&self) -> bool {
		matches!(self, Self::Discarded)
	}
}
