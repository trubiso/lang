use crate::common::r#type::{BuiltInType, Type};
use chumsky::prelude::*;

use super::ident::ident;

pub fn ty() -> token_parser!(Type) {
	ident().map(|x| {
		if let Some(ty) = BuiltInType::from_name(&x.to_string()) {
			Type::BuiltIn(ty)
		} else if x.is_discarded() {
			Type::Inferred
		} else {
			Type::User(x)
		}
	})
}
