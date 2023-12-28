use super::{count, mappings::Mappings};
use crate::{common::ident::Ident, hoister::HoistedScopeData};

pub trait ResolveData {
	fn make_all_funcs(&self, data: &mut HoistedScopeData, mappings: &mut Mappings);
	fn make_all_vars(&self, data: &mut HoistedScopeData, mappings: &mut Mappings);
	// TODO: fn make_all_tys(&self) -> Self;
	fn just_make_all_funcs(&self) -> (HoistedScopeData, Mappings);
	fn just_make_all_funcs_and_vars(&self) -> (HoistedScopeData, Mappings);
}

impl ResolveData for HoistedScopeData {
	fn make_all_funcs(&self, data: &mut HoistedScopeData, mappings: &mut Mappings) {
		for (ident, func) in self.funcs.clone() {
			let id = count();
			mappings.insert_func(id, ident);
			data.funcs.insert(Ident::Resolved(id), func);
		}
	}

	fn make_all_vars(&self, data: &mut HoistedScopeData, mappings: &mut Mappings) {
		for (ident, var) in self.vars.clone() {
			let id = count();
			mappings.insert_var(id, ident);
			data.vars.insert(Ident::Resolved(id), var);
		}
	}

	fn just_make_all_funcs(&self) -> (HoistedScopeData, Mappings) {
		let mut data = HoistedScopeData::default();
		let mut mappings = Mappings::default();
		self.make_all_funcs(&mut data, &mut mappings);
		(data, mappings)
	}

	fn just_make_all_funcs_and_vars(&self) -> (HoistedScopeData, Mappings) {
		let mut data = HoistedScopeData::default();
		let mut mappings = Mappings::default();
		self.make_all_funcs(&mut data, &mut mappings);
		self.make_all_vars(&mut data, &mut mappings);
		(data, mappings)
	}
}
