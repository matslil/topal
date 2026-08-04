use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::process::ExitCode;

use topal_language::{JsonLines, Session, TraceSink, UNICODE_VERSION};

enum Mode {
    Script,
    Interactive,
    Test,
}

struct Arguments {
    mode: Mode,
    source: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("topal: {message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = parse_arguments(env::args().skip(1))?;
    match arguments.mode {
        Mode::Interactive => interactive(arguments.source.as_deref()),
        Mode::Script | Mode::Test => {
            let source = read_source(arguments.source.as_deref())?;
            let mut session = Session::new();
            if matches!(arguments.mode, Mode::Test) {
                let stderr = io::stderr();
                evaluate_and_print(&mut session, &source, &mut JsonLines::new(stderr.lock()))
            } else {
                evaluate_and_print(&mut session, &source, &mut io::sink())
            }
        }
    }
}

fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<Arguments, String> {
    let mut mode = Mode::Script;
    let mut source = None;
    for argument in arguments {
        match argument.as_str() {
            "--interactive" if matches!(mode, Mode::Script) => mode = Mode::Interactive,
            "--test" if matches!(mode, Mode::Script) => mode = Mode::Test,
            "--interactive" | "--test" => {
                return Err("--interactive and --test are mutually exclusive".into());
            }
            "--help" => {
                println!(
                    "Usage: topal [--interactive | --test] [FILE]\n\nWith no FILE, source is read from standard input."
                );
                std::process::exit(0);
            }
            "--version" => {
                println!(
                    "topal {} (language design-0; Unicode {}.{}.{})",
                    env!("CARGO_PKG_VERSION"),
                    UNICODE_VERSION.0,
                    UNICODE_VERSION.1,
                    UNICODE_VERSION.2
                );
                std::process::exit(0);
            }
            option if option.starts_with('-') => return Err(format!("unknown option: {option}")),
            path if source.is_none() => source = Some(path.to_owned()),
            path => return Err(format!("unexpected second source file: {path}")),
        }
    }
    Ok(Arguments { mode, source })
}

fn read_source(path: Option<&str>) -> Result<String, String> {
    if let Some(path) = path {
        fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))
    } else {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .map_err(|error| format!("cannot read standard input: {error}"))?;
        Ok(source)
    }
}

fn interactive(source: Option<&str>) -> Result<(), String> {
    if source.is_some() {
        return Err("interactive mode does not accept a source file".into());
    }
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut session = Session::new();
    let terminal = io::stdin().is_terminal();
    let mut pending = String::new();
    loop {
        if terminal {
            print!("{}", if pending.is_empty() { "> " } else { "... " });
            io::stdout().flush().map_err(|error| error.to_string())?;
        }
        let mut line = String::new();
        if input
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            if !pending.is_empty()
                && let Err(error) = session.evaluate(&pending, &mut io::sink())
            {
                eprintln!("topal: {error}");
            }
            return Ok(());
        }
        if line.trim().is_empty() && pending.is_empty() {
            continue;
        }
        pending.push_str(&line);
        match session.evaluate(&pending, &mut io::sink()) {
            Ok(value) => {
                println!("{value}");
                pending.clear();
            }
            Err(error) if error.code == "E-UNTERMINATED-STRING" => {}
            Err(error) => {
                eprintln!("topal: {error}");
                pending.clear();
            }
        }
    }
}

fn evaluate_and_print(
    session: &mut Session,
    source: &str,
    trace: &mut impl TraceSink,
) -> Result<(), String> {
    let value = session
        .evaluate(source, trace)
        .map_err(|error| error.to_string())?;
    println!("{value}");
    Ok(())
}
