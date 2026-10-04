#[test]
fn records_character_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-283, TOPAL-COMPILER-LIST-CHARACTER-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-character-values.debug"),
            &language_example("list-character-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Character",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Character",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=2",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_nat_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-284, TOPAL-COMPILER-LIST-NAT-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-nat-values.debug"),
            &language_example("list-nat-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Nat",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Nat",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_rational_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-285, TOPAL-COMPILER-LIST-RATIONAL-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-rational-values.debug"),
            &language_example("list-rational-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Rational",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Rational",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_effect_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-286, TOPAL-COMPILER-LIST-EFFECT-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-effect-values.debug"),
            &language_example("list-effect-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Effect",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Effect",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=2",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_comparison_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-287, TOPAL-COMPILER-LIST-COMPARISON-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-comparison-values.debug"),
            &language_example("list-comparison-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Comparison",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Comparison",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_error_code_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-288, TOPAL-COMPILER-LIST-ERROR-CODE-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-error-code-values.debug"),
            &language_example("list-error-code-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] ErrorCode",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] ErrorCode",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=4",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_unit_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-289, TOPAL-COMPILER-LIST-UNIT-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-unit-values.debug"),
            &language_example("list-unit-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Unit",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Unit",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_completed_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-290, TOPAL-COMPILER-LIST-COMPLETED-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-completed-values.debug"),
            &language_example("list-completed-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Completed",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Completed",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_type_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-291, TOPAL-COMPILER-LIST-TYPE-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-type-values.debug"),
            &language_example("list-type-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Type",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Type",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=7",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_nominal_enum_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-292, TOPAL-COMPILER-LIST-ENUM-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-enum-values.debug"),
            &language_example("list-enum-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Color",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Color",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_nominal_modular_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-293, TOPAL-COMPILER-LIST-MODULAR-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-modular-values.debug"),
            &language_example("list-modular-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] ByteCounter",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] ByteCounter",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_optional_int_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-294, TOPAL-COMPILER-LIST-OPTIONAL-INT-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-optional-int-values.debug"),
            &language_example("list-optional-int-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Optional Int",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Optional Int",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_optional_rational_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-295, TOPAL-COMPILER-LIST-OPTIONAL-RATIONAL-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-optional-rational-values.debug"),
            &language_example("list-optional-rational-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Optional Rational",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Optional Rational",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_optional_string_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-296, TOPAL-COMPILER-LIST-OPTIONAL-STRING-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-optional-string-values.debug"),
            &language_example("list-optional-string-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Optional String",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] Optional String",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_int_pair_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-297, TOPAL-COMPILER-LIST-INT-PAIR-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-int-pair-values.debug"),
            &language_example("list-int-pair-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] (Int, Int)",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] (Int, Int)",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_int_string_pair_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-298, TOPAL-COMPILER-LIST-INT-STRING-PAIR-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-int-string-pair-values.debug"),
            &language_example("list-int-string-pair-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] (Int, String)",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] (Int, String)",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_string_int_pair_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-299, TOPAL-COMPILER-LIST-STRING-INT-PAIR-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-string-int-pair-values.debug"),
            &language_example("list-string-int-pair-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] (String, Int)",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] (String, Int)",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_string_pair_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-300, TOPAL-COMPILER-LIST-STRING-PAIR-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-string-pair-values.debug"),
            &language_example("list-string-pair-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] (String, String)",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] (String, String)",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=3",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_lexical_block_return_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-from-block.debug"),
            &language_example("function-return-from-block.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "binding.bind [TOPAL-SYN-BIND-001] adjusted",
        "function.return.explicit [TOPAL-FUNCTION-RETURN-001] Int",
        "function.exit [TOPAL-FUNCTION-ORDINARY-001] answer",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_return_operand_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERAND-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-block-operand.debug"),
            &language_example("function-return-block-operand.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_operator_operand_block_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERATOR-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-operator-operand.debug"),
            &language_example("function-return-operator-operand.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 2);
    assert_eq!(
        stdout
            .lines()
            .filter(|line| line.contains("function.entry") && line.contains("preceding"))
            .count(),
        1
    );
    for function in ["right-exit", "left-exit"] {
        assert!(
            stdout.contains(&format!(
                "function.exit [TOPAL-FUNCTION-ORDINARY-001] {function}"
            )),
            "{stdout}"
        );
    }
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("missing"));
}

#[test]
fn records_product_field_block_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-PRODUCT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-product-field.debug"),
            &language_example("function-return-product-field.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 2);
    assert_eq!(
        stdout
            .lines()
            .filter(|line| line.contains("function.entry") && line.contains("preceding"))
            .count(),
        4
    );
    for function in ["tuple-exit", "record-exit"] {
        assert!(
            stdout.contains(&format!(
                "function.exit [TOPAL-FUNCTION-ORDINARY-001] {function}"
            )),
            "{stdout}"
        );
    }
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("missing"));
}

#[test]
fn records_named_call_argument_block_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-call-argument.debug"),
            &language_example("function-return-call-argument.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 3);
    assert_eq!(
        stdout
            .lines()
            .filter(|line| line.contains("function.entry") && line.contains("preceding"))
            .count(),
        1
    );
    for function in ["right-exit", "left-exit", "unary-exit"] {
        assert!(
            stdout.contains(&format!(
                "function.exit [TOPAL-FUNCTION-ORDINARY-001] {function}"
            )),
            "{stdout}"
        );
    }
    assert!(!stdout.lines().any(|line| {
        line.contains("function.entry") && (line.contains("combine") || line.contains("identity"))
    }));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("missing"));
}

#[test]
fn records_optional_constructor_argument_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-optional-constructor.debug"),
            &language_example("function-return-optional-constructor.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("optional.some.constructed"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_strict_unary_constructor_argument_block_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241, TOPAL-TYPE-UNION-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-unary-constructor.debug"),
            &language_example("function-return-unary-constructor.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 5);
    for function in [
        "string-exit",
        "int-exit",
        "nat-exit",
        "rational-exit",
        "union-exit",
    ] {
        assert!(
            stdout.contains(&format!(
                "function.exit [TOPAL-FUNCTION-ORDINARY-001] {function}"
            )),
            "{stdout}"
        );
    }
    for event in [
        "string.from-character",
        "numeric.int.constructed",
        "numeric.nat.constructed",
        "numeric.rational.constructed",
        "union.constructed",
    ] {
        assert!(!stdout.contains(event), "{event}: {stdout}");
    }
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_positional_variant_argument_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241, TOPAL-TYPE-VARIANT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-VARIANT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-variant-constructor.debug"),
            &language_example("function-return-variant-constructor.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("variant.constructed"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_character_constructor_argument_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-STRING-CHARACTER-CLASSIFIER-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CHARACTER-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-character-constructor.debug"),
            &language_example("function-return-character-constructor.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("constraint.validated"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_named_constraint_argument_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRAINT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-constraint-constructor.debug"),
            &language_example("function-return-constraint-constructor.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("constraint.validated"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_named_modular_argument_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-NUM-MODULAR-CONSTRUCT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-modular-constructor.debug"),
            &language_example("function-return-modular-constructor.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("numeric.modular.constructed"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_modular_reduction_operand_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-NUM-MODULAR-REDUCE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-modular-reduction.debug"),
            &language_example("function-return-modular-reduction.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("numeric.modular.reduced"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_list_collect_source_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COLLECTION-COLLECT-LIST-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COLLECT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-list-collect.debug"),
            &language_example("function-return-list-collect.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("list.collected"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_unordered_collect_source_block_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-SET-COLLECT-001, TOPAL-BAG-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-unordered-collect.debug"),
            &language_example("function-return-unordered-collect.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 2);
    for function in ["set-exit", "bag-exit"] {
        assert!(
            stdout.contains(&format!(
                "function.exit [TOPAL-FUNCTION-ORDINARY-001] {function}"
            )),
            "{stdout}"
        );
    }
    assert!(!stdout.contains("set.collected"));
    assert!(!stdout.contains("bag.collected"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_map_collect_source_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-MAP-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-map-collect.debug"),
            &language_example("function-return-map-collect.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("map.collected"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_infix_collect_source_block_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-ARRAY-COLLECT-001, TOPAL-COLLECTION-COLLECT-STRING-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-INFIX-COLLECT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-infix-collect.debug"),
            &language_example("function-return-infix-collect.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 2);
    for function in ["array-exit", "string-exit"] {
        assert!(
            stdout.contains(&format!(
                "function.exit [TOPAL-FUNCTION-ORDINARY-001] {function}"
            )),
            "{stdout}"
        );
    }
    assert!(!stdout.contains("array.collected"));
    assert!(!stdout.contains("string.collected"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("binding.bound [TOPAL-SYN-BIND-001] abandoned"));
}

#[test]
fn records_boolean_decision_subject_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-DECISION-SUBJECT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-decision-subject.debug"),
            &language_example("function-return-decision-subject.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("decision.rule"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_exhaustive_boolean_decision_subject_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-EXHAUSTIVE-BOOLEAN-SUBJECT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-exhaustive-boolean-subject.debug"),
            &language_example("function-return-exhaustive-boolean-subject.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("decision.rule"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_comparison_decision_subject_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-COMPARISON-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-DECISION-SUBJECT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-comparison-decision-subject.debug"),
            &language_example("function-return-comparison-decision-subject.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("decision.rule"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_fallback_decision_subject_block_exit_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-FALLBACK-DECISION-SUBJECT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-fallback-decision-subject.debug"),
            &language_example("function-return-fallback-decision-subject.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 1);
    assert!(
        stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] answer"),
        "{stdout}"
    );
    assert!(!stdout.contains("decision.rule"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_complete_decision_subject_block_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-OPTIONAL-001, TOPAL-DECISION-RESULT-001,
    // TOPAL-DECISION-LIST-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPLETE-DECISION-SUBJECT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-complete-decision-subject.debug"),
            &language_example("function-return-complete-decision-subject.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 3);
    for function in ["optional-exit", "result-exit", "list-exit"] {
        assert!(
            stdout.contains(&format!(
                "function.exit [TOPAL-FUNCTION-ORDINARY-001] {function}"
            )),
            "{stdout}"
        );
    }
    assert!(!stdout.contains("decision.rule"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_complete_boolean_decision_action_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-BOOLEAN-DECISION-ACTIONS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-boolean-decision-actions.debug"),
            &language_example("function-return-boolean-decision-actions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 2);
    assert_eq!(stdout.matches("decision.rule.selected").count(), 2);
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001] choose")
            .count(),
        2
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_complete_comparison_value_decision_action_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-ENUM-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-VALUE-DECISION-ACTIONS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-comparison-value-decision-actions.debug"),
            &language_example("function-return-comparison-value-decision-actions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 3);
    assert_eq!(stdout.matches("decision.rule.selected").count(), 3);
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001] choose")
            .count(),
        3
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_complete_ordered_comparison_decision_action_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-COMPARISON-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ORDERED-COMPARISON-DECISION-ACTIONS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-ordered-comparison-decision-actions.debug"),
            &language_example("function-return-ordered-comparison-decision-actions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 3);
    assert_eq!(stdout.matches("decision.rule.selected").count(), 3);
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001] choose")
            .count(),
        3
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_final_fallback_enum_decision_action_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-ENUM-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ENUM-FALLBACK-DECISION-ACTIONS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-enum-fallback-decision-actions.debug"),
            &language_example("function-return-enum-fallback-decision-actions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 3);
    assert_eq!(stdout.matches("decision.rule.selected").count(), 3);
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001] choose")
            .count(),
        3
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_exhaustive_enum_decision_action_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-ENUM-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ENUM-EXHAUSTIVE-DECISION-ACTIONS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-exhaustive-enum-decision-actions.debug"),
            &language_example("function-return-exhaustive-enum-decision-actions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 3);
    assert_eq!(stdout.matches("decision.rule.selected").count(), 3);
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001] choose")
            .count(),
        3
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_optional_decision_action_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-OPTIONAL-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-DECISION-ACTIONS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-optional-decision-actions.debug"),
            &language_example("function-return-optional-decision-actions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 4);
    assert_eq!(stdout.matches("decision.rule.selected").count(), 4);
    assert_eq!(stdout.matches("optional.payload.bound").count(), 1);
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001]")
            .count(),
        4
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1001"));
}

#[test]
fn records_result_decision_action_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-RESULT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-RESULT-DECISION-ACTIONS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-result-decision-actions.debug"),
            &language_example("function-return-result-decision-actions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 2);
    assert_eq!(stdout.matches("decision.rule.selected").count(), 2);
    assert_eq!(stdout.matches("result.payload.bound").count(), 2);
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001] choose")
            .count(),
        2
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_error_code_decision_action_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-ERROR-CODE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ERROR-CODE-DECISION-ACTIONS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-error-code-decision-actions.debug"),
            &language_example("function-return-error-code-decision-actions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 6);
    assert_eq!(stdout.matches("decision.rule.selected").count(), 6);
    assert_eq!(stdout.matches("error.code.matched").count(), 3);
    assert_eq!(stdout.matches("result.payload.bound").count(), 3);
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001] recover")
            .count(),
        3
    );
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001] classify")
            .count(),
        3
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_list_decision_action_exits_reversibly() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-LIST-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-LIST-DECISION-ACTIONS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return-list-decision-actions.debug"),
            &language_example("function-return-list-decision-actions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("function.return.explicit").count(), 2);
    assert_eq!(stdout.matches("decision.rule.selected").count(), 2);
    assert_eq!(stdout.matches("list.entry.decomposed").count(), 1);
    assert_eq!(
        stdout
            .matches("function.exit [TOPAL-FUNCTION-ORDINARY-001] choose")
            .count(),
        2
    );
    assert!(!stdout.contains("integer.literal [TOPAL-NUM-LITERAL-001] 1000"));
}

#[test]
fn records_function_interface_declaration() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/source-step-history.debug"),
            &language_example("function-interface.t"),
        ])
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
            .contains("interface.implemented [TOPAL-INTERFACE-IMPLEMENTATION-001] Parser")
    );
}

#[test]
fn records_capability_composition() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/source-step-history.debug"),
            &language_example("capability-composition.t"),
        ])
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
            .contains("capability.composed [TOPAL-CAPABILITY-EVIDENCE-001]")
    );
}

#[test]
fn records_reversible_static_introspection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("static-introspection.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("introspection.object.viewed"));
    assert!(stdout.contains("introspection.context.viewed"));
    assert!(stdout.contains("(true, false, v0.1)"));
}

#[test]
fn records_reversible_native_serialization() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("native-serialization.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("serialization.serialized [TOPAL-SER-CANON-001]"));
    assert!(stdout.contains("serialization.deserialized [TOPAL-SER-DESER-001]"));
    assert!(stdout.contains("(answer is 42, accepted is true)"));
}

#[test]
fn records_reversible_function_effect_bounds() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("function-effect-bound.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(
            "function.effect-bound.declared [TOPAL-FUNCTION-EFFECT-BOUND-001] Effects ()"
        )
    );
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_packaged_function_defaults() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("packaged-function-operand.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout
            .contains("function.argument.defaulted [TOPAL-FUNCTION-PACKAGED-OPERAND-001] fallback")
    );
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn follows_reversible_task_message_transactions() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("task-message-transactions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("message.sent [TOPAL-CONC-INTERACT-001] transaction="));
    assert!(stdout.contains("message.received [TOPAL-DEBUG-MESSAGE-001] transaction="));
    assert!(stdout.contains("task.state.replaced [TOPAL-TASK-STATE-001] count"));
    assert!(stdout.contains("message.stream.started [TOPAL-TASK-MESSAGE-001] transaction="));
    assert!(stdout.contains("\n42\n"));
}
