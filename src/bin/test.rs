/// Binary for running functional tets.
///
/// Each command talks to a specific module. Options given before
/// command are generic options that works for all commands, while
/// options given after the command are specific to that command.

use topal::streamreader::StreamReader;
use topal::parseable::Parseable;
use topal::parseable;
use clap::{Parser, Subcommand, command, arg};
use tracing::{instrument, info};
use tracing::level_filters::LevelFilter;
use tracing_subscriber;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about)]
/// Functional test binary
struct App {
    #[arg(long, short, value_parser(valid_verbose), default_value("off"), env("TOPAL_LOG_LEVEL"))]
    verbose: LevelFilter,

    #[command(subcommand)]
    cmd: Command,
}

fn valid_verbose(opt: &str) -> Result<LevelFilter, String> {
    let verbose_values = HashMap::from([
        ("off", LevelFilter::OFF),
        ("error", LevelFilter::ERROR),
        ("warning", LevelFilter::WARN),
        ("info", LevelFilter::INFO),
        ("debug", LevelFilter::DEBUG),
        ("trace", LevelFilter::TRACE),
    ]);

    let lower_opt = opt.to_lowercase();
    match verbose_values.get(&*lower_opt) {
        Some(value) => Ok(value.clone()),
        None => Err(format!("{}: Not a valid verbose name, use '-h' for help", opt)),
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Test StreamReader object.
    ///
    /// StreamReader parses stdin if path is '-', a URL if path is
    /// a valid URL or a file name. It parses the stream character
    /// by character and updates line and column position.
    Streamreader {
        #[arg(required(true))]
        path: String,
    },
}

#[instrument]
fn cmd_streamreader(path: String) {
    let mut streamreader = StreamReader::new(path).unwrap();

    println!("{:#?}", streamreader);
    loop {
        let ret = streamreader.pop();
        match ret {
            Err(parseable::Error::EOS) => break,
            Err(parseable::Error::Broken(why)) => {
                println!("ERROR: {}", why);
                break;
            }
            Err(parseable::Error::SyntaxError) => {
                println!("Error parsing");
                break;
            }
            Ok(c) => print!("{}", c),
        }
    }
    println!("{:#?}", streamreader);
}

/// Main function
fn main() {
//    let default_level:usize = Level::ERROR.into();
    let args = App::parse();
    println!("{:?}", args);
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_max_level(args.verbose)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("Testing tracing");

    match args.cmd {
        Command::Streamreader { path } => cmd_streamreader(path),
    }
}

#[cfg(test)]

// Run the function tests
mod test {
    use assert_cmd;

    #[test]
    fn stdin() {
        let mut cmd = assert_cmd::Command::cargo_bin("test").unwrap();
        let run = cmd
            .arg("streamreader")
            .arg("-")
            .write_stdin("a\nbc")
            .assert();
        run.success();
    }

    #[test]
    fn url() {
        let mut cmd = assert_cmd::Command::cargo_bin("test").unwrap();
        let run = cmd
            .arg("streamreader")
            .arg("https://example.com")
            .assert();
        run.success();
    }

    #[test]
    fn file() {
        let mut cmd = assert_cmd::Command::cargo_bin("test").unwrap();
        let run = cmd
            .arg("streamreader")
            .arg("./README.rst")
            .assert();
        run.success();
    }

}

