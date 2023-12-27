use crate::{
	common::{
		func::FuncSignature,
		ident::{Id, Ident},
		r#type::Type,
		span::{AddSpan, Spanned},
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

#[derive(Clone, Display)]
enum MapRepr {
	#[display(fmt = "variable")]
	Var,
	#[display(fmt = "function")]
	Func,
	#[display(fmt = "type")]
	Type,
}

lazy_static! {
	static ref DIAGNOSTICS: Mutex<Vec<Diagnostic<usize>>> = Mutex::new(vec![]);
	static ref COUNTER: Mutex<Id> = Mutex::new(0);
	static ref MAPPINGS: Mutex<BiMap<Id, Ident>> = Mutex::new(BiMap::default());
	static ref MAPPINGS_REPR: Mutex<HashMap<Id, MapRepr>> = Mutex::new(HashMap::default());
}

fn add_diagnostic(diagnostic: Diagnostic<usize>) {
	DIAGNOSTICS.lock().unwrap().push(diagnostic);
}

fn count() -> Id {
	let mut counter = COUNTER.lock().unwrap();
	counter.add_assign(1);
	return *counter;
}

fn get_or_add_id(id: &Ident) -> Id {
	let mut mappings = MAPPINGS.lock().unwrap();
	match mappings.get_by_right(id) {
		Some(x) => *x,
		None => {
			let new_id = count();
			mappings.insert(new_id, id.clone());
			new_id
		}
	}
}

fn get_repr(id: &Id) -> Option<MapRepr> {
	MAPPINGS_REPR.lock().unwrap().get(id).cloned()
}

fn set_repr(id: &Id, repr: MapRepr) {
	MAPPINGS_REPR.lock().unwrap().insert(*id, repr);
}

trait Resolve {
	fn resolve(&self, data: &HoistedScopeData) -> Self;
}

// TODO: these resolve things are really similar to how hoisting works through
// spanned objects, we should perhaps have a common trait for these kinds of
// containers
impl<T: Resolve> Resolve for Spanned<T> {
	fn resolve(&self, data: &HoistedScopeData) -> Self {
		self.map_ref(|x| x.resolve(data))
	}
}

impl<T: Resolve> Resolve for Box<T> {
	fn resolve(&self, data: &HoistedScopeData) -> Self {
		Box::new(self.as_ref().resolve(data))
	}
}

impl<T: Resolve> Resolve for Option<T> {
	fn resolve(&self, data: &HoistedScopeData) -> Self {
		self.as_ref().map(|x| x.resolve(data))
	}
}

impl<T: Resolve> Resolve for Vec<T> {
	fn resolve(&self, data: &HoistedScopeData) -> Self {
		self.iter().map(|x| x.resolve(data)).collect()
	}
}

impl Resolve for Ident {
	fn resolve(&self, _data: &HoistedScopeData) -> Self {
		Self::Resolved(get_or_add_id(self))
	}
}

impl Resolve for Spanned<Type> {
	fn resolve(&self, data: &HoistedScopeData) -> Self {
		match self.value.clone() {
			Type::User(name) => {
				let id = name.resolve(data).id();
				match get_repr(&id) {
					Some(x) => {
						if !matches!(x, MapRepr::Type) {
							add_diagnostic(
								Diagnostic::error()
									.with_message("type mismatch")
									.with_labels(vec![Label::primary(
										self.span.file_id,
										self.span.range(),
									)
									.with_message(format!("used variable as type"))]),
							)
						}
					}
					None => {
						set_repr(&id, MapRepr::Type);
					}
				}
				Type::User(Ident::Resolved(id)).add_span(self.span.clone())
			}
			Type::Generic(..) => todo!("(generic type parsing is not even implemented yet)"),
			Type::BuiltIn(..) | Type::Inferred => self.clone(),
		}
	}
}

impl Resolve for TypedIdent {
	fn resolve(&self, data: &HoistedScopeData) -> Self {
		Self {
			ty: self.ty.resolve(data),
			ident: self.ident.resolve(data),
		}
	}
}

impl Resolve for HoistedExpr {
	fn resolve(&self, _data: &HoistedScopeData) -> Self {
		todo!("resolve expr")
	}
}

impl Resolve for FuncSignature {
	fn resolve(&self, _data: &HoistedScopeData) -> Self {
		todo!("resolve func signature")
	}
}

impl Resolve for HoistedStmt {
	fn resolve(&self, data: &HoistedScopeData) -> Self {
		match self {
			Stmt::Create {
				ty_id,
				mutable,
				value,
			} => Self::Create {
				ty_id: ty_id.resolve(data),
				mutable: *mutable,
				value: value.resolve(data),
			},
			Self::Set { id, value } => {
				let id = id.resolve(data);
				set_repr(&id.value.id(), MapRepr::Var);
				Self::Set {
					id,
					value: value.resolve(data),
				}
			}
			Self::Func {
				id,
				signature,
				body,
			} => {
				let id = id.resolve(data);
				set_repr(&id.value.id(), MapRepr::Func);
				Self::Func {
					id,
					signature: signature.resolve(data),
					body: body.resolve(data),
				}
			}
			Self::Return { value, is_yield } => Self::Return {
				value: value.resolve(data),
				is_yield: *is_yield,
			},
		}
	}
}

impl Resolve for HoistedScope {
	fn resolve(&self, data: &HoistedScopeData) -> Self {
		Self {
			stmts: self.stmts.iter().map(|x| x.resolve(data)).collect(),
			data: data.clone(),
		}
	}
}

pub fn resolve(scope: HoistedScope, imported_data: HoistedScopeData) -> HoistedScope {
	let data = scope.data.clone() + imported_data;
	scope.resolve(&data)
}
