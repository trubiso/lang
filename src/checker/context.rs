use crate::{
	common::{diagnostics::invalid_stmt, span::Spanned},
	parser::types::ParserStmt,
};
use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum Context {
	#[display(fmt = "top level")]
	TopLevel,
	#[display(fmt = "function")]
	Func,
}

macro_rules! check_stmt {
	($($v:ident => $($ctx:ident)*;)*) => {
		pub fn check_stmt(stmt: &Spanned<ParserStmt>, context: Context) {
			match stmt.value {
				$(
					ParserStmt::$v{..} => {
						if $(context != Context::$ctx)&&* {
							invalid_stmt(stmt, context);
						}
					}
				)*
			}
		}
	};
}

check_stmt!(
	Create => Func;
	Set => Func;
	Func => TopLevel Func;
	Return => Func;
);
