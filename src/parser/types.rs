use crate::{
	lexer::Token,
	common::span::{Span, Spanned},
};
use chumsky::Stream;
use std::vec::IntoIter;

pub type CodeStream<'a> = Stream<'a, Token, Span, IntoIter<Spanned<Token>>>;
