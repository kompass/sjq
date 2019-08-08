use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum JsonValue {
    Null,
    String(String),
    Number(u64),
    Boolean(bool),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}
