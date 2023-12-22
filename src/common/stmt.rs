use crate::common::{ident::Ident, typed_ident::TypedIdent, expr::Expr, func::Func, scope::Scope};

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
		func: Func<Sc>,
	},
	Return {
		value: Expr,
	},
}
