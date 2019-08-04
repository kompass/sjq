use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonValue {
    Null,
    String(String),
    Number(u64),
    Boolean(bool),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::String(s) => write!(f, "\"{}\"", s),
            JsonValue::Number(n) => write!(f, "{}", n),
            JsonValue::Boolean(b) => write!(f, "{}", b),
            JsonValue::Object(m) => {
                write!(f, "{{")?;

                let mut elems = m.iter();

                if let Some((first_key, first_val)) = elems.next() {
                    write!(f, "\"{}\": {}", first_key, first_val)?;

                    for (key, val) in elems {
                        write!(f, ", \"{}\": {}", key, val)?;
                    }
                }

                write!(f, "}}")
            },
            JsonValue::Array(v) => {
                write!(f, "[")?;

                let mut elems = v.iter();

                if let Some(val) = elems.next() {
                    write!(f, "{}", val)?;

                    for val in elems {
                        write!(f, ", {}", val)?
                    }
                }

                write!(f, "]")
            }
        }
    }
}
