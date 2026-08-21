//! Shared store identities, changes, and reference models.

use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectId(u64);
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Change<Value> {
    Inserted(ObjectId, Value),
    Replaced(ObjectId, Value),
    Removed(ObjectId),
}

#[derive(Debug)]
pub struct MemoryStore<Value> {
    next: u64,
    objects: BTreeMap<ObjectId, Value>,
    changes: VecDeque<Change<Value>>,
    change_limit: usize,
}

impl<Value: Clone> MemoryStore<Value> {
    #[must_use]
    pub const fn new(change_limit: usize) -> Self {
        Self {
            next: 1,
            objects: BTreeMap::new(),
            changes: VecDeque::new(),
            change_limit,
        }
    }
    /// Inserts an identified object.
    /// # Errors
    /// Returns `ChangeBackpressure` before mutation if its change cannot queue.
    pub fn insert(&mut self, value: Value) -> Result<ObjectId, StoreFailure> {
        if self.changes.len() >= self.change_limit {
            return Err(StoreFailure::ChangeBackpressure);
        }
        let id = ObjectId(self.next);
        self.next = self.next.checked_add(1).ok_or(StoreFailure::Exhausted)?;
        self.objects.insert(id, value.clone());
        self.changes.push_back(Change::Inserted(id, value));
        Ok(id)
    }
    #[must_use]
    pub fn lookup(&self, id: ObjectId) -> Option<&Value> {
        self.objects.get(&id)
    }
    pub fn next_change(&mut self) -> Option<Change<Value>> {
        self.changes.pop_front()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreFailure {
    ChangeBackpressure,
    Exhausted,
    Conflict,
    Uncertain,
}

pub trait QueryStore<Query> {
    type Row;
    type Failure;
    /// Executes a model-specific query.
    /// # Errors
    /// Returns a model-specific typed failure.
    fn query(&self, query: &Query) -> Result<Vec<Self::Row>, Self::Failure>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_is_not_a_name_and_changes_are_bounded() {
        let mut store = MemoryStore::new(1);
        let id = store.insert("value").unwrap();
        assert_eq!(store.lookup(id), Some(&"value"));
        assert_eq!(store.insert("other"), Err(StoreFailure::ChangeBackpressure));
        assert!(matches!(
            store.next_change(),
            Some(Change::Inserted(_, "value"))
        ));
    }
}
