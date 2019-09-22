use serde::Serialize;
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

use crate::json_path::{JsonPath, JsonPathStep};

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(untagged)]
pub enum NumberVal {
    Integer(i64),
    Float(f64),
}

/// Represents any possible value of a JSON document.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum JsonValue {
    Null,
    String(String),
    Number(NumberVal),
    Boolean(bool),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

impl JsonValue {
    pub fn normalized_string(s: &str) -> JsonValue {
        JsonValue::String(s.nfc().collect())
    }

    pub fn select<'a>(&'a self, path: &JsonPath) -> Option<&'a JsonValue> {
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

impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_| std::fmt::Error)?
        )
    }
}
