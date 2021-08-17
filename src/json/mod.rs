//! Json parsing and processing utilities.
pub mod error;
pub mod formatter;
pub mod query;
pub mod token;

use super::parser::*;
use error::{JsonErrorType, JsonParseError};
use query::JsonQuery;
use token::{JsonProperty, JsonToken};

pub type ParseError = (JsonErrorType, Pointer);
pub type ParseResult = Result<JsonToken, ParseError>;

#[derive(Debug)]
pub struct JsonTokenLexer {
    pub parser: Parser,
}

impl JsonTokenLexer {
    const WS: [char; 2] = [' ', '\t'];
    const EOL: [char; 2] = ['\n', '\r'];

    pub fn new(s: &str) -> Self {
        Self {
            parser: Parser::new(s),
        }
    }

    pub fn tokenize(&mut self) -> Result<JsonToken, JsonParseError> {
        self.trim_front().next_token().or_else(|(error_type, ptr)| {
            let position = self.parser.position(ptr);
            let string = self
                .parser
                .get_string()
                .lines()
                .skip(position.row - 1)
                .take(1)
                .collect();

            Err(JsonParseError {
                string,
                position,
                error_type,
            })
        })
    }

    /// calls `next()` and unwraps to [`ParseResult`](ParseResult).
    pub fn next_token(&mut self) -> ParseResult {
        self.next()
            .unwrap_or(Err(self.error(JsonErrorType::SyntaxError)))
    }

    /// try parsing [`JsonToken::Null`](JsonToken::Null).
    pub fn next_null(&mut self) -> ParseResult {
        self.parser
            .match_string("null")
            .and_then(|_| Some(JsonToken::Null))
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`JsonToken::Boolean`](JsonToken::Boolean).
    pub fn next_boolean(&mut self) -> ParseResult {
        let parse_true = self.parser.match_string("true");
        let parse_false = || self.parser.match_string("false");

        parse_true
            .or_else(parse_false)
            .and_then(|parsed| Some(JsonToken::Boolean(parsed == "true")))
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`JsonToken::Number`](JsonToken::Number).
    pub fn next_number(&mut self) -> ParseResult {
        let maybe_float = self.parser.parse_int().and_then(|n| Some(n as f32));

        let maybe_decimal = maybe_float.and_then(|f| {
            // parse decimal point.
            self.parser
                .match_char('.')
                // parse leading decimal zeroes.
                .and_then(|_| {
                    Some(self.parser.match_while(|&ch| ch == '0').len() as i32)
                })
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
                    self.parser.parse_uint().and_then(|n| Some(n as i32))
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
            .and_then(|number| Some(JsonToken::Number(number)))
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [`JsonToken::QString`](JsonToken::QString).
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

        self.match_char('"')
            .and_then(|_| Ok(JsonToken::QString(string)))
    }

    /// try parsing [`JsonToken::Array`](JsonToken::Array).
    pub fn next_array(&mut self) -> ParseResult {
        self.match_char('[')?;

        let mut array = Vec::new();
        if self
            .trim_front()
            .next_token()
            .and_then(|token| Ok(array.push(token)))
            .is_ok()
        {
            // try parsing token, only if comma present.
            while self.trim_front().match_char(',').is_ok() {
                self.trim_front()
                    .next_token()
                    .and_then(|token| Ok(array.push(token)))
                    .or_else(|_| {
                        Err(self
                            .untrim_front()
                            .error(JsonErrorType::TrailingCommaError))
                    })?;
            }
        }

        self.trim_front()
            .match_char(']')
            .and_then(|_| Ok(JsonToken::Array(array)))
    }

    /// try parsing [`JsonToken::Object`](JsonToken::Object).
    pub fn next_object(&mut self) -> ParseResult {
        self.match_char('{')?;

        let mut hashmap = std::collections::HashMap::new();
        let mut string_key = String::new();

        let mut json_key = self.trim_front().next_qstring().ok();

        while {
            // unwrap JsonToken key -> string key.
            // error out if 'string_key' already present in the hashmap.
            match json_key {
                Some(JsonToken::QString(key)) => {
                    if hashmap.contains_key(&key) {
                        // for better error message.
                        self.parser.pointer -= key.len() - 1;
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
                // try parsing 'JsonToken', error out if fails..
                .next_token()
                // insert 'key', 'JsonToken' to hashmap if 'value' parsed.
                .and_then(|token| {
                    Ok(hashmap.insert(string_key.clone(), token))
                })?;

            // try parsing json_key only if comma parsed.
            json_key = if self.trim_front().match_char(',').is_ok() {
                // comma needs to be followed by a string.
                self.trim_front()
                    .next_qstring()
                    .and_then(|token| Ok(Some(token)))
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
            .and_then(|_| Ok(JsonToken::Object(hashmap)))
    }

    pub fn trim_front(&mut self) -> &mut Self {
        self.parser
            .match_while(|c| Self::WS.contains(c) || Self::EOL.contains(c));
        self
    }

    /// This is used only in case of erroring out (backing up ptr to error position).
    pub fn untrim_front(&mut self) -> &mut Self {
        self.parser.pointer -= 1;

        while let Some(ch) = self.parser.peek() {
            if (Self::WS.contains(ch) || Self::EOL.contains(ch))
                && self.parser.pointer > 0
            {
                self.parser.pointer -= 1;
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

    fn error(&self, error_type: JsonErrorType) -> (JsonErrorType, Pointer) {
        (error_type, self.parser.pointer)
    }
}

impl Iterator for JsonTokenLexer {
    type Item = ParseResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.peek() {
            Some('-') | Some('0'..='9') => Some(self.next_number()),
            Some('t') | Some('f') => Some(self.next_boolean()),
            Some('"') => Some(self.next_qstring()),
            Some('n') => Some(self.next_null()),
            Some('[') => Some(self.next_array()),
            Some('{') => Some(self.next_object()),
            _ => return None,
        }
    }
}

pub struct JsonPropertyLexer {
    pub parser: Parser,
}

impl JsonPropertyLexer {
    pub fn new(s: &str) -> Self {
        Self {
            parser: Parser::new(s),
        }
    }

    /// try parsing [`JsonProperty::Dot`](JsonProperty::Dot).
    pub fn dotproperty(&mut self) -> Option<JsonProperty> {
        self.parser.match_char('.')?;

        let string = self.parser.match_while(|&ch| ch != '.' && ch != '[');
        if string.is_empty() {
            return None;
        }

        Some(JsonProperty::Dot(string))
    }

    /// try parsing [`JsonProperty::Bracket`](JsonProperty::Bracket).
    pub fn bracketproperty(&mut self) -> Option<JsonProperty> {
        self.parser.match_string("[\"")?;

        let string = self.parser.match_while(|&ch| ch != '"');
        if string.is_empty() {
            return None;
        }

        self.parser
            .match_string("\"]")
            .and_then(|_| Some(JsonProperty::Bracket(string)))
    }

    /// try parsing [`JsonProperty::Index`](JsonProperty::Index).
    pub fn arrayindex(&mut self) -> Option<JsonProperty> {
        self.parser.match_char('[')?;

        self.parser
            .match_while(|&ch| ch.is_digit(10))
            .parse()
            .ok()
            .and_then(|number| {
                self.parser
                    .match_char(']')
                    .and_then(|_| Some(JsonProperty::Index(number)))
            })
    }

    /// try parsing [`JsonProperty::Keys`](JsonProperty::Keys).
    pub fn keysfunction(&mut self) -> Option<JsonProperty> {
        self.parser
            .match_string(".keys()")
            .and_then(|_| Some(JsonProperty::Keys))
    }

    /// try parsing [`JsonProperty::Values`](JsonProperty::Values).
    pub fn valuesfunction(&mut self) -> Option<JsonProperty> {
        self.parser
            .match_string(".values()")
            .and_then(|_| Some(JsonProperty::Values))
    }

    /// try parsing [`JsonProperty::Length`](JsonProperty::Length).
    pub fn lengthfunction(&mut self) -> Option<JsonProperty> {
        self.parser
            .match_string(".length()")
            .and_then(|_| Some(JsonProperty::Length))
    }

    /// try parsing [`JsonProperty::Map(JsonQuery)`](JsonProperty::Map).
    pub fn mapfunction(&mut self) -> Option<JsonProperty> {
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
                .and_then(|_| Some(JsonProperty::Map(query)))
        })
    }
}

impl Iterator for JsonPropertyLexer {
    type Item = Result<JsonProperty, Pointer>;

    fn next(&mut self) -> Option<Self::Item> {
        let maybe_property = match self.parser.peek() {
            Some('.') => self
                .keysfunction()
                .or_else(|| self.valuesfunction())
                .or_else(|| self.lengthfunction())
                .or_else(|| self.mapfunction())
                .or_else(|| self.dotproperty()),
            Some('[') => match self.parser.peek_at(self.parser.pointer + 1) {
                Some('"') => self.bracketproperty(),
                Some('0'..='9') => self.arrayindex(),
                _ => return Some(Err(self.parser.pointer + 2)),
            },
            None => return None,
            _ => return Some(Err(self.parser.pointer + 1)),
        };

        Some(maybe_property.ok_or(self.parser.pointer))
    }
}

fn total_digits(mut n: i32) -> i32 {
    let mut digits = 0;
    while n > 0 {
        n /= 10;
        digits += 1;
    }
    digits
}
