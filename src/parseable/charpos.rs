use std::fmt;

// Keeps track of line and character position based on
// what characters are being processed

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
            '\r' => self.chr = 1,
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

    #[test]
    fn skip_ln() {
        let mut c = CharPos::new();
        c.skip('\n');
        assert_eq!("2:1", format!("{}", c));
    }

    #[test]
    fn skip_char() {
        let mut c = CharPos::new();
        c.skip('f');
        assert_eq!("1:2", format!("{}", c));
    }

    #[test]
    fn skip_ret() {
        let mut c = CharPos::new();
        c.skip('a');
        c.skip('\r');
        assert_eq!("1:1", format!("{}", c));
    }
}

