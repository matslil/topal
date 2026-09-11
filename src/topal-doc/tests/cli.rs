use std::fs;
use std::process::Command;

use topal_source::SourceText;
use topal_syntax::{extract_documentation, lex, parse};

fn scratch(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("topal-doc-{}-{name}", std::process::id()))
}

#[test]
fn generates_explicit_source_and_optional_lang_pages() {
    let output = scratch("explicit");
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../../library/std/module.t");
    let result = Command::new(env!("CARGO_BIN_EXE_topal-doc"))
        .args([
            "--output",
            output.to_str().unwrap(),
            "--include-lang",
            source,
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let standard = fs::read_to_string(output.join("std.rst")).unwrap();
    let lang = fs::read_to_string(output.join("lang.rst")).unwrap();
    assert!(standard.contains("Return the smaller value"));
    assert!(standard.contains(".. code-block:: topal"));
    assert!(lang.contains("lang Int"));
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn directory_traversal_is_shallow_unless_recursive() {
    let input = scratch("input");
    let nested = input.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let header = "use language (\n  version is v0.1\n)\n### A value.\npub value is 1\n";
    fs::write(input.join("direct.t"), header).unwrap();
    fs::write(nested.join("nested.t"), header).unwrap();
    let shallow = scratch("shallow");
    let recursive = scratch("recursive");
    assert!(
        Command::new(env!("CARGO_BIN_EXE_topal-doc"))
            .args([
                "--output",
                shallow.to_str().unwrap(),
                input.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    assert!(shallow.join("direct.rst").is_file());
    assert!(!shallow.join("nested.rst").exists());
    assert!(
        Command::new(env!("CARGO_BIN_EXE_topal-doc"))
            .args([
                "--output",
                recursive.to_str().unwrap(),
                "--recurse",
                input.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    assert!(recursive.join("nested/nested.rst").is_file());
    fs::remove_dir_all(input).unwrap();
    fs::remove_dir_all(shallow).unwrap();
    fs::remove_dir_all(recursive).unwrap();
}

fn topal_sources(directory: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            topal_sources(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "t") {
            files.push(path);
        }
    }
}

fn assert_no_single_module_directories(directory: &std::path::Path) {
    let entries = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert!(
        !(entries.len() == 1
            && entries[0]
                .file_name()
                .is_some_and(|name| name == "module.t")),
        "{} contains only module.t",
        directory.display()
    );
    for path in entries.into_iter().filter(|path| path.is_dir()) {
        assert_no_single_module_directories(&path);
    }
}

#[test]
fn recursively_generates_the_complete_standard_library() {
    let output = scratch("standard-library");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library/std");
    let result = Command::new(env!("CARGO_BIN_EXE_topal-doc"))
        .args([
            "--output",
            output.to_str().unwrap(),
            "--recurse",
            root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.join("graph.rst").is_file());
    assert!(output.join("build/graph.rst").is_file());
    assert!(output.join("pattern/regex.rst").is_file());
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn every_public_standard_library_function_has_documentation() {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = repository.join("library/std");
    assert_no_single_module_directories(&root);
    let mut corpus_files = Vec::new();
    topal_sources(&repository.join("tests"), &mut corpus_files);
    topal_sources(&repository.join("examples"), &mut corpus_files);
    let mut corpus = corpus_files
        .into_iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    corpus.push_str(
        &fs::read_to_string(repository.join("src/topal-language/src/modules.rs")).unwrap(),
    );
    let mut files = Vec::new();
    topal_sources(&root, &mut files);
    let mut functions = 0;
    for path in files {
        let text = fs::read_to_string(&path).unwrap();
        let source = SourceText::new(&text).unwrap();
        let lexed = lex(&source);
        let parsed = parse(&source, &lexed);
        assert!(
            parsed.diagnostics.is_empty(),
            "{} has parser diagnostics",
            path.display()
        );
        for declaration in extract_documentation(&source, &lexed, &parsed) {
            if declaration.syntax.starts_with("pub ") && declaration.syntax.contains(" is fn") {
                functions += 1;
                assert!(
                    declaration.documentation.is_some(),
                    "{} `{}` is undocumented",
                    path.display(),
                    declaration.name
                );
                assert!(
                    corpus.contains(&declaration.name),
                    "{} `{}` has no test or executable-example reference",
                    path.display(),
                    declaration.name
                );
            }
        }
    }
    assert!(functions > 0);
}
