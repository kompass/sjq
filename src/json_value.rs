use serde::Serialize;
use std::collections::HashMap;

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
