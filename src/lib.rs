//! This is library part

mod hello_world;

pub use crate::hello_world::GREETING;

/// Print greeting message
pub fn print_hello_world() {
    println!("{}", GREETING);
}
