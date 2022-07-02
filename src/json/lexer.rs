//! Utilities for tokenizing raw json string.
use super::{
    error::{JsonErrorType, JsonParseError},
    query::JsonQuery,
    token::{Json, Property},
};
use crate::parser::*;

pub type ParseError = (JsonErrorType, Cursor);
pub type ParseResult = Result<Json, ParseError>;

#[derive(Debug)]
pub struct JsonLexer {
    pub parser: Parser,
}

impl JsonLexer {
    pub fn new(s: &str) -> Self {
        Self {
            parser: Parser::new(s),
        }
    }

    pub fn tokenize(&mut self) -> Result<Json, JsonParseError> {
        self.trim_front()
            .next_token()
            .or_else(|(error_type, cursor)| {
                let position = self.parser.position(cursor);
                let line = self
                    .parser
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

    /// try parsing next token.
    pub fn next_token(&mut self) -> ParseResult {
        match self.parser.peek() {
            Some('-' | '0'..='9') => self.next_number(),
            Some('t' | 'f') => self.next_boolean(),
            Some('"') => self.next_qstring(),
            Some('n') => self.next_null(),
            Some('[') => self.next_array(),
            Some('{') => self.next_object(),
            _ => return Err(self.error(JsonErrorType::SyntaxError)),
        }
    }

    /// try parsing [`Json::Null`](Json::Null).
    pub fn next_null(&mut self) -> ParseResult {
        self.parser
            .match_string("null")
            .map(|_| Json::Null)
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`Json::Boolean`](Json::Boolean).
    pub fn next_boolean(&mut self) -> ParseResult {
        let parse_true = self.parser.match_string("true");
        let parse_false = || self.parser.match_string("false");

        parse_true
            .or_else(parse_false)
            .map(|parsed| Json::Boolean(parsed == "true"))
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`Json::Number`](Json::Number).
    pub fn next_number(&mut self) -> ParseResult {
        let maybe_float = self.parser.parse_int().map(|n| n as f32);

        let total_digits = |mut n: i32| -> i32 {
            let mut digits = 0;
            while n > 0 {
                n /= 10;
                digits += 1;
            }
            digits
        };

        let maybe_decimal = maybe_float.and_then(|f| {
            // parse decimal point.
            self.parser
                .match_char('.')
                // parse leading decimal zeroes.
                .map(|_| self.parser.match_while(|&ch| ch == '0').len() as i32)
                // parse decimal number.
                .and_then(|total_zeroes| {
                    self.parser.parse_int().and_then(|number| {
                        if number >= 0 {
                            let digits = total_digits(number) + total_zeroes;
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
            if self
                .parser
                .match_char('e')
                .or_else(|| self.parser.match_char('E'))
                .is_some()
            {
                let exponent = if self.parser.match_char('+').is_some() {
                    self.parser.parse_uint().map(|n| n as i32)
                } else {
                    self.parser.parse_int()
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
    pub fn next_qstring(&mut self) -> ParseResult {
        self.match_char('"')?;

        let mut escaped = false;
        let string = self.parser.match_while(|&ch| {
            if ch == '"' && !escaped {
                return false;
            }
            escaped = ch == '\\';
            true
        });

        self.match_char('"').and_then(|_| Ok(Json::QString(string)))
    }

    /// try parsing [`Json::Array`](Json::Array).
    pub fn next_array(&mut self) -> ParseResult {
        self.match_char('[')?;

        let mut array = Vec::new();
        if self
            .trim_front()
            .next_token()
            .map(|token| array.push(token))
            .is_ok()
        {
            // try parsing token, only if comma present.
            while self.trim_front().match_char(',').is_ok() {
                self.trim_front()
                    .next_token()
                    .map(|token| array.push(token))
                    .or_else(|_| {
                        Err(self
                            .untrim_front()
                            .error(JsonErrorType::TrailingCommaError))
                    })?;
            }
        }

        self.trim_front()
            .match_char(']')
            .and_then(|_| Ok(Json::Array(array)))
    }

    /// try parsing [`Json::Object`](Json::Object).
    pub fn next_object(&mut self) -> ParseResult {
        self.match_char('{')?;

        let mut hashmap = std::collections::HashMap::new();
        let mut string_key = String::new();

        let mut json_key = self.trim_front().next_qstring().ok();

        while {
            // unwrap Json key -> string key.
            // error out if 'string_key' already present in the hashmap.
            match json_key {
                Some(Json::QString(key)) => {
                    if hashmap.contains_key(&key) {
                        // for better error message.
                        self.parser.cursor -= key.len() - 1;
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
                .match_char(':')?
                .trim_front()
                // try parsing 'Json', error out if fails..
                .next_token()
                // insert 'key', 'Json' to hashmap if 'value' parsed.
                .map(|token| hashmap.insert(string_key.clone(), token))?;

            // try parsing json_key only if comma parsed.
            json_key = if self.trim_front().match_char(',').is_ok() {
                // comma needs to be followed by a string.
                self.trim_front()
                    .next_qstring()
                    .map(|token| Some(token))
                    .or_else(|_| {
                        Err(self
                            .untrim_front()
                            .error(JsonErrorType::TrailingCommaError))
                    })?
            } else {
                None
            };
        }

        self.trim_front()
            .match_char('}')
            .and_then(|_| Ok(Json::Object(hashmap)))
    }

    // TODO: use some helper function for triming whitespace characters, instead
    // of checking manually hardcoded characters.
    pub fn trim_front(&mut self) -> &mut Self {
        self.parser.match_while(|c| c.is_whitespace());
        self
    }

    /// This is used only in case of erroring out (backing up cursor to error position).
    pub fn untrim_front(&mut self) -> &mut Self {
        self.parser.cursor -= 1;

        while let Some(ch) = self.parser.peek() {
            if ch.is_whitespace() && self.parser.cursor > 0 {
                self.parser.cursor -= 1;
            } else {
                break;
            }
        }

        self
    }

    fn match_char(&mut self, c: char) -> Result<&mut Self, ParseError> {
        self.parser
            .match_char(c)
            .ok_or(self.error(JsonErrorType::SyntaxError))?;
        Ok(self)
    }

    fn error(&self, error_type: JsonErrorType) -> (JsonErrorType, Cursor) {
        (error_type, self.parser.cursor)
    }
}

pub struct PropertyLexer {
    pub parser: Parser,
}

impl PropertyLexer {
    pub fn new(s: &str) -> Self {
        Self {
            parser: Parser::new(s),
        }
    }

    #[inline(always)]
    pub fn try_match(&mut self, s: &str, t: Property) -> Option<Property> {
        self.parser.match_string(s).and_then(|_| Some(t))
    }

    /// try parsing [`Property::Dot`](Property::Dot).
    pub fn dotproperty(&mut self) -> Option<Property> {
        self.parser.match_char('.')?;

        let string = self.parser.match_while(|&ch| ch != '.' && ch != '[');
        if string.is_empty() {
            return None;
        }

        Some(Property::Dot(string))
    }

    /// try parsing [`Property::Bracket`](Property::Bracket).
    pub fn bracketproperty(&mut self) -> Option<Property> {
        self.parser.match_string("[\"")?;

        let string = self.parser.match_while(|&ch| ch != '"');
        if string.is_empty() {
            return None;
        }

        self.parser
            .match_string("\"]")
            .and_then(|_| Some(Property::Bracket(string)))
    }

    /// try parsing [`Property::Index`](Property::Index).
    pub fn arrayindex(&mut self) -> Option<Property> {
        self.parser.match_char('[')?;

        self.parser.parse_int().and_then(|number| {
            self.parser
                .match_char(']')
                .and_then(|_| Some(Property::Index(number)))
        })
    }

    /// try parsing [`Property::Map(JsonQuery)`](Property::Map).
    pub fn mapfunction(&mut self) -> Option<Property> {
        self.parser.match_string(".map(")?;

        let mut depth = 0;
        let query_string = self.parser.match_while(|&ch| match ch {
            '(' => {
                depth += 1;
                true
            }
            ')' => match depth {
                0 => false,
                _ => {
                    depth -= 1;
                    true
                }
            },
            _ => true,
        });

        JsonQuery::new(&query_string).ok().and_then(|query| {
            self.parser
                .match_char(')')
                .and_then(|_| Some(Property::Map(query)))
        })
    }
}

impl Iterator for PropertyLexer {
    type Item = Result<Property, Cursor>;

    fn next(&mut self) -> Option<Self::Item> {
        let maybe_property = match self.parser.peek() {
            Some('.') => self
                .try_match(".keys()", Property::Keys)
                .or_else(|| self.try_match(".values()", Property::Values))
                .or_else(|| self.try_match(".length()", Property::Length))
                .or_else(|| self.mapfunction())
                .or_else(|| self.dotproperty()),
            Some('[') => match self.parser.peek_at(self.parser.cursor + 1) {
                Some('"') => self.bracketproperty(),
                Some('-' | '0'..='9') => self.arrayindex(),
                _ => return Some(Err(self.parser.cursor + 2)),
            },
            None => return None,
            _ => return Some(Err(self.parser.cursor + 1)),
        };

        Some(maybe_property.ok_or(self.parser.cursor))
    }
}
