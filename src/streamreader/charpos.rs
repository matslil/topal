use std::fmt;

const SPACES_PER_TAB: usize = 8;

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

    pub fn skip(&mut self, c: char) {
        match c {
            '\t' => self.chr += SPACES_PER_TAB,
            '\n' => {
                self.line += 1;
                self.chr = 1;
            },
            _    => self.chr += 1,
        };
    }
}

impl fmt::Display for CharPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.chr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_tab() {
        let mut c = CharPos::new();
        c.skip('\t');
        assert_eq!("1:9", format!("{}", c));
    }
}

