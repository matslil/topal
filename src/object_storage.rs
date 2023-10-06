// Let's use the serde crate for serialization and deserialization.
// You'll need to add serde and serde_derive to your dependencies.
use serde::{Serialize, Deserialize};
use std::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Handle(usize);

impl From<Handle> for usize {
    fn from(handle: Handle) -> usize {
        handle.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Storage<T> {
    objects: Vec<T>,
}

impl<T> Storage<T>
where
    T: Serialize + for<'de> Deserialize<'de> + std::cmp::PartialEq<T> + Clone,
{
    pub fn new() -> Self {
        Storage {
            objects: Vec::new(),
        }
    }

    // Add an item and return its integer handle.
    pub fn get_handle(&mut self, item: &T) -> Handle {
        let idx = self.objects.iter().position(|r| r == item).unwrap_or_else(|| {
            self.objects.push((*item).clone());
            self.objects.len() - 1
        });

        let handle = Handle(idx);
        handle
    }

    // Get a reference to an item by its handle.
    pub fn get(&self, handle: Handle) -> Option<&T> {
        let index : usize = usize::from(handle);
        self.objects.get(index)
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    struct TestStruct {
        a: i32,
    }

    #[test]
    fn test_retrieve() {
        let mut storage = Storage::new();
        let item = "Testing string".to_string();
        let dummy = "Ignore this".to_string();

        let handle1 = storage.get_handle(&item);
        let _handle = storage.get_handle(&dummy);
        let handle2 = storage.get_handle(&item);

        assert_eq!(handle1, handle2);
    }

    #[test]
    fn test_storage() {
        let mut storage = Storage::new();

        let item = TestStruct { a: 42 };
        let handle = storage.get_handle(&item);

        assert_eq!(storage.get(handle).unwrap().a, 42);

        let serialized = serde_json::to_string(&storage).unwrap();
        fs::write("object_token_serialize_test.json", serialized).unwrap();
        let data = fs::read_to_string("object_token_serialize_test.json").unwrap();
        let storage2 : Storage<TestStruct> = serde_json::from_str(&data).unwrap();
        
        assert_eq!(storage2.get(handle).unwrap().a, 42);
    }
}

