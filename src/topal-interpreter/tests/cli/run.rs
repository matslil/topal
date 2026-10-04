fn run(arguments: &[&str], input: &str) -> std::process::Output {
    let carries_v01_context = arguments.contains(&"--interactive")
        && !arguments.contains(&"--language-version")
        && input.contains("use language (\n  version is v0.1\n)");
    let mut command = Command::new(env!("CARGO_BIN_EXE_topal"));
    command.args(arguments);
    if carries_v01_context {
        command.args(["--language-version", "v0.1"]);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let input = if arguments.contains(&"--interactive") {
        input.replacen("use language (\n  version is v0.1\n)\n", "", 1)
    } else if arguments.contains(&"--version") || arguments.contains(&"--help") {
        input.to_owned()
    } else {
        let input = input
            .strip_prefix("#!")
            .and_then(|after_marker| after_marker.split_once('\n').map(|(_, body)| body))
            .unwrap_or(input);
        format!("use language (\n  version is v0.1\n)\n{input}")
    };
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn run_file(path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_topal"))
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap()
}

#[test]
fn positional_and_explicit_input_call_solve_and_print_only_its_result() {
    let directory =
        std::env::temp_dir().join(format!("topal-explicit-input-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("solver.t");
    let input = directory.join("data.txt");
    std::fs::write(
        &source,
        "use language ( version is v0.1 )\nsolve is fn (input : String) -> Nat\n  entry-count input\n",
    )
    .unwrap();
    std::fs::write(&input, "åb\n").unwrap();

    let explicit = Command::new(env!("CARGO_BIN_EXE_topal"))
        .args(["--input", input.to_str().unwrap(), source.to_str().unwrap()])
        .output()
        .unwrap();
    let positional = Command::new(env!("CARGO_BIN_EXE_topal"))
        .args([source.to_str().unwrap(), input.to_str().unwrap()])
        .output()
        .unwrap();

    std::fs::remove_dir_all(&directory).unwrap();
    for output in [explicit, positional] {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"3\n");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn application_without_input_reports_the_missing_argument() {
    let directory = std::env::temp_dir().join(format!(
        "topal-missing-application-input-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("solver.t");
    std::fs::write(
        &source,
        "use language ( version is v0.1 )\nsolve is fn (input : String) -> Nat\n  entry-count input\n",
    )
    .unwrap();

    let output = run_file(&source);
    std::fs::remove_dir_all(&directory).unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires an input file"), "{stderr}");
    assert!(
        stderr.contains("usage: topal APPLICATION INPUT"),
        "{stderr}"
    );
}

#[test]
fn topal_test_command_discovers_filters_and_reports_individual_programs() {
    let directory = std::env::temp_dir().join(format!("topal-test-runner-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("passes.t"),
        "use language ( version is v0.1 )\nPass is Boolean constraint { value } value = true\npasses : Pass is Pass true\npasses\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("fails.t"),
        "use language ( version is v0.1 )\nPass is Boolean constraint { value } value = true\nfails : Pass is Pass false\nfails\n",
    )
    .unwrap();

    let listed = Command::new(env!("CARGO_BIN_EXE_topal"))
        .args(["test", "--list"])
        .arg(&directory)
        .output()
        .unwrap();
    assert!(listed.status.success());
    let listing = String::from_utf8(listed.stdout).unwrap();
    assert!(listing.contains("fails.t"));
    assert!(listing.contains("passes.t"));

    let passed = Command::new(env!("CARGO_BIN_EXE_topal"))
        .args(["test", "--filter", "passes.t"])
        .arg(&directory)
        .output()
        .unwrap();
    assert!(passed.status.success());
    assert!(
        String::from_utf8(passed.stdout)
            .unwrap()
            .contains("1 Topal tests: 1 passed; 0 failed")
    );

    let failed = Command::new(env!("CARGO_BIN_EXE_topal"))
        .args(["test", "--filter", "fails.t"])
        .arg(&directory)
        .output()
        .unwrap();
    std::fs::remove_dir_all(&directory).unwrap();
    assert!(!failed.status.success());
    assert!(String::from_utf8(failed.stdout).unwrap().contains("FAIL"));
    assert!(
        String::from_utf8(failed.stderr)
            .unwrap()
            .contains("E-CONSTRAINT-REJECTED")
    );
}

#[test]
fn topal_test_command_executes_application_manifests() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = "tests/advent-of-code/2025/day01-part1.t";
    let output = Command::new(env!("CARGO_BIN_EXE_topal"))
        .current_dir(&root)
        .args(["test", "--exact", manifest, manifest])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("1 Topal tests: 1 passed; 0 failed")
    );
}

#[test]
fn every_interpreter_example_is_an_executable_script() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language");
    let mut examples = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "t"))
        .collect::<Vec<_>>();
    examples.sort();
    assert_eq!(examples.len(), 306);
    for example in examples {
        let output = run_file(&example);
        assert!(
            output.status.success(),
            "{} failed:\n{}",
            example.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn declared_standard_library_file_executes_directly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_topal"))
        .args(["--library-root", root.join("library").to_str().unwrap()])
        .arg(root.join("examples/data-transfer/rest-controller.t"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn every_interpreter_example_documents_its_feature() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language");
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|extension| extension == "t") {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source
                    .lines()
                    .skip(1)
                    .take(8)
                    .any(|line| line.starts_with('#')),
                "{} lacks a feature comment",
                path.display()
            );
        }
    }
}

#[test]
fn test_mode_preserves_script_standard_output() {
    let source = "value is 40 + 2\nvalue\n";
    assert_eq!(run(&[], source).stdout, run(&["--test"], source).stdout);
}

#[test]
fn interactive_mode_preserves_complete_unit_results() {
    let source = "40 + 2\n";
    assert_eq!(
        run(&[], source).stdout,
        run(&["--interactive"], source).stdout
    );
}

#[test]
fn unicode_diagnostics_preserve_source_columns() {
    let output = run(&[], "name is \"å\"\nnamé\n");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("5 | namé"));
    assert!(diagnostic.contains('^'));
}

#[test]
fn version_output_is_reproducible() {
    let first = run(&["--version"], "");
    let second = run(&["--version"], "");
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert!(
        String::from_utf8(first.stdout)
            .unwrap()
            .contains("highest language v0.2")
    );
}

#[test]
fn script_mode_requires_an_explicit_source_language_version() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_topal"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"42\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("E-MISSING-LANGUAGE-VERSION"));
    assert!(stderr.contains("use language"));
}

#[test]
fn interactive_language_version_is_explicit_or_defaults_to_highest_supported() {
    assert_eq!(
        run(&["--interactive"], "40 + 2\n").stdout,
        run(&["--interactive", "--language-version", "v0.1"], "40 + 2\n").stdout
    );
    let unsupported = run(&["--interactive", "--language-version", "v9.0"], "");
    assert!(!unsupported.status.success());
    assert!(
        String::from_utf8(unsupported.stderr)
            .unwrap()
            .contains("highest language version")
    );
}

#[test]
fn test_mode_records_discard_after_its_initializer() {
    let output = run(&["--test"], "_ is 20 + 22\n7\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"7\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"profiles\":[\"debugging\",\"testing\"]"));
    let initializer = trace.find("root.+(Int,Int)").unwrap();
    let discard = trace.find("\"event\":\"binding.discarded\"").unwrap();
    assert!(initializer < discard);
    assert!(trace.contains("\"rule\":\"TOPAL-SYN-BIND-001\""));
}

#[test]
fn test_mode_records_labeled_record_construction() {
    let output = run(&["--test"], "(name is \"Ada\", active is true)\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"(name is \"Ada\", active is true)\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"product.record\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-PRODUCT-001\""));
    assert!(trace.contains("\"detail\":\"fields=2\""));
}

#[test]
fn mixed_product_fields_suggest_explicit_nesting() {
    let output = run(&[], "(1, name is \"Ada\")\n");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("E-MIXED-PRODUCT-FIELDS"));
    assert!(diagnostic.contains("nest a tuple in a labeled field"));
}

#[test]
fn test_mode_records_record_field_selection() {
    let output = run(&["--test"], "(name is \"Ada\", active is true) name\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"\"Ada\"\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"record.field.selected\""));
    assert!(trace.contains("\"detail\":\"name\""));
}

#[test]
fn test_mode_records_plain_string_concatenation() {
    let output = run(&["--test"], "\"Hello, \" concat \"Topal\"\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"\"Hello, Topal\"\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    let selection = trace.find("root.concat(String,String)").unwrap();
    let evaluation = trace.find("TOPAL-STRING-CONCAT-001").unwrap();
    assert!(selection < evaluation);
}

#[test]
fn test_mode_records_adjacent_literal_composition() {
    let output = run(&["--test"], "\"Hello, \" \"Topal\"\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"\"Hello, Topal\"\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("string.literals.composed"));
    assert!(trace.contains("TOPAL-STRING-LITERAL-COMPOSE-001"));
}

#[test]
fn test_mode_records_empty_string_construction() {
    let output = run(&["--test"], "empty String\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"\"\"\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root.empty(String)\""));
    assert!(trace.contains("\"rule\":\"TOPAL-STRING-EMPTY-001\""));
}

#[test]
fn every_mode_tests_string_emptiness() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "empty? (empty String)\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"true\n");
    }

    let output = run(&["--test"], "empty? \"Topal\"\n");
    assert_eq!(output.stdout, b"false\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("root.empty?(String)"));
    assert!(trace.contains("TOPAL-STRING-EMPTY-PREDICATE-001"));
}

#[test]
fn test_mode_records_string_character_count() {
    let output = run(&["--test"], "character-count \"a\u{301}👩‍🔬🇸🇪\"\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"3\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    let selection = trace.find("root.character-count(String)").unwrap();
    let evaluation = trace.find("TOPAL-STRING-CHARACTER-COUNT-001").unwrap();
    assert!(selection < evaluation);
    assert!(trace.contains("characters=3"));
}

#[test]
fn every_mode_counts_string_sequence_entries() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "entry-count \"a\u{301}👩‍🔬🇸🇪\"\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"3\n");
    }

    let output = run(&["--test"], "entry-count \"👩‍🔬\"\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("root.entry-count(String)"));
    assert!(trace.contains("TOPAL-STRING-ENTRY-COUNT-001"));
    assert!(trace.contains("characters=1"));
}

#[test]
fn every_mode_counts_prospective_utf8_bytes() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "\"e\u{301}👩‍🔬\" byte-count Utf8\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"14\n");
    }

    let output = run(&["--test"], "\"é\" byte-count Utf8\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("root.byte-count(String,Utf8)"));
    assert!(trace.contains("TOPAL-STRING-UTF8-BYTE-COUNT-001"));
    assert!(trace.contains("bytes=2"));
}

#[test]
fn every_mode_normalizes_strings_to_nfc_explicitly() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "\"e\u{301}\" normalize NFC\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, "\"é\"\n".as_bytes());
    }

    let output = run(&["--test"], "\"é\" normalize NFC\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("root.normalize(String,NFC)"));
    assert!(trace.contains("TOPAL-STRING-NORMALIZE-NFC-001"));
    assert!(trace.contains("changed=false"));
}

#[test]
fn every_mode_normalizes_strings_to_nfd_explicitly() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "\"é\" normalize NFD\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, "\"e\u{301}\"\n".as_bytes());
    }

    let output = run(&["--test"], "\"e\u{301}\" normalize NFD\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("root.normalize(String,NFD)"));
    assert!(trace.contains("TOPAL-STRING-NORMALIZE-NFD-001"));
    assert!(trace.contains("changed=false"));
}

#[test]
fn version_exposes_the_language_context_unicode_version() {
    let output = run(&["--version"], "");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "topal 0.1.0 (highest language v0.2; Unicode 17.0.0)\n"
    );
}

#[test]
fn script_rejects_non_nfc_identifier_without_rewriting_it() {
    let output = run(&[], "e\u{301} is 1\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NON-NFC-TOKEN")
    );
}

#[test]
fn script_rejects_non_nfc_literal_tag() {
    let output = run(&[], "tag\u{301}\"value\"tag\u{301}\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NON-NFC-TOKEN")
    );
}

#[test]
fn script_preserves_non_nfc_string_contents() {
    let output = run(&[], "\"e\u{301}\"\n");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "\"e\u{301}\"\n");
}

#[test]
fn test_trace_records_the_pinned_unicode_context() {
    let output = run(&["--test"], "1\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"context.selected\""));
    assert!(trace.contains("\"rule\":\"TOPAL-SYN-UNICODE-001\""));
    assert!(trace.contains("\"detail\":\"design-0;Unicode=17.0.0\""));
}

#[test]
fn script_mode_is_default() {
    let output = run(&[], "123456789012345678901234567890\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"123456789012345678901234567890\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn every_mode_evaluates_boolean_literals() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "true\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"true\n");
    }

    let output = run(&[], "false\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"false\n");
}

#[test]
fn boolean_literal_spellings_cannot_be_rebound() {
    let output = run(&[], "true is 1\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-RESERVED-BOOLEAN-LITERAL")
    );
}

#[test]
fn test_trace_explains_boolean_literal_construction() {
    let output = run(&["--test"], "false\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"token.boolean\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-BOOLEAN-001\""));
    assert!(trace.contains("\"detail\":\"false\""));
    assert!(trace.contains("\"detail\":\"Boolean\""));
}

#[test]
fn every_mode_evaluates_fundamental_equality() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(
            arguments,
            "(1, \"same\", true, ()) = (1, \"same\", true, ())\n",
        );
        assert!(output.status.success());
        assert_eq!(output.stdout, b"true\n");
    }

    let output = run(&[], "\"e\u{301}\" = \"é\"\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"false\n");
}

#[test]
fn equality_uses_canonical_exact_conversion() {
    let output = run(&["--test"], "1 = 1.0\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"true\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    let conversion = trace.find("\"event\":\"conversion.applied\"").unwrap();
    let selection = trace.find("\"event\":\"operator.selected\"").unwrap();
    assert!(conversion < selection);
    assert!(trace.contains("\"event\":\"evaluation.equal\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-EQUALITY-001\""));
}

#[test]
fn equality_requires_a_shared_operation() {
    let output = run(&[], "true = 1\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NO-APPLICABLE-OVERLOAD")
    );

    let output = run(&[], "(1,) = (1, 2)\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NO-APPLICABLE-OVERLOAD")
    );
}

#[test]
fn every_mode_derives_record_equality() {
    let source = "(name is \"Ada\", score is 1) = (score is 1.0, name is \"Ada\")\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert_eq!(output.stdout, b"true\n");
    }

    let output = run(&[], "(name is \"Ada\") = (alias is \"Ada\")\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NO-APPLICABLE-OVERLOAD")
    );
}

#[test]
fn every_mode_derives_nominal_sum_equality() {
    // TOPAL-INTP-SUBSET-256, TOPAL-TYPE-SUM-EQUALITY-001,
    // TOPAL-TYPE-EQUALITY-001
    let source = include_str!("../../../../examples/language/sum-equality.t");
    let expected = "(true, false, true, false, true, true, true, false, true, false, true)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    let traced = run(&["--test"], source);
    let trace = String::from_utf8(traced.stderr).unwrap();
    assert_eq!(trace.matches("\"event\":\"equality.sum\"").count(), 12);
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-SUM-EQUALITY-001\""));

    for unsupported in [
        "use language (version is v0.1)\nHolder is Union\n  Blank\n  Window : Range Int\n\nleft : Holder is Blank\nright : Holder is Blank\nleft = right\n",
        "use language (version is v0.1)\nLeft is Union\n  LeftEmpty\n\nRight is Union\n  RightEmpty\n\nleft : Left is LeftEmpty\nright : Right is RightEmpty\nleft = right\n",
    ] {
        let output = run(&[], unsupported);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("E-NO-APPLICABLE-OVERLOAD"));
    }
}

#[test]
fn every_mode_associates_packaged_fields_by_label() {
    // TOPAL-INTP-SUBSET-257, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let source =
        include_str!("../../../../examples/language/packaged-function-association-order.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("(42, 42, 42)"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_applies_compound_packaged_operands() {
    // TOPAL-INTP-SUBSET-258, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let source =
        include_str!("../../../../examples/language/compound-packaged-function-operands.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("(42, 42, 42, 42, 42)"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_applies_structured_packaged_fields() {
    // TOPAL-INTP-SUBSET-259, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let source =
        include_str!("../../../../examples/language/structured-packaged-function-fields.t");
    let expected = "(((20, 22), \"Ada\"), ((20, 22), \"Ada\", true), ((21, 21), \"default\", false), ((20, 22), \"Grace\", true))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_applies_sum_packaged_fields() {
    // TOPAL-INTP-SUBSET-260, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let source = include_str!("../../../../examples/language/sum-packaged-function-fields.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("(42, 42, 42, 42)"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_applies_function_packaged_fields() {
    // TOPAL-INTP-SUBSET-261, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let source = include_str!("../../../../examples/language/function-packaged-fields.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("(42, 42, 42, 42)"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_applies_function_aggregate_packaged_fields() {
    // TOPAL-INTP-SUBSET-262, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let source = include_str!("../../../../examples/language/function-aggregate-packaged-fields.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("(42, 42, 42)"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_applies_container_packaged_fields() {
    // TOPAL-INTP-SUBSET-263, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let source = include_str!("../../../../examples/language/container-packaged-fields.t");
    let expected = "((Entry ( 20, Entry ( 22, Empty ) ), Some 42, 42, 40 ..= 42), (Entry ( 20, Entry ( 22, Empty ) ), None, 42, 40 ..= 42))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_applies_collection_packaged_fields() {
    // TOPAL-INTP-SUBSET-264, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let source = include_str!("../../../../examples/language/collection-packaged-fields.t");
    let expected = "((Array (2, 1, 2), Set (2, 1), Bag ((2, 2), (1, 1)), Map ((\"Ada\", 11), (\"Lin\", 8))), (Array (2, 1, 2), Set (2, 1), Bag ((2, 2), (1, 1)), Map ((\"Ada\", 11), (\"Lin\", 8))))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_applies_scope_packaged_fields() {
    // TOPAL-INTP-SUBSET-265, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-NAMESPACE-FUNCTION-BOUNDARY-001, TOPAL-TYPE-CALL-001
    let source = include_str!("../../../../examples/language/scope-packaged-fields.t");
    let expected = "((42, 42), (42, 42), (42, 42))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_resolves_live_root_data_inside_functions() {
    // TOPAL-INTP-SUBSET-266, TOPAL-NAMESPACE-ROOT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-001
    let source = include_str!("../../../../examples/language/function-root-data.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("(42, 0, \"ready\")"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_forwards_live_root_data_between_functions() {
    // TOPAL-INTP-SUBSET-267, TOPAL-NAMESPACE-ROOT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001
    let source = include_str!("../../../../examples/language/function-root-data-forwarding.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("(42, 0, \"ready\")"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_evaluates_derived_inequality() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "(1, true) != (1.0, false)\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"true\n");
    }

    let output = run(&["--test"], "1 != 1.0\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"false\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root.!=(Equality,Equality)\""));
    assert!(trace.contains("\"event\":\"evaluation.equal\""));
}

#[test]
fn inequality_preserves_equality_applicability() {
    let output = run(&[], "false != 0\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NO-APPLICABLE-OVERLOAD")
    );
}

#[test]
fn every_mode_evaluates_exact_ordering_predicates() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "-2 < 1\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"true\n");
    }
    for (expression, expected) in [
        ("2 > 2", "false\n"),
        ("2 <= 2", "true\n"),
        ("3 >= 4", "false\n"),
    ] {
        let output = run(&[], &format!("{expression}\n"));
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
}

#[test]
fn exact_ordering_traces_conversion_and_three_way_decision() {
    let output = run(&["--test"], "1 < 1.5\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"true\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    let conversion = trace.find("\"event\":\"conversion.applied\"").unwrap();
    let comparison = trace.find("\"event\":\"comparison.result\"").unwrap();
    let selection = trace.find("\"event\":\"operator.selected\"").unwrap();
    assert!(conversion < selection);
    assert!(selection < comparison);
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-COMPARE-001\""));
    assert!(trace.contains("\"detail\":\"Less\""));
}

#[test]
fn exact_ordering_rejects_values_without_total_order() {
    let output = run(&[], "true < false\n");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-NO-APPLICABLE-OVERLOAD]"));
    assert!(diagnostic.contains("= help: use operands supported by one overload"));
}

#[test]
fn every_mode_derives_lexicographic_tuple_ordering() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "(1, (2, 3)) < (1.0, (2, 4))\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"true\n");
    }

    let output = run(&[], "(2, true) > (1, false)\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"true\n");
}

#[test]
fn tuple_ordering_requires_comparable_fields_until_decided() {
    let unsupported_field = run(&[], "(1, true) < (1, false)\n");
    assert!(!unsupported_field.status.success());
    assert!(
        String::from_utf8(unsupported_field.stderr)
            .unwrap()
            .contains("E-NO-APPLICABLE-OVERLOAD")
    );

    let different_arity = run(&[], "(1,) < (1, 2)\n");
    assert!(!different_arity.status.success());
    assert!(
        String::from_utf8(different_arity.stderr)
            .unwrap()
            .contains("E-NO-APPLICABLE-OVERLOAD")
    );
}

#[test]
fn tuple_ordering_trace_names_the_derived_rule() {
    let output = run(&["--test"], "(1, 2) >= (1, 2)\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"comparison.result\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-ORDERING-001\""));
    assert!(trace.contains("\"detail\":\"Equal\""));
}

#[test]
fn every_mode_evaluates_the_unit_product() {
    let script = run(&[], "()\n");
    assert!(script.status.success());
    assert_eq!(script.stdout, b"()\n");

    let interactive = run(&["--interactive"], "()\n");
    assert!(interactive.status.success());
    assert_eq!(interactive.stdout, b"()\n");

    let test = run(&["--test"], "()\n");
    assert!(test.status.success());
    assert_eq!(test.stdout, b"()\n");
    let trace = String::from_utf8(test.stderr).unwrap();
    assert!(trace.contains("\"event\":\"product.unit\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-PRODUCT-001\""));
    assert!(trace.contains("\"detail\":\"Tuple()\""));
}

#[test]
fn every_mode_evaluates_positional_products() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "(1, \"two\", ())\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"(1, \"two\", ())\n");
    }

    let grouped = run(&[], "(1)\n");
    assert!(grouped.status.success());
    assert_eq!(grouped.stdout, b"1\n");

    let singleton = run(&[], "(1,)\n");
    assert!(singleton.status.success());
    assert_eq!(singleton.stdout, b"(1,)\n");

    let traced = run(&["--test"], "(1, 2)\n");
    let trace = String::from_utf8(traced.stderr).unwrap();
    assert!(trace.contains("\"event\":\"product.tuple\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-PRODUCT-001\""));
    assert!(trace.contains("\"detail\":\"fields=2\""));
}

#[test]
fn every_mode_evaluates_multiline_products() {
    let source = "(\n1,\n(\n2\n),\n3\n)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert_eq!(output.stdout, b"(1, 2, 3)\n");
    }
}

#[test]
fn script_mode_ignores_hashbang_launcher_line() {
    let output = run(&[], "#!/usr/bin/env topal\n42\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"42\n");
}

#[test]
fn interactive_mode_evaluates_each_input() {
    let output = run(&["--interactive"], "1\n2\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"1\n2\n");
}

#[test]
fn interactive_mode_recovers_after_diagnostic() {
    let output = run(&["--interactive"], "1 \u{200b} 2\n2\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"2\n");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNKNOWN-TOKEN")
    );
}

#[test]
fn interactive_mode_preserves_bindings() {
    let output = run(&["--interactive"], "answer is 42\nanswer\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"()\n42\n");
}

#[test]
fn test_mode_emits_stable_decisions() {
    let output = run(&["--test"], "42\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"42\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"schema\":\"topal.test-trace/1\""));
    assert!(trace.contains("\"event\":\"token.integer\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-LITERAL-001\""));
    assert!(trace.contains("\"event\":\"evaluation.result\""));
    assert!(trace.contains("\"detail\":\"Int\""));
}

#[test]
fn unsupported_syntax_is_explicit() {
    let output = run(&[], "1 \u{200b} 2\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNKNOWN-TOKEN")
    );
}

#[test]
fn script_diagnostic_shows_source_marker_and_help() {
    let output = run(&[], "value + \u{200b}\n");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-UNKNOWN-TOKEN]"));
    assert!(diagnostic.contains(" --> <stdin>:4:9"));
    assert!(diagnostic.contains("4 | value + \u{200b}"));
    assert!(diagnostic.contains("  |         ^"));
    assert!(diagnostic.contains("= help: remove this character"));
}

#[test]
fn interactive_diagnostic_uses_an_interactive_source_label() {
    let output = run(&["--interactive"], "missing\n");
    assert!(output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains(" --> <interactive>:1:1"));
    assert!(diagnostic.contains("= help: declare this name earlier"));
}

#[test]
fn unbound_name_diagnostic_suggests_a_close_visible_binding() {
    let output = run(&[], "answer is 42\nanwser\n");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-UNBOUND-NAME]"));
    assert!(diagnostic.contains("5 | anwser"));
    assert!(diagnostic.contains("= help: did you mean `answer`?"));
}

#[test]
fn diagnostics_suggest_implemented_root_operations() {
    let output = run(&[], "charcter-count \"Topal\"\n");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-UNBOUND-NAME]"));
    assert!(diagnostic.contains("= help: did you mean `character-count`?"));

    let output = run(&[], "\"a\" concatenate \"b\"\n");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-UNSUPPORTED-APPLICATION]"));
    assert!(diagnostic.contains("= help: did you mean `concat`?"));
}

#[test]
fn script_executes_bindings_in_source_order() {
    let output = run(&[], "answer is 42\nanswer\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"42\n");
}

#[test]
fn every_mode_declares_and_calls_static_nullary_functions() {
    let source = "answer is fn static () -> Int\n  40 + 2\nanswer ()\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n42\n");
        } else {
            assert_eq!(output.stdout, b"42\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let declared = trace.find("function.declared").unwrap();
    let entered = trace.find("function.entry").unwrap();
    let body = trace.find("root.+(Int,Int)").unwrap();
    let returned = trace.find("function.exit").unwrap();
    assert!(declared < entered && entered < body && body < returned);
}

#[test]
fn static_function_result_classifier_is_checked() {
    let output = run(&[], "wrong is fn static () -> Int\n  \"text\"\nwrong ()\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-FUNCTION-RESULT-TYPE")
    );
}

#[test]
fn every_mode_declares_and_calls_static_unary_functions() {
    let source = "increment is fn static (input : Int) -> Int\n  input + 1\nincrement 41\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n42\n");
        } else {
            assert_eq!(output.stdout, b"42\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let bound = trace.find("function.argument.bound").unwrap();
    let selected = trace.find("function.selected").unwrap();
    let entered = trace.find("function.entry").unwrap();
    let body = trace.find("root.+(Int,Int)").unwrap();
    let returned = trace.find("function.exit").unwrap();
    assert!(bound < selected && selected < entered && entered < body && body < returned);
}

#[test]
fn static_unary_function_checks_argument_classifier() {
    let output = run(
        &[],
        "increment is fn static (input : Int) -> Int\n  input + 1\nincrement \"one\"\n",
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-FUNCTION-ARGUMENT-TYPE")
    );
}

#[test]
fn every_mode_calls_static_product_functions() {
    let source = "add is fn static (left : Int, right : Int) -> Int\n  left + right\n20 add 22\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n42\n");
        } else {
            assert_eq!(output.stdout, b"42\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let left = trace.find("\"detail\":\"left\"").unwrap();
    let right = trace.find("\"detail\":\"right\"").unwrap();
    let entered = trace.find("function.entry").unwrap();
    assert!(left < right && right < entered);
}

#[test]
fn static_product_function_checks_shape_and_arity() {
    let declaration = "add is fn static (left : Int, right : Int) -> Int\n  left + right\n";
    let shape = run(&[], &format!("{declaration}add 1\n"));
    assert!(!shape.status.success());
    assert!(
        String::from_utf8(shape.stderr)
            .unwrap()
            .contains("E-FUNCTION-ARGUMENT-SHAPE")
    );

    let arity = run(&[], &format!("{declaration}add (1, 2, 3)\n"));
    assert!(!arity.status.success());
    assert!(
        String::from_utf8(arity.stderr)
            .unwrap()
            .contains("E-FUNCTION-ARGUMENT-ARITY")
    );
}

#[test]
fn every_mode_executes_multi_statement_function_bodies() {
    let source = "answer is fn static () -> Int\n  local is 40 + 2\n  local\nanswer ()\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n42\n");
        } else {
            assert_eq!(output.stdout, b"42\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let entered = trace.find("function.entry").unwrap();
    let created = trace.find("binding.bind").unwrap();
    let resolved = trace.find("binding.resolved").unwrap();
    let returned = trace.find("function.exit").unwrap();
    assert!(entered < created && created < resolved && resolved < returned);
}

#[test]
fn every_mode_returns_early_from_functions() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let source = if arguments == ["--interactive"] {
            "answer is fn static () -> Int\n  return 40 + 2\nanswer ()\n"
        } else {
            "answer is fn static () -> Int\n  return 40 + 2\n  missing\nanswer ()\n"
        };
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n42\n");
        } else {
            assert_eq!(output.stdout, b"42\n");
        }
    }

    let output = run(
        &["--test"],
        "answer is fn static () -> Int\n  return 40 + 2\n  missing\nanswer ()\n",
    );
    let trace = String::from_utf8(output.stderr).unwrap();
    let explicit = trace.find("function.return.explicit").unwrap();
    let returned = trace.find("function.exit").unwrap();
    assert!(explicit < returned);
    assert!(!trace.contains("missing"));
}

#[test]
fn return_outside_a_function_is_rejected() {
    let output = run(&[], "return 42\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-RETURN-OUTSIDE-FUNCTION")
    );
}

#[test]
fn every_mode_returns_from_an_unconditional_lexical_block() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-001
    let source = include_str!("../../../../examples/language/function-return-from-block.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"42\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert!(trace.contains("function.return.explicit"));
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_a_block_inside_an_explicit_return() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERAND-001
    let source = include_str!("../../../../examples/language/function-return-block-operand.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"42\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 1);
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_symbolic_operator_operands() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERATOR-001
    let source = include_str!("../../../../examples/language/function-return-operator-operand.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(42, 43)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 2);
            assert_eq!(
                trace
                    .lines()
                    .filter(|line| {
                        line.contains("function.entry") && line.contains("\"detail\":\"preceding\"")
                    })
                    .count(),
                1
            );
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("missing"));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_product_fields() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-PRODUCT-001
    let source = include_str!("../../../../examples/language/function-return-product-field.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(42, 43)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 2);
            assert_eq!(
                trace
                    .lines()
                    .filter(|line| {
                        line.contains("function.entry") && line.contains("\"detail\":\"preceding\"")
                    })
                    .count(),
                4
            );
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("missing"));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_named_call_arguments() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-CALL-001
    let source = include_str!("../../../../examples/language/function-return-call-argument.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(42, 43, 44)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 3);
            assert_eq!(
                trace
                    .lines()
                    .filter(|line| {
                        line.contains("function.entry") && line.contains("\"detail\":\"preceding\"")
                    })
                    .count(),
                1
            );
            assert!(!trace.lines().any(|line| {
                line.contains("function.entry")
                    && (line.contains("\"detail\":\"combine\"")
                        || line.contains("\"detail\":\"identity\""))
            }));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("missing"));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_optional_constructor_arguments() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-001
    let source =
        include_str!("../../../../examples/language/function-return-optional-constructor.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"42\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 1);
            assert!(!trace.contains("optional.some.constructed"));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_strict_unary_constructor_arguments() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241, TOPAL-TYPE-UNION-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001
    let source = include_str!("../../../../examples/language/function-return-unary-constructor.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(42, 43, 44, 45, 46)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 5);
            for event in [
                "string.from-character",
                "numeric.int.constructed",
                "numeric.nat.constructed",
                "numeric.rational.constructed",
                "union.constructed",
            ] {
                assert!(!trace.contains(event), "{event}: {trace}");
            }
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_positional_variant_arguments() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241, TOPAL-TYPE-VARIANT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-VARIANT-001
    let source =
        include_str!("../../../../examples/language/function-return-variant-constructor.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"42\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 1);
            assert!(!trace.contains("variant.constructed"));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_character_constructor_arguments() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-STRING-CHARACTER-CLASSIFIER-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CHARACTER-001
    let source =
        include_str!("../../../../examples/language/function-return-character-constructor.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"42\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 1);
            assert!(!trace.contains("constraint.validated"));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}
