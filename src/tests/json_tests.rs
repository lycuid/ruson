#[cfg(test)]
mod json_tests {
    use crate::json::{
        error::JsonErrorType, token::JsonToken, JsonParser, ParseResult,
    };

    fn assert_result(json_parser: &ParseResult, field: &str) {
        match json_parser {
            Ok(_) => {}
            Err(_) => println!("> JsonParser Error at \"{}\"", field),
        }
    }

    #[test]
    fn sucess_null() {
        let mut json_parser = JsonParser::new("null");
        assert_result(&json_parser.null(), "null");
        assert_eq!(json_parser.token.unwrap(), JsonToken::Null);
    }

    #[test]
    fn error_null() {
        let mut json_parser: JsonParser;
        for xs in ["Null", "NULL"].iter() {
            json_parser = JsonParser::new(xs);
            match &json_parser.null() {
                Ok(_) => assert!(false),
                Err((ref error_type, _)) => {
                    assert_eq!(error_type, &JsonErrorType::SyntaxError)
                }
            };
        }
    }

    #[test]
    fn sucess_bool() {
        let mut json_parser = JsonParser::new("true");
        assert_result(&json_parser.boolean(), "true");
        assert_eq!(json_parser.token.unwrap(), JsonToken::Boolean(true));

        let mut json_parser = JsonParser::new("false");
        assert_result(&json_parser.boolean(), "false");
        assert_eq!(json_parser.token.unwrap(), JsonToken::Boolean(false));
    }

    #[test]
    fn error_bool() {
        let mut json_parser: JsonParser;
        for xs in ["False", "True"].iter() {
            json_parser = JsonParser::new(xs);
            match &json_parser.boolean() {
                Ok(_) => assert!(false),
                Err((error_type, _)) => {
                    assert_eq!(error_type, &JsonErrorType::SyntaxError)
                }
            };
        }
    }

    #[test]
    fn sucess_number() {
        let mut json_parser: JsonParser;
        for (xs, j) in [
            ("10", JsonToken::Number(10.0)),
            ("-91", JsonToken::Number(-91.0)),
            ("-9823.0", JsonToken::Number(-9823.0)),
            ("0.9832", JsonToken::Number(0.9832)),
            ("-1.8923", JsonToken::Number(-1.8923)),
            ("40.2", JsonToken::Number(40.2)),
            ("40.", JsonToken::Number(40.0)),
            ("40 ", JsonToken::Number(40.0)),
        ]
        .iter()
        {
            json_parser = JsonParser::new(xs);
            assert_result(&json_parser.number(), xs);
            assert_eq!(json_parser.token.unwrap(), *j);
        }
    }

    #[test]
    fn error_number() {
        let mut json_parser: JsonParser;
        for number in [".10", "-.10"].iter() {
            json_parser = JsonParser::new(number);
            match &json_parser.number() {
                Ok(_) => assert!(false),
                Err((error_type, _)) => {
                    assert_eq!(error_type, &JsonErrorType::SyntaxError)
                }
            };
        }
    }

    #[test]
    fn sucess_string() {
        let mut json_parser: JsonParser;
        for (xs, j) in [
            (r#""string""#, JsonToken::QString(String::from("string"))),
            (
                r#""string with spaces""#,
                JsonToken::QString(String::from("string with spaces")),
            ),
            (
                r#""string with 'quotes'""#,
                JsonToken::QString(String::from("string with 'quotes'")),
            ),
            (
                r#""string with \"escaped double quotes\"""#,
                JsonToken::QString(String::from(
                    "string with \\\"escaped double quotes\\\"",
                )),
            ),
        ]
        .iter()
        {
            json_parser = JsonParser::new(xs);
            assert_result(&json_parser.string(), xs);
            assert_eq!(json_parser.token.unwrap(), *j);
        }
    }

    #[test]
    fn error_string() {
        let mut json_parser: JsonParser;
        for string in [r#"klasd"#, r#""#].iter() {
            json_parser = JsonParser::new(string);
            match &json_parser.string() {
                Ok(_) => assert!(false),
                Err((error_type, _)) => {
                    assert_eq!(error_type, &JsonErrorType::SyntaxError)
                }
            };
        }
    }

    #[test]
    fn sucess_array() {
        let xs = r#"["string", null, 1.03, true]"#;
        let mut json_parser = JsonParser::new(xs);
        assert_result(&json_parser.array(), xs);
        assert_eq!(
            json_parser.token.take().unwrap(),
            JsonToken::Array(vec![
                JsonToken::QString(String::from("string")),
                JsonToken::Null,
                JsonToken::Number(1.03),
                JsonToken::Boolean(true)
            ])
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
            match &json_parser.array() {
                Ok(_) => assert!(false),
                Err((error_type, _)) => assert_eq!(error_type, err),
            };
        }
    }

    #[test]
    fn sucess_object() {
        let xs = r#"{
            "key1": "string",
            "key2": null,
            "key3": 1.03,
            "key4": true
        }"#;
        let mut json_parser = JsonParser::new(xs);

        assert_result(&json_parser.object(), xs);
        let mut map = std::collections::HashMap::new();
        map.insert(
            String::from("key1"),
            JsonToken::QString(String::from("string")),
        );
        map.insert(String::from("key2"), JsonToken::Null);
        map.insert(String::from("key3"), JsonToken::Number(1.03));
        map.insert(String::from("key4"), JsonToken::Boolean(true));
        assert_eq!(json_parser.token.take().unwrap(), JsonToken::Object(map));
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
            match &json_parser.object() {
                Ok(_) => assert!(false),
                Err((error_type, _)) => assert_eq!(error_type, err),
            };
        }
    }
}
