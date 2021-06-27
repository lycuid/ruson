use crate::{error::ErrorString, parser::Pointer};

#[derive(Debug, PartialEq)]
pub enum QueryErrorType {
    InvalidStartingProperty, // special error type for only one use case.
    SyntaxError,
}

pub struct QueryParseError {
    pub string: String,
    pub pointer: Pointer,
    pub error_type: QueryErrorType,
}

impl std::fmt::Display for QueryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable_error = format!("{:?}", self.error_type).uncamelized();
        writeln!(f, "Query {} ({})", printable_error, self.pointer)?;

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

impl std::fmt::Debug for QueryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
