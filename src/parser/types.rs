use crate::{
	lexer::Token,
	span::{Span, Spanned},
};
use chumsky::Stream;
use std::vec::IntoIter;

pub type CodeStream<'a> = Stream<'a, Token, Span, IntoIter<Spanned<Token>>>;
