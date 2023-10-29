use curl::easy::Easy;
use curl::Error as CurlError;
use std::io::{self, BufRead, Cursor};
use std::fs::File;
use std::{thread, fmt};
use crossbeam_channel as crossbeam;

pub trait Parseable {
    fn take(&self) -> Result<char, ParseError>;
    fn peek(&self) -> Result<char, ParseError>;
    fn skip(&self) -> Result<(), ParseError>;
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
    Open((String, io::Error, CurlError)),
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Open((name, io_err, curl_err)) =>
                write!(f, "Failed to open file '{}':\n  Tried as URL: {}\n  Tried as local file: {}",
                    name, io_err, curl_err)
        }
    }
}

pub struct Stream {
    name: String,
    buf:  Box<dyn BufRead>,
    pos:  CharPos,
}

impl Stream {
    pub fn new(path: &str) -> Result<Self, StreamError> {
        if path == "-" {
            Ok(Self {
                name: "<stdin>".to_string(),
                buf: Box::new(io::stdin().lock()),
                pos: CharPos::new(),
            })
        } else {
            let mut easy = Easy::new();
            match easy.url(path) {
                Ok(_) =>
                    Ok(Self {
                        name: match easy.effective_url() {
                            Ok(opt_url) => match opt_url {
                                Some(url) => url.to_string(),
                                None => "".to_string(),
                            },
                            Err(err) => format!("<Invalid URL: {}>", err),
                        },
                        buf: Box::new(CurlReader::new(easy)),
                        pos: CharPos::new(),
                    }),
                Err(urlerr) => match File::open(path) {
                    Ok(file) =>
                        Ok(Self {
                            name: path.to_string(),
                            buf: Box::new(io::BufReader::new(file)),
                            pos: CharPos::new(),
                        }),
                    Err(fileerr) =>
                        Err(StreamError::Open((path.to_string(), fileerr, urlerr))),
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

struct CurlReader {
    rx: crossbeam::Receiver<Result<Vec<u8>, CurlError>>,
    buffer: Vec<u8>,
    cursor: Cursor<Vec<u8>>,
    error: Option<CurlError>,
}

impl CurlReader {
    fn new(mut handle: Easy) -> Self {
        let (tx, rx) = crossbeam::bounded(0);
        let tx2 = tx.clone();

        thread::spawn(move || {
            let mut transfer = handle.transfer();
            if let Err(err) = transfer.write_function(move |new_data| {
                tx.send(Ok(new_data.to_vec())).unwrap();
                Ok(new_data.len())
            }) {
                tx2.send(Err(err)).unwrap();
                return;
            }

            if let Err(err) = transfer.perform() {
                tx2.send(Err(err)).unwrap();
            }
        });

        CurlReader {
            rx,
            buffer: Vec::new(),
            cursor: Cursor::new(Vec::new()),
            error: None,
        }
    }
}

impl BufRead for CurlReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.cursor.position() as usize == self.buffer.len() {
            match self.rx.recv() {
                Ok(Ok(data)) => {
                    self.buffer = data;
                    self.cursor = Cursor::new(self.buffer.clone());
                }
                Ok(Err(curl_error)) => {
                    self.error = Some(curl_error);
                    return Err(io::Error::new(io::ErrorKind::Other, "Curl error occurred"));
                }
                Err(_) => {
                    return Ok(&[]);
                }
            }
        }

        self.cursor.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.cursor.consume(amt);
    }
}

impl io::Read for CurlReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(ref err) = self.error {
            return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
        }

        // Check the remaining data in the buffer first.
        let available = self.fill_buf()?;
        if available.is_empty() {
            return Ok(0);
        }

        let len = std::cmp::min(available.len(), buf.len());
        buf[0..len].copy_from_slice(&available[0..len]);
        self.consume(len);

        Ok(len)
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
        assert_eq!(err.0, path);
        assert_eq!(err.1.kind(), io::ErrorKind::NotFound);
        assert!(err.2.is_url_malformed());
    }

    #[test]
    fn test_stream_stdin_new() {
        assert_eq!(format!("{}", Stream::new("-").unwrap()), "<stdin>:0:0");
    }

    #[test]
    fn test_curl_reader() {
        let url = "https://www.example.com";
        let mut handle = Easy::new();
        handle.url(url).unwrap();
        let mut reader = CurlReader::new(handle);

        let mut line = String::new();
        while let Ok(bytes_read) = reader.read_line(&mut line) {
            if bytes_read == 0 {
                break;
            }
            println!("{}", line);
            line.clear();
        }

        // Handle the error, if any
        if let Some(err) = reader.error {
            println!("Error occurred: {}", err);
        }
    }
}
