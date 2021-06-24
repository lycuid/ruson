pub mod error;
pub mod token;

use crate::utils::*;
use error::{JsonErrorType, JsonParseError};
use token::JsonToken;

pub struct JsonParser {
    pub string: String,
    pub token: Option<JsonToken>,
    pub stack: Stack,
    pub pointer: Pointer,
}

pub type ErrorTuple = (JsonErrorType, Pointer);
pub type ParseResult<'a> = Result<&'a mut JsonParser, ErrorTuple>;

impl JsonParser {
    const WS: [char; 2] = [' ', '\t'];
    const EOL: [char; 2] = ['\n', '\r'];

    pub fn parse(&mut self) -> Result<JsonToken, JsonParseError> {
        self.trim_front()
            .next_token()
            .and_then(|this| Ok(this.token.take().unwrap()))
            .or_else(|(error_type, ptr)| {
                let position = self.position(&ptr);
                let string = self
                    .string
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

    pub fn new(s: &str) -> Self {
        Self {
            string: String::from(s),
            token: None,
            stack: s.chars().collect(),
            pointer: 0,
        }
    }

    pub fn next_token(&mut self) -> ParseResult {
        match self.stack.get(self.pointer) {
            Some('-') | Some('0'..='9') => self.number(),
            Some('t') | Some('f') => self.boolean(),
            Some('"') => self.string(),
            Some('n') => self.null(),
            Some('[') => self.array(),
            Some('{') => self.object(),
            _ => Err(self.error(JsonErrorType::SyntaxError)),
        }
    }

    pub fn trim_front(&mut self) -> &mut Self {
        let (_, ptr) = parse_while(&self.stack, &self.pointer, |c| {
            Self::WS.contains(c) || Self::EOL.contains(c)
        });
        self.move_pointer(ptr)
    }

    // This is in case of erroring out (backing up ptr to error position).
    pub fn untrim_front(&mut self) -> &mut Self {
        let mut ptr = std::cmp::max(0, self.pointer - 1);
        while (Self::WS.contains(&self.stack[ptr])
            || Self::EOL.contains(&self.stack[ptr]))
            && ptr > 0
        {
            ptr -= 1;
        }
        self.move_pointer(ptr + 1)
    }

    pub fn peek_char(&mut self, c: char) -> ParseResult {
        parse_char(c, &self.stack, &self.pointer)
            .ok_or_else(|| self.error(JsonErrorType::SyntaxError))?;

        Ok(self.move_pointer(self.pointer + 1))
    }

    pub fn string(&mut self) -> ParseResult {
        self.peek_char('"')?;

        let mut escaped = false;
        let (string, ptr) = parse_while(&self.stack, &self.pointer, |&ch| {
            if ch == '"' && !escaped {
                return false;
            }
            escaped = ch == '\\';
            true
        });

        self.move_pointer(ptr)
            .set_token(JsonToken::QString(string))
            .peek_char('"')
    }

    pub fn null(&mut self) -> ParseResult {
        let (_, ptr) = parse_string("null", &self.stack, &self.pointer)
            .ok_or_else(|| self.error(JsonErrorType::SyntaxError))?;

        Ok(self.move_pointer(ptr).set_token(JsonToken::Null))
    }

    pub fn boolean(&mut self) -> ParseResult {
        let parse_true = || parse_string("true", &self.stack, &self.pointer);
        let parse_false = || parse_string("false", &self.stack, &self.pointer);

        let (parsed, ptr) = parse_true()
            .or_else(parse_false)
            .ok_or_else(|| self.error(JsonErrorType::SyntaxError))?;

        Ok(self
            .move_pointer(ptr)
            .set_token(JsonToken::Boolean(parsed == "true")))
    }

    pub fn number(&mut self) -> ParseResult {
        let parse_unit_number = || {
            parse_int(&self.stack, &self.pointer)
                .and_then(|(number, ptr)| Some((number as f32, ptr)))
                .ok_or((0f32, self.pointer))
        };

        // just moving the pointer ahead, if parse successful.
        let parse_decimal_point = |(number, ptr)| {
            parse_char('.', &self.stack, &ptr)
                .and(Some((number, ptr + 1)))
                .ok_or((number, ptr))
        };

        // never fails, only moves the pointer ahead.
        let parse_decimal_leading_zeroes = |(number, ptr)| {
            let (zeroes, new_ptr) =
                parse_while(&self.stack, &ptr, |&c| c == '0');
            Ok((number, zeroes.len() as i32, new_ptr))
        };

        let parse_decimal_number = |(number, total_zeroes, ptr)| {
            if let Some((decimal_num, new_ptr)) = parse_int(&self.stack, &ptr) {
                if decimal_num >= 0 {
                    let digits = no_of_digits(decimal_num) + total_zeroes;
                    let divisor = (10i32).pow(digits as u32) as f32;
                    let decimal = decimal_num as f32 / divisor;

                    return Ok((
                        number
                            + if number >= 0f32 { decimal } else { -decimal },
                        new_ptr,
                    ));
                }
            }

            Err((number, ptr))
        };

        let (float, ptr) = parse_unit_number() //       () -> (f32, Pointer)
            .and_then(parse_decimal_point) //           (f32, Pointer) -> (f32, Pointer)
            .and_then(parse_decimal_leading_zeroes) //  (f32, Pointer) -> (f32, i32, Pointer)
            .and_then(parse_decimal_number) //          (f32, i32, Pointer) -> (f32, Pointer)
            .or_else(|(float, ptr)| {
                if ptr > self.pointer {
                    Ok((float, ptr))
                } else {
                    Err(self.error(JsonErrorType::SyntaxError))
                }
            })?;

        Ok(self.move_pointer(ptr).set_token(JsonToken::Number(float)))
    }

    pub fn array(&mut self) -> ParseResult {
        self.peek_char('[')?;

        let mut array = Vec::new();
        let mut token_parsed = self
            .trim_front()
            .next_token()
            .and_then(|this| Ok(array.push(this.token.take().unwrap())))
            .is_ok();

        while token_parsed {
            // try parsing token, only if comma present.
            token_parsed = if self.trim_front().peek_char(',').is_ok() {
                // if comma is not followed by a 'JsonToken', then error out.
                self.trim_front()
                    .next_token()
                    .and_then(|this| {
                        array.push(this.token.take().unwrap());
                        Ok(true)
                    })
                    .or_else(|_| {
                        Err(self
                            .untrim_front()
                            .error(JsonErrorType::TrailingCommaError))
                    })?
            } else {
                false
            }
        }

        self.set_token(JsonToken::Array(array))
            .trim_front()
            .peek_char(']')
    }

    pub fn object(&mut self) -> ParseResult {
        self.peek_char('{')?;

        let mut hashmap = std::collections::HashMap::new();
        let mut string_key = String::new();

        let mut json_key = self
            .trim_front()
            .string()
            .and_then(|this| Ok(this.token.take()))
            .unwrap_or(None);

        while {
            // unwrap JsonToken key -> string key.
            // error out if 'string_key' already present in the hashmap.
            match json_key {
                Some(JsonToken::QString(key)) => {
                    if hashmap.contains_key(&key) {
                        // for better error message.
                        self.move_pointer(self.pointer - key.len() - 1);
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
                .peek_char(':')?
                .trim_front()
                // try parsing 'JsonToken', error out if fails..
                .next_token()
                // insert 'key', 'JsonToken' to hashmap if 'value' parsed.
                .and_then(|this| {
                    Ok(hashmap
                        .insert(string_key.clone(), this.token.take().unwrap()))
                })?;

            // try parsing json_key only if comma parsed.
            json_key = if self.trim_front().peek_char(',').is_ok() {
                // comma needs to be followed by a string.
                self.trim_front()
                    .string()
                    .and_then(|this| Ok(this.token.take()))
                    .or_else(|_| {
                        Err(self
                            .untrim_front()
                            .error(JsonErrorType::TrailingCommaError))
                    })?
            } else {
                None
            };
        }

        self.set_token(JsonToken::Object(hashmap))
            .trim_front()
            .peek_char('}')
    }

    fn move_pointer(&mut self, ptr: Pointer) -> &mut Self {
        self.pointer = ptr;
        self
    }

    fn error(&self, error_type: JsonErrorType) -> ErrorTuple {
        (error_type, self.pointer)
    }

    fn set_token(&mut self, token: JsonToken) -> &mut Self {
        self.token = Some(token);
        self
    }

    pub fn position(&self, xi: &Pointer) -> Position {
        let Position { mut row, mut col } = Position::new();
        let mut chars = self.string.chars().take(*xi);

        while let Some(ref ch) = chars.next() {
            if Self::EOL.contains(ch) {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        Position { row, col }
    }
}
