//! Main file for Topal

use topal::stream::Stream;
use clap;

/// Main function
fn main() {
    let cmd = clap::Command::new("topal")
        .bin_name("topal")
        .subcommand_required(true)
        .subcommand(
            clap::command!("stream").arg(
                clap::arg!(--"file" <PATH>)
                .value_parser(clap::value_parser!(String)),
            ),
        );
    let matches = cmd.get_matches();
    let matches = match matches.subcommand() {
        Some(("stream", matches)) => matches,
        _ => unreachable!("Something went wrong with parsing"),
    };
    let stream_file = matches.get_one::<String>("file").unwrap();

    let mut stream = Stream::new(stream_file);

    println!("{:#?}", stream);
}
