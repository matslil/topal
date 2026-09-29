use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use topal_compiler::{
    CompileError, CompileOptions, Emit, ExplanationDestination, OptimizationLevel,
    OptimizationOverride, OptimizationRequest, TARGET_TRIPLE, TargetSelection, compile_source,
    optimization_listing,
};

mod test_runner;

fn main() -> ExitCode {
    let result = std::thread::Builder::new()
        .name("topalc".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(run)
        .map_err(|error| format!("cannot start compiler worker: {error}"))
        .and_then(|worker| {
            worker
                .join()
                .map_err(|_| "compiler worker panicked".to_owned())?
        });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.as_slice() == ["--list-optimizations"] {
        print!("{}", optimization_listing());
        return Ok(());
    }
    let mut arguments = arguments.into_iter().peekable();
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
        library_root: arguments.library_root,
        target: arguments.target,
        optimization: arguments.optimization,
    };
    compile_source(&source, &options).map_err(|error| render_error(error, &source_name))?;
    Ok(())
}

struct Arguments {
    source: PathBuf,
    output: PathBuf,
    emit: Emit,
    llvm_tools: Option<PathBuf>,
    library_root: PathBuf,
    target: TargetSelection,
    optimization: OptimizationRequest,
}

fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<Arguments, String> {
    let mut source = None;
    let mut output = None;
    let mut emit = Emit::Executable;
    let mut llvm_tools = None;
    let mut library_root =
        env::var_os("TOPAL_LIBRARY_ROOT").map_or_else(|| PathBuf::from("library"), PathBuf::from);
    let mut target = TargetSelection::default();
    let mut optimization_level = None;
    let mut optimization_goals = Vec::new();
    let mut optimization_limits = Vec::new();
    let mut optimization_overrides = Vec::new();
    let mut only_optimization = None;
    let mut explain_optimizations = None;
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-O0" | "-O1" | "-O2" | "-O3" | "-Os" | "-Oz" => {
                let selected = match argument.as_str() {
                    "-O0" => OptimizationLevel::O0,
                    "-O1" => OptimizationLevel::O1,
                    "-O2" => OptimizationLevel::O2,
                    "-O3" => OptimizationLevel::O3,
                    "-Os" => OptimizationLevel::Os,
                    "-Oz" => OptimizationLevel::Oz,
                    _ => unreachable!(),
                };
                if optimization_level.replace(selected).is_some() {
                    return Err("only one standard optimization profile may be selected".into());
                }
            }
            "-g" => {}
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
                if target.target.replace(value).is_some() {
                    return Err("--target may be specified only once".into());
                }
            }
            "--cpu" => {
                let value = arguments.next().ok_or("--cpu requires a profile")?;
                if target.cpu.replace(value).is_some() {
                    return Err("--cpu may be specified only once".into());
                }
            }
            "--board" => {
                let value = arguments.next().ok_or("--board requires a profile")?;
                if target.board.replace(value).is_some() {
                    return Err("--board may be specified only once".into());
                }
            }
            "--target-model" => {
                let value =
                    PathBuf::from(arguments.next().ok_or("--target-model requires a path")?);
                if target.model.replace(value).is_some() {
                    return Err("--target-model may be specified only once".into());
                }
            }
            "--optimization-goal" => optimization_goals.push(
                arguments
                    .next()
                    .ok_or("--optimization-goal requires a dimension")?,
            ),
            "--optimization-limit" => optimization_limits.push(
                arguments
                    .next()
                    .ok_or("--optimization-limit requires DIMENSION=QUANTITY")?,
            ),
            "--enable-optimization" => optimization_overrides.push(OptimizationOverride::Enable(
                arguments
                    .next()
                    .ok_or("--enable-optimization requires a stable ID")?,
            )),
            "--disable-optimization" => optimization_overrides.push(OptimizationOverride::Disable(
                arguments
                    .next()
                    .ok_or("--disable-optimization requires a stable ID")?,
            )),
            "--only-optimization" => {
                let identity = arguments
                    .next()
                    .ok_or("--only-optimization requires a stable ID")?;
                if only_optimization.replace(identity).is_some() {
                    return Err("--only-optimization may be specified only once".into());
                }
            }
            "--explain-optimizations" => {
                if explain_optimizations
                    .replace(ExplanationDestination::StandardError)
                    .is_some()
                {
                    return Err("--explain-optimizations may be specified only once".into());
                }
            }
            "--llvm-tools" => {
                llvm_tools = Some(PathBuf::from(
                    arguments
                        .next()
                        .ok_or("--llvm-tools requires a directory")?,
                ));
            }
            "--library-root" => {
                library_root = PathBuf::from(
                    arguments
                        .next()
                        .ok_or("--library-root requires a directory")?,
                );
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
                    "Usage: topalc [-O0|-O1|-O2|-O3|-Os|-Oz] [-g] [--target TRIPLE] [--cpu PROFILE] [--board PROFILE] [--target-model PATH] [--optimization-goal DIMENSION] [--optimization-limit DIMENSION=QUANTITY] [--enable-optimization ID | --disable-optimization ID | --only-optimization ID] [--explain-optimizations[=PATH]] [--emit llvm-ir|object|executable] [--llvm-tools DIR] [--library-root DIR] -o OUTPUT SOURCE\n       topalc --list-optimizations\n       topalc test [--list | --exact ID] [--llvm-tools DIR]\n\nThe default is the generic host-family target at -O0. This increment qualifies only {TARGET_TRIPLE} with --cpu generic. O1 isolates private runtime pruning; O2, O3, Os, and Oz select the matching LLVM 22 default pipeline. Executables are static PIEs with a Topal Linux syscall runtime and no C/C++ runtime dependency."
                );
                std::process::exit(0);
            }
            option if option.starts_with("--explain-optimizations=") => {
                let path = option.trim_start_matches("--explain-optimizations=");
                if path.is_empty() {
                    return Err("--explain-optimizations requires a nonempty path after =".into());
                }
                if explain_optimizations
                    .replace(ExplanationDestination::Path(path.into()))
                    .is_some()
                {
                    return Err("--explain-optimizations may be specified only once".into());
                }
            }
            "--list-optimizations" => {
                return Err("--list-optimizations must be used without other arguments".into());
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
        library_root,
        target,
        optimization: OptimizationRequest {
            level: optimization_level.unwrap_or_default(),
            explicit_level: optimization_level.is_some(),
            goals: optimization_goals,
            limits: optimization_limits,
            overrides: optimization_overrides,
            only: only_optimization,
            explain: explain_optimizations,
        },
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
