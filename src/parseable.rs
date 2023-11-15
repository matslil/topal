use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Action<T> {
    Request(T),
    Require,
    Return(T),
}

pub trait Parseable {
    fn pop(&mut self) -> Result<char, Error>;
    fn peek(&mut self) -> Result<char, Error>;
    fn skip(&mut self) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub enum Error {
    EOS,
    Broken(String),
    SyntaxError,
}

impl dyn Parseable {
    pub fn take(&mut self, target: char) -> Result<bool, Error> {
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

    pub fn scan<T>(&mut self, cb: impl Fn(&str) -> Option<Action<T>>)
        -> Result<Option<T>, Error> {
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

    pub fn transform<T>(&mut self, cb: impl FnOnce(char) -> Option<T>)
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EOS => write!(f, "End of stream"),
            Self::Broken(str) => write!(f, "Unexpected end of stream: {}", str),
            Self::SyntaxError => write!(f, "Error parsing"),
        }
    }
}
