use crate::common::{
	expr::Expr, func::FuncSignature, ident::Ident, scope::Scope, typed_ident::TypedIdent,
};

#[derive(Debug, Clone)]
pub enum Stmt<Sc: Scope> {
	Create {
		ty_id: TypedIdent,
		mutable: bool,
		value: Option<Expr<Sc>>,
	},
	Set {
		id: Ident,
		value: Expr<Sc>,
	},
	Func {
		id: Ident,
		signature: FuncSignature,
		body: Option<Sc>,
	},
	Return {
		value: Expr<Sc>,
		is_yield: bool,
	},
}
