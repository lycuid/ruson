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

    #[inline(always)]
    fn invalid(&self, prop: &Property) -> String {
        format!(" {}, found '{}' instead.", prop.invalid(), self.variant())
    }

    pub fn consume(&mut self, property: &Property) -> Result<&Self, String> {
        *self = match property {
            Property::Dot(s) | Property::Bracket(s) => match self {
                Self::Object(hashmap) => hashmap
                    .get(s)
                    .and_then(|token| Some(token.clone()))
                    .ok_or(format!(" key doesn't exist: '{}'", s)),
                _ => Err(self.invalid(property)),
            },
            Property::Index(i) => match self {
                Self::Array(array) => array
                    .get(*i as usize)
                    .and_then(|token| Some(token.clone()))
                    .ok_or(format!(
                        " Invalid index {} (for array of len {})",
                        i,
                        array.len()
                    )),
                _ => Err(self.invalid(property)),
            },
            Property::Keys => match self {
                Self::Object(hashmap) => Ok(Self::Array(
                    hashmap.keys().map(|k| Json::QString(k.clone())).collect(),
                )),
                _ => Err(self.invalid(property)),
            },
            Property::Values => match self {
                Self::Object(hashmap) => Ok(Self::Array(
                    hashmap.values().map(|v| v.clone()).collect(),
                )),
                _ => Err(self.invalid(property)),
            },
            Property::Length => match self {
                Self::Array(array) => Ok(Self::Number(array.len() as f32)),
                Self::QString(string) => Ok(Self::Number(string.len() as f32)),
                _ => Err(self.invalid(property)),
            },
            Property::Map(query) => match self {
                Self::Array(array) => Ok(Self::Array(
                    array
                        .iter_mut()
                        .map(|token| token.apply(query))
                        .collect::<Result<Vec<Json>, String>>()?,
                )),
                _ => Err(self.invalid(property)),
            },
        }?;
        Ok(self)
    }

    /// This is used for extracting a `Json` value that matches the given
    /// [`JsonQuery`](JsonQuery), from the current object.
    pub fn apply(&self, query: &JsonQuery) -> Result<Self, String> {
        let mut token = self.clone();
        for property in query.properties() {
            token.consume(&property)?;
        }
        Ok(token)
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
