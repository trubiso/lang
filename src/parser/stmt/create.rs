use crate::parser::{core::ty_ident::ty_ident, types::ParserStmt};
use chumsky::prelude::*;

fn let_var() -> token_parser!(ParserStmt) {
	jkeyword!(Let)
		.ignore_then(assg!(optexpr ignore Set))
		.map(|(ident, expr)| ParserStmt::Create {
			ty_id: ident.infer_type(),
			mutable: false,
			value: expr,
		})
}

fn mut_var() -> token_parser!(ParserStmt) {
	jkeyword!(Mut)
		.ignore_then(assg!(optexpr ignore Set))
		.map(|(id, expr)| ParserStmt::Create {
			ty_id: id.infer_type(),
			mutable: true,
			value: expr,
		})
}

fn ty_var() -> token_parser!(ParserStmt) {
	jkeyword!(Mut)
		.or_not()
		.then(ty_ident())
		.then(assg!(noident ignore Set).or_not())
		.map(|((mutable, ty_id), expr)| ParserStmt::Create {
			ty_id,
			mutable: mutable.is_some(),
			value: expr,
		})
}

pub fn create_stmt() -> token_parser!(ParserStmt) {
	choice((let_var(), mut_var(), ty_var()))
}
