use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use topal_language::{Session, declares_library, load_module_tree};

struct Arguments {
    paths: Vec<PathBuf>,
    library_root: PathBuf,
    exact: Option<String>,
    filter: Option<String>,
    list: bool,
    jobs: usize,
}

#[derive(Debug)]
struct Outcome {
    identity: String,
    failure: Option<String>,
}

pub(crate) fn run(arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let arguments = parse_arguments(arguments)?;
    let mut tests = discover(&arguments.paths)?;
    let working_directory = env::current_dir().map_err(|error| error.to_string())?;
    tests = tests
        .into_iter()
        .filter(|path| {
            let identity = identity(path, &working_directory);
            arguments
                .exact
                .as_ref()
                .is_none_or(|exact| exact == &identity)
                && arguments
                    .filter
                    .as_ref()
                    .is_none_or(|filter| identity.contains(filter))
        })
        .collect();
    if tests.is_empty() {
        return Err("no Topal tests matched".into());
    }
    if arguments.list {
        for path in tests {
            println!("{}", identity(&path, &working_directory));
        }
        return Ok(());
    }

    let worker_count = arguments.jobs.min(tests.len());
    let (sender, receiver) = mpsc::channel();
    let next = std::sync::Arc::new(std::sync::Mutex::new(tests.into_iter()));
    thread::scope(|scope| {
        for _ in 0..worker_count {
            let sender = sender.clone();
            let next = next.clone();
            let library_root = arguments.library_root.clone();
            let working_directory = working_directory.clone();
            thread::Builder::new()
                .name("topal-test".into())
                .stack_size(32 * 1024 * 1024)
                .spawn_scoped(scope, move || {
                    loop {
                        let Some(path) = next.lock().expect("test queue is available").next()
                        else {
                            break;
                        };
                        let outcome = execute(&path, &library_root, &working_directory);
                        sender
                            .send(outcome)
                            .expect("test result receiver is available");
                    }
                })
                .expect("Topal test worker can start");
        }
    });
    drop(sender);
    let mut outcomes = receiver.into_iter().collect::<Vec<_>>();
    outcomes.sort_by(|left, right| left.identity.cmp(&right.identity));
    let mut failed = 0;
    for outcome in &outcomes {
        if let Some(failure) = &outcome.failure {
            failed += 1;
            println!("FAIL {}", outcome.identity);
            eprintln!("{failure}");
        } else {
            println!("PASS {}", outcome.identity);
        }
    }
    println!(
        "{} Topal tests: {} passed; {} failed",
        outcomes.len(),
        outcomes.len() - failed,
        failed
    );
    if failed == 0 {
        Ok(())
    } else {
        Err("Topal test run failed".into())
    }
}

fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<Arguments, String> {
    let mut paths = Vec::new();
    let mut library_root =
        env::var_os("TOPAL_LIBRARY_ROOT").map_or_else(|| PathBuf::from("library"), PathBuf::from);
    let mut exact = None;
    let mut filter = None;
    let mut list = false;
    let mut jobs = thread::available_parallelism().map_or(1, usize::from);
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--library-root" => {
                library_root = PathBuf::from(
                    arguments
                        .next()
                        .ok_or("--library-root requires a directory")?,
                )
            }
            "--exact" => exact = Some(arguments.next().ok_or("--exact requires a test identity")?),
            "--filter" => filter = Some(arguments.next().ok_or("--filter requires text")?),
            "--jobs" => {
                jobs = arguments
                    .next()
                    .ok_or("--jobs requires a positive count")?
                    .parse()
                    .map_err(|_| "--jobs requires a positive count")?;
                if jobs == 0 {
                    return Err("--jobs requires a positive count".into());
                }
            }
            "--list" => list = true,
            "--help" => {
                println!(
                    "Usage: topal test [--library-root DIR] [--exact NAME | --filter TEXT] [--jobs COUNT] [--list] [PATH...]\n\nDiscover .t files recursively. Each file is one independently reported Topal test. PATH defaults to tests."
                );
                std::process::exit(0);
            }
            option if option.starts_with('-') => {
                return Err(format!("unknown test option: {option}"));
            }
            path => paths.push(PathBuf::from(path)),
        }
    }
    if exact.is_some() && filter.is_some() {
        return Err("--exact and --filter are mutually exclusive".into());
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("tests"));
    }
    Ok(Arguments {
        paths,
        library_root,
        exact,
        filter,
        list,
        jobs,
    })
}

fn discover(roots: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    fn visit(path: &Path, tests: &mut Vec<PathBuf>) -> Result<(), String> {
        if path.is_file() {
            if path.extension().is_some_and(|extension| extension == "t") {
                tests.push(path.to_owned());
            }
            return Ok(());
        }
        if !path.is_dir() {
            return Err(format!(
                "Topal test path does not exist: {}",
                path.display()
            ));
        }
        for entry in fs::read_dir(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?
        {
            visit(&entry.map_err(|error| error.to_string())?.path(), tests)?;
        }
        Ok(())
    }
    let mut tests = Vec::new();
    for root in roots {
        visit(root, &mut tests)?;
    }
    tests.sort();
    tests.dedup();
    Ok(tests)
}

fn identity(path: &Path, working_directory: &Path) -> String {
    path.strip_prefix(working_directory)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn execute(path: &Path, library_root: &Path, working_directory: &Path) -> Outcome {
    let identity = identity(path, working_directory);
    let failure = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
        .and_then(|source| {
            let mut session = Session::new();
            let mut trace = Vec::new();
            if declares_library(&source, "std") {
                load_module_tree(&mut session, library_root, &mut trace)
                    .map_err(|error| format!("{identity}: {error}"))?;
            }
            session
                .evaluate_source_file(&source, &mut trace)
                .map(|_| ())
                .map_err(|error| error.render(&identity))
        })
        .err();
    Outcome { identity, failure }
}
