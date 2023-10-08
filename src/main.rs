//! Main file for Topal

use topal::object_storage::{Storage};

/// Main function
fn main() {
    let mut storage = Storage::new();

    let _handle = storage.get_handle(&"Testing".to_string());

    println!("{:#?}", storage);
}
