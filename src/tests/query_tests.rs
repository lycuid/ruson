use crate::json::{query::JsonQuery, token::JsonProperty};

#[test]
fn success_query() {
    let string = r#"[1].array.map(.obj.list.keys())[0].values()["property"].another_property["another_array"][90].length()"#;
    let query1 = JsonQuery::from(vec![
        JsonProperty::Index(1),
        JsonProperty::Dot("array".into()),
        JsonProperty::Map(JsonQuery::from(vec![
            JsonProperty::Dot("obj".into()),
            JsonProperty::Dot("list".into()),
            JsonProperty::Keys,
        ])),
        JsonProperty::Index(0),
        JsonProperty::Values,
        JsonProperty::Bracket("property".into()),
        JsonProperty::Dot("another_property".into()),
        JsonProperty::Bracket("another_array".into()),
        JsonProperty::Index(90),
        JsonProperty::Length,
    ]);

    let query2 = JsonQuery::new(string);
    assert!(query2.is_ok());
    assert_eq!(query2.unwrap(), query1);
}
