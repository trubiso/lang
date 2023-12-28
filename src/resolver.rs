use crate::{
	common::{
		expr::Expr,
		func::FuncSignature,
		ident::{Id, Ident},
		r#type::Type,
		span::{AddSpan, Span, Spanned},
		stmt::Stmt,
		typed_ident::TypedIdent,
	},
	hoister::{HoistedExpr, HoistedScope, HoistedScopeData, HoistedStmt},
};
use bimap::BiMap;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use derive_more::Display;
use lazy_static::lazy_static;
use std::{collections::HashMap, ops::AddAssign, sync::Mutex};

trait ResolveData {
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

#[derive(Clone, Display, PartialEq, Eq)]
pub enum MapRepr {
	#[display(fmt = "variable")]
	Var,
	#[display(fmt = "function")]
	Func,
	#[display(fmt = "type")]
	Type,
}

#[derive(Clone, Default)]
pub struct Mappings {
	pub mappings: BiMap<Id, Ident>,
	pub reprs: HashMap<Id, MapRepr>,
}

lazy_static! {
	static ref DIAGNOSTICS: Mutex<Vec<Diagnostic<usize>>> = Mutex::new(vec![]);
	static ref COUNTER: Mutex<Id> = Mutex::new(0);
}

fn add_diagnostic(diagnostic: Diagnostic<usize>) {
	DIAGNOSTICS.lock().unwrap().push(diagnostic);
}

fn type_mismatch_diagnostic(span: Span, used: MapRepr, desired: MapRepr) {
	add_diagnostic(
		Diagnostic::error()
			.with_message("type mismatch")
			.with_labels(vec![Label::primary(span.file_id, span.range())
				.with_message(format!("used {} as {}", used, desired))]),
	)
}

fn count() -> Id {
	let mut counter = COUNTER.lock().unwrap();
	counter.add_assign(1);
	return *counter;
}

impl Mappings {
	pub fn get_or_add_id(&mut self, id: &Ident) -> Id {
		match self.mappings.get_by_right(id) {
			Some(x) => *x,
			None => {
				let new_id = count();
				self.mappings.insert(new_id, id.clone());
				new_id
			}
		}
	}

	pub fn get_by_id(&self, id: &Id) -> Option<&Ident> {
		self.mappings.get_by_left(id)
	}

	pub fn get_repr(&self, id: &Id) -> Option<MapRepr> {
		self.reprs.get(id).cloned()
	}

	pub fn set_repr(&mut self, id: &Id, repr: MapRepr) {
		self.reprs.insert(*id, repr);
	}

	pub fn insert(&mut self, id: Id, ident: Ident, repr: MapRepr) {
		self.mappings.insert(id, ident);
		self.reprs.insert(id, repr);
	}

	pub fn insert_var(&mut self, id: Id, ident: Ident) {
		self.insert(id, ident, MapRepr::Var);
	}

	pub fn insert_func(&mut self, id: Id, ident: Ident) {
		self.insert(id, ident, MapRepr::Func);
	}

	pub fn insert_ty(&mut self, id: Id, ident: Ident) {
		self.insert(id, ident, MapRepr::Type);
	}

	pub fn ensure_repr(&mut self, id: Id, want: MapRepr, span: Span) {
		match self.get_repr(&id) {
			Some(x) => {
				if x != want {
					type_mismatch_diagnostic(span, x, want);
				}
			}
			None => {
				self.set_repr(&id, want);
			}
		}
	}
}

trait Resolve {
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
				mappings.ensure_repr(id, MapRepr::Type, self.span.clone());
				Type::User(Ident::Resolved(id)).add_span(self.span.clone())
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

// TODO: resolve_must_exist(), sometimes we don't want to create new things,
// e.g. in Expr
impl Resolve for HoistedExpr {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		match self {
			Expr::NumberLiteral(..) => self.clone(),
			Expr::Identifier(x) => Expr::Identifier(x.resolve(data, mappings)),
			Expr::BinaryOp(lhs, op, rhs) => Expr::BinaryOp(
				lhs.resolve(data, mappings),
				op.clone(),
				rhs.resolve(data, mappings),
			),
			Expr::UnaryOp(op, value) => Expr::UnaryOp(op.clone(), value.resolve(data, mappings)),
			Expr::Scope(scope) => Expr::Scope(scope.resolve(data, mappings)),
			Expr::Call {
				callee,
				generics,
				args,
			} => Expr::Call {
				callee: callee.resolve(data, mappings),
				generics: generics.resolve(data, mappings),
				args: args.resolve(data, mappings),
			},
		}
	}
}

impl Resolve for FuncSignature {
	/// WARNING: adds args and generics to Mappings
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		let mut resolved_generics = Vec::new();
		for generic in &self.generics.value {
			let id = count();
			mappings.insert_ty(id, generic.value.clone());
			resolved_generics.push(Ident::Resolved(id).add_span(generic.span.clone()))
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
					ident: new_ident.add_span(arg.value.ident.span.clone()),
				}
				.add_span(arg.span.clone()),
			)
		}
		let resolved_generics = resolved_generics.add_span(self.generics.span.clone());
		let resolved_args = resolved_args.add_span(self.args.span.clone());
		Self {
			attribs: self.attribs.clone(),
			linkage: self.linkage.clone(),
			return_ty: self.return_ty.resolve(data, mappings),
			args: resolved_args,
			generics: resolved_generics,
		}
	}
}

// TODO: shadowing doesn't work!! :( resolve_make_new?
impl Resolve for HoistedStmt {
	fn resolve(&self, data: &HoistedScopeData, mappings: &mut Mappings) -> Self {
		match self {
			Stmt::Create {
				ty_id,
				mutable,
				value,
			} => {
				let ty_id = ty_id.resolve(data, mappings);
				mappings.ensure_repr(ty_id.ident().id(), MapRepr::Var, ty_id.span.clone());
				Self::Create {
					ty_id: ty_id.resolve(data, mappings),
					mutable: *mutable,
					value: value.resolve(data, mappings),
				}
			}
			Self::Set { id, value } => {
				let id = id.resolve(data, mappings);
				mappings.set_repr(&id.value.id(), MapRepr::Var);
				Self::Set {
					id,
					value: value.resolve(data, mappings),
				}
			}
			Self::Func {
				id,
				signature,
				body,
			} => {
				// the id will have been created for us already
				let id = id.resolve(data, mappings);
				mappings.ensure_repr(id.value.id(), MapRepr::Func, id.span.clone());
				let mut mappings = mappings.clone();
				Self::Func {
					id,
					signature: signature.resolve(&data, &mut mappings),
					body: body.resolve(&data, &mut mappings),
				}
			}
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
		Self {
			stmts: self
				.stmts
				.iter()
				.map(|x| x.resolve(&mut data, &mut mappings))
				.collect(),
			data: data.clone(),
		}
	}
}

pub fn resolve(
	scope: HoistedScope,
	imported_data: HoistedScopeData,
) -> Result<HoistedScope, (HoistedScope, Vec<Diagnostic<usize>>)> {
	let resolved = scope.resolve(&imported_data, &mut Mappings::default());
	let diagnostics = DIAGNOSTICS.lock().unwrap();
	if diagnostics.is_empty() {
		Ok(resolved)
	} else {
		Err((resolved, diagnostics.clone()))
	}
}
