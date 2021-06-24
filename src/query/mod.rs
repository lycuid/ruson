pub mod error;

use super::{json::token::JsonProperty, utils::*};
use error::{QueryErrorType, QueryParseError};

#[derive(Debug, Clone)]
pub struct Query {
    pub string: String,
    pub properties: Vec<JsonProperty>,
}

impl Query {
    pub fn new(s: &str) -> Result<Self, QueryParseError> {
        let string = String::from(s);
        let mut properties = Vec::new();

        let stack = string.chars().collect();
        let mut pointer = 0;

        pointer = parse_string("data", &stack, &pointer)
            .and_then(|(_, ptr)| Some(ptr))
            .ok_or(QueryParseError {
                string: string.clone(),
                pointer,
                error_type: QueryErrorType::InvalidStartingProperty,
            })?;

        while pointer < stack.len() {
            let maybe_property = match stack.get(pointer) {
                Some('.') => JsonProperty::parse_dot_prop(&stack, &pointer),
                Some('[') => match stack.get(pointer + 1) {
                    Some('"') => {
                        JsonProperty::parse_bracket_prop(&stack, &pointer)
                    }
                    Some('0'..='9') => {
                        JsonProperty::parse_index_prop(&stack, &pointer)
                    }
                    _ => {
                        return Err(QueryParseError {
                            string,
                            pointer: pointer + 2,
                            error_type: QueryErrorType::SyntaxError,
                        })
                    }
                },
                _ => {
                    return Err(QueryParseError {
                        string,
                        pointer: pointer + 1,
                        error_type: QueryErrorType::SyntaxError,
                    })
                }
            };

            if let Some((prop, ptr)) = maybe_property {
                pointer = ptr;
                properties.push(prop);
            } else {
                return Err(QueryParseError {
                    string,
                    pointer: pointer + 1,
                    error_type: QueryErrorType::SyntaxError,
                });
            }
        }

        Ok(Self { string, properties })
    }
}
