//! TOPAL-SYSTEMS-TAIL-BOUNDS-001: independent algebra and defensive IR cases.
use super::*;

fn checked(step: &str) -> (CompilerProgram, String) {
    let text = format!(
        "use language (version is v0.1)\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat (remaining - 1, {step})\nrepeat (4, 0)\n"
    );
    let program = topal_language::analyze_for_compiler(&text).unwrap();
    let K::Call { symbol, .. } = &program.main.result.kind else {
        panic!("ordinary entry call")
    };
    let symbol = symbol.clone();
    (program, symbol)
}
fn edge_mut<'a>(program: &'a mut CompilerProgram, symbol: &str) -> &'a mut CompilerExpression {
    let entry = program
        .functions
        .iter_mut()
        .find(|f| f.symbol == symbol)
        .unwrap();
    let K::OrderedComparisonDecision { otherwise, .. } = &mut entry.body.result.kind else {
        panic!()
    };
    otherwise
}
fn argument_mut<'a>(program: &'a mut CompilerProgram, symbol: &str) -> &'a mut CompilerExpression {
    let K::Call { arguments, .. } = &mut edge_mut(program, symbol).kind else {
        panic!()
    };
    &mut arguments[1]
}
fn bound(program: &CompilerProgram, entry: &str) -> SystemsTailStorageBound {
    analyze_systems_tail_storage(program, entry).unwrap()
}
fn refusal(program: &CompilerProgram, entry: &str, reason: &str) {
    let error = analyze_systems_tail_storage(program, entry)
        .unwrap_err()
        .to_string();
    assert!(error.contains(reason), "{error}");
}

#[test]
fn actual_clearer_has_a_65_bit_offset_and_constant_per_step_storage() {
    let source = include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t");
    let program =
        topal_language::compiler::analyze_kernel_region_tail_source(source, &[], "clear-prefix", 3)
            .unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "clear-prefix")
        .unwrap()
        .symbol
        .clone();
    let result = bound(&program, &entry);
    let offset = result.values.iter().find(|v| v.parameter == 2).unwrap();
    assert_eq!(
        (offset.maximum_bits, offset.positive_weight.as_str()),
        (65, "1")
    );
    assert_eq!(offset.upper, (word_maximum() * 2u8).to_string());
    assert_eq!(
        (
            result.bank_bytes,
            result.initial_bridge_bytes,
            result.private_metadata_bytes
        ),
        (80, 72, 160)
    );
    assert_eq!(
        (result.recursive_step_bytes, result.terminal_step_bytes),
        (576, 0)
    );
    assert_eq!(result.maximum_private_temporary_bytes, 968);
    assert_eq!(result.caller_input_frame_bytes, 24);
    assert_eq!(
        (
            result.source_frames_per_step.as_str(),
            result.source_region_loans_per_step.as_str()
        ),
        ("3", "3")
    );
    assert_eq!(
        result.maximum_back_edges_for_identity_budget,
        (u64::MAX - 2) / 3 - 1
    );
    assert!(result.qualification.contains("no native stack"));
    assert!(result.source_plan.functions.iter().any(|f| f.writes_owner));
}

#[test]
fn signed_positive_and_negative_drift_use_separate_potentials() {
    for (source, positive, negative) in [("total + 3", 3_i64, 0_i64), ("total - 5", 0, 5)] {
        let (program, entry) = checked(source);
        let result = bound(&program, &entry);
        assert_eq!(result.initial_bridge_bytes, 72);
        assert_eq!(result.caller_input_frame_bytes, 24);
        let value = &result.values[1];
        assert_eq!(value.positive_weight, positive.to_string());
        assert_eq!(value.negative_weight, negative.to_string());
        assert_eq!(value.lower, (-word_maximum() * (negative + 1)).to_string());
        assert_eq!(value.upper, (word_maximum() * (positive + 1)).to_string());
        // Independent execution oracle for both potential inequalities.
        for initial in 0_i64..=64 {
            for total in [-64_i64, 0, 64] {
                let mut x = total;
                let mut m = initial;
                while m > 0 {
                    x += positive - negative;
                    m -= 1;
                    assert!(x + positive * m <= 64 + positive * 64);
                    assert!(-x + negative * m <= 64 + negative * 64);
                }
            }
        }
    }
}

#[test]
fn nonunit_progress_uses_the_ceiling_of_drift_per_decrement() {
    let (mut program, entry) = checked("total + 5");
    let function = program
        .functions
        .iter_mut()
        .find(|f| f.symbol == entry)
        .unwrap();
    let K::OrderedComparisonDecision {
        rules, otherwise, ..
    } = &mut function.body.result.kind
    else {
        panic!()
    };
    rules[0].operand.kind = K::Int(2.into());
    let K::Call { arguments, .. } = &mut otherwise.kind else {
        panic!()
    };
    let K::IntToNat(step) = &mut arguments[0].kind else {
        panic!()
    };
    let K::Binary { right, .. } = &mut step.kind else {
        panic!()
    };
    right.kind = K::Int(3.into());
    assert_eq!(bound(&program, &entry).values[1].positive_weight, "2");
}

#[test]
fn nonlinear_growth_is_refused_instead_of_widened_until_it_fits() {
    let (program, entry) = checked("total * 2");
    refusal(&program, &entry, "potential baseline");
}

#[test]
fn constant_resets_expand_baseline_and_preserve_multiple_limbs_and_sign() {
    for value in [BigInt::from(1) << 159usize, -(BigInt::from(1) << 159usize)] {
        let (mut program, entry) = checked("total");
        argument_mut(&mut program, &entry).kind = K::Int(value.clone());
        let result = bound(&program, &entry);
        assert_eq!(result.values[1].maximum_bits, 160);
        assert_eq!(result.values[1].bank_slot_bytes, 40);
        assert!(
            result.values[1].lower == value.to_string()
                || result.values[1].upper == value.to_string()
        );
    }
}

#[test]
fn arithmetic_ceiling_applies_to_carried_constants_before_publication() {
    let (mut program, entry) = checked("total");
    argument_mut(&mut program, &entry).kind = K::Int(BigInt::from(1) << 256usize);
    refusal(&program, &entry, "proof ceiling");
}

#[test]
fn a_reset_inside_global_but_outside_baseline_is_refused() {
    let (mut program, entry) = checked("total + 1");
    let first = edge_mut(&mut program, &entry).clone();
    let mut second = first.clone();
    let K::Call { arguments, .. } = &mut second.kind else {
        panic!()
    };
    // Exactly M+1 fits the widened global upper bound 2M. Keep this
    // deliberately non-linear in the symbolic resolver: |-(M+1)|.
    let mut literal = arguments[1].clone();
    literal.kind = K::Int(-(word_maximum() + BigInt::from(1)));
    let mut number = literal.clone();
    number.kind = K::Absolute(Box::new(literal));
    // Make the two result paths differ, preventing exact constant folding.
    let mut zero = number.clone();
    zero.kind = K::Int(0.into());
    let mut subject = zero.clone();
    subject.kind = K::Boolean(true);
    subject.value_type = CompilerType::Boolean;
    arguments[1].kind = K::BooleanDecision {
        subject: Box::new(subject.clone()),
        when_true: Box::new(number),
        when_false: Box::new(zero),
    };
    edge_mut(&mut program, &entry).kind = K::BooleanDecision {
        subject: Box::new(subject),
        when_true: Box::new(first),
        when_false: Box::new(second),
    };
    refusal(&program, &entry, "potential baseline");
}

#[test]
fn equal_source_spans_do_not_confuse_different_tail_edges() {
    let (mut program, entry) = checked("total + 1");
    let first = edge_mut(&mut program, &entry).clone();
    let mut second = first.clone();
    let K::Call { arguments, .. } = &mut second.kind else {
        panic!()
    };
    let K::Binary { right, .. } = &mut arguments[1].kind else {
        panic!()
    };
    right.kind = K::Int((-4).into());
    let mut subject = first.clone();
    subject.kind = K::Boolean(true);
    subject.value_type = CompilerType::Boolean;
    edge_mut(&mut program, &entry).kind = K::BooleanDecision {
        subject: Box::new(subject),
        when_true: Box::new(first),
        when_false: Box::new(second),
    };
    let result = bound(&program, &entry);
    assert_eq!(
        (
            result.values[1].positive_weight.as_str(),
            result.values[1].negative_weight.as_str()
        ),
        ("1", "4")
    );
    assert_eq!(result.source_plan.back_edges.len(), 2);
}

#[test]
fn legacy_numeric_bounds_still_refuse_subtraction() {
    let (mut program, entry) = checked("total - 1");
    let expression = argument_mut(&mut program, &entry).clone();
    let mut bounds = Bounds {
        functions: BTreeMap::new(),
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
        maximum_bits: 0,
        regions: false,
        tail: false,
        tail_visits: 0,
    };
    let env = BTreeMap::from([("total".into(), Value::Number(Range(0.into(), 4.into())))]);
    assert!(
        bounds
            .expression(&expression, &env)
            .err()
            .unwrap()
            .to_string()
            .contains("unqualified numeric operation")
    );
}

#[test]
fn fresh_identity_counts_include_duplicate_calls_to_one_helper() {
    let text = "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\ndouble is fn (value : Int) -> Int\n  (identity(value)) + (identity(value))\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then double(total)\n    otherwise repeat(remaining - 1, identity(total))\nrepeat(4, 0)\n";
    let program = topal_language::analyze_for_compiler(text).unwrap();
    let K::Call { symbol, .. } = &program.main.result.kind else {
        panic!()
    };
    let result = bound(&program, symbol);
    assert_eq!(result.source_frames_per_step, "3");
    assert_eq!(result.source_region_loans_per_step, "0");
    assert_eq!(
        result.maximum_back_edges_for_identity_budget,
        (u64::MAX - 2) / 3 - 1
    );
    assert!(result.terminal_step_bytes > 0);
}

#[test]
fn affine_relations_pass_through_actual_checked_helpers() {
    let (program, _) = checked("total + 7");
    let source = program
        .source
        .as_str()
        .replace(
            "repeat is fn",
            "advance is fn (value : Int) -> Int\n  value + 7\nrepeat is fn",
        )
        .replace("remaining - 1, total + 7", "remaining - 1, advance(total)");
    let program = topal_language::analyze_for_compiler(&source).unwrap();
    let K::Call { symbol, .. } = &program.main.result.kind else {
        panic!()
    };
    let result = bound(&program, symbol);
    assert_eq!(result.values[1].positive_weight, "7");
    assert_eq!(result.source_frames_per_step, "2");
    assert_eq!(result.recursive_step_bytes, 576);
}

#[test]
fn binding_storage_identity_and_prefix_cost_survive_branch_refinement() {
    let (mut program, entry) = checked("total + 3");
    let value = argument_mut(&mut program, &entry).clone();
    argument_mut(&mut program, &entry).kind = K::Local("derived".into());
    let function = program
        .functions
        .iter_mut()
        .find(|f| f.symbol == entry)
        .unwrap();
    function.body.statements.push(CompilerStatement::Binding(
        topal_language::compiler::CompilerBinding {
            name: "total".into(),
            storage_name: "derived".into(),
            span: value.span,
            value,
        },
    ));
    let result = bound(&program, &entry);
    assert_eq!(result.values[1].positive_weight, "3");
    assert_eq!(result.terminal_step_bytes, 320);
    assert_eq!(result.recursive_step_bytes, 576);
}

#[test]
fn discarded_argument_work_is_charged_even_when_not_part_of_the_relation() {
    let (mut program, entry) = checked("total + 1");
    let expression = argument_mut(&mut program, &entry).clone();
    let mut result = expression.clone();
    result.kind = K::Local("total".into());
    argument_mut(&mut program, &entry).kind = K::Block(Box::new(CompilerBlock {
        statements: vec![CompilerStatement::Discard(expression); 2],
        result,
    }));
    let result = bound(&program, &entry);
    assert_eq!(result.values[1].positive_weight, "0");
    // Two 65-bit additions (320 each), plus the 64-bit decrement (256).
    assert_eq!(result.recursive_step_bytes, 896);
}

#[test]
fn ordinary_optional_loads_keep_their_cost_and_read_only_effects() {
    let source = include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t").replace(
        "region byte store (offset-bytes is offset, value is 0)\n    true then 1\n    false then 0",
        "region byte load (offset-bytes is offset)\n    Some value then 1\n    None then 0",
    );
    let program = topal_language::compiler::analyze_kernel_region_tail_source(
        &source,
        &[],
        "clear-prefix",
        3,
    )
    .unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "clear-prefix")
        .unwrap();
    let result = bound(&program, &entry.symbol);
    assert_eq!(result.recursive_step_bytes, 616);
    assert_eq!(result.maximum_private_temporary_bytes, 1008);
    let root = result
        .source_plan
        .functions
        .iter()
        .find(|f| f.symbol == entry.symbol)
        .unwrap();
    assert!(root.reads_owner && !root.writes_owner);
}

#[test]
fn optional_tail_payloads_do_not_lose_edge_identity_or_baseline_checks() {
    let (mut program, entry) = checked("total");
    let mut some = edge_mut(&mut program, &entry).clone();
    let absent = some.clone();
    let K::Call { arguments, .. } = &mut some.kind else {
        panic!()
    };
    arguments[1].kind = K::Local("payload".into());
    let mut number = arguments[1].clone();
    number.kind = K::Int(9.into());
    let mut subject = number.clone();
    subject.value_type = CompilerType::Optional(Box::new(CompilerType::Int));
    subject.kind = K::OptionalSome(Box::new(number));
    edge_mut(&mut program, &entry).kind = K::OptionalDecision {
        subject: Box::new(subject),
        some_binding: Some(("payload".into(), some.span)),
        some_action: Box::new(some),
        none_action: Box::new(absent),
    };
    assert_eq!(bound(&program, &entry).recursive_step_bytes, 272);
}

#[test]
fn unchecked_negative_nat_successor_is_refused() {
    let (mut program, entry) = checked("total - 1");
    let function = program
        .functions
        .iter_mut()
        .find(|f| f.symbol == entry)
        .unwrap();
    function.parameters[1].value_type = CompilerType::Nat;
    let argument = argument_mut(&mut program, &entry);
    argument.value_type = CompilerType::Nat;
    argument.kind = K::IntToNat(Box::new(argument.clone()));
    refusal(&program, &entry, "unproven Nat conversion");
}

#[test]
fn total_private_allocation_remains_subject_to_the_existing_reserve() {
    let (mut program, entry) = checked("total + 1");
    let expression = argument_mut(&mut program, &entry).clone();
    program
        .functions
        .iter_mut()
        .find(|f| f.symbol == entry)
        .unwrap()
        .body
        .statements = vec![CompilerStatement::Discard(expression); 260];
    refusal(&program, &entry, "declared reserve");
}

#[test]
fn helper_dag_call_multiplicity_is_not_its_depth_or_function_count() {
    // Construct a compact shared DAG explicitly. Ordinary source analysis may
    // specialize each call site; this defensive IR case claims no source origin.
    let (mut program, entry) = checked("total");
    let prototype = program
        .functions
        .iter()
        .find(|f| f.symbol == entry)
        .unwrap()
        .clone();
    let argument = argument_mut(&mut program, &entry).clone();
    let mut prior = String::new();
    for index in 0..=7 {
        let name = format!("test-helper-{index}");
        let mut function = prototype.clone();
        function.symbol = name.clone();
        function.source_name = name.clone();
        function.parameters = vec![prototype.parameters[1].clone()];
        let mut result = argument.clone();
        if index > 0 {
            let mut call = argument.clone();
            call.kind = K::Call {
                symbol: prior,
                arguments: vec![argument.clone()],
            };
            result.kind = K::Binary {
                operation: Op::Add,
                left: Box::new(call.clone()),
                right: Box::new(call),
            };
        }
        function.body = CompilerBlock {
            statements: Vec::new(),
            result,
        };
        program.functions.push(function);
        prior = name;
    }
    let function = program
        .functions
        .iter_mut()
        .find(|f| f.symbol == entry)
        .unwrap();
    let K::OrderedComparisonDecision { rules, .. } = &mut function.body.result.kind else {
        panic!()
    };
    rules[0].action.kind = K::Call {
        symbol: prior,
        arguments: vec![argument],
    };
    let result = bound(&program, &entry);
    assert_eq!(result.source_frames_per_step, "255");
    assert_eq!(result.source_plan.maximum_one_step_function_depth, 9);
    assert_eq!(result.terminal_step_bytes, 40640);
}

#[test]
fn ordered_tail_operands_and_early_terminal_actions_are_charged() {
    let (mut program, entry) = checked("total + 1");
    let tail = edge_mut(&mut program, &entry).clone();
    let argument = argument_mut(&mut program, &entry).clone();
    let mut subject = argument.clone();
    subject.kind = K::Local("total".into());
    let mut terminal = subject.clone();
    terminal.kind = K::Absolute(Box::new(subject.clone()));
    edge_mut(&mut program, &entry).kind = K::OrderedComparisonDecision {
        subject: Box::new(subject),
        rules: vec![topal_language::compiler::CompilerComparisonRule {
            operation: Op::LessEqual,
            operand: argument,
            action: terminal,
            subject_to_rational: false,
            span: tail.span,
        }],
        otherwise: Box::new(tail),
    };
    let result = bound(&program, &entry);
    assert_eq!(result.recursive_step_bytes, 896);
    assert_eq!(result.terminal_step_bytes, 640);
}

#[test]
fn additional_outer_terminal_operands_and_actions_are_charged() {
    // TOPAL-SYSTEMS-TAIL-BOUNDS-001: the positive-measure terminal path and
    // its tested operand must not disappear behind the first guard's cost.
    let (mut program, entry) = checked("total");
    let original = bound(&program, &entry);
    let total = argument_mut(&mut program, &entry).clone();
    let mut operand = total.clone();
    operand.kind = K::Absolute(Box::new(total));
    let mut action = operand.clone();
    action.kind = K::Absolute(Box::new(operand.clone()));
    let function = program
        .functions
        .iter_mut()
        .find(|f| f.symbol == entry)
        .unwrap();
    let K::OrderedComparisonDecision { rules, .. } = &mut function.body.result.kind else {
        panic!()
    };
    let mut extra = rules[0].clone();
    extra.operand = operand;
    extra.action = action;
    rules.push(extra);
    let extended = bound(&program, &entry);
    assert!(extended.terminal_step_bytes > original.terminal_step_bytes);
    assert!(extended.recursive_step_bytes > original.recursive_step_bytes);
}

#[test]
fn tail_proof_work_has_an_explicit_budget() {
    let (mut program, entry) = checked("total");
    let expression = argument_mut(&mut program, &entry).clone();
    let mut bounds = Bounds {
        functions: BTreeMap::new(),
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
        maximum_bits: 0,
        regions: true,
        tail: true,
        tail_visits: 65536,
    };
    assert!(
        bounds
            .expression(&expression, &BTreeMap::new())
            .err()
            .unwrap()
            .to_string()
            .contains("proof budget")
    );
}

#[test]
fn symbolic_intermediates_are_limited_before_large_multiplication_chains() {
    let (mut program, entry) = checked("total * 2");
    let argument = argument_mut(&mut program, &entry);
    let K::Binary { right, .. } = &mut argument.kind else {
        panic!()
    };
    right.kind = K::Int(BigInt::from(1) << 255usize);
    let factor = right.clone();
    let inner = argument.clone();
    argument.kind = K::Binary {
        operation: Op::Multiply,
        left: Box::new(inner),
        right: factor,
    };
    refusal(&program, &entry, "symbolic numeric proof ceiling");
}

#[test]
fn identity_budget_reserves_root_and_representable_next_counter() {
    // Independent small-counter execution oracle, including exact divisibility
    // and already-consumed history rather than testing only a fresh maximum.
    for maximum in 3_u64..=31 {
        for next in 2..=maximum {
            for per_step in 1..=9 {
                let mut counter = next;
                let mut steps = 0_u64;
                while counter + per_step <= maximum {
                    counter += per_step;
                    steps += 1;
                }
                let result =
                    identity_back_edge_limit(&per_step.into(), &maximum.into(), &next.into());
                if steps == 0 {
                    assert!(result.is_err());
                } else {
                    assert_eq!(result.unwrap(), BigInt::from(steps - 1));
                }
            }
        }
    }
    assert!(identity_back_edge_limit(&0.into(), &31.into(), &2.into()).is_err());
    assert!(identity_back_edge_limit(&1.into(), &31.into(), &1.into()).is_err());
    assert!(identity_back_edge_limit(&1.into(), &31.into(), &32.into()).is_err());
    let (program, entry) = checked("total");
    let result = bound(&program, &entry);
    assert_eq!(result.identity_counter_initial_next, 2);
    assert_eq!(result.maximum_back_edges_for_identity_budget, u64::MAX - 3);
}
