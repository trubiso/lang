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

impl<Sc: Scope + std::fmt::Display> std::fmt::Display for Stmt<Sc>
where
	Expr<Sc>: std::fmt::Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Stmt::Create {
				ty_id,
				mutable,
				value,
			} => f.write_fmt(format_args!(
				"{}{ty_id}{}",
				if *mutable { "mut " } else { "" },
				match value {
					Some(x) => format!(" = {}", x),
					None => String::new(),
				}
			)),
			Stmt::Set { id, value } => f.write_fmt(format_args!("{id} = {value}")),
			Stmt::Func {
				id,
				signature,
				body,
			} => f.write_fmt(format_args!(
				"func {id} [{signature}]{}",
				match body {
					Some(body) => format!(" {body}"),
					None => ";".into(),
				}
			)),
			Stmt::Return { value, is_yield } => f.write_fmt(format_args!(
				"{} {value}",
				if *is_yield { "yield" } else { "return" }
			)),
		}
	}
}
