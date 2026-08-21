//! File-store specialization with identity separate from paths.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FileId(u64);
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileFailure {
    InvalidName,
    NotFound,
    Exists,
}

#[derive(Debug, Default)]
pub struct MemoryFileStore {
    next: u64,
    files: BTreeMap<FileId, Vec<u8>>,
    names: BTreeMap<String, FileId>,
}

impl MemoryFileStore {
    /// Creates an identified file and one namespace relation.
    /// # Errors
    /// Rejects empty names, separators, dot relations, and existing names.
    pub fn create(&mut self, name: &str, bytes: Vec<u8>) -> Result<FileId, FileFailure> {
        if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
            return Err(FileFailure::InvalidName);
        }
        if self.names.contains_key(name) {
            return Err(FileFailure::Exists);
        }
        self.next += 1;
        let id = FileId(self.next);
        self.files.insert(id, bytes);
        self.names.insert(name.into(), id);
        Ok(id)
    }
    #[must_use]
    pub fn resolve(&self, name: &str) -> Option<FileId> {
        self.names.get(name).copied()
    }
    #[must_use]
    pub fn read(&self, id: FileId) -> Option<&[u8]> {
        self.files.get(&id).map(Vec::as_slice)
    }
    /// Renames only the namespace relation; open identity remains valid.
    /// # Errors
    /// Returns typed invalid-name, missing, or collision outcomes.
    pub fn rename(&mut self, old: &str, new: &str) -> Result<(), FileFailure> {
        if new.is_empty() || new == "." || new == ".." || new.contains(['/', '\\']) {
            return Err(FileFailure::InvalidName);
        }
        if self.names.contains_key(new) {
            return Err(FileFailure::Exists);
        }
        let id = self.names.remove(old).ok_or(FileFailure::NotFound)?;
        self.names.insert(new.into(), id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_are_queries_not_identity() {
        let mut store = MemoryFileStore::default();
        let id = store.create("a", b"x".to_vec()).unwrap();
        store.rename("a", "b").unwrap();
        assert_eq!(store.resolve("a"), None);
        assert_eq!(store.read(id), Some(&b"x"[..]));
        assert_eq!(store.create("../x", vec![]), Err(FileFailure::InvalidName));
    }
}
