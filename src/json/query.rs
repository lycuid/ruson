//! list of properties (chronological) needed to extract sub tree from `json`.
use super::{
    error::{JsonQueryError, JsonQueryErrorType},
    lexer::PropertyLexer,
    token::Property,
};

#[derive(Debug, Clone, PartialEq)]
pub struct JsonQuery(Vec<Property>);

impl JsonQuery {
    pub fn new(s: &str) -> Result<Self, JsonQueryError> {
        let mut properties = Vec::new();
        for maybe_property in PropertyLexer::new(s) {
            let property = maybe_property.or_else(|cursor| {
                Err(JsonQueryError {
                    line: s.into(),
                    cursor,
                    error_type: JsonQueryErrorType::SyntaxError,
                })
            })?;
            properties.push(property)
        }
        Ok(Self(properties))
    }

    pub fn properties<'a>(&'a self) -> std::slice::Iter<'a, Property> {
        self.0.iter()
    }
}

// required by mod `tests`.
impl From<std::slice::Iter<'_, Property>> for JsonQuery {
    fn from(iter: std::slice::Iter<'_, Property>) -> Self {
        Self(iter.map(|prop| prop.to_owned()).collect())
    }
}
