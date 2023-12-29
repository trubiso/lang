use super::{count, mappings::Mappings};
use crate::{
	common::{
		diagnostics::{discarded_ident, nonexistent_item},
		ident::Ident,
		r#type::Type,
		span::{AddSpan, Spanned},
		typed_ident::TypedIdent,
	},
	hoister::HoistedScopeData,
};

pub trait ResolveSpecific {
	#[must_use]
	fn resolve_make_new(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self;
	#[must_use]
	fn resolve_must_exist(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self;
}

impl<T: ResolveSpecific> ResolveSpecific for Spanned<T> {
	fn resolve_make_new(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		self.map_ref(|x| x.resolve_make_new(data, mappings))
	}

	fn resolve_must_exist(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		self.map_ref(|x| x.resolve_must_exist(data, mappings))
	}
}

impl<T: ResolveSpecific> ResolveSpecific for Box<T> {
	fn resolve_make_new(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		Box::new(self.as_ref().resolve_make_new(data, mappings))
	}

	fn resolve_must_exist(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		Box::new(self.as_ref().resolve_must_exist(data, mappings))
	}
}

impl<T: ResolveSpecific> ResolveSpecific for Option<T> {
	fn resolve_make_new(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		self.as_ref().map(|x| x.resolve_make_new(data, mappings))
	}

	fn resolve_must_exist(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		self.as_ref().map(|x| x.resolve_must_exist(data, mappings))
	}
}

impl<T: ResolveSpecific> ResolveSpecific for Vec<T> {
	fn resolve_make_new(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		self.iter()
			.map(|x| x.resolve_make_new(data, mappings))
			.collect()
	}

	fn resolve_must_exist(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		self.iter()
			.map(|x| x.resolve_must_exist(data, mappings))
			.collect()
	}
}

fn fail_ident() -> Ident {
	Ident::Resolved(0)
}

impl ResolveSpecific for Spanned<Ident> {
	fn resolve_make_new(&self, _data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		let id = count();
		mappings.insert_var(id, self.value.clone());
		Ident::Resolved(id).add_span(self.span.clone())
	}

	fn resolve_must_exist(&self, _data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		if let Ident::Discarded = self.value {
			discarded_ident(self.span.clone());
			fail_ident()
		} else if let Some(id) = mappings.get_by_ident(&self.value) {
			Ident::Resolved(*id)
		} else {
			nonexistent_item(self.span.clone(), &self.value);
			fail_ident()
		}
		.add_span(self.span.clone())
	}
}

impl ResolveSpecific for Spanned<Type> {
	fn resolve_make_new(&self, _data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		match &self.value {
			Type::User(name) => {
				let id = count();
				mappings.insert_ty(id, name.clone());
				Type::User(Ident::Resolved(id)).add_span(self.span.clone())
			}
			Type::Generic(..) => todo!("(generic type parsing is not even implemented yet)"),
			Type::BuiltIn(..) | Type::Inferred => self.clone(),
		}
	}

	fn resolve_must_exist(&self, _data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		match &self.value {
			Type::User(name) => Type::User(if let Some(id) = mappings.get_by_ident(name) {
				Ident::Resolved(id.clone())
			} else {
				nonexistent_item(self.span.clone(), name);
				fail_ident()
			})
			.add_span(self.span.clone()),
			Type::Generic(..) => todo!("(generic type parsing is not even implemented yet)"),
			Type::BuiltIn(..) | Type::Inferred => self.clone(),
		}
	}
}

impl ResolveSpecific for TypedIdent {
	/// NOTE: assumes ty is `resolve_must_exist`
	fn resolve_make_new(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		Self {
			ty: self.ty.resolve_must_exist(data, mappings),
			ident: self.ident.resolve_make_new(data, mappings),
		}
	}

	fn resolve_must_exist(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		Self {
			ty: self.ty.resolve_must_exist(data, mappings),
			ident: self.ident.resolve_must_exist(data, mappings),
		}
	}
}
