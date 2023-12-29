use super::Engine;
use crate::{
	common::{ident::Id, join::Join, r#type::BuiltInType},
	lexer::NumberLiteralType,
};

pub type TypeId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeInfo {
	Unknown,
	SameAs(TypeId),
	BuiltIn(BuiltInType),
	/// This type is an incomplete number type, i.e. a number that may be any
	/// signed number, any unsigned number, any float... Fully known number
	/// types do not pass through `TypeInfo::Number` and instead become
	/// `TypeInfo::BuiltIn` immediately. If the inner value is `None`, we know
	/// absolutely nothing about the number type it might be, only that it is,
	/// in fact, a number.
	Number(Option<NumberLiteralType>),
	FuncSignature {
		return_ty: TypeId,
		args: Vec<TypeId>,
		generics: Vec<TypeId>,
	},
	/// This type is passed in as a generic to a function/struct/class. It does
	/// not unify with anything, it simply is a type that we don't know in the
	/// function/struct/class body that varies depending on who calls it.
	Generic(Id),
	/// This type is also passed in as a generic to a function/struct/class.
	/// Instead of simply accepting that it is unknown, we actually have to
	/// resolve this one and it does unify with other types. The difference with
	/// `TypeInfo::Generic` is precisely that this type is used outside of the
	/// function/struct/class body, whereas `TypeInfo::Generic` is used inside
	/// of it and thus does not need unification.
	UnknownGeneric(Id),
	Bottom,
}

impl TypeInfo {
	pub fn display_custom(&self, follow_ref: impl Fn(&usize) -> String + Clone) -> String {
		match self {
			TypeInfo::Unknown => "?".into(),
			TypeInfo::SameAs(x) => follow_ref(x),
			TypeInfo::BuiltIn(x) => format!("{x}"),
			TypeInfo::Number(x) => match x {
				Some(x) => format!("{x}"),
				None => "num".into(),
			},
			TypeInfo::FuncSignature {
				return_ty,
				args,
				generics,
			} => format!(
				"[{}{} -> {}]",
				(&generics
					.iter()
					.map(follow_ref.clone())
					.collect::<Vec<String>>())
					.join_comma_wrapped("<", ">"),
				(&args.iter().map(follow_ref.clone()).collect::<Vec<String>>())
					.join_comma_wrapped("(", ")"),
				follow_ref(return_ty)
			),
			TypeInfo::Generic(x) => format!("[generic @{x}]"),
			TypeInfo::UnknownGeneric(x) => format!("[unresolved generic @{x}]"),
			TypeInfo::Bottom => format!("[!]"),
		}
	}

	pub fn display(&self, engine: &Engine) -> String {
		self.display_custom(|id| engine.tys[id].display(engine))
	}
}
