use crate::json::{query::JsonQuery, token::Property};

macro_rules! query {
    ($($prop:expr),*) => {
        JsonQuery([$($prop),*].iter().cloned().collect())
    };
}

#[test]
fn success_query() {
    let string = r#"[1].array.map(.obj.list.keys())[0].values()["property"].another_property["another_array"][90].length()"#;
    let query1 = query![
        Property::Index(1),
        Property::Dot("array".into()),
        Property::Map(query![
            Property::Dot("obj".into()),
            Property::Dot("list".into()),
            Property::Keys
        ]),
        Property::Index(0),
        Property::Values,
        Property::Bracket("property".into()),
        Property::Dot("another_property".into()),
        Property::Bracket("another_array".into()),
        Property::Index(90),
        Property::Length
    ];

    let query2 = JsonQuery::new(string);
    assert!(query2.is_ok());
    assert_eq!(query2.unwrap(), query1);
}
