//! Main file for Topal

use topal::streamreader::{StreamReader, ParseError, Parseable};
use clap;
use url::Url;

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
    let path = matches.get_one::<String>("file").unwrap();

    let mut streamreader = if path == "-" {
        Ok(StreamReader::from_stdin())
    } else {
        match Url::parse(path) {
            // Could be parsed as URL, assume it is
            Ok(_) => StreamReader::from_url(path),
            // Not an URL, assume it's a file path
            Err(_) => StreamReader::from_path(path),
        }
    }.unwrap();

    println!("{:#?}", streamreader);
    loop {
        let ret = streamreader.take();
        match ret {
            Err(ParseError::EOS) => break,
            Err(ParseError::Broken(why)) => {
                println!("ERROR: {}", why);
                break;
            },
            Ok(c) => print!("{}", c),
        }
    }
    println!("{:#?}", streamreader);
}
