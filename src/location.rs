use std::fmt;
use serde::{Serialize, Deserialize};
use crate::object_storage::{Storage, Handle};
use usl::Url;

pub struct FileNames {
    files: Storage<Url>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileLine {
    file: String,
    line: usize
}

impl fmt::Display for FileLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.file, self.line);
    }
}

