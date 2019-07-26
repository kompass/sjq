use std::str::FromStr;
use combine::parser::Parser;

use combine::parser::item::token;
use combine::parser::repeat::{many, sep_by};
use combine::parser::sequence::between;

use crate::parse_basics::{string_expr, number_expr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseJsonPathError {
	kind: JsonPathErrorKind
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JsonPathErrorKind {
	Empty,
	Invalid,
}

#[derive(Debug, Clone)]
pub enum JsonPathStage {
	Node(String),
	Element(u64),
}

impl JsonPathStage {
	fn is_node(&self) -> bool {
		match self {
			&JsonPathStage::Node(_) => true,
			_ => false
		}
	}
}

#[derive(Debug, Clone)]
pub struct JsonPath(Vec<JsonPathStage>);

impl JsonPath {
	pub fn root() -> JsonPath {
		JsonPath(Vec::new())
	}

	pub fn is_root(&self) -> bool {
		self.0.is_empty()
	}

	pub fn push_node(&mut self, node_name: String) {
		self.0.push(JsonPathStage::Node(node_name));
	}

	pub fn pop_node(&mut self) {
		assert!(self.0.last().map_or(false, |x| x.is_node()));

		self.0.pop();
	}
}

impl FromStr for JsonPath {
	type Err = ParseJsonPathError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut path = JsonPath::root();

		let path_part_expr = string_expr().and(many::<Vec<_>, _>(between(token(b'['), token(b']'), number_expr())));

		let mut path_expr = token(b'.').with(sep_by::<Vec<_>, _, _>(path_part_expr, token(b'.')));

		let (parse_tree, rest) = path_expr.parse(s.as_bytes()).unwrap();
		assert!(rest.is_empty());

		dbg!(parse_tree);

		Ok(JsonPath::root())
	}
}

#[cfg(test)]
mod tests{
	use super::*;

	#[test]
	fn root_is_root() {
		assert!(JsonPath::root().is_root());
	}

	#[test]
	fn json_path_from_str() {
		assert!(JsonPath::from_str(".").unwrap().is_root());
	}
}