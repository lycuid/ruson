use super::{
    error::{JsonQueryErrorType, JsonQueryParseError},
    token::JsonProperty,
    JsonPropertyLexer,
};

#[derive(Debug, Clone)]
pub struct JsonQuery {
    pub properties: Vec<JsonProperty>,
}

impl JsonQuery {
    pub fn new(s: &str) -> Result<Self, JsonQueryParseError> {
        let mut properties = Vec::new();
        let mut lexer = JsonPropertyLexer::new(s);

        lexer
            .parser
            .match_string(&format!("{}", JsonProperty::Data))
            .ok_or(JsonQueryParseError {
                string: s.into(),
                pointer: lexer.parser.pointer,
                error_type: JsonQueryErrorType::InvalidRootProperty,
            })?;

        while let Some(maybe_property) = lexer.next() {
            let property = maybe_property.or_else(|pointer| {
                Err(JsonQueryParseError {
                    string: s.into(),
                    pointer,
                    error_type: JsonQueryErrorType::SyntaxError,
                })
            })?;

            properties.push(property);
        }

        Ok(Self { properties })
    }
}
