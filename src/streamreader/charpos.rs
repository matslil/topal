use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharPos {
    line: usize,
    chr: usize,
}

impl CharPos {
    pub fn new() -> Self {
        Self {
            line: 1,
            chr:  1,
        }
    }

    pub fn skip(&self, c: char) {
        match c {
            '\t' => self.pos.chr += 8,
            '\n' => {
                self.pos.line += 1;
                self.pos.chr = 1;
            },
            _    => self.pos.char += 1,
        };
    }
}

impl fmt::Display for CharPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.chr)
    }
}

