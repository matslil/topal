use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, BufRead, Cursor, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use topal_language::{
    Execution, ExecutionHistory, ExecutionStep, ExecutionTransition, Session, declares_library,
    lang_documentation, load_module_tree,
};
use topal_source::SourceText;
use topal_syntax::{DocumentedDeclaration, Statement, extract_documentation, lex, parse};

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
    let current_source_start = history.transitions().len();
    let execution = session
        .prepare_source_file(&source, &mut history)
        .map_err(|error| error.render(&source_name))?;
    history.rewind();
    let mut debuggee = Debuggee {
        session,
        execution,
        history,
        current_source_start,
        complete: false,
    };

    println!(
        "loaded {} transitions",
        debuggee.history.transitions().len()
    );
    if let Some(commands) = arguments.commands {
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
                Some("<stdin>"),
                false,
                header_lines,
            )
        } else {
            let script = fs::read_to_string(&commands)
                .map_err(|error| format!("cannot read command script {commands}: {error}"))?;
            let (body, header_lines) = debug_script_body(&script, &commands)?;
            command_loop(
                Cursor::new(body),
                &mut debuggee,
                &source,
                &source_name,
                Some(&commands),
                false,
                header_lines,
            )
        }
    } else {
        let prompt = io::stdin().is_terminal();
        command_loop(
            io::stdin().lock(),
            &mut debuggee,
            &source,
            &source_name,
            None,
            prompt,
            0,
        )
    }
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
    complete: bool,
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

#[allow(clippy::too_many_lines)] // Command aliases remain visible in one deterministic dispatcher.
fn command_loop(
    mut input: impl BufRead,
    debuggee: &mut Debuggee,
    source: &str,
    source_name: &str,
    script_name: Option<&str>,
    prompt: bool,
    initial_line_number: usize,
) -> Result<(), String> {
    let Debuggee {
        session,
        execution,
        history,
        current_source_start,
        complete,
    } = debuggee;
    let mut breakpoints = BTreeSet::new();
    let mut watchpoints = BTreeSet::new();
    let mut checkpoints = BTreeMap::new();
    let mut line_number = initial_line_number;
    loop {
        let Some(command) = read_command(&mut input, prompt)? else {
            return Ok(());
        };
        line_number += 1;
        let command = if prompt {
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
                )? => {}
            command
                if handle_frame_command(
                    command,
                    history,
                    source,
                    source_name,
                    session,
                    execution,
                    *current_source_start,
                    complete,
                ) => {}
            command
                if handle_continue_command(
                    command,
                    history,
                    source,
                    source_name,
                    &breakpoints,
                    &watchpoints,
                ) => {}
            "step" | "s" => match history.step_forward() {
                Some(transition) => print_transition(transition),
                None => println!("end of execution"),
            },
            "reverse-step" | "rs" => {
                if history.cursor() == 0 {
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
                print_breakpoints(&breakpoints, source_name);
            }
            "watchpoints" => {
                print_watchpoints(&watchpoints);
            }
            "checkpoints" => {
                print_checkpoints(&checkpoints);
            }
            command if command.starts_with("break ") => {
                update_breakpoint(command, "break ", &mut breakpoints, true);
            }
            command if command.starts_with("delete ") => {
                update_breakpoint(command, "delete ", &mut breakpoints, false);
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
                print_identifier_help(command[5..].trim(), source);
            }
            "help" | "h" => {
                println!(
                    "step | reverse-step | source-step | reverse-source-step | next | reverse-next | finish | reverse-finish | backtrace | break LINE | delete LINE | breakpoints | watch NAME | unwatch NAME | watchpoints | continue | reverse-continue | checkpoint NAME | restore NAME | checkpoints | delete-checkpoint NAME | where | why | history | print | bindings | quit"
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

fn print_identifier_help(identifier: &str, source: &str) {
    let mut declarations = documented_source(source);
    declarations.extend(documented_source(include_str!(
        "../../../library/std/module.t"
    )));
    declarations.extend(lang_documentation());
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
    "restore",
    "reverse-continue",
    "reverse-finish",
    "reverse-next",
    "reverse-source-step",
    "reverse-step",
    "source-step",
    "step",
    "unwatch",
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
) -> Result<bool, String> {
    match command {
        "source-step" | "ss" | "expression-step" | "es" => {
            if live_source_step(history, session, execution, complete)
                .map_err(|error| error.render(source_name))?
            {
                print_source_location(history, source, source_name);
            } else {
                println!("end of source execution");
            }
        }
        "reverse-source-step" | "rss" => match history.step_source_backward() {
            Some(_) => print_source_location(history, source, source_name),
            None => println!("start of source execution"),
        },
        "where" | "w" => print_source_location(history, source, source_name),
        _ => return Ok(false),
    }
    Ok(true)
}

fn print_reason(history: &ExecutionHistory) {
    if let Some(transition) = history.current() {
        println!(
            "decision #{}: {} because {} ({})",
            transition.sequence, transition.event, transition.rule, transition.detail
        );
    } else {
        println!("before the first semantic decision");
    }
}

fn handle_continue_command(
    command: &str,
    history: &mut ExecutionHistory,
    source: &str,
    source_name: &str,
    breakpoints: &BTreeSet<usize>,
    watchpoints: &BTreeSet<String>,
) -> bool {
    let reverse = match command {
        "continue" | "c" => false,
        "reverse-continue" | "rc" => true,
        _ => return false,
    };
    if continue_to_stop(history, source, breakpoints, watchpoints, reverse) {
        print_source_location(history, source, source_name);
    } else if reverse {
        println!("no earlier breakpoint");
    } else {
        println!("no later breakpoint");
    }
    true
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
    if let Some(state) = history.state() {
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
    complete: &mut bool,
) -> bool {
    match command {
        "next" | "n" => {
            match live_next(history, session, execution, current_source_start, complete) {
                Ok(true) => print_source_location(history, source, source_name),
                Ok(false) => println!("end of current frame"),
                Err(error) => println!("{}", error.render(source_name)),
            }
        }
        "reverse-next" | "rn" => match reverse_next(history, current_source_start) {
            Some(_) => print_source_location(history, source, source_name),
            None => println!("start of current frame"),
        },
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
        "backtrace" | "bt" => print_backtrace(history, source, source_name),
        _ => return false,
    }
    true
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

fn print_backtrace(history: &ExecutionHistory, source: &str, source_name: &str) {
    if let Some(range) = history.state().and_then(|state| state.source_range) {
        let (line, column) = line_column(source, range.start);
        println!("#0 <script> at {source_name}:{line}:{column}");
    } else {
        println!("#0 <script> before first statement in {source_name}");
    }
}

fn valid_checkpoint_name(name: &str) -> bool {
    !name.is_empty() && !name.chars().any(char::is_whitespace)
}

fn print_breakpoints(breakpoints: &BTreeSet<usize>, source_name: &str) {
    for line in breakpoints {
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

fn update_breakpoint(command: &str, prefix: &str, breakpoints: &mut BTreeSet<usize>, insert: bool) {
    match command[prefix.len()..].trim().parse::<usize>() {
        Ok(0) | Err(_) => println!("breakpoint line must be a positive integer"),
        Ok(line) if insert => {
            breakpoints.insert(line);
            println!("breakpoint set at line {line}");
        }
        Ok(line) => {
            if breakpoints.remove(&line) {
                println!("breakpoint removed from line {line}");
            } else {
                println!("no breakpoint at line {line}");
            }
        }
    }
}

fn continue_to_stop(
    history: &mut ExecutionHistory,
    source: &str,
    breakpoints: &BTreeSet<usize>,
    watchpoints: &BTreeSet<String>,
    reverse: bool,
) -> bool {
    let current_bindings = history
        .state()
        .map(|state| state.bindings.clone())
        .unwrap_or_default();
    let matches = |state: &topal_language::ExecutionState| {
        let breakpoint = state.source_range.is_some_and(|range| {
            let (line, _) = line_column(source, range.start);
            breakpoints.contains(&line)
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
    let Some(range) = history.state().and_then(|state| state.source_range) else {
        println!("before first source statement");
        return;
    };
    let (line, column) = line_column(source, range.start);
    let source_line = source.lines().nth(line.saturating_sub(1)).unwrap_or("");
    println!("{source_name}:{line}:{column}");
    println!("{source_line}");
    println!(
        "{}{}",
        " ".repeat(column.saturating_sub(1)),
        "^".repeat(range.end.saturating_sub(range.start).max(1))
    );
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
    println!(
        "#{} {} [{}] {}{}",
        transition.sequence, transition.event, transition.rule, transition.detail, transaction
    );
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
        println!(
            "{marker} #{} {} [{}] {}{}",
            transition.sequence, transition.event, transition.rule, transition.detail, transaction
        );
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
    }
}
