use std::fmt::{self, Debug, Display};
use tracing::{trace, instrument};
use utf8_chars::BufReadCharsExt;
use std::io::{self, BufRead, BufReader};
use url::Url;
use std::fs;

mod charpos;
mod curl_reader;

use crate::parseable::charpos::CharPos;
use crate::parseable::curl_reader::CurlReader;

#[derive(PartialEq, Clone, Copy)]
pub enum Action<T>
where
    T: fmt::Debug + Clone + Copy
{
    /// If next callback call returns None or end-of-stream state is reached,
    /// return T without advancing the position.
    Request(T),

    /// If next callback call returns None, return None without advancing the
    /// position. If next callback call returns an error, this error will be
    /// returned.
    Require,

    /// Advance position and return T.
    Return(T),
}

impl<T> fmt::Debug for Action<T>
where
    T: fmt::Debug + Clone + Copy
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Request(item) => write!(f, "Request({:?})", item),
            Self::Require => write!(f, "Require"),
            Self::Return(item) => write!(f, "Return({:?})", item),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Error {
    EOS,
    Broken(String),
    SyntaxError,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EOS => write!(f, "End of stream"),
            Self::Broken(str) => write!(f, "Unexpected end of stream: {:?}", str),
            Self::SyntaxError => write!(f, "Error parsing"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EOS => write!(f, "End of stream"),
            Self::Broken(str) => write!(f, "Unexpected end of stream: {}", str),
            Self::SyntaxError => write!(f, "Error parsing"),
        }
    }
}

/// Parse stream of UTF-8 characters. Supports one character lookahead.
pub struct Parseable {
    name: String,
    chr: Option<char>,
    pos: CharPos,
    reader: Box<dyn BufRead>,
}

impl Parseable {
    /// Create a Parseable with given name and stream
    pub fn new(name: &str, stream: Box<dyn BufRead>) -> Self {
        Self {
            name: name.to_string(),
            chr: None,
            pos: CharPos::new(),
            reader: stream,
        }
    }

    pub fn from_path(path: &str) -> Result<Self, Error> {
        if path == "-" {
            Ok(Self::new("<stdin>", Box::new(BufReader::new(io::stdin()))))
        } else {
            match Url::parse(&path) {
                // Could be parsed as URL, assume it is
                Ok(_) => match CurlReader::new(path) {
                    Ok(curl_reader) => Ok(Self::new(path, Box::new(curl_reader))),
                    Err(err) => Err(Error::Broken(err.to_string())),
                }
                // Not an URL, assume it's a file path
                Err(_) => match fs::File::open(path) {
                    Ok(file) => Ok(Self::new(path, Box::new(BufReader::new(file)))),
                    Err(fileerr) => {
                        let msg = format!("{}: {}", path, fileerr);
                        Err(Error::Broken(msg.to_string()))
                    },
                }
            }
        }
    }

    /// Return next character, advance position.
    pub fn pop(&mut self) -> Result<char, Error> {
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
                Ok(None) => Err(Error::EOS),
                Err(err) => Err(Error::Broken(err.to_string())),
            }
        }
    }


    /// Return next character without advancing position.
    pub fn peek(&mut self) -> Result<char, Error> {
        match self.chr {
            None => {
                match self.reader.read_char_raw() {
                    Ok(Some(c)) => {
                        self.chr = Some(c);
                        Ok(c)
                    }
                    Ok(None) => Err(Error::EOS),
                    Err(err) => Err(Error::Broken(err.to_string())),
                }
            }
            Some(c) => {
                self.chr = Some(c);
                Ok(c)
            }
        }
    }

    /// Advance position to next character. Each call to `skip()` must have
    /// been preceded with a call to `peek()`.
    pub fn skip(&mut self) {
        self.pos.skip(self.chr.unwrap());
        self.chr.unwrap();
    }

    /// If `target` character matches character at current position,
    /// advance position to next character and return true,
    /// otherwise leave position unchanged and return false.
    #[instrument]
    pub fn take(&mut self, target: char) -> Result<bool, Error> {
        match self.peek() {
            Ok(c) => if target == c {
                self.skip();
                Ok(true)
            } else {
                Ok(false)
            }
            Err(err) => Err(err)
        }
    }

    #[instrument(skip(cb))]
    pub fn scan<T>(&mut self, cb: impl Fn(&str) -> Option<Action<T>>)
        -> Result<Option<T>, Error>
    where
        T: fmt::Debug + Clone + Copy
    {
        let mut seq = String::new();
        let mut require = false;
        let mut request = None;

        loop {
            match self.peek() {
                Ok(target) => {
                    seq.push(target);
                    let cb_result = cb(&seq);
                    trace!("callback({:?} -> {:?}", seq, cb_result);

                    match cb_result {
                        Some(Action::Return(result)) => {
                            self.skip();
                            break Ok(Some(result));
                        }
                        Some(Action::Request(result)) => {
                            self.skip();
                            trace!("require: {:?} -> false", require);
                            require = false;
                            request = Some(result);
                        }
                        Some(Action::Require) => {
                            self.skip();
                            trace!("require: {:?} -> true", require);
                            require = true
                        }
                        None => {
                            if require {
                                break Err(Error::SyntaxError)
                            } else {
                                break Ok(request);
                            }
                        }
                    }
                }
                Err(Error::EOS) => {
                    trace!("peek: Error::EOS");
                    if require {
                        break Err(Error::SyntaxError);
                    } else {
                        break Ok(request);
                    }
                }
                Err(err) => {
                    trace!("peek: {:?}", err);
                    break Err(err);
                }
            }
        }
    }

    /// Invoke `cb` with character at current position. Return what `cb` returns,
    /// but only advance position if return value is not `None`.
    #[instrument(skip(cb))]
    pub fn transform<T>(&mut self, cb: impl FnOnce(char) -> Option<T>)
        -> Result<Option<T>, Error> {
        match self.peek() {
            Ok(input) => match cb(input) {
                Some(output) => {
                    self.skip();
                    Ok(Some(output))
                }
                None => Ok(None),
            }
            Err(err) => Err(err),
        }
    }
}

impl Debug for Parseable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{:?}({:?})", self.name, self.pos, self.chr)
    }
}

impl Display for Parseable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.pos)
    }
}


#[cfg(test)]
mod scan {
    use super::*;
    use std::sync::Once;
    use std::env;
    use mry;
    use tracing::level_filters::LevelFilter;
    use std::str::FromStr;

    #[mry::mry]
    #[derive(fmt::Debug)]
    struct ParseReader {
    }

    #[mry::mry]
    impl Parseable for ParseReader {
        fn pop(&mut self) -> Result<char, Error> {
            Ok(' ')
        }

        fn peek(&mut self) -> Result<char, Error> {
            Ok(' ')
        }
    }

    fn mock_scan_callback<T>(returns: Option<Action<T>>)
        -> impl Fn(&str) -> Option<Action<T>>
        where T: fmt::Debug + Clone + Copy
    {
        move |input| -> Option<Action<T>> {
            returns.clone()
        }
    }

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            let mut level: LevelFilter = LevelFilter::OFF;
            if let Ok(value_str) = env::var("TOPAL_TEST_TRACING_LEVEL") {
                level = LevelFilter::from_str(&value_str).unwrap();
            }

            let subscriber = tracing_subscriber::fmt()
                .compact()
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_target(false)
                .with_max_level(level)
                .finish();
            tracing::subscriber::set_global_default(subscriber).unwrap();
            });
    }

    #[test]
    fn cb_returns_char() {
        setup();

        // We use Parseable both at System Under Test as well as mock.
        // We will mock functions that needs to be provided, but test the
        // implemented functions.
        let mut sut = mry::new!(ParseReader{});

        let cb = mock_scan_callback(Some(Action::Return('a')));
        sut.mock_peek().returns(Ok('a'));
        sut.mock_pop().returns(Ok('a'));

        assert_eq!('a', sut.scan(cb).unwrap().unwrap());
    }

    #[test]
    fn cb_returns_none() {
        setup();

        // We use Parseable both at System Under Test as well as mock.
        // We will mock functions that needs to be provided, but test the
        // implemented functions.
        let mut sut = mry::new!(ParseReader{});

        let cb = mock_scan_callback::<char>(None);
        sut.mock_peek().returns(Ok('a'));

        assert_eq!(None, sut.scan(cb).unwrap());
    }

    #[test]
    fn peek_returns_eos() {
        setup();

        // We use Parseable both at System Under Test as well as mock.
        // We will mock functions that needs to be provided, but test the
        // implemented functions.
        let mut sut = mry::new!(ParseReader{});

        let cb = mock_scan_callback::<char>(Some(Action::Require));
        sut.mock_peek().returns(Err(Error::EOS));

        assert_eq!(None, sut.scan(cb).unwrap());
    }

    #[test]
    fn cb_returns_require_peek_returns_eos() {
        setup();

        // We use Parseable both at System Under Test as well as mock.
        // We will mock functions that needs to be provided, but test the
        // implemented functions.
        let mut sut = mry::new!(ParseReader{});

        let cb = mock_scan_callback::<char>(Some(Action::Require));
        sut.mock_peek().returns(Ok('a'));
        sut.mock_peek().returns(Err(Error::EOS));

        assert_eq!(Error::SyntaxError, sut.scan(cb).unwrap_err());
    }
}

