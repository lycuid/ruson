pub trait Formatter {
    type Token;
    fn dump(&self, token: &Self::Token) -> String;
}

pub fn total_digits(mut n: i32) -> i32 {
    let mut digits = 0;
    while n > 0 {
        n /= 10;
        digits += 1;
    }
    digits
}
