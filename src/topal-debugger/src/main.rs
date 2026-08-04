use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::ExitCode;

use topal_language::{ExecutionHistory, ExecutionTransition, Session};

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
    let mut history = ExecutionHistory::new();
    Session::new()
        .evaluate(&source, &mut history)
        .map_err(|error| error.render(&arguments.source))?;
    history.rewind();

    println!("loaded {} transitions", history.transitions().len());
    if let Some(commands) = arguments.commands {
        if commands == "-" {
            command_loop(io::stdin().lock(), &mut history, &source, &arguments.source)
        } else {
            let file = fs::File::open(&commands)
                .map_err(|error| format!("cannot read command script {commands}: {error}"))?;
            command_loop(
                BufReader::new(file),
                &mut history,
                &source,
                &arguments.source,
            )
        }
    } else {
        command_loop(io::stdin().lock(), &mut history, &source, &arguments.source)
    }
}

struct Arguments {
    source: String,
    commands: Option<String>,
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

fn command_loop(
    input: impl Read,
    history: &mut ExecutionHistory,
    source: &str,
    source_name: &str,
) -> Result<(), String> {
    let lines = BufReader::new(input).lines();
    for line in lines {
        let command = line.map_err(|error| error.to_string())?;
        match command.trim() {
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
            "source-step" | "ss" => match history.step_source_forward() {
                Some(_) => print_source_location(history, source, source_name),
                None => println!("end of source execution"),
            },
            "reverse-source-step" | "rss" => match history.step_source_backward() {
                Some(_) => print_source_location(history, source, source_name),
                None => println!("start of source execution"),
            },
            "where" | "w" => print_source_location(history, source, source_name),
            "print" | "p" => match history.state().and_then(|state| state.value.as_ref()) {
                Some(value) => println!("{value}"),
                None => println!("no value at current execution state"),
            },
            "bindings" => {
                if let Some(state) = history.state() {
                    for (name, value) in &state.bindings {
                        println!("{name} = {value}");
                    }
                }
            }
            "help" | "h" => {
                println!(
                    "step | reverse-step | source-step | reverse-source-step | where | history | print | bindings | quit"
                );
            }
            "quit" | "q" => return Ok(()),
            "" => {}
            unknown => println!("unknown command: {unknown}; use help"),
        }
        io::stdout().flush().map_err(|error| error.to_string())?;
    }
    Ok(())
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
