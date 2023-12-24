pub trait Join<T: std::fmt::Display>: IntoIterator<Item = T> + Copy {
	fn join_comma(&self) -> Option<String> {
		self.into_iter()
			.map(|x| format!("{x}"))
			.reduce(|acc, b| acc + ", " + &b)
	}

	fn join_comma_or_empty(&self) -> String {
		match self.join_comma() {
			Some(x) => x,
			None => String::new(),
		}
	}

	fn join_comma_wrapped(&self, wrap_l: &str, wrap_r: &str) -> String {
		match self.join_comma() {
			Some(x) => wrap_l.to_string() + &x + wrap_r,
			None => String::new(),
		}
	}
}

impl<T: std::fmt::Display, S: IntoIterator<Item = T> + Copy> Join<T> for S {}
