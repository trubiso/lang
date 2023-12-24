use crate::parser::{ty_ident::ty_ident_nodiscard, types::ParserStmt};
use chumsky::prelude::*;

pub fn declare_stmt() -> token_parser!(ParserStmt) {
	jkeyword!(Mut)
		.or_not()
		.then(ty_ident_nodiscard())
		.map(|(mutable, ty_id)| ParserStmt::Declare {
			ty_id,
			mutable: mutable.is_some(),
		})
}
