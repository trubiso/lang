use super::{engine, Mappings, TypeId, TypeInfo};
use crate::common::span::Spanned;

pub trait ToInfo {
	fn to_info(&self, mappings: &mut Mappings) -> TypeInfo;
	fn convert_and_add(&self, mappings: &mut Mappings) -> TypeId {
		let info = self.to_info(mappings);
		engine().add_ty(info)
	}
}

// TODO: these ToInfo things are really similar to how hoisting works through
// spanned objects, we should perhaps have a common trait for these kinds of
// containers
impl<T: ToInfo> ToInfo for Spanned<T> {
	fn to_info(&self, mappings: &mut Mappings) -> TypeInfo {
		self.map_ref(|x| x.to_info(mappings)).value
	}
}

impl<T: ToInfo> ToInfo for Box<T> {
	fn to_info(&self, mappings: &mut Mappings) -> TypeInfo {
		self.as_ref().to_info(mappings)
	}
}
