//! Utilities for tokenizing raw json string.
use super::{
    error::{JsonErrorType, JsonParseError},
    query::JsonQuery,
    token::{Json, Property},
};
use crate::parser::*;

macro_rules! parser {
    ($self:ident) => {
        $self.0
    };
}

macro_rules! parse {
    ($self:ident, byte{$char:expr}) => {
        parser!($self).parse_byte($char)
    };
    ($self:ident, string{$string:expr}) => {
        parser!($self).parse_string($string)
    };
    ($self:ident, int) => {
        parser!($self).parse_int()
    };
    ($self:ident, uint) => {
        parser!($self).parse_uint()
    };
    ($self:ident, $fn:expr) => {
        parser!($self).parse_while($fn)
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

type JsonLexerResult<T> = Result<T, (JsonErrorType, usize)>;

#[derive(Debug)]
pub struct JsonLexer(Parser);

impl JsonLexer /* Public */ {
    pub fn new(s: &str) -> Self {
        Self(Parser::new(s))
    }

    pub fn tokenize(&mut self) -> Result<Json, JsonParseError> {
        self.trim_front()
            .consume_any()
            .or_else(|(error_type, cursor)| {
                let position = parser!(self).position(cursor);
                let line = parser!(self)
                    .get_string()
                    .lines()
                    .skip(position.row - 1)
                    .take(1)
                    .collect();
                Err(JsonParseError {
                    line,
                    position,
                    error_type,
                })
            })
    }

    /// try parsing any token.
    pub fn consume_any(&mut self) -> JsonLexerResult<Json> {
        match parser!(self).peek() {
            Some('-' | '0'..='9') => self.consume_number(),
            Some('t' | 'f') => self.consume_boolean(),
            Some('"') => self.consume_qstring(),
            Some('n') => self.consume_null(),
            Some('[') => self.consume_array(),
            Some('{') => self.consume_object(),
            _ => return Err(self.error(JsonErrorType::SyntaxError)),
        }
    }

    /// try parsing [`Json::Null`](Json::Null).
    pub fn consume_null(&mut self) -> JsonLexerResult<Json> {
        parse!(self, string{"null"})
            .map(|_| Json::Null)
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`Json::Boolean`](Json::Boolean).
    pub fn consume_boolean(&mut self) -> JsonLexerResult<Json> {
        parse!(self, string{"true"})
            .or_else(|| parse!(self, string{"false"}))
            .map(|parsed| Json::Boolean(parsed == "true"))
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`Json::Number`](Json::Number).
    pub fn consume_number(&mut self) -> JsonLexerResult<Json> {
        let maybe_float = parse!(self, int).map(|n| n as f32);
        let maybe_decimal = maybe_float.and_then(|f| {
            // parse decimal point.
            parse!(self, byte{'.'})
                // parse leading decimal zeroes.
                .map(|_| parse!(self, |&ch| ch == '0').len() as i32)
                // parse decimal number.
                .and_then(|leading_zeroes| {
                    parse!(self, int).and_then(|number| {
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
            if parse!(self, byte{'e'})
                .or_else(|| parse!(self, byte{'E'}))
                .is_some()
            {
                let exponent = if parse!(self, byte{'+'}).is_some() {
                    parse!(self, uint).map(|n| n as i32)
                } else {
                    parse!(self, int)
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
    pub fn consume_qstring(&mut self) -> JsonLexerResult<Json> {
        self.consume_byte('"')?;
        let mut escaped = false;
        let string = parse!(self, |&ch| {
            if ch == '"' && !escaped {
                return false;
            }
            escaped = ch == '\\';
            true
        });
        self.consume_byte('"').and(Ok(Json::QString(string)))
    }

    /// try parsing [`Json::Array`](Json::Array).
    pub fn consume_array(&mut self) -> JsonLexerResult<Json> {
        self.consume_byte('[')?;
        let mut array = Vec::new();
        if self
            .trim_front()
            .consume_any()
            .map(|token| array.push(token))
            .is_ok()
        {
            // try parsing token, only if comma present.
            while self.trim_front().consume_byte(',').is_ok() {
                self.trim_front()
                    .consume_any()
                    .map(|token| array.push(token))
                    .or_else(|_| {
                        Err(self
                            .untrim_front()
                            .error(JsonErrorType::TrailingCommaError))
                    })?;
            }
        }
        self.trim_front()
            .consume_byte(']')
            .and(Ok(Json::Array(array)))
    }

    /// try parsing [`Json::Object`](Json::Object).
    pub fn consume_object(&mut self) -> JsonLexerResult<Json> {
        self.consume_byte('{')?;
        let mut hashmap = std::collections::HashMap::new();
        let mut string_key = String::new();
        let mut json_key = self.trim_front().consume_qstring().ok();
        while {
            // unwrap Json key -> string key.
            // error out if 'string_key' already present in the hashmap.
            match json_key {
                Some(Json::QString(key)) => {
                    if hashmap.contains_key(&key) {
                        // for better error message.
                        parser!(self).cursor -= key.len() - 1;
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
            // try parsing 'colon', error out if fails.
            self.trim_front()
                .consume_byte(':')?
                .trim_front()
                // try parsing 'Json', error out if fails..
                .consume_any()
                // insert 'key', 'Json' to hashmap if 'value' parsed.
                .map(|token| hashmap.insert(string_key.clone(), token))?;
            // try parsing json_key only if comma parsed.
            json_key = if self.trim_front().consume_byte(',').is_ok() {
                // comma needs to be followed by a string.
                self.trim_front().consume_qstring().map(Some).or_else(|_| {
                    Err(self
                        .untrim_front()
                        .error(JsonErrorType::TrailingCommaError))
                })?
            } else {
                None
            };
        }
        self.trim_front()
            .consume_byte('}')
            .and(Ok(Json::Object(hashmap)))
    }
}

impl JsonLexer /* Private */ {
    // TODO: use some helper function for triming whitespace characters, instead
    // of checking manually hardcoded characters.
    fn trim_front(&mut self) -> &mut Self {
        parse!(self, |c| c.is_whitespace());
        self
    }

    /// This is used only in case of erroring out (backing up cursor to error position).
    fn untrim_front(&mut self) -> &mut Self {
        parser!(self).cursor -= 1;
        while let Some(ch) = parser!(self).peek() {
            if ch.is_whitespace() && parser!(self).cursor > 0 {
                parser!(self).cursor -= 1;
            } else {
                break;
            }
        }
        self
    }

    #[inline]
    fn consume_byte(&mut self, c: char) -> JsonLexerResult<&mut Self> {
        parse!(self, byte { c })
            .ok_or(self.error(JsonErrorType::SyntaxError))?;
        Ok(self)
    }

    #[inline]
    fn error(&self, error_type: JsonErrorType) -> (JsonErrorType, Cursor) {
        (error_type, parser!(self).cursor)
    }
}

pub struct PropertyLexer(Parser);

impl PropertyLexer /* Public */ {
    pub fn new(s: &str) -> Self {
        Self(Parser::new(s))
    }

    pub fn consume_any(&mut self) -> Option<Result<Property, usize>> {
        let maybe_property = match parser!(self).peek() {
            Some('.') => self
                .try_consume(".keys()", Property::Keys)
                .or_else(|| self.try_consume(".values()", Property::Values))
                .or_else(|| self.try_consume(".length()", Property::Length))
                .or_else(|| self.consume_map_func())
                .or_else(|| self.consume_dot_prop()),
            Some('[') => {
                match parser!(self).peek_at(parser!(self).cursor + 1) {
                    Some('"') => self.consume_bracket_prop(),
                    Some('-' | '0'..='9') => self.consume_array_index(),
                    _ => return Some(Err(parser!(self).cursor + 2)),
                }
            }
            None => return None,
            _ => return Some(Err(parser!(self).cursor + 1)),
        };
        Some(maybe_property.ok_or(parser!(self).cursor))
    }

    /// try parsing [`Property::Dot`](Property::Dot).
    #[inline(always)]
    pub fn consume_dot_prop(&mut self) -> Option<Property> {
        parse!(self, byte{'.'})?;
        let prop = parse!(self, |&ch| !".[)".contains(ch));
        if prop.is_empty() {
            return None;
        }
        Some(Property::Dot(prop))
    }

    /// try parsing [`Property::Bracket`](Property::Bracket).
    #[inline(always)]
    pub fn consume_bracket_prop(&mut self) -> Option<Property> {
        parse!(self, string{"[\""})?;
        let prop = parse!(self, |&ch| ch != '"');
        if prop.is_empty() {
            return None;
        }
        parse!(self, string{"\"]"}).and(Some(Property::Bracket(prop)))
    }

    /// try parsing [`Property::Index`](Property::Index).
    #[inline(always)]
    pub fn consume_array_index(&mut self) -> Option<Property> {
        parse!(self, byte{'['})?;
        parse!(self, int).and_then(|inner| {
            parse!(self, byte{']'}).and(Some(Property::Index(inner)))
        })
    }

    /// try parsing [`Property::Map(JsonQuery)`](Property::Map).
    #[inline(always)]
    pub fn consume_map_func(&mut self) -> Option<Property> {
        parse!(self, string{".map("})?;
        let mut properties = vec![];
        while let Some(maybe_property) = self.consume_any() {
            if let Ok(property) = maybe_property {
                properties.push(property);
            } else {
                break;
            }
        }
        parse!(self, byte{')'}).and(Some(Property::Map(JsonQuery(properties))))
    }
}

impl PropertyLexer /* Private */ {
    #[inline(always)]
    fn try_consume(&mut self, s: &str, t: Property) -> Option<Property> {
        parse!(self, string { s }).and(Some(t))
    }
}

impl Iterator for PropertyLexer {
    type Item = Result<Property, usize>;
    fn next(&mut self) -> Option<Self::Item> {
        self.consume_any()
    }
}
