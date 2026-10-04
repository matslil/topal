#[test]
fn records_anonymous_product_functions_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}anonymous-product-functions.debug"),
            &language_example("anonymous-product-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(stdout.contains("function.anonymous.captured"));
    assert!(stdout.contains("function.anonymous.called"));
    assert!(stdout.contains("<anonymous fn/2>"));
}

#[test]
fn records_repeated_anonymous_pattern_identity_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}repeated-anonymous-patterns.debug"),
            &language_example("repeated-anonymous-patterns.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-MATCH-001"));
    assert!(stdout.contains("pattern.identity.matched"));
    assert!(stdout.contains("<anonymous fn/1>"));
}

#[test]
fn records_repeated_anonymous_aggregate_identity_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}repeated-anonymous-aggregate-patterns.debug"),
            &language_example("repeated-anonymous-aggregate-patterns.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-MATCH-001"));
    assert!(stdout.contains("pattern.identity.matched"));
    assert!(stdout.contains("<anonymous fn/2>"));
}

#[test]
fn records_repeated_sum_identity_reversibly() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-SUM-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}repeated-sum-patterns.debug"),
            &language_example("repeated-sum-patterns.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-MATCH-001"));
    assert!(stdout.contains("pattern.identity.matched"));
    assert!(stdout.contains("function.anonymous.called"));
}

#[test]
fn records_nominal_sum_equality_reversibly() {
    // TOPAL-INTP-SUBSET-256, TOPAL-TYPE-SUM-EQUALITY-001,
    // TOPAL-TYPE-EQUALITY-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}sum-equality.debug"),
            &language_example("sum-equality.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-SUM-EQUALITY-001"));
    assert!(stdout.contains("equality.sum"));
    assert!(stdout.contains("evaluation.equal"));
}

#[test]
fn records_packaged_field_association_reversibly() {
    // TOPAL-INTP-SUBSET-257, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}packaged-function-association-order.debug"),
            &language_example("packaged-function-association-order.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let right = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] right-value")
        .unwrap();
    let left = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] left-value")
        .unwrap();
    assert!(right < left, "{stdout}");
    assert!(stdout.contains("function.argument.bound"));
    assert!(stdout.contains("TOPAL-FUNCTION-PACKAGED-OPERAND-001"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_compound_packaged_operands_reversibly() {
    // TOPAL-INTP-SUBSET-258, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}compound-packaged-function-operands.debug"),
            &language_example("compound-packaged-function-operands.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let left = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] left-value")
        .unwrap();
    let right = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] right-value")
        .unwrap();
    let scale_value = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] scale-value")
        .unwrap();
    let scale_factor = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] scale-factor")
        .unwrap();
    assert!(left < right && scale_value < scale_factor, "{stdout}");
    assert!(stdout.contains("function.argument.defaulted"));
    assert!(stdout.contains("TOPAL-FUNCTION-PACKAGED-OPERAND-001"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_structured_packaged_fields_reversibly() {
    // TOPAL-INTP-SUBSET-259, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}structured-packaged-function-fields.debug"),
            &language_example("structured-packaged-function-fields.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let person = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-person")
        .unwrap();
    let pair = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-pair")
        .unwrap();
    assert!(person < pair, "{stdout}");
    assert!(stdout.contains("function.argument.defaulted"));
    assert!(stdout.contains("TOPAL-FUNCTION-PACKAGED-OPERAND-001"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_sum_packaged_fields_reversibly() {
    // TOPAL-INTP-SUBSET-260, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}sum-packaged-function-fields.debug"),
            &language_example("sum-packaged-function-fields.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let message = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-message")
        .unwrap();
    let offset = stdout
        .rfind("function.selected [TOPAL-TYPE-CALL-001] offset-value")
        .unwrap();
    assert!(message < offset, "{stdout}");
    assert!(stdout.contains("function.argument.defaulted"));
    assert!(stdout.contains("TOPAL-FUNCTION-PACKAGED-OPERAND-001"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_function_packaged_fields_reversibly() {
    // TOPAL-INTP-SUBSET-261, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-packaged-fields.debug"),
            &language_example("function-packaged-fields.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let value = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-value")
        .unwrap();
    let operation = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-operation")
        .unwrap();
    assert!(value < operation, "{stdout}");
    assert!(stdout.contains("function.argument.defaulted"));
    assert!(stdout.contains("TOPAL-FUNCTION-PACKAGED-OPERAND-001"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_function_aggregate_packaged_fields_reversibly() {
    // TOPAL-INTP-SUBSET-262, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-aggregate-packaged-fields.debug"),
            &language_example("function-aggregate-packaged-fields.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("function.selected [TOPAL-TYPE-CALL-001] make-record"));
    assert!(stdout.contains("function.selected [TOPAL-TYPE-CALL-001] make-tuple"));
    assert!(stdout.contains("function.argument.defaulted"));
    assert!(stdout.contains("TOPAL-FUNCTION-PACKAGED-OPERAND-001"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_container_packaged_fields_reversibly() {
    // TOPAL-INTP-SUBSET-263, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}container-packaged-fields.debug"),
            &language_example("container-packaged-fields.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let span = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-span")
        .unwrap();
    let outcome = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-outcome")
        .unwrap();
    let maybe = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-maybe")
        .unwrap();
    let values = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-values")
        .unwrap();
    assert!(
        span < outcome && outcome < maybe && maybe < values,
        "{stdout}"
    );
    assert!(stdout.contains("function.argument.defaulted"));
    assert!(stdout.contains("TOPAL-FUNCTION-PACKAGED-OPERAND-001"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_collection_packaged_fields_reversibly() {
    // TOPAL-INTP-SUBSET-264, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-TYPE-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}collection-packaged-fields.debug"),
            &language_example("collection-packaged-fields.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let scores = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-map")
        .unwrap();
    let occurrences = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-bag")
        .unwrap();
    let members = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-set")
        .unwrap();
    let array = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-array")
        .unwrap();
    assert!(
        scores < occurrences && occurrences < members && members < array,
        "{stdout}"
    );
    assert!(stdout.contains("function.argument.bound [TOPAL-FUNCTION-ORDINARY-001] array"));
    assert!(stdout.contains("function.argument.bound [TOPAL-FUNCTION-ORDINARY-001] scores"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_scope_packaged_fields_reversibly() {
    // TOPAL-INTP-SUBSET-265, TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-NAMESPACE-FUNCTION-BOUNDARY-001, TOPAL-TYPE-CALL-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scope-packaged-fields.debug"),
            &language_example("scope-packaged-fields.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let make_value = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-value")
        .unwrap();
    let scope = stdout
        .find("function.argument.bound [TOPAL-FUNCTION-ORDINARY-001] scope")
        .unwrap();
    let value = stdout
        .find("function.argument.bound [TOPAL-FUNCTION-ORDINARY-001] value")
        .unwrap();
    assert!(make_value < scope && scope < value, "{stdout}");
    assert!(
        stdout.contains("function.argument.defaulted [TOPAL-FUNCTION-PACKAGED-OPERAND-001] scope")
    );
    assert!(stdout.contains("namespace.alias.member.resolved [TOPAL-NAMESPACE-ALIAS-001] answer"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_function_root_data_reversibly() {
    // TOPAL-INTP-SUBSET-266, TOPAL-NAMESPACE-ROOT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-root-data.debug"),
            &language_example("function-root-data.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let initializer = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-answer")
        .unwrap();
    let invocation = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] read")
        .unwrap();
    let root_answer = stdout
        .find("namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] answer")
        .unwrap();
    let root_label = stdout
        .find("namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] label")
        .unwrap();
    assert!(initializer < invocation && invocation < root_answer && root_answer < root_label);
    assert!(stdout.contains("function.argument.bound [TOPAL-FUNCTION-ORDINARY-001] answer"));
    assert!(stdout.contains("evaluation.result [TOPAL-SYN-GRAMMAR-001] (Int, Int, String)"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_function_root_data_forwarding_reversibly() {
    // TOPAL-INTP-SUBSET-267, TOPAL-NAMESPACE-ROOT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-root-data-forwarding.debug"),
            &language_example("function-root-data-forwarding.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let forward = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] forward")
        .unwrap();
    let relay = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] relay")
        .unwrap();
    let read = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] read")
        .unwrap();
    let root_answer = stdout
        .find("namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] answer")
        .unwrap();
    let root_label = stdout
        .find("namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] label")
        .unwrap();
    assert!(
        forward < relay && relay < read && read < root_answer && root_answer < root_label,
        "{stdout}"
    );
    assert!(stdout.contains("evaluation.result [TOPAL-SYN-GRAMMAR-001] (Int, Int, String)"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_repeated_function_aggregate_identity_reversibly() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}repeated-function-aggregate-patterns.debug"),
            &language_example("repeated-function-aggregate-patterns.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-MATCH-001"));
    assert!(stdout.contains("pattern.identity.matched"));
    assert!(stdout.contains("<anonymous fn/2>"));
}

#[test]
fn records_repeated_captured_function_identity_reversibly() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}repeated-captured-function-patterns.debug"),
            &language_example("repeated-captured-function-patterns.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-MATCH-001"));
    assert!(stdout.contains("pattern.identity.matched"));
    assert!(stdout.contains("function.anonymous.captured"));
}

#[test]
fn records_repeated_captured_named_function_identity_reversibly() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-NESTED-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}repeated-captured-named-function-patterns.debug"),
            &language_example("repeated-captured-named-function-patterns.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-MATCH-001"));
    assert!(stdout.contains("pattern.identity.matched"));
    assert!(stdout.contains("function.value.called"));
}

#[test]
fn records_repeated_captured_function_aggregate_identity_reversibly() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}repeated-captured-function-aggregate-patterns.debug"),
            &language_example("repeated-captured-function-aggregate-patterns.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-MATCH-001"));
    assert!(stdout.contains("pattern.identity.matched"));
    assert!(stdout.contains("function.anonymous.captured"));
}

#[test]
fn records_short_circuiting_traversal_control_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("traversal-control.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-EXEC-TRAVERSAL-CONTROL-001"));
    assert!(stdout.contains("traversal.finished"));
}

#[test]
fn records_symbolic_callable_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}callable-values.debug"),
            &language_example("callable-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-CALLABLE-VALUE-001"));
    assert!(stdout.contains("function.callable.called"));
}

#[test]
fn records_the_complete_symbolic_callable_vocabulary_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}expanded-callable-values.debug"),
            &language_example("expanded-callable-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-CALLABLE-VALUE-001"));
    assert!(stdout.contains("function.callable.called"));
    assert!(stdout.contains('*'));
}

#[test]
fn records_named_function_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}named-function-values.debug"),
            &language_example("named-function-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-VALUE-001"));
    assert!(stdout.contains("<fn increment>"));
}

#[test]
fn records_function_results_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-results.debug"),
            &language_example("function-results.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-VALUE-001"));
    assert!(stdout.contains("TOPAL-FUNCTION-CALLABLE-VALUE-001"));
    assert!(stdout.contains("<fn increment>"));
    assert!(stdout.contains('+'));
}

#[test]
fn records_lazy_iterate_construction_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}iterate-generator.debug"),
            &language_example("iterate-generator.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-ITERATE-001"));
    assert!(stdout.contains("<Generator Int Unit Unit>"));
}

#[test]
fn records_lazy_take_while_construction_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-print-digits.debug"),
            &language_example("iterate-take-while.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-TAKE-WHILE-001"));
    assert!(stdout.contains("<Generator Int Unit Unit>"));
}

#[test]
fn records_generated_foreach_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}generated-foreach.debug"),
            &language_example("generated-foreach.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-ITERATE-FOREACH-001"));
    assert!(stdout.contains("generator.returned"));
}

#[test]
fn records_generated_collection_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-print-digits.debug"),
            &language_example("generated-collect.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-COLLECT-001"));
    assert!(stdout.contains("Entry ( 0"));
}

#[test]
fn records_lazy_unfold_construction_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}unfold-generator.debug"),
            &language_example("unfold-generator.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-UNFOLD-001"));
    assert!(stdout.contains("<Generator Int Unit Unit>"));
}

#[test]
fn records_unfold_collection_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("unfold-collect.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-UNFOLD-COLLECT-001"));
    assert!(stdout.contains("Entry ( 4"));
}

#[test]
fn records_root_namespace_resolution_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}root-namespace.debug"),
            &language_example("root-namespace.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NAMESPACE-ROOT-001"));
    assert!(stdout.contains("<namespace root>"));
}

#[test]
fn records_namespace_alias_resolution_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-print-current.debug"),
            &language_example("namespace-alias.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NAMESPACE-ALIAS-001"));
    assert!(stdout.contains("<namespace root>"));
}

#[test]
fn records_namespace_use_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-print-current.debug"),
            &language_example("use-namespace.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("TOPAL-NAMESPACE-USE-001")
    );
}

#[test]
fn records_namespace_snapshot_visibility_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}namespace-snapshot.debug"),
            &language_example("namespace-snapshot.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("<namespace root>")
    );
}

#[test]
fn records_namespace_overload_selection_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-print-api.debug"),
            &language_example("namespace-overloads.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("TOPAL-FUNCTION-OVERLOAD-001")
    );
}

#[test]
fn records_qualified_namespace_generators_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-print-api.debug"),
            &language_example("namespace-generator.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("generator.yielded")
    );
}

#[test]
fn records_scope_classification_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-print-api.debug"),
            &language_example("scope-classifier.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("<namespace root>")
    );
}

#[test]
fn records_namespace_alias_chains_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}namespace-alias-chain.debug"),
            &language_example("namespace-alias-chain.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("<namespace root>")
    );
}

#[test]
fn records_scope_function_parameters_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("namespace-function-parameter.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("function.argument.bound")
    );
}

#[test]
fn records_fundamental_type_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("type-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("TOPAL-ABSTRACTION-TYPE-VALUE-001")
    );
}

#[test]
fn steps_over_diagnostic_controls_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("diagnostic-controls.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
}

#[test]
fn steps_through_lexical_blocks_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("empty-block.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
}

#[test]
fn steps_through_discard_input_patterns() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("discard-function-pattern.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("function.argument.discarded")
    );
}

#[test]
fn records_defining_context_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/source-step-history.debug"),
            &language_example("constructed-context.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("context.member.selected [TOPAL-CONTEXT-SELECT-001] offset"));
    assert!(stdout.contains("evaluation.add [TOPAL-NUM-ADD-001] Int"));
}

#[test]
fn records_defining_context_forwarding_reversibly() {
    // TOPAL-INTP-SUBSET-268, TOPAL-CONTEXT-SELECT-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}defining-context-forwarding.debug"),
            &language_example("defining-context-forwarding.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let forward = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] forward")
        .unwrap();
    let relay = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] relay")
        .unwrap();
    let read = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] read")
        .unwrap();
    let offset = stdout
        .find("context.member.selected [TOPAL-CONTEXT-SELECT-001] offset")
        .unwrap();
    let label = stdout
        .find("context.member.selected [TOPAL-CONTEXT-SELECT-001] label")
        .unwrap();
    assert!(forward < relay && relay < read && read < offset && offset < label);
    assert!(stdout.contains("evaluation.result [TOPAL-SYN-GRAMMAR-001] (Int, Int, String)"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_recursive_scalar_environments_reversibly() {
    // TOPAL-INTP-SUBSET-269,
    // TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}recursive-scalar-environments.debug"),
            &language_example("recursive-scalar-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let even = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] cycle-even")
        .unwrap();
    let odd = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] cycle-odd")
        .unwrap();
    let context = stdout
        .find("context.member.selected [TOPAL-CONTEXT-SELECT-001] captured")
        .unwrap();
    let root_member = stdout
        .find("namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live")
        .unwrap();
    assert!(even < odd);
    assert!(root_member < context);
    assert!(stdout.contains("evaluation.result [TOPAL-SYN-GRAMMAR-001]"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_aggregate_environments_reversibly() {
    // TOPAL-INTP-SUBSET-270,
    // TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}aggregate-environments.debug"),
            &language_example("aggregate-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let context_pair = stdout
        .find("context.member.selected [TOPAL-CONTEXT-SELECT-001] context-pair")
        .unwrap();
    let context_record = stdout
        .find("context.member.selected [TOPAL-CONTEXT-SELECT-001] context-record")
        .unwrap();
    let root_pair = stdout
        .find("namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-pair")
        .unwrap();
    let root_record = stdout
        .find("namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-record")
        .unwrap();
    let context_sum = stdout
        .find("context.member.selected [TOPAL-CONTEXT-SELECT-001] context-token")
        .unwrap();
    let root_sum = stdout
        .find("namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-token")
        .unwrap();
    assert!(context_pair < context_record);
    assert!(context_record < root_pair);
    assert!(root_pair < root_record);
    assert!(root_record < context_sum);
    assert!(context_sum < root_sum);
    assert!(stdout.contains("evaluation.result [TOPAL-SYN-GRAMMAR-001]"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_overload_selected_environments_reversibly() {
    // TOPAL-INTP-SUBSET-271,
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}overload-environments.debug"),
            &language_example("overload-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let context_number = stdout
        .find("context.member.selected [TOPAL-CONTEXT-SELECT-001] context-number")
        .unwrap();
    let context_label = stdout
        .find("context.member.selected [TOPAL-CONTEXT-SELECT-001] context-label")
        .unwrap();
    let context_pair = stdout
        .find("context.member.selected [TOPAL-CONTEXT-SELECT-001] context-pair")
        .unwrap();
    let root_pair = stdout
        .find("namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-pair")
        .unwrap();
    assert!(context_number < context_label);
    assert!(context_label < context_pair);
    assert!(context_pair < root_pair);
    assert!(
        stdout.contains(
            "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] choose-context (Int)"
        )
    );
    assert!(stdout.contains(
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] choose-context (String)"
    ));
    assert!(
        stdout.contains("function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] cross (String)")
    );
    assert!(stdout.contains("function.selected [TOPAL-TYPE-CALL-001] cross (Int)"));
    assert!(stdout.contains(
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] choose-product ((Int, String))"
    ));
    assert!(stdout.contains(
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] choose-product (Record (amount : Int, label : String))"
    ));
    assert!(stdout.contains("evaluation.result [TOPAL-SYN-GRAMMAR-001]"));
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_local_named_function_environments_reversibly() {
    // TOPAL-INTP-SUBSET-272,
    // TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}local-function-environments.debug"),
            &language_example("local-function-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let root_alias = stdout
        .find("binding.resolved [TOPAL-SYN-BIND-001] read-context")
        .unwrap();
    let chained_alias = stdout
        .find("binding.resolved [TOPAL-SYN-BIND-001] context-operation")
        .unwrap();
    let first_call = stdout
        .find("function.value.called [TOPAL-FUNCTION-VALUE-001] read-context")
        .unwrap();
    let nested_declaration = stdout
        .find("function.declared [TOPAL-FUNCTION-ORDINARY-001] nested-context")
        .unwrap();
    let nested_call = stdout
        .find("function.value.called [TOPAL-FUNCTION-VALUE-001] nested-context")
        .unwrap();
    assert!(root_alias < chained_alias);
    assert!(chained_alias < first_call);
    assert!(first_call < nested_declaration);
    assert!(nested_declaration < nested_call);
    for event in [
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] context-chain (Int)",
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] context-chain (String)",
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] root-chain (Int)",
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] root-chain (String)",
        "function.selected [TOPAL-TYPE-CALL-001] context-operation (Int)",
        "function.selected [TOPAL-TYPE-CALL-001] root-operation (Int)",
        "context.member.selected [TOPAL-CONTEXT-SELECT-001] context-pair",
        "namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-pair",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_function_environment_boundaries_reversibly() {
    // TOPAL-INTP-SUBSET-273,
    // TOPAL-COMPILER-FUNCTION-ENVIRONMENT-BOUNDARY-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-environment-boundaries.debug"),
            &language_example("function-environment-boundaries.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let named_result = stdout
        .find("binding.bind [TOPAL-SYN-BIND-001] context-operation")
        .unwrap();
    let aggregate_boundary = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] forward-record")
        .unwrap();
    let anonymous_call = stdout
        .find("function.anonymous.called [TOPAL-FUNCTION-ANONYMOUS-001]")
        .unwrap();
    let nested_boundary = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] forward-int")
        .unwrap();
    assert!(named_result < aggregate_boundary);
    assert!(aggregate_boundary < anonymous_call);
    assert!(anonymous_call < nested_boundary);
    for event in [
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] context-operation (Int)",
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] context-operation (String)",
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] read-root (Int)",
        "function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] read-pair (String)",
        "context.member.selected [TOPAL-CONTEXT-SELECT-001] context-number",
        "namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-number",
        "function.value.called [TOPAL-FUNCTION-VALUE-001] increase",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
}

#[test]
fn records_escaping_nested_function_environments_reversibly() {
    // TOPAL-INTP-SUBSET-274,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}escaping-nested-function-environments.debug"),
            &language_example("escaping-nested-function-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_factory = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-operation")
        .unwrap();
    let forwarded = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] return-operation")
        .unwrap();
    let aggregate = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] forward-record")
        .unwrap();
    let first_nested = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] increase")
        .unwrap();
    assert!(first_factory < forwarded);
    assert!(forwarded < aggregate);
    assert!(aggregate < first_nested);
    for event in [
        "binding.bind [TOPAL-SYN-BIND-001] first-operation",
        "binding.bind [TOPAL-SYN-BIND-001] second-operation",
        "binding.bind [TOPAL-SYN-BIND-001] forwarded-operation",
        "binding.bind [TOPAL-SYN-BIND-001] pair-operation",
        "function.selected [TOPAL-TYPE-CALL-001] apply-record",
        "function.value.called [TOPAL-FUNCTION-VALUE-001] read-pair",
        "function.selected [TOPAL-TYPE-CALL-001] pair-operation",
        "context.member.selected [TOPAL-CONTEXT-SELECT-001] context-offset",
        "namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-offset",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_optional_function_environments_reversibly() {
    // TOPAL-INTP-SUBSET-275,
    // TOPAL-COMPILER-OPTIONAL-FUNCTION-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}optional-function-environments.debug"),
            &language_example("optional-function-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_factory = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-optional")
        .unwrap();
    let first_selection = stdout
        .find("optional.payload.bound [TOPAL-DECISION-OPTIONAL-001] operation")
        .unwrap();
    let first_nested = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] operation")
        .unwrap();
    assert!(first_factory < first_selection);
    assert!(first_selection < first_nested);
    for event in [
        "optional.some.constructed [TOPAL-TYPE-OPTIONAL-CONTEXT-001] Function",
        "optional.none.constructed [TOPAL-TYPE-OPTIONAL-CONTEXT-001] Function",
        "binding.bind [TOPAL-SYN-BIND-001] first",
        "binding.bind [TOPAL-SYN-BIND-001] second",
        "function.selected [TOPAL-TYPE-CALL-001] return-optional",
        "function.selected [TOPAL-TYPE-CALL-001] return-tuple",
        "function.selected [TOPAL-TYPE-CALL-001] apply-package",
        "function.selected [TOPAL-TYPE-CALL-001] apply-record",
        "pattern.identity.matched [TOPAL-TYPE-MATCH-001] candidate",
        "context.member.selected [TOPAL-CONTEXT-SELECT-001] context-offset",
        "namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-offset",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_sum_function_environments_reversibly() {
    // TOPAL-INTP-SUBSET-276,
    // TOPAL-COMPILER-SUM-FUNCTION-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}sum-function-environments.debug"),
            &language_example("sum-function-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_factory = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-operation")
        .unwrap();
    let first_selection = stdout
        .find("union.payload.bound [TOPAL-DECISION-UNION-001] operation")
        .unwrap();
    let first_nested = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] operation")
        .unwrap();
    assert!(first_factory < first_selection);
    assert!(first_selection < first_nested);
    for event in [
        "union.constructed [TOPAL-TYPE-UNION-001] Apply",
        "variant.constructed [TOPAL-TYPE-VARIANT-001] 0",
        "binding.bind [TOPAL-SYN-BIND-001] first",
        "binding.bind [TOPAL-SYN-BIND-001] second",
        "function.selected [TOPAL-TYPE-CALL-001] return-operation",
        "function.selected [TOPAL-TYPE-CALL-001] apply-package",
        "function.selected [TOPAL-TYPE-CALL-001] apply-choice",
        "pattern.identity.matched [TOPAL-TYPE-MATCH-001] candidate",
        "context.member.selected [TOPAL-CONTEXT-SELECT-001] context-offset",
        "namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-offset",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_result_function_environments_reversibly() {
    // TOPAL-INTP-SUBSET-277,
    // TOPAL-COMPILER-RESULT-FUNCTION-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}result-function-environments.debug"),
            &language_example("result-function-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_factory = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-result")
        .unwrap();
    let first_selection = stdout
        .find("result.payload.bound [TOPAL-DECISION-RESULT-001] operation")
        .unwrap();
    let first_nested = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] operation")
        .unwrap();
    assert!(first_factory < first_selection);
    assert!(first_selection < first_nested);
    for event in [
        "result.success.projected [TOPAL-TYPE-RESULT-PROJECT-001] quotient",
        "result.error.constructed [TOPAL-TYPE-RESULT-001] root./(Rational,Rational);division-by-zero",
        "binding.bind [TOPAL-SYN-BIND-001] first",
        "binding.bind [TOPAL-SYN-BIND-001] second",
        "function.selected [TOPAL-TYPE-CALL-001] return-result",
        "function.selected [TOPAL-TYPE-CALL-001] apply-package",
        "function.selected [TOPAL-TYPE-CALL-001] apply-record",
        "context.member.selected [TOPAL-CONTEXT-SELECT-001] context-offset",
        "namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-offset",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_list_function_environments_reversibly() {
    // TOPAL-INTP-SUBSET-278,
    // TOPAL-COMPILER-LIST-FUNCTION-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-function-environments.debug"),
            &language_example("list-function-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_factory = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-list")
        .unwrap();
    let first_decomposition = stdout
        .find("list.entry.decomposed [TOPAL-DECISION-LIST-001] first=operation;rest=remaining")
        .unwrap();
    let first_nested = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] operation")
        .unwrap();
    assert!(first_factory < first_decomposition);
    assert!(first_decomposition < first_nested);
    for event in [
        "list.empty.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Function",
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Function",
        "binding.bind [TOPAL-SYN-BIND-001] first",
        "binding.bind [TOPAL-SYN-BIND-001] second",
        "function.selected [TOPAL-TYPE-CALL-001] return-list",
        "function.selected [TOPAL-TYPE-CALL-001] apply-package",
        "function.selected [TOPAL-TYPE-CALL-001] apply-record",
        "context.member.selected [TOPAL-CONTEXT-SELECT-001] context-offset",
        "namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-offset",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_array_function_environments_reversibly() {
    // TOPAL-INTP-SUBSET-279,
    // TOPAL-COMPILER-ARRAY-FUNCTION-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}array-function-environments.debug"),
            &language_example("array-function-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_factory = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-array")
        .unwrap();
    let first_access = stdout
        .find("array.checked-access [TOPAL-ARRAY-GET-CHECKED-001] array-at?")
        .unwrap();
    let first_nested = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] operation")
        .unwrap();
    assert!(first_factory < first_access);
    assert!(first_access < first_nested);
    for event in [
        "array.collected [TOPAL-ARRAY-COLLECT-001] count=2",
        "array.collected [TOPAL-ARRAY-COLLECT-001] count=0",
        "optional.payload.bound [TOPAL-DECISION-OPTIONAL-001] operation",
        "function.selected [TOPAL-TYPE-CALL-001] return-array",
        "function.selected [TOPAL-TYPE-CALL-001] apply-package",
        "function.selected [TOPAL-TYPE-CALL-001] apply-record",
        "collection.entry-count [TOPAL-COLLECTION-ENTRY-COUNT-001] entries=2",
        "collection.empty.tested [TOPAL-COLLECTION-EMPTY-PREDICATE-001] true",
        "context.member.selected [TOPAL-CONTEXT-SELECT-001] context-offset",
        "namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-offset",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_map_function_environments_reversibly() {
    // TOPAL-INTP-SUBSET-280,
    // TOPAL-COMPILER-MAP-FUNCTION-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}map-function-environments.debug"),
            &language_example("map-function-environments.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_factory = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] make-map")
        .unwrap();
    let first_lookup = stdout
        .find("map.lookup [TOPAL-MAP-LOOKUP-001] map-lookup")
        .unwrap();
    let first_nested = stdout
        .find("function.selected [TOPAL-TYPE-CALL-001] operation")
        .unwrap();
    assert!(first_factory < first_lookup);
    assert!(first_lookup < first_nested);
    for event in [
        "map.collected [TOPAL-MAP-COLLECT-001] keep-first",
        "map.collected [TOPAL-MAP-COLLECT-001] keep-last",
        "map.collected [TOPAL-MAP-COLLECT-001] reject",
        "optional.payload.bound [TOPAL-DECISION-OPTIONAL-001] operation",
        "function.selected [TOPAL-TYPE-CALL-001] return-map",
        "function.selected [TOPAL-TYPE-CALL-001] apply-package",
        "function.selected [TOPAL-TYPE-CALL-001] apply-record",
        "collection.entry-count [TOPAL-COLLECTION-ENTRY-COUNT-001] entries=2",
        "collection.empty.tested [TOPAL-COLLECTION-EMPTY-PREDICATE-001] false",
        "context.member.selected [TOPAL-CONTEXT-SELECT-001] context-offset",
        "namespace.member.resolved [TOPAL-NAMESPACE-ROOT-001] live-offset",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_boolean_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-281, TOPAL-COMPILER-LIST-BOOLEAN-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-boolean-values.debug"),
            &language_example("list-boolean-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] Boolean",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=2",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}

#[test]
fn records_string_list_values_reversibly() {
    // TOPAL-INTP-SUBSET-282, TOPAL-COMPILER-LIST-STRING-CORE-001
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-string-values.debug"),
            &language_example("list-string-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for event in [
        "list.entry.constructed [TOPAL-TYPE-LIST-CONSTRUCT-001] String",
        "list.entry.decomposed [TOPAL-DECISION-LIST-001] first=first;rest=rest",
        "equality.list [TOPAL-TYPE-LIST-EQUALITY-001] String",
        "list.entry-count [TOPAL-LIST-ENTRY-COUNT-001] entries=2",
        "list.empty.tested [TOPAL-LIST-EMPTY-PREDICATE-001] true",
        "evaluation.result [TOPAL-SYN-GRAMMAR-001]",
    ] {
        assert!(stdout.contains(event), "{event}: {stdout}");
    }
    assert!(stdout.contains("function.exit"));
}
