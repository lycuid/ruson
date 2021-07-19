//! Error formatting utilities.
pub trait RusonResult<T> {
    fn unwrap_or_exit(self) -> T;
    fn unwrap_or_exit_with(self, exit_code: i32) -> T;
}

impl<T, E: std::fmt::Display> RusonResult<T> for Result<T, E> {
    fn unwrap_or_exit(self) -> T {
        self.unwrap_or_exit_with(1)
    }

    fn unwrap_or_exit_with(self, exit_code: i32) -> T {
        match self {
            Ok(t) => t,
            Err(displayable) => {
                let exit_string = format!("{}", displayable).errorfmt();

                match exit_code {
                    0 => {
                        println!("{}", exit_string);
                    }
                    2 => {
                        let bin = std::env::args().next().unwrap();
                        eprintln!("{}", exit_string);
                        eprintln!("Try '{} --help' for more information.", bin);
                    }
                    _ => {
                        eprintln!("{}", exit_string);
                    }
                };

                std::process::exit(exit_code);
            }
        }
    }
}

pub trait ErrorString {
    const SUBSTR_WIDTH: usize;

    fn shorten(&self, start: usize) -> Self;
    fn uncamelize(&self) -> Self;
    fn errorfmt(&self) -> Self;
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

    fn uncamelize(&self) -> Self {
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

    fn errorfmt(&self) -> Self {
        format!("{}: {}", std::env::args().next().unwrap(), self)
    }
}
