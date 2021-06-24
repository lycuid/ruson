#[cfg(test)]
mod query_tests {
    use crate::{json::token::JsonProperty, query::Query};

    #[test]
    fn success_query() {
        let string = r#"data[1].array[0]["property"].another_property["another_array"][90]"#;
        let props = vec![
            JsonProperty::Index(1),
            JsonProperty::Dot(String::from("array")),
            JsonProperty::Index(0),
            JsonProperty::Bracket(String::from("property")),
            JsonProperty::Dot(String::from("another_property")),
            JsonProperty::Bracket(String::from("another_array")),
            JsonProperty::Index(90),
        ];

        let query = Query::new(string);
        assert!(query.is_ok());
        assert_eq!(query.unwrap().properties, props);
    }

    // #[test]
    // fn error_query() {
    //     todo!();
    // }
}
