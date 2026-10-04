use super::*;

fn evaluate(source: &str) -> Result<Value, Diagnostic> {
    Session::new().evaluate(source, &mut std::io::sink())
}

#[test]
fn evaluates_qualified_static_introspection() {
    assert!(matches!(
        evaluate("lang context\n").unwrap(),
        Value::Introspection(_)
    ));
    assert_eq!(
        evaluate("lang version\n").unwrap(),
        Value::Version(LanguageVersion::DESIGN_0)
    );

    let identity = evaluate("lang identity Int\n").unwrap();
    assert!(matches!(
        identity,
        Value::Introspection(value)
            if matches!(&*value, IntrospectionValue::Identity { canonical, .. } if canonical == "type:Int")
    ));

    let view = evaluate("lang view Int\n").unwrap();
    assert!(matches!(
        view,
        Value::Introspection(value)
            if matches!(&*value, IntrospectionValue::TypeView { form, identity } if form == "PrimitiveType" && identity == "Int")
    ));

    let effect = evaluate("lang view (Effects ())\n").unwrap();
    assert!(matches!(
        effect,
        Value::Introspection(value)
            if matches!(&*value, IntrospectionValue::EffectView { identities } if identities.is_empty())
    ));
}

#[test]
fn preserves_constructed_language_variant_features() {
    let value = Session::new()
        .evaluate_source_file(
            "use language ( version is v0.1, features is ( debug, lint ) )\nlang context\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert!(matches!(
        value,
        Value::Introspection(context)
            if matches!(&*context, IntrospectionValue::LanguageContext { features, .. }
                if features == &["debug", "lint"])
    ));
}

#[test]
fn declaration_view_exposes_attached_documentation() {
    let value = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.1 )\n### The documented answer.\npub answer is 42\nlang declaration answer\n",
                &mut std::io::sink(),
            )
            .unwrap();
    assert!(matches!(
        value,
        Value::Introspection(view)
            if matches!(&*view, IntrospectionValue::DeclarationView { documentation, .. }
                if documentation.as_deref() == Some("The documented answer."))
    ));
}

#[test]
fn exposes_lint_namespace_only_in_the_lint_variant() {
    let value = Session::new()
        .evaluate_source_file(
            "use language ( version is v0.1, features is ( lint ) )\nlang lint\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert!(matches!(value, Value::Namespace(namespace) if namespace.name == "lang lint"));

    let error = Session::new()
        .evaluate_source_file(
            "use language ( version is v0.1 )\nlang lint\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LINT-VARIANT");
}

#[test]
fn evaluates_static_object_relations_without_runtime_reflection() {
    assert_eq!(
        evaluate("Int lang same-object Int\n").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        evaluate("Int lang equivalent-type Rational\n").unwrap(),
        Value::Boolean(false)
    );
    let error = evaluate("lang view 42\n").unwrap_err();
    assert_eq!(error.code, "E-STATIC-INTROSPECTION-SUBJECT");
    let error = evaluate("1 lang same-object 1\n").unwrap_err();
    assert_eq!(error.code, "E-STATIC-INTROSPECTION-SUBJECT");
}

#[test]
fn serializes_with_an_explicit_version_and_deserializes_validated_streams() {
    let direct = evaluate("v0.1 (lang serialize) (answer is 42, ok is true)\n").unwrap();
    assert!(matches!(direct, Value::SerializationStream(_)));

    let round_trip = evaluate(
            "serialize is lang version (lang serialize)\nstream is serialize (answer is 42, ok is true)\nlang deserialize stream\n",
        )
        .unwrap();
    assert_eq!(round_trip.to_string(), "(answer is 42, ok is true)");

    let huge = "1606938044258990275541962092341162602522202993782792835301376";
    let round_trip = evaluate(&format!(
        "stream is v0.1 (lang serialize) {huge}\nlang deserialize stream\n"
    ))
    .unwrap();
    assert_eq!(round_trip.to_string(), huge);
}

#[test]
fn native_serialization_round_trips_numeric_and_collection_schemas() {
    let values = [
        Value::Rational(BigRational::new(BigInt::from(1), BigInt::from(3))),
        Value::List {
            element_classifier: "Int".into(),
            entries: vec![Value::Int(BigInt::from(1)), Value::Int(BigInt::from(2))],
        },
        Value::Set {
            element_classifier: "Int".into(),
            entries: vec![Value::Int(BigInt::from(1)), Value::Int(BigInt::from(2))],
        },
        Value::Map {
            key_classifier: "Int".into(),
            value_classifier: "String".into(),
            entries: vec![
                (Value::Int(BigInt::from(1)), Value::String("one".into())),
                (Value::Int(BigInt::from(2)), Value::String("two".into())),
            ],
        },
    ];
    for value in values {
        let stream = stream_for_value(LanguageVersion::DESIGN_0, &value).unwrap();
        let bytes = serialize_native(&stream).unwrap();
        let decoded = deserialize_native(&bytes, SerializationLimits::default()).unwrap();
        assert_eq!(
            value_from_serialized(&decoded.events[0], &decoded.types),
            Some(value)
        );
    }

    let described = Value::Type("Int".into());
    let stream = stream_for_value(LanguageVersion::DESIGN_0, &described).unwrap();
    let bytes = serialize_native(&stream).unwrap();
    let decoded = deserialize_native(&bytes, SerializationLimits::default()).unwrap();
    assert!(matches!(
        value_from_serialized(&decoded.events[0], &decoded.types),
        Some(Value::ObjectDescription { kind, .. }) if kind == "Type"
    ));
}

#[test]
fn retains_explicit_function_effect_upper_bounds_for_static_views() {
    let value = evaluate(
        "identity is fn ( value : Int ) -> Int\n  : Effects ()\n  value\nlang view identity\n",
    )
    .unwrap();
    assert!(matches!(
        value,
        Value::Introspection(view)
            if matches!(&*view, IntrospectionValue::FunctionView { effects, .. } if effects == &["Effects ()"])
    ));
}

#[test]
fn v02_enforces_function_preconditions_and_named_result_postconditions() {
    let source = "use language ( version is v0.2 )\nincrement is fn ( value : Int )\n  requires ( value >= 0 )\n  effects ( Effects () )\n-> result : Int\n  ensures ( result > value )\n  value + 1\nincrement 41\n";
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate_source_file(source, &mut trace)
        .unwrap();
    assert_eq!(value, Value::Int(BigInt::from(42)));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-CONTRACT-REQUIRES-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-CONTRACT-ENSURES-001"))
    );

    let error = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.2 )\nidentity is fn ( value : Int ) requires ( value >= 0 ) -> Int\n  value\nnegative is 0 - 1\nidentity negative\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
    assert_eq!(error.code, "E-CONTRACT-REQUIRES", "{error:?}");

    let error = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.2 )\nidentity is fn ( value : Int ) -> result : Int ensures ( result > value )\n  value\nidentity 1\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
    assert_eq!(error.code, "E-CONTRACT-ENSURES");
}

#[test]
fn language_revisions_keep_function_clause_syntax_disjoint() {
    let v01 = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.1 )\nidentity is fn ( value : Int ) requires true -> Int\n  value\nidentity 1\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
    assert_eq!(v01.code, "E-UNSUPPORTED-LANGUAGE-CONSTRUCT");

    let v02 = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.2 )\nidentity is fn ( value : Int ) -> Int : Effects ()\n  value\nidentity 1\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
    assert_eq!(v02.code, "E-FUNCTION-CLAUSE-PLACEMENT");
}

#[test]
fn v02_language_owned_properties_cannot_be_shadowed_at_root() {
    for property in [
        "AtomicCommit",
        "Compensates",
        "Consumes",
        "DeadlineMet",
        "Durable",
        "Exclusive",
        "ImmediateHandler",
        "MultiShot",
        "NoAlloc",
        "OAlloc",
        "OExec",
        "Progress",
        "ReleaseJitter",
        "ResourceBound",
        "ResponseWithin",
        "RetrySafe",
        "Specialized",
        "WorstCaseExecution",
    ] {
        let source = format!("use language ( version is v0.2 )\n{property} is 1\n");
        let error = Session::new()
            .evaluate_source_file(&source, &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-LANGUAGE-NAME-CONFLICT");
    }

    let value = Session::new()
        .evaluate_source_file(
            "use language ( version is v0.1 )\nRetrySafe is 1\nRetrySafe\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value, Value::Int(BigInt::from(1)));

    let ordinary_mechanism_names = Session::new()
        .evaluate_source_file(
            "use language ( version is v0.2 )\natomic is 40\nlock is 2\natomic + lock\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(ordinary_mechanism_names, Value::Int(BigInt::from(42)));
}

#[test]
fn interpreter_requires_evidence_for_hard_implementation_guarantees() {
    let hard = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.2 )\nidentity is fn ( value : Int ) guarantees ( Progress LockFree ) -> Int\n  value\nidentity 1\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
    assert_eq!(hard.code, "E-IMPLEMENTATION-EVIDENCE-UNAVAILABLE");

    let unknown_hard = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.2 )\nidentity is fn ( value : Int ) guarantees ( LibraryProperty value ) -> Int\n  value\nidentity 1\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
    assert_eq!(unknown_hard.code, "E-IMPLEMENTATION-EVIDENCE-UNAVAILABLE");

    let preferred = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.2 )\nidentity is fn ( value : Int ) guarantees ( Prefer ( Progress LockFree ) ) -> Int\n  value\nidentity 1\n",
                &mut std::io::sink(),
            )
            .unwrap();
    assert_eq!(preferred, Value::Int(BigInt::from(1)));
}

#[test]
fn interpreter_fails_closed_for_unproved_parameter_evidence() {
    for qualifier in ["Exclusive", "Consumes"] {
        let source = format!(
            "use language ( version is v0.2 )\nidentity is fn ( value : String : {qualifier} ) -> String\n  value\n"
        );
        let error = Session::new()
            .evaluate_source_file(&source, &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-PARAMETER-EVIDENCE-UNAVAILABLE");
    }
}

#[test]
fn binds_packaged_function_operands_and_fills_field_defaults() {
    let value = evaluate(
            "sum is fn ( ( value : Int, fallback : Int default 2 ) ) -> Int\n  value + fallback\nsum (value is 40)\n",
        )
        .unwrap();
    assert_eq!(value, Value::Int(BigInt::from(42)));

    let value = evaluate(
            "sum is fn ( ( value : Int, fallback : Int default 2 ) ) -> Int\n  value + fallback\nsum (value is 40, fallback is 3)\n",
        )
        .unwrap();
    assert_eq!(value, Value::Int(BigInt::from(43)));
}

#[test]
fn constructs_tasks_and_routes_stateful_message_transactions() {
    let source = "Counter is Task (queue-size is 10, identity is counter)\
\ncounter-service is Counter\
\n  count : Nat\
\n  start is fn ( initial : Nat ) -> Completed\
\n    @ count is initial\
\n    Completed\
\n  increment is fn ( _ : MessageContext, amount : Nat ) -> Unit\
\n    @ count is @ count + amount\
\n  current is fn ( _ : MessageContext, _ : Unit ) -> Result ( Nat, () )\
\n    @ count\
\ncounter is counter-service 40\
\ncounter increment 2\
\ncounter current ()\n";
    let mut trace = Vec::new();
    let value = Session::new().evaluate(source, &mut trace).unwrap();
    assert_eq!(value, Value::Int(BigInt::from(42)));
    assert!(trace.iter().any(|event| event.contains("message.sent")));
    assert!(trace.iter().any(|event| event.contains("message.received")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("task.state.replaced"))
    );
}

#[test]
fn task_termination_discards_events_and_fails_requests_in_task_domain() {
    let source = "Counter is Task (identity is counter)\
\nservice is Counter\
\n  count : Nat\
\n  start is fn (initial : Nat) -> Completed\
\n    @ count is initial\
\n    Completed\
\n  ping is fn (_ : MessageContext, _ : Unit) -> Unit\
\n    ()\
\n  current is fn (_ : MessageContext, _ : Unit) -> Result (Nat, ())\
\n    @ count\
\n  terminate is fn (_ : String) -> Unit\
\n    ()\
\ninstance is service 1\
\ninstance terminate \"done\"\
\ninstance ping ()\
\ninstance current ()\n";
    assert!(matches!(
        evaluate(source).unwrap(),
        Value::Error { domain, code, .. }
            if domain == "lang task" && code == "task-terminated"
    ));
}

#[test]
fn task_streams_follow_transactions_and_reacquire_current_state() {
    let source = "Counter is Task (identity is counter)\
\nservice is Counter\
\n  count : Nat\
\n  start is fn (initial : Nat) -> Completed\
\n    @ count is initial\
\n    Completed\
\n  values is generator (_ : MessageContext, _ : Unit)\
\n    yields Nat\
\n    resumes Unit\
\n    -> Result (Unit, ())\
\n    yield @ count\
\n    @ count is @ count + 1\
\n    yield @ count\
\n    ()\
\n  current is fn (_ : MessageContext, _ : Unit) -> Result (Nat, ())\
\n    @ count\
\ninstance is service 1\
\nstream is instance values ()\
\nstream foreach { value }\
\n  ()\
\ninstance current ()\n";
    let mut trace = Vec::new();
    let value = Session::new().evaluate(source, &mut trace).unwrap();
    assert_eq!(value, Value::Int(BigInt::from(2)));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("message.stream.started"))
    );
}

#[test]
fn constructs_external_layouts_ranges_offsets_and_locations() {
    let source = "UInt32LE is (storage-size is 32[b], encoding is UnsignedBinary, endian is Little) Layout Nat\
\nDeviceAddresses is AddressRange (caching is Uncached, minimum-access-size is 32[b], medium is MMIO)\
\ndevice is DeviceAddresses (0x40000000 .. 0x4000ffff)\
\nDeviceOffset is AddressOffset (range is device, alignment is 4)\
\ncontrol-offset is DeviceOffset 32\
\nControlLocation is Location UInt32LE\
\ncontrol is ControlLocation control-offset\
\nstored is UInt32LE 42\
\ncontrol write stored\
\nread control\n";
    let value = evaluate(source).unwrap();
    assert!(
        matches!(value, Value::LayoutBacked { value, .. } if *value == Value::Int(BigInt::from(42)))
    );
}

#[test]
fn advances_a_prepared_execution_one_statement_at_a_time() {
    let mut session = Session::new();
    let mut trace = Vec::new();
    let mut execution = session
        .prepare("answer is 40\nanswer + 2\n", &mut trace)
        .unwrap();

    let first = execution.step(&mut session, &mut trace).unwrap();
    assert!(matches!(
        first,
        ExecutionStep::Advanced {
            value: Value::Unit,
            ..
        }
    ));
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("evaluation.result"))
    );

    let second = execution.step(&mut session, &mut trace).unwrap();
    assert!(matches!(second, ExecutionStep::Complete(Value::Int(_))));
    assert!(trace.last().unwrap().contains("evaluation.result"));
}

#[test]
fn evaluates_discard_without_introducing_a_binding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("_ is 20 + 22\n7\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "7");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("binding.discarded"))
    );
    assert!(
        Session::new()
            .evaluate("_\n", &mut std::io::sink())
            .is_err()
    );
}

#[test]
fn evaluates_labeled_record_products_in_field_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("(name is \"Ada\", active is true)\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "(name is \"Ada\", active is true)");
    assert!(trace.iter().any(|event| event.contains("product.record")));

    let duplicate = Session::new()
        .evaluate("(name is 1, name is 2)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(duplicate.code, "E-DUPLICATE-RECORD-FIELD");

    let mixed = Session::new()
        .evaluate("(1, name is 2)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(mixed.code, "E-MIXED-PRODUCT-FIELDS");
}

#[test]
fn selects_record_fields_without_resolving_the_label_as_a_name() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "person is (name is \"Ada\", active is true)\nperson name\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "\"Ada\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("record.field.selected"))
    );

    let error = Session::new()
        .evaluate("(name is \"Ada\") age\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-SUCH-RECORD-FIELD");
}

#[test]
fn derives_equality_for_records_with_the_same_shape() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(name is \"Ada\", score is 1) = (score is 1.0, name is \"Ada\")\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "true");
    assert!(trace.iter().any(|event| event.contains("Int->Rational")));

    let error = Session::new()
        .evaluate(
            "(name is \"Ada\") = (name is \"Ada\", active is true)\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn derives_equality_only_for_fully_equality_capable_nominal_sums() {
    // TOPAL-TYPE-SUM-EQUALITY-001, TOPAL-TYPE-EQUALITY-001
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../../examples/language/sum-equality.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(true, false, true, false, true, true, true, false, true, false, true)"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("equality.sum"))
            .count(),
        12
    );

    let unsupported = Session::new()
            .evaluate(
                "use language (version is v0.1)\nHolder is Union\n  Blank\n  Window : Range Int\n\nleft : Holder is Blank\nright : Holder is Blank\nleft = right\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
    assert_eq!(unsupported.code, "E-NO-APPLICABLE-OVERLOAD");

    let nominal = Session::new()
            .evaluate(
                "use language (version is v0.1)\nLeft is Union\n  LeftEmpty\n\nRight is Union\n  RightEmpty\n\nleft : Left is LeftEmpty\nright : Right is RightEmpty\nleft = right\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
    assert_eq!(nominal.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn concatenates_plain_strings_without_normalizing_the_join() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("\"e\" concat \"\u{301}\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"e\u{301}\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CONCAT-001"))
    );

    let error = Session::new()
        .evaluate("\"value\" concat 1\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn composes_only_adjacent_string_literals_implicitly() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("\"Hello, \" \"Topal\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"Hello, Topal\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-LITERAL-COMPOSE-001"))
    );

    let value = Session::new()
        .evaluate(
            "left is \"Hello, \"\nright is \"Topal\"\nleft concat right\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "\"Hello, Topal\"");

    let error = Session::new()
        .evaluate(
            "left is \"Hello, \"\nright is \"Topal\"\nleft right\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNSUPPORTED-APPLICATION");
}

#[test]
fn constructs_the_unique_empty_plain_string() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("empty String\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-EMPTY-001"))
    );
}

#[test]
fn tests_plain_string_emptiness() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("empty? (empty String)\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "true");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-EMPTY-PREDICATE-001"))
    );
    assert_eq!(
        Session::new()
            .evaluate("empty? \"Topal\"\n", &mut std::io::sink())
            .unwrap()
            .to_string(),
        "false"
    );
}

#[test]
fn counts_unicode_user_perceived_characters() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("character-count \"a\u{301}👩‍🔬🇸🇪\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "3");
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-STRING-CHARACTER-COUNT-001") && event.contains("characters=3")
    }));

    let error = Session::new()
        .evaluate("character-count 1\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn string_entry_count_agrees_with_character_count() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("entry-count \"a\u{301}👩‍🔬🇸🇪\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "3");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-ENTRY-COUNT-001"))
    );
}

#[test]
fn counts_prospective_utf8_bytes_without_normalizing() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("\"e\u{301}👩‍🔬\" byte-count Utf8\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "14");
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-STRING-UTF8-BYTE-COUNT-001") && event.contains("bytes=14")
    }));

    let error = Session::new()
        .evaluate("\"text\" byte-count Utf16\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn normalizes_plain_strings_to_nfc_explicitly() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("\"e\u{301}\" normalize NFC\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"é\"");
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-STRING-NORMALIZE-NFC-001") && event.contains("changed=true")
    }));
}

#[test]
fn normalizes_plain_strings_to_nfd_explicitly() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("\"é\" normalize NFD\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"e\u{301}\"");
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-STRING-NORMALIZE-NFD-001") && event.contains("changed=true")
    }));

    let error = Session::new()
        .evaluate("\"text\" normalize NFKD\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn adds_signed_arbitrary_precision_integers() {
    assert_eq!(
        evaluate("-1 + 123456789012345678901234567890")
            .unwrap()
            .to_string(),
        "123456789012345678901234567889"
    );
}

#[test]
fn follows_left_association() {
    assert_eq!(evaluate("1 + 2 + 3").unwrap().to_string(), "6");
}

#[test]
fn negates_and_subtracts_exact_integers() {
    assert_eq!(evaluate("- 42").unwrap().to_string(), "-42");
    assert_eq!(evaluate("10 - 3 - 2").unwrap().to_string(), "5");
    assert_eq!(evaluate("10 - -2").unwrap().to_string(), "12");
}

#[test]
fn multiplies_without_hidden_precedence() {
    assert_eq!(evaluate("2 + 3 * 4").unwrap().to_string(), "20");
    assert_eq!(evaluate("2 + (3 * 4)").unwrap().to_string(), "14");
    assert_eq!(
        evaluate("99999999999999999999 * 99999999999999999999")
            .unwrap()
            .to_string(),
        "9999999999999999999800000000000000000001"
    );
}

#[test]
fn divides_to_canonical_rational() {
    assert_eq!(evaluate("6 / 8").unwrap().to_string(), "Rational ( 3, 4 )");
    assert_eq!(
        evaluate("6 / -8").unwrap().to_string(),
        "Rational ( -3, 4 )"
    );
    assert_eq!(evaluate("6 / 3").unwrap().to_string(), "Rational ( 2, 1 )");
}

#[test]
fn rejects_statically_evident_zero_divisor() {
    assert_eq!(evaluate("1 / 0").unwrap_err().code, "E-DIVISION-BY-ZERO");
}

#[test]
fn renders_unicode_aligned_actionable_diagnostics() {
    let error = evaluate("α is 1\nα + missing").unwrap_err();
    assert_eq!(
        error.render("example.t"),
        "error[E-UNBOUND-NAME]: name is not bound\n --> example.t:2:5\n  |\n2 | α + missing\n  |     ^^^^^^^\n  |\n  = help: declare this name earlier in the same source session"
    );
}

#[test]
fn edit_distance_counts_unicode_scalars() {
    assert_eq!(edit_distance("räknare", "räknaren"), 1);
    assert_eq!(edit_distance("αβ", "βα"), 2);
}

#[test]
fn diagnostics_suggest_root_operations_and_the_concat_migration() {
    let error = Session::new()
        .evaluate("charcter-count \"Topal\"\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
    assert_eq!(
        error.help.as_deref(),
        Some("did you mean `character-count`?")
    );

    let error = Session::new()
        .evaluate("\"a\" concatenate \"b\"\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNSUPPORTED-APPLICATION");
    assert_eq!(error.help.as_deref(), Some("did you mean `concat`?"));
}

#[test]
fn raises_integer_to_natural_power_exactly() {
    assert_eq!(
        evaluate("2 ^ 100").unwrap().to_string(),
        "1267650600228229401496703205376"
    );
    assert_eq!(evaluate("0 ^ 0").unwrap().to_string(), "1");
    assert_eq!(evaluate("-2 ^ 3").unwrap().to_string(), "-8");
}

#[test]
fn exponentiation_uses_ordinary_left_association() {
    assert_eq!(evaluate("2 + 3 ^ 2").unwrap().to_string(), "25");
    assert_eq!(evaluate("2 + (3 ^ 2)").unwrap().to_string(), "11");
}

#[test]
fn raises_rationals_to_natural_powers_exactly() {
    assert_eq!(
        evaluate("1.5 ^ 3").unwrap().to_string(),
        "Rational ( 27, 8 )"
    );
    assert_eq!(
        evaluate("0.0 ^ 0").unwrap().to_string(),
        "Rational ( 1, 1 )"
    );
    assert_eq!(
        evaluate("1.5 ^ -2").unwrap().to_string(),
        "Rational ( 4, 9 )"
    );
    assert_eq!(
        evaluate("1.5 ^ 2.0").unwrap_err().code,
        "E-NO-APPLICABLE-OVERLOAD"
    );
    assert_eq!(evaluate("0.0 ^ -1").unwrap_err().code, "E-DIVISION-BY-ZERO");
}

#[test]
fn rejects_negative_integer_exponent() {
    assert_eq!(
        evaluate("2 ^ -1").unwrap_err().code,
        "E-NO-APPLICABLE-OVERLOAD"
    );
}

#[test]
fn constructs_exact_rational_literals() {
    assert_eq!(evaluate("0.1").unwrap().to_string(), "Rational ( 1, 10 )");
    assert_eq!(
        evaluate("1.25e3").unwrap().to_string(),
        "Rational ( 1250, 1 )"
    );
    assert_eq!(
        evaluate("-6.022e-24").unwrap().to_string(),
        "Rational ( -3011, 500000000000000000000000000 )"
    );
    assert_eq!(
        evaluate("1_000.000_125").unwrap().to_string(),
        "Rational ( 8000001, 8000 )"
    );
}

#[test]
fn rejects_malformed_rational_literal() {
    assert_eq!(evaluate("1.2e").unwrap_err().code, "E-NUMERIC-LITERAL");
}

#[test]
fn evaluates_exact_rational_arithmetic() {
    assert_eq!(
        evaluate("0.5 + 0.25").unwrap().to_string(),
        "Rational ( 3, 4 )"
    );
    assert_eq!(
        evaluate("- 1.5 - 0.25").unwrap().to_string(),
        "Rational ( -7, 4 )"
    );
    assert_eq!(
        evaluate("1.5 * 0.5").unwrap().to_string(),
        "Rational ( 3, 4 )"
    );
    assert_eq!(
        evaluate("1.5 / 0.25").unwrap().to_string(),
        "Rational ( 6, 1 )"
    );
}

#[test]
fn converts_int_for_mixed_exact_arithmetic() {
    assert_eq!(
        evaluate("1 + 0.5").unwrap().to_string(),
        "Rational ( 3, 2 )"
    );
    assert_eq!(
        evaluate("0.5 * 2").unwrap().to_string(),
        "Rational ( 1, 1 )"
    );
    assert_eq!(
        evaluate("1 / 0.5").unwrap().to_string(),
        "Rational ( 2, 1 )"
    );
}

#[test]
fn rejects_rational_zero_divisor() {
    assert_eq!(
        evaluate("1.0 / 0.0").unwrap_err().code,
        "E-DIVISION-BY-ZERO"
    );
}

#[test]
fn preserves_ordinary_and_tagged_string_contents() {
    assert_eq!(
        evaluate(r#""plain\n{value}""#).unwrap().to_string(),
        r#""plain\n{value}""#
    );
    assert_eq!(
        evaluate(r#"text"He said "hello"."text"#)
            .unwrap()
            .to_string(),
        r#"text"He said "hello"."text"#
    );
    assert_eq!(
        evaluate("\"first\nsecond\"").unwrap().to_string(),
        "\"first\nsecond\""
    );
}

#[test]
fn display_extends_colliding_string_tag() {
    assert_eq!(
        evaluate(r#"tag"contains "text closing"tag"#)
            .unwrap()
            .to_string(),
        r#"text_"contains "text closing"text_"#
    );
}

#[test]
fn parentheses_group_addition() {
    assert_eq!(evaluate("1 + (2 + 3)").unwrap().to_string(), "6");
}

#[test]
fn evaluates_binding_and_lookup() {
    assert_eq!(
        evaluate("answer is 40 + 2\nanswer").unwrap().to_string(),
        "42"
    );
}

#[test]
fn rejects_incomplete_grouping() {
    assert_eq!(evaluate("12_34").unwrap_err().code, "E-NUMERIC-LITERAL");
}
