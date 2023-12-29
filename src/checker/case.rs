use crate::{
	common::ident::Ident,
	common::{
		diagnostics::invalid_case,
		span::{Span, Spanned},
	},
};
use derive_more::Display;

#[derive(Display, PartialEq, Eq, Clone, Copy)]
pub enum Case {
	#[display(fmt = "pascal case")]
	PascalCase,
	#[display(fmt = "snake case")]
	SnakeCase,
	#[display(fmt = "snake or camel case")]
	SnakeCamel,
	#[display(fmt = "camel case")]
	CamelCase,
	#[display(fmt = "upper snake case")]
	UpperSnakeCase,
}

fn is_uppercase(str: &str) -> bool {
	for char in str.chars() {
		if char.is_alphabetic() && char.is_lowercase() {
			return false;
		}
	}
	true
}

fn has_uppercase(str: &str) -> bool {
	for char in str.chars() {
		if char.is_alphabetic() && char.is_uppercase() {
			return true;
		}
	}
	false
}

fn begins_with_uppercase(str: &str) -> bool {
	str.chars().next().unwrap().is_uppercase()
}

pub fn check(span: &Span, name: &str, wanted: Case) {
	let found = match wanted {
		Case::PascalCase => {
			if name.contains('_') {
				if is_uppercase(name) {
					Case::UpperSnakeCase
				} else {
					Case::SnakeCase
				}
			} else if !begins_with_uppercase(name) {
				if has_uppercase(name) {
					Case::CamelCase
				} else {
					Case::SnakeCamel
				}
			} else {
				Case::PascalCase
			}
		}
		Case::SnakeCase => {
			if has_uppercase(name) {
				if name.contains('_') && !is_uppercase(name) {
					Case::SnakeCase
				} else if name.contains('_') {
					Case::UpperSnakeCase
				} else if begins_with_uppercase(name) {
					Case::PascalCase
				} else {
					Case::CamelCase
				}
			} else {
				Case::SnakeCase
			}
		}
		Case::UpperSnakeCase => {
			if is_uppercase(name) {
				Case::UpperSnakeCase
			} else if name.contains('_') {
				Case::SnakeCase
			} else {
				Case::PascalCase
			}
		}
		_ => panic!("why"),
	};
	if found != wanted {
		invalid_case(*span, wanted, found);
	}
}

pub fn check_ident(ident: &Spanned<Ident>, wanted: Case) {
	check(&ident.span, &ident.value.to_string(), wanted);
}
