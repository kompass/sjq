use serde::Serialize;
use std::collections::HashMap;

use crate::json_path::{JsonPath, JsonPathStep};

/// Represents any possible value of a JSON document.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum JsonValue {
    Null,
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

impl JsonValue {
    pub fn select<'a>(&'a self, path: JsonPath) -> Option<&'a JsonValue> {
        let mut selected = self;

        for step in path.iter() {
            match (step, selected) {
                (JsonPathStep::Field(ref field_name), JsonValue::Object(ref fields)) => {
                    if let Some(field) = fields.get(field_name) {
                        selected = field;
                    } else {
                        return None;
                    }
                }
                (JsonPathStep::Index(id), JsonValue::Array(ref elems)) => {
                    if let Some(elem) = elems.get(*id as usize) {
                        selected = elem;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(selected)
    }
}
