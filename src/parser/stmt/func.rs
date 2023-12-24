use crate::{
	common::typed_ident::TypedIdent,
	parser::{
		expr::expr,
		ty_ident::{ty_ident, ty_ident_nodiscard},
		types::{ParserFunc, ParserScope, ParserStmt, ScopeRecursive},
	},
};
use chumsky::prelude::*;

fn func_args() -> token_parser!(Vec<TypedIdent>) {
	parened!(ty_ident(),)
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
		.then(func_args())
		.then(func_body(scope))
		.map(|((ty_id, args), body)| ParserStmt::Func {
			id: ty_id.ident,
			func: ParserFunc {
				return_ty: ty_id.ty,
				args,
				generics: Vec::new(), // TODO: generics
				body,
			},
		})
}
