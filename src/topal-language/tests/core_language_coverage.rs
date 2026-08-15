use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct LedgerEntry {
    rules: usize,
    phase: usize,
    owner: String,
    disposition: String,
    status: String,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root must exist")
}

fn table_cell(cell: &str) -> String {
    cell.trim().trim_matches('`').to_owned()
}

fn read_ledger(root: &Path) -> BTreeMap<String, LedgerEntry> {
    let text = fs::read_to_string(root.join("se/core-language-coverage.md"))
        .expect("coverage ledger must be readable");
    let mut entries = BTreeMap::new();

    for line in text.lines() {
        if !line.starts_with("| `spec/") {
            continue;
        }
        let cells: Vec<_> = line.split('|').skip(1).map(table_cell).collect();
        assert!(cells.len() >= 6, "malformed coverage row: {line}");
        let path = cells[0].clone();
        let entry = LedgerEntry {
            rules: cells[1].parse().expect("rule count must be an integer"),
            phase: cells[2].parse().expect("phase must be an integer"),
            owner: cells[3].clone(),
            disposition: cells[4].clone(),
            status: cells[5].clone(),
        };
        assert!(
            entries.insert(path.clone(), entry).is_none(),
            "duplicate {path}"
        );
    }

    entries
}

#[test]
fn every_stable_specification_rule_has_a_completion_owner() {
    let root = workspace_root();
    let entries = read_ledger(&root);
    let traceability = fs::read_to_string(root.join("se/traceability.md"))
        .expect("traceability matrix must be readable");
    let implementation_coverage = traceability
        .split_once("## Implementation coverage")
        .expect("traceability must contain implementation coverage")
        .1;
    let mut observed = BTreeMap::new();

    for directory_entry in fs::read_dir(root.join("spec")).expect("spec must be readable") {
        let path = directory_entry.expect("spec entry must be readable").path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("specification must be readable");
        let rules = text
            .lines()
            .filter(|line| line.starts_with("### TOPAL-"))
            .count();
        if rules == 0 {
            continue;
        }
        let relative = format!("spec/{}", path.file_name().unwrap().to_string_lossy());
        observed.insert(relative, rules);
    }

    assert_eq!(
        entries.keys().collect::<Vec<_>>(),
        observed.keys().collect::<Vec<_>>(),
        "coverage ledger and stable specification files differ"
    );

    for (path, actual_rules) in observed {
        let entry = &entries[&path];
        assert_eq!(entry.rules, actual_rules, "stale rule count for {path}");
        assert!(
            (2..=9).contains(&entry.phase),
            "invalid completion phase for {path}"
        );
        assert!(
            !entry.owner.is_empty(),
            "missing implementation owner for {path}"
        );
        assert!(
            !entry.disposition.is_empty(),
            "missing disposition for {path}"
        );
        assert!(
            matches!(entry.status.as_str(), "planned" | "complete"),
            "invalid completion status for {path}"
        );
        if entry.status == "complete" {
            let specification = fs::read_to_string(root.join(&path)).unwrap();
            let mut missing = Vec::new();
            for line in specification.lines() {
                let Some(rule) = line
                    .strip_prefix("### ")
                    .and_then(|line| line.split_once(' '))
                else {
                    continue;
                };
                if !implementation_coverage.contains(rule.0) {
                    missing.push(rule.0);
                }
            }
            assert!(
                missing.is_empty(),
                "completed rules from {path} lack implementation evidence: {missing:?}"
            );
        }
    }
}

#[test]
fn accepted_core_has_no_planned_rule_and_examples_explain_their_feature() {
    let root = workspace_root();
    let entries = read_ledger(&root);
    let bootstrap = fs::read_to_string(root.join("se/standard-library-bootstrap.md"))
        .expect("standard-library bootstrap contract must be readable");
    assert!(bootstrap.contains("`v0.1`"));
    assert!(bootstrap.contains("no implicit reflection"));
    let specification_planned = entries
        .iter()
        .filter_map(|(path, entry)| (entry.status != "complete").then_some(path))
        .collect::<Vec<_>>();
    let corrections = fs::read_to_string(root.join("se/core-language-corrections.md"))
        .expect("correction ledger must be readable");
    let correction_planned = corrections
        .lines()
        .filter(|line| line.starts_with('|') && line.ends_with("| planned |"))
        .count();
    let completion_plan = fs::read_to_string(root.join("se/core-language-completion-plan.md"))
        .expect("completion plan must be readable");
    assert_eq!(
        completion_plan.contains("**Completion record:**"),
        specification_planned.is_empty() && correction_planned == 0,
        "a completion record is valid only when both ledgers are terminal"
    );

    let mut examples = fs::read_dir(root.join("examples/language"))
        .expect("interpreter examples must be readable")
        .map(|entry| entry.expect("example entry must be readable").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "t"))
        .collect::<Vec<_>>();
    examples.sort();
    assert!(!examples.is_empty());
    for path in examples {
        let source = fs::read_to_string(&path).expect("example source must be readable");
        assert!(
            source
                .lines()
                .any(|line| line.starts_with("# Demonstrates")),
            "{} does not explain the demonstrated feature",
            path.display()
        );
        assert!(
            source.contains("version is v0.1"),
            "{} does not select the accepted core revision",
            path.display()
        );
    }
}
