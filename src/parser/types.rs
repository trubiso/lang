use crate::{
	lexer::Token,
	span::{Span, Spanned},
};
use chumsky::Stream;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub enum Ident {
	Named(Span, String),
}

pub type CodeStream<'a> = Stream<'a, Token, Span, IntoIter<Spanned<Token>>>;
