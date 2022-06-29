//! Json Formatter: can call `dump()`, returns string of formatted json token.
use super::token::Json;

pub trait Formatter {
    type Token;
    fn dump(&self, token: &Self::Token) -> String;
}

pub struct RawJson {}

impl Formatter for RawJson {
    type Token = Json;
    fn dump(&self, token: &Self::Token) -> String {
        format!("{}", token)
    }
}

pub struct PrettyJson<'a> {
    pub indent: &'a str,
}

impl<'a> PrettyJson<'a> {
    fn prettified(&self, s: &mut String, token: &Json, depth: usize) {
        match token {
            Json::Array(tokens) => {
                let mut tokens = tokens.iter();

                s.push_str("[\n");
                if let Some(token) = tokens.next() {
                    s.push_str(&format!("{}", self.indented(depth + 1, &"")));
                    self.prettified(s, token, depth + 1);
                }

                for token in tokens {
                    s.push_str(&format!(
                        ",\n{}",
                        self.indented(depth + 1, &"")
                    ));
                    self.prettified(s, token, depth + 1);
                }
                s.push_str(&format!("\n{}", self.indented(depth, &"]")));
            }
            Json::Object(pairs) => {
                let mut pairs = pairs.iter();

                s.push_str("{\n");
                if let Some((key, token)) = pairs.next() {
                    s.push_str(&format!(
                        "{}: ",
                        self.indented(depth + 1, &Json::QString(key.into()))
                    ));
                    self.prettified(s, token, depth + 1);
                }

                for (key, token) in pairs {
                    s.push_str(&format!(
                        ",\n{}: ",
                        self.indented(depth + 1, &Json::QString(key.into()))
                    ));
                    self.prettified(s, token, depth + 1)
                }
                s.push_str(&format!("\n{}", self.indented(depth, &"}")));
            }
            _ => s.push_str(&format!("{}", token)),
        }
    }

    fn indented(&self, depth: usize, s: &dyn std::fmt::Display) -> String {
        format!("{}{}", vec![self.indent; depth].join(""), s)
    }
}

impl<'a> Formatter for PrettyJson<'a> {
    type Token = Json;
    fn dump(&self, token: &Self::Token) -> String {
        let mut string = String::new();
        self.prettified(&mut string, token, 0);
        string
    }
}

pub struct TableJson {}

impl Formatter for TableJson {
    type Token = Json;
    fn dump(&self, token: &Self::Token) -> String {
        match token {
            Json::Array(array) => {
                let mut string = String::new();
                let mut iter = array.iter();
                if let Some(value) = iter.next() {
                    string.push_str(&format!("{}", value));
                }
                while let Some(value) = iter.next() {
                    string.push_str(&format!("\n{}", value));
                }
                string
            }
            Json::Object(map) => {
                let mut string = String::new();
                let mut iter = map.iter();
                if let Some((key, value)) = iter.next() {
                    string.push_str(&format!("{}\t{}", key, value));
                }
                while let Some((key, value)) = iter.next() {
                    string.push_str(&format!("\n{}\t{}", key, value));
                }
                string
            }
            _ => format!("{}", token),
        }
    }
}
