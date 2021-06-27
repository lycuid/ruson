pub mod error;
pub mod token;

use super::parser::*;
use error::{JsonErrorType, JsonParseError};
use token::{JsonProperty, JsonToken};

pub type ParseError = (JsonErrorType, Pointer);
pub type ParseResult = Result<JsonToken, ParseError>;

#[derive(Debug)]
pub struct JsonLexer {
    pub parser: Parser,
}

impl Iterator for JsonLexer {
    type Item = ParseResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.peek() {
            Some('-') | Some('0'..='9') => Some(self.number()),
            Some('t') | Some('f') => Some(self.boolean()),
            Some('"') => Some(self.qstring()),
            Some('n') => Some(self.null()),
            Some('[') => Some(self.array()),
            Some('{') => Some(self.object()),
            _ => return None,
        }
    }
}

impl JsonLexer {
    pub const WS: [char; 2] = [' ', '\t'];
    pub const EOL: [char; 2] = ['\n', '\r'];

    pub fn new(s: &str) -> Self {
        Self {
            parser: Parser::new(s),
        }
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

    /// calls `next()` and converts it to [ParseResult](type.ParseResult.html).
    pub fn next_token(&mut self) -> ParseResult {
        self.next()
            .unwrap_or(Err(self.error(JsonErrorType::SyntaxError)))
    }

    /// try parsing [JsonToken::Null](token/enum.JsonToken.html#variant.Null).
    pub fn null(&mut self) -> ParseResult {
        self.parser
            .match_string("null")
            .and_then(|_| Some(JsonToken::Null))
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [JsonToken::Boolean](token/enum.JsonToken.html#variant.Boolean).
    pub fn boolean(&mut self) -> ParseResult {
        let parse_true = self.parser.match_string("true");
        let parse_false = || self.parser.match_string("false");

        parse_true
            .or_else(parse_false)
            .and_then(|parsed| Some(JsonToken::Boolean(parsed == "true")))
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [JsonToken::Number](token/enum.JsonToken.html#variant.Number).
    pub fn number(&mut self) -> ParseResult {
        let maybe_number = self.parser.parse_int().and_then(|n| Some(n as f32));

        let maybe_float = if let Some(f) = maybe_number {
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
                            let digits = no_of_digits(number) + total_zeroes;
                            let divisor = 10i32.pow(digits as u32) as f32;
                            let decimal = number as f32 / divisor;

                            Some(f + if f >= 0. { decimal } else { -decimal })
                        } else {
                            None
                        }
                    })
                })
                // any of the above fails, then return original number.
                .or(Some(f))
        } else {
            None
        };

        maybe_float
            .and_then(|float| Some(JsonToken::Number(float)))
            .ok_or(self.error(JsonErrorType::SyntaxError))
    }

    /// try parsing [JsonToken::QString](token/enum.JsonToken.html#variant.QString).
    pub fn qstring(&mut self) -> ParseResult {
        self.try_char('"')?;

        let mut escaped = false;
        let string = self.parser.match_while(|&ch| {
            if ch == '"' && !escaped {
                return false;
            }
            escaped = ch == '\\';
            true
        });

        self.try_char('"')
            .and_then(|_| Ok(JsonToken::QString(string)))
    }

    /// try parsing [JsonToken::Array](token/enum.JsonToken.html#variant.Array).
    pub fn array(&mut self) -> ParseResult {
        self.try_char('[')?;

        let mut array = Vec::new();
        if self
            .trim_front()
            .next_token()
            .and_then(|token| Ok(array.push(token)))
            .is_ok()
        {
            // try parsing token, only if comma present.
            while self.trim_front().try_char(',').is_ok() {
                self.trim_front()
                    .next_token()
                    .and_then(|token| {
                        array.push(token);
                        Ok(true)
                    })
                    .or_else(|_| {
                        Err(self
                            .untrim_front()
                            .error(JsonErrorType::TrailingCommaError))
                    })?;
            }
        }

        self.trim_front()
            .try_char(']')
            .and_then(|_| Ok(JsonToken::Array(array)))
    }

    /// try parsing [JsonToken::Object](token/enum.JsonToken.html#variant.Object).
    pub fn object(&mut self) -> ParseResult {
        self.try_char('{')?;

        let mut hashmap = std::collections::HashMap::new();
        let mut string_key = String::new();

        let mut json_key = self.trim_front().qstring().ok();

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
                .try_char(':')?
                .trim_front()
                // try parsing 'JsonToken', error out if fails..
                .next_token()
                // insert 'key', 'JsonToken' to hashmap if 'value' parsed.
                .and_then(|token| {
                    Ok(hashmap.insert(string_key.clone(), token))
                })?;

            // try parsing json_key only if comma parsed.
            json_key = if self.trim_front().try_char(',').is_ok() {
                // comma needs to be followed by a string.
                self.trim_front()
                    .qstring()
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
            .try_char('}')
            .and_then(|_| Ok(JsonToken::Object(hashmap)))
    }

    fn try_char<'a>(&'a mut self, c: char) -> Result<&'a mut Self, ParseError> {
        self.parser
            .match_char(c)
            .ok_or(self.error(JsonErrorType::SyntaxError))?;
        Ok(self)
    }

    fn error(&self, error_type: JsonErrorType) -> (JsonErrorType, Pointer) {
        (error_type, self.parser.pointer)
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

    /// try parsing [JsonProperty::Root](token/enum.JsonProperty.html#variant.Root).
    pub fn root(&mut self) -> Option<JsonProperty> {
        self.parser
            .match_string(format!("{}", JsonProperty::Root).as_str())
            .and(Some(JsonProperty::Root))
    }

    /// try parsing [JsonProperty::Dot](token/enum.JsonProperty.html#variant.Dot).
    pub fn dotproperty(&mut self) -> Option<JsonProperty> {
        self.parser.match_char('.')?;

        let string = self.parser.match_while(|&ch| ch != '.' && ch != '[');
        if string.is_empty() {
            return None;
        }

        Some(JsonProperty::Dot(string))
    }

    /// try parsing [JsonProperty::Bracket](token/enum.JsonProperty.html#variant.Bracket).
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

    /// try parsing [JsonProperty::Index](token/enum.JsonProperty.html#variant.Index).
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
}

impl Iterator for JsonPropertyLexer {
    type Item = Result<JsonProperty, Pointer>;

    fn next(&mut self) -> Option<Self::Item> {
        let maybe_property = match self.parser.peek() {
            Some('.') => self.dotproperty(),
            Some('[') => match self.parser.peek_at(self.parser.pointer + 1) {
                Some('"') => self.bracketproperty(),
                Some('0'..='9') => self.arrayindex(),
                _ => return Some(Err(self.parser.pointer + 2)),
            },
            Some('r') => self.root(),
            None => return None,
            _ => return Some(Err(self.parser.pointer + 1)),
        };

        Some(maybe_property.ok_or(self.parser.pointer))
    }
}

fn no_of_digits(mut n: i32) -> i32 {
    let mut digits = 0;
    while n > 0 {
        n /= 10;
        digits += 1;
    }
    digits
}
