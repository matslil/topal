use std::io::BufRead;
use std::fmt;
use utf8_chars::BufReadCharsExt;
use super::CharPos;
use super::{Parseable, ParseError};

pub struct Stream<T>
where T: BufRead
{
    name: String,
    chr: Option<char>,
    pos: CharPos,
    reader: T,
}

impl<T: BufRead> Stream<T>
{
    pub fn new(buf: T, name: &str) -> Self {
        Self {
            name: name.to_string(),
            chr: None,
            pos: CharPos::new(),
            reader: buf,
        }
    }
}

impl<T: BufRead> fmt::Display for Stream<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.pos)
    }
}

impl<T: BufRead> fmt::Debug for Stream<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.pos)
    }
}

impl<T: BufRead> Parseable for Stream<T> {
    fn take(&mut self) -> Result<char, ParseError> {
        match self.chr.take() {
            Some(c) => {
                self.pos.skip(c);
                Ok(c)
            },
            None => match self.reader.read_char_raw() {
                Ok(Some(c)) => {
                    self.pos.skip(c);
                    Ok(c)
                },
                Ok(None) => Err(ParseError::EOS),
                Err(err) => Err(ParseError::Broken(err.to_string())),
            }
        }
    }
    fn peek(&mut self) -> Result<char, ParseError> {
        match self.chr {
            None => {
                match self.reader.read_char_raw() {
                    Ok(Some(c)) => {
                        self.chr = Some(c);
                        Ok(c)
                    }
                    Ok(None) => Err(ParseError::EOS),
                    Err(err) => Err(ParseError::Broken(err.to_string())),
                }
            }
            Some(c) => Ok(c),
        }
    }
    fn skip(&mut self) -> Result<(), ParseError> {
        match self.take() {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::io;

    const TESTFILE: &str = "First line\nSecond line\n";

    #[test]
    fn peek() {
        let buf = io::BufReader::new(TESTFILE.as_bytes());
        let mut stream = Stream::new(buf, "Test");
        assert_eq!('F', stream.peek().unwrap());
    }
}

