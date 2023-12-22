use self::{
	ty_ident::{ty_ident, ty_ident_nodiscard},
	types::{ParserScope, ParserStmt, ScopeRecursive},
};
use chumsky::prelude::*;

#[macro_use]
pub mod macros;
pub mod types;

pub mod expr;
pub mod ident;
pub mod ty;
pub mod ty_ident;

fn let_stmt() -> token_parser!(ParserStmt) {
	jkeyword!(Let)
		.ignore_then(assg!(ignore Set))
		.map(|(ident, expr)| ParserStmt::Create {
			ty_id: ident.infer_type(),
			mutable: false,
			value: expr,
		})
}

fn mut_stmt() -> token_parser!(ParserStmt) {
	jkeyword!(Mut)
		.ignore_then(assg!(ignore Set))
		.map(|(id, expr)| ParserStmt::Create {
			ty_id: id.infer_type(),
			mutable: true,
			value: expr,
		})
}

fn create_stmt() -> token_parser!(ParserStmt) {
	jkeyword!(Mut)
		.or_not()
		.then(ty_ident())
		.then(assg!(noident ignore Set))
		.map(|((mutable, ty_id), expr)| ParserStmt::Create {
			ty_id,
			mutable: mutable.is_some(),
			value: expr,
		})
}

fn declare_stmt() -> token_parser!(ParserStmt) {
	jkeyword!(Mut)
		.or_not()
		.then(ty_ident_nodiscard())
		.map(|(mutable, ty_id)| ParserStmt::Declare {
			ty_id,
			mutable: mutable.is_some(),
		})
}

pub fn stmt(_scope: ScopeRecursive) -> token_parser!(ParserStmt) {
	choice((
		let_stmt(),
		mut_stmt(),
		create_stmt(),
		declare_stmt()
	))
}

pub fn bare_scope() -> token_parser!(ParserScope) {
	recursive(|scope| stmt(scope).repeated().map(|stmts| ParserScope { stmts }))
}

pub fn parser() -> token_parser!(ParserScope) {
	bare_scope().then_ignore(end())
}
