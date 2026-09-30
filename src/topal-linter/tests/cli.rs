//! Functional conformance evidence for TOPAL-BEST-PRACTICE-IDENTITY-001
//! through TOPAL-BEST-PRACTICE-RECTIFICATION-001, TOPAL-DIAG-MODEL-001,
//! TOPAL-DIAG-ADAPTER-001, and the TOPAL-LINT-* tool requirements.

use std::fs;
use std::process::Command;

use std::collections::BTreeSet;

use topal_best_practices::Catalog;

fn temporary_source(name: &str, source: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("topal-lint-{}-{name}.t", std::process::id()));
    fs::write(&path, source).unwrap();
    path
}

struct ConfigurableDiagnosticCase {
    identity: &'static str,
    code: &'static str,
    source: &'static str,
    source_suppressed: &'static str,
}

const CONFIGURABLE_DIAGNOSTIC_CASES: &[ConfigurableDiagnosticCase] = &[
    ConfigurableDiagnosticCase {
        identity: "lang best-practice design-pattern ap-03-algebraic-state",
        code: "L-DESIGN-PATTERN-AP-03",
        source: "use language (version is v0.1)\ngate-state : Boolean is false\n",
        source_suppressed: "use language (version is v0.1)\nlang disable-diagnostic ( lang best-practice design-pattern ap-03-algebraic-state )\ngate-state : Boolean is false\n",
    },
    ConfigurableDiagnosticCase {
        identity: "lang best-practice design-pattern ap-04-typestate-protocol",
        code: "L-DESIGN-PATTERN-AP-04",
        source: "use language (version is v0.1)\nconnection-protocol : Boolean is false\n",
        source_suppressed: "use language (version is v0.1)\nlang disable-diagnostic ( lang best-practice design-pattern ap-04-typestate-protocol )\nconnection-protocol : Boolean is false\n",
    },
    ConfigurableDiagnosticCase {
        identity: "lang best-practice task declaration-order",
        code: "L-TASK-DECLARATION-ORDER",
        source: "use language (version is v0.1)\nCounter is Task (queue-size is 2)\nservice is Counter\n  start is fn (initial : Nat) -> Completed\n    Completed\n  increment is fn (_ : MessageContext, amount : Nat) -> Unit\n    ()\n  count : Nat\n",
        source_suppressed: "use language (version is v0.1)\nCounter is Task (queue-size is 2)\nlang disable-diagnostic ( lang best-practice task declaration-order )\nservice is Counter\n  start is fn (initial : Nat) -> Completed\n    Completed\n  increment is fn (_ : MessageContext, amount : Nat) -> Unit\n    ()\n  count : Nat\n",
    },
    ConfigurableDiagnosticCase {
        identity: "lang best-practice task state-machine",
        code: "L-TASK-STATE-MACHINE",
        source: "use language (version is v0.1)\nCounter is Task (queue-size is 2)\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n",
        source_suppressed: "use language (version is v0.1)\nCounter is Task (queue-size is 2)\nlang disable-diagnostic ( lang best-practice task state-machine )\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n",
    },
];

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
            .contains("class: recommended\nrecommendation: Start an event-driven state machine")
    );
}

#[test]
fn explains_lifecycle_applicability_and_rule_attachment() {
    let mut catalog = Catalog::builtin();
    catalog
        .entries
        .retain(|entry| entry.identity.ends_with("declaration-order"));
    let entry = &mut catalog.entries[0];
    entry.identity = "org.example best-practice historical order".into();
    entry.status.kind = "obsolete".into();
    entry.status.since_language_version = Some("v0.2".into());
    entry.status.explanation = Some("the language now orders these declarations".into());
    entry.status.replacement = Some(vec!["org.example".into(), "compiler-check".into()]);
    let path = std::env::temp_dir().join(format!(
        "topal-lint-explain-catalog-{}.json",
        std::process::id()
    ));
    fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg("--catalog")
        .arg(&path)
        .args(["--explain", "org.example best-practice historical order"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert!(output.contains("obsolete since: v0.2"));
    assert!(output.contains("status explanation: the language now orders"));
    assert!(output.contains("replacement: org.example compiler-check"));
    assert!(output.contains("required features: task"));
    assert!(output.contains("excluded features: none"));
    assert!(output.contains("rule: topal rule v0.1 syntax task-declaration-order/1"));

    let entry = &mut catalog.entries[0];
    entry.status.kind = "deprecated".into();
    entry.status.since_language_version = None;
    entry.status.explanation = Some("a clearer recommendation replaced this one".into());
    entry.lint_rule = None;
    fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg("--catalog")
        .arg(&path)
        .args(["--explain", "org.example best-practice historical order"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert!(output.contains("status: deprecated"));
    assert!(output.contains("status explanation: a clearer recommendation"));
    assert!(output.contains("rule: none"));
    assert!(!output.contains("obsolete since:"));
    fs::remove_file(path).unwrap();
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
fn every_configurable_diagnostic_can_be_disabled_by_selector_or_source_control() {
    let catalog_codes = Catalog::builtin()
        .entries
        .into_iter()
        .filter_map(|entry| {
            entry
                .lint_rule
                .map(|rule| (entry.identity, rule.diagnostic_code))
        })
        .collect::<BTreeSet<_>>();
    let covered_codes = CONFIGURABLE_DIAGNOSTIC_CASES
        .iter()
        .map(|case| (case.identity.to_owned(), case.code.to_owned()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        covered_codes, catalog_codes,
        "add command-line and source-control coverage for every configurable lint diagnostic"
    );

    for case in CONFIGURABLE_DIAGNOSTIC_CASES {
        let path = temporary_source(case.code, case.source);
        let enabled = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
            .args(["--enable", case.identity])
            .arg(&path)
            .output()
            .unwrap();
        assert!(enabled.status.success());
        assert!(String::from_utf8_lossy(&enabled.stderr).contains(case.code));

        let disabled = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
            .args(["--enable", case.identity, "--disable", case.identity])
            .arg(&path)
            .output()
            .unwrap();
        assert!(disabled.status.success());
        assert!(disabled.stderr.is_empty(), "{}", case.code);
        fs::remove_file(path).unwrap();

        let path = temporary_source(&format!("{}-source", case.code), case.source_suppressed);
        let suppressed = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
            .args(["--enable", case.identity])
            .arg(&path)
            .output()
            .unwrap();
        assert!(suppressed.status.success());
        assert!(suppressed.stderr.is_empty(), "{}", case.code);
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn source_controls_cannot_hide_shared_language_errors() {
    let path = temporary_source(
        "unsuppressible-syntax-error",
        "use language (version is v0.1)\nlang disable-diagnostic ( E-UNKNOWN-TOKEN )\nvalue is #\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("error[E-UNKNOWN-TOKEN]"));
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
    assert_eq!(
        best_practice["properties"]["checkability"],
        "formally-decidable"
    );
    assert!(
        best_practice["help"]["text"]
            .as_str()
            .unwrap()
            .contains("state fields")
    );
    assert!(
        best_practice["locations"][0]["physicalLocation"]["region"]["endColumn"]
            .as_u64()
            .unwrap()
            > best_practice["locations"][0]["physicalLocation"]["region"]["startColumn"]
                .as_u64()
                .unwrap()
    );
    assert_eq!(
        best_practice["properties"]["rectification"]["kind"],
        "suggestion"
    );
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
fn algebraic_state_pattern_fixtures_distinguish_advisory_and_limitation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let identity = "lang best-practice design-pattern ap-03-algebraic-state";
    let positive = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args(["--enable", identity])
        .arg(root.join("examples/language/design-pattern-algebraic-state.t"))
        .output()
        .unwrap();
    assert!(
        positive.status.success(),
        "{}",
        String::from_utf8_lossy(&positive.stderr)
    );

    for (fixture, expected) in [
        (
            "examples/linter/design-pattern-ap-03-boolean-state.t",
            "Boolean state",
        ),
        (
            "examples/linter/design-pattern-ap-03-state-explosion.t",
            "state explosion",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
            .args(["--enable", identity])
            .arg(root.join(fixture))
            .output()
            .unwrap();
        assert!(output.status.success());
        let diagnostic = String::from_utf8(output.stderr).unwrap();
        assert!(diagnostic.contains("warning[L-DESIGN-PATTERN-AP-03]"));
        assert!(diagnostic.contains(expected));
    }
}

#[test]
fn typestate_protocol_fixtures_distinguish_model_limitation_and_advisory() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let identity = "lang best-practice design-pattern ap-04-typestate-protocol";
    for (fixture, expected) in [
        (
            "examples/language/design-pattern-typestate-protocol.t",
            "cannot statically enforce transition order",
        ),
        (
            "examples/linter/design-pattern-ap-04-boolean-protocol.t",
            "Boolean protocol state",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
            .args(["--enable", identity])
            .arg(root.join(fixture))
            .output()
            .unwrap();
        assert!(output.status.success());
        let diagnostic = String::from_utf8(output.stderr).unwrap();
        assert!(diagnostic.contains("warning[L-DESIGN-PATTERN-AP-04]"));
        assert!(diagnostic.contains(expected));
    }
}

#[test]
fn accepts_standard_library_sources_and_topal_tests() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for relative in [
        "library/std/module.t",
        "library/std/data/spans.t",
        "library/std/transfer/queues.t",
        "library/std/store/memory.t",
        "library/std/network/addresses.t",
        "library/std/device/i2c.t",
        "library/std/web/http.t",
        "library/std/build/module.t",
        "library/std/build/graph.t",
        "library/advent-of-code/module.t",
        "library/advent-of-code/geometry.t",
        "library/advent-of-code/graph.t",
        "library/advent-of-code/machine.t",
        "library/advent-of-code/packing.t",
        "tests/standard-library/harness.t",
        "tests/standard-library/data-spans.t",
        "tests/standard-library/transfer-queues.t",
        "tests/standard-library/store-memory.t",
        "tests/standard-library/network-addresses.t",
        "tests/standard-library/device-i2c.t",
        "tests/standard-library/build-graph.t",
        "tests/standard-library/web-http.t",
        "examples/data-transfer/firewall.t",
        "examples/data-transfer/packet-filter.t",
        "examples/data-transfer/rest-controller.t",
    ] {
        let path = root.join(relative);
        let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty(), "{}", path.display());
        assert!(output.stderr.is_empty(), "{}", path.display());
    }
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
    assert!(
        finding["help"]
            .as_str()
            .unwrap()
            .contains("Declare all state fields")
    );
    assert_eq!(finding["end_line"], finding["line"]);
    assert!(finding["end_column"].as_u64().unwrap() > finding["column"].as_u64().unwrap());
    assert_eq!(finding["rectification"]["kind"], "suggestion");
    assert!(
        finding["rectification"]["message"]
            .as_str()
            .unwrap()
            .contains("before `start`")
    );

    let terminal = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args(["--enable", "lang best-practice task declaration-order"])
        .arg(&path)
        .output()
        .unwrap();
    let terminal = String::from_utf8(terminal.stderr).unwrap();
    assert!(terminal.contains("10 |   count : Nat"));
    assert!(terminal.contains("|   ^^^^^"));
    assert!(terminal.contains("help: Declare all state fields"));

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
    let terminal = String::from_utf8(as_error.stderr).unwrap();
    assert!(terminal.contains("error[L-TASK-DECLARATION-ORDER]"));
    assert!(terminal.contains("= suggestion:"));

    let before = fs::read_to_string(&path).unwrap();
    let suggestion_only = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args([
            "--fix",
            "--enable",
            "lang best-practice task declaration-order",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(suggestion_only.status.success());
    assert!(
        String::from_utf8(suggestion_only.stderr)
            .unwrap()
            .contains("= suggestion:")
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
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
    assert_eq!(finding["checkability"], "heuristic");
    assert!(
        finding["confidence"]
            .as_str()
            .unwrap()
            .contains("indirect transitions")
    );
    assert_eq!(
        finding["best_practice"],
        "lang best-practice task state-machine"
    );
    assert!(
        finding["rectification"]["message"]
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
fn source_suppression_uses_identity_instead_of_configured_severity() {
    let path = temporary_source(
        "source-suppression",
        "use language (\n  version is v0.1\n)\n# Demonstrates severity-neutral suppression of one lint finding.\nCounter is Task (queue-size is 2)\nlang disable-diagnostic ( lang best-practice task state-machine )\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .args([
            "--enable",
            "lang best-practice task state-machine",
            "--severity",
            "lang best-practice task state-machine=error",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    fs::remove_file(path).unwrap();
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

#[test]
fn lint_rule_admission_enforces_deterministic_resource_bounds() {
    let oversized = temporary_source(
        "oversized-rule",
        &format!(
            "use language ( version is v0.1, features is ( lint ) )\n# {}\nrule is fn static () -> Boolean\n  true\n",
            "x".repeat(17_000)
        ),
    );
    let rejected = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg("--check-rule")
        .arg(&oversized)
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("L-RULE-RESOURCE")
    );
    fs::remove_file(oversized).unwrap();

    let unbounded_operation = temporary_source(
        "unbounded-rule-operation",
        "use language ( version is v0.1, features is ( lint ) )\nrule is fn static () -> Int\n  2 ^ 1000000\n",
    );
    let rejected = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg("--check-rule")
        .arg(&unbounded_operation)
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("L-RULE-CONTAINMENT")
    );
    fs::remove_file(unbounded_operation).unwrap();

    let nested = format!("{}true{}", "not (".repeat(130), ")".repeat(130));
    let excessive_tree = temporary_source(
        "excessive-rule-tree",
        &format!(
            "use language ( version is v0.1, features is ( lint ) )\nrule is fn static () -> Boolean\n  {nested}\n"
        ),
    );
    let rejected = Command::new(env!("CARGO_BIN_EXE_topal-lint"))
        .arg("--check-rule")
        .arg(&excessive_tree)
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("L-RULE-RESOURCE")
    );
    fs::remove_file(excessive_tree).unwrap();
}
