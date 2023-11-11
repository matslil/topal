use std::fmt;

pub trait Parseable {
    fn take(&mut self) -> Result<char, ParseError>;
    fn peek(&mut self) -> Result<char, ParseError>;
    fn skip(&mut self) -> Result<(), ParseError>;
}

#[derive(Debug, Clone)]
pub enum ParseError {
    EOS,
    Broken(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EOS => write!(f, "End of stream"),
            Self::Broken(str) => write!(f, "{}", str),
        }
    }
}
