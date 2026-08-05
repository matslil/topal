use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::process::ExitCode;

use topal_language::{Execution, ExecutionHistory, ExecutionStep, ExecutionTransition, Session};

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
    let source = fs::read_to_string(&arguments.source)
        .map_err(|error| format!("cannot read {}: {error}", arguments.source))?;
    let session = Session::new();
    let mut history = ExecutionHistory::new();
    let execution = session
        .prepare(&source, &mut history)
        .map_err(|error| error.render(&arguments.source))?;
    history.rewind();
    let mut debuggee = Debuggee {
        session,
        execution,
        history,
        complete: false,
    };

    println!(
        "loaded {} transitions",
        debuggee.history.transitions().len()
    );
    if let Some(commands) = arguments.commands {
        if commands == "-" {
            command_loop(
                io::stdin().lock(),
                &mut debuggee,
                &source,
                &arguments.source,
                Some("<stdin>"),
                false,
            )
        } else {
            let file = fs::File::open(&commands)
                .map_err(|error| format!("cannot read command script {commands}: {error}"))?;
            command_loop(
                BufReader::new(file),
                &mut debuggee,
                &source,
                &arguments.source,
                Some(&commands),
                false,
            )
        }
    } else {
        let prompt = io::stdin().is_terminal();
        command_loop(
            io::stdin().lock(),
            &mut debuggee,
            &source,
            &arguments.source,
            None,
            prompt,
        )
    }
}

struct Arguments {
    source: String,
    commands: Option<String>,
}

struct Debuggee {
    session: Session,
    execution: Execution,
    history: ExecutionHistory,
    complete: bool,
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Arguments, String> {
    let first = arguments
        .next()
        .ok_or_else(|| "usage: topal-debug [--script COMMANDS] FILE".to_owned())?;
    let (commands, source) = if first == "--script" {
        let commands = arguments
            .next()
            .ok_or_else(|| "--script requires a command file or -".to_owned())?;
        let source = arguments
            .next()
            .ok_or_else(|| "--script requires a Topal source file".to_owned())?;
        (Some(commands), source)
    } else if first.starts_with('-') {
        return Err(format!("unknown option: {first}"));
    } else {
        (None, first)
    };
    if let Some(extra) = arguments.next() {
        return Err(format!("unexpected second source file: {extra}"));
    }
    Ok(Arguments { source, commands })
}

#[allow(clippy::too_many_lines)] // Command aliases remain visible in one deterministic dispatcher.
fn command_loop(
    mut input: impl BufRead,
    debuggee: &mut Debuggee,
    source: &str,
    source_name: &str,
    script_name: Option<&str>,
    prompt: bool,
) -> Result<(), String> {
    let Debuggee {
        session,
        execution,
        history,
        complete,
    } = debuggee;
    let mut breakpoints = BTreeSet::new();
    let mut watchpoints = BTreeSet::new();
    let mut checkpoints = BTreeMap::new();
    let mut line_number = 0;
    loop {
        let Some(command) = read_command(&mut input, prompt)? else {
            return Ok(());
        };
        line_number += 1;
        match command.trim() {
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
            "print" | "p" => match history.state().and_then(|state| state.value.as_ref()) {
                Some(value) => println!("{value}"),
                None => println!("no value at current execution state"),
            },
            command if command.starts_with("print ") || command.starts_with("p ") => {
                let expression = command.split_once(' ').unwrap().1.trim();
                print_expression(history, expression);
            }
            "bindings" => print_bindings(history),
            "help" | "h" => {
                println!(
                    "step | reverse-step | source-step | reverse-source-step | next | reverse-next | finish | reverse-finish | backtrace | break LINE | delete LINE | breakpoints | watch NAME | unwatch NAME | watchpoints | continue | reverse-continue | checkpoint NAME | restore NAME | checkpoints | delete-checkpoint NAME | where | why | history | print | bindings | quit"
                );
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

fn handle_frame_command(
    command: &str,
    history: &mut ExecutionHistory,
    source: &str,
    source_name: &str,
    session: &mut Session,
    execution: &mut Execution,
    complete: &mut bool,
) -> bool {
    match command {
        "next" | "n" => match live_source_step(history, session, execution, complete) {
            Ok(true) => print_source_location(history, source, source_name),
            Ok(false) => println!("end of current frame"),
            Err(error) => println!("{}", error.render(source_name)),
        },
        "reverse-next" | "rn" => match history.step_source_backward() {
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
    println!(
        "#{} {} [{}] {}",
        transition.sequence, transition.event, transition.rule, transition.detail
    );
}

fn print_history(history: &ExecutionHistory) {
    for transition in history.transitions() {
        let marker = if transition.sequence + 1 == history.cursor() {
            ">"
        } else {
            " "
        };
        println!(
            "{marker} #{} {} [{}] {}",
            transition.sequence, transition.event, transition.rule, transition.detail
        );
    }
}
