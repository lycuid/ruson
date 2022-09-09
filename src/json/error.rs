//! Error types (mainly parsing related), implements [`Display`](std::fmt::Display)
//! for well formatted error messages.
use crate::{
    error::ErrorString,
    lexer::{Cursor, Position},
};

#[derive(Debug, PartialEq)]
pub enum JsonErrorType {
    SyntaxError,
    DuplicateKeyError,
    TrailingCommaError,
}

pub struct JsonParseError {
    pub line: String,
    pub position: Position,
    pub error_type: JsonErrorType,
}

impl std::fmt::Display for JsonParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable_error = format!("{:?}", self.error_type).uncamelize();
        writeln!(
            f,
            "{}:{} Json {} ",
            self.position.row, self.position.col, printable_error
        )?;

        let start = std::cmp::max(0, self.position.col as i32 - 26);
        let printable_string = &self.line.shorten(start as usize);
        writeln!(f, "{}.\t| {}", self.position.row, printable_string)?;

        let error_position = if self.line.len() > 50 {
            std::cmp::min(self.position.col, 25)
        } else {
            self.position.col
        };
        write!(
            f,
            "\t| {}^",
            (1..error_position).map(|_| ' ').collect::<String>()
        )
    }
}

impl std::fmt::Debug for JsonParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[derive(Debug, PartialEq)]
pub enum JsonQueryErrorType {
    SyntaxError,
}

pub struct JsonQueryError {
    pub line: String,
    pub cursor: Cursor,
    pub error_type: JsonQueryErrorType,
}

impl std::fmt::Display for JsonQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable_error = format!("{:?}", self.error_type).uncamelize();
        writeln!(f, "{} JsonQuery {}", self.cursor, printable_error)?;

        let start = std::cmp::max(0, self.cursor as i32 - 26);
        let printable_string = self.line.shorten(start as usize);
        writeln!(f, "near: '{}'", printable_string)?;

        let error_position = if self.line.len() > 50 {
            std::cmp::min(self.cursor, 25)
        } else {
            self.cursor
        };
        write!(
            f,
            "       {}^",
            (1..error_position).map(|_| ' ').collect::<String>()
        )
    }
}

impl std::fmt::Debug for JsonQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
