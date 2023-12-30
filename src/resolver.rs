use self::{
	mappings::{MapRepr, Mappings},
	resolve::Resolve,
	resolve_data::ResolveData,
	resolve_specific::ResolveSpecific,
};
use crate::{
	common::{
		expr::Expr,
		func::Signature,
		ident::{Id, Ident},
		r#type::Type,
		span::{Add, Spanned},
		stmt::Stmt,
		typed_ident::TypedIdent,
	},
	hoister::{HoistedExpr, HoistedFunc, HoistedScope, HoistedScopeData, HoistedStmt},
};
use lazy_static::lazy_static;
use std::{ops::AddAssign, sync::Mutex, collections::HashMap};

pub mod mappings;
pub mod resolve;
pub mod resolve_data;
pub mod resolve_specific;

lazy_static! {
	static ref COUNTER: Mutex<Id> = Mutex::new(0);
}

fn count() -> Id {
	let mut counter = COUNTER.lock().unwrap();
	counter.add_assign(1);
	*counter
}

impl Resolve for Ident {
	fn resolve(&self, _data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		match self {
			Self::Discarded => Self::Discarded,
			_ => Self::Resolved(mappings.get_or_add_id(self)),
		}
	}
}

impl Resolve for Spanned<Type> {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		match self.value.clone() {
			Type::User(name) => {
				let id = name.resolve(data, mappings).id();
				mappings.ensure_repr(id, MapRepr::Type, self.span);
				Type::User(Ident::Resolved(id)).add_span(self.span)
			}
			Type::Generic(..) => todo!("(generic type parsing is not even implemented yet)"),
			Type::BuiltIn(..) | Type::Inferred => self.clone(),
		}
	}
}

impl Resolve for TypedIdent {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		Self {
			ty: self.ty.resolve(data, mappings),
			ident: self.ident.resolve(data, mappings),
		}
	}
}

impl Resolve for Spanned<HoistedExpr> {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		match self.value.clone() {
			Expr::NumberLiteral(x) => Expr::NumberLiteral(x),
			Expr::Identifier(x) => Expr::Identifier(
				x.add_span(self.span)
					.resolve_must_exist(data, mappings)
					.value,
			),
			Expr::BinaryOp(lhs, op, rhs) => {
				Expr::BinaryOp(lhs.resolve(data, mappings), op, rhs.resolve(data, mappings))
			}
			Expr::UnaryOp(op, value) => Expr::UnaryOp(op, value.resolve(data, mappings)),
			Expr::Scope(scope) => Expr::Scope(scope.resolve(data, mappings)),
			Expr::Call {
				callee,
				generics,
				args,
			} => Expr::Call {
				callee: callee.resolve(data, mappings),
				generics: generics.resolve_must_exist(data, mappings),
				args: args.resolve(data, mappings),
			},
		}
		.add_span(self.span)
	}
}

impl Resolve for Signature {
	/// WARNING: adds args and generics to Mappings
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		let mut resolved_generics = Vec::new();
		for generic in &self.generics.value {
			let id = count();
			mappings.insert_ty(id, generic.value.clone());
			resolved_generics.push(Ident::Resolved(id).add_span(generic.span));
		}
		let mut resolved_args = Vec::new();
		for arg in &self.args.value {
			let new_ident = if arg.ident().is_discarded() {
				Ident::Discarded
			} else {
				let id = count();
				mappings.insert_var(id, arg.ident().clone());
				Ident::Resolved(id)
			};
			resolved_args.push(
				TypedIdent {
					ty: arg.value.ty.resolve(data, mappings),
					ident: new_ident.add_span(arg.value.ident.span),
				}
				.add_span(arg.span),
			);
		}
		let resolved_generics = resolved_generics.add_span(self.generics.span);
		let resolved_args = resolved_args.add_span(self.args.span);
		Self {
			attribs: self.attribs.clone(),
			linkage: self.linkage.clone(),
			return_ty: self.return_ty.resolve(data, mappings),
			args: resolved_args,
			generics: resolved_generics,
		}
	}
}

impl Resolve for HoistedFunc {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		// the id will have been created for us already, no need for must exist
		let id = self.id.resolve(data, mappings);
		mappings.ensure_repr(id.value.id(), MapRepr::Func, self.id.span);
		let mut mappings = mappings.clone();
		Self {
			id,
			signature: self.signature.resolve(data, &mut mappings),
			body: self.body.resolve(data, &mut mappings),
		}
	}
}

impl Resolve for HoistedStmt {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		match self {
			Stmt::Create {
				ty_id,
				mutable,
				value,
			} => {
				let ty_id = ty_id.resolve_make_new(data, mappings);
				mappings.ensure_repr(ty_id.ident().id(), MapRepr::Var, ty_id.span);
				Self::Create {
					ty_id,
					mutable: *mutable,
					value: value.resolve(data, mappings),
				}
			}
			Self::Set { id, value } => {
				let id = id.resolve_must_exist(data, mappings);
				mappings.set_repr(&id.value.id(), MapRepr::Var);
				Self::Set {
					id,
					value: value.resolve(data, mappings),
				}
			}
			Self::Func { .. } => unreachable!(),
			Self::Return { value, is_yield } => Self::Return {
				value: value.resolve(data, mappings),
				is_yield: *is_yield,
			},
		}
	}
}

impl Resolve for HoistedScope {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		// duplicate mappings so that changing stuff inside a scope doesn't change stuff
		// outside of it
		let mut mappings = mappings.clone();
		let mut data = data.clone();
		// add hoisted funcs from scope (we only need to add vars in top level, vars in
		// non-top-level contexts are actually inaccurate due to shadowing)
		self.data.make_all_funcs(&mut data, &mut mappings);
		let mut new_scope = Self {
			stmts: self
				.stmts
				.iter()
				.map(|x| x.resolve(&data, &mut mappings))
				.collect(),
			data: HoistedScopeData::default(),
		};
		new_scope.data = HoistedScopeData {
			// FIXME: this is inaccurate in case of shadowing
			vars: self
				.data
				.vars
				.iter()
				.map(|(ident, var)| {
					(
						Ident::Resolved(*mappings.get_by_ident(ident).unwrap()),
						var.clone(),
					)
				})
				.collect(),
			funcs: HashMap::default(),
		};
		new_scope.data.funcs = self
			.data
			.funcs
			.iter()
			.map(|(ident, func)| (ident.resolve(&new_scope.data, &mut mappings), func.resolve(&new_scope.data, &mut mappings)))
			.collect::<HashMap<Ident, Spanned<HoistedFunc>>>();
		new_scope.data.funcs.extend(data.funcs);
		new_scope
	}
}

#[must_use]
pub fn resolve(scope: &HoistedScope, imported_data: &HoistedScopeData) -> HoistedScope {
	scope.resolve(imported_data, &mut Mappings::default())
}
