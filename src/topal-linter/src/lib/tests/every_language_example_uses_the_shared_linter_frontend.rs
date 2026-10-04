// Shared-engine conformance evidence for TOPAL-SYN-SOURCE-001,
// TOPAL-SYN-GRAMMAR-001, TOPAL-BEST-PRACTICE-RULE-CONTAINMENT-001,
// and TOPAL-BEST-PRACTICE-RECTIFICATION-001.

use super::*;
use std::path::Path;

#[test]
fn every_language_example_uses_the_shared_linter_frontend() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language");
    let mut examples = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "t"))
        .collect::<Vec<_>>();
    examples.sort();
    assert_eq!(examples.len(), 309);
    for example in examples {
        let source = std::fs::read_to_string(&example).unwrap();
        let report = lint_text(&source, &[]).unwrap();
        assert!(
            !report.has_errors,
            "{} produced linter errors: {:?}",
            example.display(),
            report.diagnostics
        );
    }
}

#[test]
fn automatic_edits_are_ordered_and_review_required_edits_are_ignored() {
    let edits = [
        SourceEdit {
            span: Span::new(4, 5),
            replacement: "B".into(),
            safety: RectificationSafety::SyntaxPreserving,
        },
        SourceEdit {
            span: Span::new(0, 1),
            replacement: "A".into(),
            safety: RectificationSafety::SemanticsProven,
        },
        SourceEdit {
            span: Span::new(2, 3),
            replacement: "ignored".into(),
            safety: RectificationSafety::ReviewRequired,
        },
    ];
    assert_eq!(apply_source_edits("01245", &edits).unwrap(), "A124B");
}

#[test]
fn automatic_edits_reject_overlap_and_invalid_utf8_boundaries() {
    let edit = |span| SourceEdit {
        span,
        replacement: String::new(),
        safety: RectificationSafety::PresentationOnly,
    };
    assert!(
        apply_source_edits("abcd", &[edit(Span::new(0, 2)), edit(Span::new(1, 3))])
            .unwrap_err()
            .contains("overlap")
    );
    assert!(
        apply_source_edits("å", &[edit(Span::new(1, 2))])
            .unwrap_err()
            .contains("invalid source span")
    );
}

#[test]
fn rectified_candidate_must_reparse_before_replacement() {
    assert!(require_clean_syntax("value is 1").is_ok());
    assert!(require_clean_syntax("value is #").is_err());
}

#[test]
fn accepted_candidate_replaces_the_complete_file_transactionally() {
    let path = std::env::temp_dir().join(format!(
        "topal-lint-rectification-transaction-{}.t",
        std::process::id()
    ));
    fs::write(&path, "value is 1\n").unwrap();
    replace_source_atomically(&path, "value is 2\n").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "value is 2\n");
    fs::remove_file(path).unwrap();
}

fn entry() -> CatalogEntry {
    Catalog::builtin().entries.remove(0)
}

#[test]
fn exact_identity_overrides_namespace_and_tag_regardless_of_order() {
    let entry = entry();
    let settings = vec![
        Override {
            selector: Selector::Identity(entry.identity.clone()),
            enabled: Some(true),
            severity: SeveritySetting::Set(Severity::Error),
            order: 0,
        },
        Override {
            selector: Selector::Namespace("lang".into()),
            enabled: Some(false),
            severity: SeveritySetting::Set(Severity::Warning),
            order: 1,
        },
        Override {
            selector: Selector::Tag(entry.tags[0].clone()),
            enabled: Some(false),
            severity: SeveritySetting::Keep,
            order: 2,
        },
    ];
    assert_eq!(
        policy(&entry, &settings).unwrap(),
        Policy {
            enabled: true,
            severity: Severity::Error
        }
    );
}

#[test]
fn later_setting_wins_within_one_specificity() {
    let entry = entry();
    let settings = vec![
        Override {
            selector: Selector::Identity(entry.identity.clone()),
            enabled: Some(true),
            severity: SeveritySetting::Keep,
            order: 0,
        },
        Override {
            selector: Selector::Identity(entry.identity.clone()),
            enabled: Some(false),
            severity: SeveritySetting::Keep,
            order: 1,
        },
    ];
    assert!(!policy(&entry, &settings).unwrap().enabled);
}

#[test]
fn topal_rule_decides_adjacent_phase_order() {
    let entry = Catalog::builtin()
        .entries
        .into_iter()
        .find(|entry| entry.identity.ends_with("declaration-order"))
        .unwrap();
    let rule = entry.lint_rule.unwrap();
    let source = rule.source_text;
    assert!(evaluate_topal_phase_rule(&source, &rule.entry_point, 0, 1).unwrap());
    assert!(evaluate_topal_phase_rule(&source, &rule.entry_point, 2, 3).unwrap());
    assert!(!evaluate_topal_phase_rule(&source, &rule.entry_point, 2, 1).unwrap());
    assert!(!evaluate_topal_phase_rule(&source, &rule.entry_point, 3, 0).unwrap());
}

#[test]
fn topal_rule_decides_whether_state_has_a_message_transition() {
    let entry = Catalog::builtin()
        .entries
        .into_iter()
        .find(|entry| entry.identity.ends_with("state-machine"))
        .unwrap();
    let rule = entry.lint_rule.unwrap();
    assert!(
        evaluate_topal_boolean_rule(&rule.source_text, &rule.entry_point, false, false).unwrap()
    );
    assert!(evaluate_topal_boolean_rule(&rule.source_text, &rule.entry_point, true, true).unwrap());
    assert!(
        !evaluate_topal_boolean_rule(&rule.source_text, &rule.entry_point, true, false).unwrap()
    );
}

#[test]
fn algebraic_state_rule_identifies_its_advisory_and_limitation_shapes() {
    let entry = Catalog::builtin()
        .entries
        .into_iter()
        .find(|entry| entry.identity.ends_with("ap-03-algebraic-state"))
        .unwrap();
    let rule = entry.lint_rule.unwrap();
    assert!(
        evaluate_topal_boolean_rule(&rule.source_text, &rule.entry_point, false, false).unwrap()
    );
    assert!(
        !evaluate_topal_boolean_rule(&rule.source_text, &rule.entry_point, true, false).unwrap()
    );
    assert!(
        !evaluate_topal_boolean_rule(&rule.source_text, &rule.entry_point, false, true).unwrap()
    );

    let boolean_state = "use language (version is v0.1)\ngate-state : Boolean is false\n";
    let report = lint_text(
        boolean_state,
        &["lang best-practice design-pattern ap-03-algebraic-state"],
    )
    .unwrap();
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "L-DESIGN-PATTERN-AP-03" && diagnostic.message.contains("Boolean state")
    }));

    let small_union =
        "use language (version is v0.1)\nWorkflowState is Union\n  Idle\n  Working\n  Failed\n";
    assert!(
        lint_text(
            small_union,
            &["lang best-practice design-pattern ap-03-algebraic-state"]
        )
        .unwrap()
        .diagnostics
        .is_empty()
    );

    let large_union = "use language (version is v0.1)\nWorkflowState is Union\n  Idle\n  Armed\n  Reading\n  Validating\n  Writing\n  Retrying\n  Recovering\n  Failed\n";
    let report = lint_text(
        large_union,
        &["lang best-practice design-pattern ap-03-algebraic-state"],
    )
    .unwrap();
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "L-DESIGN-PATTERN-AP-03"
            && diagnostic.message.contains("state explosion")
    }));
}

#[test]
fn unsupported_read_only_view_revision_is_rejected_before_rule_execution() {
    let mut entry = Catalog::builtin()
        .entries
        .into_iter()
        .find(|entry| entry.identity.ends_with("state-machine"))
        .unwrap();
    entry.lint_rule.as_mut().unwrap().view = "task-state-machine/2".into();
    let source = SourceText::new("use language ( version is v0.1 )\n").unwrap();
    let lexed = lex(&source);
    let parsed = parse(&source, &lexed);
    let Err(error) = topal_rule(&entry, &source, &parsed.statements) else {
        panic!("unsupported view revision was accepted");
    };
    assert!(error.contains("unsupported read-only view"));
}

#[test]
fn in_memory_api_matches_shared_frontend_and_rule_diagnostics() {
    let syntax = lint_text("use language ( version is v0.1 )\nvalue is #\n", &[]).unwrap();
    assert!(syntax.has_errors);
    assert!(
        syntax
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E-UNKNOWN-TOKEN")
    );

    let task = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
    assert!(lint_text(task, &[]).unwrap().diagnostics.is_empty());
    let report = lint_text(task, &["lang best-practice task state-machine"]).unwrap();
    assert!(!report.has_errors);
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "L-TASK-STATE-MACHINE")
        .unwrap();
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert!(
        diagnostic
            .best_practice
            .as_ref()
            .unwrap()
            .suggestion
            .is_some()
    );
}

#[test]
fn structured_source_control_suppresses_regardless_of_configured_severity() {
    let task = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nlang disable-diagnostic ( lang best-practice task state-machine )\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
    let controls = [
        LintControl::Enable("lang best-practice task state-machine".into()),
        LintControl::Severity {
            selector: "lang best-practice task state-machine".into(),
            severity: Severity::Error,
        },
    ];
    let report = lint_text_with_controls(task, &controls).unwrap();
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);

    let scoped = task.replace(
        "lang disable-diagnostic ( lang best-practice task state-machine )",
        "lang push-disable-diagnostic ( lang best-practice task state-machine )",
    ) + "lang pop-disable-diagnostic ( lang best-practice task state-machine )\n";
    let report = lint_text_with_controls(&scoped, &controls).unwrap();
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
}

#[test]
fn in_memory_controls_share_selector_precedence_and_severity() {
    let source = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
    let identity = "lang best-practice task state-machine";
    let report = lint_text_with_controls(
        source,
        &[
            LintControl::Enable("tag:lang best-practice tag architecture".into()),
            LintControl::Severity {
                selector: identity.into(),
                severity: Severity::Error,
            },
        ],
    )
    .unwrap();
    assert!(report.has_errors);
    assert_eq!(report.diagnostics[0].severity, Severity::Error);

    let report = lint_text_with_controls(
        source,
        &[
            LintControl::Enable("tag:lang best-practice tag architecture".into()),
            LintControl::Disable(identity.into()),
        ],
    )
    .unwrap();
    assert!(report.diagnostics.is_empty());
}

#[test]
fn in_memory_engine_requires_explicit_noncolliding_external_catalogs() {
    let mut external = Catalog::builtin();
    external.entries.truncate(1);
    external.entries[0].identity = "org.example best-practice task order".into();
    let source = serde_json::to_string(&external).unwrap();
    let mut engine = LintEngine::builtin();
    engine.add_catalog_json(&source).unwrap();
    assert!(
        engine
            .lint_text(
                "use language ( version is v0.1 )\nvalue is 1\n",
                &[LintControl::Enable(
                    "org.example best-practice task order".into()
                )]
            )
            .is_ok()
    );

    let duplicate = serde_json::to_string(&Catalog::builtin()).unwrap();
    assert!(
        engine
            .add_catalog_json(&duplicate)
            .unwrap_err()
            .contains("more than one catalog")
    );
}

#[test]
fn applicability_uses_the_selected_source_language_version() {
    let mut entry = Catalog::builtin()
        .entries
        .into_iter()
        .find(|entry| entry.identity.ends_with("declaration-order"))
        .unwrap();
    let features = BTreeSet::from(["task".to_string()]);
    assert!(entry_applies(&entry, Some("v0.1"), &features).unwrap());
    assert!(entry_applies(&entry, Some("v0.2"), &features).unwrap());
    assert!(!entry_applies(&entry, None, &features).unwrap());

    entry.status.kind = "obsolete".into();
    entry.status.since_language_version = Some("v0.2".into());
    entry.status.explanation = Some("covered by the language".into());
    assert!(entry_applies(&entry, Some("v0.1"), &features).unwrap());
    assert!(!entry_applies(&entry, Some("v0.2"), &features).unwrap());
    assert!(!entry_applies(&entry, Some("v0.3"), &features).unwrap());
}

#[test]
fn applicability_tracks_each_selected_context_and_used_feature() {
    let mut external = Catalog::builtin();
    external
        .entries
        .retain(|entry| entry.identity.ends_with("declaration-order"));
    external.entries[0].identity = "org.example best-practice selected task order".into();
    external.entries[0].required_features = vec!["task".into(), "experimental".into()];
    external.entries[0].excluded_features = vec!["legacy".into()];
    let mut engine = LintEngine::builtin();
    engine
        .add_catalog_json(&serde_json::to_string(&external).unwrap())
        .unwrap();
    let source = "use language ( version is v0.1 )\nFirst is Task (queue-size is 1)\nfirst is First\n  start is fn (initial : Nat) -> Completed\n    Completed\n  count : Nat\nuse language ( version is v0.1, features is ( experimental ) )\nSecond is Task (queue-size is 1)\nsecond is Second\n  start is fn (initial : Nat) -> Completed\n    Completed\n  count : Nat\nuse language ( version is v0.1, features is ( experimental, legacy ) )\nThird is Task (queue-size is 1)\nthird is Third\n  start is fn (initial : Nat) -> Completed\n    Completed\n  count : Nat\n";
    let report = engine
        .lint_text(
            source,
            &[LintControl::Enable(
                "org.example best-practice selected task order".into(),
            )],
        )
        .unwrap();
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].line, 12);
}

#[test]
fn deprecated_entries_are_off_by_default_but_remain_selectable() {
    let mut entry = entry();
    entry.status.kind = "deprecated".into();
    entry.status.explanation = Some("superseded guidance".into());
    entry.default_enabled = true;
    assert!(!policy(&entry, &[]).unwrap().enabled);

    let settings = [Override {
        selector: Selector::Identity(entry.identity.clone()),
        enabled: Some(true),
        severity: SeveritySetting::Keep,
        order: 0,
    }];
    assert!(policy(&entry, &settings).unwrap().enabled);
}
