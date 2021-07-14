use crate::{
    error::ErrorString,
    parser::{Pointer, Position},
};

#[derive(Debug, PartialEq)]
pub enum JsonErrorType {
    SyntaxError,
    DuplicateKeyError,
    TrailingCommaError,
}

pub struct JsonParseError {
    pub string: String,
    pub position: Position,
    pub error_type: JsonErrorType,
}

impl std::fmt::Display for JsonParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable_error = format!("{:?}", self.error_type).uncamelized();
        writeln!(
            f,
            "Json {} ({}:{})",
            printable_error, self.position.row, self.position.col
        )?;

        let start = std::cmp::max(0, self.position.col as i32 - 26);
        let printable_string = &self.string.shorten(start as usize);
        writeln!(f, "{}.\t| {}", self.position.row, printable_string)?;

        let error_position = if self.string.len() > 50 {
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
    InvalidRootProperty, // special error type for only one use case.
    SyntaxError,
}

pub struct JsonQueryParseError {
    pub string: String,
    pub pointer: Pointer,
    pub error_type: JsonQueryErrorType,
}

impl std::fmt::Display for JsonQueryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable_error = format!("{:?}", self.error_type).uncamelized();
        writeln!(f, "JsonQuery {} ({})", printable_error, self.pointer)?;

        let start = std::cmp::max(0, self.pointer as i32 - 26);
        let printable_string = self.string.shorten(start as usize);
        writeln!(f, "near: '{}'", printable_string)?;

        let error_position = if self.string.len() > 50 {
            std::cmp::min(self.pointer, 25)
        } else {
            self.pointer
        };
        write!(
            f,
            "       {}^",
            (1..error_position).map(|_| ' ').collect::<String>()
        )
    }
}

impl std::fmt::Debug for JsonQueryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
