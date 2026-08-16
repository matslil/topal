use std::fs;
use std::process::Command;

fn temporary_source(name: &str, source: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("topal-lint-{}-{name}.t", std::process::id()));
    fs::write(&path, source).unwrap();
    path
}

#[test]
fn lists_and_explains_the_built_in_catalog() {
    let list = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args([
            "--list",
            "--enable",
            "namespace:lang",
            "--disable",
            "tag:lang best-practice tag concurrency",
            "--enable",
            "lang best-practice task state-machine",
        ])
        .output()
        .unwrap();
    assert!(list.status.success());
    let output = String::from_utf8(list.stdout).unwrap();
    assert!(
        output.contains("lang best-practice task state-machine\tv0.1\tproposed\tenabled\twarning")
    );

    let explain = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args(["--explain", "lang best-practice task state-machine"])
        .output()
        .unwrap();
    assert!(explain.status.success());
    assert!(
        String::from_utf8(explain.stdout)
            .unwrap()
            .contains("class: recommended")
    );
}

#[test]
fn emits_shared_style_terminal_and_json_syntax_diagnostics() {
    let path = temporary_source(
        "invalid",
        "#!/usr/bin/env topal\n# Demonstrates linter diagnostics.\nvalue is #\n",
    );
    let terminal = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(terminal.status.code(), Some(1));
    assert!(
        String::from_utf8(terminal.stderr)
            .unwrap()
            .contains("error[E-UNKNOWN-TOKEN]")
    );

    let json = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args(["--format", "json"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(json.status.code(), Some(1));
    let first = json.stdout.split(|byte| *byte == b'\n').next().unwrap();
    let finding: serde_json::Value = serde_json::from_slice(first).unwrap();
    assert_eq!(finding["code"], "E-UNKNOWN-TOKEN");
    assert_eq!(finding["severity"], "error");
    fs::remove_file(path).unwrap();
}

#[test]
fn accepts_clean_shared_language_source() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg(root.join("examples/language/task-message-transactions.t"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn external_catalogs_are_explicit_and_cannot_replace_owned_identities() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args(["--catalog"])
        .arg(root.join("best-practices/generated/lint-catalog.json"))
        .arg("--list")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("supplied by more than one catalog")
    );
}

#[test]
fn rejects_a_selector_that_matches_no_catalog_entry() {
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args(["--list", "--enable", "lang best-practice missing"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("matches no best-practice")
    );
}
