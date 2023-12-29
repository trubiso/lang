use super::{
	engine,
	type_info::{TypeId, TypeInfo},
	Mappings,
};
use crate::common::span::{AddSpan, Spanned};

pub trait ToInfo {
	fn to_info(&self, mappings: &mut Mappings) -> Spanned<TypeInfo>;
	fn convert_and_add(&self, mappings: &mut Mappings) -> Spanned<TypeId> {
		let info = self.to_info(mappings);
		engine().add_ty(info.value).add_span(info.span)
	}
}

// TODO: these ToInfo things are really similar to how hoisting works through
// spanned objects, we should perhaps have a common trait for these kinds of
// containers
impl<T: ToInfo> ToInfo for Spanned<T> {
	fn to_info(&self, mappings: &mut Mappings) -> Spanned<TypeInfo> {
		self.map_ref(|x| x.to_info(mappings)).value
	}
}

impl<T: ToInfo> ToInfo for Box<T> {
	fn to_info(&self, mappings: &mut Mappings) -> Spanned<TypeInfo> {
		self.as_ref().to_info(mappings)
	}
}
