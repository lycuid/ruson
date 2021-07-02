use crate::query::Query;
use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonProperty {
    // Root json tree.
    Root,
    /// equivalent to `jsonObject.prop`
    Dot(String),
    /// equivalent to `jsonObject["prop"]`
    Bracket(String),
    /// equivalent to `jsonArray[0]`
    Index(usize),
}

impl std::fmt::Display for JsonProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Root => {
                write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
            }
            Self::Dot(string) => write!(f, ".{}", string),
            Self::Bracket(string) => write!(f, "[\"{}\"]", string),
            Self::Index(index) => write!(f, "[{}]", index),
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
    /// This is used for extracting a `JsonToken` value
    /// that matches the given [`Query`], from the current object.
    ///
    /// [`Query`]: /ruson/query/struct.Query.html
    pub fn apply(&self, query: &Query) -> Result<Self, String> {
        let mut token = self;
        let mut properties = query.properties.iter();

        let maybe_orphan = loop {
            if let Some(prop) = properties.next() {
                match prop {
                    JsonProperty::Root => {}
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
                };
            } else {
                break None;
            }
        };

        if let Some(prop) = maybe_orphan {
            Err(format!("query structure doesn't match (near '{}').", prop))
        } else {
            Ok(token.clone())
        }
    }
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

pub enum Jsonfmt<'a> {
    Raw(&'a JsonToken),
    Pretty(&'a JsonToken),
    Table(&'a JsonToken),
}

impl<'a> std::fmt::Display for Jsonfmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Raw(token) => writeln!(f, "{}", token),
            Self::Pretty(token) => writeln!(f, "{:#}", token),
            Self::Table(token) => match token {
                JsonToken::Array(array) => {
                    for value in array {
                        writeln!(f, "{}", value)?;
                    }
                    Ok(())
                }
                JsonToken::Object(map) => {
                    for (key, value) in map {
                        writeln!(f, "{}\t{}", key, value)?;
                    }
                    Ok(())
                }
                _ => writeln!(f, "{}", token),
            },
        }
    }
}
