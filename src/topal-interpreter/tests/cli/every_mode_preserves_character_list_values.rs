#[test]
fn every_mode_preserves_character_list_values() {
    // TOPAL-INTP-SUBSET-283, TOPAL-COMPILER-LIST-CHARACTER-CORE-001
    let source = include_str!("../../../../examples/language/list-character-values.t");
    let expected = "(\"A\u{30a}\", \"👩‍💻\", true, true, 2, true, \"K\", \"A\u{30a}\", \"R\", Entry ( \"A\u{30a}\", Entry ( \"👩‍💻\", Empty ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("list.entry.constructed"));
    assert!(trace.contains("list.entry.decomposed"));
    assert!(trace.contains("list.entry-count"));
    assert!(trace.contains("list.empty.tested"));
    assert!(trace.contains("equality.list"));
}

#[test]
fn every_mode_preserves_nat_list_values() {
    // TOPAL-INTP-SUBSET-284, TOPAL-COMPILER-LIST-NAT-CORE-001
    let source = include_str!("../../../../examples/language/list-nat-values.t");
    let expected = "(0, 123456789012345678901234567890, true, true, 3, true, 5, 0, 6, Entry ( 0, Entry ( 123456789012345678901234567890, Entry ( +Infinity, Empty ) ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("list.entry.constructed"));
    assert!(trace.contains("list.entry.decomposed"));
    assert!(trace.contains("list.entry-count"));
    assert!(trace.contains("list.empty.tested"));
    assert!(trace.contains("equality.list"));
}

#[test]
fn every_mode_preserves_rational_list_values() {
    // TOPAL-INTP-SUBSET-285, TOPAL-COMPILER-LIST-RATIONAL-CORE-001
    let source = include_str!("../../../../examples/language/list-rational-values.t");
    let expected = "(Rational ( 1, 2 ), Rational ( 17636684144620811271604938270, 1 ), true, true, 3, true, Rational ( 5, 1 ), Rational ( 1, 2 ), Rational ( 6, 1 ), Entry ( Rational ( 1, 2 ), Entry ( Rational ( 17636684144620811271604938270, 1 ), Entry ( +Infinity, Empty ) ) ))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_effect_list_values() {
    // TOPAL-INTP-SUBSET-286, TOPAL-COMPILER-LIST-EFFECT-CORE-001
    let source = include_str!("../../../../examples/language/list-effect-values.t");
    let expected = "(Effects (), Effects (), true, true, 2, true, Effects (), Effects (), Effects (), Entry ( Effects (), Entry ( Effects (), Empty ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_comparison_list_values() {
    // TOPAL-INTP-SUBSET-287, TOPAL-COMPILER-LIST-COMPARISON-CORE-001
    let source = include_str!("../../../../examples/language/list-comparison-values.t");
    let expected = "(Less, Equal, true, true, 3, true, Equal, Less, Greater, Entry ( Less, Entry ( Equal, Entry ( Greater, Empty ) ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_error_code_list_values() {
    // TOPAL-INTP-SUBSET-288, TOPAL-COMPILER-LIST-ERROR-CODE-CORE-001
    let source = include_str!("../../../../examples/language/list-error-code-values.t");
    let expected = "(out-of-range, not-representable, true, true, 4, true, division-by-zero, out-of-range, indeterminate, Entry ( out-of-range, Entry ( not-representable, Entry ( division-by-zero, Entry ( indeterminate, Empty ) ) ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_unit_list_values() {
    // TOPAL-INTP-SUBSET-289, TOPAL-COMPILER-LIST-UNIT-CORE-001
    let source = include_str!("../../../../examples/language/list-unit-values.t");
    let expected = "((), (), true, true, 3, true, (), (), (), Entry ( (), Entry ( (), Entry ( (), Empty ) ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_completed_list_values() {
    // TOPAL-INTP-SUBSET-290, TOPAL-COMPILER-LIST-COMPLETED-CORE-001
    let source = include_str!("../../../../examples/language/list-completed-values.t");
    let expected = "(Completed, Completed, true, true, 3, true, Completed, Completed, Completed, Entry ( Completed, Entry ( Completed, Entry ( Completed, Empty ) ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_type_list_values() {
    // TOPAL-INTP-SUBSET-291, TOPAL-COMPILER-LIST-TYPE-CORE-001
    let source = include_str!("../../../../examples/language/list-type-values.t");
    let expected = "(Boolean, Int, true, true, true, 7, true, String, Boolean, Unit, Entry ( Boolean, Entry ( Int, Entry ( Nat, Entry ( Rational, Entry ( String, Entry ( Unit, Entry ( Scope, Empty ) ) ) ) ) ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_nominal_enum_list_values() {
    // TOPAL-INTP-SUBSET-292, TOPAL-COMPILER-LIST-ENUM-CORE-001
    let source = include_str!("../../../../examples/language/list-enum-values.t");
    let expected = "(Red, Green, true, true, true, 3, true, Blue, Red, Green, Entry ( Red, Entry ( Green, Entry ( Blue, Empty ) ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_nominal_modular_list_values() {
    // TOPAL-INTP-SUBSET-293, TOPAL-COMPILER-LIST-MODULAR-CORE-001
    let source = include_str!("../../../../examples/language/list-modular-values.t");
    let expected = "(ByteCounter 0, ByteCounter 255, true, true, true, 3, true, ByteCounter 7, ByteCounter 0, ByteCounter 8, Entry ( ByteCounter 0, Entry ( ByteCounter 255, Entry ( ByteCounter 42, Empty ) ) ))";
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
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_optional_int_list_values() {
    // TOPAL-INTP-SUBSET-294, TOPAL-COMPILER-LIST-OPTIONAL-INT-CORE-001
    let source = include_str!("../../../../examples/language/list-optional-int-values.t");
    let expected = "(Some 1, None, true, true, true, 3, true, Some 7, Some 1, Some 8, Entry ( Some 1, Entry ( None, Entry ( Some -2, Empty ) ) ))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_optional_rational_list_values() {
    // TOPAL-INTP-SUBSET-295, TOPAL-COMPILER-LIST-OPTIONAL-RATIONAL-CORE-001
    let source = include_str!("../../../../examples/language/list-optional-rational-values.t");
    let expected = "(Some Rational ( 1, 2 ), None, true, true, true, 3, true, Some Rational ( 7, 3 ), Some Rational ( 1, 2 ), Some Rational ( 8, 5 ), Entry ( Some Rational ( 1, 2 ), Entry ( None, Entry ( Some Rational ( -3, 4 ), Empty ) ) ))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_optional_string_list_values() {
    // TOPAL-INTP-SUBSET-296, TOPAL-COMPILER-LIST-OPTIONAL-STRING-CORE-001
    let source = include_str!("../../../../examples/language/list-optional-string-values.t");
    let expected = "(Some \"first\", None, true, true, true, 3, true, Some \"fallback\", Some \"first\", Some \"record\", Entry ( Some \"first\", Entry ( None, Entry ( Some \"世界\", Empty ) ) ))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_int_pair_list_values() {
    // TOPAL-INTP-SUBSET-297, TOPAL-COMPILER-LIST-INT-PAIR-CORE-001
    let source = include_str!("../../../../examples/language/list-int-pair-values.t");
    let expected = "((1, 2), (3, -4), true, true, true, 3, true, (7, 8), (1, 2), (11, 12), Entry ( (1, 2), Entry ( (3, -4), Entry ( (340282366920938463463374607431768211456, -170141183460469231731687303715884105728), Empty ) ) ))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_int_string_pair_list_values() {
    // TOPAL-INTP-SUBSET-298, TOPAL-COMPILER-LIST-INT-STRING-PAIR-CORE-001
    let source = include_str!("../../../../examples/language/list-int-string-pair-values.t");
    let expected = "((1, \"one\"), (-4, \"räv\"), true, true, true, true, 3, true, (7, \"seven\"), (1, \"one\"), (11, \"eleven\"), Entry ( (1, \"one\"), Entry ( (-4, \"räv\"), Entry ( (340282366920938463463374607431768211456, \"\"), Empty ) ) ))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_string_int_pair_list_values() {
    // TOPAL-INTP-SUBSET-299, TOPAL-COMPILER-LIST-STRING-INT-PAIR-CORE-001
    let source = include_str!("../../../../examples/language/list-string-int-pair-values.t");
    let expected = "((\"one\", 1), (\"räv\", -4), true, true, true, true, 3, true, (\"seven\", 7), (\"one\", 1), (\"eleven\", 11), Entry ( (\"one\", 1), Entry ( (\"räv\", -4), Entry ( (\"\", 340282366920938463463374607431768211456), Empty ) ) ))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_string_pair_list_values() {
    // TOPAL-INTP-SUBSET-300, TOPAL-COMPILER-LIST-STRING-PAIR-CORE-001
    let source = include_str!("../../../../examples/language/list-string-pair-values.t");
    let expected = "((\"one\", \"first\"), (\"räv\", \"andra\"), true, true, true, true, 3, true, (\"seven\", \"seventh\"), (\"one\", \"first\"), (\"eleven\", \"elfte\"), Entry ( (\"one\", \"first\"), Entry ( (\"räv\", \"andra\"), Entry ( (\"\", \"最後\"), Empty ) ) ))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for event in [
        "list.entry.constructed",
        "list.entry.decomposed",
        "list.entry-count",
        "list.empty.tested",
        "equality.list",
    ] {
        assert!(trace.contains(event), "{event}: {trace}");
    }
}

#[test]
fn every_mode_preserves_infinities_across_private_boundaries() {
    // TOPAL-INTP-SUBSET-251, TOPAL-INTP-SUBSET-252,
    // TOPAL-NUM-INFINITY-001, TOPAL-NUM-INFINITY-ARITHMETIC-001
    let source = include_str!("../../../../examples/language/infinity-private-boundaries.t");
    let expected = "(+Infinity, +Infinity, -Infinity, +Infinity, (+Infinity, -Infinity), (integer is +Infinity, ratio is -Infinity), -Infinity, +Infinity, +Infinity)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
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
fn every_mode_declares_function_interfaces() {
    let source = include_str!("../../../../examples/language/function-interface.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8(output.stdout).unwrap().contains("true"));
    }
}

#[test]
fn every_mode_composes_capability_promises() {
    let source = include_str!("../../../../examples/language/capability-composition.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("Equality and Ordering")
        );
    }
}

#[test]
fn every_mode_accepts_broad_unicode_identifiers() {
    let source = include_str!("../../../../examples/language/unicode-identifiers.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
    }
}
