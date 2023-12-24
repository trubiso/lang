use crate::common::{
	expr::Expr, func::FuncSignature, ident::Ident, scope::Scope, typed_ident::TypedIdent,
};

#[derive(Debug, Clone)]
pub enum Stmt<Sc: Scope> {
	Create {
		ty_id: TypedIdent,
		mutable: bool,
		value: Expr,
	},
	Declare {
		ty_id: TypedIdent,
		mutable: bool,
	},
	Set {
		id: Ident,
		value: Expr,
	},
	Func {
		id: Ident,
		signature: FuncSignature,
		body: Option<Sc>,
	},
	Return {
		value: Expr,
	},
}
