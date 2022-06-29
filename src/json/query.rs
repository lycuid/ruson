//! list of properties (chronological) needed to extract sub tree from `json`.
use super::{
    error::{JsonQueryError, JsonQueryErrorType},
    lexer::PropertyLexer,
    token::Property,
};

#[derive(Debug, Clone, PartialEq)]
pub struct JsonQuery {
    properties: Vec<Property>,
}

impl JsonQuery {
    pub fn new(s: &str) -> Result<Self, JsonQueryError> {
        let mut properties = Vec::new();
        for maybe_property in PropertyLexer::new(s) {
            let property = maybe_property.or_else(|pointer| {
                Err(JsonQueryError {
                    line: s.into(),
                    pointer,
                    error_type: JsonQueryErrorType::SyntaxError,
                })
            })?;
            properties.push(property)
        }
        Ok(Self { properties })
    }

    pub fn properties<'a>(&'a self) -> std::slice::Iter<'a, Property> {
        self.properties.iter()
    }
}

impl From<std::slice::Iter<'_, Property>> for JsonQuery {
    fn from(it: std::slice::Iter<'_, Property>) -> Self {
        Self {
            properties: it.map(|prop| prop.clone()).collect(),
        }
    }
}
