use std::env;
use std::path::PathBuf;

use topal_build::{Options, run};

fn main() {
    if let Err(error) = execute() {
        eprintln!("topal-build: {error}");
        std::process::exit(1);
    }
}

fn execute() -> Result<(), String> {
    let mut source_root = env::current_dir().map_err(|error| error.to_string())?;
    let mut build_root = None;
    let mut library_root = None;
    let mut manifest = PathBuf::from("topal-build.json");
    let mut dry_run = false;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--source-root" => source_root = value(&mut arguments, &argument)?.into(),
            "--build-root" => build_root = Some(PathBuf::from(value(&mut arguments, &argument)?)),
            "--library-root" => {
                library_root = Some(PathBuf::from(value(&mut arguments, &argument)?));
            }
            "--manifest" => manifest = value(&mut arguments, &argument)?.into(),
            "--dry-run" => dry_run = true,
            "--help" => {
                println!(
                    "usage: topal-build [--source-root PATH] [--build-root PATH] \
                     [--library-root PATH] [--manifest PATH] [--dry-run]"
                );
                return Ok(());
            }
            _ => return Err(format!("unknown option `{argument}`")),
        }
    }
    let build_root = build_root.unwrap_or_else(|| source_root.join(".topal-build"));
    let library_root = library_root.unwrap_or_else(|| source_root.join("library"));
    let outcome = run(&Options {
        source_root,
        build_root,
        library_root,
        manifest,
        dry_run,
    })?;
    for identity in outcome.selected {
        println!("selected {identity}");
    }
    Ok(())
}

fn value(arguments: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("{option} requires a value"))
}
