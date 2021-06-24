pub trait RusonResult {
    type Item;
    fn or_exit(self) -> Self::Item;
    fn or_exit_with(self, exit_code: i32) -> Self::Item;
}

impl<T, E: std::fmt::Display> RusonResult for Result<T, E> {
    type Item = T;

    fn or_exit(self) -> Self::Item {
        self.or_exit_with(1)
    }

    fn or_exit_with(self, exit_code: i32) -> Self::Item {
        match self {
            Ok(t) => t,
            Err(error) => {
                eprintln!("ruson: {}", error);
                std::process::exit(exit_code);
            }
        }
    }
}

pub trait ErrorString {
    const SUBSTR_WIDTH: usize;

    fn shorten(&self, start: usize) -> Self;
    fn uncamelized(&self) -> Self;
}

impl ErrorString for String {
    const SUBSTR_WIDTH: usize = 50;

    fn shorten(&self, start: usize) -> Self {
        if self.len() > Self::SUBSTR_WIDTH {
            self.chars()
                .skip(start)
                .take(Self::SUBSTR_WIDTH)
                .enumerate()
                .map(|(index, value)| if index < 2 { '.' } else { value })
                .collect()
        } else {
            self.to_owned()
        }
    }

    fn uncamelized(&self) -> Self {
        let mut new_string = String::new();
        let mut chars = self.chars();

        new_string.push(chars.next().unwrap_or(' '));
        for ch in chars {
            if ch.is_ascii_uppercase() {
                new_string.extend([' ', ch].iter());
            } else {
                new_string.push(ch);
            }
        }

        new_string
    }
}
