use std::fmt;
use tracing::{trace, instrument};

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

/// Parse stream of UTF-8 characters. Supports one character lookahead.
pub trait Parseable : fmt::Debug {
    /// Return next character, advance position.
    fn pop(&mut self) -> Result<char, Error>;

    /// Return next character without advancing position.
    fn peek(&mut self) -> Result<char, Error>;

    /// Advance position to next character. Each call to `skip()` must have
    /// been preceded with a call to `peek()`.
    fn skip(&mut self);

    /// If `target` character matches character at current position,
    /// advance position to next character and return true,
    /// otherwise leave position unchanged and return false.
    #[instrument]
    fn take(&mut self, target: char) -> Result<bool, Error> {
        match self.peek() {
            Ok(c) => if target == c {
                match self.skip() {
                    Ok(()) => Ok(true),
                    Err(err) => {
                        trace!("Error: {}", err);
                        Err(err)
                    }
                }
            } else {
                Ok(false)
            }
            Err(err) => Err(err)
        }
    }

    #[instrument(skip(cb))]
    fn scan<T>(&mut self, cb: impl Fn(&str) -> Option<Action<T>>)
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
                                match self.skip() {
                                    Ok(()) => {
                                        break Ok(Some(result));
                                    }
                                    Err(err) => {
                                        break Err(err);
                                    }
                                }
                            }
                            Some(Action::Request(result)) => {
                                match self.skip() {
                                    Ok(()) => {
                                        trace!("require: {:?} -> false", require);
                                        require = false;
                                        request = Some(result);
                                    }
                                    Err(err) => {
                                        break Err(err);
                                    }
                                }
                            }
                            Some(Action::Require) => {
                                match self.skip() {
                                    Ok(()) => {
                                        trace!("require: {:?} -> true", require);
                                        require = true
                                    }
                                    Err(err) => {
                                        break Err(err);
                                    }
                                }
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
    fn transform<T>(&mut self, cb: impl FnOnce(char) -> Option<T>)
        -> Result<Option<T>, Error> {
        match self.peek() {
            Ok(input) => match cb(input) {
                Some(output) => {
                    match self.skip() {
                        Ok(()) => Ok(Some(output)),
                        Err(err) => Err(err),
                    }
                }
                None => Ok(None),
            }
            Err(err) => Err(err),
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

