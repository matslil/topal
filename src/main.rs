//! Main file for Topal

use topal::stream::Stream;

/// Main function
fn main() {
    let mut stream = Stream::new("-");

    println!("{:#?}", stream);
}
