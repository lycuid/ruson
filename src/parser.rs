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

#[derive(Debug)]
pub struct Parser {
    pub stack: Stack,
    pub pointer: Pointer,
}

impl Parser {
    pub fn new(s: &str) -> Self {
        Self {
            stack: s.chars().collect(),
            pointer: 0,
        }
    }

    pub fn peek(&self) -> Option<&char> {
        self.peek_at(self.pointer)
    }

    pub fn peek_at(&self, ptr: Pointer) -> Option<&char> {
        self.stack.get(ptr)
    }

    pub fn match_while<F: FnMut(&char) -> bool>(&mut self, mut f: F) -> String {
        let string: String = self.stack[self.pointer..]
            .iter()
            .take_while(|&ch| (f)(ch))
            .collect();
        self.pointer += string.chars().count();
        string
    }

    pub fn match_char(&mut self, x: char) -> Option<char> {
        if let Some(&ch) = self.peek() {
            if x == ch {
                self.pointer += 1;
                return Some(x);
            }
        }
        None
    }

    pub fn match_string(&mut self, ys: &str) -> Option<String> {
        let mut cs = ys.chars();
        let mut next_index: usize = self.pointer;

        while let Some(c) = cs.next() {
            if let Some(&x) = self.stack.get(next_index) {
                if c != x {
                    return None;
                }
            }
            next_index += 1;
        }
        self.pointer = next_index;

        Some(ys.into())
    }

    pub fn parse_uint(&mut self) -> Option<u32> {
        self.match_while(|&ch| ch.is_digit(10)).parse().ok()
    }

    pub fn parse_int(&mut self) -> Option<i32> {
        let mul = self.match_char('-').and(Some(-1)).unwrap_or(1);

        self.parse_uint().and_then(|n| Some(n as i32 * mul))
    }

    pub fn get_string(&self) -> String {
        self.stack.iter().collect()
    }

    pub fn position(&self, ptr: Pointer) -> Position {
        let string: String = self.stack.iter().take(ptr).collect();

        Position {
            row: string.lines().count(),
            col: string.lines().last().unwrap_or("").len(),
        }
    }
}
