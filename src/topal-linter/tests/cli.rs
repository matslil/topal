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
    let terminal = String::from_utf8(terminal.stderr).unwrap();
    assert!(terminal.contains("error[E-UNKNOWN-TOKEN]"));
    assert!(terminal.contains("3 | value is #"));
    assert!(terminal.contains("|          ^"));

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
fn aggregates_shared_diagnostics_as_sarif() {
    let syntax_path = temporary_source(
        "sarif-syntax",
        "#!/usr/bin/env topal\n# Demonstrates SARIF syntax diagnostics.\nvalue is #\n",
    );
    let rule_path = temporary_source(
        "sarif-rule",
        "use language (\n  version is v0.1\n)\n# Demonstrates SARIF best-practice provenance.\nCounter is Task (queue-size is 2)\nservice is Counter\n  start is fn (initial : Nat) -> Completed\n    Completed\n  increment is fn (_ : MessageContext, amount : Nat) -> Unit\n    ()\n  count : Nat\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args([
            "--enable",
            "lang best-practice task declaration-order",
            "--format",
            "sarif",
        ])
        .arg(&syntax_path)
        .arg(&rule_path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["version"], "2.1.0");
    assert_eq!(report["runs"][0]["tool"]["driver"]["name"], "topal-lint");
    let results = report["runs"][0]["results"].as_array().unwrap();
    let syntax = results
        .iter()
        .find(|result| result["ruleId"] == "E-UNKNOWN-TOKEN")
        .unwrap();
    assert_eq!(syntax["level"], "error");
    assert_eq!(
        syntax["locations"][0]["physicalLocation"]["region"]["startLine"],
        3
    );
    let best_practice = results
        .iter()
        .find(|result| result["ruleId"] == "L-TASK-DECLARATION-ORDER")
        .unwrap();
    assert_eq!(best_practice["level"], "warning");
    assert_eq!(
        best_practice["properties"]["bestPractice"],
        "lang best-practice task declaration-order"
    );
    assert_eq!(best_practice["properties"]["bestPracticeVersion"], "v0.1");
    assert_eq!(best_practice["properties"]["ruleVersion"], "v0.1");
    fs::remove_file(syntax_path).unwrap();
    fs::remove_file(rule_path).unwrap();
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

#[test]
fn proposed_task_order_rule_is_off_by_default_and_configurable() {
    let path = temporary_source(
        "task-order",
        "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nservice is Counter\n  start is fn (initial : Nat) -> Completed\n    Completed\n  increment is fn (_ : MessageContext, amount : Nat) -> Unit\n    ()\n  count : Nat\n",
    );
    let default = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg(&path)
        .output()
        .unwrap();
    assert!(default.status.success());
    assert!(default.stderr.is_empty());

    let enabled = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args([
            "--enable",
            "lang best-practice task declaration-order",
            "--format",
            "json",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(enabled.status.success());
    let finding: serde_json::Value = serde_json::from_slice(&enabled.stdout).unwrap();
    assert_eq!(finding["severity"], "warning");
    assert_eq!(
        finding["best_practice"],
        "lang best-practice task declaration-order"
    );
    assert_eq!(finding["best_practice_version"], "v0.1");
    assert_eq!(finding["rule_version"], "v0.1");
    assert_eq!(finding["code"], "L-TASK-DECLARATION-ORDER");
    assert!(finding["help"].as_str().unwrap().contains("before `start`"));

    let as_error = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args([
            "--enable",
            "lang best-practice task declaration-order",
            "--severity",
            "lang best-practice task declaration-order=error",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(as_error.status.code(), Some(1));
    assert!(
        String::from_utf8(as_error.stderr)
            .unwrap()
            .contains("error[L-TASK-DECLARATION-ORDER]")
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn shared_task_order_example_conforms_when_the_proposed_rule_is_enabled() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args(["--enable", "lang best-practice task declaration-order"])
        .arg(root.join("examples/language/task-declaration-order.t"))
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
fn topal_state_machine_rule_finds_state_without_a_message_transition() {
    let path = temporary_source(
        "task-state-machine",
        "use language (\n  version is v0.1\n)\n# Demonstrates a stateful task without an event transition.\nCounter is Task (queue-size is 2)\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args([
            "--enable",
            "lang best-practice task state-machine",
            "--format",
            "json",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let finding: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(finding["code"], "L-TASK-STATE-MACHINE");
    assert_eq!(
        finding["best_practice"],
        "lang best-practice task state-machine"
    );
    assert!(
        finding["help"]
            .as_str()
            .unwrap()
            .contains("state-changing event")
    );
    fs::remove_file(path).unwrap();

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let conforming = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args(["--enable", "lang best-practice task state-machine"])
        .arg(root.join("examples/language/task-message-transactions.t"))
        .output()
        .unwrap();
    assert!(
        conforming.status.success(),
        "{}",
        String::from_utf8_lossy(&conforming.stderr)
    );
    assert!(conforming.stderr.is_empty());
}

#[test]
fn validates_explicit_lint_variant_rule_modules() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let valid = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg("--check-rule")
        .arg(root.join("examples/linter/task-declaration-order-rule.t"))
        .output()
        .unwrap();
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );

    let path = temporary_source(
        "missing-lint-variant",
        "use language ( version is v0.1, features is ( debug ) )\nrule is fn static () -> Unit\n  ()\n",
    );
    let rejected = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg("--check-rule")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("L-RULE-VARIANT")
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn rule_entry_point_must_exist_and_be_static() {
    let path = temporary_source(
        "ordinary-rule",
        "use language ( version is v0.1, features is ( lint ) )\ncheck is fn () -> Unit\n  ()\n",
    );
    let rejected = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args(["--check-rule"])
        .arg(&path)
        .args(["--entry-point", "check"])
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("L-RULE-STATIC")
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn lint_rule_module_cannot_acquire_debugger_authority() {
    let path = temporary_source(
        "debug-authority",
        "use language ( version is v0.1, features is ( lint, debug ) )\nrule is fn static () -> Unit\n  ()\n",
    );
    let rejected = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg("--check-rule")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("L-RULE-AUTHORITY")
    );
    fs::remove_file(path).unwrap();
}
