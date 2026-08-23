use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, BufRead, Cursor, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};

use topal_language::{
    Execution, ExecutionHistory, ExecutionStep, ExecutionTransition, Session, TraceSink,
    declares_library, lang_documentation, load_module_tree,
};
use topal_source::SourceText;
use topal_syntax::{DocumentedDeclaration, Statement, extract_documentation, lex, parse};

static HUMAN_OUTPUT: AtomicBool = AtomicBool::new(false);

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
    let (source, source_name, mut debuggee) = prepare_debuggee(&arguments)?;
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupt_request = Arc::clone(&interrupted);
    ctrlc::set_handler(move || interrupt_request.store(true, Ordering::SeqCst))
        .map_err(|error| format!("cannot install interrupt handler: {error}"))?;

    println!(
        "loaded {} transitions",
        debuggee.history.transitions().len()
    );
    print_initial_position(&source, &source_name);
    if let Some(commands) = &arguments.commands {
        if commands == "-" {
            let mut script = String::new();
            io::stdin().read_to_string(&mut script).map_err(|error| {
                format!("cannot read debugger script from standard input: {error}")
            })?;
            let (body, header_lines) = debug_script_body(&script, "<stdin>")?;
            command_loop(
                Cursor::new(body),
                &mut debuggee,
                &source,
                &source_name,
                &arguments,
                &interrupted,
                Some("<stdin>"),
                header_lines,
            )
        } else {
            let script = fs::read_to_string(commands)
                .map_err(|error| format!("cannot read command script {commands}: {error}"))?;
            let (body, header_lines) = debug_script_body(&script, commands)?;
            command_loop(
                Cursor::new(body),
                &mut debuggee,
                &source,
                &source_name,
                &arguments,
                &interrupted,
                Some(commands),
                header_lines,
            )
        }
    } else if io::stdin().is_terminal() {
        interactive_command_loop(
            &mut debuggee,
            &source,
            &source_name,
            &arguments,
            &interrupted,
        )
    } else {
        command_loop(
            io::stdin().lock(),
            &mut debuggee,
            &source,
            &source_name,
            &arguments,
            &interrupted,
            None,
            0,
        )
    }
}

fn prepare_debuggee(arguments: &Arguments) -> Result<(String, String, Debuggee), String> {
    let source_path = source_entry(&arguments.source)?;
    let source_name = source_path.display().to_string();
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("cannot read {source_name}: {error}"))?;
    let mut session = Session::new();
    let mut history = ExecutionHistory::new();
    if Path::new(&arguments.source).is_dir() {
        load_module_tree(&mut session, Path::new(&arguments.source), &mut history)?;
    } else if declares_library(&source, "std") {
        load_module_tree(&mut session, &arguments.library_root, &mut history)?;
    }
    let documentation_root = if Path::new(&arguments.source).is_dir() {
        Some(Path::new(&arguments.source))
    } else if arguments.library_root.is_dir() {
        Some(arguments.library_root.as_path())
    } else {
        None
    };
    let documentation = debugger_documentation(&source, documentation_root)?;
    let current_source_start = history.transitions().len();
    history.push_source(&source_name, &source);
    let execution = session
        .prepare_source_file(&source, &mut history)
        .map_err(|error| error.render(&source_name))?;
    history.rewind();
    let debuggee = Debuggee {
        session,
        execution,
        history,
        current_source_start,
        dependency_frame: None,
        documentation,
        complete: false,
    };

    Ok((source, source_name, debuggee))
}

fn source_entry(source: &str) -> Result<PathBuf, String> {
    let path = Path::new(source);
    if !path.is_dir() {
        return Ok(path.to_owned());
    }
    let entry = path.join("application.t");
    if entry.is_file() {
        Ok(entry)
    } else {
        Err(format!("{} has no application.t", path.display()))
    }
}

#[derive(Clone)]
struct Arguments {
    source: String,
    commands: Option<String>,
    library_root: PathBuf,
}

fn debug_script_body(script: &str, name: &str) -> Result<(String, usize), String> {
    let source = SourceText::new(script).map_err(|error| format!("{name}: {error}"))?;
    let parsed = parse(&source, &lex(&source));
    if let Some(error) = parsed.diagnostics.first() {
        let position = source.position(error.span.start);
        return Err(format!(
            "{name}:{}:{}: error[{}]: {}",
            position.line, position.column, error.code, error.message
        ));
    }
    let Some(Statement::LanguageSelection {
        version,
        features,
        span,
    }) = parsed.statements.first()
    else {
        return Err(format!(
            "{name}:1:1: error[D-MISSING-DEBUG-LANGUAGE]: debugger scripts begin with `use language ( version is v0.1, features is ( debug ) )`"
        ));
    };
    if source.slice(*version) != "v0.1" {
        let position = source.position(version.start);
        return Err(format!(
            "{name}:{}:{}: error[D-DEBUG-LANGUAGE-VERSION]: debugger supports language version v0.1",
            position.line, position.column
        ));
    }
    if !features
        .iter()
        .any(|feature| source.slice(*feature) == "debug")
    {
        let position = source.position(span.start);
        return Err(format!(
            "{name}:{}:{}: error[D-MISSING-DEBUG-VARIANT]: debugger scripts select the `debug` language feature",
            position.line, position.column
        ));
    }
    let header_lines = source.position(span.end).line;
    let body = source.as_str()[span.end..]
        .strip_prefix('\n')
        .unwrap_or(&source.as_str()[span.end..])
        .to_owned();
    Ok((body, header_lines))
}

struct Debuggee {
    session: Session,
    execution: Execution,
    history: ExecutionHistory,
    current_source_start: usize,
    dependency_frame: Option<SourceFrame>,
    documentation: Vec<DocumentedDeclaration>,
    complete: bool,
}

#[derive(Clone)]
struct SourceFrame {
    return_cursor: usize,
    caller_name: String,
    caller_source: String,
    caller_range: topal_language::SourceRange,
}

type Breakpoint = (String, usize);

struct DebugHelper;

impl Helper for DebugHelper {}
impl Highlighter for DebugHelper {}
impl Validator for DebugHelper {}

impl Hinter for DebugHelper {
    type Hint = String;

    fn hint(&self, line: &str, _position: usize, _context: &Context<'_>) -> Option<String> {
        let command = line.trim();
        command_argument_hint(command).map(|hint| format!(" {hint}"))
    }
}

impl Completer for DebugHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        position: usize,
        _context: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let before = &line[..position];
        if let Some((command, argument)) = before.split_once(' ') {
            let candidates = argument_candidates(command)
                .into_iter()
                .filter(|candidate| candidate.starts_with(argument))
                .map(|candidate| Pair {
                    display: candidate.to_owned(),
                    replacement: candidate.to_owned(),
                })
                .collect();
            return Ok((command.len() + 1, candidates));
        }
        let candidates = DEBUG_COMMANDS
            .iter()
            .filter(|command| command.starts_with(before))
            .map(|command| Pair {
                display: (*command).to_owned(),
                replacement: (*command).to_owned(),
            })
            .collect();
        Ok((0, candidates))
    }
}

fn command_argument_hint(command: &str) -> Option<&'static str> {
    Some(match command {
        "break" | "delete" => "LINE",
        "watch" | "unwatch" | "checkpoint" | "restore" | "delete-checkpoint" => "NAME",
        "help" => "COMMAND or identifier",
        "print" => "EXPRESSION",
        "until" => "[LINE | FILE:LINE | CONDITION]",
        _ => return None,
    })
}

fn argument_candidates(command: &str) -> Vec<&'static str> {
    match command {
        "help" => DEBUG_COMMANDS.to_vec(),
        "until" => vec!["<LINE>", "<FILE:LINE>", "<CONDITION>"],
        command => command_argument_hint(command).into_iter().collect(),
    }
}

struct InteractiveInput {
    editor: Editor<DebugHelper, DefaultHistory>,
    buffer: Vec<u8>,
    position: usize,
    unique_history: Vec<String>,
    last_progressing: Option<String>,
    eof: bool,
}

impl InteractiveInput {
    fn new() -> Result<Self, String> {
        let mut editor = Editor::new().map_err(|error| error.to_string())?;
        editor.set_helper(Some(DebugHelper));
        Ok(Self {
            editor,
            buffer: Vec::new(),
            position: 0,
            unique_history: Vec::new(),
            last_progressing: None,
            eof: false,
        })
    }

    fn refill(&mut self) -> io::Result<()> {
        if self.eof || self.position < self.buffer.len() {
            return Ok(());
        }
        self.buffer.clear();
        self.position = 0;
        let entered = match self.editor.readline("(topal-debug) ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => String::new(),
            Err(ReadlineError::Eof) => {
                self.eof = true;
                return Ok(());
            }
            Err(error) => return Err(io::Error::other(error.to_string())),
        };
        let trimmed = entered.trim();
        let command = if trimmed.is_empty() {
            self.last_progressing.clone().unwrap_or_default()
        } else {
            self.unique_history.retain(|previous| previous != trimmed);
            self.unique_history.push(trimmed.to_owned());
            self.editor.clear_history().map_err(io::Error::other)?;
            for item in &self.unique_history {
                self.editor
                    .add_history_entry(item)
                    .map_err(io::Error::other)?;
            }
            let resolved = resolve_prompt_command(trimmed).unwrap_or_else(|_| trimmed.to_owned());
            if is_progressing_command(&resolved) {
                self.last_progressing = Some(trimmed.to_owned());
            }
            entered
        };
        self.buffer.extend_from_slice(command.as_bytes());
        self.buffer.push(b'\n');
        Ok(())
    }
}

impl Read for InteractiveInput {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.refill()?;
        let available = &self.buffer[self.position..];
        let count = available.len().min(output.len());
        output[..count].copy_from_slice(&available[..count]);
        self.position += count;
        Ok(count)
    }
}

impl BufRead for InteractiveInput {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.refill()?;
        Ok(&self.buffer[self.position..])
    }

    fn consume(&mut self, amount: usize) {
        self.position = (self.position + amount).min(self.buffer.len());
    }
}

fn is_progressing_command(command: &str) -> bool {
    matches!(
        command.split_whitespace().next().unwrap_or(""),
        "step"
            | "reverse-step"
            | "source-step"
            | "expression-step"
            | "reverse-source-step"
            | "next"
            | "reverse-next"
            | "finish"
            | "reverse-finish"
            | "continue"
            | "reverse-continue"
            | "until"
            | "run"
    )
}

fn interactive_command_loop(
    debuggee: &mut Debuggee,
    source: &str,
    source_name: &str,
    arguments: &Arguments,
    interrupted: &Arc<AtomicBool>,
) -> Result<(), String> {
    HUMAN_OUTPUT.store(true, Ordering::Relaxed);
    command_loop(
        InteractiveInput::new()?,
        debuggee,
        source,
        source_name,
        arguments,
        interrupted,
        None,
        0,
    )
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Arguments, String> {
    let mut commands = None;
    let mut source = None;
    let mut library_root =
        env::var_os("TOPAL_LIBRARY_ROOT").map_or_else(|| PathBuf::from("library"), PathBuf::from);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--script" => {
                commands = Some(
                    arguments
                        .next()
                        .ok_or_else(|| "--script requires a command file or -".to_owned())?,
                );
            }
            "--library-root" => {
                library_root = PathBuf::from(
                    arguments
                        .next()
                        .ok_or_else(|| "--library-root requires a directory".to_owned())?,
                );
            }
            option if option.starts_with('-') && option != "-" => {
                return Err(format!("unknown option: {option}"));
            }
            value if source.replace(value.to_owned()).is_some() => {
                return Err(format!("unexpected second source file: {value}"));
            }
            _ => {}
        }
    }
    let source = source.ok_or_else(|| {
        "usage: topal-debug [--library-root DIR] [--script COMMANDS] FILE".to_owned()
    })?;
    Ok(Arguments {
        source,
        commands,
        library_root,
    })
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // The dispatcher owns one coherent debug session.
fn command_loop(
    mut input: impl BufRead,
    debuggee: &mut Debuggee,
    source: &str,
    source_name: &str,
    arguments: &Arguments,
    interrupted: &Arc<AtomicBool>,
    script_name: Option<&str>,
    initial_line_number: usize,
) -> Result<(), String> {
    let Debuggee {
        session,
        execution,
        history,
        current_source_start,
        dependency_frame,
        documentation,
        complete,
    } = debuggee;
    let mut breakpoints = BTreeSet::new();
    let mut watchpoints = BTreeSet::new();
    let mut checkpoints = BTreeMap::new();
    let mut line_number = initial_line_number;
    loop {
        let Some(command) = read_command(&mut input, false)? else {
            return Ok(());
        };
        line_number += 1;
        let command = if script_name.is_none() {
            match resolve_prompt_command(command.trim()) {
                Ok(command) => command,
                Err(message) => {
                    println!("{message}");
                    continue;
                }
            }
        } else {
            let command = resolve_script_command(command.trim());
            reject_script_shortcut(&command, script_name, line_number)?;
            command
        };
        match command.as_str() {
            command if command.starts_with("# ") => {}
            command
                if handle_source_command(
                    command,
                    history,
                    source,
                    source_name,
                    session,
                    execution,
                    complete,
                ) => {}
            command
                if handle_frame_command(
                    command,
                    history,
                    source,
                    source_name,
                    session,
                    execution,
                    *current_source_start,
                    dependency_frame,
                    complete,
                ) => {}
            command
                if handle_continue_command(
                    command,
                    history,
                    source,
                    source_name,
                    session,
                    execution,
                    complete,
                    &breakpoints,
                    &watchpoints,
                    interrupted,
                ) => {}
            command if command == "until" || command.starts_with("until ") => {
                handle_until_command(
                    command,
                    history,
                    source,
                    source_name,
                    session,
                    execution,
                    complete,
                    dependency_frame,
                    interrupted,
                );
            }
            "run" => match prepare_debuggee(arguments) {
                Ok((_, _, restarted)) => {
                    *session = restarted.session;
                    *execution = restarted.execution;
                    *history = restarted.history;
                    *current_source_start = restarted.current_source_start;
                    *dependency_frame = None;
                    *documentation = restarted.documentation;
                    *complete = false;
                    print_initial_position(source, source_name);
                }
                Err(error) => println!("cannot restart application: {error}"),
            },
            "step" | "s" => {
                if dependency_frame.is_some() || at_library_selection(history, source, source_name)
                {
                    if dependency_frame.is_none() {
                        *dependency_frame = source_frame(history, source, source_name);
                        history.rewind();
                    }
                    match history.step_source_forward() {
                        Some(_) => {
                            if history.cursor()
                                >= dependency_frame
                                    .as_ref()
                                    .map_or(*current_source_start, |frame| frame.return_cursor)
                            {
                                *dependency_frame = None;
                            }
                            print_source_location(history, source, source_name);
                        }
                        None => println!("end of dependency source"),
                    }
                } else {
                    match history.step_forward() {
                        Some(transition) => print_transition(transition),
                        None => println!("end of execution"),
                    }
                }
            }
            "reverse-step" | "rs" => {
                if let Some(frame) = dependency_frame.as_ref() {
                    let return_cursor = frame.return_cursor;
                    if history.step_source_backward().is_some() {
                        print_source_location(history, source, source_name);
                    } else {
                        history.seek(return_cursor);
                        *dependency_frame = None;
                        print_source_location(history, source, source_name);
                    }
                } else if history.cursor() == 0 {
                    println!("start of execution");
                } else {
                    history.step_backward();
                    match history.current() {
                        Some(transition) => print_transition(transition),
                        None => println!("before first transition"),
                    }
                }
            }
            "history" => print_history(history),
            "why" => print_reason(history),
            "breakpoints" => {
                print_breakpoints(&breakpoints);
            }
            "watchpoints" => {
                print_watchpoints(&watchpoints);
            }
            "checkpoints" => {
                print_checkpoints(&checkpoints);
            }
            command if command.starts_with("break ") => {
                let current_name = current_source_name(history, source_name);
                update_breakpoint(command, "break ", &current_name, &mut breakpoints, true);
            }
            command if command.starts_with("delete ") => {
                let current_name = current_source_name(history, source_name);
                update_breakpoint(command, "delete ", &current_name, &mut breakpoints, false);
            }
            command if command.starts_with("watch ") => {
                update_watchpoint(command, "watch ", &mut watchpoints, true);
            }
            command if command.starts_with("unwatch ") => {
                update_watchpoint(command, "unwatch ", &mut watchpoints, false);
            }
            command if command.starts_with("checkpoint ") => {
                save_checkpoint(command, &mut checkpoints, history.cursor());
            }
            command if command.starts_with("restore ") => {
                restore_checkpoint(command, &checkpoints, history, source, source_name);
            }
            command if command.starts_with("delete-checkpoint ") => {
                delete_checkpoint(command, &mut checkpoints);
            }
            "print" | "p" => match history.state().and_then(|state| state.value) {
                Some(value) => println!("{value}"),
                None => println!("no value at current execution state"),
            },
            command if command.starts_with("print ") || command.starts_with("p ") => {
                let expression = command.split_once(' ').unwrap().1.trim();
                print_expression(history, expression);
            }
            "bindings" => print_bindings(history),
            command if command.starts_with("help ") => {
                print_identifier_help(command[5..].trim(), documentation);
            }
            "help" | "h" => {
                println!(
                    "step | reverse-step | source-step | reverse-source-step | next | reverse-next | finish | reverse-finish | until [LINE|FILE:LINE|CONDITION] | run | backtrace | break LINE | delete LINE | breakpoints | watch NAME | unwatch NAME | watchpoints | continue | reverse-continue | checkpoint NAME | restore NAME | checkpoints | delete-checkpoint NAME | where | why | history | print | bindings | quit"
                );
                println!("step enters use-clause work; next stays in the current source file");
                println!("expression-step (es) advances to the next recorded expression state");
            }
            "quit" | "q" => return Ok(()),
            "" => {}
            unknown => {
                if let Some(script_name) = script_name {
                    return Err(format!(
                        "{script_name}:{line_number}: error[D-UNKNOWN-COMMAND]: unknown debugger command `{unknown}`; use `help`"
                    ));
                }
                println!("unknown command: {unknown}; use help");
            }
        }
        io::stdout().flush().map_err(|error| error.to_string())?;
    }
}

fn print_identifier_help(identifier: &str, declarations: &[DocumentedDeclaration]) {
    if let Some(documentation) = debugger_command_documentation(identifier) {
        println!("{identifier}: {documentation}");
        return;
    }
    let qualified = identifier.contains(' ');
    let matches = declarations
        .iter()
        .filter(|declaration| {
            declaration.name == identifier
                || (!qualified
                    && declaration
                        .name
                        .split_whitespace()
                        .next_back()
                        .is_some_and(|name| name == identifier))
        })
        .collect::<Vec<_>>();
    if matches.is_empty() {
        println!("no documentation for `{identifier}`");
        return;
    }
    let distinct_names = matches
        .iter()
        .map(|entry| &entry.name)
        .collect::<BTreeSet<_>>();
    if distinct_names.len() > 1 {
        println!("ambiguous identifier `{identifier}`; candidates:");
        for name in distinct_names {
            println!("  {name}");
        }
        return;
    }
    for declaration in matches {
        println!("{}", declaration.syntax);
        if let Some(documentation) = &declaration.documentation {
            println!("\n{documentation}");
        }
        for parameter in &declaration.parameters {
            if let Some(documentation) = &parameter.documentation {
                println!("\n{}: {documentation}", parameter.name);
            }
        }
    }
}

fn debugger_documentation(
    source: &str,
    library_root: Option<&Path>,
) -> Result<Vec<DocumentedDeclaration>, String> {
    let mut declarations = documented_source(source);
    if let Some(root) = library_root {
        collect_module_documentation(root, root, &mut declarations)?;
    }
    declarations.extend(lang_documentation());
    Ok(declarations)
}

fn collect_module_documentation(
    root: &Path,
    directory: &Path,
    declarations: &mut Vec<DocumentedDeclaration>,
) -> Result<(), String> {
    let mut paths = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    paths.sort();
    for path in paths {
        if path.is_dir() {
            collect_module_documentation(root, &path, declarations)?;
        } else if path.extension().is_some_and(|extension| extension == "t") {
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("application.t" | "package.t" | "library.t")
            ) {
                continue;
            }
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            let relative = path.strip_prefix(root).unwrap_or(&path);
            let mut namespace = relative
                .parent()
                .into_iter()
                .flat_map(Path::components)
                .map(|component| component.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            if path.file_name().and_then(|name| name.to_str()) != Some("module.t")
                && let Some(stem) = path.file_stem()
            {
                namespace.push(stem.to_string_lossy().into_owned());
            }
            for mut declaration in documented_source(&text) {
                if !namespace.is_empty() {
                    declaration.name = format!("{} {}", namespace.join(" "), declaration.name);
                }
                declarations.push(declaration);
            }
        }
    }
    Ok(())
}

fn debugger_command_documentation(command: &str) -> Option<&'static str> {
    Some(match command {
        "step" => "enter the current source clause, including a selected dependency",
        "reverse-step" => "move to the preceding step, returning through source frames",
        "next" => "advance to the next location in the current source file",
        "reverse-next" => "move to the preceding location in the current source file",
        "source-step" | "expression-step" => "advance to the next recorded source expression",
        "reverse-source-step" => "move to the preceding recorded source expression",
        "finish" => "finish the current source frame and return to its caller",
        "reverse-finish" => "rewind to the start of source execution",
        "continue" | "reverse-continue" => "run to a source breakpoint or binding watchpoint",
        "until" => {
            "without an argument, leave the current source frame; otherwise run to a source line or true Boolean expression"
        }
        "run" => "restart the application and stop at its initial source position",
        "backtrace" => "show the current source frame and its callers",
        "break" => "break LINE sets a breakpoint in the current source file",
        "delete" => "delete LINE removes a breakpoint in the current source file",
        "breakpoints" => "list source-qualified breakpoints",
        "watch" | "unwatch" | "watchpoints" => "manage binding-change watchpoints",
        "checkpoint" | "restore" | "checkpoints" | "delete-checkpoint" => {
            "manage named execution-history positions"
        }
        "where" => "show the current source location",
        "why" => "explain the current semantic decision",
        "history" => "list recorded semantic decisions",
        "print" => "print the current value or evaluate a read-only Topal expression",
        "bindings" => "list bindings visible at the current execution state",
        "help" => "list commands, or use help NAME for command or declaration documentation",
        "quit" => "leave the debugger",
        _ => return None,
    })
}

fn documented_source(text: &str) -> Vec<DocumentedDeclaration> {
    let Ok(source) = SourceText::new(text) else {
        return Vec::new();
    };
    let lexed = lex(&source);
    let parsed = parse(&source, &lexed);
    extract_documentation(&source, &lexed, &parsed)
}

const DEBUG_COMMANDS: &[&str] = &[
    "backtrace",
    "bindings",
    "break",
    "breakpoints",
    "checkpoint",
    "checkpoints",
    "continue",
    "delete",
    "delete-checkpoint",
    "expression-step",
    "finish",
    "help",
    "history",
    "next",
    "print",
    "quit",
    "run",
    "restore",
    "reverse-continue",
    "reverse-finish",
    "reverse-next",
    "reverse-source-step",
    "reverse-step",
    "source-step",
    "step",
    "unwatch",
    "until",
    "watch",
    "watchpoints",
    "where",
    "why",
];

const PROMPT_ALIASES: &[(&str, &str)] = &[
    ("bt", "backtrace"),
    ("c", "continue"),
    ("es", "expression-step"),
    ("h", "help"),
    ("n", "next"),
    ("p", "print"),
    ("q", "quit"),
    ("rc", "reverse-continue"),
    ("rf", "reverse-finish"),
    ("rn", "reverse-next"),
    ("rs", "reverse-step"),
    ("rss", "reverse-source-step"),
    ("s", "step"),
    ("ss", "source-step"),
    ("w", "where"),
];

fn resolve_prompt_command(input: &str) -> Result<String, String> {
    let (head, tail) = input.split_once(' ').unwrap_or((input, ""));
    if head.is_empty() || DEBUG_COMMANDS.contains(&head) {
        return Ok(input.to_owned());
    }
    if let Some((_, command)) = PROMPT_ALIASES.iter().find(|(alias, _)| *alias == head) {
        return Ok(join_command(command, tail));
    }
    let matches = DEBUG_COMMANDS
        .iter()
        .copied()
        .filter(|command| command.starts_with(head))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [command] => Ok(join_command(command, tail)),
        [] => Ok(input.to_owned()),
        _ => Err(format!(
            "ambiguous debugger command `{head}`: {}",
            matches.join(", ")
        )),
    }
}

fn join_command(command: &str, arguments: &str) -> String {
    if arguments.is_empty() {
        command.to_owned()
    } else {
        format!("{command} {arguments}")
    }
}

fn resolve_script_command(input: &str) -> String {
    input
        .strip_prefix("lang debug ")
        .unwrap_or(input)
        .to_owned()
}

fn reject_script_shortcut(
    input: &str,
    script_name: Option<&str>,
    line: usize,
) -> Result<(), String> {
    let head = input.split_once(' ').map_or(input, |(head, _)| head);
    let Some((_, command)) = PROMPT_ALIASES.iter().find(|(alias, _)| *alias == head) else {
        return Ok(());
    };
    Err(format!(
        "{}:{line}: error[D-SCRIPT-SHORTCUT]: debugger scripts use the complete function name `{command}`",
        script_name.unwrap_or("<script>")
    ))
}

fn handle_source_command(
    command: &str,
    history: &mut ExecutionHistory,
    source: &str,
    source_name: &str,
    session: &mut Session,
    execution: &mut Execution,
    complete: &mut bool,
) -> bool {
    match command {
        "source-step" | "ss" | "expression-step" | "es" => {
            match live_source_step(history, session, execution, complete) {
                Ok(true) => print_source_location(history, source, source_name),
                Ok(false) => println!("end of source execution"),
                Err(error) => println!("{}", error.render(source_name)),
            }
        }
        "reverse-source-step" | "rss" => match history.step_source_backward() {
            Some(_) => print_source_location(history, source, source_name),
            None => println!("start of source execution"),
        },
        "where" | "w" => print_source_location(history, source, source_name),
        _ => return false,
    }
    true
}

fn print_reason(history: &ExecutionHistory) {
    if let Some(transition) = history.current() {
        if HUMAN_OUTPUT.load(Ordering::Relaxed) {
            println!(
                "decision #{}: {} because {} ({})",
                transition.sequence,
                human_event(transition.event),
                human_rule(transition.rule),
                transition.detail
            );
        } else {
            println!(
                "decision #{}: {} because {} ({})",
                transition.sequence, transition.event, transition.rule, transition.detail
            );
        }
    } else {
        println!("before the first semantic decision");
    }
}

#[allow(clippy::too_many_arguments)] // Live control shares all mutable debuggee components.
fn handle_continue_command(
    command: &str,
    history: &mut ExecutionHistory,
    source: &str,
    source_name: &str,
    session: &mut Session,
    execution: &mut Execution,
    complete: &mut bool,
    breakpoints: &BTreeSet<Breakpoint>,
    watchpoints: &BTreeSet<String>,
    interrupted: &AtomicBool,
) -> bool {
    let reverse = match command {
        "continue" | "c" => false,
        "reverse-continue" | "rc" => true,
        _ => return false,
    };
    if reverse && continue_to_stop(history, source, breakpoints, watchpoints, true) {
        print_source_location(history, source, source_name);
    } else if reverse {
        println!("no earlier breakpoint");
    } else {
        interrupted.store(false, Ordering::SeqCst);
        loop {
            if continue_to_stop(history, source, breakpoints, watchpoints, false) {
                print_source_location(history, source, source_name);
                break;
            }
            if interrupted.swap(false, Ordering::SeqCst) {
                println!("execution interrupted");
                print_source_location(history, source, source_name);
                break;
            }
            match live_source_step(history, session, execution, complete) {
                Ok(true) => {}
                Ok(false) => {
                    println!("application finished");
                    print_source_location(history, source, source_name);
                    break;
                }
                Err(error) => {
                    println!("{}", error.render(source_name));
                    break;
                }
            }
        }
    }
    true
}

#[allow(clippy::too_many_arguments)]
fn handle_until_command(
    command: &str,
    history: &mut ExecutionHistory,
    source: &str,
    source_name: &str,
    session: &mut Session,
    execution: &mut Execution,
    complete: &mut bool,
    dependency_frame: &mut Option<SourceFrame>,
    interrupted: &AtomicBool,
) {
    let target = command.strip_prefix("until").unwrap().trim();
    if target.is_empty()
        && let Some(frame) = dependency_frame.take()
    {
        history.seek(frame.return_cursor);
        print_source_location(history, source, source_name);
        return;
    }
    interrupted.store(false, Ordering::SeqCst);
    loop {
        if !target.is_empty() && until_target_matches(history, target) {
            print_source_location(history, source, source_name);
            return;
        }
        if interrupted.swap(false, Ordering::SeqCst) {
            println!("execution interrupted");
            print_source_location(history, source, source_name);
            return;
        }
        match live_source_step(history, session, execution, complete) {
            Ok(true) => {}
            Ok(false) => {
                println!("application finished");
                print_source_location(history, source, source_name);
                return;
            }
            Err(error) => {
                println!("{}", error.render(source_name));
                return;
            }
        }
    }
}

fn until_target_matches(history: &ExecutionHistory, target: &str) -> bool {
    if let Ok(line) = target.parse::<usize>() {
        return current_line(history).is_some_and(|(_, current)| current == line);
    }
    if let Some((file, line)) = target.rsplit_once(':')
        && let Ok(line) = line.parse::<usize>()
    {
        return current_line(history).is_some_and(|(name, current)| {
            current == line && (name == file || name.ends_with(file))
        });
    }
    let Some(state) = history.state() else {
        return false;
    };
    Session::inspect(&state.bindings, target, &mut io::sink())
        .is_ok_and(|value| value.to_string() == "true")
}

fn current_line(history: &ExecutionHistory) -> Option<(String, usize)> {
    let state = history.state()?;
    let source = state.source.as_deref()?;
    let name = state.source_name.as_deref()?;
    let range = state.source_range?;
    Some((name.to_owned(), line_column(source, range.start).0))
}

fn read_command(input: &mut impl BufRead, prompt: bool) -> Result<Option<String>, String> {
    if prompt {
        print!("(topal-debug) ");
        io::stdout().flush().map_err(|error| error.to_string())?;
    }
    let mut command = String::new();
    let count = input
        .read_line(&mut command)
        .map_err(|error| error.to_string())?;
    Ok((count != 0).then_some(command))
}

fn print_bindings(history: &ExecutionHistory) {
    let Some(state) = history.state() else {
        println!("no execution state selected");
        return;
    };
    if state.bindings.is_empty() {
        println!("no visible bindings");
    } else {
        for (name, value) in &state.bindings {
            println!("{name} = {value}");
        }
    }
}

fn print_expression(history: &ExecutionHistory, expression: &str) {
    let Some(state) = history.state() else {
        println!("no execution state selected");
        return;
    };
    match Session::inspect(&state.bindings, expression, &mut io::sink()) {
        Ok(value) => println!("{value}"),
        Err(error) => println!("{}", error.render("<debugger-expression>")),
    }
}

#[allow(clippy::too_many_arguments)] // Frame navigation needs the live execution and source boundary.
fn handle_frame_command(
    command: &str,
    history: &mut ExecutionHistory,
    source: &str,
    source_name: &str,
    session: &mut Session,
    execution: &mut Execution,
    current_source_start: usize,
    dependency_frame: &mut Option<SourceFrame>,
    complete: &mut bool,
) -> bool {
    match command {
        "next" | "n" if dependency_frame.is_some() => {
            let return_cursor = dependency_frame.as_ref().unwrap().return_cursor;
            let current_name = current_source_name(history, source_name);
            if step_to_source_in_file(history, &current_name, return_cursor, false) {
                print_source_location(history, source, source_name);
            } else {
                history.seek(return_cursor);
                *dependency_frame = None;
                print_source_location(history, source, source_name);
            }
        }
        "next" | "n" => {
            match live_next(history, session, execution, current_source_start, complete) {
                Ok(true) => print_source_location(history, source, source_name),
                Ok(false) => println!("end of current frame"),
                Err(error) => println!("{}", error.render(source_name)),
            }
        }
        "reverse-next" | "rn" if dependency_frame.is_some() => {
            let current_name = current_source_name(history, source_name);
            if step_to_source_in_file(history, &current_name, 0, true) {
                print_source_location(history, source, source_name);
            } else {
                let return_cursor = dependency_frame.as_ref().unwrap().return_cursor;
                history.seek(return_cursor);
                *dependency_frame = None;
                print_source_location(history, source, source_name);
            }
        }
        "reverse-next" | "rn" => match reverse_next(history, current_source_start) {
            Some(_) => print_source_location(history, source, source_name),
            None => println!("start of current frame"),
        },
        "finish" if dependency_frame.is_some() => {
            let return_cursor = dependency_frame.take().unwrap().return_cursor;
            history.seek(return_cursor);
            print_source_location(history, source, source_name);
        }
        "finish" => {
            while !*complete {
                match execution.step(session, history) {
                    Ok(ExecutionStep::Complete(_) | ExecutionStep::Returned { .. }) => {
                        *complete = true;
                    }
                    Ok(ExecutionStep::Advanced { .. }) => {}
                    Err(error) => {
                        println!("{}", error.render(source_name));
                        break;
                    }
                }
            }
            history.finish();
            print_source_location(history, source, source_name);
        }
        "reverse-finish" => {
            history.reverse_finish();
            print_source_location(history, source, source_name);
        }
        "backtrace" | "bt" => {
            print_backtrace(history, source, source_name, dependency_frame.as_ref());
        }
        _ => return false,
    }
    true
}

fn step_to_source_in_file(
    history: &mut ExecutionHistory,
    source_name: &str,
    boundary: usize,
    reverse: bool,
) -> bool {
    loop {
        let advanced = if reverse {
            history.step_source_backward().is_some()
        } else {
            history.cursor() < boundary && history.step_source_forward().is_some()
        };
        if !advanced {
            return false;
        }
        if current_source_name(history, "") == source_name {
            return true;
        }
    }
}

fn live_next(
    history: &mut ExecutionHistory,
    session: &mut Session,
    execution: &mut Execution,
    current_source_start: usize,
    complete: &mut bool,
) -> Result<bool, topal_language::Diagnostic> {
    if history.cursor() < current_source_start {
        history.seek(current_source_start);
    }
    live_source_step(history, session, execution, complete)
}

fn print_initial_position(source: &str, source_name: &str) {
    let start = source
        .char_indices()
        .find(|(_, character)| !character.is_whitespace())
        .map_or(0, |(offset, _)| offset);
    let range = topal_language::SourceRange { start, end: start };
    print!(
        "{}",
        render_source_position(source, source_name, range, None)
    );
}

fn reverse_next(
    history: &mut ExecutionHistory,
    current_source_start: usize,
) -> Option<topal_language::ExecutionState> {
    let state = history.step_source_backward()?;
    if history.cursor() < current_source_start {
        history.seek(current_source_start);
        None
    } else {
        Some(state)
    }
}

fn live_source_step(
    history: &mut ExecutionHistory,
    session: &mut Session,
    execution: &mut Execution,
    complete: &mut bool,
) -> Result<bool, topal_language::Diagnostic> {
    if history.step_source_forward().is_some() {
        return Ok(true);
    }
    history.seek(history.transitions().len());
    if *complete {
        return Ok(false);
    }
    let frontier = history.transitions().len();
    let step = execution.step(session, history)?;
    *complete = matches!(
        step,
        ExecutionStep::Complete(_) | ExecutionStep::Returned { .. }
    );
    history.seek(frontier);
    if history.step_source_forward().is_none() {
        history.seek(history.transitions().len());
    }
    Ok(true)
}

fn print_backtrace(
    history: &ExecutionHistory,
    source: &str,
    source_name: &str,
    dependency_frame: Option<&SourceFrame>,
) {
    if let Some(state) = history.state()
        && let Some(range) = state.source_range
    {
        let shown_source = state.source.as_deref().unwrap_or(source);
        let shown_name = state.source_name.as_deref().unwrap_or(source_name);
        let (line, column) = line_column(shown_source, range.start);
        let frame_kind = if dependency_frame.is_some() {
            "<dependency>"
        } else {
            "<script>"
        };
        println!("#0 {frame_kind} at {shown_name}:{line}:{column}");
        if let Some(frame) = dependency_frame {
            let (line, column) = line_column(&frame.caller_source, frame.caller_range.start);
            println!("#1 <script> at {}:{line}:{column}", frame.caller_name);
        }
    } else {
        println!("#0 <script> before first statement in {source_name}");
    }
}

fn source_frame(
    history: &ExecutionHistory,
    source: &str,
    source_name: &str,
) -> Option<SourceFrame> {
    let state = history.state()?;
    Some(SourceFrame {
        return_cursor: history.cursor(),
        caller_name: state
            .source_name
            .as_deref()
            .unwrap_or(source_name)
            .to_owned(),
        caller_source: state.source.as_deref().unwrap_or(source).to_owned(),
        caller_range: state.source_range?,
    })
}

fn current_source_name(history: &ExecutionHistory, fallback: &str) -> String {
    history
        .state()
        .and_then(|state| state.source_name)
        .as_deref()
        .unwrap_or(fallback)
        .to_owned()
}

fn valid_checkpoint_name(name: &str) -> bool {
    !name.is_empty() && !name.chars().any(char::is_whitespace)
}

fn print_breakpoints(breakpoints: &BTreeSet<Breakpoint>) {
    for (source_name, line) in breakpoints {
        println!("{source_name}:{line}");
    }
}

fn print_watchpoints(watchpoints: &BTreeSet<String>) {
    for name in watchpoints {
        println!("{name}");
    }
}

fn print_checkpoints(checkpoints: &BTreeMap<String, usize>) {
    for (name, cursor) in checkpoints {
        println!("{name} = #{cursor}");
    }
}

fn save_checkpoint(command: &str, checkpoints: &mut BTreeMap<String, usize>, cursor: usize) {
    let name = command["checkpoint ".len()..].trim();
    if valid_checkpoint_name(name) {
        checkpoints.insert(name.to_owned(), cursor);
        println!("checkpoint {name} saved at #{cursor}");
    } else {
        println!("checkpoint name must be one nonempty word");
    }
}

fn restore_checkpoint(
    command: &str,
    checkpoints: &BTreeMap<String, usize>,
    history: &mut ExecutionHistory,
    source: &str,
    source_name: &str,
) {
    let name = command["restore ".len()..].trim();
    if let Some(cursor) = checkpoints.get(name) {
        history.seek(*cursor);
        println!("checkpoint {name} restored at #{cursor}");
        print_source_location(history, source, source_name);
    } else {
        println!("unknown checkpoint: {name}");
    }
}

fn delete_checkpoint(command: &str, checkpoints: &mut BTreeMap<String, usize>) {
    let name = command["delete-checkpoint ".len()..].trim();
    if checkpoints.remove(name).is_some() {
        println!("checkpoint {name} deleted");
    } else {
        println!("unknown checkpoint: {name}");
    }
}

fn update_watchpoint(
    command: &str,
    prefix: &str,
    watchpoints: &mut BTreeSet<String>,
    insert: bool,
) {
    let name = command[prefix.len()..].trim();
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        println!("watchpoint name must be one identifier");
    } else if insert {
        watchpoints.insert(name.to_owned());
        println!("watchpoint set for {name}");
    } else if watchpoints.remove(name) {
        println!("watchpoint removed for {name}");
    } else {
        println!("no watchpoint for {name}");
    }
}

fn update_breakpoint(
    command: &str,
    prefix: &str,
    source_name: &str,
    breakpoints: &mut BTreeSet<Breakpoint>,
    insert: bool,
) {
    match command[prefix.len()..].trim().parse::<usize>() {
        Ok(0) | Err(_) => println!("breakpoint line must be a positive integer"),
        Ok(line) if insert => {
            breakpoints.insert((source_name.to_owned(), line));
            println!("breakpoint set at line {line} in {source_name}");
        }
        Ok(line) => {
            if breakpoints.remove(&(source_name.to_owned(), line)) {
                println!("breakpoint removed from line {line} in {source_name}");
            } else {
                println!("no breakpoint at {source_name}:{line}");
            }
        }
    }
}

fn continue_to_stop(
    history: &mut ExecutionHistory,
    _source: &str,
    breakpoints: &BTreeSet<Breakpoint>,
    watchpoints: &BTreeSet<String>,
    reverse: bool,
) -> bool {
    let current_bindings = history
        .state()
        .map(|state| state.bindings.clone())
        .unwrap_or_default();
    let matches = |state: &topal_language::ExecutionState| {
        let breakpoint = state.source_range.is_some_and(|range| {
            let Some(source) = state.source.as_deref() else {
                return false;
            };
            let Some(source_name) = state.source_name.as_deref() else {
                return false;
            };
            let (line, _) = line_column(source, range.start);
            breakpoints.contains(&(source_name.to_owned(), line))
        });
        let watched_change = watchpoints
            .iter()
            .any(|name| state.bindings.get(name) != current_bindings.get(name));
        breakpoint || watched_change
    };
    if reverse {
        history.continue_source_backward(matches).is_some()
    } else {
        history.continue_source_forward(matches).is_some()
    }
}

fn print_source_location(history: &ExecutionHistory, source: &str, source_name: &str) {
    let Some(state) = history.state() else {
        println!("before first source statement");
        return;
    };
    let Some(range) = state.source_range else {
        println!("before first source statement");
        return;
    };
    let shown_source = state.source.as_deref().unwrap_or(source);
    let shown_name = state.source_name.as_deref().unwrap_or(source_name);
    print!(
        "{}",
        render_source_position(shown_source, shown_name, range, None)
    );
}

fn at_library_selection(history: &ExecutionHistory, source: &str, source_name: &str) -> bool {
    let Some(state) = history.state() else {
        return false;
    };
    if state
        .source_name
        .as_deref()
        .is_some_and(|name| name != source_name)
    {
        return false;
    }
    let Some(range) = state.source_range else {
        return false;
    };
    let Ok(source_text) = SourceText::new(source) else {
        return false;
    };
    parse(&source_text, &lex(&source_text))
        .statements
        .iter()
        .any(|statement| matches!(statement, Statement::LibrarySelection { span, .. } if span.start == range.start))
}

fn render_source_position(
    source: &str,
    source_name: &str,
    position: topal_language::SourceRange,
    emphasis: Option<topal_language::SourceRange>,
) -> String {
    let (line, column) = line_column(source, position.start);
    let source_line = source.lines().nth(line.saturating_sub(1)).unwrap_or("");
    let mut rendered = format!("{source_name}:{line}:{column}\n{source_line}\n");
    if let Some(emphasis) = emphasis {
        let (_, emphasis_column) = line_column(source, emphasis.start);
        rendered.push_str(&" ".repeat(emphasis_column.saturating_sub(1)));
        rendered.push_str(&"^".repeat(emphasis.end.saturating_sub(emphasis.start).max(1)));
        rendered.push('\n');
    }
    rendered
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let prefix = &source[..offset.min(source.len())];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
    (line, column)
}

fn print_transition(transition: &ExecutionTransition) {
    let transaction = transition.transaction.map_or_else(String::new, |identity| {
        format!(
            " transaction={identity} sender={} receiver={}",
            transition.sender.unwrap_or_default(),
            transition.receiver.unwrap_or_default()
        )
    });
    if HUMAN_OUTPUT.load(Ordering::Relaxed) {
        println!(
            "#{} {} — {}: {}{}",
            transition.sequence,
            human_event(transition.event),
            human_rule(transition.rule),
            transition.detail,
            transaction
        );
    } else {
        println!(
            "#{} {} [{}] {}{}",
            transition.sequence, transition.event, transition.rule, transition.detail, transaction
        );
    }
}

fn print_history(history: &ExecutionHistory) {
    for transition in history.transitions() {
        let marker = if transition.sequence + 1 == history.cursor() {
            ">"
        } else {
            " "
        };
        let transaction = transition.transaction.map_or_else(String::new, |identity| {
            format!(
                " transaction={identity} sender={} receiver={}",
                transition.sender.unwrap_or_default(),
                transition.receiver.unwrap_or_default()
            )
        });
        if HUMAN_OUTPUT.load(Ordering::Relaxed) {
            println!(
                "{marker} #{} {} — {}: {}{}",
                transition.sequence,
                human_event(transition.event),
                human_rule(transition.rule),
                transition.detail,
                transaction
            );
        } else {
            println!(
                "{marker} #{} {} [{}] {}{}",
                transition.sequence,
                transition.event,
                transition.rule,
                transition.detail,
                transaction
            );
        }
    }
}

fn human_event(event: &str) -> String {
    event.replace(['.', '_'], " ")
}

fn human_rule(rule: &str) -> String {
    let words = rule
        .strip_prefix("TOPAL-")
        .unwrap_or(rule)
        .split('-')
        .filter(|part| !part.chars().all(|character| character.is_ascii_digit()))
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    if words.is_empty() {
        "language semantics".to_owned()
    } else {
        words.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_expands_unique_commands_and_aliases() {
        assert_eq!(
            resolve_prompt_command("reverse-f").unwrap(),
            "reverse-finish"
        );
        assert_eq!(resolve_prompt_command("p answer").unwrap(), "print answer");
    }

    #[test]
    fn prompt_reports_ambiguous_recursive_lookup() {
        let error = resolve_prompt_command("bre").unwrap_err();
        assert!(error.contains("break"));
        assert!(error.contains("breakpoints"));
    }

    #[test]
    fn scripts_resolve_complete_names_from_lang_debug() {
        assert_eq!(resolve_script_command("step"), "step");
        assert_eq!(resolve_script_command("lang debug break 42"), "break 42");
        assert!(
            DEBUG_COMMANDS
                .iter()
                .all(|command| debugger_command_documentation(command).is_some())
        );
    }

    #[test]
    fn completion_describes_commands_and_arguments() {
        let helper = DebugHelper;
        let history = DefaultHistory::new();
        let context = Context::new(&history);
        let (_, commands) = helper.complete("reverse-f", 9, &context).unwrap();
        assert_eq!(commands[0].replacement, "reverse-finish");
        assert_eq!(
            command_argument_hint("until"),
            Some("[LINE | FILE:LINE | CONDITION]")
        );
        assert!(is_progressing_command("continue"));
        assert!(!is_progressing_command("help"));
    }

    #[test]
    fn requirement_identifiers_are_rendered_as_words() {
        assert_eq!(human_rule("TOPAL-DEBUG-CONTROL-001"), "debug control");
        assert_eq!(human_event("binding.bind"), "binding bind");
    }
}
