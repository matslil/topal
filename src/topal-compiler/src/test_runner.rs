use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use topal_compiler::{CompileError, CompileOptions, Emit, compile_source};
use topal_language::Session;

const SHARED_REGRESSIONS: &[&str] = &[
    "examples/language/bindings-and-discard.t",
    "examples/language/boolean-decisions.t",
    "examples/language/boolean-logic.t",
    "examples/language/function-call-chains.t",
    "examples/language/function-local-shadowing.t",
    "examples/language/ordinary-functions.t",
];

pub(crate) fn run(arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let arguments = Arguments::parse(arguments)?;
    let root = env::current_dir().map_err(|error| error.to_string())?;
    let tests = SHARED_REGRESSIONS
        .iter()
        .filter(|identity| {
            arguments
                .exact
                .as_deref()
                .is_none_or(|exact| exact == **identity)
        })
        .copied()
        .collect::<Vec<_>>();
    if tests.is_empty() {
        return Err("no compiler regressions matched".into());
    }
    if arguments.list {
        for identity in tests {
            println!("{identity}");
        }
        return Ok(());
    }

    let temporary = root.join("target/topalc-regressions");
    fs::create_dir_all(&temporary)
        .map_err(|error| format!("cannot create {}: {error}", temporary.display()))?;
    let mut failures = Vec::new();
    for identity in &tests {
        if let Err(error) = execute(identity, &root, &temporary, arguments.llvm_tools.as_deref()) {
            failures.push((identity, error));
        } else {
            println!("PASS {identity}");
        }
    }
    for (identity, error) in &failures {
        println!("FAIL {identity}");
        eprintln!("{error}");
    }
    println!(
        "{} compiler regressions: {} passed; {} failed",
        tests.len(),
        tests.len() - failures.len(),
        failures.len()
    );
    if failures.is_empty() {
        Ok(())
    } else {
        Err("compiler regression run failed".into())
    }
}

struct Arguments {
    exact: Option<String>,
    list: bool,
    llvm_tools: Option<PathBuf>,
}

impl Arguments {
    fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut exact = None;
        let mut list = false;
        let mut llvm_tools = None;
        let mut arguments = arguments.peekable();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--exact" => exact = Some(arguments.next().ok_or("--exact requires an identity")?),
                "--list" => list = true,
                "--llvm-tools" => {
                    llvm_tools = Some(PathBuf::from(
                        arguments
                            .next()
                            .ok_or("--llvm-tools requires a directory")?,
                    ));
                }
                option => return Err(format!("unknown compiler-test option: {option}")),
            }
        }
        Ok(Self {
            exact,
            list,
            llvm_tools,
        })
    }
}

fn execute(
    identity: &str,
    root: &Path,
    temporary: &Path,
    llvm_tools: Option<&Path>,
) -> Result<(), String> {
    let path = root.join(identity);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let expected = Session::new()
        .evaluate_source_file(&source, &mut std::io::sink())
        .map_err(|diagnostic| diagnostic.render(identity))?
        .to_string()
        + "\n";
    let executable = temporary.join(identity.replace(['/', '\\'], "-") + ".bin");
    let options = CompileOptions {
        source_name: identity.into(),
        output: executable.clone(),
        emit: Emit::Executable,
        llvm_tools: llvm_tools.map(Path::to_owned),
    };
    compile_source(&source, &options).map_err(|error| render_error(error, identity))?;
    let output = Command::new(&executable)
        .output()
        .map_err(|error| format!("cannot execute {}: {error}", executable.display()))?;
    if !output.status.success() {
        return Err(format!(
            "compiled executable exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let actual = String::from_utf8(output.stdout)
        .map_err(|error| format!("compiled executable emitted non-UTF-8 output: {error}"))?;
    if actual != expected {
        return Err(format!(
            "interpreter: {expected:?}\ncompiler:    {actual:?}"
        ));
    }
    Ok(())
}

fn render_error(error: CompileError, identity: &str) -> String {
    match error {
        CompileError::Diagnostic(diagnostic) => diagnostic.render(identity),
        error => error.to_string(),
    }
}
