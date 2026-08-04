use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
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
    let path = parse_path(env::args().skip(1))?;
    let source =
        fs::read_to_string(&path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let mut history = ExecutionHistory::new();
    let value = Session::new()
        .evaluate(&source, &mut history)
        .map_err(|error| error.render(&path))?;
    history.rewind();

    println!("loaded {} transitions", history.transitions().len());
    command_loop(&mut history, &value)
}

fn parse_path(mut arguments: impl Iterator<Item = String>) -> Result<String, String> {
    let path = arguments
        .next()
        .ok_or_else(|| "usage: topal-debug FILE".to_owned())?;
    if path.starts_with('-') {
        return Err(format!("unknown option: {path}"));
    }
    if let Some(extra) = arguments.next() {
        return Err(format!("unexpected second source file: {extra}"));
    }
    Ok(path)
}

fn command_loop(
    history: &mut ExecutionHistory,
    value: &impl std::fmt::Display,
) -> Result<(), String> {
    let stdin = io::stdin();
    let lines = stdin.lock().lines();
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
