use crate::common::{
	expr::Expr, func::FuncSignature, ident::Ident, scope::Scope, span::Spanned,
	typed_ident::TypedIdent,
};

#[derive(Debug, Clone)]
pub enum Stmt<Sc: Scope> {
	Create {
		ty_id: Spanned<TypedIdent>,
		mutable: bool,
		value: Option<Spanned<Expr<Sc>>>,
	},
	Set {
		id: Spanned<Ident>,
		value: Spanned<Expr<Sc>>,
	},
	Func {
		id: Spanned<Ident>,
		signature: FuncSignature,
		body: Option<Spanned<Sc>>,
	},
	Return {
		value: Spanned<Expr<Sc>>,
		is_yield: bool,
	},
}

impl<Sc: Scope> Stmt<Sc> {
	pub fn variant(&self) -> &str {
		match self {
			Self::Create { value, .. } => value.as_ref().map_or("declare", |_| "create"),
			Self::Set { .. } => "set",
			Self::Func { .. } => "function",
			Self::Return { is_yield, .. } => {
				if *is_yield {
					"yield"
				} else {
					"return"
				}
			}
		}
	}
}
