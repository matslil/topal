use std::io::{self,BufReader};
use std::fs;
use std::fmt;

mod curl_reader;
mod charpos;
mod stream;
use crate::streamreader::curl_reader::CurlReader;
use crate::streamreader::charpos::CharPos;
use crate::streamreader::stream::Stream;
use crate::parseable::{Parseable, ParseError};

// pub trait Parseable {
//     fn take(&mut self) -> Result<char, ParseError>;
//     fn peek(&mut self) -> Result<char, ParseError>;
//     fn skip(&mut self) -> Result<(), ParseError>;
// }
// 
// #[derive(Debug, Clone)]
// pub enum ParseError {
//     EOS,
//     Broken(String),
// }
// 
// impl fmt::Display for ParseError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Self::EOS => write!(f, "End of stream"),
//             Self::Broken(str) => write!(f, "{}", str),
//         }
//     }
// }

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
            Err(fileerr) => {
                let msg = format!("{}: {}", path, fileerr);
                Err(StreamError::Open(msg.to_string()))
            },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_stdin() {
        let streamreader = StreamReader::from_stdin();
        assert_eq!("<stdin>:1:1", format!("{}", streamreader));
        assert_eq!("<stdin>:1:1", format!("{:}", streamreader));
    }

    #[test]
    fn open_url() {
        let streamreader = StreamReader::from_url("https://example.com").unwrap();
        assert_eq!("https://example.com:1:1", format!("{}", streamreader));
        assert_eq!("https://example.com:1:1", format!("{:}", streamreader));
    }

    #[test]
    fn file_1_5() {
        let filename = "testfiles/1.5";
        let mut streamreader = StreamReader::from_path(filename).unwrap();
        loop {
            match streamreader.skip() {
                Ok(_) => (),
                Err(ParseError::EOS) => break,
                Err(err) => panic!("{}", err),
            };
        }
        assert_eq!(format!("{}:1:5", filename), format!("{}", streamreader));
        assert_eq!(format!("{}:1:5", filename), format!("{:}", streamreader));
    }
}
