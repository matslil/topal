use url::Url;
use std::io::{self, BufRead, BufReader};
use std::fs;
use std::fmt;
use utf8_chars::BufReadCharsExt;

mod curl_reader;
use crate::stream::curl_reader::CurlReader;

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

// struct StdinReader {
//     stdin : io::StdinLock<'static>,
// }
// 
// impl BufReadCharsExt for StdinReader {
// }
// 
// impl BufRead for StdinReader {
//     fn fill_buf(&mut self) -> io::Result<&[u8]> {
//         self.stdin.fill_buf()
//     }
// }
// 
// impl Read for StdinReader {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         self.stdin.read(buf)
//     }
// }
// 
// struct FileReader {
//     file : fs::File;
// }
// 
// impl BufReadCharsExt for FileReader {
// }
// 
// impl BufRead for FileReader {
//     fn fill_buf(&mut self) -> io::Result<&[u8]> {
//         self.file.fill_buf()
//     }
// }
// 
// impl Read for FileReader {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         self.file.read(buf)
//     }
// }
// 
enum StreamType {
    Stdin(io::StdinLock<'static>),
    Curl(CurlReader),
    File(BufReader<fs::File>),
}

struct StreamReader {
    stream: StreamType,
}

impl StreamReader {
    pub fn from_stdin(stdin: io::StdinLock<'static>) -> Self {
        Self { stream: StreamType::Stdin(stdin) }
    }

    pub fn from_curl(curl: CurlReader) -> Self {
        Self { stream: StreamType::Curl(curl) }
    }

    pub fn from_file(file: fs::File) -> Self {
        Self { stream: StreamType::File(BufReader::<fs::File>::new(file)) }
    }
}

impl BufRead for StreamReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match &mut self.stream {
            StreamType::Stdin(s) => s.fill_buf(),
            StreamType::Curl(s) =>  s.fill_buf(),
            StreamType::File(s) =>  s.fill_buf(),
        }
    }

    fn consume(&mut self, amount: usize) {
        match &mut self.stream {
            StreamType::Stdin(s) => s.consume(amount),
            StreamType::Curl(s) =>  s.consume(amount),
            StreamType::File(s) =>  s.consume(amount),
        }
    }
}

impl io::Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.stream {
            StreamType::Stdin(s) => s.read(buf),
            StreamType::Curl(s) => s.read(buf),
            StreamType::File(s) => s.read(buf),
        }
    }
}

pub struct Stream {
    name: String,
    chr: Option<char>,
    pos: CharPos,
    reader: StreamReader,
}

impl Stream {
    pub fn new(path: &str) -> Result<Self, StreamError> {
        if path == "-" {
            Ok(Self {
                name: "<stdin>".to_string(),
                chr: None,
                pos: CharPos::new(),
                reader: StreamReader::from_stdin(io::stdin().lock()),
            })
        } else {
            match Url::parse(path) {
                // Could be parsed as URL, assume it is
                Ok(_) => {
                    match CurlReader::new(path) {
                        Ok(curl_reader) => {
                            Ok(Self {
                                name: path.into(),
                                chr: None,
                                pos: CharPos::new(),
                                reader: StreamReader::from_curl(curl_reader),
                            })
                        },
                        Err(err) => Err(StreamError::Open(err.to_string())),
                    }
                },
                // Not an URL, assume it's a file path
                Err(_) => match fs::File::open(path) {
                    Ok(file) => {
                        Ok(Self {
                            name: path.to_string(),
                            chr: None,
                            pos: CharPos::new(),
                            reader: StreamReader::from_file(file),
                        })
                    },
                    Err(fileerr) => Err(StreamError::Open(fileerr.to_string())),
                }
            }
        }
    }
}

impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.pos)
    }
}

impl fmt::Debug for Stream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.pos)
    }
}

impl Parseable for Stream {
    fn take(&mut self) -> Result<char, ParseError> {
        match self.chr.take() {
            Some(c) => Ok(c),
            None => match self.reader.read_char_raw() {
                Ok(Some(c)) => Ok(c),
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
        if self.chr == None {
            match self.reader.read_char_raw() {
                Ok(Some(_)) => Ok(()),
                Ok(None) => Err(ParseError::EOS),
                Err(err) => Err(ParseError::Broken(err.to_string())),
            }
        } else {
            self.chr = None;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_new_illegal_url() {
        let path = "[::::1]";
        let url = "http://".to_owned() + path;
        let StreamError::Open(err) = Stream::new(&url).unwrap_err();
        assert_eq!(err, "".to_string());
    }

    #[test]
    fn test_stream_stdin_new() {
        assert_eq!(format!("{}", Stream::new("-").unwrap()), "<stdin>:0:0");
    }
}
