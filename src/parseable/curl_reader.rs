use curl::easy::Easy;
use curl::Error as CurlError;
use std::io::{self, BufRead, Cursor};
use std::thread;
use crossbeam_channel as crossbeam;

// Make a url implementation for BufReader using cURL

pub struct CurlReader {
    // A separate thread is being spawned that communicates with cURL.
    // This threads provides data being received via this channel.
    rx: crossbeam::Receiver<Result<Vec<u8>, CurlError>>,

    // When fill_buf() is called, data received from rx above is
    // written to this cursor containing the buffered data.
    cursor: Cursor<Vec<u8>>,

    // If rx channel fails, this is the error.
    error: Option<CurlError>,
}

impl CurlReader {
    pub fn new(url: &str) -> Result<Self, CurlError> {
        let (tx, rx) = crossbeam::bounded(0);
        let tx2 = tx.clone();
        let mut handle = Easy::new();
        handle.url(url)?;

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

        Ok(CurlReader {
            rx,
            cursor: Cursor::new(Vec::new()),
            error: None,
        })
    }
}

impl BufRead for CurlReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.cursor.position() as usize == self.cursor.get_ref().len() {
            match self.rx.recv() {
                Ok(Ok(data)) => {
                    self.cursor = Cursor::new(data);
                }
                Ok(Err(curl_error)) => {
                    self.error = Some(curl_error.clone());
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe, curl_error));
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
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, err.clone()));
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
    fn example_dot_com() {
        let url = "https://www.example.com";
        let mut reader = CurlReader::new(url).unwrap();

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

    #[test]
    fn something_dot_invalid() {
        // This URL has the correct format, but should not be resolvable
        let url = "https://something.invalid";
        let mut reader = CurlReader::new(url).unwrap();

        let mut line = String::new();
        let err = reader.read_line(&mut line).unwrap_err();
        assert_eq!(io::ErrorKind::BrokenPipe, err.kind());
        assert_eq!("[6] Couldn't resolve host name (Could not resolve host: something.invalid)", format!("{}", err.get_ref().unwrap()));
    }
}
