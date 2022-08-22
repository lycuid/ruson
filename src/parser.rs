//! Text parsing utility struct.
pub type Stack = Vec<char>;
pub type Cursor = usize;

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

#[derive(Debug)]
pub struct Parser {
    pub stack: Stack,
    pub cursor: Cursor,
}

impl Parser {
    pub fn new(s: &str) -> Self {
        Self {
            stack: s.chars().collect(),
            cursor: 0,
        }
    }

    pub fn peek(&self) -> Option<&char> {
        self.peek_at(self.cursor)
    }

    pub fn peek_at(&self, cursor: Cursor) -> Option<&char> {
        self.stack.get(cursor)
    }

    pub fn parse_while<F: FnMut(&char) -> bool>(&mut self, mut f: F) -> String {
        let string: String = self.stack[self.cursor..]
            .iter()
            .take_while(|&ch| (f)(ch))
            .collect();
        self.cursor += string.chars().count();
        string
    }

    pub fn parse_byte(&mut self, x: char) -> Option<char> {
        if let Some(&ch) = self.peek() {
            if x == ch {
                self.cursor += 1;
                return Some(x);
            }
        }
        None
    }

    pub fn parse_string(&mut self, ys: &str) -> Option<String> {
        let mut cs = ys.chars();
        let mut next_index: usize = self.cursor;
        while let Some(c) = cs.next() {
            if let Some(&x) = self.stack.get(next_index) {
                if c != x {
                    return None;
                }
            }
            next_index += 1;
        }
        self.cursor = next_index;
        Some(ys.into())
    }

    pub fn parse_uint(&mut self) -> Option<u32> {
        self.parse_while(|&ch| ch.is_digit(10)).parse().ok()
    }

    pub fn parse_int(&mut self) -> Option<i32> {
        let mul = self.parse_byte('-').and(Some(-1)).unwrap_or(1);
        self.parse_uint().and_then(|n| Some(n as i32 * mul))
    }

    #[inline]
    pub fn get_string(&self) -> String {
        self.stack.iter().collect()
    }

    #[inline]
    pub fn position(&self, cursor: Cursor) -> Position {
        let string: String = self.stack.iter().take(cursor).collect();

        Position {
            row: string.lines().count(),
            col: string.lines().last().unwrap_or("").len(),
        }
    }
}
