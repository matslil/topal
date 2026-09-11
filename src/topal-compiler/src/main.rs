use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use topal_compiler::{CompileError, CompileOptions, Emit, TARGET_TRIPLE, compile_source};

mod test_runner;

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
    let mut arguments = env::args().skip(1).peekable();
    if arguments.peek().is_some_and(|argument| argument == "test") {
        arguments.next();
        return test_runner::run(arguments);
    }
    let arguments = parse_arguments(arguments)?;
    let source = fs::read_to_string(&arguments.source)
        .map_err(|error| format!("cannot read {}: {error}", arguments.source.display()))?;
    let source_name = arguments.source.to_string_lossy().into_owned();
    let options = CompileOptions {
        source_name: source_name.clone(),
        output: arguments.output,
        emit: arguments.emit,
        llvm_tools: arguments.llvm_tools,
    };
    compile_source(&source, &options).map_err(|error| render_error(error, &source_name))?;
    Ok(())
}

struct Arguments {
    source: PathBuf,
    output: PathBuf,
    emit: Emit,
    llvm_tools: Option<PathBuf>,
}

fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<Arguments, String> {
    let mut source = None;
    let mut output = None;
    let mut emit = Emit::Executable;
    let mut llvm_tools = None;
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-O0" | "-g" => {}
            "-o" => {
                output = Some(PathBuf::from(arguments.next().ok_or("-o requires a path")?));
            }
            "--emit" => {
                let value = arguments
                    .next()
                    .ok_or("--emit requires llvm-ir, object, or executable")?;
                emit = match value.as_str() {
                    "llvm-ir" => Emit::LlvmIr,
                    "object" => Emit::Object,
                    "executable" | "exe" => Emit::Executable,
                    _ => return Err(format!("unsupported emission kind `{value}`")),
                };
            }
            "--target" => {
                let value = arguments.next().ok_or("--target requires a triple")?;
                if value != TARGET_TRIPLE {
                    return Err(format!(
                        "unsupported target `{value}`; this increment supports only `{TARGET_TRIPLE}`"
                    ));
                }
            }
            "--llvm-tools" => {
                llvm_tools = Some(PathBuf::from(
                    arguments
                        .next()
                        .ok_or("--llvm-tools requires a directory")?,
                ));
            }
            "--version" => {
                println!(
                    "topalc {} (LLVM 22; {TARGET_TRIPLE})",
                    env!("CARGO_PKG_VERSION")
                );
                std::process::exit(0);
            }
            "--help" | "-h" => {
                println!(
                    "Usage: topalc [-O0] [-g] [--target {TARGET_TRIPLE}] [--emit llvm-ir|object|executable] [--llvm-tools DIR] -o OUTPUT SOURCE\n       topalc test [--list | --exact ID] [--llvm-tools DIR]\n\n-O0 and full DWARF debugging are the mandatory semantics of this compiler increment. Executables are static PIEs with a Topal Linux syscall runtime and no C/C++ runtime dependency."
                );
                std::process::exit(0);
            }
            option if option.starts_with('-') => return Err(format!("unknown option: {option}")),
            path if source.is_none() => source = Some(PathBuf::from(path)),
            path => return Err(format!("unexpected input path: {path}")),
        }
    }
    let source = source.ok_or("a Topal source file is required")?;
    let output = output.unwrap_or_else(|| default_output(&source, emit));
    Ok(Arguments {
        source,
        output,
        emit,
        llvm_tools,
    })
}

fn default_output(source: &Path, emit: Emit) -> PathBuf {
    match emit {
        Emit::Executable => source.with_extension(""),
        Emit::Object => source.with_extension("o"),
        Emit::LlvmIr => source.with_extension("ll"),
    }
}

fn render_error(error: CompileError, source_name: &str) -> String {
    match error {
        CompileError::Diagnostic(diagnostic) => diagnostic.render(source_name),
        error => error.to_string(),
    }
}
