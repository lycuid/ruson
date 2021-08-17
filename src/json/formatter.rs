use super::token::JsonToken;

#[derive(Debug, PartialEq)]
pub enum JsonFormat {
    Raw,
    Pretty,
    Table,
}

pub struct JsonFormatter<'a> {
    token: &'a JsonToken,
    with: JsonFormat,
}

impl<'a> JsonFormatter<'a> {
    pub fn new(token: &'a JsonToken) -> Self {
        Self {
            token,
            with: JsonFormat::Raw,
        }
    }

    pub fn with(&mut self, with: JsonFormat) -> &mut Self {
        self.with = with;
        self
    }

    fn prettified(
        f: &mut std::fmt::Formatter,
        token: &JsonToken,
        depth: usize,
    ) -> std::fmt::Result {
        match token {
            JsonToken::Array(tokens) => {
                let mut tokens = tokens.iter();

                writeln!(f, "[")?;
                if let Some(token) = tokens.next() {
                    write!(f, "{}", Self::indented(depth + 1, &""))?;
                    Self::prettified(f, token, depth + 1)?
                }

                for token in tokens {
                    writeln!(f, ",",)?;
                    write!(f, "{}", Self::indented(depth + 1, &""))?;
                    Self::prettified(f, token, depth + 1)?
                }
                writeln!(f, "")?;
                write!(f, "{}", Self::indented(depth, &"]"))
            }
            JsonToken::Object(pairs) => {
                let mut pairs = pairs.iter();

                writeln!(f, "{{")?;
                if let Some((key, token)) = pairs.next() {
                    write!(
                        f,
                        "{}: ",
                        Self::indented(
                            depth + 1,
                            &JsonToken::QString(key.into())
                        )
                    )?;
                    Self::prettified(f, token, depth + 1)?
                }

                for (key, token) in pairs {
                    writeln!(f, ",",)?;
                    write!(
                        f,
                        "{}: ",
                        Self::indented(
                            depth + 1,
                            &JsonToken::QString(key.into())
                        )
                    )?;
                    Self::prettified(f, token, depth + 1)?
                }
                writeln!(f, "")?;
                write!(f, "{}", Self::indented(depth, &"}"))
            }
            _ => write!(f, "{}", token),
        }
    }

    fn indented(depth: usize, s: &dyn std::fmt::Display) -> String {
        format!("{}{}", vec!["\t"; depth].join(""), s)
    }
}

impl<'a> std::fmt::Display for JsonFormatter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.with {
            JsonFormat::Raw => write!(f, "{}", self.token),
            JsonFormat::Pretty => Self::prettified(f, self.token, 0),
            JsonFormat::Table => match self.token {
                JsonToken::Array(array) => {
                    let mut iter = array.iter();
                    if let Some(value) = iter.next() {
                        write!(f, "{}", value)?;
                    }
                    while let Some(value) = iter.next() {
                        write!(f, "\n{}", value)?;
                    }
                    Ok(())
                }
                JsonToken::Object(map) => {
                    let mut iter = map.iter();
                    if let Some((key, value)) = iter.next() {
                        write!(f, "{}\t{}", key, value)?;
                    }
                    while let Some((key, value)) = iter.next() {
                        write!(f, "\n{}\t{}", key, value)?;
                    }
                    Ok(())
                }
                _ => write!(f, "{}", self.token),
            },
        }
    }
}
