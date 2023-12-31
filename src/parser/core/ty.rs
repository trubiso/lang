use super::ident::ident;
use crate::common::{
	r#type::{BuiltIn, Type},
	span::Add,
};
use chumsky::prelude::*;

pub fn ty() -> token_parser!(Type) {
	ident().map_with_span(|x, span| {
		if let Some(ty) = BuiltIn::from_name(&x.value.to_string()) {
			Type::BuiltIn(ty)
		} else if x.value.is_discarded() {
			Type::Inferred
		} else {
			Type::User(x.value)
		}
		.add_span(span)
	})
}
