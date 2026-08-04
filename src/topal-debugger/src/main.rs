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
    let value = Session::new()
        .evaluate(&source, &mut history)
        .map_err(|error| error.render(&arguments.source))?;
    history.rewind();

    println!("loaded {} transitions", history.transitions().len());
    if let Some(commands) = arguments.commands {
        if commands == "-" {
            command_loop(io::stdin().lock(), &mut history, &value)
        } else {
            let file = fs::File::open(&commands)
                .map_err(|error| format!("cannot read command script {commands}: {error}"))?;
            command_loop(BufReader::new(file), &mut history, &value)
        }
    } else {
        command_loop(io::stdin().lock(), &mut history, &value)
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
    value: &impl std::fmt::Display,
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
            "print" | "p" if history.cursor() == history.transitions().len() => {
                println!("{value}");
            }
            "print" | "p" => println!("value is available at end of execution"),
            "help" | "h" => println!("step | reverse-step | history | print | quit"),
            "quit" | "q" => return Ok(()),
            "" => {}
            unknown => println!("unknown command: {unknown}; use help"),
        }
        io::stdout().flush().map_err(|error| error.to_string())?;
    }
    Ok(())
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
