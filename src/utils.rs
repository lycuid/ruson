pub trait Formatter {
    type Token;
    fn dump(&self, token: &Self::Token) -> String;
}

pub fn get_version(configs: &'static str) -> String {
    let line = configs
        .lines()
        .filter(|&line| line.contains("version"))
        .next()
        .unwrap_or(r#"version = "0.1.0""#);

    line.chars()
        .skip_while(|&ch| ch != '"')
        .skip(1)
        .take_while(|&ch| ch != '"')
        .collect()
}
