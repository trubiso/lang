use crate::parser::{
	core::ty_ident::ty_ident,
	types::{ParserStmt, ScopeRecursive},
};
use chumsky::prelude::*;

fn let_var(s: ScopeRecursive) -> token_parser_no_span!(ParserStmt : '_) {
	jkeyword!(Let)
		.ignore_then(assg!(s, optexpr ignore Set))
		.map(|(ident, expr)| ParserStmt::Create {
			ty_id: ident.infer_type(),
			mutable: false,
			value: expr,
		})
}

fn mut_var(s: ScopeRecursive) -> token_parser_no_span!(ParserStmt : '_) {
	jkeyword!(Mut)
		.ignore_then(assg!(s, optexpr ignore Set))
		.map(|(id, expr)| ParserStmt::Create {
			ty_id: id.infer_type(),
			mutable: true,
			value: expr,
		})
}

fn ty_var(s: ScopeRecursive) -> token_parser_no_span!(ParserStmt : '_) {
	jkeyword!(Mut)
		.or_not()
		.then(ty_ident())
		.then(assg!(s, noident ignore Set).or_not())
		.map(|((mutable, ty_id), expr)| ParserStmt::Create {
			ty_id,
			mutable: mutable.is_some(),
			value: expr,
		})
}

pub fn stmt(s: ScopeRecursive) -> token_parser_no_span!(ParserStmt : '_) {
	choice((let_var(s.clone()), mut_var(s.clone()), ty_var(s)))
}
