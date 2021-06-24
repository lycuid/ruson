pub type Stack = Vec<char>;
pub type Pointer = usize;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub const MINROW: usize = 1;
    pub const MINCOL: usize = 1;

    pub fn new() -> Self {
        Self {
            row: Self::MINROW,
            col: Self::MINCOL,
        }
    }
}

pub fn parse_while<F: FnMut(&char) -> bool>(
    xs: &Stack,
    xi: &Pointer,
    mut f: F,
) -> (String, Pointer) {
    let string: String = xs[*xi..].iter().take_while(|c| f(c)).collect();
    let count = xi + string.chars().count();
    (string, count)
}

pub fn parse_char(x: char, xs: &Stack, xi: &Pointer) -> Option<char> {
    if let Some(&ch) = xs.get(*xi) {
        if x == ch {
            return Some(x);
        }
    }
    None
}

pub fn parse_string(
    ys: &str,
    xs: &Stack,
    xi: &Pointer,
) -> Option<(String, usize)> {
    let mut cs = ys.chars();
    let mut next_index: usize = *xi;

    if next_index > xs.len() - 1 {
        return None;
    }

    while let Some(c) = cs.next() {
        if let Some(&x) = xs.get(next_index) {
            if c != x {
                return None;
            }
        }
        next_index += 1;
    }

    Some((String::from(ys), next_index))
}

pub fn parse_int(xs: &Stack, xi: &Pointer) -> Option<(i32, usize)> {
    let (mut number, next_index) =
        parse_string("-", xs, xi).unwrap_or((String::new(), *xi));

    let (n, _) =
        parse_while(xs, &next_index, |&c| c as u8 >= 48 && c as u8 <= 57);
    number.extend(n.chars());

    number
        .parse()
        .and_then(|int| Ok((int, xi + number.len())))
        .ok()
}

pub fn no_of_digits(mut n: i32) -> i32 {
    let mut digits = 0;
    while n > 0 {
        n /= 10;
        digits += 1;
    }
    digits
}
