//! list of properties (chronological) needed to extract sub tree from `json`.
use super::{
    error::{JsonQueryError, JsonQueryErrorType},
    parser::PropertyParser,
    token::Property,
};

#[derive(Debug, Clone, PartialEq)]
pub struct JsonQuery(pub Vec<Property>);

impl JsonQuery {
    pub fn new(s: &str) -> Result<Self, JsonQueryError> {
        let mut properties = Vec::new();
        for maybe_property in PropertyParser::new(s) {
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
