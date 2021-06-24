use crate::{query::Query, utils::*};
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

#[derive(Debug, Clone, PartialEq)]
pub enum JsonProperty {
    Dot(String),     // obj.prop
    Bracket(String), // obj["prop"]
    Index(usize),    // array[0]
}

type MaybeJsonProperty = Option<(JsonProperty, Pointer)>;

impl JsonProperty {
    pub fn parse_dot_prop(xs: &Stack, xi: &Pointer) -> MaybeJsonProperty {
        let parse_dot = || parse_char('.', xs, xi).and_then(|_| Some(xi + 1));

        let parse_property = |ptr| {
            let (string, new_ptr) =
                parse_while(xs, &ptr, |&ch| ch != '.' && ch != '[');
            if string.is_empty() {
                None
            } else {
                Some((string, new_ptr))
            }
        };

        parse_dot()
            .and_then(parse_property)
            .and_then(|(string, ptr)| Some((JsonProperty::Dot(string), ptr)))
    }

    pub fn parse_bracket_prop(xs: &Stack, xi: &Pointer) -> MaybeJsonProperty {
        let parse_bracket_open = || {
            parse_string(r#"[""#, xs, xi).and_then(|(_, new_ptr)| Some(new_ptr))
        };

        let parse_property = |ptr| {
            let (string, new_ptr) = parse_while(xs, &ptr, |&ch| ch != '"');
            if string.is_empty() {
                None
            } else {
                Some((string, new_ptr))
            }
        };

        let parse_bracket_close = |(string, ptr)| {
            parse_string(r#""]"#, xs, &ptr)
                .and_then(|(_, new_ptr)| Some((string, new_ptr)))
        };

        parse_bracket_open()
            .and_then(parse_property)
            .and_then(parse_bracket_close)
            .and_then(|(string, ptr)| {
                Some((JsonProperty::Bracket(string), ptr))
            })
    }

    pub fn parse_index_prop(xs: &Stack, xi: &Pointer) -> MaybeJsonProperty {
        let parse_bracket_open =
            || parse_char('[', xs, xi).and_then(|_| Some(xi + 1));

        let parse_index_number = |ptr| {
            let (string, new_ptr) =
                parse_while(xs, &ptr, |&ch| ch.is_digit(10));

            if let Ok(number) = string.parse() {
                Some((number, new_ptr))
            } else {
                None
            }
        };

        let parse_bracket_close = |(number, ptr)| {
            parse_char(']', &xs, &ptr).and_then(|_| Some((number, ptr + 1)))
        };

        parse_bracket_open()
            .and_then(parse_index_number)
            .and_then(parse_bracket_close)
            .and_then(|(number, ptr)| Some((JsonProperty::Index(number), ptr)))
    }
}

impl std::fmt::Display for JsonProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Dot(string) => write!(f, ".{}", string),
            Self::Bracket(string) => write!(f, "[\"{}\"]", string),
            Self::Index(index) => write!(f, "[{}]", index),
        }
    }
}