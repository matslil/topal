#[test]
fn closed_invalid_list_boundary_has_source_help() {
    let output = run(&[], "values : List Int is one 1\nvalues take 2\n");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("E-LIST-BOUNDARY-OUT-OF-RANGE"));
    assert!(stderr.contains("use a boundary no greater"));
}

#[test]
fn every_mode_collects_fundamental_containers() {
    let source = include_str!("../../../../examples/language/fundamental-containers.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Array (2, 1, 2)"));
        assert!(stdout.contains("Set (2, 1)"));
        assert!(stdout.contains("Bag ((2, 2), (1, 1))"));
        assert!(stdout.contains("Map ((\"Ada\", 11), (\"Lin\", 8))"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for rule in [
        "TOPAL-ARRAY-COLLECT-001",
        "TOPAL-SET-COLLECT-001",
        "TOPAL-BAG-COLLECT-001",
        "TOPAL-MAP-COLLECT-001",
    ] {
        assert!(trace.contains(rule), "missing {rule}");
    }
}

#[test]
fn every_mode_executes_recursive_products_variants_and_unions() {
    let source = include_str!("../../../../examples/language/unions-and-recursive-products.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("((10, (20, 30)), (0, (0, 0)), \"text\", \"number\")")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-TYPE-UNION-001"));
    assert!(trace.contains("TOPAL-TYPE-VARIANT-001"));
    assert!(trace.contains("TOPAL-DECISION-UNION-001"));
}

#[test]
fn every_mode_validates_constraints_and_derives_base_capabilities() {
    let source =
        include_str!("../../../../examples/language/constraints-and-derived-capabilities.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("(3, true, true, 5"));
        assert!(stdout.contains("domain is root.Positive(Int)"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-TYPE-CONSTRAINT-001"));
    assert!(trace.contains("TOPAL-TYPE-CONSTRAINT-VALIDATE-001"));
    assert!(trace.contains("constraint->base"));
}

#[test]
fn every_mode_validates_constraints_over_fundamental_bases() {
    // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMPILER-CONSTRAINT-FUNDAMENTAL-BASES-001
    let source = include_str!("../../../../examples/language/constraint-fundamental-bases.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("(true, \"Topal\", Rational ( 3, 2 ), true"));
        assert!(stdout.contains("domain is root.Pass(Boolean)"));
        assert!(stdout.contains("domain is root.Nonempty(String)"));
        assert!(stdout.contains("domain is root.PositiveRational(Rational)"));
        assert!(stdout.contains("domain is root.Small(Nat)"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-TYPE-CONSTRAINT-VALIDATE-001"));
}

#[test]
fn closed_constraint_rejection_has_source_help() {
    let output = run(
        &[],
        "Positive is Int constraint { value } value > 0\nPositive 0\n",
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-CONSTRAINT-REJECTED")
    );
}

#[test]
fn every_mode_composes_optional_result_and_error_fields() {
    let source = include_str!("../../../../examples/language/optional-result-composition.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("(4, 0, root./(Rational,Rational), division-by-zero"));
        assert!(stdout.contains("None, None, Some (line is"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-DECISION-OPTIONAL-001"));
    assert!(trace.contains("TOPAL-TYPE-RESULT-PROJECT-001"));
    assert!(trace.matches("TOPAL-ERROR-FIELD-001").count() >= 5);
}

#[test]
fn every_mode_executes_settled_modular_numbers() {
    let source = include_str!("../../../../examples/language/modular-numbers.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(
            "(ByteCounter 0, SignedByte -128, ByteCounter 255, SignedByte -128, true, true)"
        ));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for rule in [
        "TOPAL-NUM-MODULAR-TYPE-001",
        "TOPAL-NUM-MODULAR-CONSTRUCT-001",
        "TOPAL-NUM-MODULAR-REDUCE-001",
        "TOPAL-NUM-MODULAR-ARITHMETIC-001",
    ] {
        assert!(trace.contains(rule), "missing {rule}");
    }
}

#[test]
fn every_mode_checks_dynamic_modular_construction_from_a_named_range() {
    let source = include_str!("../../../../examples/language/modular-checked-construction.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("ByteCounter 255"));
        assert!(stdout.contains("domain is root.ByteCounter(Int)"));
        assert!(stdout.contains("code is out-of-range"));
        assert!(stdout.contains("Some (line is"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-NUM-MODULAR-TYPE-001"));
    assert!(trace.contains("TOPAL-NUM-MODULAR-CONSTRUCT-001"));
    assert!(trace.contains("TOPAL-ERROR-FIELD-001"));
}

#[test]
fn modular_function_boundaries_preserve_nominal_identity() {
    // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001
    let foreign = run(
        &[],
        "ByteCounter is ModNat (0 ..= 255)\nHour is ModNat (0 ..= 23)\nretain is fn (value : ByteCounter) -> ByteCounter\n  value\nretain (Hour 1)\n",
    );
    assert!(!foreign.status.success());
    let diagnostic = String::from_utf8(foreign.stderr).unwrap();
    assert!(
        diagnostic.contains("E-FUNCTION-ARGUMENT-TYPE"),
        "{diagnostic}"
    );
}

#[test]
fn closed_modular_construction_rejection_is_source_located() {
    let output = run(&[], "Byte is ModNat (0 ..= 255)\nByte 256\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-MODULAR-OUT-OF-RANGE")
    );
}

#[test]
fn every_mode_selects_values_and_indexes_by_range() {
    let source = include_str!("../../../../examples/language/range-selection.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(
            "(Entry ( 2, Entry ( 4, Entry ( 3, Empty ) ) ), Entry ( 2, Entry ( 4, Entry ( 7, Empty ) ) ), \"opa\")"
        ));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-RANGE-VALUE-SELECTION-001"));
    assert!(trace.contains("TOPAL-RANGE-INDEX-SELECTION-001"));
}

#[test]
fn every_mode_returns_explicit_completion_evidence() {
    let source = include_str!("../../../../examples/language/completed-evidence.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("Completed")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-EXEC-COMPLETED-001"));
    assert!(trace.contains("function.exit"));
}

#[test]
fn every_mode_reconstructs_records_immutably() {
    let source = include_str!("../../../../examples/language/record-reconstruction.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(36, \"Ada\", 37)")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-TYPE-RECONSTRUCT-001"));
}

#[test]
fn every_mode_preserves_record_function_boundaries() {
    // TOPAL-COMPILER-RECORD-BOUNDARY-001, TOPAL-TYPE-PRODUCT-001
    let source = include_str!("../../../../examples/language/record-function-boundaries.t");
    let expected = b"((name is \"Ada\", active is true), (active is false, name is \"Grace\"), (active is true, name is \"first\"), (name is \"second\", active is false), (score is 42, person is (name is \"Lin\", active is true)), \"Grace\", (value is -2, label is \"negative\"), (label is \"nonnegative\", value is 2), (label is \"less\", value is -1), (label is \"right\", value is 1), (value is 7, label is \"some\"), (label is \"none\", value is 0), (label is \"ok\", value is Rational ( 1, 2 )), (value is Rational ( 0, 1 ), label is \"error\"))\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(expected));
    }
}

#[test]
fn every_mode_passes_bound_anonymous_function_values() {
    let source = include_str!("../../../../examples/language/bound-anonymous-functions.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Entry ( 2, Entry ( 4, Entry ( 6, Empty ) ) )"));
        assert!(stdout.contains(", 6)"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(trace.matches("binding.resolved").count() >= 3);
}

#[test]
fn every_mode_directly_applies_anonymous_function_values() {
    let source = include_str!("../../../../examples/language/anonymous-function-application.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(42, 42)")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("function.anonymous.called").count(), 2);
}

#[test]
fn every_mode_short_circuits_fold_with_traversal_control() {
    let source = include_str!("../../../../examples/language/traversal-control.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(1, (Continue 1, Finish 2))")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("function.anonymous.called").count(), 1);
    assert!(trace.contains("traversal.finished"));
    assert!(trace.contains("TOPAL-EXEC-TRAVERSAL-CONTROL-001"));
}

#[test]
fn every_mode_applies_bound_symbolic_callable_values() {
    let source = include_str!("../../../../examples/language/callable-values.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(42, -5, Less)")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("function.callable.captured").count(), 3);
    assert_eq!(trace.matches("function.callable.called").count(), 3);
}

#[test]
fn every_mode_applies_the_complete_symbolic_callable_vocabulary() {
    let source = include_str!("../../../../examples/language/expanded-callable-values.t");
    let expected = "(true, true, true, true, true, true, 42, Rational ( 3, 4 ), (3, 2), 3, 1024, 0 .. 3, 0 <.. 3, 0 ..= 3, 0 <..= 3, 42)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("function.callable.called").count(), 16);
    assert!(trace.contains("TOPAL-FUNCTION-CALLABLE-VALUE-001"));
    assert!(trace.contains("TOPAL-TYPE-CALL-001"));
}

#[test]
fn every_mode_applies_bound_named_function_values() {
    let source = include_str!("../../../../examples/language/named-function-values.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-VALUE-001"));
    assert!(trace.contains("function.entry"));
}

#[test]
fn every_mode_constructs_lazy_iterate_generators() {
    let source = include_str!("../../../../examples/language/iterate-generator.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("<Generator Int Unit Unit>")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-ITERATE-001"));
    assert!(!trace.contains("function.anonymous.called"));
}

#[test]
fn every_mode_constructs_lazy_take_while_prefixes() {
    let source = include_str!("../../../../examples/language/iterate-take-while.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("<Generator Int Unit Unit>")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-TAKE-WHILE-001"));
    assert!(!trace.contains("function.anonymous.called"));
}

#[test]
fn every_mode_traverses_bounded_generated_prefixes() {
    let source = include_str!("../../../../examples/language/generated-foreach.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains("()"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(
        trace.matches("TOPAL-GENERATOR-ITERATE-FOREACH-001").count(),
        11
    );
    assert!(trace.contains("TOPAL-GENERATOR-TAKE-WHILE-001"));
}

#[test]
fn every_mode_collects_finite_generated_traversals() {
    let source = include_str!("../../../../examples/language/generated-collect.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("Entry ( 0, Entry ( 1, Entry ( 2, Entry ( 3, Entry ( 4, Empty ) ) ) ) )")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-COLLECT-001"));
}

#[test]
fn every_mode_constructs_lazy_unfold_generators() {
    let source = include_str!("../../../../examples/language/unfold-generator.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("<Generator Int Unit Unit>")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-UNFOLD-001"));
    assert!(!trace.contains("function.anonymous.called"));
}

#[test]
fn every_mode_collects_finite_unfold_generators() {
    let source = include_str!("../../../../examples/language/unfold-collect.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("Entry ( 4, Entry ( 5, Entry ( 6, Empty ) ) )")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("generator.yielded").count(), 3);
    assert!(trace.contains("TOPAL-GENERATOR-UNFOLD-COLLECT-001"));
}

#[test]
fn every_mode_resolves_the_root_namespace_explicitly() {
    let source = include_str!("../../../../examples/language/root-namespace.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(<namespace root>, 42)")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-NAMESPACE-ROOT-001"));
    assert!(trace.contains("namespace.member.resolved"));
}

#[test]
fn every_mode_resolves_members_through_namespace_aliases() {
    let source = include_str!("../../../../examples/language/namespace-alias.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-NAMESPACE-ALIAS-001"));
    assert!(trace.contains("namespace.alias.member.resolved"));
}

#[test]
fn every_mode_makes_namespaces_available_with_use() {
    let source = include_str!("../../../../examples/language/use-namespace.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
    }
    assert!(
        String::from_utf8(run(&["--test"], source).stderr)
            .unwrap()
            .contains("TOPAL-NAMESPACE-USE-001")
    );
}

#[test]
fn every_mode_preserves_namespace_capture_visibility() {
    let source = include_str!("../../../../examples/language/namespace-snapshot.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(41, 42)")
        );
    }
}

#[test]
fn every_mode_preserves_namespace_overload_sets() {
    let source = include_str!("../../../../examples/language/namespace-overloads.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(42, \"Topal\")")
        );
    }
    assert_eq!(
        String::from_utf8(run(&["--test"], source).stderr)
            .unwrap()
            .matches("function.overload.selected")
            .count(),
        2
    );
}

#[test]
fn every_mode_applies_qualified_namespace_generators() {
    let source = include_str!("../../../../examples/language/namespace-generator.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-DECLARATION-001"));
    assert!(trace.contains("namespace.alias.member.resolved"));
}

#[test]
fn every_mode_classifies_namespaces_as_scope() {
    let source = include_str!("../../../../examples/language/scope-classifier.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
    }
}

#[test]
fn missing_namespace_member_suggests_only_a_member() {
    let output = run(&[], "answer is 42\napi is root\napi answr\n");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("E-NAMESPACE-MEMBER-NOT-FOUND"));
    assert!(stderr.contains("did you mean `answer`?"));
}

#[test]
fn every_mode_preserves_namespace_alias_chains() {
    let source = include_str!("../../../../examples/language/namespace-alias-chain.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
    }
}

#[test]
fn every_mode_passes_namespaces_through_scope_parameters() {
    let source = include_str!("../../../../examples/language/namespace-function-parameter.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
    }
}

#[test]
fn every_mode_resolves_fundamental_type_values() {
    let source = include_str!("../../../../examples/language/type-values.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    assert_eq!(
        String::from_utf8(run(&["--test"], source).stderr)
            .unwrap()
            .matches("TOPAL-ABSTRACTION-TYPE-VALUE-001")
            .count(),
        7
    );
}

#[test]
fn every_mode_compares_type_identity() {
    let source = include_str!("../../../../examples/language/type-identity.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(
            String::from_utf8(run(arguments, source).stdout)
                .unwrap()
                .contains("(true, false)")
        );
    }
}

#[test]
fn every_mode_classifies_type_values() {
    let source = include_str!("../../../../examples/language/type-classifier.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(
            String::from_utf8(run(arguments, source).stdout)
                .unwrap()
                .contains("Int")
        );
    }
}

#[test]
fn every_mode_classifies_function_values() {
    let source = include_str!("../../../../examples/language/function-classifier.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(
            String::from_utf8(run(arguments, source).stdout)
                .unwrap()
                .contains("42")
        );
    }
}

#[test]
fn every_mode_classifies_constraint_values() {
    let source = include_str!("../../../../examples/language/constraint-classifier.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_passes_type_values_through_functions() {
    let source = include_str!("../../../../examples/language/type-function-boundary.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(
            String::from_utf8(run(arguments, source).stdout)
                .unwrap()
                .contains("Int")
        );
    }
}

#[test]
fn every_mode_passes_callable_values_through_functions() {
    let source = include_str!("../../../../examples/language/function-value-boundary.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(
            String::from_utf8(run(arguments, source).stdout)
                .unwrap()
                .contains("42")
        );
    }
}

#[test]
fn every_mode_returns_callable_values_from_functions() {
    let source = include_str!("../../../../examples/language/function-results.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(42, 42)")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-VALUE-001"));
    assert!(trace.contains("TOPAL-FUNCTION-CALLABLE-VALUE-001"));
}

#[test]
fn every_mode_preserves_anonymous_captures_and_function_results() {
    let source = include_str!("../../../../examples/language/anonymous-function-captures.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(42, 42)")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(trace.contains("function.anonymous.captured"));
    assert!(trace.contains("function.anonymous.called"));
}

#[test]
fn every_mode_forwards_captured_function_parameters() {
    let source = include_str!("../../../../examples/language/capturing-function-parameters.t");
    let expected = "(42, 42, (7, \"seven\"), 42)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(trace.contains("\"event\":\"function.declared\",\"rule\":\"TOPAL-FUNCTION-ORDINARY-001\",\"detail\":\"add\""));
    assert!(trace.matches("function.anonymous.called").count() >= 3);
    assert!(trace.matches("function.value.called").count() >= 4);
}

#[test]
fn every_mode_returns_captured_functions() {
    let source = include_str!("../../../../examples/language/capturing-function-results.t");
    let expected = "(42, (7, \"seven\"), 42, 42)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(trace.matches("function.anonymous.captured").count() >= 4);
    assert!(trace.matches("function.anonymous.called").count() >= 4);
    assert!(trace.matches("function.value.called").count() >= 1);
}

#[test]
fn every_mode_transports_captures_through_function_aggregates() {
    let source =
        include_str!("../../../../examples/language/capturing-function-aggregate-boundaries.t");
    let expected = "((42, 40), (42, 40), 42, 42)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(trace.contains("function.anonymous.captured"));
    assert!(trace.matches("function.anonymous.called").count() >= 5);
    assert!(trace.matches("function.value.called").count() >= 1);
}

#[test]
fn every_mode_applies_exact_function_result_chains() {
    let source = include_str!("../../../../examples/language/function-result-chains.t");
    let expected = "(42, 42, (7, \"seven\"), 42, 42, 42)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-VALUE-001"));
    assert!(trace.contains("TOPAL-FUNCTION-CALLABLE-VALUE-001"));
    assert!(trace.matches("function.anonymous.called").count() >= 4);
    assert!(trace.matches("function.value.called").count() >= 4);
}

#[test]
fn every_mode_destructures_nested_anonymous_patterns() {
    let source = include_str!("../../../../examples/language/nested-anonymous-patterns.t");
    let expected = "(42, 42, 42, 42, 42)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(trace.matches("function.anonymous.called").count() >= 5);
    assert!(trace.contains("pattern.identity.matched"));
}

#[test]
fn every_mode_preserves_function_values_inside_aggregates() {
    let source = include_str!("../../../../examples/language/function-aggregate-boundaries.t");
    let expected = "(42, 42, 42, 42)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-VALUE-001"));
    assert!(trace.contains("function.value.called"));
    assert!(trace.contains("function.callable.called"));
    assert!(trace.matches("function.anonymous.called").count() >= 3);
}

#[test]
fn every_mode_destructures_bound_and_returned_anonymous_product_functions() {
    let source = include_str!("../../../../examples/language/anonymous-product-functions.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("(42, 42, 42, 42)")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(trace.contains("function.anonymous.captured"));
    assert!(trace.contains("function.anonymous.called"));
}

#[test]
fn every_mode_requires_exact_repeated_anonymous_pattern_identity() {
    let source = include_str!("../../../../examples/language/repeated-anonymous-patterns.t");
    let expected = "(42, 20, 7, \"same\", 42)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("pattern.identity.matched").count(), 5);
    assert!(trace.contains("TOPAL-TYPE-MATCH-001"));

    let mismatch = run(
        &[],
        "use language (version is v0.1)\noperation : Function is { (value, value) } value\noperation (1, 1.0)\n",
    );
    assert!(!mismatch.status.success());
    let diagnostic = String::from_utf8(mismatch.stderr).unwrap();
    assert!(diagnostic.contains("error[E-ANONYMOUS-PATTERN-IDENTITY]"));
    assert!(diagnostic.contains("same exact value"));
    assert!(diagnostic.contains("pass the same exact value"));
}

#[test]
fn every_mode_requires_exact_repeated_anonymous_aggregate_identity() {
    let source =
        include_str!("../../../../examples/language/repeated-anonymous-aggregate-patterns.t");
    let expected = "((7, \"seven\"), (active is true, name is \"Ada\"), Some 9, Entry ( 1, Entry ( 2, Empty ) ))";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains(expected), "{arguments:?}: {stdout}");
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("pattern.identity.matched").count(), 4);
    assert!(trace.contains("TOPAL-TYPE-MATCH-001"));

    let mismatch = run(
        &[],
        "use language (version is v0.1)\noperation : Function is { value, value } value\noperation ((1, \"same\"), (2, \"same\"))\n",
    );
    assert!(!mismatch.status.success());
    let diagnostic = String::from_utf8(mismatch.stderr).unwrap();
    assert!(diagnostic.contains("error[E-ANONYMOUS-PATTERN-IDENTITY]"));
    assert!(diagnostic.contains("same exact value"));
}

#[test]
fn every_mode_requires_exact_repeated_sum_identity() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-SUM-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let source = include_str!("../../../../examples/language/repeated-sum-patterns.t");
    let expected = "(Stop, Number 7, Label \"seven\", Pair (7, \"seven\"), Wrapped Code 9)";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains(expected), "{arguments:?}: {stdout}");
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("pattern.identity.matched").count(), 5);
    assert!(trace.contains("TOPAL-TYPE-MATCH-001"));

    for right in ["Number 8", "Label \"seven\""] {
        let mismatch = run(
            &[],
            &format!(
                "use language (version is v0.1)\nToken is Union\n  Number : Int\n  Label : String\n\nrepeat : Function is {{ value, value }} value\nleft : Token is Number 7\nright : Token is {right}\nrepeat (left, right)\n"
            ),
        );
        assert!(!mismatch.status.success());
        let diagnostic = String::from_utf8(mismatch.stderr).unwrap();
        assert!(diagnostic.contains("error[E-ANONYMOUS-PATTERN-IDENTITY]"));
        assert!(diagnostic.contains("same exact value"));
    }
}

#[test]
fn every_mode_requires_exact_repeated_function_aggregate_identity() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let source =
        include_str!("../../../../examples/language/repeated-function-aggregate-patterns.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("(42, 42, 42)"), "{arguments:?}: {stdout}");
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("pattern.identity.matched").count(), 3);
    assert!(trace.contains("TOPAL-TYPE-MATCH-001"));

    let mismatch = run(
        &[],
        "use language (version is v0.1)\nleft is (operation is +, value is 1)\nright is (operation is -, value is 1)\nrepeat : Function is { value, value } 42\nrepeat (left, right)\n",
    );
    assert!(!mismatch.status.success());
    let diagnostic = String::from_utf8(mismatch.stderr).unwrap();
    assert!(diagnostic.contains("error[E-ANONYMOUS-PATTERN-IDENTITY]"));
    assert!(diagnostic.contains("same exact value"));
}

#[test]
fn every_mode_requires_exact_repeated_captured_function_identity() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let source =
        include_str!("../../../../examples/language/repeated-captured-function-patterns.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("(42, 42)"), "{arguments:?}: {stdout}");
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("pattern.identity.matched").count(), 2);
    assert!(trace.contains("TOPAL-TYPE-MATCH-001"));

    let mismatch = run(
        &[],
        "use language (version is v0.1)\nmake-operation is fn (offset : Int) -> Function\n  operation : Function is { value } value + offset\n  operation\ncompare-captures is fn (left : Int, right : Int) -> Int\n  repeat : Function is { operation, operation } 42\n  repeat (make-operation left, make-operation right)\ncompare-captures (1, 2)\n",
    );
    assert!(!mismatch.status.success());
    let diagnostic = String::from_utf8(mismatch.stderr).unwrap();
    assert!(diagnostic.contains("error[E-ANONYMOUS-PATTERN-IDENTITY]"));
    assert!(diagnostic.contains("same exact value"));
}

#[test]
fn every_mode_requires_exact_repeated_captured_named_function_identity() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-NESTED-001
    let source =
        include_str!("../../../../examples/language/repeated-captured-named-function-patterns.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("42"), "{arguments:?}: {stdout}");
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("pattern.identity.matched").count(), 1);
    assert!(trace.contains("TOPAL-TYPE-MATCH-001"));

    let mismatch = run(
        &[],
        "use language (version is v0.1)\ncompare is fn (offset : Int) -> Int\n  left is fn (value : Int) -> Int\n    value + offset\n  right is fn (value : Int) -> Int\n    value + offset\n  repeat : Function is { operation, operation } 42\n  repeat (left, right)\ncompare 1\n",
    );
    assert!(!mismatch.status.success());
    let diagnostic = String::from_utf8(mismatch.stderr).unwrap();
    assert!(diagnostic.contains("error[E-ANONYMOUS-PATTERN-IDENTITY]"));
    assert!(diagnostic.contains("same exact value"));
}

#[test]
fn every_mode_requires_exact_repeated_captured_function_aggregate_identity() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let source = include_str!(
        "../../../../examples/language/repeated-captured-function-aggregate-patterns.t"
    );
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("(42, 42, 42)"), "{arguments:?}: {stdout}");
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("pattern.identity.matched").count(), 3);
    assert!(trace.contains("TOPAL-TYPE-MATCH-001"));

    let mismatch = run(
        &[],
        "use language (version is v0.1)\nmake is fn (offset : Int) -> Record (operation : Function, value : Int)\n  operation : Function is { value } value + offset\n  (operation is operation, value is 41)\ncompare is fn (left : Int, right : Int) -> Int\n  repeat : Function is { package, package } 42\n  repeat (make left, make right)\ncompare (1, 2)\n",
    );
    assert!(!mismatch.status.success());
    let diagnostic = String::from_utf8(mismatch.stderr).unwrap();
    assert!(diagnostic.contains("error[E-ANONYMOUS-PATTERN-IDENTITY]"));
    assert!(diagnostic.contains("same exact value"));
}

#[test]
fn every_mode_constructs_empty_effect_rows() {
    let source = include_str!("../../../../examples/language/empty-effects.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_classifies_effect_rows() {
    let source = include_str!("../../../../examples/language/effect-classifier.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_compares_effect_rows() {
    let source = include_str!("../../../../examples/language/effect-identity.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(
            String::from_utf8(run(arguments, source).stdout)
                .unwrap()
                .contains("true")
        );
    }
}

#[test]
fn every_mode_passes_effect_values_through_functions() {
    let source = include_str!("../../../../examples/language/effect-function-boundary.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_packages_effect_products() {
    let source = include_str!("../../../../examples/language/effect-products.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_retains_effect_lists() {
    let source = include_str!("../../../../examples/language/effect-list.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_composes_completion_with_effect_values() {
    let source = include_str!("../../../../examples/language/completion-effect-value.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_keeps_unit_distinct_beside_effect_values() {
    let source = include_str!("../../../../examples/language/unit-effect-value.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_resolves_layout_endian_policies() {
    let source = include_str!("../../../../examples/language/layout-endian.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_resolves_layout_access_policies() {
    let source = include_str!("../../../../examples/language/layout-access.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_resolves_layout_bit_order() {
    let source = include_str!("../../../../examples/language/layout-bit-order.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_resolves_layout_packing() {
    let source = include_str!("../../../../examples/language/layout-packing.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_resolves_declared_field_order() {
    let source = include_str!("../../../../examples/language/layout-field-order.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_resolves_payload_placement() {
    let source = include_str!("../../../../examples/language/layout-payload-placement.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_resolves_layout_absence_policies() {
    let source = include_str!("../../../../examples/language/layout-absence-policies.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn every_mode_accepts_balanced_diagnostic_controls() {
    let source = include_str!("../../../../examples/language/diagnostic-controls.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains("42"));
    }
}

#[test]
fn every_mode_evaluates_empty_blocks() {
    let source = include_str!("../../../../examples/language/empty-block.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
}

#[test]
fn script_and_test_modes_load_directory_applications() {
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/applications/module-loading");
    for arguments in [&[][..], &["--test"][..]] {
        let output = Command::new(env!("CARGO_BIN_EXE_topal"))
            .args(arguments)
            .arg(&directory)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "42");
    }
}

#[test]
fn script_and_test_modes_execute_the_shared_standard_library_example() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
    for arguments in [&[][..], &["--test"][..]] {
        let output = Command::new(env!("CARGO_BIN_EXE_topal"))
            .args(arguments)
            .arg(&directory)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            "(2, Rational ( 9, 2 ), (1, 2), (3, 7), -1, -1, 9, Rational ( 3, 2 ), Some 5, true, Some (2, \"items\"), Rational ( 5, 1 ), 6, true, Rational ( 1, 2 ), 6, 6, (-2, 5), 0 ..= 8, \"text\", \"abab\", true, Some 2, Entry ( 2, Empty ), Entry ( 1, Entry ( 1, Entry ( 2, Entry ( 2, Entry ( 3, Entry ( 3, Empty ) ) ) ) ) ), Entry ( 3, Entry ( 4, Entry ( 5, Empty ) ) ))"
        );
    }
}

#[test]
fn script_and_test_modes_preserve_nested_module_paths() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/applications/nested-module-loading");
    for arguments in [&[][..], &["--test"][..]] {
        let output = Command::new(env!("CARGO_BIN_EXE_topal"))
            .args(arguments)
            .arg(&directory)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "42");
    }
}

#[test]
fn every_mode_selects_defining_context_members() {
    let source = include_str!("../../../../examples/language/constructed-context.t");
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

#[test]
fn every_mode_forwards_defining_context_members() {
    // TOPAL-INTP-SUBSET-268, TOPAL-CONTEXT-SELECT-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001
    let source = include_str!("../../../../examples/language/defining-context-forwarding.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("(40, 2, \"ready\")"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_forwards_recursive_scalar_environments() {
    // TOPAL-INTP-SUBSET-269,
    // TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001
    let source = include_str!("../../../../examples/language/recursive-scalar-environments.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("((false, 0, 2), (true, 40, 0))"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_forwards_aggregate_environments() {
    // TOPAL-INTP-SUBSET-270,
    // TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001
    let source = include_str!("../../../../examples/language/aggregate-environments.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(
                "((40, \"context\"), (amount is 2, enabled is true), (7, \"root\"), (amount is 9, enabled is false), Label \"context-sum\", Number 11)"
            ),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_forwards_overload_selected_environments() {
    // TOPAL-INTP-SUBSET-271,
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001
    let source = include_str!("../../../../examples/language/overload-environments.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(
                "(40, \"context\", (2, \"context-pair\"), (7, \"root-pair\"), 7, \"root\", 47, 40, \"root\")"
            ),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_forwards_local_named_function_environments() {
    // TOPAL-INTP-SUBSET-272,
    // TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001
    let source = include_str!("../../../../examples/language/local-function-environments.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(
                "((42, \"context\", 9, \"root\", (2, \"context-pair\"), (7, \"root-pair\")), (42, 10))"
            ),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_forwards_function_environments_across_value_boundaries() {
    // TOPAL-INTP-SUBSET-273,
    // TOPAL-COMPILER-FUNCTION-ENVIRONMENT-BOUNDARY-001
    let source = include_str!("../../../../examples/language/function-environment-boundaries.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(
                "(42, \"context\", 9, \"root\", (2, \"context-pair\"), (7, \"root-pair\"), (42, \"context\", 9, \"root\", (2, \"context-pair\"), (7, \"root-pair\")), 48, 49, (49, 50))"
            ),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_preserves_escaping_nested_function_environments() {
    // TOPAL-INTP-SUBSET-274,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001
    let source =
        include_str!("../../../../examples/language/escaping-nested-function-environments.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("(43, 44, 45, 46, 47, (7, \"seven\"))"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn every_mode_preserves_optional_function_environments() {
    // TOPAL-INTP-SUBSET-275,
    // TOPAL-COMPILER-OPTIONAL-FUNCTION-001
    let source = include_str!("../../../../examples/language/optional-function-environments.t");
    let expected = "(43, 44, 45, 5, 42, Some +, 46, 47, 48, 0, 1, Some <fn increase>, None)";
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
    assert!(trace.contains("pattern.identity.matched"));
    assert!(trace.contains(
        "\"event\":\"function.value.called\",\"rule\":\"TOPAL-FUNCTION-VALUE-001\",\"detail\":\"increase\""
    ));

    let mismatch = run(
        &[],
        "use language (version is v0.1)\nmake is fn (offset : Int) -> Optional Function\n  increase is fn (value : Int) -> Int\n    value + offset\n  Some increase\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 2)\n",
    );
    assert!(!mismatch.status.success());
    assert!(
        String::from_utf8_lossy(&mismatch.stderr).contains("error[E-ANONYMOUS-PATTERN-IDENTITY]")
    );
}

#[test]
fn every_mode_preserves_sum_function_environments() {
    // TOPAL-INTP-SUBSET-276,
    // TOPAL-COMPILER-SUM-FUNCTION-001
    let source = include_str!("../../../../examples/language/sum-function-environments.t");
    let expected =
        "(43, 44, 45, 5, 42, Apply +, 46, 47, 48, 0, 1, Apply <fn increase>, Unavailable)";
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
    assert!(trace.contains("pattern.identity.matched"));
    assert!(trace.contains("union.payload.bound"));
    assert!(trace.contains(
        "\"event\":\"function.value.called\",\"rule\":\"TOPAL-FUNCTION-VALUE-001\",\"detail\":\"increase\""
    ));

    let mismatch = run(
        &[],
        "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nmake is fn (offset : Int) -> Operation\n  increase is fn (value : Int) -> Int\n    value + offset\n  Apply increase\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 2)\n",
    );
    assert!(!mismatch.status.success());
    assert!(
        String::from_utf8_lossy(&mismatch.stderr).contains("error[E-ANONYMOUS-PATTERN-IDENTITY]")
    );
}

#[test]
fn every_mode_preserves_result_function_environments() {
    // TOPAL-INTP-SUBSET-277,
    // TOPAL-COMPILER-RESULT-FUNCTION-001
    let source = include_str!("../../../../examples/language/result-function-environments.t");
    let expected = "(43, 44, 45, 5, 42, +, 46, 47, 48, 49, 49, 0, 0, 0, <fn increase>, Error ( domain is root./(Rational,Rational), code is division-by-zero ))";
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
    assert!(trace.contains("result.success.projected"));
    assert!(trace.contains("result.error.constructed"));
    assert!(trace.contains("result.payload.bound"));
    assert!(trace.contains(
        "\"event\":\"function.value.called\",\"rule\":\"TOPAL-FUNCTION-VALUE-001\",\"detail\":\"increase\""
    ));
}

#[test]
fn every_mode_preserves_list_function_environments() {
    // TOPAL-INTP-SUBSET-278,
    // TOPAL-COMPILER-LIST-FUNCTION-001
    let source = include_str!("../../../../examples/language/list-function-environments.t");
    let expected = "(43, 44, 45, 5, 42, Entry ( +, Empty ), 46, 47, 48, 1, 0, Entry ( <fn increment>, Entry ( <fn increase>, Empty ) ), Empty)";
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
    assert!(trace.contains(
        "\"event\":\"function.value.called\",\"rule\":\"TOPAL-FUNCTION-VALUE-001\",\"detail\":\"increase\""
    ));
}

#[test]
fn every_mode_preserves_array_function_environments() {
    // TOPAL-INTP-SUBSET-279,
    // TOPAL-COMPILER-ARRAY-FUNCTION-001
    let source = include_str!("../../../../examples/language/array-function-environments.t");
    let expected = "(2, 43, 44, 45, 5, 42, Array (+), 46, 47, 48, 2, 0, true, Array (<fn increment>, <fn increase>), Array ())";
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
    assert!(trace.contains("array.collected"));
    assert!(trace.contains("array.checked-access"));
    assert!(trace.contains(
        "\"event\":\"function.value.called\",\"rule\":\"TOPAL-FUNCTION-VALUE-001\",\"detail\":\"increase\""
    ));
}

#[test]
fn every_mode_preserves_map_function_environments() {
    // TOPAL-INTP-SUBSET-280,
    // TOPAL-COMPILER-MAP-FUNCTION-001
    let source = include_str!("../../../../examples/language/map-function-environments.t");
    let expected = "(2, 43, 44, 45, 5, 42, 3, 46, 47, 48, 2, 8, 2, 0, false, Map ((\"increment\", <fn increment>), (\"increase\", <fn increase>)))";
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
    assert!(trace.contains("map.collected"));
    assert!(trace.contains("map.lookup"));
    assert!(trace.contains(
        "\"event\":\"function.value.called\",\"rule\":\"TOPAL-FUNCTION-VALUE-001\",\"detail\":\"increase\""
    ));
}

#[test]
fn every_mode_preserves_boolean_list_values() {
    // TOPAL-INTP-SUBSET-281, TOPAL-COMPILER-LIST-BOOLEAN-001
    let source = include_str!("../../../../examples/language/list-boolean-values.t");
    let expected = "(true, false, true, true, 2, true, true, true, false, Entry ( true, Entry ( false, Empty ) ))";
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
}

#[test]
fn every_mode_preserves_string_list_values() {
    // TOPAL-INTP-SUBSET-282, TOPAL-COMPILER-LIST-STRING-CORE-001
    let source = include_str!("../../../../examples/language/list-string-values.t");
    let expected = "(\"Top\", \"al\", true, true, 2, true, \"package\", \"Top\", \"record\", Entry ( \"Top\", Entry ( \"al\", Empty ) ))";
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
