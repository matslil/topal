#[test]
fn models_rational_lists_across_private_boundaries() {
    // TOPAL-COMPILER-LIST-RATIONAL-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-rational-values.t"
    ))
    .unwrap();
    let list = CompilerType::List(Box::new(CompilerType::Rational));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, CompilerType::Rational);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list.clone(), CompilerType::Rational])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Rational List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list);
    let unsupported = "use language (version is v0.1)\nhalf : Rational is Rational (1, 2)\nvalues : List Rational is Entry (half, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_effect_lists_across_private_boundaries() {
    // TOPAL-EFFECT-EMPTY-001, TOPAL-EFFECT-IDENTITY-001,
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-EFFECT-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-effect-values.t"
    ))
    .unwrap();
    let list_effect = CompilerType::List(Box::new(CompilerType::Effect));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list_effect);
    assert_eq!(head.result_type, CompilerType::Effect);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list_effect.clone(), CompilerType::Effect])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list_effect.clone()),
            ("fallback".into(), CompilerType::Effect),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Effect List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list_effect);

    let unsupported = "use language (version is v0.1)\nvalues : List Effect is Entry (Effects (), Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_comparison_lists_across_private_boundaries() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-COMPARISON-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-comparison-values.t"
    ))
    .unwrap();
    let list = CompilerType::List(Box::new(CompilerType::Comparison));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, CompilerType::Comparison);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list.clone(), CompilerType::Comparison])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list.clone()),
            ("fallback".into(), CompilerType::Comparison),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Comparison List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list);

    let unsupported = "use language (version is v0.1)\nvalues : List Comparison is Entry (1 <=> 2, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_error_code_lists_across_private_boundaries() {
    // TOPAL-NUM-ARITHMETIC-ERROR-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-ERROR-CODE-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-error-code-values.t"
    ))
    .unwrap();
    let list = CompilerType::List(Box::new(CompilerType::ErrorCode));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, CompilerType::ErrorCode);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list.clone(), CompilerType::ErrorCode])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list.clone()),
            ("fallback".into(), CompilerType::ErrorCode),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared ErrorCode List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list);

    let unsupported = "use language (version is v0.1)\nvalues : List ErrorCode is Entry (lang arithmetic out-of-range, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_unit_lists_across_private_boundaries() {
    // TOPAL-TYPE-PRODUCT-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-UNIT-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-unit-values.t"
    ))
    .unwrap();
    let list = CompilerType::List(Box::new(CompilerType::Unit));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, CompilerType::Unit);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list.clone(), CompilerType::Unit])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list.clone()),
            ("fallback".into(), CompilerType::Unit),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Unit List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list);

    let unsupported =
        "use language (version is v0.1)\nvalues : List Unit is Entry ((), Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_completed_lists_across_private_boundaries() {
    // TOPAL-FUNCTION-COMPLETED-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-COMPLETED-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-completed-values.t"
    ))
    .unwrap();
    let list = CompilerType::List(Box::new(CompilerType::Completed));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, CompilerType::Completed);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list.clone(), CompilerType::Completed])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list.clone()),
            ("fallback".into(), CompilerType::Completed),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Completed List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list);

    let unsupported = "use language (version is v0.1)\nvalues : List Completed is Entry (Completed, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_type_lists_across_private_boundaries() {
    // TOPAL-ABSTRACTION-TYPE-VALUE-001, TOPAL-ABSTRACTION-TYPE-IDENTITY-001,
    // TOPAL-ABSTRACTION-TYPE-BOUNDARY-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-TYPE-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-type-values.t"
    ))
    .unwrap();
    let list = CompilerType::List(Box::new(CompilerType::Type));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, CompilerType::Type);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list.clone(), CompilerType::Type])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list.clone()),
            ("fallback".into(), CompilerType::Type),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Type List regression returns a Tuple")
    };
    assert_eq!(results.len(), 11);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[10].value_type, list);

    let unsupported = "use language (version is v0.1)\nvalues : List Type is Entry (Int, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_nominal_enum_lists_across_private_boundaries() {
    // TOPAL-TYPE-ENUM-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-ENUM-001, TOPAL-COMPILER-LIST-ENUM-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-enum-values.t"
    ))
    .unwrap();
    let color = CompilerType::Enum(CompilerEnumType {
        name: "Color".into(),
        alternatives: vec!["Red".into(), "Green".into(), "Blue".into()],
    });
    let list = CompilerType::List(Box::new(color.clone()));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, color);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list.clone(), color.clone()])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list.clone()),
            ("fallback".into(), color),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared nominal Enum List regression returns a Tuple")
    };
    assert_eq!(results.len(), 11);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[10].value_type, list);

    let unsupported = "use language (version is v0.1)\nColor is Enum (Red, Green)\nvalues : List Color is Entry (Red, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let wrong_nominal = "use language (version is v0.1)\nColor is Enum (Red, Green)\nMode is Enum (On, Off)\nvalues : List Color is Entry (On, Empty)\nvalues\n";
    assert_eq!(
        analyze_for_compiler(wrong_nominal).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
    let unsupported_container = "use language (version is v0.1)\nColor is Enum (Red, Green)\nvalue : Optional Color is Some Red\nvalue\n";
    assert_eq!(
        analyze_for_compiler(unsupported_container)
            .unwrap_err()
            .code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_nominal_modular_lists_across_private_boundaries() {
    // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001,
    // TOPAL-COMPILER-MODULAR-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-MODULAR-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-modular-values.t"
    ))
    .unwrap();
    let byte_counter = CompilerType::Modular(CompilerModularType {
        name: "ByteCounter".into(),
        signed: false,
        lower: BigInt::from(0),
        upper: BigInt::from(255),
    });
    let list = CompilerType::List(Box::new(byte_counter.clone()));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, byte_counter);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list.clone(), byte_counter.clone()])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared nominal modular List regression returns a Tuple")
    };
    assert_eq!(results.len(), 11);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[10].value_type, list);

    let records = analyze_for_compiler(
            "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nretain-record is fn (package : Record (values : List ByteCounter, fallback : ByteCounter)) -> Record (values : List ByteCounter, fallback : ByteCounter)\n  package\nvalues : List ByteCounter is Entry (ByteCounter 1, Empty)\nretained is retain-record (values is values, fallback is ByteCounter 7)\nretained values\n",
        )
        .unwrap();
    let retain_record = records
        .functions
        .iter()
        .find(|function| function.source_name == "retain-record")
        .unwrap();
    assert!(matches!(
        &retain_record.result_type,
        CompilerType::Record(fields)
            if fields.as_slice()
                == [
                    ("fallback".into(), byte_counter.clone()),
                    ("values".into(), list.clone()),
                ]
    ));

    let unsupported = "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nvalues : List ByteCounter is Entry (ByteCounter 1, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let wrong_nominal = "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nSignedByte is ModInt ((-128) ..= 127)\nvalues : List ByteCounter is Entry (SignedByte 1, Empty)\nvalues\n";
    assert_eq!(
        analyze_for_compiler(wrong_nominal).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
    let unsupported_container = "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nvalue : Optional ByteCounter is Some (ByteCounter 1)\nvalue\n";
    assert_eq!(
        analyze_for_compiler(unsupported_container)
            .unwrap_err()
            .code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_optional_int_lists_across_private_boundaries() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001, TOPAL-COMPILER-LIST-OPTIONAL-INT-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-optional-int-values.t"
    ))
    .unwrap();
    let optional = CompilerType::Optional(Box::new(CompilerType::Int));
    let list = CompilerType::List(Box::new(optional.clone()));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, optional);
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Optional Int List regression returns a Tuple")
    };
    assert_eq!(results.len(), 11);
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[10].value_type, list);
    let unsupported = "use language (version is v0.1)\nvalues : List Optional Int is Entry (Some 1, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_optional_rational_lists_across_private_boundaries() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001,
    // TOPAL-COMPILER-LIST-OPTIONAL-RATIONAL-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-optional-rational-values.t"
    ))
    .unwrap();
    let optional = CompilerType::Optional(Box::new(CompilerType::Rational));
    let list = CompilerType::List(Box::new(optional.clone()));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, optional);
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Optional Rational List regression returns a Tuple")
    };
    assert_eq!(results.len(), 11);
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[10].value_type, list);
    let unsupported = "use language (version is v0.1)\nvalues : List Optional Rational is Entry (Some 1.5, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_optional_string_lists_across_private_boundaries() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001,
    // TOPAL-COMPILER-LIST-OPTIONAL-STRING-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-optional-string-values.t"
    ))
    .unwrap();
    let optional = CompilerType::Optional(Box::new(CompilerType::String));
    let list = CompilerType::List(Box::new(optional.clone()));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, optional);
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Optional String List regression returns a Tuple")
    };
    assert_eq!(results.len(), 11);
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[10].value_type, list);
    let unsupported = "use language (version is v0.1)\nvalues : List Optional String is Entry (Some \"value\", Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_int_pair_lists_across_private_boundaries() {
    // TOPAL-COMPILER-TUPLE-EQUALITY-001, TOPAL-COMPILER-LIST-INT-PAIR-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-int-pair-values.t"
    ))
    .unwrap();
    let pair = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]);
    let list = CompilerType::List(Box::new(pair.clone()));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, pair);
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Int-pair List regression returns a Tuple")
    };
    assert_eq!(results.len(), 11);
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[10].value_type, list);
    let unsupported = "use language (version is v0.1)\nvalues : List (Int, Int) is Entry ((1, 2), Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_int_string_pair_lists_across_private_boundaries() {
    // TOPAL-COMPILER-TUPLE-EQUALITY-001,
    // TOPAL-COMPILER-LIST-INT-STRING-PAIR-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-int-string-pair-values.t"
    ))
    .unwrap();
    let pair = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String]);
    let list = CompilerType::List(Box::new(pair.clone()));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, pair);
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Int/String-pair List regression returns a Tuple")
    };
    assert_eq!(results.len(), 12);
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[7].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[11].value_type, list);
    let reversed = analyze_for_compiler(
            "use language (version is v0.1)\nvalues : List (Int, String) is Entry ((1, \"one\"), Empty)\nvalues reverse\n",
        )
        .unwrap();
    assert!(matches!(
        reversed.main.result.kind,
        CompilerExpressionKind::ListReverse(_)
    ));
}

#[test]
fn models_string_int_pair_lists_across_private_boundaries() {
    // TOPAL-COMPILER-TUPLE-EQUALITY-001,
    // TOPAL-COMPILER-LIST-STRING-INT-PAIR-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-string-int-pair-values.t"
    ))
    .unwrap();
    let pair = CompilerType::Tuple(vec![CompilerType::String, CompilerType::Int]);
    let list = CompilerType::List(Box::new(pair.clone()));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, pair);
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared String/Int-pair List regression returns a Tuple")
    };
    assert_eq!(results.len(), 12);
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[7].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[11].value_type, list);
    let unsupported = "use language (version is v0.1)\nvalues : List (String, Int) is Entry ((\"one\", 1), Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_string_pair_lists_across_private_boundaries() {
    // TOPAL-COMPILER-TUPLE-EQUALITY-001,
    // TOPAL-COMPILER-LIST-STRING-PAIR-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-string-pair-values.t"
    ))
    .unwrap();
    let pair = CompilerType::Tuple(vec![CompilerType::String, CompilerType::String]);
    let list = CompilerType::List(Box::new(pair.clone()));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list);
    assert_eq!(head.result_type, pair);
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared String-pair List regression returns a Tuple")
    };
    assert_eq!(results.len(), 12);
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[7].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[11].value_type, list);
    let unsupported = "use language (version is v0.1)\nvalues : List (String, String) is Entry ((\"one\", \"first\"), Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_contextual_int_list_map_select_and_fold() {
    // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
    // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-LIST-INT-FUNCTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-list-functions.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared anonymous List regression returns a Tuple")
    };
    assert_eq!(results.len(), 3);
    assert!(matches!(
        results[0].kind,
        CompilerExpressionKind::ListMap { .. }
    ));
    assert!(matches!(
        results[1].kind,
        CompilerExpressionKind::ListSelect { .. }
    ));
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::ListFold { .. }
    ));
    assert_eq!(
        results[0].value_type,
        CompilerType::List(Box::new(CompilerType::Int))
    );
    assert_eq!(results[2].value_type, CompilerType::Int);

    let wrong_map = "use language (version is v0.1)\nvalues : List Int is one 1\nvalues map { value } value > 0\n";
    assert_eq!(
        analyze_for_compiler(wrong_map).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
    let wrong_select = "use language (version is v0.1)\nvalues : List Int is one 1\nvalues select { value } value + 1\n";
    assert_eq!(
        analyze_for_compiler(wrong_select).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
    let wrong_fold = "use language (version is v0.1)\nvalues : List Int is one 1\nvalues fold 0 { value } value\n";
    assert_eq!(
        analyze_for_compiler(wrong_fold).unwrap_err().code,
        "E-ANONYMOUS-FUNCTION-ARITY"
    );
}

#[test]
fn models_int_pair_list_product_map_pattern() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COLLECTION-MAP-001,
    // TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-LIST-INT-PAIR-MAP-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-product-pattern.t"
    ))
    .unwrap();
    let [CompilerStatement::Binding(pairs)] = program.main.statements.as_slice() else {
        panic!("shared product-pattern regression binds its pair List")
    };
    assert_eq!(
        pairs.value.value_type,
        CompilerType::List(Box::new(CompilerType::Tuple(vec![
            CompilerType::Int,
            CompilerType::Int,
        ])))
    );
    let CompilerExpressionKind::ListMap {
        parameters, body, ..
    } = &program.main.result.kind
    else {
        panic!("shared product-pattern regression maps the pair List")
    };
    assert_eq!(parameters.len(), 2);
    assert_eq!(parameters[0].name, "left");
    assert_eq!(parameters[1].name, "right");
    assert!(
        parameters
            .iter()
            .all(|parameter| parameter.value_type == CompilerType::Int)
    );
    assert!(matches!(
        body.result.kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add,
            ..
        }
    ));
    assert_eq!(
        program.main.result.value_type,
        CompilerType::List(Box::new(CompilerType::Int))
    );

    let wrong_arity = "use language (version is v0.1)\npairs : List (Int, Int) is Entry ((1, 2), Empty)\npairs map { (a, b, c) } a + b\n";
    assert_eq!(
        analyze_for_compiler(wrong_arity).unwrap_err().code,
        "E-ANONYMOUS-FUNCTION-ARITY"
    );
    let duplicate = "use language (version is v0.1)\npairs : List (Int, Int) is Entry ((1, 2), Empty)\npairs map { (same, same) } same\n";
    assert_eq!(
        analyze_for_compiler(duplicate).unwrap_err().code,
        "E-DUPLICATE-BINDING"
    );
    let bound = "use language (version is v0.1)\ncombine is { (left, right) } left + right\npairs : List (Int, Int) is Entry ((1, 2), Empty)\npairs map combine\n";
    assert!(analyze_for_compiler(bound).is_ok());
}

#[test]
fn models_exact_recursive_int_string_lists() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-TYPE-LIST-RECURSIVE-001, TOPAL-LIST-FIRST-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-COMPILER-LIST-RECURSIVE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/nested-lists.t"
    ))
    .unwrap();
    let pair = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String]);
    let inner = CompilerType::List(Box::new(pair));
    let nested = CompilerType::List(Box::new(inner.clone()));
    assert_eq!(program.functions[0].parameters[0].value_type, nested);
    assert_eq!(program.functions[0].result_type, nested);
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared recursive List regression returns a Tuple")
    };
    assert!(matches!(
        results[0].kind,
        CompilerExpressionKind::ListFirst(_)
    ));
    assert_eq!(
        results[0].value_type,
        CompilerType::Optional(Box::new(inner))
    );
    assert!(matches!(
        results[1].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));

    let inner_boundary = "use language (version is v0.1)\npreserve is fn (values : List (Int, String)) -> List (Int, String)\n  values\nvalues : List (Int, String) is Entry ((1, \"one\"), Empty)\npreserve values\n";
    assert!(analyze_for_compiler(inner_boundary).is_ok());

    let nested_int = "use language (version is v0.1)\npreserve is fn (values : List List Int) -> List List Int\n  values\ninner : List Int is Entry (1, Entry (2, Empty))\nnested : List List Int is Entry (inner, Empty)\ncopy : List List Int is Entry (inner, Empty)\nsingleton : List String is one \"solo\"\nsingleton-copy : List String is Entry (\"solo\", Empty)\n((preserve nested) = copy, entry-count nested, singleton = singleton-copy)\n";
    let program = analyze_for_compiler(nested_int).unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("nested Int List regression returns a Tuple")
    };
    assert_eq!(results[0].value_type, CompilerType::Boolean);
    assert_eq!(results[1].value_type, CompilerType::Nat);
    assert_eq!(results[2].value_type, CompilerType::Boolean);

    let unsupported_rest = "use language (version is v0.1)\nvalues : List (Int, String) is Entry ((1, \"one\"), Empty)\nnested : List List (Int, String) is Entry (values, Empty)\nrest nested\n";
    assert_eq!(
        analyze_for_compiler(unsupported_rest).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let deeper =
        "use language (version is v0.1)\nvalues : List List List (Int, String) is Empty\nvalues\n";
    assert_eq!(
        analyze_for_compiler(deeper).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_bound_anonymous_int_list_functions() {
    // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
    // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-LIST-INT-BOUND-FUNCTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/bound-anonymous-functions.t"
    ))
    .unwrap();
    assert_eq!(
        program
            .main
            .statements
            .iter()
            .filter(|statement| matches!(
                statement,
                CompilerStatement::Binding(binding)
                    if binding.value.value_type == CompilerType::Function
            ))
            .count(),
        3
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared bound anonymous List regression returns a Tuple")
    };
    assert!(matches!(
        results.as_slice(),
        [
            CompilerExpression {
                kind: CompilerExpressionKind::ListMap { .. },
                ..
            },
            CompilerExpression {
                kind: CompilerExpressionKind::ListSelect { .. },
                ..
            },
            CompilerExpression {
                kind: CompilerExpressionKind::ListFold { .. },
                ..
            }
        ]
    ));
}

#[test]
fn models_short_circuiting_int_list_fold_control() {
    // TOPAL-EXEC-TRAVERSAL-CONTROL-001,
    // TOPAL-COMPILER-TRAVERSAL-CONTROL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/traversal-control.t"
    ))
    .unwrap();
    let [_, CompilerStatement::Binding(controls), _] = program.main.statements.as_slice() else {
        panic!("shared traversal regression binds values, controls, and its action")
    };
    let CompilerExpressionKind::Tuple(control_values) = &controls.value.kind else {
        panic!("controls binding retains both constructor values")
    };
    assert!(matches!(
        control_values.as_slice(),
        [
            CompilerExpression {
                kind: CompilerExpressionKind::TraversalControl { finish: false, .. },
                value_type: CompilerType::TraversalControl(_),
                ..
            },
            CompilerExpression {
                kind: CompilerExpressionKind::TraversalControl { finish: true, .. },
                value_type: CompilerType::TraversalControl(_),
                ..
            }
        ]
    ));
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared traversal regression returns a Tuple")
    };
    let CompilerExpressionKind::ListFold { body, .. } = &results[0].kind else {
        panic!("first result is the controlled fold")
    };
    assert_eq!(
        body.result.value_type,
        CompilerType::TraversalControl(Box::new(CompilerType::Int))
    );
    assert_eq!(results[0].value_type, CompilerType::Int);

    let invalid_constructor = "use language (version is v0.1)\nContinue true\n";
    assert_eq!(
        analyze_for_compiler(invalid_constructor).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
    let invalid_fold = "use language (version is v0.1)\nvalues : List Int is one 1\nvalues fold 0 { state, value } true\n";
    assert_eq!(
        analyze_for_compiler(invalid_fold).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
}

#[test]
fn models_range_selection_without_exposing_slice_storage() {
    // TOPAL-RANGE-VALUE-SELECTION-001, TOPAL-RANGE-INDEX-SELECTION-001,
    // TOPAL-COMPILER-RANGE-SELECTION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/range-selection.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared range-selection regression returns a Tuple")
    };
    assert!(matches!(
        results.as_slice(),
        [
            CompilerExpression {
                kind: CompilerExpressionKind::ListRangeSelect { indexes: false, .. },
                ..
            },
            CompilerExpression {
                kind: CompilerExpressionKind::ListRangeSelect { indexes: true, .. },
                ..
            },
            CompilerExpression {
                kind: CompilerExpressionKind::String(value),
                ..
            }
        ] if value == "opa"
    ));

    let unicode = analyze_for_compiler(
            "use language (version is v0.1)\ntext : String is \"a\u{301}👩‍🔬🇸🇪x\"\ntext select-index (1 ..= 2)\n",
        )
        .unwrap();
    assert!(matches!(
        unicode.main.result.kind,
        CompilerExpressionKind::String(ref value) if value == "👩‍🔬🇸🇪"
    ));

    let wrong_selector = analyze_for_compiler(
        "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\nvalues select 1\n",
    )
    .unwrap_err();
    assert_eq!(wrong_selector.code, "E-TYPE-MISMATCH");

    let unsupported_element = analyze_for_compiler(
            "use language (version is v0.1)\nvalues : List Effect is Entry (Effects (), Empty)\nvalues select-index (0 .. 1)\n",
        )
        .unwrap_err();
    assert_eq!(unsupported_element.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_private_tuple_function_results_recursively() {
    // TOPAL-EXEC-COMPLETION-EFFECT-VALUE-001,
    // TOPAL-COMPILER-TUPLE-RESULT-001
    let completion = analyze_for_compiler(include_str!(
        "../../../../../examples/language/completion-effect-value.t"
    ))
    .unwrap();
    assert_eq!(completion.functions.len(), 1);
    assert_eq!(
        completion.functions[0].result_type,
        CompilerType::Tuple(vec![CompilerType::Completed, CompilerType::Effect])
    );
    let [CompilerStatement::Binding(binding)] = completion.main.statements.as_slice() else {
        panic!("expected one result binding")
    };
    assert_eq!(binding.name, "result");
    assert!(matches!(
        binding.value.kind,
        CompilerExpressionKind::Call { .. }
    ));
    assert!(matches!(
        completion.main.result.kind,
        CompilerExpressionKind::Local(ref storage) if storage == &binding.storage_name
    ));

    let nested = analyze_for_compiler(
            "use language (version is v0.1)\nmake is fn static () -> ((Int, Boolean), String)\n  ((42, true), \"Topal\")\nmake ()\n",
        )
        .unwrap();
    assert_eq!(
        nested.functions[0].result_type,
        CompilerType::Tuple(vec![
            CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Boolean]),
            CompilerType::String,
        ])
    );
}

#[test]
fn models_tuple_results_for_every_admitted_decision_family() {
    // TOPAL-COMPILER-TUPLE-DECISION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/tuple-decision-results.t"
    ))
    .unwrap();
    for function in program
        .functions
        .iter()
        .filter(|function| function.source_name.starts_with("choose-"))
    {
        assert!(matches!(function.result_type, CompilerType::Tuple(_)));
    }
    for (name, expected) in [
        ("choose-boolean", "boolean"),
        ("choose-ordered", "ordered"),
        ("choose-comparison", "comparison"),
        ("choose-enum", "enum"),
        ("choose-optional", "optional"),
        ("choose-result", "result"),
    ] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        let actual = match function.body.result.kind {
            CompilerExpressionKind::BooleanDecision { .. } => "boolean",
            CompilerExpressionKind::OrderedComparisonDecision { .. } => "ordered",
            CompilerExpressionKind::ComparisonValueDecision { .. } => "comparison",
            CompilerExpressionKind::EnumDecision { .. } => "enum",
            CompilerExpressionKind::OptionalDecision { .. } => "optional",
            CompilerExpressionKind::ResultDecision { .. } => "result",
            _ => panic!("expected a checked decision result for {name}"),
        };
        assert_eq!(actual, expected);
    }
}

#[test]
fn models_private_tuple_parameters_and_candidate_specific_product_calls() {
    // TOPAL-COMPILER-TUPLE-PARAMETER-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/tuple-function-parameters.t"
    ))
    .unwrap();

    let retain = program
        .functions
        .iter()
        .find(|function| function.source_name == "retain")
        .unwrap();
    assert_eq!(retain.parameters.len(), 1);
    assert_eq!(
        retain.parameters[0].value_type,
        CompilerType::Tuple(vec![
            CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Boolean]),
            CompilerType::String,
        ])
    );

    for function in program
        .functions
        .iter()
        .filter(|function| function.source_name == "choose")
    {
        assert!(matches!(
            function.parameters.as_slice(),
            [
                CompilerParameter {
                    value_type: CompilerType::Tuple(_),
                    ..
                },
                CompilerParameter {
                    value_type: CompilerType::Boolean,
                    ..
                }
            ]
        ));
    }

    let tuple_first = program
        .functions
        .iter()
        .find(|function| function.source_name == "tuple-first")
        .unwrap();
    assert!(matches!(
        tuple_first.parameters.as_slice(),
        [CompilerParameter {
            value_type: CompilerType::Tuple(_),
            ..
        }]
    ));
    let fields_first = program
        .functions
        .iter()
        .find(|function| function.source_name == "fields-first")
        .unwrap();
    assert!(matches!(
        fields_first.parameters.as_slice(),
        [
            CompilerParameter {
                value_type: CompilerType::Int,
                ..
            },
            CompilerParameter {
                value_type: CompilerType::String,
                ..
            }
        ]
    ));

    let discard = program
        .functions
        .iter()
        .find(|function| function.source_name == "discard-pair")
        .unwrap();
    assert!(discard.parameters[0].discarded);
    assert!(matches!(
        discard.parameters[0].value_type,
        CompilerType::Tuple(_)
    ));
}

#[test]
fn models_order_preserving_private_record_boundaries() {
    // TOPAL-COMPILER-RECORD-BOUNDARY-001, TOPAL-TYPE-PRODUCT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/record-function-boundaries.t"
    ))
    .unwrap();
    let person_type = CompilerType::Record(vec![
        ("active".to_owned(), CompilerType::Boolean),
        ("name".to_owned(), CompilerType::String),
    ]);

    let function = |name: &str| {
        program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap()
    };
    let make = function("make-person");
    assert_eq!(make.result_type, person_type);
    let retain = function("retain-person");
    assert_eq!(retain.parameters[0].value_type, person_type);
    assert_eq!(retain.result_type, person_type);
    for (name, expected) in [
        ("choose-person", "boolean"),
        ("choose-ordered", "ordered"),
        ("choose-comparison", "comparison"),
        ("choose-enum", "enum"),
        ("choose-optional", "optional"),
        ("choose-result", "result"),
    ] {
        let actual = match function(name).body.result.kind {
            CompilerExpressionKind::BooleanDecision { .. } => "boolean",
            CompilerExpressionKind::OrderedComparisonDecision { .. } => "ordered",
            CompilerExpressionKind::ComparisonValueDecision { .. } => "comparison",
            CompilerExpressionKind::EnumDecision { .. } => "enum",
            CompilerExpressionKind::OptionalDecision { .. } => "optional",
            CompilerExpressionKind::ResultDecision { .. } => "result",
            _ => panic!("expected a checked Record decision result for {name}"),
        };
        assert_eq!(actual, expected);
    }

    let wrapper = function("retain-wrapper");
    assert_eq!(
        wrapper.result_type,
        CompilerType::Record(vec![
            ("person".to_owned(), person_type),
            ("score".to_owned(), CompilerType::Int),
        ])
    );
    assert_eq!(wrapper.parameters[0].value_type, wrapper.result_type);
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::Tuple(_)
    ));
}

#[test]
fn models_closed_fundamental_type_values_and_identity() {
    // TOPAL-ABSTRACTION-TYPE-VALUE-001,
    // TOPAL-ABSTRACTION-TYPE-IDENTITY-001,
    // TOPAL-ABSTRACTION-TYPE-BOUNDARY-001
    let values = analyze_for_compiler(include_str!(
        "../../../../../examples/language/type-values.t"
    ))
    .unwrap();
    assert_eq!(
        values.main.result.value_type,
        CompilerType::Tuple(vec![CompilerType::Type; 7])
    );
    let CompilerExpressionKind::Tuple(fields) = &values.main.result.kind else {
        panic!("expected fundamental Type-value product")
    };
    assert_eq!(
        fields
            .iter()
            .map(|field| match field.kind {
                CompilerExpressionKind::TypeValue(value) => value,
                _ => panic!("expected a fundamental Type value"),
            })
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5, 6]
    );

    let boundary = analyze_for_compiler(include_str!(
        "../../../../../examples/language/type-function-boundary.t"
    ))
    .unwrap();
    let function = &boundary.functions[0];
    assert_eq!(function.parameters[0].value_type, CompilerType::Type);
    assert_eq!(function.result_type, CompilerType::Type);
    assert!(matches!(
        boundary.main.result.kind,
        CompilerExpressionKind::Call { .. }
    ));
    assert!(
        analyze_for_compiler(include_str!(
            "../../../../../examples/language/type-identity.t"
        ))
        .is_ok()
    );
    assert!(
        analyze_for_compiler(include_str!(
            "../../../../../examples/language/type-classifier.t"
        ))
        .is_ok()
    );
    assert_ne!(CompilerType::Type, CompilerType::Int);
}

#[test]
fn models_named_constraint_identity_with_a_checked_predicate() {
    // TOPAL-ABSTRACTION-CONSTRAINT-CLASSIFIER-001,
    // TOPAL-TYPE-CONSTRAINT-001, TOPAL-COMPILER-CONSTRAINT-VALUE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/constraint-classifier.t"
    ))
    .unwrap();
    assert_eq!(program.constraints.len(), 2);
    let constraint = &program.constraints[0];
    assert_eq!(constraint.name, "Positive");
    assert_eq!(constraint.base_type, CompilerType::Int);
    assert_eq!(constraint.parameter, "value");
    assert_eq!(constraint.predicate.value_type, CompilerType::Boolean);
    assert!(matches!(
        constraint.predicate.kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Greater,
            ..
        }
    ));
    assert_eq!(program.constraints[1].name, "rule");
    assert_eq!(program.constraints[1].base_type, CompilerType::Int);
    assert_eq!(program.constraints[1].predicate, constraint.predicate);
    assert!(matches!(
        program.main.statements[1],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::ConstraintValue(1),
                value_type: CompilerType::Constraint,
                ..
            },
            ..
        })
    ));
    assert!(matches!(
        program.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::ConstraintValue(0),
                value_type: CompilerType::Constraint,
                ..
            },
            ..
        })
    ));
    assert_eq!(program.main.result.value_type, CompilerType::Constraint);
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::Local(_)
    ));

    let non_boolean =
        "use language (version is v0.1)\nBroken is Int constraint { value } value + 1\nBroken\n";
    assert_eq!(
        analyze_for_compiler(non_boolean).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
    let captured = "use language (version is v0.1)\nlimit is 0\nPositive is Int constraint { value } value > limit\nPositive\n";
    let error = analyze_for_compiler(captured).unwrap_err();
    assert_eq!(error.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        error
            .message
            .contains("captured constraint predicate value")
    );
}
