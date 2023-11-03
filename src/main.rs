//! Main file for Topal

use topal::stream::{Stream, ParseError, Parseable};
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

    let mut stream = Stream::new(stream_file).unwrap();

    println!("{:#?}", stream);
    loop {
        let ret = stream.take();
        match ret {
            Err(ParseError::EOS) => break,
            Err(ParseError::Broken(why)) => {
                println!("ERROR: {}", why);
                break;
            },
            Ok(c) => print!("{}", c),
        }
    }
    println!("{:#?}", stream);
}
