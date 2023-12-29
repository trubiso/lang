use super::mappings::Mappings;
use crate::{common::span::Spanned, hoister::HoistedScopeData};

pub trait Resolve {
	#[must_use]
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self;
}

// TODO: these resolve things are really similar to how hoisting works through
// spanned objects, we should perhaps have a common trait for these kinds of
// containers
impl<T: Resolve> Resolve for Spanned<T> {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		self.map_ref(|x| x.resolve(data, mappings))
	}
}

impl<T: Resolve> Resolve for Box<T> {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		Box::new(self.as_ref().resolve(data, mappings))
	}
}

impl<T: Resolve> Resolve for Option<T> {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		self.as_ref().map(|x| x.resolve(data, mappings))
	}
}

impl<T: Resolve> Resolve for Vec<T> {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		self.iter().map(|x| x.resolve(data, mappings)).collect()
	}
}
