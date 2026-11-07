use super::*;
use topal_language::compiler::{CompilerComparisonRule, CompilerParameter};

fn checked(text: &str) -> (CompilerProgram, String) {
    let program = topal_language::analyze_for_compiler(text).unwrap();
    let K::Call { symbol, .. } = &program.main.result.kind else {
        panic!("test entry must be an ordinary shared-frontend call")
    };
    let symbol = symbol.clone();
    (program, symbol)
}

fn counting() -> (CompilerProgram, String) {
    checked(
        "use language (version is v0.1)\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat (remaining - 1, total + 3)\nrepeat (4, 0)\n",
    )
}

fn refusal(program: &CompilerProgram, entry: &str, expected: &str) {
    let error = analyze_systems_tail_traversal(program, entry)
        .unwrap_err()
        .to_string();
    assert!(error.contains(expected), "{error}");
}

fn entry_mut<'a>(program: &'a mut CompilerProgram, symbol: &str) -> &'a mut CompilerFunction {
    program
        .functions
        .iter_mut()
        .find(|f| f.symbol == symbol)
        .unwrap()
}

fn edge_mut<'a>(program: &'a mut CompilerProgram, symbol: &str) -> &'a mut Expression {
    let K::OrderedComparisonDecision { otherwise, .. } =
        &mut entry_mut(program, symbol).body.result.kind
    else {
        panic!("countdown decision")
    };
    otherwise
}

#[test]
fn actual_checked_source_proves_natural_descent_without_emission_authority() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001, TOPAL-SYSTEMS-TAIL-REUSE-001.
    let (program, entry) = counting();
    let plan = analyze_systems_tail_traversal(&program, &entry).unwrap();
    assert_eq!(plan.measure_parameter, 0);
    assert_eq!(plan.base_bound, "0");
    assert_eq!(plan.minimum_decrement, "1");
    assert_eq!(plan.back_edges.len(), 1);
    assert_eq!(plan.maximum_one_step_function_depth, 1);
    assert_eq!(plan.functions[0].parameter_classifiers, ["Nat", "Int"]);
    assert_eq!(
        plan.qualification,
        "source-shape-only; no native emission or resource certificate"
    );
    assert_eq!(
        plan.source_sha256,
        digest(program.source.as_str().as_bytes())
    );
    assert!(!plan.functions[0].reads_owner && !plan.functions[0].writes_owner);
    // The source guard and argument order remain the actual shared checked IR.
    let K::OrderedComparisonDecision { otherwise, .. } = &program.functions[0].body.result.kind
    else {
        panic!()
    };
    let K::Call { arguments, .. } = &otherwise.kind else {
        panic!()
    };
    assert!(matches!(arguments[0].kind, K::IntToNat(_)));
    assert!(matches!(
        arguments[1].kind,
        K::Binary {
            operation: Op::Add,
            ..
        }
    ));
}

#[test]
fn nonunit_steps_charge_the_terminal_step_and_early_finish_only_reduces_work() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: independent mathematical execution oracle.
    let (program, entry) = checked(include_str!("../../../examples/language/nat-recursion.t"));
    let plan = analyze_systems_tail_traversal(&program, &entry).unwrap();
    assert_eq!(plan.base_bound, "2");
    assert_eq!(plan.minimum_decrement, "3");
    for initial in 0_u64..=257 {
        let mut remaining = initial;
        let mut back_edges = 0;
        while remaining > 2 {
            remaining -= 3;
            back_edges += 1;
        }
        let bound = initial.saturating_sub(2).div_ceil(3);
        assert_eq!(back_edges, bound);
        assert!(remaining <= 2);
        assert_eq!(back_edges + 1, bound + 1); // final guard/base action is live work
    }
}

#[test]
fn malformed_or_nonprogressing_edges_never_receive_a_plan() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: IR mutations test defensive inspection only.
    let (original, entry) = counting();
    for decrement in [-1, 0, 2] {
        let mut program = original.clone();
        let K::Call { arguments, .. } = &mut edge_mut(&mut program, &entry).kind else {
            panic!()
        };
        let K::IntToNat(value) = &mut arguments[0].kind else {
            panic!()
        };
        let K::Binary { right, .. } = &mut value.kind else {
            panic!()
        };
        right.kind = K::Int(BigInt::from(decrement));
        refusal(&program, &entry, "Nat descent");
    }
    let mut program = original.clone();
    let K::Call { arguments, .. } = &mut edge_mut(&mut program, &entry).kind else {
        panic!()
    };
    let K::IntToNat(value) = &mut arguments[0].kind else {
        panic!()
    };
    let K::Binary { left, .. } = &mut value.kind else {
        panic!()
    };
    left.kind = K::Local("total".into());
    refusal(&program, &entry, "Nat descent");
    let mut program = original.clone();
    let K::Call { arguments, .. } = &mut edge_mut(&mut program, &entry).kind else {
        panic!()
    };
    arguments.pop();
    refusal(&program, &entry, "arity");
    let mut program = original;
    let K::Call { arguments, .. } = &mut edge_mut(&mut program, &entry).kind else {
        panic!()
    };
    arguments[0].kind = K::Int(BigInt::from(0));
    refusal(&program, &entry, "Nat adaptation");
}

#[test]
fn recursion_in_base_arguments_and_nontail_expressions_is_refused() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: even a never-selected edge must be checked.
    let (original, entry) = counting();
    let edge = edge_mut(&mut original.clone(), &entry).clone();
    let mut program = original.clone();
    let K::OrderedComparisonDecision { rules, .. } =
        &mut entry_mut(&mut program, &entry).body.result.kind
    else {
        panic!()
    };
    rules[0].action = edge.clone();
    refusal(&program, &entry, "tail position");
    let mut program = original.clone();
    let current = edge_mut(&mut program, &entry);
    let mut operand = current.clone();
    operand.kind = K::Int(BigInt::from(1));
    current.kind = K::Binary {
        operation: Op::Add,
        left: Box::new(edge.clone()),
        right: Box::new(operand),
    };
    refusal(&program, &entry, "tail position");
    let mut program = original;
    let K::Call { arguments, .. } = &mut edge_mut(&mut program, &entry).kind else {
        panic!()
    };
    arguments[1] = edge;
    refusal(&program, &entry, "tail position");
}

#[test]
fn every_guard_branch_and_helper_identity_is_checked() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001.
    let (original, entry) = counting();
    let mut program = original.clone();
    let K::OrderedComparisonDecision { rules, .. } =
        &mut entry_mut(&mut program, &entry).body.result.kind
    else {
        panic!()
    };
    rules[0].operation = Op::Less;
    refusal(&program, &entry, "inclusive");
    let mut program = original.clone();
    let K::OrderedComparisonDecision { rules, .. } =
        &mut entry_mut(&mut program, &entry).body.result.kind
    else {
        panic!()
    };
    rules[0].operand.kind = K::Int(BigInt::from(-1));
    refusal(&program, &entry, "nonnegative literal");
    let mut program = original.clone();
    let K::OrderedComparisonDecision { rules, .. } =
        &mut entry_mut(&mut program, &entry).body.result.kind
    else {
        panic!()
    };
    let rule: CompilerComparisonRule = rules[0].clone();
    rules.push(rule);
    refusal(&program, &entry, "one rule");
    let mut program = original.clone();
    let K::Call { symbol, .. } = &mut edge_mut(&mut program, &entry).kind else {
        panic!()
    };
    *symbol = "unresolved-helper".into();
    refusal(&program, &entry, "unresolved helper");
    let mut program = original.clone();
    program.functions.push(program.functions[0].clone());
    refusal(&program, &entry, "duplicate symbol");
    refusal(&original, "no-entry", "missing entry");
}

#[test]
fn actual_helper_graph_and_dependencies_are_bound_with_independent_order() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001.
    let (mut program, entry) = checked(
        "use language (version is v0.1)\nadd is fn (total : Int) -> Int\n  total + 3\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat (remaining - 1, add total)\nrepeat (4, 0)\n",
    );
    let original = analyze_systems_tail_traversal(&program, &entry).unwrap();
    assert_eq!(original.functions.len(), 2);
    assert_eq!(original.maximum_one_step_function_depth, 2);
    program.functions.reverse();
    assert_eq!(
        analyze_systems_tail_traversal(&program, &entry).unwrap(),
        original
    );
    let helper = program
        .functions
        .iter_mut()
        .find(|f| f.symbol != entry)
        .unwrap();
    let mut recursive = helper.body.result.clone();
    recursive.kind = K::Call {
        symbol: helper.symbol.clone(),
        arguments: vec![Expression {
            kind: K::Local(helper.parameters[0].name.clone()),
            ..helper.body.result.clone()
        }],
    };
    helper.body.result = recursive;
    refusal(&program, &entry, "helper cycle");
}

#[test]
fn signature_and_discovery_limits_fail_closed() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001.
    let (original, entry) = counting();
    let mut program = original.clone();
    let parameter: CompilerParameter = program.functions[0].parameters[0].clone();
    entry_mut(&mut program, &entry).parameters.push(parameter);
    refusal(&program, &entry, "parameter budget");
    let mut program = original.clone();
    entry_mut(&mut program, &entry).parameters[0].source_visible = false;
    refusal(&program, &entry, "capture");
    let mut program = original.clone();
    let mut nested = edge_mut(&mut program, &entry).clone();
    for _ in 0..129 {
        let mut wrapper = nested.clone();
        wrapper.kind = K::Absolute(Box::new(nested));
        nested = wrapper;
    }
    *edge_mut(&mut program, &entry) = nested;
    refusal(&program, &entry, "budget");
}

#[test]
fn actual_region_source_retains_ordered_effects_and_legacy_refusal() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001, TOPAL-SYSTEMS-TAIL-REUSE-001.
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
    let plan = analyze_systems_tail_traversal(&program, &entry).unwrap();
    assert_eq!(plan.measure_parameter, 1);
    assert!(
        plan.functions
            .iter()
            .any(|f| f.symbol == entry && f.writes_owner && !f.reads_owner)
    );
    assert!(
        topal_language::compiler::analyze_kernel_region_framed_action_library(
            source,
            &[],
            "clear-prefix",
            3
        )
        .is_err()
    );
    let mut changed = program.clone();
    let K::Call { arguments, .. } = &mut edge_mut(&mut changed, &entry).kind else {
        panic!()
    };
    arguments[0].kind = K::Local("different-owner".into());
    refusal(&changed, &entry, "changed borrowed region");
    // A source-only symbolic selection is never an ordinary hosted capability.
    assert!(topal_language::analyze_for_compiler(source).is_err());
}

#[test]
fn checked_body_changes_cannot_keep_the_same_graph_fingerprint() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: source text alone cannot bind constructed IR.
    let (mut program, entry) = counting();
    let before = analyze_systems_tail_traversal(&program, &entry).unwrap();
    let K::Call { arguments, .. } = &mut edge_mut(&mut program, &entry).kind else {
        panic!()
    };
    let K::Binary { right, .. } = &mut arguments[1].kind else {
        panic!()
    };
    right.kind = K::Int(BigInt::from(4));
    let after = analyze_systems_tail_traversal(&program, &entry).unwrap();
    assert_eq!(before.source_sha256, after.source_sha256);
    assert_eq!(before.minimum_decrement, after.minimum_decrement);
    assert_ne!(before.checked_graph_sha256, after.checked_graph_sha256);
}

#[test]
fn imported_helper_inputs_bind_identity_path_and_exact_bytes() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: actual ordinary module selection.
    let source = "use language (version is v0.1)\nuse library support (version is v0.1)\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat (remaining - 1, support values add total)\nrepeat (4, 0)\n";
    let module = topal_language::compiler::CompilerSourceModule {
        identity: vec!["support".into(), "values".into()],
        source_name: "support/values.t".into(),
        source: "use language (version is v0.1)\npub add is fn (total : Int) -> Int\n  total + 3\n"
            .into(),
    };
    let program =
        topal_language::analyze_for_compiler_with_modules(source, &[module.clone()]).unwrap();
    let K::Call { symbol: entry, .. } = &program.main.result.kind else {
        panic!()
    };
    let plan = analyze_systems_tail_traversal(&program, entry).unwrap();
    assert_eq!(plan.dependencies.len(), 1);
    assert_eq!(plan.dependencies[0].source_name, module.source_name);
    assert_eq!(
        plan.dependencies[0].sha256,
        digest(module.source.as_bytes())
    );
    assert_eq!(plan.functions.len(), 2);
    let mut altered = program.clone();
    altered.dependencies[0].source_name.push_str(".different");
    let changed = analyze_systems_tail_traversal(&altered, entry).unwrap();
    assert_ne!(plan.dependencies, changed.dependencies);
    altered.dependencies[0].source_text.push('\n');
    let changed_again = analyze_systems_tail_traversal(&altered, entry).unwrap();
    assert_ne!(
        changed.dependencies[0].sha256,
        changed_again.dependencies[0].sha256
    );
    altered.dependencies.push(altered.dependencies[0].clone());
    refusal(&altered, entry, "duplicate dependency");
}

#[test]
fn all_branch_edges_need_progress_even_when_the_subject_is_constant() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: constructed IR exercises exhaustive inspection.
    let (mut program, entry) = counting();
    let K::OrderedComparisonDecision {
        rules, otherwise, ..
    } = &mut entry_mut(&mut program, &entry).body.result.kind
    else {
        panic!()
    };
    rules[0].operand.kind = K::Int(BigInt::from(1));
    let first = otherwise.as_ref().clone();
    let mut second = first.clone();
    let K::Call { arguments, .. } = &mut second.kind else {
        panic!()
    };
    let K::IntToNat(step) = &mut arguments[0].kind else {
        panic!()
    };
    let K::Binary { right, .. } = &mut step.kind else {
        panic!()
    };
    right.kind = K::Int(BigInt::from(2));
    let mut subject = first.clone();
    subject.kind = K::Boolean(true);
    subject.value_type = CompilerType::Boolean;
    otherwise.kind = K::BooleanDecision {
        subject: Box::new(subject),
        when_true: Box::new(first),
        when_false: Box::new(second),
    };
    let plan = analyze_systems_tail_traversal(&program, &entry).unwrap();
    assert_eq!(
        plan.back_edges
            .iter()
            .map(|e| e.decrement.as_str())
            .collect::<Vec<_>>(),
        ["1", "2"]
    );
    assert_eq!(plan.minimum_decrement, "1");
    for initial in 0_u64..=257 {
        let mut remaining = initial;
        let mut steps = 0;
        while remaining > 1 {
            remaining -= if steps % 2 == 0 { 1 } else { 2 };
            steps += 1;
        }
        assert!(steps <= initial.saturating_sub(1));
    }
    let K::BooleanDecision { when_false, .. } = &mut edge_mut(&mut program, &entry).kind else {
        panic!()
    };
    let K::Call { arguments, .. } = &mut when_false.kind else {
        panic!()
    };
    let K::IntToNat(step) = &mut arguments[0].kind else {
        panic!()
    };
    let K::Binary { right, .. } = &mut step.kind else {
        panic!()
    };
    right.kind = K::Int(BigInt::from(0));
    refusal(&program, &entry, "Nat descent");
}

#[test]
fn source_selection_rejects_unproved_packaged_recursion_and_unsupported_entries() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: same ordinary proof, no trusted systems bypass.
    let source = include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t");
    for bad in [
        source.replace("remaining - 1", "remaining"),
        source.replace("remaining - 1", "remaining - 2"),
        source.replace("Decreases remaining", "Decreases offset"),
    ] {
        assert!(
            topal_language::compiler::analyze_kernel_region_tail_source(
                &bad,
                &[],
                "clear-prefix",
                3
            )
            .is_err()
        );
    }
    for count in [0, 2, 4, 17] {
        assert!(
            topal_language::compiler::analyze_kernel_region_tail_source(
                source,
                &[],
                "clear-prefix",
                count
            )
            .is_err()
        );
    }
    let private = source.replace("pub clear-prefix", "clear-prefix");
    assert!(
        topal_language::compiler::analyze_kernel_region_tail_source(
            &private,
            &[],
            "clear-prefix",
            3
        )
        .is_err()
    );
    let defaulted = source.replace(
        "offset : Nat, status : Nat",
        "offset : Nat is 0, status : Nat",
    );
    assert!(
        topal_language::compiler::analyze_kernel_region_tail_source(
            &defaulted,
            &[],
            "clear-prefix",
            3
        )
        .is_err()
    );
}

#[test]
fn transitive_owner_effects_cannot_fit_an_explicit_empty_effect_row() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001; existing effect containment remains mandatory.
    let source = include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t");
    let mut program =
        topal_language::compiler::analyze_kernel_region_tail_source(source, &[], "clear-prefix", 3)
            .unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "clear-prefix")
        .unwrap()
        .symbol
        .clone();
    let plan = analyze_systems_tail_traversal(&program, &entry).unwrap();
    assert!(
        plan.functions
            .iter()
            .find(|f| f.symbol == entry)
            .unwrap()
            .writes_owner
    );
    // Obtain the actual empty-row representation from ordinary checked source.
    let (pure, _) = checked(
        "use language (version is v0.1)\npure is fn (value : Int) -> Int : Effects ()\n  value\npure 1\n",
    );
    entry_mut(&mut program, &entry).declared_effects = pure.functions[0].declared_effects.clone();
    assert!(entry_mut(&mut program, &entry).declared_effects.is_some());
    refusal(&program, &entry, "empty Effects");
}

#[test]
fn limits_cover_unreachable_code_flat_work_and_parameter_fields() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: no budget may drop source work silently.
    let (original, entry) = counting();
    let mut program = original.clone();
    let mut unused = program.functions[0].clone();
    unused.symbol = "unused".into();
    unused.body.result.kind = K::Int(BigInt::from(0));
    program.functions.push(unused.clone());
    refusal(&program, &entry, "unreachable");
    for index in 0..64 {
        let mut function = unused.clone();
        function.symbol = format!("unused.{index}");
        program.functions.push(function);
    }
    refusal(&program, &entry, "function identity budget");
    let mut program = original.clone();
    let mut value = program.functions[0].body.result.clone();
    value.kind = K::Int(BigInt::from(0));
    entry_mut(&mut program, &entry).body.statements =
        vec![CompilerStatement::Discard(value); 16384];
    refusal(&program, &entry, "budget");
    let mut program = original;
    for index in 0..15 {
        let mut p = program.functions[0].parameters[1].clone();
        p.name = format!("extra.{index}");
        entry_mut(&mut program, &entry).parameters.push(p);
    }
    refusal(&program, &entry, "parameter budget");
}

#[test]
fn actual_read_traversal_infers_reads_without_fabricating_write_rights() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: ordinary Optional byte access through helpers.
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
        .unwrap()
        .symbol
        .clone();
    let plan = analyze_systems_tail_traversal(&program, &entry).unwrap();
    let root = plan.functions.iter().find(|f| f.symbol == entry).unwrap();
    assert!(root.reads_owner && !root.writes_owner);
    assert_eq!(plan.maximum_one_step_function_depth, 3);
}

#[test]
fn checked_transport_widens_nat_but_never_invents_a_narrowing_proof() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001.
    let (mut program, entry) = counting();
    entry_mut(&mut program, &entry).parameters[1].value_type = CompilerType::Nat;
    refusal(&program, &entry, "classifiers");
    let (mut program, entry) = counting();
    entry_mut(&mut program, &entry).is_static = true;
    refusal(&program, &entry, "classifier");
    let (mut program, entry) = counting();
    entry_mut(&mut program, &entry).result_type = CompilerType::KernelByteRegion;
    refusal(&program, &entry, "classifier");
}

#[test]
fn shadowed_storage_cannot_fabricate_tail_progress_or_region_identity() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: a local n+1 followed by n-1 need not progress.
    use topal_language::compiler::CompilerBinding;
    let (original, entry) = counting();
    let function = &original.functions[0];
    let mut value = function.body.result.clone();
    value.kind = K::IntToNat(Box::new(Expression {
        kind: K::Int(BigInt::from(1)),
        ..value.clone()
    }));
    value.value_type = CompilerType::Nat;
    let binding = CompilerStatement::Binding(CompilerBinding {
        name: "remaining".into(),
        storage_name: function.parameters[0].name.clone(),
        value: value.clone(),
        span: value.span,
    });
    let mut program = original.clone();
    entry_mut(&mut program, &entry)
        .body
        .statements
        .push(binding.clone());
    refusal(&program, &entry, "shadowed");
    let mut program = original.clone();
    let edge = edge_mut(&mut program, &entry).clone();
    edge_mut(&mut program, &entry).kind = K::Block(Box::new(CompilerBlock {
        statements: vec![binding],
        result: edge,
    }));
    refusal(&program, &entry, "shadowed");
    let mut program = original;
    let edge = edge_mut(&mut program, &entry).clone();
    let mut subject = value.clone();
    subject.value_type = CompilerType::Optional(Box::new(CompilerType::Nat));
    subject.kind = K::OptionalSome(Box::new(value.clone()));
    let mut none = edge.clone();
    none.kind = K::Int(BigInt::from(0));
    edge_mut(&mut program, &entry).kind = K::OptionalDecision {
        subject: Box::new(subject),
        some_binding: Some(("remaining".into(), value.span)),
        some_action: Box::new(edge),
        none_action: Box::new(none),
    };
    refusal(&program, &entry, "shadowed");
    let source = include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t");
    let mut program =
        topal_language::compiler::analyze_kernel_region_tail_source(source, &[], "clear-prefix", 3)
            .unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "clear-prefix")
        .unwrap()
        .symbol
        .clone();
    value.kind = K::Local("region".into());
    value.value_type = CompilerType::KernelByteRegion;
    entry_mut(&mut program, &entry)
        .body
        .statements
        .push(CompilerStatement::Binding(CompilerBinding {
            name: "another display name".into(),
            storage_name: "region".into(),
            span: value.span,
            value,
        }));
    refusal(&program, &entry, "shadowed");
}

#[test]
fn display_name_reuse_with_distinct_storage_does_not_shadow_a_parameter() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: identity is the checked storage, not spelling.
    let (mut program, entry) = counting();
    let mut value = program.functions[0].body.result.clone();
    value.kind = K::Int(BigInt::from(1));
    entry_mut(&mut program, &entry)
        .body
        .statements
        .push(CompilerStatement::Binding(
            topal_language::compiler::CompilerBinding {
                name: "remaining".into(),
                storage_name: "distinct.remaining".into(),
                span: value.span,
                value,
            },
        ));
    let plan = analyze_systems_tail_traversal(&program, &entry).unwrap();
    assert_eq!(plan.minimum_decrement, "1");
}
