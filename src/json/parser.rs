//! Utilities for tokenizing raw json string.
use super::{
    error::{JsonErrorType, JsonParseError},
    query::JsonQuery,
    token::{Json, Property},
};
use crate::lexer::*;

macro_rules! lexer {
    ($self:expr) => {
        $self.0
    };
}

macro_rules! ndigits {
    ($num:ident) => {{
        let (mut num, mut digits) = ($num, 0);
        while num > 0 {
            (num, digits) = (num / 10, digits + 1);
        }
        digits
    }};
}

type JsonParseResult<T> = Result<T, (JsonErrorType, usize)>;

#[derive(Debug)]
pub struct JsonParser(Lexer);

impl JsonParser /* Public */ {
    pub fn new(s: &str) -> Self {
        Self(Lexer::new(s))
    }

    #[inline(always)]
    pub fn parse(&mut self) -> Result<Json, JsonParseError> {
        self.trim_front()
            .parse_any()
            .or_else(|(error_type, cursor)| {
                let position = lexer!(self).position(cursor);
                Err(JsonParseError {
                    line: lexer!(self)
                        .get_string()
                        .lines()
                        .skip(position.row - 1)
                        .take(1)
                        .collect(),
                    position,
                    error_type,
                })
            })
    }

    /// try parsing any token.
    #[inline(always)]
    pub fn parse_any(&mut self) -> JsonParseResult<Json> {
        match lexer!(self).peek() {
            Some('-' | '0'..='9') => self.parse_number(),
            Some('t' | 'f') => self.parse_boolean(),
            Some('"') => self.parse_qstring(),
            Some('n') => self.parse_null(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            _ => return Err(self.error(JsonErrorType::SyntaxError)),
        }
    }

    /// try parsing [`Json::Null`](Json::Null).
    pub fn parse_null(&mut self) -> JsonParseResult<Json> {
        lexer!(self)
            .consume_string("null")
            .map(|_| Json::Null)
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`Json::Boolean`](Json::Boolean).
    pub fn parse_boolean(&mut self) -> JsonParseResult<Json> {
        lexer!(self)
            .consume_string("true")
            .or_else(|| lexer!(self).consume_string("false"))
            .map(|parsed| Json::Boolean(parsed == "true"))
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`Json::Number`](Json::Number).
    pub fn parse_number(&mut self) -> JsonParseResult<Json> {
        let maybe_float = lexer!(self).consume_int().map(|n| n as f32);
        let maybe_decimal = maybe_float.and_then(|f| {
            // parse decimal point.
            lexer!(self)
                .consume_byte('.')
                // parse leading decimal zeroes.
                .map(|_| {
                    lexer!(self).consume_while(|&ch| ch == '0').len() as i32
                })
                // parse decimal number.
                .and_then(|leading_zeroes| {
                    lexer!(self).consume_int().and_then(|number| {
                        if number >= 0 {
                            let digits = ndigits!(number) + leading_zeroes;
                            let decimal = number as f32 / 10f32.powi(digits);
                            Some(f + if f >= 0. { decimal } else { -decimal })
                        } else {
                            None
                        }
                    })
                })
                // any of the above fails, then return original number.
                .or(Some(f))
        });
        let maybe_exponent = maybe_decimal.and_then(|f| {
            // if 'e' or 'E' parsed, then try parsing '[sign]int'.
            if lexer!(self)
                .consume_byte('e')
                .or_else(|| lexer!(self).consume_byte('E'))
                .is_some()
            {
                let exponent = if lexer!(self).consume_byte('+').is_some() {
                    lexer!(self).consume_uint().map(|n| n as i32)
                } else {
                    lexer!(self).consume_int()
                };
                exponent.and_then(|exp| format!("{}e{}", f, exp).parse().ok())
            } else {
                // return previously parsed float, if 'e' or 'E' not present
                // immediately after.
                Some(f)
            }
        });
        maybe_exponent
            .map(Json::Number)
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`Json::QString`](Json::QString).
    pub fn parse_qstring(&mut self) -> JsonParseResult<Json> {
        self.parse_byte('"')?;
        let mut escaped = false;
        let string = lexer!(self).consume_while(|&ch| {
            if ch == '"' && !escaped {
                return false;
            }
            escaped = ch == '\\';
            true
        });
        self.parse_byte('"').and(Ok(Json::QString(string)))
    }

    /// try parsing [`Json::Array`](Json::Array).
    pub fn parse_array(&mut self) -> JsonParseResult<Json> {
        self.parse_byte('[')?;
        let mut array = Vec::new();
        if self
            .trim_front()
            .parse_any()
            .map(|token| array.push(token))
            .is_ok()
        {
            // try parsing token, only if comma present.
            while self.trim_front().parse_byte(',').is_ok() {
                self.trim_front()
                    .parse_any()
                    .map(|token| array.push(token))
                    .or_else(|_| {
                        Err(self
                            .untrim_front()
                            .error(JsonErrorType::TrailingCommaError))
                    })?;
            }
        }
        self.trim_front()
            .parse_byte(']')
            .and(Ok(Json::Array(array)))
    }

    /// try parsing [`Json::Object`](Json::Object).
    pub fn parse_object(&mut self) -> JsonParseResult<Json> {
        self.parse_byte('{')?;
        let mut hashmap = std::collections::HashMap::new();
        let mut string_key = String::new();
        let mut json_key = self.trim_front().parse_qstring().ok();
        while {
            // unwrap Json key -> string key.
            match json_key {
                Some(Json::QString(key)) => {
                    if hashmap.contains_key(&key) {
                        lexer!(self).cursor -= key.len() - 1; // for better error message.
                        return Err(
                            self.error(JsonErrorType::DuplicateKeyError)
                        );
                    }
                    string_key = key;
                    true
                }
                _ => false,
            }
        } {
            self.trim_front()
                .parse_byte(':')?
                .trim_front()
                .parse_any()
                .map(|token| hashmap.insert(string_key.clone(), token))?;
            // try parsing 'json_key' only if comma parsed.
            json_key = if self.trim_front().parse_byte(',').is_ok() {
                // comma needs to be followed by a string.
                self.trim_front().parse_qstring().map(Some).or_else(|_| {
                    Err(self
                        .untrim_front()
                        .error(JsonErrorType::TrailingCommaError))
                })?
            } else {
                None
            };
        }
        self.trim_front()
            .parse_byte('}')
            .and(Ok(Json::Object(hashmap)))
    }
}

impl JsonParser /* Private */ {
    #[inline]
    fn trim_front(&mut self) -> &mut Self {
        lexer!(self).consume_while(|c| c.is_whitespace());
        self
    }

    /// This is used only in case of erroring out (backing up cursor to error position).
    #[inline]
    fn untrim_front(&mut self) -> &mut Self {
        lexer!(self).cursor -= 1;
        while let Some(ch) = lexer!(self).peek() {
            if ch.is_whitespace() && lexer!(self).cursor > 0 {
                lexer!(self).cursor -= 1;
            } else {
                break;
            }
        }
        self
    }

    #[inline]
    fn parse_byte(&mut self, c: char) -> JsonParseResult<&mut Self> {
        lexer!(self)
            .consume_byte(c)
            .ok_or(self.error(JsonErrorType::SyntaxError))?;
        Ok(self)
    }

    #[inline(always)]
    fn error(&self, error_type: JsonErrorType) -> (JsonErrorType, Cursor) {
        (error_type, lexer!(self).cursor)
    }
}

pub struct PropertyParser(Lexer);

impl PropertyParser /* Public */ {
    #[rustfmt::skip]
    pub fn new(s: &str) -> Self { Self(Lexer::new(s)) }

    pub fn parse_any(&mut self) -> Option<Result<Property, usize>> {
        let maybe_property = match lexer!(self).peek() {
            Some('.') => self
                .try_consume(".keys()", Property::Keys)
                .or_else(|| self.try_consume(".values()", Property::Values))
                .or_else(|| self.try_consume(".length()", Property::Length))
                .or_else(|| self.parse_map_func())
                .or_else(|| self.parse_dot_prop()),
            Some('[') => match lexer!(self).peek_at(lexer!(self).cursor + 1) {
                Some('"') => self.parse_bracket_prop(),
                Some('-' | '0'..='9') => self.parse_array_index(),
                _ => return Some(Err(lexer!(self).cursor + 2)),
            },
            None => return None,
            _ => return Some(Err(lexer!(self).cursor + 1)),
        };
        Some(maybe_property.ok_or(lexer!(self).cursor))
    }

    /// try parsing [`Property::Dot`](Property::Dot).
    #[inline(always)]
    pub fn parse_dot_prop(&mut self) -> Option<Property> {
        lexer!(self).consume_byte('.')?;
        let prop = lexer!(self).consume_while(|&ch| !".[)".contains(ch));
        if prop.is_empty() {
            return None;
        }
        Some(Property::Dot(prop))
    }

    /// try parsing [`Property::Bracket`](Property::Bracket).
    #[inline(always)]
    pub fn parse_bracket_prop(&mut self) -> Option<Property> {
        lexer!(self).consume_string("[\"")?;
        let prop = lexer!(self).consume_while(|&ch| ch != '"');
        if prop.is_empty() {
            return None;
        }
        lexer!(self)
            .consume_string("\"]")
            .and(Some(Property::Bracket(prop)))
    }

    /// try parsing [`Property::Index`](Property::Index).
    #[inline(always)]
    pub fn parse_array_index(&mut self) -> Option<Property> {
        lexer!(self).consume_byte('[')?;
        lexer!(self).consume_int().and_then(|num| {
            lexer!(self)
                .consume_byte(']')
                .and(Some(Property::Index(num)))
        })
    }

    /// try parsing [`Property::Map(JsonQuery)`](Property::Map).
    #[inline(always)]
    pub fn parse_map_func(&mut self) -> Option<Property> {
        lexer!(self).consume_string(".map(")?;
        let mut properties = vec![];
        while let Some(maybe_property) = self.parse_any() {
            if let Ok(property) = maybe_property {
                properties.push(property);
            } else {
                break;
            }
        }
        lexer!(self)
            .consume_byte(')')
            .and(Some(Property::Map(JsonQuery(properties))))
    }
}

impl PropertyParser /* Private */ {
    #[inline(always)]
    fn try_consume(&mut self, s: &str, t: Property) -> Option<Property> {
        lexer!(self).consume_string(s).and(Some(t))
    }
}

impl Iterator for PropertyParser {
    type Item = Result<Property, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_any()
    }
}
