#[test]
fn every_mode_returns_from_direct_named_constraint_arguments() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRAINT-001
    let source =
        include_str!("../../../../examples/language/function-return-constraint-constructor.t");
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

#[test]
fn every_mode_returns_from_direct_named_modular_arguments() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-NUM-MODULAR-CONSTRUCT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-001
    let source =
        include_str!("../../../../examples/language/function-return-modular-constructor.t");
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
            assert!(!trace.contains("numeric.modular.constructed"));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_modular_reduction_operands() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-NUM-MODULAR-REDUCE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001
    let source = include_str!("../../../../examples/language/function-return-modular-reduction.t");
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
            assert!(!trace.contains("numeric.modular.reduced"));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_list_collect_sources() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COLLECTION-COLLECT-LIST-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COLLECT-001
    let source = include_str!("../../../../examples/language/function-return-list-collect.t");
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
            assert!(!trace.contains("list.collected"));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_unordered_collect_sources() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-SET-COLLECT-001, TOPAL-BAG-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001
    let source = include_str!("../../../../examples/language/function-return-unordered-collect.t");
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
            assert_eq!(trace.matches("function.return.explicit").count(), 2);
            assert!(!trace.contains("set.collected"));
            assert!(!trace.contains("bag.collected"));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_map_collect_sources() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-MAP-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001
    let source = include_str!("../../../../examples/language/function-return-map-collect.t");
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
            assert!(!trace.contains("map.collected"));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_infix_collect_sources() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-ARRAY-COLLECT-001, TOPAL-COLLECTION-COLLECT-STRING-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-INFIX-COLLECT-001
    let source = include_str!("../../../../examples/language/function-return-infix-collect.t");
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
            assert_eq!(trace.matches("function.return.explicit").count(), 2);
            assert!(!trace.contains("array.collected"));
            assert!(!trace.contains("string.collected"));
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"abandoned\""));
        }
    }
}

#[test]
fn every_mode_returns_from_direct_boolean_decision_subjects() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-DECISION-SUBJECT-001
    let source = include_str!("../../../../examples/language/function-return-decision-subject.t");
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
            assert!(!trace.contains("decision.rule"));
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_exhaustive_boolean_decision_subjects() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-EXHAUSTIVE-BOOLEAN-SUBJECT-001
    let source =
        include_str!("../../../../examples/language/function-return-exhaustive-boolean-subject.t");
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
            assert!(!trace.contains("decision.rule"));
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_comparison_decision_subjects() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-COMPARISON-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-DECISION-SUBJECT-001
    let source =
        include_str!("../../../../examples/language/function-return-comparison-decision-subject.t");
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
            assert!(!trace.contains("decision.rule"));
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_fallback_decision_subjects() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-COMPILER-LEXICAL-RETURN-FALLBACK-DECISION-SUBJECT-001
    let source =
        include_str!("../../../../examples/language/function-return-fallback-decision-subject.t");
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
            assert!(!trace.contains("decision.rule"));
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_complete_decision_subjects() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-OPTIONAL-001, TOPAL-DECISION-RESULT-001,
    // TOPAL-DECISION-LIST-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPLETE-DECISION-SUBJECT-001
    let source =
        include_str!("../../../../examples/language/function-return-complete-decision-subject.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(40, 41, 42)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 3);
            assert!(!trace.contains("decision.rule"));
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_complete_boolean_decision_actions() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-BOOLEAN-DECISION-ACTIONS-001
    let source =
        include_str!("../../../../examples/language/function-return-boolean-decision-actions.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(40, 41)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 2);
            assert_eq!(trace.matches("decision.rule.selected").count(), 2);
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_complete_comparison_value_decision_actions() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-ENUM-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-VALUE-DECISION-ACTIONS-001
    let source = include_str!(
        "../../../../examples/language/function-return-comparison-value-decision-actions.t"
    );
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(40, 41, 42)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 3);
            assert_eq!(trace.matches("decision.rule.selected").count(), 3);
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_complete_ordered_comparison_decision_actions() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-COMPARISON-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ORDERED-COMPARISON-DECISION-ACTIONS-001
    let source = include_str!(
        "../../../../examples/language/function-return-ordered-comparison-decision-actions.t"
    );
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(40, 41, 42)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 3);
            assert_eq!(trace.matches("decision.rule.selected").count(), 3);
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_final_fallback_enum_decision_actions() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-ENUM-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ENUM-FALLBACK-DECISION-ACTIONS-001
    let source = include_str!(
        "../../../../examples/language/function-return-enum-fallback-decision-actions.t"
    );
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(40, 41, 42)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 3);
            assert_eq!(trace.matches("decision.rule.selected").count(), 3);
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_exhaustive_enum_decision_actions() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-ENUM-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ENUM-EXHAUSTIVE-DECISION-ACTIONS-001
    let source = include_str!(
        "../../../../examples/language/function-return-exhaustive-enum-decision-actions.t"
    );
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(43, 44, 45)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 3);
            assert_eq!(trace.matches("decision.rule.selected").count(), 3);
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_optional_decision_actions() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-OPTIONAL-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-DECISION-ACTIONS-001
    let source =
        include_str!("../../../../examples/language/function-return-optional-decision-actions.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(43, 40, 44, 45)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 4);
            assert_eq!(trace.matches("decision.rule.selected").count(), 4);
            assert_eq!(trace.matches("optional.payload.bound").count(), 1);
            assert!(!trace.contains("\"detail\":\"1000\""));
            assert!(!trace.contains("\"detail\":\"1001\""));
        }
    }
}

#[test]
fn every_mode_returns_from_result_decision_actions() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-RESULT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-RESULT-DECISION-ACTIONS-001
    let source =
        include_str!("../../../../examples/language/function-return-result-decision-actions.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(true, true)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 2);
            assert_eq!(trace.matches("decision.rule.selected").count(), 2);
            assert_eq!(trace.matches("result.payload.bound").count(), 2);
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_error_code_decision_actions() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-ERROR-CODE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ERROR-CODE-DECISION-ACTIONS-001
    let source =
        include_str!("../../../../examples/language/function-return-error-code-decision-actions.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(true, false, true, 0, 3, 4)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 6);
            assert_eq!(trace.matches("decision.rule.selected").count(), 6);
            assert_eq!(trace.matches("error.code.matched").count(), 3);
            assert_eq!(trace.matches("result.payload.bound").count(), 3);
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_returns_from_list_decision_actions() {
    // TOPAL-INTP-SUBSET-039, TOPAL-INTP-SUBSET-241,
    // TOPAL-DECISION-LIST-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-LIST-DECISION-ACTIONS-001
    let source =
        include_str!("../../../../examples/language/function-return-list-decision-actions.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(43, 40)\n"));
        if arguments == ["--test"] {
            let trace = String::from_utf8(output.stderr).unwrap();
            assert_eq!(trace.matches("function.return.explicit").count(), 2);
            assert_eq!(trace.matches("decision.rule.selected").count(), 2);
            assert_eq!(trace.matches("list.entry.decomposed").count(), 1);
            assert!(!trace.contains("\"detail\":\"1000\""));
        }
    }
}

#[test]
fn every_mode_executes_ordinary_runtime_functions() {
    let source = "subtract is fn (left : Int, right : Int) -> Int\n  difference is left - right\n  return difference\n50 subtract 8\n";
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
    assert!(trace.contains("TOPAL-FUNCTION-ORDINARY-001"));
}

#[test]
fn every_mode_validates_nat_function_boundaries() {
    let source = "identity is fn (value : Nat) -> Nat\n  value\nidentity 42\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"42\n"));
    }

    let negative_argument = run(
        &["--test"],
        "identity is fn (value : Nat) -> Nat\n  value\nidentity -1\n",
    );
    assert!(!negative_argument.status.success());
    let error = String::from_utf8(negative_argument.stderr).unwrap();
    assert!(error.contains("E-FUNCTION-ARGUMENT-TYPE"));
    assert!(!error.contains("function.entry"));

    let negative_result = run(&[], "negative is fn () -> Nat\n  -1\nnegative ()\n");
    assert!(!negative_result.status.success());
    assert!(
        String::from_utf8(negative_result.stderr)
            .unwrap()
            .contains("E-FUNCTION-RESULT-TYPE")
    );
}

#[test]
fn every_mode_executes_proven_nat_recursion() {
    let source = "count-down is fn (value : Nat) -> Nat\n  value\n    <= 0 then 0\n    otherwise count-down (value - 1)\ncount-down 3\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"0\n"));
    }
    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-RECURSION-NAT-001"));
    assert_eq!(trace.matches("function.recursion.descended").count(), 3);

    let unsafe_step = source.replace("value - 1", "value - 2");
    let output = run(&[], &unsafe_step);
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNPROVEN-RECURSION")
    );
}

#[test]
fn every_mode_executes_an_explicit_multi_parameter_measure() {
    let source = include_str!("../../../../examples/language/explicit-multi-parameter-decreases.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"12\n"));
    }
    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-DECREASES-001"));
    assert_eq!(trace.matches("function.recursion.descended").count(), 4);

    let unproven = source.replace("count - 1", "count - total");
    let output = run(&[], &unproven);
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNPROVEN-RECURSION")
    );
}

#[test]
fn every_mode_destructures_an_anonymous_product_pattern() {
    let source = include_str!("../../../../examples/language/anonymous-product-pattern.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output
                .stdout
                .ends_with(b"Entry ( 5, Entry ( 12, Empty ) )\n")
        );
    }
}

#[test]
fn nat_recursion_accepts_only_bound_preserving_decrements() {
    let safe = "count-down is fn (value : Nat) -> Nat\n  value\n    <= 2 then value\n    otherwise count-down (value - 3)\ncount-down 8\n";
    let output = run(&["--test"], safe);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"2\n");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("TOPAL-FUNCTION-RECURSION-NAT-001")
    );

    let unsafe_step = safe.replace("value - 3", "value - 4");
    let output = run(&[], &unsafe_step);
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNPROVEN-RECURSION")
    );
}

#[test]
fn every_mode_executes_proven_increasing_nat_recursion() {
    let source = "advance is fn (value : Nat) -> Nat\n  value\n    >= 5 then value\n    otherwise advance (value + 2)\nadvance 0\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"6\n"));
    }
    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"));
    assert_eq!(trace.matches("function.recursion.descended").count(), 3);
}

#[test]
fn every_mode_executes_proven_mutual_nat_recursion() {
    let source = "even is fn (value : Nat) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Nat) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\n(even 6, odd 6)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(true, false)\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"));
    assert!(trace.contains("function.recursion.cycle.proven"));
}

#[test]
fn test_mode_traces_proven_mutual_increasing_nat_recursion() {
    let source = "even is fn (value : Nat) -> Boolean\n  value\n    >= 6 then true\n    otherwise odd (value + 1)\nodd is fn (value : Nat) -> Boolean\n  value\n    >= 6 then false\n    otherwise even (value + 1)\n(even 0, odd 0)\n";
    let output = run(&["--test"], source);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"(true, false)\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"));
    assert!(trace.contains("function.recursion.cycle.proven"));
}

#[test]
fn every_mode_declares_and_compares_enum_values() {
    let source = "Color is Enum (Red, Green, Blue)\n(Red, Green, Red = Red, Red = Green)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(Red, Green, true, false)\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("enum.declared"));
    assert!(trace.contains("TOPAL-TYPE-ENUM-001"));

    let duplicate = run(&[], "Color is Enum (Red, Red)\n");
    assert!(!duplicate.status.success());
    assert!(
        String::from_utf8(duplicate.stderr)
            .unwrap()
            .contains("E-DUPLICATE-ENUM-ALTERNATIVE")
    );
}

#[test]
fn every_mode_uses_enum_function_classifiers() {
    let source = "Color is Enum (Red, Green)\nidentity is fn (value : Color) -> Color\n  value\n(identity Red, identity Green)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(Red, Green)\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("identity (Color)"));
}

#[test]
fn every_mode_executes_exhaustive_enum_decisions() {
    let source = "Color is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"red\"\n    Green then \"green\"\n(name Red, name Green)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(\"red\", \"green\")\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-DECISION-ENUM-001"));

    let incomplete = source.replace("    Green then \"green\"\n", "");
    let output = run(&[], &incomplete);
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-INCOMPLETE-DECISION")
    );
}

#[test]
fn every_mode_resolves_arithmetic_error_codes_qualified() {
    let source = "(lang arithmetic division-by-zero, lang arithmetic indeterminate, (lang arithmetic division-by-zero) = (lang arithmetic division-by-zero))\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"(division-by-zero, indeterminate, true)\n");
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-NUM-ARITHMETIC-ERROR-001"));
    assert!(!trace.contains("Error.domain"));
}

#[test]
fn every_mode_executes_successful_result_contracts() {
    let source = "identity is fn (value : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  value\nidentity 1.5\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"Rational ( 3, 2 )\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("function.result.contract"));
    assert!(trace.contains("TOPAL-TYPE-RESULT-001"));
}

#[test]
fn every_mode_returns_dynamic_rational_division_error() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\n1.0 divide 0.0\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(
            b"Error ( domain is root./(Rational,Rational), code is division-by-zero )\n"
        ));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("result.error.constructed"));
    assert!(trace.contains("TOPAL-NUM-DIVZERO-001"));

    let static_zero = run(&[], "1.0 / 0.0\n");
    assert!(!static_zero.status.success());
    assert!(
        String::from_utf8(static_zero.stderr)
            .unwrap()
            .contains("E-DIVISION-BY-ZERO")
    );
}

#[test]
fn every_mode_executes_nested_function_calls() {
    let source = "answer is fn () -> Int\n  increment 41\nincrement is fn (input : Int) -> Int\n  input + 1\nanswer ()\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n()\n42\n");
        } else {
            assert_eq!(output.stdout, b"42\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let outer_entry = trace.find("\"event\":\"function.entry\"").unwrap();
    let inner_entry = trace[outer_entry + 1..]
        .find("\"event\":\"function.entry\"")
        .unwrap()
        + outer_entry
        + 1;
    let inner_return = trace.rfind("\"detail\":\"increment\"").unwrap();
    let outer_return = trace.rfind("\"detail\":\"answer\"").unwrap();
    assert!(outer_entry < inner_entry && inner_entry < inner_return && inner_return < outer_return);
}

#[test]
fn static_function_cannot_call_an_ordinary_function() {
    let output = run(
        &[],
        "runtime is fn () -> Int\n  42\ncompile is fn static () -> Int\n  runtime ()\ncompile ()\n",
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-STATIC-CALLS-RUNTIME-FUNCTION")
    );
}

#[test]
fn every_mode_preserves_outer_bindings_across_local_shadowing() {
    let source =
        "value is 40\nanswer is fn () -> Int\n  value is 42\n  value\n(answer (), value)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n()\n(42, 40)\n");
        } else {
            assert_eq!(output.stdout, b"(42, 40)\n");
        }
    }
}

#[test]
fn every_mode_selects_typed_function_overloads() {
    let source = "describe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  value\n(describe 42, describe \"Topal\")\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n()\n(\"integer\", \"Topal\")\n");
        } else {
            assert_eq!(output.stdout, b"(\"integer\", \"Topal\")\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let integer = trace.find("describe (Int)").unwrap();
    let string = trace.find("describe (String)").unwrap();
    assert!(integer < string);
}

#[test]
fn overload_failure_lists_available_signatures() {
    let output = run(
        &[],
        "describe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  value\ndescribe true\n",
    );
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("E-NO-APPLICABLE-OVERLOAD"));
    assert!(error.contains("available overloads: describe (Int), describe (String)"));
}

#[test]
fn malformed_source_regex_reports_a_diagnostic_instead_of_a_nonmatch() {
    let library_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("library");
    let output = run(
        &["--library-root", library_root.to_str().unwrap()],
        "use library std ( version is v0.1 )\nmatches? is std pattern regex contains?\nmatches? (\"text\", \"[\")\n",
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("E-RESULT-PROJECTION-INFALLIBLE"));
    assert!(error.contains("RegexValid"));
}

#[test]
fn every_mode_executes_complete_boolean_decisions() {
    let source = "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    otherwise 0\n(choose true, choose false)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n(42, 0)\n");
        } else {
            assert_eq!(output.stdout, b"(42, 0)\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"decision.rule.considered\""));
    assert!(trace.contains("\"event\":\"decision.rule.selected\""));
}

#[test]
fn every_mode_executes_exhaustive_boolean_decisions_without_otherwise() {
    let source = "describe-flag is fn (flag : Boolean) -> String\n  flag\n    true then \"enabled\"\n    false then \"disabled\"\n(describe-flag true, describe-flag false)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n(\"enabled\", \"disabled\")\n");
        } else {
            assert_eq!(output.stdout, b"(\"enabled\", \"disabled\")\n");
        }
    }
}

#[test]
fn every_mode_calls_a_later_function_declaration() {
    let source = "render is fn (text : String) -> String\n  decorate text\ndecorate is fn (text : String) -> String\n  \"[\" concat text concat \"]\"\nrender \"Topal\"\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n()\n\"[Topal]\"\n");
        } else {
            assert_eq!(output.stdout, b"\"[Topal]\"\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let render = trace
        .find("function.entry\",\"rule\":\"TOPAL-FUNCTION-ORDINARY-001\",\"detail\":\"render")
        .unwrap();
    let decorate = trace
        .find("function.entry\",\"rule\":\"TOPAL-FUNCTION-ORDINARY-001\",\"detail\":\"decorate")
        .unwrap();
    assert!(render < decorate);
}

#[test]
fn every_mode_executes_proven_mutual_int_recursion() {
    let source = "even is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\n(even 6, odd 6)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n()\n(true, false)\n");
        } else {
            assert_eq!(output.stdout, b"(true, false)\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("function.recursion.edge.candidate"));
    assert!(trace.contains("function.recursion.cycle.proven"));
    assert!(trace.contains("TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001"));
}

#[test]
fn every_mode_executes_proven_mutual_increasing_int_recursion() {
    let source = "even-up is fn (value : Int) -> Boolean\n  value\n    >= 0 then true\n    otherwise odd-up (value + 1)\nodd-up is fn (value : Int) -> Boolean\n  value\n    >= 0 then false\n    otherwise even-up (value + 1)\n(even-up (-6), odd-up (-6))\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n()\n(true, false)\n");
        } else {
            assert_eq!(output.stdout, b"(true, false)\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001"));
    assert!(trace.contains("function.recursion.cycle.proven"));
}

#[test]
fn every_mode_distinguishes_overloads_from_recursion() {
    let source = "describe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  (describe 42) concat \":\" concat value\ndescribe \"Topal\"\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n()\n\"integer:Topal\"\n");
        } else {
            assert_eq!(output.stdout, b"\"integer:Topal\"\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let string = trace.find("describe (String)").unwrap();
    let integer = trace.find("describe (Int)").unwrap();
    assert!(string < integer);
    assert!(!trace.contains("E-UNPROVEN-RECURSION"));
}

#[test]
fn every_mode_executes_positive_literal_recursion_steps() {
    let source = "down-hops is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise 1 + (down-hops (value - 3))\nup-hops is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise 1 + (up-hops (value + 2))\n(down-hops 7, up-hops (-5))\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n()\n(3, 3)\n");
        } else {
            assert_eq!(output.stdout, b"(3, 3)\n");
        }
    }
}

#[test]
fn every_mode_executes_multiple_proven_recursive_calls() {
    let source = "branch-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (branch-count (value - 1)) + (branch-count (value - 2))\nbranch-count 3\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n5\n");
        } else {
            assert_eq!(output.stdout, b"5\n");
        }
    }
}

#[test]
fn every_mode_executes_multiple_calls_on_a_proven_mutual_edge() {
    let source = "first-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second-count (value - 1)) + (second-count (value - 2))\nsecond-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first-count (value - 1)\nfirst-count 3\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n()\n3\n");
        } else {
            assert_eq!(output.stdout, b"3\n");
        }
    }
}

#[test]
fn every_mode_executes_comparison_decisions() {
    let source = "minimum is fn (left : Int, right : Int) -> Int\n  left\n    < right then left\n    otherwise right\n(42 minimum 50, 60 minimum 50)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n(42, 50)\n");
        } else {
            assert_eq!(output.stdout, b"(42, 50)\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("TOPAL-DECISION-COMPARISON-001"));
    assert!(trace.contains("TOPAL-NUM-COMPARE-001"));
}

#[test]
fn every_mode_executes_comparison_decision_forms() {
    let source = include_str!("../../../../examples/language/comparison-decision-forms.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(-1, 0, 1, -1, 0, 1, true, false)\n")
        );
    }
}

#[test]
fn every_mode_preserves_tuple_decision_results() {
    let source = include_str!("../../../../examples/language/tuple-decision-results.t");
    let expected = b"((42, \"true\"), (0, \"false\"), (-2, \"negative\"), (2, \"nonnegative\"), (-1, \"less\"), (1, \"right\"), (7, \"some\"), (0, \"none\"), (Rational ( 1, 2 ), \"ok\"), (Rational ( 0, 1 ), \"error\"))\n";
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
fn every_mode_preserves_tuple_function_parameters() {
    let source = include_str!("../../../../examples/language/tuple-function-parameters.t");
    let expected = b"(((42, true), \"nested\"), (7, \"kept\"), (0, \"fallback\"), \"tuple\", \"fields\", \"discarded\")\n";
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
fn every_mode_executes_proven_decreasing_int_recursion() {
    let source = "sum-down is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise value + (sum-down (value - 1))\nsum-down 5\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n15\n");
        } else {
            assert_eq!(output.stdout, b"15\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"function.recursion.proven\""));
    assert_eq!(trace.matches("function.recursion.descended").count(), 5);
}

#[test]
fn every_mode_executes_proven_increasing_int_recursion() {
    let source = "distance-up is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise 1 + (distance-up (value + 1))\ndistance-up (-5)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n5\n");
        } else {
            assert_eq!(output.stdout, b"5\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-RECURSION-INT-INCREASING-001"));
    assert_eq!(trace.matches("function.recursion.descended").count(), 5);
}

#[test]
fn every_mode_executes_comparison_operand_expressions() {
    let source = "within is fn (value : Int, limit : Int) -> Boolean\n  value\n    < limit + 1 then true\n    otherwise false\n(5 within 5, 6 within 5)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if arguments == ["--interactive"] {
            assert_eq!(output.stdout, b"()\n(true, false)\n");
        } else {
            assert_eq!(output.stdout, b"(true, false)\n");
        }
    }

    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let addition = trace.find("root.+(Int,Int)").unwrap();
    let comparison = trace.find("root.<(TotalOrder,TotalOrder)").unwrap();
    assert!(addition < comparison);
}

#[test]
fn every_mode_executes_nested_lexical_functions() {
    let source = "answer is fn (input : Int) -> Int\n  add-input is fn (value : Int) -> Int\n    value + input\n  add-input 2\nanswer 40\n";
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
    let outer = trace.find("\"detail\":\"answer\"").unwrap();
    let nested = trace.find("\"detail\":\"add-input\"").unwrap();
    assert!(outer < nested);
}

#[test]
fn script_executes_arbitrary_precision_based_integer() {
    let output = run(&[], "0xFFFF_FFFF_FFFF_FFFF_FFFF\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"1208925819614629174706175\n");
}

#[test]
fn test_mode_preserves_based_literal_lexeme_in_trace() {
    let output = run(&["--test"], "0b1010_1100\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"172\n");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("\"detail\":\"0b1010_1100\"")
    );
}

#[test]
fn script_rejects_discarded_expression_values() {
    let output = run(&[], "1\n2\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-DISCARDED-VALUE")
    );
}

#[test]
fn test_trace_explains_binding_decisions() {
    let output = run(&["--test"], "answer is 42\nanswer\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"binding.bind\""));
    assert!(trace.contains("\"event\":\"binding.resolved\""));
    assert!(trace.contains("\"rule\":\"TOPAL-SYN-BIND-001\""));
}

#[test]
fn all_modes_execute_signed_exact_addition() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "-0x1 + 1_000\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"999\n");
    }
}

#[test]
fn test_trace_explains_exact_addition() {
    let output = run(&["--test"], "40 + 2\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"operator.selected\""));
    assert!(trace.contains("\"detail\":\"root.+(Int,Int)\""));
    assert!(trace.contains("\"event\":\"evaluation.add\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-ADD-001\""));
}

#[test]
fn all_modes_execute_exact_subtraction() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "1_000 - 0x1 - 1\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"998\n");
    }
}

#[test]
fn test_trace_distinguishes_literal_sign_and_prefix_negation() {
    let literal = run(&["--test"], "-42\n");
    let literal_trace = String::from_utf8(literal.stderr).unwrap();
    assert!(literal_trace.contains("\"detail\":\"-42\""));
    assert!(!literal_trace.contains("evaluation.negate"));

    let prefix = run(&["--test"], "- 42\n");
    let prefix_trace = String::from_utf8(prefix.stderr).unwrap();
    assert!(prefix_trace.contains("\"detail\":\"root.-(Int)\""));
    assert!(prefix_trace.contains("\"rule\":\"TOPAL-NUM-NEG-001\""));
}

#[test]
fn test_trace_explains_binary_subtraction() {
    let output = run(&["--test"], "10 - 3\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root.-(Int,Int)\""));
    assert!(trace.contains("\"event\":\"evaluation.subtract\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-SUB-001\""));
}

#[test]
fn all_modes_execute_exact_multiplication_left_to_right() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "2 + 3 * 4\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"20\n");
    }
    let grouped = run(&[], "2 + (3 * 4)\n");
    assert!(grouped.status.success());
    assert_eq!(grouped.stdout, b"14\n");
}

#[test]
fn test_trace_explains_exact_multiplication() {
    let output = run(&["--test"], "6 * 7\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root.*(Int,Int)\""));
    assert!(trace.contains("\"event\":\"evaluation.multiply\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-MUL-001\""));
}

#[test]
fn all_modes_execute_exact_rational_division() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "-6 / -0x8\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"Rational ( 3, 4 )\n");
    }
}

#[test]
fn division_retains_rational_type_for_whole_result() {
    let output = run(&[], "6 / 3\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Rational ( 2, 1 )\n");
}

#[test]
fn division_by_zero_is_rejected() {
    let output = run(&[], "1 / 0\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-DIVISION-BY-ZERO")
    );
}

#[test]
fn test_trace_explains_exact_division() {
    let output = run(&["--test"], "6 / 8\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"obligation.proved\""));
    assert!(trace.contains("\"detail\":\"divisor.nonzero\""));
    assert!(trace.contains("\"detail\":\"root./(Int,Int)\""));
    assert!(trace.contains("\"event\":\"evaluation.divide\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-DIV-001\""));
    assert!(trace.contains("\"detail\":\"Rational\""));
}

#[test]
fn test_trace_explains_zero_division_rejection() {
    let output = run(&["--test"], "1 / 0\n");
    assert!(!output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"obligation.refuted\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-DIVZERO-001\""));
    assert!(!trace.contains("root./(Int,Int)"));
    assert!(!trace.contains("evaluation.divide"));
    assert!(trace.contains("error[E-DIVISION-BY-ZERO]"));
    assert!(trace.contains("<stdin>:4:5"));
}

#[test]
fn all_modes_execute_exact_natural_exponentiation() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "2 ^ 100\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"1267650600228229401496703205376\n");
    }
}

#[test]
fn zero_to_zero_is_empty_product() {
    let output = run(&["--test"], "0 ^ 0\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"1\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root.^(Int,Nat)\""));
    assert!(trace.contains("\"event\":\"evaluation.power\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-POW-001\""));
}

#[test]
fn exponentiation_has_no_hidden_precedence() {
    assert_eq!(run(&[], "2 + 3 ^ 2\n").stdout, b"25\n");
    assert_eq!(run(&[], "2 + (3 ^ 2)\n").stdout, b"11\n");
}

#[test]
fn negative_exponent_is_rejected_for_int_overload() {
    let output = run(&["--test"], "2 ^ -1\n");
    assert!(!output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"obligation.refuted\""));
    assert!(trace.contains("\"detail\":\"exponent.finite-nat\""));
    assert!(!trace.contains("root.^(Int,Nat)"));
    assert!(trace.contains("error[E-NO-APPLICABLE-OVERLOAD]"));
    assert!(trace.contains("<stdin>:4:5"));
}

#[test]
fn all_modes_construct_exact_rational_literals() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "-6.022e-24\n");
        assert!(output.status.success());
        assert_eq!(
            output.stdout,
            b"Rational ( -3011, 500000000000000000000000000 )\n"
        );
    }
}

#[test]
fn rational_literal_trace_retains_exact_lexeme() {
    let output = run(&["--test"], "1_000.000_125\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"token.rational\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-RATIONAL-LITERAL-001\""));
    assert!(trace.contains("\"detail\":\"1_000.000_125\""));
    assert!(trace.contains("\"detail\":\"Rational\""));
}

#[test]
fn all_modes_execute_exact_rational_arithmetic_left_to_right() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "0.5 + 0.25 * 2.0\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"Rational ( 3, 2 )\n");
    }
    let grouped = run(&[], "0.5 + (0.25 * 2.0)\n");
    assert_eq!(grouped.stdout, b"Rational ( 1, 1 )\n");
}

#[test]
fn rational_arithmetic_trace_identifies_overload_and_rule() {
    let output = run(&["--test"], "1.5 / 0.25\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root./(Rational,Rational)\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-RAT-DIV-001\""));
    assert!(trace.contains("\"event\":\"obligation.proved\""));
}

#[test]
fn all_modes_execute_mixed_exact_arithmetic() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "1 + 0.5 * 2\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"Rational ( 3, 1 )\n");
    }
}

#[test]
fn every_mode_executes_arbitrary_integer_arithmetic() {
    let source = include_str!("../../../../examples/language/arbitrary-integer-arithmetic.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(
            b"(123456789012345678901234567890, -123456789012345678901234567890, 864197532086419753208641975320, 1111111110111111111011111111100, -864197532086419753208641975320, 0, 121932631137021795226185032733622923332237463801111263526900, -121932631137021795226185032733622923332237463801111263526900, true, true, true, true)\n"
        ));
    }
}

#[test]
fn conversion_trace_precedes_rational_overload_selection() {
    let output = run(&["--test"], "1 + 0.5\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    let conversion = trace.find("\"event\":\"conversion.applied\"").unwrap();
    let selection = trace
        .find("\"detail\":\"root.+(Rational,Rational)\"")
        .unwrap();
    assert!(conversion < selection);
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-CONVERT-001\""));
    assert!(trace.contains("\"detail\":\"Int->Rational:left\""));
}
