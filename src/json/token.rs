use super::query::JsonQuery;
use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonProperty {
    /// equivalent to `jsonObject.prop`
    Dot(String),
    /// equivalent to `jsonObject["prop"]`
    Bracket(String),
    /// equivalent to `jsonArray[0]`
    Index(usize),
    /// [`JsonToken::Object`](JsonToken::Object) keys.
    Keys,
    /// [`JsonToken::Object`](JsonToken::Object) values.
    Values,
    /// length of [`JsonToken::Array`](JsonToken::Array).
    Length,
    /// map function.
    Map(JsonQuery),
}

impl std::fmt::Display for JsonProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Dot(string) => write!(f, ".{}", string),
            Self::Bracket(string) => write!(f, "[\"{}\"]", string),
            Self::Index(index) => write!(f, "[{}]", index),
            _ => write!(f, "{:?}", format!("{:?}", self).to_ascii_lowercase()),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum JsonToken {
    Null,
    Boolean(bool),
    Number(f32),
    QString(String),
    Array(Vec<JsonToken>),
    Object(HashMap<String, JsonToken>),
}

impl JsonToken {
    fn variant(&self) -> String {
        match self {
            Self::Null => format!("Null"),
            Self::Boolean(_) => format!("Boolean"),
            Self::Number(_) => format!("Number"),
            Self::QString(_) => format!("String"),
            Self::Array(_) => format!("Array"),
            Self::Object(_) => format!("Object"),
        }
    }

    /// This is used for extracting a `JsonToken` value
    /// that matches the given [`JsonQuery`](JsonQuery), from the current object.
    pub fn apply(&self, query: &JsonQuery) -> Result<Self, String> {
        let mut token = self;
        let mut properties = query.properties();

        let maybe_orphan = loop {
            if let Some(prop) = properties.next() {
                match prop {
                    JsonProperty::Dot(string)
                    | JsonProperty::Bracket(string) => {
                        match token {
                            Self::Object(hashmap) => {
                                if let Some(t) = hashmap.get(string) {
                                    token = t;
                                } else {
                                    return Err(format!(
                                        "key doesn't exist: '{}'",
                                        string
                                    ));
                                }
                            }
                            _ => break Some(prop),
                        };
                    }
                    JsonProperty::Index(index) => match token {
                        Self::Array(array) => {
                            if let Some(t) = array.get(*index) {
                                token = t;
                            } else {
                                return Err(format!(
                                    "Invalid index: '{}'",
                                    index
                                ));
                            }
                        }
                        _ => break Some(prop),
                    },
                    JsonProperty::Keys => match token {
                        Self::Object(hashmap) => {
                            let keys = hashmap
                                .keys()
                                .map(|s| JsonToken::QString(s.clone()))
                                .collect();

                            let q = JsonQuery::from(
                                properties
                                    .map(|prop| prop.to_owned())
                                    .collect::<Vec<JsonProperty>>(),
                            );

                            return Self::Array(keys).apply(&q);
                        }
                        _ => {
                            return Err(vec![
                                "'keys' can only be applied on 'Object'."
                                    .into(),
                                format!("Found '{}' instead.", token.variant()),
                            ]
                            .join("\n"))
                        }
                    },
                    JsonProperty::Values => match token {
                        Self::Object(hashmap) => {
                            let values = hashmap
                                .values()
                                .map(|t| t.to_owned())
                                .collect();

                            let q = JsonQuery::from(
                                properties
                                    .map(|prop| prop.to_owned())
                                    .collect::<Vec<JsonProperty>>(),
                            );

                            return Self::Array(values).apply(&q);
                        }
                        _ => {
                            return Err(vec![
                                "'values' can only be applied on 'Object'."
                                    .into(),
                                format!("Found '{}' instead.", token.variant()),
                            ]
                            .join("\n"))
                        }
                    },
                    JsonProperty::Length => match token {
                        Self::Array(array) => {
                            return Ok(Self::Number(array.len() as f32));
                        }
                        _ => {
                            return Err(vec![
                                "'length' can only be applied on 'Array'."
                                    .into(),
                                format!("Found '{}' instead.", token.variant()),
                            ]
                            .join("\n"))
                        }
                    },
                    JsonProperty::Map(query) => match token {
                        Self::Array(array) => {
                            let arr = array
                                .iter()
                                .map(|json| json.apply(query))
                                .collect::<Result<Vec<JsonToken>, String>>()?;

                            let q = JsonQuery::from(
                                properties
                                    .map(|prop| prop.to_owned())
                                    .collect::<Vec<JsonProperty>>(),
                            );

                            return Self::Array(arr).apply(&q);
                        }
                        _ => {
                            return Err(vec![
                                "'map' can only be applied on 'Array'.".into(),
                                format!("Found '{}' instead.", token.variant()),
                            ]
                            .join("\n"))
                        }
                    },
                };
            } else {
                break None;
            }
        };

        if let Some(prop) = maybe_orphan {
            Err(format!("Query structure doesn't match (near '{}').", prop))
        } else {
            Ok(token.clone())
        }
    }
}

impl fmt::Display for JsonToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Boolean(value) => write!(f, "{}", value),
            Self::Number(value) => write!(f, "{}", value),
            Self::QString(value) => write!(f, "\"{}\"", value),
            Self::Array(array) => write!(f, "{:?}", array),
            Self::Object(map) => write!(f, "{:?}", map),
        }
    }
}

impl fmt::Debug for JsonToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
