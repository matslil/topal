use std::fs;
use std::path::{Path, PathBuf};

use topal_language::{Session, load_module_tree};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn topal_tests() -> Vec<PathBuf> {
    let directory = repository().join("tests/standard-library");
    let mut paths = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "t"))
        .collect::<Vec<_>>();
    paths.sort();
    assert!(!paths.is_empty(), "{} contains no Topal tests", directory.display());
    paths
}

#[test]
fn topal_standard_library_expectations_pass() {
    for path in topal_tests() {
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &repository().join("library"), &mut trace).unwrap();
        let source = fs::read_to_string(&path).unwrap();
        session.evaluate_source_file(&source, &mut trace).unwrap_or_else(|error| {
            panic!("{}", error.render(&path.display().to_string()))
        });
    }
}

#[test]
fn false_topal_expectation_fails_without_a_rust_golden_value() {
    let mut session = Session::new();
    let mut trace = Vec::new();
    let error = session
        .evaluate_source_file(
            "use language ( version is v0.1 )\nPass is Boolean constraint { value } value = true\nfailed : Pass is Pass false\nfailed",
            &mut trace,
        )
        .unwrap_err();
    assert_eq!(error.code, "E-CONSTRAINT-REJECTED");
}
