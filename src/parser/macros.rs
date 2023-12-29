#[macro_export]
macro_rules! token_parser_no_span {
	($ty:ty) => {
		impl chumsky::Parser<$crate::lexer::Token, $ty, Error = chumsky::error::Simple<$crate::lexer::Token, $crate::common::span::Span>>
	};

	($ty:ty : '_) => {
		impl chumsky::Parser<$crate::lexer::Token, $ty, Error = chumsky::error::Simple<$crate::lexer::Token, $crate::common::span::Span>> + '_
	};
}

#[macro_export]
macro_rules! token_parser {
	($ty:ty) => {
		token_parser_no_span!($crate::common::span::Spanned<$ty>)
	};

	($ty:ty : '_) => {
		token_parser_no_span!($crate::common::span::Spanned<$ty> : '_)
	};
}

#[macro_export]
macro_rules! ident {
	($span:expr, $str:expr) => {
		$crate::common::ident::Ident::Named($span, $str.to_string())
	};
}

#[macro_export]
macro_rules! span {
	($x:expr) => {
		$x.map_with_span(|x, s| $crate::common::span::Add::add_span(x, s))
	};
}

macro_rules! token_gen {
	($name:ident, $jname:ident => $ident:ident) => {
		#[macro_export]
		macro_rules! $name {
			($var:ident) => {
				$crate::lexer::Token::$ident($crate::lexer::$ident::$var)
			};
		}
		#[macro_export]
		macro_rules! $jname {
			($var:ident) => {
				just($name!($var))
			};
		}
	};
}

#[macro_export]
macro_rules! jkeyword {
	($var:ident) => {
		filter(|token: &$crate::lexer::Token| token.is_keyword($crate::lexer::Keyword::$var))
	};
}

token_gen!(punct, jpunct => Punctuation);
token_gen!(_assg_op, jassg_op => AssignmentOp);
token_gen!(op, jop => Operator);

macro_rules! delim_gen {
	($name:ident => ($mac:ident, $wmac:ident) + $l:ident, $r:ident) => {
		#[macro_export]
		macro_rules! $name {
			($arg:expr) => {
				$arg.delimited_by($wmac!($l), $wmac!($r))
			};
			($arg:expr, $sep:ident) => {
				$name!($arg.separated_by(jpunct!($sep)).allow_trailing())
			};
			($arg:expr,) => {
				$name!($arg, Comma)
			};
		}
	};
	($name:ident => $l:ident, $r:ident) => {
		delim_gen!($name => (punct, jpunct) + $l, $r);
	};
}

delim_gen!(parened => LParen, RParen);
delim_gen!(braced => LBrace, RBrace);
delim_gen!(bracketed => LBracket, RBracket);
delim_gen!(angled => (op, jop) + Lt, Gt);

#[macro_export]
macro_rules! builtin {
	($span:expr, $var:ident) => {
		Type::Builtin($span, BuiltinType::$var)
	};
}

#[macro_export]
macro_rules! force_token {
	($value:expr => Identifier) => {
		match $value {
			$crate::lexer::Token::Identifier(x) => $crate::common::ident::Ident::Named(x),
			_ => unreachable!(),
		}
	};
	($value:expr => $kind:ident) => {
		match $value {
			$crate::lexer::Token::$kind(x) => x,
			_ => unreachable!(),
		}
	};
	($value:expr => $kind:ident) => {
		force_token!($value => $kind)
	};
}

#[macro_export]
macro_rules! assg {
	($s:expr, $ident:ident) => {
		$crate::parser::core::ident::ident()
			.then(jassg_op!($ident))
			.then($crate::parser::core::expr::expr($s))
	};
	($s:expr, ignore $ident:ident) => {
		$crate::parser::core::ident::ident()
			.then_ignore(jassg_op!($ident))
			.then($crate::parser::core::expr::expr($s))
	};
	($s:expr, noident $ident:ident) => {
		jassg_op!($ident).then($crate::parser::core::expr::expr($s))
	};
	($s:expr, noident ignore $ident:ident) => {
		jassg_op!($ident).ignore_then($crate::parser::core::expr::expr($s))
	};
	($s:expr, optexpr ignore $ident:ident) => {
		$crate::parser::core::ident::ident()
			.then_ignore(jassg_op!($ident))
			.then($crate::parser::core::expr::expr($s).or_not())
	};
	($s:expr, $op:ident -> $ident:ident) => {
		$crate::parser::core::ident::ident()
			.then(jop!($op))
			.then(jassg_op!($ident))
			.then($crate::parser::core::expr::expr($s))
	};
	($s:expr, $op:ident -> ignore $ident:ident) => {
		$crate::parser::core::ident::ident()
			.then_ignore(jop!($op))
			.then_ignore(jassg_op!($ident))
			.then($crate::parser::core::expr::expr($s))
	};
	($s:expr, $op:ident -> noident $ident:ident) => {
		jop!($op)
			.then(jassg_op!($ident))
			.then($crate::parser::core::expr::expr($s))
	};
	($s:expr, $op:ident -> noident ignore $ident:ident) => {
		jop!($op)
			.then(jassg_op!($ident))
			.ignore_then($crate::parser::core::expr::expr($s))
	};
}
