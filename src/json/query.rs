//! list of properties (chronological) needed to extract sub tree from `json`.
use super::{
    error::{JsonQueryError, JsonQueryErrorType},
    token::JsonProperty,
    JsonPropertyLexer,
};

#[derive(Debug, Clone, PartialEq)]
pub struct JsonQuery {
    properties: Vec<JsonProperty>,
}

impl JsonQuery {
    pub fn new(s: &str) -> Result<Self, JsonQueryError> {
        let mut properties = Vec::new();
        let mut lexer = JsonPropertyLexer::new(s);

        while let Some(maybe_property) = lexer.next() {
            let property = maybe_property.or_else(|pointer| {
                Err(JsonQueryError {
                    line: s.into(),
                    pointer,
                    error_type: JsonQueryErrorType::SyntaxError,
                })
            })?;
            properties.push(property);
        }

        Ok(Self::from(properties))
    }

    pub fn from(properties: Vec<JsonProperty>) -> Self {
        Self { properties }
    }

    pub fn properties<'a>(&'a self) -> std::slice::Iter<'a, JsonProperty> {
        self.properties.iter()
    }
}
