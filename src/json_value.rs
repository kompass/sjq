use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    String(String),
    Number(u64),
    Boolean(bool),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}