use std::fmt;

#[derive(PartialEq, Clone)]
pub enum Action<T>
where
    T: fmt::Debug
{
    Request(T),
    Require,
    Return(T),
}

impl<T> fmt::Debug for Action<T>
where
    T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Request(item) => write!(f, "Request({:?})", item),
            Self::Require => write!(f, "Require"),
            Self::Return(item) => write!(f, "Return({:?})", item),
        }
    }
}

pub trait Parseable {
    fn pop(&mut self) -> Result<char, Error>;
    fn peek(&mut self) -> Result<char, Error>;
    fn skip(&mut self) -> Result<(), Error> {
        match self.pop() {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn take(&mut self, target: char) -> Result<bool, Error> {
        match self.peek() {
            Ok(c) => if target == c {
                match self.skip() {
                    Ok(()) => Ok(true),
                    Err(err) => Err(err),
                }
            } else {
                Ok(false)
            }
            Err(err) => Err(err)
        }
    }

    fn scan<T>(&mut self, cb: impl Fn(&str) -> Option<Action<T>>)
        -> Result<Option<T>, Error>
    where
        T: fmt::Debug
    {
            let mut seq = String::new();
            let mut require = false;
            let mut request = None;

            loop {
                match self.peek() {
                    Ok(target) => {
                        seq.push(target);

                        match cb(&seq) {
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
                                    Ok(()) => require = true,
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
                        if require {
                            break Err(Error::EOS);
                        } else {
                            break Ok(request);
                        }
                    }
                    Err(err) => {
                        break Err(err);
                    }
                }
            }
        }

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
mod test {
    use super::*;
    use mry;

    #[mry::mry]
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

    #[mry::mry]
    fn scan_cb(current: &str) -> Option<Action<char>> {
        return None;
    }

    #[test]
    #[mry::lock(scan_cb)]
    fn scan_request_returns_eos() {
        // We use Parseable both at System Under Test as well as mock.
        // We will mock functions that needs to be provided, but test the
        // implemented functions.
        let mut sut = mry::new!(ParseReader{});

        mock_scan_cb(mry::Any).returns(Some(Action::Return('a')));
        sut.mock_peek().returns(Ok('a'));
        sut.mock_pop().returns(Ok('a'));

        assert_eq!('a', sut.scan(scan_cb).unwrap().unwrap());
    }
}

