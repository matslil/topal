//! Structured numeric policy admission and independent evaluation obligations.
use super::*;

fn requirement() -> CompilerSystemsSourceLibraryRequirement {
    CompilerSystemsSourceLibraryRequirement {
        source: "policy/records.t".into(),
        export: "sleep-deadline".into(),
        interface: CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1,
        stack_limit_bytes: 4096,
        runtime_limit_bytes: Some(65536),
        state_limit_bytes: None,
    }
}
fn input(helpers: &str, body: &str) -> SystemsSourceLibraryInput {
    SystemsSourceLibraryInput {
        source: requirement().source.clone(),
        source_name: requirement().source,
        text: format!(
            "use language (version is v0.1)\n{helpers}\npub sleep-deadline is fn ((seconds : Nat, nanoseconds : Nat, ticks : Nat, wrapped : Boolean)) -> Int\n{body}\n"
        ),
        modules: Vec::new(),
    }
}
fn evaluate(unit: &CheckedNumericLibrary) -> Evaluation {
    let mut bounds = Bounds {
        functions: unit
            .program
            .functions
            .iter()
            .map(|f| (f.symbol.clone(), f))
            .collect(),
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
        maximum_bits: 0,
        regions: false,
        tail: false,
        tail_visits: 0,
    };
    let number = |n| Value::Number(Range(BigInt::from(n), BigInt::from(n)));
    bounds
        .function(
            &unit.entry_symbol,
            vec![number(3), number(5), number(7), Value::Boolean],
        )
        .unwrap()
}
#[test]
fn nested_records_and_reconstruction_preserve_base_values_and_charge_all_fields() {
    // TOPAL-SYSTEMS-NUMERIC-RECORDS-001, TOPAL-TYPE-RECONSTRUCT-001
    let helper = "sum is fn (state : Record (left : Nat, right : Record (value : Nat))) -> Nat\n  nested is state right\n  left is state left\n  right is nested value\n  left + right\n";
    let body = "  original is (left is seconds + ticks, right is (value is nanoseconds))\n  changed is original with (left is nanoseconds * ticks)\n  left is sum original\n  right is sum changed\n  left + right";
    let unit = check_numeric_library(&requirement(), &input(helper, body)).unwrap();
    let result = evaluate(&unit);
    assert_eq!(
        result.value.number().unwrap(),
        &Range(BigInt::from(55), BigInt::from(55))
    );
    assert!(result.bytes > 0);
    let inline = "  sum ((left is seconds + ticks, right is (value is nanoseconds)) with (left is nanoseconds * ticks))";
    let bound = "  original is (left is seconds + ticks, right is (value is nanoseconds))\n  sum (original with (left is nanoseconds * ticks))";
    let a = check_numeric_library(&requirement(), &input(helper, inline)).unwrap();
    let b = check_numeric_library(&requirement(), &input(helper, bound)).unwrap();
    assert_eq!(evaluate(&a).bytes, evaluate(&b).bytes);
    assert_eq!(evaluate(&a).value, evaluate(&b).value);
    // An unselected field still evaluates and consumes its numeric allocation.
    let cheap = check_numeric_library(
        &requirement(),
        &input(
            "",
            "  state is (chosen is seconds, unused is nanoseconds)\n  state chosen",
        ),
    )
    .unwrap();
    let costly = check_numeric_library(
        &requirement(),
        &input(
            "",
            "  state is (chosen is seconds, unused is nanoseconds * ticks)\n  state chosen",
        ),
    )
    .unwrap();
    assert_eq!(evaluate(&cheap).value, evaluate(&costly).value);
    assert!(evaluate(&costly).bytes > evaluate(&cheap).bytes);
}
#[test]
fn joined_records_retain_each_labels_full_numeric_range_independently_of_order() {
    // TOPAL-SYSTEMS-NUMERIC-RECORDS-001
    let source = input(
        "choose is fn ((seconds : Nat, nanoseconds : Nat, ticks : Nat, wrapped : Boolean)) -> Record (a : Nat, b : Nat)\n  wrapped\n    true then (a is seconds, b is nanoseconds)\n    false then (b is ticks, a is seconds + ticks)\n",
        "  choice is choose (seconds, nanoseconds, ticks, wrapped)\n  a is choice a\n  b is choice b\n  a + b",
    );
    let checked = check_numeric_library(&requirement(), &source).unwrap();
    let result = evaluate(&checked);
    assert_eq!(
        result.value.number().unwrap(),
        &Range(BigInt::from(8), BigInt::from(17))
    );
    let reordered = SystemsSourceLibraryInput {
        text: source.text.replace(
            "(a is seconds, b is nanoseconds)",
            "(b is nanoseconds, a is seconds)",
        ),
        ..source
    };
    let variant = check_numeric_library(&requirement(), &reordered).unwrap();
    assert_eq!(evaluate(&variant).value, result.value);
    assert_eq!(evaluate(&variant).bytes, result.bytes);
}
#[test]
fn malformed_shapes_unqualified_payloads_and_excessive_work_fail_before_emission() {
    // TOPAL-SYSTEMS-NUMERIC-RECORDS-001
    for body in [
        "  state is (value is seconds, value is ticks)\n  state value",
        "  state is (value is seconds)\n  changed is state with (missing is ticks)\n  changed value",
        "  state is (value is seconds)\n  changed is state with (value is true)\n  changed value",
        "  state is (value is seconds, text is \"unqualified\")\n  state value",
        "  state is (value is seconds, action is { value } value)\n  state value",
    ] {
        assert!(
            check_numeric_library(&requirement(), &input("", body)).is_err(),
            "accepted {body}"
        );
    }
    let mut body = "  state0 is (value is seconds)\n".to_owned();
    for i in 1..31 {
        let previous = i - 1;
        body.push_str(&format!("  state{i} is state{previous} with (value is (state{previous} value) * (state{previous} value))\n"));
    }
    body.push_str("  state30 value");
    let error = match check_numeric_library(&requirement(), &input("", &body)) {
        Ok(_) => panic!("unbounded reconstruction chain admitted"),
        Err(error) => error.to_string(),
    };
    assert!(
        error.contains("E-SYSTEMS-NUMERIC-LIBRARY")
            && (error.contains("bound")
                || error.contains("reserve")
                || error.contains("certificate")),
        "{error}"
    );
}

#[test]
fn actual_large_record_call_frames_must_fit_the_declared_native_stack() {
    // TOPAL-SYSTEMS-NUMERIC-RECORDS-001
    let fields = (0..256)
        .map(|i| format!("f{i} : Nat"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = (0..256)
        .map(|i| format!("f{i} is seconds"))
        .collect::<Vec<_>>()
        .join(", ");
    let helpers = format!("select-head is fn (record : Record ({fields})) -> Nat\n  record f0\n");
    let source = input(&helpers, &format!("  select-head ({values})"));
    let tools = crate::LlvmTools::discover(None).unwrap();
    let directory =
        std::env::temp_dir().join(format!("topal-numeric-record-stack-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let error = match build_numeric_library(&requirement(), &source, &tools, &directory) {
        Ok(_) => panic!("oversized aggregate call stack admitted"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("numeric stack requires"), "{error}");
    std::fs::remove_dir_all(directory).unwrap();
}
