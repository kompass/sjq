use std::str::FromStr;

use combine::parser::Parser;

use combine::parser::choice::choice;
use combine::parser::combinator::attempt;
use combine::parser::item::{eof, token};
use combine::parser::repeat::many1;
use combine::parser::sequence::between;

use crate::parse_basics::{ident_expr, index_expr, string_expr};

/// Represents a step in a JSON path.
///
/// Since the arrays and the objects are the only elements of a JSON document containing sub-elements,
/// a path step is designing a field of an object or the index of an element in an array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonPathStep {
    Field(String),
    Index(u64),
}

impl JsonPathStep {
    fn is_node(&self) -> bool {
        match *self {
            JsonPathStep::Field(_) => true,
            _ => false,
        }
    }

    fn is_index(&self) -> bool {
        match *self {
            JsonPathStep::Index(_) => true,
            _ => false,
        }
    }
}

/// Represents any possible JSON path.
///
/// A path is a sequence of steps from the root.
/// Since a JSON document is a tree and a step can't go back, there is one and only one path for each document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonPath(Vec<JsonPathStep>);

impl JsonPath {
    /// Create a no-op path, a path to the root.
    pub fn root() -> JsonPath {
        JsonPath(Vec::new())
    }

    /// Append a object field to cross to the path.
    pub fn push_field(&mut self, node_name: &str) {
        self.0.push(JsonPathStep::Field(node_name.to_string()));
    }

    /// Remove the last field step from the path.
    ///
    /// # Panics
    /// Panics if the last step of the path isn't a `Field`.
    pub fn pop_field(&mut self) {
        assert!(self.0.last().map_or(false, |x| x.is_node()));

        self.0.pop();
    }

    /// Append a array index to cross to the path.
    pub fn push_index(&mut self, index: u64) {
        self.0.push(JsonPathStep::Index(index));
    }

    /// Increment the last array index of the path.
    ///
    /// # Panics
    /// Panics if the last step of the path isn't an `Index`.
    pub fn inc_index(&mut self) {
        if let JsonPathStep::Index(ref mut i) = self.0.last_mut().unwrap() {
            *i += 1;
        } else {
            panic!("A node in a JsonPath can't be incremented.");
        }
    }

    /// Remove the last array index from the path
    ///
    /// # Panics
    /// Panics if the last step of the path isn't an `Index`.
    pub fn pop_index(&mut self) {
        assert!(self.0.last().map_or(false, |x| x.is_index()));

        self.0.pop();
    }

    /// Iterate over the steps of the path.
    pub fn iter(&self) -> std::slice::Iter<'_, JsonPathStep> {
        self.0.iter()
    }

    /// Get the number of steps of the path.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl FromStr for JsonPath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let max_text_length = s.len();

        let field_path_expr = token('.')
            .with(string_expr(max_text_length).or(ident_expr(max_text_length)))
            .map(|field_name| JsonPathStep::Field(field_name));

        let index_path_expr = between(token('['), token(']'), index_expr())
            .map(|array_index| JsonPathStep::Index(array_index));

        let path_step_expr = field_path_expr.or(index_path_expr);

        let mut path_expr = choice((
            attempt((token('.'), eof())).map(|_| JsonPath::root()),
            many1::<Vec<_>, _>(path_step_expr)
                .skip(eof())
                .map(|v| JsonPath(v)),
        ));

        let (path, rest) = path_expr.parse(s).unwrap();
        assert!(rest.is_empty());

        Ok(path)
    }
}
