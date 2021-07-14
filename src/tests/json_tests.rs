use crate::json::{error::JsonErrorType, token::JsonToken, JsonTokenLexer};

#[test]
fn success_null() {
    let mut json_lexer = JsonTokenLexer::new("null");
    assert_eq!(json_lexer.null().unwrap(), JsonToken::Null);
}

#[test]
fn error_null() {
    let mut json_lexer: JsonTokenLexer;
    for xs in ["Null", "NULL"].iter() {
        json_lexer = JsonTokenLexer::new(xs);
        match &json_lexer.null() {
            Ok(_) => assert!(false),
            Err((ref error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_bool() {
    let mut json_lexer = JsonTokenLexer::new("true");
    assert_eq!(json_lexer.boolean().unwrap(), JsonToken::Boolean(true));

    let mut json_lexer = JsonTokenLexer::new("false");
    assert_eq!(json_lexer.boolean().unwrap(), JsonToken::Boolean(false));
}

#[test]
fn error_bool() {
    let mut json_lexer: JsonTokenLexer;
    for xs in ["False", "True"].iter() {
        json_lexer = JsonTokenLexer::new(xs);
        match &json_lexer.boolean() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_number() {
    let mut json_lexer: JsonTokenLexer;
    for (xs, j) in [
        ("10", JsonToken::Number(10.0)),
        ("-91", JsonToken::Number(-91.0)),
        ("-9823.0", JsonToken::Number(-9823.0)),
        ("0.9832", JsonToken::Number(0.9832)),
        ("-1.8923", JsonToken::Number(-1.8923)),
        ("40.2", JsonToken::Number(40.2)),
        ("40.", JsonToken::Number(40.0)),
        ("40 ", JsonToken::Number(40.0)),
        ("-2.12e+12", JsonToken::Number(-2.12e+12)),
        ("-2.12e-12", JsonToken::Number(-2.12e-12)),
        ("-2.12e12", JsonToken::Number(-2.12e12)),
        ("2.12E+12", JsonToken::Number(2.12e+12)),
        ("2.12E-12", JsonToken::Number(2.12E-12)),
        ("2.12E12", JsonToken::Number(2.12E12)),
    ]
    .iter()
    {
        json_lexer = JsonTokenLexer::new(xs);
        assert_eq!(json_lexer.number().unwrap(), *j);
    }
}

#[test]
fn error_number() {
    let mut json_lexer: JsonTokenLexer;
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
        json_lexer = JsonTokenLexer::new(number);
        match &json_lexer.number() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_string() {
    let mut json_lexer: JsonTokenLexer;
    for (xs, j) in [
        (r#""string""#, JsonToken::QString("string".into())),
        (
            r#""string with spaces""#,
            JsonToken::QString("string with spaces".into()),
        ),
        (
            r#""string with 'quotes'""#,
            JsonToken::QString("string with 'quotes'".into()),
        ),
        (
            r#""string with \"escaped double quotes\"""#,
            JsonToken::QString(
                "string with \\\"escaped double quotes\\\"".into(),
            ),
        ),
    ]
    .iter()
    {
        json_lexer = JsonTokenLexer::new(xs);
        assert_eq!(json_lexer.qstring().unwrap(), *j);
    }
}

#[test]
fn error_string() {
    let mut json_lexer: JsonTokenLexer;
    for string in [r#"klasd"#, r#""#].iter() {
        json_lexer = JsonTokenLexer::new(string);
        match &json_lexer.qstring() {
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
    let mut json_lexer = JsonTokenLexer::new(xs);
    assert_eq!(
        json_lexer.array().unwrap(),
        JsonToken::Array(vec![
            JsonToken::QString("string".into()),
            JsonToken::Null,
            JsonToken::Number(1.03),
            JsonToken::Boolean(true)
        ])
    );
}

#[test]
fn error_array() {
    let mut json_lexer: JsonTokenLexer;
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
        json_lexer = JsonTokenLexer::new(xs);
        match &json_lexer.array() {
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
    let mut json_lexer = JsonTokenLexer::new(xs);

    let mut map = std::collections::HashMap::new();
    map.insert("key1".into(), JsonToken::QString("string".into()));
    map.insert("key2".into(), JsonToken::Null);
    map.insert("key3".into(), JsonToken::Number(1.03));
    map.insert("key4".into(), JsonToken::Boolean(true));
    assert_eq!(json_lexer.object().unwrap(), JsonToken::Object(map));
}

#[test]
fn error_object() {
    let mut json_lexer: JsonTokenLexer;
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
        json_lexer = JsonTokenLexer::new(xs);
        match &json_lexer.object() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => assert_eq!(error_type, err),
        };
    }
}
