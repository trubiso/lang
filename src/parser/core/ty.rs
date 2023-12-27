use super::ident::ident;
use crate::common::{
	r#type::{BuiltInType, Type},
	span::AddSpan,
};
use chumsky::prelude::*;

pub fn ty() -> token_parser!(Type) {
	ident().map_with_span(|x, span| {
		if let Some(ty) = BuiltInType::from_name(&x.value.to_string()) {
			Type::BuiltIn(ty)
		} else if x.value.is_discarded() {
			Type::Inferred
		} else {
			Type::User(x.value)
		}
		.add_span(span)
	})
}
