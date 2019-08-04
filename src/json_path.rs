use std::str::FromStr;
use combine::parser::Parser;

use combine::parser::item::token;
use combine::parser::repeat::{many, sep_by};
use combine::parser::sequence::between;

use crate::parse_basics::{string_expr, number_expr, ident_expr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseJsonPathError {
    kind: JsonPathErrorKind
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JsonPathErrorKind {
    Empty,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonPathStage {
    Node(String),
    Index(u64),
}

impl JsonPathStage {
    fn is_node(&self) -> bool {
        match self {
            &JsonPathStage::Node(_) => true,
            _ => false
        }
    }

    fn is_index(&self) -> bool {
        match self {
            &JsonPathStage::Index(_) => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonPath(Vec<JsonPathStage>);

impl JsonPath {
    pub fn root() -> JsonPath {
        JsonPath(Vec::new())
    }

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push_node(&mut self, node_name: &str) {
        self.0.push(JsonPathStage::Node(node_name.to_string()));
    }

    pub fn pop_node(&mut self) {
        assert!(self.0.last().map_or(false, |x| x.is_node()));

        self.0.pop();
    }

    pub fn push_index(&mut self, index: u64) {
        self.0.push(JsonPathStage::Index(index));
    }

    pub fn inc_index(&mut self) {
        if let JsonPathStage::Index(ref mut i) = self.0.last_mut().unwrap() {
            *i += 1;
        } else {
            panic!("A node in a JsonPath can't be incremented.");
        }
    }

    pub fn pop_index(&mut self) {
        assert!(self.0.last().map_or(false, |x| x.is_index()));

        self.0.pop();
    }

    pub fn is_part(&self, other: &JsonPath) -> bool {
        other.0.starts_with(&self.0)
    }

    pub fn is(&self, other: &JsonPath) -> bool {
        self == other
    }
}

impl FromStr for JsonPath {
    type Err = ParseJsonPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut path = JsonPath::root();

        let path_part_expr = string_expr().or(ident_expr()).and(many::<Vec<_>, _>(between(token('['), token(']'), number_expr())));

        let mut path_expr = token('.').with(sep_by::<Vec<_>, _, _>(path_part_expr, token('.')));

        let (parse_tree, rest) = path_expr.parse(s).unwrap();
        assert!(rest.is_empty());

        for node in parse_tree {
            path.push_node(&node.0);

            for elem in node.1 {
                path.push_index(elem);
            }
        }

        Ok(path)
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
        assert_eq!(JsonPath::from_str(".abc").unwrap(), JsonPath(vec![JsonPathStage::Node("abc".to_string())]));
        assert_eq!(JsonPath::from_str(".abc.defgh").unwrap(), JsonPath(vec![
            JsonPathStage::Node("abc".to_string()),
            JsonPathStage::Node("defgh".to_string())
        ]));
        assert_eq!(JsonPath::from_str(".abc[394].defgh").unwrap(), JsonPath(vec![
            JsonPathStage::Node("abc".to_string()),
            JsonPathStage::Index(394u64),
            JsonPathStage::Node("defgh".to_string())
        ]));
        assert_eq!(JsonPath::from_str(".abc[394][9380].defgh").unwrap(), JsonPath(vec![
            JsonPathStage::Node("abc".to_string()),
            JsonPathStage::Index(394u64),
            JsonPathStage::Index(9380u64),
            JsonPathStage::Node("defgh".to_string())
        ]));
    }

    #[test]
    fn is_part() {
        assert_eq!(JsonPath::from_str(".").unwrap().is_part(&JsonPath::from_str(".abc").unwrap()), true);
        assert_eq!(JsonPath::from_str(".abc").unwrap().is_part(&JsonPath::from_str(".").unwrap()), false);
        assert_eq!(JsonPath::from_str(".abc").unwrap().is_part(&JsonPath::from_str(".abc").unwrap()), true);
        assert_eq!(JsonPath::from_str(".abc").unwrap().is_part(&JsonPath::from_str(".abc[1]").unwrap()), true);
        assert_eq!(JsonPath::from_str(".abc[2]").unwrap().is_part(&JsonPath::from_str(".abc[1]").unwrap()), false);
    }
}