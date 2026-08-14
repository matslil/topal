use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use topal_language::{JsonLines, LanguageVersion, Session, TraceSink, UNICODE_VERSION};

enum Mode {
    Script,
    Interactive,
    Test,
}

struct Arguments {
    mode: Mode,
    source: Option<String>,
    language_version: Option<LanguageVersion>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = parse_arguments(env::args().skip(1))?;
    match arguments.mode {
        Mode::Interactive => interactive(arguments.source.as_deref(), arguments.language_version),
        Mode::Script | Mode::Test => {
            if arguments.language_version.is_some() {
                return Err("--language-version supplies interactive context only; source files declare their own version".into());
            }
            let source_name = arguments.source.as_deref().unwrap_or("<stdin>");
            let mut session = Session::new();
            if matches!(arguments.mode, Mode::Test) {
                let stderr = io::stderr();
                let mut trace = JsonLines::new(stderr.lock());
                evaluate_input(
                    &mut session,
                    arguments.source.as_deref(),
                    source_name,
                    &mut trace,
                )
            } else {
                evaluate_input(
                    &mut session,
                    arguments.source.as_deref(),
                    source_name,
                    &mut io::sink(),
                )
            }
        }
    }
}

fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<Arguments, String> {
    let mut mode = Mode::Script;
    let mut source = None;
    let mut language_version = None;
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--interactive" if matches!(mode, Mode::Script) => mode = Mode::Interactive,
            "--test" if matches!(mode, Mode::Script) => mode = Mode::Test,
            "--interactive" | "--test" => {
                return Err("--interactive and --test are mutually exclusive".into());
            }
            "--language-version" => {
                let value = arguments
                    .next()
                    .ok_or("--language-version requires a version such as v0.1")?;
                language_version = Some(
                    value
                        .parse::<LanguageVersion>()
                        .map_err(|error| format!("invalid language version `{value}`: {error}"))?,
                );
            }
            "--help" => {
                println!(
                    "Usage: topal [--interactive [--language-version VERSION] | --test] [FILE]\n\nWith no FILE, source is read from standard input. Source files declare their language version."
                );
                std::process::exit(0);
            }
            "--version" => {
                println!(
                    "topal {} (highest language {}; Unicode {}.{}.{})",
                    env!("CARGO_PKG_VERSION"),
                    Session::highest_supported_language_version(),
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
    Ok(Arguments {
        mode,
        source,
        language_version,
    })
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

fn interactive(
    source: Option<&str>,
    language_version: Option<LanguageVersion>,
) -> Result<(), String> {
    if source.is_some() {
        return Err("interactive mode does not accept a source file".into());
    }
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let language_version =
        language_version.unwrap_or_else(Session::highest_supported_language_version);
    let mut session = Session::for_language_version(language_version)
        .map_err(|error| format!("unsupported language version `{language_version}`: {error}"))?;
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
            if !pending.is_empty() {
                match session.evaluate(&pending, &mut io::sink()) {
                    Ok(value) => println!("{value}"),
                    Err(error) => eprintln!("{}", error.render("<interactive>")),
                }
            }
            return Ok(());
        }
        if !pending.is_empty()
            && (line.trim().is_empty() || !line.chars().next().is_some_and(char::is_whitespace))
            && Session::awaits_dedent(&pending)
        {
            match session.evaluate(&pending, &mut io::sink()) {
                Ok(value) => println!("{value}"),
                Err(error) => eprintln!("{}", error.render("<interactive>")),
            }
            pending.clear();
        }
        if line.trim().is_empty() && pending.is_empty() {
            continue;
        }
        if pending.is_empty() && line.trim_start().starts_with('#') {
            continue;
        }
        pending.push_str(&line);
        if line.chars().next().is_some_and(char::is_whitespace) && Session::awaits_dedent(&pending)
        {
            continue;
        }
        match session.evaluate(&pending, &mut io::sink()) {
            Ok(value) => {
                println!("{value}");
                pending.clear();
            }
            Err(error)
                if matches!(
                    error.code,
                    "E-UNTERMINATED-STRING"
                        | "E-EXPECTED-RPAREN"
                        | "E-EXPECTED-FUNCTION-BODY"
                        | "E-EXPECTED-FOREACH-BODY"
                        | "E-UNSUPPORTED-GENERATOR-HEADER"
                        | "E-EXPECTED-INDENTED-GENERATOR-HEADER"
                        | "E-EXPECTED-GENERATOR-BODY"
                        | "E-EMPTY-UNION"
                        | "E-UNSUPPORTED-INCOMPLETE-DECISION"
                        | "E-INCOMPLETE-ERROR-CODE-DECISION"
                ) => {}
            Err(error) => {
                eprintln!("{}", error.render("<interactive>"));
                pending.clear();
            }
        }
    }
}

fn evaluate_and_print(
    session: &mut Session,
    source: &str,
    source_name: &str,
    trace: &mut impl TraceSink,
) -> Result<(), String> {
    let value = session
        .evaluate_source_file(source, trace)
        .map_err(|error| error.render(source_name))?;
    println!("{value}");
    Ok(())
}

fn evaluate_input(
    session: &mut Session,
    path: Option<&str>,
    source_name: &str,
    trace: &mut impl TraceSink,
) -> Result<(), String> {
    if let Some(path) = path.filter(|path| Path::new(path).is_dir()) {
        return evaluate_directory(session, Path::new(path), trace);
    }
    let source = read_source(path)?;
    evaluate_and_print(session, &source, source_name, trace)
}

fn evaluate_directory(
    session: &mut Session,
    directory: &Path,
    trace: &mut impl TraceSink,
) -> Result<(), String> {
    load_directory_children(session, directory, trace)?;
    let entry = directory.join("application.t");
    if !entry.is_file() {
        return Err(format!("{} has no application.t", directory.display()));
    }
    let source = fs::read_to_string(&entry)
        .map_err(|error| format!("cannot read {}: {error}", entry.display()))?;
    evaluate_and_print(session, &source, &entry.display().to_string(), trace)
}

fn load_directory_children(
    session: &mut Session,
    directory: &Path,
    trace: &mut impl TraceSink,
) -> Result<(), String> {
    let mut paths = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    paths.sort();
    for path in &paths {
        if path.is_dir() {
            let mut child = Session::new();
            let descriptor = path.join("module.t");
            if descriptor.is_file() {
                let source = fs::read_to_string(&descriptor)
                    .map_err(|error| format!("cannot read {}: {error}", descriptor.display()))?;
                child
                    .evaluate_source_file(&source, trace)
                    .map_err(|error| error.render(&descriptor.display().to_string()))?;
            }
            load_directory_children(&mut child, path, trace)?;
            let name = path.file_name().unwrap().to_string_lossy();
            session
                .attach_module(&name, child, trace)
                .map_err(|error| error.render(&path.display().to_string()))?;
        } else if path.extension().is_some_and(|extension| extension == "t")
            && path.file_name().is_some_and(|name| {
                !matches!(
                    name.to_str(),
                    Some("application.t" | "package.t" | "library.t" | "module.t")
                )
            })
        {
            let name = path.file_stem().unwrap().to_string_lossy();
            let source = fs::read_to_string(path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            session
                .load_module(&name, &source, trace)
                .map_err(|error| error.render(&path.display().to_string()))?;
        }
    }
    Ok(())
}
