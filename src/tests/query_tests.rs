use crate::json::{query::JsonQuery, token::JsonProperty};

#[test]
fn success_query() {
    let string =
        r#"data[1].array[0]["property"].another_property["another_array"][90]"#;
    let props = vec![
        JsonProperty::Index(1),
        JsonProperty::Dot("array".into()),
        JsonProperty::Index(0),
        JsonProperty::Bracket("property".into()),
        JsonProperty::Dot("another_property".into()),
        JsonProperty::Bracket("another_array".into()),
        JsonProperty::Index(90),
    ];

    let query = JsonQuery::new(string);
    assert!(query.is_ok());
    assert_eq!(query.unwrap().properties, props);
}
