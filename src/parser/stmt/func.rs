use crate::{
	common::{
		func::{Attribs, Linkage, Signature},
		ident::Ident,
		r#type::Type,
		span::{Add, Spanned},
		typed_ident::TypedIdent,
	},
	parser::{
		core::{
			expr::expr,
			ident::ident,
			ty::ty,
			ty_ident::{self, ty_ident},
		},
		types::{ParserScope, ParserStmt, ScopeRecursive},
	},
};
use chumsky::prelude::*;

fn func_linkage() -> token_parser!(Linkage) {
	span!(jkeyword!(Extern)
		.or_not()
		.map(|x| x.map(|_| Linkage::External).unwrap_or_default()))
}

// TODO: warn about order of attribs (better to write "pure unsafe" than "unsafe
// pure")
fn func_attribs() -> token_parser!(Attribs) {
	macro_rules! func_attribs {
		($($kw:ident => $prop:ident)*) => {
			span!(choice(($(jkeyword!($kw),)*))
			.repeated()
			.validate(|attribs, span: $crate::common::span::Span, emit| {
				let mut final_attribs = Attribs::default();
				for i in 0..attribs.len() {
					match attribs[i].as_keyword().unwrap() {
						$(
							$crate::lexer::Keyword::$kw => {
								if final_attribs.$prop {
									emit(chumsky::error::Simple::custom(span.clone(), "cannot apply attribute twice"));
								}
								final_attribs.$prop = true;
							}
						)*
						_ => unreachable!(),
					}
				}
				final_attribs
			}))
		};
	}
	func_attribs!(
		Pure => is_pure
		Unsafe => is_unsafe
	)
}

fn func_args() -> token_parser!(Vec<Spanned<TypedIdent>>) {
	span!(parened!(choice((
		ty_ident(),
		span!(ty().map(Spanned::<Type>::add_discarded_ident))
	)),))
}

fn func_generics() -> token_parser!(Vec<Spanned<Ident>>) {
	span!(angled!(ident(),).or_not().map(Option::unwrap_or_default))
}

fn func_body(s: ScopeRecursive) -> token_parser!(ParserScope : '_) {
	choice((
		jpunct!(FatArrow)
			.ignore_then(expr(s.clone()))
			.then_ignore(jpunct!(Semicolon))
			.map_with_span(|expr, span| {
				ParserScope {
					stmts: vec![ParserStmt::Return {
						value: expr,
						is_yield: false,
					}
					.add_span(span)],
				}
				.add_span(span)
			}),
		span!(braced!(s)),
	))
}

pub fn stmt(s: ScopeRecursive) -> token_parser_no_span!(ParserStmt : '_) {
	func_linkage()
		.then(ty_ident::nodiscard())
		.then(func_generics())
		.then(func_args())
		.then(func_attribs())
		.then(func_body(s).or_not())
		.map(
			|(((((linkage, ty_id), generics), args), attribs), body)| ParserStmt::Func {
				id: ty_id.value.ident,
				signature: Signature {
					attribs,
					linkage,
					return_ty: ty_id.value.ty,
					args,
					generics,
				},
				body,
			},
		)
}
