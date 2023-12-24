use crate::{
	common::{func::FuncSignature, ident::Ident, typed_ident::TypedIdent},
	parser::{
		expr::expr,
		ident::ident,
		ty_ident::ty_ident_nodiscard,
		types::{ParserScope, ParserStmt, ScopeRecursive},
	},
};
use chumsky::prelude::*;

fn func_args() -> token_parser!(Vec<TypedIdent>) {
	parened!(ty_ident_nodiscard(),)
}

fn func_generics() -> token_parser!(Vec<Ident>) {
	angled!(ident(),).or_not().map(|x| x.unwrap_or_default())
}

fn func_body(scope: ScopeRecursive) -> token_parser!(ParserScope : '_) {
	choice((
		jpunct!(FatArrow)
			.ignore_then(expr())
			.then_ignore(jpunct!(Semicolon))
			.map(|expr| ParserScope {
				stmts: vec![ParserStmt::Return { value: expr }],
			}),
		braced!(scope),
	))
}

pub fn func_stmt(scope: ScopeRecursive) -> token_parser!(ParserStmt : '_) {
	ty_ident_nodiscard()
		.then(func_generics())
		.then(func_args())
		.then(func_body(scope).or_not())
		.map(|(((ty_id, generics), args), body)| ParserStmt::Func {
			id: ty_id.ident,
			signature: FuncSignature {
				return_ty: ty_id.ty,
				args,
				generics,
			},
			body,
		})
}
