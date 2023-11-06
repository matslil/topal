use std::io::{self, BufRead, BufReader};
use std::fs;
use std::fmt;
use utf8_chars::BufReadCharsExt;

mod curl_reader;
use crate::streamreader::curl_reader::CurlReader;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CharPos {
    pub line: usize,
    pub chr: usize,
}

impl CharPos {
    fn new() -> Self {
        Self {
            line: 0,
            chr:  0,
        }
    }
}

impl fmt::Display for CharPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.chr)
    }
}

// Collect different error objects into one. Then let
// formatted output function figure out which error object
// that takes precedence.
#[derive(Debug)]
pub enum StreamError {
    Open(String),
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Open(name) =>
                write!(f, "Failed to open file '{}':\n", name)
        }
    }
}

enum StreamType {
    Stdin(Stream<BufReader<io::Stdin>>),
    Curl(Stream<CurlReader>),
    File(Stream<BufReader<fs::File>>),
}

pub struct StreamReader {
    stream: StreamType,
}

impl StreamReader {
    pub fn from_stdin() -> Self {
        Self {
            stream: StreamType::Stdin(
                        Stream::new(
                            BufReader::new(io::stdin()), "<stdin>"
                            )
                        )
        }
    }

    pub fn from_url(url: &str) -> Result<Self, StreamError> {
        match CurlReader::new(url) {
            Ok(curl_reader) => Ok(Self {
                stream: StreamType::Curl(
                            Stream::new(curl_reader, url)
                            )
            }),
            Err(err) => Err(StreamError::Open(err.to_string())),
        }
    }

    pub fn from_path(path: &str) -> Result<Self, StreamError> {
        match fs::File::open(path) {
            Ok(file) => Ok(Self {
                stream: StreamType::File(
                            Stream::new(
                                io::BufReader::new(file), path)
                            )
            }),
            Err(fileerr) => Err(StreamError::Open(fileerr.to_string())),
        }
    }
}

impl Parseable for StreamReader {
    fn take(&mut self) -> Result<char, ParseError> {
        match &mut self.stream {
            StreamType::Stdin(s) => s.take(),
            StreamType::Curl(s) =>  s.take(),
            StreamType::File(s) =>  s.take(),
        }
    }

    fn peek(&mut self) -> Result<char, ParseError> {
        match &mut self.stream {
            StreamType::Stdin(s) => s.peek(),
            StreamType::Curl(s) =>  s.peek(),
            StreamType::File(s) =>  s.peek(),
        }
    }

    fn skip(&mut self) -> Result<(), ParseError> {
        match &mut self.stream {
            StreamType::Stdin(s) => s.skip(),
            StreamType::Curl(s) =>  s.skip(),
            StreamType::File(s) =>  s.skip(),
        }
    }
}

impl fmt::Display for StreamReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.stream {
            StreamType::Stdin(s) => s.fmt(f),
            StreamType::Curl(s) =>  s.fmt(f),
            StreamType::File(s) =>  s.fmt(f),
        }
    }
}

impl fmt::Debug for StreamReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.stream {
            StreamType::Stdin(s) => s.fmt(f),
            StreamType::Curl(s) =>  s.fmt(f),
            StreamType::File(s) =>  s.fmt(f),
        }
    }
}

struct Stream<T>
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
                if self.pos.chr == 0 { self.pos.chr = 1; }
                if self.pos.line == 0 { self.pos.line = 1; }
                match c {
                    '\t' => self.pos.chr += 8,
                    '\n' => {
                        self.pos.line += 1;
                        self.pos.chr = 1;
                    },
                    _    => self.pos.chr += 1,
                };
                Ok(c)
            },
            None => match self.reader.read_char_raw() {
                Ok(Some(c)) => {
                    if self.pos.chr == 0 { self.pos.chr = 1; }
                    if self.pos.line == 0 { self.pos.line = 1; }
                    match c {
                        '\t' => self.pos.chr += 8,
                        '\n' => self.pos.line += 1,
                        _    => self.pos.chr += 1,
                    };
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

    #[test]
    fn test_stream_peek() {
        let buf = bufread_from_url_or_file("-");
        let stream = Stream::new(buf);
        println!("{}", stream);
    }

//    #[test]
//    fn test_stream_new_illegal_url() {
//        let path = "[::::1]";
//        let url = "http://".to_owned() + path;
//        let StreamError::Open(err) = Stream::new(&url).unwrap_err();
//        assert_eq!(err, "".to_string());
//    }
//
//    #[test]
//    fn test_stream_stdin_new() {
//        assert_eq!(format!("{}", Stream::new("-").unwrap()), "<stdin>:0:0");
//    }
}
