use crate::json::{error::JsonErrorType, parser::JsonParser, token::Json};

macro_rules! json {
    ()                           => { Json::Null };
    (true)                       => { Json::Boolean(true) };
    (false)                      => { Json::Boolean(false) };
    ($str:literal)               => { Json::QString($str.into()) };
    ($($item:expr),*)            => { Json::Array(vec![$($item),*]) };
    ($($k:literal => $v:expr),*) => {
        Json::Object(std::collections::HashMap::from([$(($k.into(), $v)),*]))
    };
}

#[test]
fn success_null() {
    let mut json_parser = JsonParser::new("null");
    assert_eq!(json_parser.parse_null().unwrap(), json!());
}

#[test]
fn error_null() {
    let mut json_parser: JsonParser;
    for xs in ["Null", "NULL"].iter() {
        json_parser = JsonParser::new(xs);
        match &json_parser.parse_null() {
            Ok(_) => assert!(false),
            Err((ref error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_bool() {
    let mut json_parser = JsonParser::new("true");
    assert_eq!(json_parser.parse_boolean().unwrap(), json!(true));

    let mut json_parser = JsonParser::new("false");
    assert_eq!(json_parser.parse_boolean().unwrap(), json!(false));
}

#[test]
fn error_bool() {
    let mut json_parser: JsonParser;
    for xs in ["False", "True"].iter() {
        json_parser = JsonParser::new(xs);
        match &json_parser.parse_boolean() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_number() {
    let mut json_parser: JsonParser;
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
        json_parser = JsonParser::new(xs);
        assert_eq!(json_parser.parse_number().unwrap(), *j);
    }
}

#[test]
fn error_number() {
    let mut json_parser: JsonParser;
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
        json_parser = JsonParser::new(number);
        match &json_parser.parse_number() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => {
                assert_eq!(error_type, &JsonErrorType::SyntaxError)
            }
        };
    }
}

#[test]
fn success_string() {
    let mut json_parser: JsonParser;
    for (xs, j) in [
        (r#""string""#, json!("string")),
        (r#""string with spaces""#, json!("string with spaces")),
        (r#""string with 'quotes'""#, json!("string with 'quotes'")),
        (
            r#""string with \"escaped double quotes\"""#,
            json!("string with \\\"escaped double quotes\\\""),
        ),
    ]
    .iter()
    {
        json_parser = JsonParser::new(xs);
        assert_eq!(json_parser.parse_qstring().unwrap(), *j);
    }
}

#[test]
fn error_string() {
    let mut json_parser: JsonParser;
    for string in [r#"klasd"#, r#""#].iter() {
        json_parser = JsonParser::new(string);
        match &json_parser.parse_qstring() {
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
    let mut json_parser = JsonParser::new(xs);
    assert_eq!(
        json_parser.parse_array().unwrap(),
        json![json!("string"), json!(), Json::Number(1.03), json!(true)]
    );
}

#[test]
fn error_array() {
    let mut json_parser: JsonParser;
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
        json_parser = JsonParser::new(xs);
        match &json_parser.parse_array() {
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
    let mut json_parser = JsonParser::new(xs);
    assert_eq!(
        json_parser.parse_object().unwrap(),
        json! {
            "key1" => json!("string"),
            "key2" => json!(),
            "key3" => Json::Number(1.03),
            "key4" => json!(true)
        }
    );
}

#[test]
fn error_object() {
    let mut json_parser: JsonParser;
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
        json_parser = JsonParser::new(xs);
        match &json_parser.parse_object() {
            Ok(_) => assert!(false),
            Err((error_type, _)) => assert_eq!(error_type, err),
        };
    }
}
