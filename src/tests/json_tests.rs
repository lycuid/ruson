use crate::json::{error::JsonErrorType, lexer::JsonLexer, token::Json};

#[test]
fn success_null() {
    let mut json_lexer = JsonLexer::new("null");
    assert_eq!(json_lexer.next_null().unwrap(), Json::Null);
}

#[test]
fn error_null() {
    let mut json_lexer: JsonLexer;
    for xs in ["Null", "NULL"].iter() {
        json_lexer = JsonLexer::new(xs);
        match &json_lexer.next_null() {
            Ok(_) => assert!(false),
            Err((ref error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_bool() {
    let mut json_lexer = JsonLexer::new("true");
    assert_eq!(json_lexer.next_boolean().unwrap(), Json::Boolean(true));

    let mut json_lexer = JsonLexer::new("false");
    assert_eq!(json_lexer.next_boolean().unwrap(), Json::Boolean(false));
}

#[test]
fn error_bool() {
    let mut json_lexer: JsonLexer;
    for xs in ["False", "True"].iter() {
        json_lexer = JsonLexer::new(xs);
        match &json_lexer.next_boolean() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_number() {
    let mut json_lexer: JsonLexer;
    for (xs, j) in [
        ("10", Json::Number(10.0)),
        ("-91", Json::Number(-91.0)),
        ("-9823.0", Json::Number(-9823.0)),
        ("0.9832", Json::Number(0.9832)),
        ("-1.8923", Json::Number(-1.8923)),
        ("40.2", Json::Number(40.2)),
        ("40.", Json::Number(40.0)),
        ("40 ", Json::Number(40.0)),
        ("-2.12e+12", Json::Number(-2.12e+12)),
        ("-2.12e-12", Json::Number(-2.12e-12)),
        ("-2.12e12", Json::Number(-2.12e12)),
        ("2.12E+12", Json::Number(2.12e+12)),
        ("2.12E-12", Json::Number(2.12E-12)),
        ("2.12E12", Json::Number(2.12E12)),
    ]
    .iter()
    {
        json_lexer = JsonLexer::new(xs);
        assert_eq!(json_lexer.next_number().unwrap(), *j);
    }
}

#[test]
fn error_number() {
    let mut json_lexer: JsonLexer;
    for number in [
        ".10",
        "-.10",
        "4.873e+-23",
        "4.873e-+23",
        "4.873E+-23",
        "4.873E-+23",
    ]
    .iter()
    {
        json_lexer = JsonLexer::new(number);
        match &json_lexer.next_number() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_string() {
    let mut json_lexer: JsonLexer;
    for (xs, j) in [
        (r#""string""#, Json::QString("string".into())),
        (
            r#""string with spaces""#,
            Json::QString("string with spaces".into()),
        ),
        (
            r#""string with 'quotes'""#,
            Json::QString("string with 'quotes'".into()),
        ),
        (
            r#""string with \"escaped double quotes\"""#,
            Json::QString("string with \\\"escaped double quotes\\\"".into()),
        ),
    ]
    .iter()
    {
        json_lexer = JsonLexer::new(xs);
        assert_eq!(json_lexer.next_qstring().unwrap(), *j);
    }
}

#[test]
fn error_string() {
    let mut json_lexer: JsonLexer;
    for string in [r#"klasd"#, r#""#].iter() {
        json_lexer = JsonLexer::new(string);
        match &json_lexer.next_qstring() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_array() {
    let xs = r#"["string", null, 1.03, true]"#;
    let mut json_lexer = JsonLexer::new(xs);
    assert_eq!(
        json_lexer.next_array().unwrap(),
        Json::Array(vec![
            Json::QString("string".into()),
            Json::Null,
            Json::Number(1.03),
            Json::Boolean(true)
        ])
    );
}

#[test]
fn error_array() {
    let mut json_lexer: JsonLexer;
    for (xs, err) in [
        // multple trailing commas.
        (r#"[1, 2, 3,]"#, JsonErrorType::TrailingCommaError),
        (r#"[1, 2, ,]"#, JsonErrorType::TrailingCommaError),
        // leading commas with empty array.
        (r#"[, ,   ,,,]"#, JsonErrorType::SyntaxError),
        // leading comma with valid array.
        (r#"[,1, 2]"#, JsonErrorType::SyntaxError),
    ]
    .iter()
    {
        json_lexer = JsonLexer::new(xs);
        match &json_lexer.next_array() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => assert_eq!(error_type, err),
        };
    }
}

#[test]
fn success_object() {
    let xs = r#"{
        "key1": "string",
        "key2": null,
        "key3": 1.03,
        "key4": true
    }"#;
    let mut json_lexer = JsonLexer::new(xs);

    let mut map = std::collections::HashMap::new();
    map.insert("key1".into(), Json::QString("string".into()));
    map.insert("key2".into(), Json::Null);
    map.insert("key3".into(), Json::Number(1.03));
    map.insert("key4".into(), Json::Boolean(true));
    assert_eq!(json_lexer.next_object().unwrap(), Json::Object(map));
}

#[test]
fn error_object() {
    let mut json_lexer: JsonLexer;
    for (xs, err) in [
        // single trailing comma.
        (
            r#"{ "key1": "string", "key4": true, }"#,
            JsonErrorType::TrailingCommaError,
        ),
        // multiple trailig commas,
        (
            r#"{ "key1": "string", "key4": true, , }"#,
            JsonErrorType::TrailingCommaError,
        ),
        // missing value.
        (
            r#"{ "key1": "string", "key4": , }"#,
            JsonErrorType::SyntaxError,
        ),
        // missing colon.
        (
            r#"{ "key1": "string", "key4" true }"#,
            JsonErrorType::SyntaxError,
        ),
        // leading comma (missing 'key -> colon -> value').
        (
            r#"{ ,"key1": "string", "key4": true, , }"#,
            JsonErrorType::SyntaxError,
        ),
        // comma after key (missing 'colon -> value').
        (
            r#"{ "key1", : "string", "key4": true, , }"#,
            JsonErrorType::SyntaxError,
        ),
    ]
    .iter()
    {
        json_lexer = JsonLexer::new(xs);
        match &json_lexer.next_object() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => assert_eq!(error_type, err),
        };
    }
}
