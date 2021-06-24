use crate::{error::ErrorString, utils::Position};

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
