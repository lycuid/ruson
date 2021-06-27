pub mod error;

use super::json::{token::JsonProperty, JsonPropertyLexer};
use error::{QueryErrorType, QueryParseError};

#[derive(Debug, Clone)]
pub struct Query {
    pub properties: Vec<JsonProperty>,
}

impl Query {
    pub fn new(s: &str) -> Result<Self, QueryParseError> {
        let mut lexer = JsonPropertyLexer::new(s);
        let mut properties = Vec::new();

        lexer.root().ok_or(QueryParseError {
            string: String::from(s),
            pointer: lexer.parser.pointer,
            error_type: QueryErrorType::SyntaxError,
        })?;

        while let Some(maybe_property) = lexer.next() {
            let property = maybe_property.or_else(|pointer| {
                Err(QueryParseError {
                    string: String::from(s),
                    pointer,
                    error_type: QueryErrorType::SyntaxError,
                })
            })?;

            properties.push(property);
        }

        Ok(Self { properties })
    }
}
