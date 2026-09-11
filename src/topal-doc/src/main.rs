use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use topal_language::lang_documentation;
use topal_source::SourceText;
use topal_syntax::{DocumentedDeclaration, extract_documentation, lex, parse};

struct Options {
    output: PathBuf,
    recurse: bool,
    include_lang: bool,
    inputs: Vec<PathBuf>,
}

fn main() -> std::process::ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("topal-doc: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let options = options(arguments)?;
    let files = source_files(&options.inputs, options.recurse)?;
    fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let mut pages = Vec::new();
    let mut names = BTreeSet::new();
    for path in files {
        let page = page_name(&path, &options.inputs);
        if !names.insert(page.clone()) {
            return Err(format!("multiple inputs would produce `{page}.rst`"));
        }
        let text =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let source =
            SourceText::new(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        let lexed = lex(&source);
        let parsed = parse(&source, &lexed);
        if let Some(diagnostic) = parsed.diagnostics.first() {
            return Err(format!(
                "{}: {}: {}",
                path.display(),
                diagnostic.code,
                diagnostic.message
            ));
        }
        let declarations = extract_documentation(&source, &lexed, &parsed);
        write_page(
            &options.output.join(format!("{page}.rst")),
            &path.display().to_string(),
            &declarations,
        )?;
        pages.push(page);
    }
    if options.include_lang {
        write_page(
            &options.output.join("lang.rst"),
            "Built-in lang namespace",
            &lang_documentation(),
        )?;
        pages.push("lang".into());
    }
    write_index(&options.output.join("index.rst"), &pages)
}

fn options(mut arguments: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut output = None;
    let mut recurse = false;
    let mut include_lang = false;
    let mut inputs = Vec::new();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--output" => {
                output = Some(PathBuf::from(
                    arguments.next().ok_or("--output requires a directory")?,
                ));
            }
            "--recurse" => recurse = true,
            "--include-lang" => include_lang = true,
            "--help" | "-h" => {
                return Err(
                    "usage: topal-doc --output DIRECTORY [--recurse] [--include-lang] PATH..."
                        .into(),
                );
            }
            _ if argument.starts_with('-') => return Err(format!("unknown option `{argument}`")),
            _ => inputs.push(PathBuf::from(argument)),
        }
    }
    let output = output.ok_or("--output DIRECTORY is required")?;
    if inputs.is_empty() && !include_lang {
        return Err("provide at least one source path or --include-lang".into());
    }
    Ok(Options {
        output,
        recurse,
        include_lang,
        inputs,
    })
}

fn source_files(inputs: &[PathBuf], recurse: bool) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for input in inputs {
        if input.is_file() {
            files.push(input.clone());
        } else if input.is_dir() {
            visit_directory(input, recurse, &mut files)?;
        } else {
            return Err(format!("{} is not a file or directory", input.display()));
        }
    }
    files.retain(|path| path.extension().is_some_and(|extension| extension == "t"));
    files.sort();
    files.dedup();
    Ok(files)
}

fn visit_directory(
    directory: &Path,
    recurse: bool,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|extension| extension == "t") {
            files.push(path);
        } else if recurse && path.is_dir() {
            visit_directory(&path, true, files)?;
        }
    }
    Ok(())
}

fn sanitize_page_component(component: &str) -> String {
    component
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn page_name(path: &Path, inputs: &[PathBuf]) -> String {
    let directory_root = inputs
        .iter()
        .filter(|input| input.is_dir() && path.starts_with(input))
        .max_by_key(|input| input.components().count());
    if let Some(root) = directory_root {
        let relative = path.strip_prefix(root).expect("selected input is a prefix");
        let page = if path.file_name().is_some_and(|name| name == "module.t") {
            relative
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .to_path_buf()
        } else {
            relative.with_extension("")
        };
        let components = page
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .map(sanitize_page_component)
            .collect::<Vec<_>>();
        if !components.is_empty() {
            return components.join("/");
        }
        return sanitize_page_component(
            root.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("source"),
        );
    }

    let canonical = if path.file_name().is_some_and(|name| name == "module.t") {
        path.parent().and_then(Path::file_name)
    } else {
        path.file_stem()
    };
    sanitize_page_component(
        canonical
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("source"),
    )
}

fn write_page(
    path: &Path,
    title: &str,
    declarations: &[DocumentedDeclaration],
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut output = format!("{title}\n{}\n\n", "=".repeat(title.chars().count()));
    for declaration in declarations {
        let _ = writeln!(
            output,
            "{}\n{}\n",
            declaration.name,
            "-".repeat(declaration.name.chars().count())
        );
        output.push_str(".. code-block:: topal\n\n");
        for line in declaration.syntax.lines() {
            let _ = writeln!(output, "   {line}");
        }
        output.push('\n');
        if let Some(documentation) = &declaration.documentation {
            output.push_str(documentation);
            output.push_str("\n\n");
        }
        let documented = declaration
            .parameters
            .iter()
            .filter(|parameter| parameter.documentation.is_some())
            .collect::<Vec<_>>();
        if !documented.is_empty() {
            output.push_str("Parameters\n~~~~~~~~~~\n\n");
            for parameter in documented {
                let _ = writeln!(
                    output,
                    "``{}``\n   {}\n",
                    parameter.syntax,
                    parameter.documentation.as_deref().unwrap_or_default()
                );
            }
        }
    }
    fs::write(path, output).map_err(|error| format!("{}: {error}", path.display()))
}

fn write_index(path: &Path, pages: &[String]) -> Result<(), String> {
    let mut output =
        "Topal reference\n===============\n\n.. toctree::\n   :maxdepth: 2\n\n".to_owned();
    for page in pages {
        let _ = writeln!(output, "   {page}");
    }
    fs::write(path, output).map_err(|error| format!("{}: {error}", path.display()))
}
