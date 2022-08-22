//! AST.
use super::query::JsonQuery;
use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Property {
    /// equivalent to `jsonObject.prop`
    Dot(String),
    /// equivalent to `jsonObject["prop"]`
    Bracket(String),
    /// equivalent to `jsonArray[0]`
    Index(i32),
    /// [`Json::Object`](Json::Object) keys.
    Keys,
    /// [`Json::Object`](Json::Object) values.
    Values,
    /// length of [`Json::Array`](Json::Array).
    Length,
    /// map function.
    Map(JsonQuery),
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Dot(s) => write!(f, ".{}", s),
            Self::Bracket(s) => write!(f, "[\"{}\"]", s),
            Self::Index(i) => write!(f, "[{}]", i),
            Self::Map(_) => write!(f, ".map()"),
            _ => write!(f, "{}", format!(".{:?}()", self).to_ascii_lowercase()),
        }
    }
}

impl Property {
    #[inline(always)]
    fn invalid(&self) -> String {
        match self {
            Self::Dot(_) | Self::Bracket(_) => {
                "Dot/Bracket properties are only valid on 'Object'".into()
            }
            Self::Index(_) => "Indexing is only valid on 'Array'".into(),
            Self::Keys | Self::Values => {
                format!("'{}' can only be applied on 'Object'", self)
            }
            Self::Length => {
                format!("'{}' can only be applied on 'Array' or 'String'", self)
            }
            Self::Map(_) => {
                format!("'{}' can only be applied on 'Array'", self)
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f32),
    QString(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

impl Json {
    #[inline(always)]
    fn variant(&self) -> &str {
        match self {
            Self::Null => "Null",
            Self::Boolean(_) => "Boolean",
            Self::Number(_) => "Number",
            Self::QString(_) => "String",
            Self::Array(_) => "Array",
            Self::Object(_) => "Object",
        }
    }

    #[inline]
    pub fn update(&mut self, property: &Property) -> Result<&Self, String> {
        macro_rules! match_only {
            ($($pattern:pat => $expr:expr),*) => {
                match self {
                    $($pattern => $expr),*,
                    _ => Err(format!(" {}, found '{}' instead.",
                                     property.invalid(), self.variant())),
                }
            }
        }
        *self = match property {
            Property::Dot(s) | Property::Bracket(s) => match_only! {
                Self::Object(hashmap) => hashmap
                    .get(s)
                    .cloned()
                    .ok_or(format!(" key doesn't exist: '{}'", s))
            },
            Property::Index(i) => match_only! {
                Self::Array(array) => {
                    array.get(*i as usize).cloned().ok_or(format!(
                        " Invalid index {} (for array of len {})",
                        i,
                        array.len()
                    ))
                }
            },
            Property::Keys => match_only! {
                Self::Object(hashmap) => Ok(Self::Array(
                    hashmap.keys().cloned().map(Json::QString).collect()
                ))
            },
            Property::Values => match_only! {
                Self::Object(hashmap) => {
                    Ok(Self::Array(hashmap.values().cloned().collect()))
                }
            },
            Property::Length => match_only! {
                Self::Array(array) => Ok(Self::Number(array.len() as f32)),
                Self::QString(string) => Ok(Self::Number(string.len() as f32))
            },
            Property::Map(query) => match_only! {
                Self::Array(array) => Ok(Self::Array(
                    array
                        .iter_mut()
                        .map(|token| token.apply(query))
                        .collect::<Result<Vec<Json>, String>>()?,
                ))
            },
        }?;
        Ok(self)
    }

    /// This is used for extracting a `Json` value that matches the given
    /// [`JsonQuery`](JsonQuery), from the current object.
    pub fn apply(&self, query: &JsonQuery) -> Result<Self, String> {
        let mut json = self.clone();
        for property in query.properties() {
            json.update(&property)?;
        }
        Ok(json)
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Boolean(boolean) => write!(f, "{}", boolean),
            Self::Number(float) => write!(f, "{}", float),
            Self::QString(string) => write!(f, "\"{}\"", string),
            Self::Array(array) => write!(f, "{:?}", array),
            Self::Object(hashmap) => write!(f, "{:?}", hashmap),
        }
    }
}

impl fmt::Debug for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
