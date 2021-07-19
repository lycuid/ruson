use super::{
    error::{JsonQueryError, JsonQueryErrorType},
    token::JsonProperty,
    JsonPropertyLexer,
};

#[derive(Debug, Clone)]
pub struct JsonQuery {
    properties: Vec<JsonProperty>,
}

impl JsonQuery {
    pub fn new(s: &str) -> Result<Self, JsonQueryError> {
        let mut properties = Vec::new();
        let mut lexer = JsonPropertyLexer::new(s);

        lexer
            .parser
            .match_string(&format!("{}", JsonProperty::Root))
            .ok_or(JsonQueryError {
                string: s.into(),
                pointer: lexer.parser.pointer,
                error_type: JsonQueryErrorType::InvalidRootProperty,
            })?;

        while let Some(maybe_property) = lexer.next() {
            let property = maybe_property.or_else(|pointer| {
                Err(JsonQueryError {
                    string: s.into(),
                    pointer,
                    error_type: JsonQueryErrorType::SyntaxError,
                })
            })?;

            properties.push(property);
        }

        Ok(Self { properties })
    }

    pub fn properties<'a>(&'a self) -> std::slice::Iter<'a, JsonProperty> {
        self.properties.iter()
    }
}
