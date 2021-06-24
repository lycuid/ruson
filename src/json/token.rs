use std::{collections::HashMap, fmt};

#[derive(Clone, PartialEq)]
pub enum JsonToken {
    Null,
    Boolean(bool),
    Number(f32),
    QString(String),
    Array(Vec<JsonToken>),
    Object(HashMap<String, JsonToken>),
}

impl fmt::Debug for JsonToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Boolean(value) => write!(f, "{}", value),
            Self::Number(value) => write!(f, "{}", value),
            Self::QString(value) => write!(f, "\"{}\"", value),
            Self::Array(array) => f.debug_list().entries(array.iter()).finish(),
            Self::Object(map) => f.debug_map().entries(map.iter()).finish(),
        }
    }
}

impl fmt::Display for JsonToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

