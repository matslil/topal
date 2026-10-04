use topal_language::compiler::{
    CompilerSourceModule, analyze_for_compiler, analyze_for_compiler_with_modules,
};

use super::*;

#[test]
fn emits_target_platform_runtime_and_debug_metadata() {
    let source = "use language (version is v0.1)\nrank is fn (value : Comparison) -> Int\n  value\n    Less then -1\n    Equal then 0\n    Greater then 1\nminimum is fn (left : Int, right : Int) -> Int\n  left\n    < right then left\n    otherwise right\nvalue is 40 + 2\nproduct is 6 * 7\nratio is 6 / 8\ninterval is 0 ..= 2.5\nordered is product >= value\n(value, product, ratio, 1 in interval, ordered, rank (value <=> product), value minimum product, true, \"Topal\")\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/example.t").emit();
    assert!(llvm.contains("target triple = \"x86_64-unknown-linux-gnu\""));
    assert!(llvm.contains("asm sideeffect \"syscall\""));
        assert!(llvm.contains("define void @_start() naked"));
        assert!(llvm.contains("andq $$-16, %rsp"));
        assert!(llvm.contains("@llvm.used"));
        assert!(llvm.contains("define internal ptr @memset"));
        assert!(llvm.contains("call ptr @topal.runtime.int.add"));
    assert!(llvm.contains("call ptr @topal.runtime.int.multiply"));
    assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
    assert!(llvm.contains("call ptr @topal.runtime.rational.divide"));
    assert!(llvm.contains("call ptr @topal.runtime.range.make"));
    assert!(llvm.contains("call i1 @topal.runtime.range.rational.contains"));
    assert!(llvm.contains("@llvm.ctlz.i32"));
    assert!(llvm.contains("name: \"Rational\""));
    assert!(llvm.contains("name: \"Range Rational\""));
    assert!(llvm.contains("comparison.decision.next"));
    assert!(llvm.contains("comparison.value.less"));
    assert!(llvm.contains("constant { i64, i64, [1 x i32] }"));
    assert!(llvm.contains("\\54\\6F\\70\\61\\6C"));
    assert!(llvm.contains("#dbg_value"));
    assert!(llvm.contains("Dwarf Version"));
}

#[test]
fn emits_private_effect_list_nodes_and_pointer_boundaries() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COMPILER-LIST-EFFECT-001,
    // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
    let source = "use language (version is v0.1)\nretain is fn (rows : List Effect) -> List Effect\n  rows\nrows : List Effect is Entry (Effects (), Empty)\nretain rows\n";
    let program = analyze_for_compiler(source).unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "list-effect-boundary.t").emit();

    assert!(llvm.contains(&format!("define internal fastcc ptr @{symbol}(ptr %arg0)")));
    assert!(llvm.contains(&format!("call fastcc ptr @{symbol}(ptr")));
    assert!(llvm.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(llvm.contains("store i8 0, ptr"));
    assert!(llvm.contains("getelementptr i8, ptr"));
    assert!(llvm.contains("store ptr null, ptr"));
    assert!(llvm.contains("print.list.loop"));
    assert!(llvm.contains("print.list.close.loop"));
    assert!(llvm.contains("DW_TAG_typedef, name: \"List Effect\""));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalList.Effect\""));
    assert!(llvm.contains("#dbg_value(ptr"));
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains the generated source entry")
        .1;
    assert!(!main.contains("topal.runtime.list.int"));
    assert!(!llvm.contains("topal.runtime.list.nested.int-string"));
    assert!(!llvm.contains("%topal.ListStorage"));
}

#[test]
fn emits_exact_int_list_containment_loops() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-LIST-CONTAINS-ENTRY-001,
    // TOPAL-LIST-CONTAINS-SEQUENCE-001, TOPAL-LIST-CONTAINS-SUBSEQUENCE-001,
    // TOPAL-COMPILER-LIST-INT-CONTAINMENT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-containment.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "list-containment.t").emit();

    assert!(llvm.contains("%topal.ListStorage = type { ptr, ptr }"));
    assert!(llvm.contains("store ptr "));
    assert!(llvm.contains("define internal i1 @topal.runtime.list.int.contains.entry"));
    assert!(llvm.contains("define internal i1 @topal.runtime.list.int.contains.sequence"));
    assert!(llvm.contains("define internal i1 @topal.runtime.list.int.contains.subsequence"));
    assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
    assert!(llvm.contains("call i1 @topal.runtime.list.int.contains.entry"));
    assert!(llvm.contains("call i1 @topal.runtime.list.int.contains.sequence"));
    assert!(llvm.contains("call i1 @topal.runtime.list.int.contains.subsequence"));
    assert!(!llvm.contains("topal.runtime.list.int.remove"));
    assert!(llvm.contains("DW_TAG_typedef, name: \"List Int\""));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalList.Int\""));
}

#[test]
fn emits_immutable_int_list_removal_loops() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-LIST-REMOVE-FIRST-001,
    // TOPAL-LIST-REMOVE-ALL-001, TOPAL-COMPILER-LIST-INT-REMOVAL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-removal.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "list-removal.t").emit();

    assert!(llvm.contains("%topal.ListStorage = type { ptr, ptr }"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.int.remove.first"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.int.remove.all"));
    assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
    assert!(llvm.contains("call ptr @topal.platform.allocate(i64 %allocation.length)"));
    assert!(llvm.contains("ret ptr %list"));
    assert!(llvm.contains("ret ptr %remaining"));
    assert!(llvm.contains("call ptr @topal.runtime.list.int.remove.first"));
    assert!(llvm.contains("call ptr @topal.runtime.list.int.remove.all"));
    assert!(!llvm.contains("topal.runtime.list.int.contains"));
    assert!(llvm.contains("DW_TAG_typedef, name: \"List Int\""));
}

#[test]
fn emits_basic_int_list_operations_and_total_decision() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-PREPEND-001,
    // TOPAL-LIST-APPEND-001, TOPAL-LIST-CONCAT-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-LIST-EMPTY-001, TOPAL-LIST-ONE-001, TOPAL-LIST-UNCONS-001,
    // TOPAL-LIST-FIRST-001, TOPAL-LIST-REST-001, TOPAL-LIST-REVERSE-001,
    // TOPAL-COMPILER-LIST-INT-CORE-001
    let program =
        analyze_for_compiler(include_str!("../../../../../examples/language/lists.t")).unwrap();
    let llvm = Generator::new(&program, "lists.t").emit();

    assert_eq!(llvm.matches("%topal.ListStorage = type").count(), 1);
    assert!(llvm.contains("%topal.ListUnconsStorage = type { ptr, ptr }"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.int.concat"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.int.reverse"));
    assert!(llvm.contains("define internal i1 @topal.runtime.list.int.equal"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.int.entry.count"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.int.first"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.int.rest"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.int.uncons"));
    assert!(llvm.contains("list.decision.entry"));
    assert!(llvm.contains("list.decision.empty"));
    assert!(llvm.contains("call ptr @topal.runtime.optional.some"));
    assert!(llvm.contains("call ptr @topal.runtime.optional.none"));
    assert!(!llvm.contains("topal.runtime.list.int.contains"));
    assert!(!llvm.contains("topal.runtime.list.int.remove"));
}

#[test]
fn emits_complete_list_sequence_operations_as_private_o0_control_flow() {
    // TOPAL-LIST-BOUNDARY-CHECK-001 through TOPAL-LIST-UNZIP-001,
    // TOPAL-COLLECTION-FOREACH-001, TOPAL-COLLECTION-ENTRIES-001,
    // TOPAL-COLLECTION-COLLECT-LIST-001, TOPAL-COLLECTION-COLLECT-STRING-001,
    // TOPAL-COMPILER-LIST-SEQUENCE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-sequence-operations.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "list-sequence-operations.t").emit();

    assert_eq!(llvm.matches("%topal.ListStorage = type").count(), 1);
    for helper in [
        "list.int.insert.at",
        "list.int.split.at",
        "list.int.take",
        "list.int.drop",
        "list.int.remove.index",
        "list.int.remove.index.range",
        "list.int.zip.exact",
        "list.int.zip.shortest",
        "list.int.zip.longest",
        "list.int.unzip",
        "list.int.entries",
        "list.string.collect",
    ] {
        assert!(llvm.contains(helper), "missing {helper}");
    }
    assert!(llvm.contains("list.foreach.loop"));
    assert!(llvm.contains("list.select.loop"));
    assert!(llvm.contains("xor i1"));
    assert!(llvm.contains("call ptr @topal.runtime.result.failure(i32 0"));
    assert!(llvm.contains("DW_TAG_typedef, name: \"List String\""));
    assert!(!llvm.contains("declare i8* @malloc"));
    assert!(!llvm.contains("declare i32 @printf"));
}

#[test]
fn emits_fundamental_containers_as_private_o0_collections() {
    // TOPAL-ARRAY-COLLECT-001, TOPAL-SET-COLLECT-001,
    // TOPAL-BAG-COLLECT-001, TOPAL-MAP-COLLECT-001,
    // TOPAL-COLLECTION-ENTRY-COUNT-001,
    // TOPAL-COLLECTION-EMPTY-PREDICATE-001,
    // TOPAL-ARRAY-GET-CHECKED-001, TOPAL-MAP-LOOKUP-001,
    // TOPAL-SET-CONTAINS-001, TOPAL-BAG-MULTIPLICITY-001,
    // TOPAL-COMPILER-FUNDAMENTAL-CONTAINERS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/fundamental-containers.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "fundamental-containers.t").emit();

    for helper in [
        "container.array.int.collect",
        "container.set.int.collect",
        "container.bag.int.collect",
        "container.map.string-int.collect",
        "container.entry.count",
        "container.empty",
        "container.array.int.at",
        "container.set.int.contains",
        "container.bag.int.multiplicity",
        "container.map.string-int.lookup",
    ] {
        assert!(llvm.contains(helper), "missing {helper}");
    }
    for value_type in ["Array 3 Int", "Set Int", "Bag Int", "Map (String, Int)"] {
        assert!(
            llvm.contains(&format!("DW_TAG_typedef, name: \"{value_type}\"")),
            "missing debug type {value_type}"
        );
    }
    assert!(llvm.contains("%topal.ContainerSequenceHeader = type { i64, ptr }"));
    assert!(llvm.contains("%topal.ContainerBagHeader = type { i64, i64, ptr }"));
    assert!(!llvm.contains("declare i8* @malloc"));
    assert!(!llvm.contains("declare i32 @printf"));
}

#[test]
fn emits_contextual_int_list_functions_as_finite_loops() {
    // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
    // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-LIST-INT-FUNCTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-list-functions.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "anonymous-list-functions.t").emit();

    assert!(llvm.contains("list.map.loop"));
    assert!(llvm.contains("list.select.loop"));
    assert!(llvm.contains("list.fold.loop"));
    assert!(llvm.contains("phi ptr"));
    assert!(llvm.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(!llvm.contains("topal.fn.anonymous"));
    assert!(!llvm.contains("topal.runtime.list.int.functions"));
}

#[test]
fn emits_int_pair_list_product_map_without_a_generic_runtime() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COLLECTION-MAP-001,
    // TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-LIST-INT-PAIR-MAP-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-product-pattern.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "anonymous-product-pattern.t").emit();

    assert!(llvm.contains("call ptr @topal.platform.allocate(i64 24)"));
    assert!(llvm.contains("getelementptr i8, ptr %"));
    assert!(llvm.contains("i64 16"));
    assert!(llvm.contains("list.map.loop"));
    assert!(llvm.contains("DW_TAG_typedef, name: \"List (Int, Int)\""));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalList.(Int, Int)\""));
    assert!(!llvm.contains("topal.runtime.list.pair"));
    assert!(!llvm.contains("topal.fn.anonymous"));
    assert!(!llvm.contains("call ptr %"));

    let bound = "use language (version is v0.1)\ncombine is { (left, right) } left + right\npairs : List (Int, Int) is Entry ((1, 2), Empty)\npairs map combine\n";
    let program = analyze_for_compiler(bound).unwrap();
    let llvm = Generator::new(&program, "bound-product-pattern.t").emit();
    assert!(llvm.contains("list.map.loop"));
    assert!(llvm.contains("<anonymous fn/1>"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_exact_recursive_int_string_list_nodes_and_core_loops() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-TYPE-LIST-RECURSIVE-001, TOPAL-LIST-FIRST-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-COMPILER-LIST-RECURSIVE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/nested-lists.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "nested-lists.t").emit();

    assert!(llvm.contains(&format!("define internal fastcc ptr @{symbol}(ptr %arg0)")));
    assert!(llvm.contains(&format!("call fastcc ptr @{symbol}(ptr")));
    assert!(llvm.contains("call ptr @topal.platform.allocate(i64 24)"));
    assert!(llvm.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.nested.int-string.first"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.nested.int-string.entry.count"));
    assert!(llvm.contains("define internal i1 @topal.runtime.list.nested.int-string.equal"));
    assert!(llvm.contains("call i1 @topal.runtime.list.int-string.equal"));
    assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
    assert!(llvm.contains("call i1 @topal.runtime.string.equal"));
    assert!(llvm.contains("print.list.advance"));
    assert!(llvm.contains("DW_TAG_typedef, name: \"List (Int, String)\""));
    assert!(llvm.contains("DW_TAG_typedef, name: \"List List (Int, String)\""));
    assert!(llvm.contains("TopalList.(Int, String)"));
    assert!(llvm.contains("TopalList.List (Int, String)"));
    assert!(!llvm.contains("%topal.ListStorage"));
    assert!(!llvm.contains("call ptr %"));

    let nested_int = "use language (version is v0.1)\npreserve is fn (values : List List Int) -> List List Int\n  values\ninner : List Int is Entry (1, Entry (2, Empty))\nnested : List List Int is Entry (inner, Empty)\ncopy : List List Int is Entry (inner, Empty)\nsingleton : List String is one \"solo\"\nsingleton-copy : List String is Entry (\"solo\", Empty)\n((preserve nested) = copy, entry-count nested, singleton = singleton-copy)\n";
    let program = analyze_for_compiler(nested_int).unwrap();
    let llvm = Generator::new(&program, "nested-int-lists.t").emit();
    assert!(llvm.contains("@topal.runtime.list.nested.int.equal"));
    assert!(llvm.contains("@topal.runtime.list.nested.int.entry.count"));
    assert!(llvm.contains("call i1 @topal.runtime.list.int.equal"));
    assert!(llvm.contains("DW_TAG_typedef, name: \"List List Int\""));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn specializes_bound_anonymous_int_list_functions_without_dispatch() {
    // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
    // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-LIST-INT-BOUND-FUNCTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/bound-anonymous-functions.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "bound-anonymous-functions.t").emit();

    assert!(llvm.contains("list.map.loop"));
    assert!(llvm.contains("list.select.loop"));
    assert!(llvm.contains("list.fold.loop"));
    assert!(llvm.contains("<anonymous fn/1>"));
    assert!(llvm.contains("<anonymous fn/2>"));
    assert!(!llvm.contains("topal.fn.anonymous"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_short_circuiting_int_list_fold_control_inline() {
    // TOPAL-EXEC-TRAVERSAL-CONTROL-001,
    // TOPAL-COMPILER-TRAVERSAL-CONTROL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/traversal-control.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "traversal-control.t").emit();

    assert!(llvm.contains("store i64 0, ptr"));
    assert!(llvm.contains("store i64 1, ptr"));
    assert!(llvm.contains("list.fold.continue"));
    assert!(llvm.contains("list.fold.finish"));
    assert!(llvm.contains("icmp eq i64"));
    assert!(llvm.contains("DW_TAG_typedef, name: \"TraversalControl Int\""));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalTraversalControl.Int\""));
    assert!(!llvm.contains("topal.runtime.traversal.control"));
    assert!(!llvm.contains("topal.fn.anonymous"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_int_list_range_selection_as_private_finite_loops() {
    // TOPAL-RANGE-VALUE-SELECTION-001, TOPAL-RANGE-INDEX-SELECTION-001,
    // TOPAL-COMPILER-RANGE-SELECTION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/range-selection.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "range-selection.t").emit();

    assert!(llvm.contains("call ptr @topal.runtime.list.int.select.value.range"));
    assert!(llvm.contains("call ptr @topal.runtime.list.int.select.index.range"));
    assert!(llvm.contains("define internal ptr @topal.runtime.list.int.select.range"));
    assert!(llvm.contains("call i1 @topal.runtime.range.int.contains"));
    assert!(llvm.contains("call ptr @topal.runtime.int.from.u64"));
    assert!(llvm.contains("c\"\\6F\\70\\61\""));
    assert!(!llvm.contains("RangeSelectionOf"));
    assert!(!llvm.contains("SliceOf"));
}

#[test]
fn erases_diagnostic_controls_before_llvm_lowering() {
    // TOPAL-COMPILER-DIAGNOSTIC-CONTROL-001, TOPAL-SYN-DIAG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/diagnostic-controls.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "diagnostic-controls.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module defines the source entry point")
        .1;
    assert_eq!(main.matches("call ptr @topal.runtime.int.add(").count(), 1);
    assert!(!llvm.contains("disable-warning"));
    assert!(!llvm.contains("disable-diagnostic"));
    assert!(!llvm.contains("topal.runtime.diagnostic"));
}

#[test]
fn emits_nat_comparison_with_the_exact_int_representation() {
    // TOPAL-COMPILER-NAT-COMPARISON-001
    let source = include_str!("../../../../../examples/language/nat-equality-and-ordering.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/nat-equality-and-ordering.t").emit();
    assert!(llvm.matches("call i32 @topal.runtime.int.compare").count() >= 12);
    assert!(
        llvm.matches("call i32 @topal.runtime.rational.compare")
            .count()
            >= 1
    );
    assert!(!llvm.contains("topal.runtime.nat.compare"));
    assert!(llvm.contains("name: \"Nat\""));
    assert!(llvm.contains("define internal fastcc i32 @topal.fn.compare_2dnat.0"));
    assert!(llvm.contains("DILocalVariable(name: \"left\", arg: 1"));
    assert!(llvm.contains("DILocalVariable(name: \"right\", arg: 2"));
}

#[test]
fn emits_root_local_infinity_sentinels_and_exact_range_operations() {
    // TOPAL-NUM-INFINITY-001, TOPAL-RANGE-BOUNDS-001,
    // TOPAL-RANGE-INTERSECTION-001,
    // TOPAL-COMPILER-INFINITY-001
    let source = include_str!("../../../../../examples/language/infinity-values-and-ranges.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/infinity-values-and-ranges.t").emit();

    assert!(llvm.contains("@topal.runtime.int.positive.infinity = private constant"));
    assert!(llvm.contains("@topal.runtime.int.negative.infinity = private constant"));
    assert!(llvm.contains("c\"+Infinity\""));
    assert!(llvm.contains("c\"-Infinity\""));
    assert!(llvm.matches("call i32 @topal.runtime.int.compare").count() >= 8);
    assert!(llvm.matches("call ptr @topal.runtime.range.make").count() >= 2);
    assert!(llvm.contains("call i1 @topal.runtime.range.int.contains"));
    assert!(llvm.contains("name: \"Int\""));
    assert!(llvm.contains("name: \"Nat\""));
    assert!(llvm.contains("name: \"Range Int\""));
    assert!(!llvm.contains("@topal.runtime.int.add(ptr @topal.runtime.int.positive.infinity"));
}

#[test]
fn emits_runtime_constructed_rational_infinities_and_exact_ranges() {
    // TOPAL-NUM-INFINITY-001, TOPAL-RANGE-RATIONAL-001,
    // TOPAL-COMPILER-INFINITY-001
    let source =
        include_str!("../../../../../examples/language/rational-infinity-values-and-ranges.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/rational-infinity-values-and-ranges.t").emit();

    assert!(
        llvm.matches("call ptr @topal.runtime.rational.from.int(ptr @topal.runtime.int.")
            .count()
            >= 3
    );
    assert!(llvm.contains("%either.infinity = or i1"));
    assert!(
        llvm.matches("call i32 @topal.runtime.rational.compare")
            .count()
            >= 8
    );
    assert!(llvm.contains("call i1 @topal.runtime.range.rational.contains"));
    assert!(llvm.contains("call ptr @topal.runtime.range.rational.intersection"));
    assert!(llvm.contains("name: \"Rational\""));
    assert!(llvm.contains("name: \"Range Rational\""));
    assert!(!llvm.contains("@topal.runtime.rational.positive.infinity ="));
    assert!(!llvm.contains("@topal.runtime.rational.negative.infinity ="));
}

#[test]
fn emits_o0_infinity_arithmetic_through_validating_runtime_paths() {
    // TOPAL-NUM-INFINITY-ARITHMETIC-001, TOPAL-COMPILER-INFINITY-001
    let source = include_str!("../../../../../examples/language/infinity-arithmetic.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/infinity-arithmetic.t").emit();

    for operation in ["add", "subtract", "multiply", "negate", "absolute"] {
        assert!(
            llvm.contains(&format!("call ptr @topal.runtime.int.{operation}")),
            "missing Int infinity operation {operation}"
        );
    }
    for operation in ["add", "subtract", "multiply", "negate", "absolute"] {
        assert!(
            llvm.contains(&format!("call ptr @topal.runtime.rational.{operation}")),
            "missing Rational infinity operation {operation}"
        );
    }
    assert!(llvm.contains("%either.infinity = or i1"));
    assert!(llvm.contains("label %indeterminate.infinity"));
    assert!(llvm.contains("label %make.infinity"));
    assert!(llvm.contains("name: \"Int\""));
    assert!(llvm.contains("name: \"Rational\""));
    assert!(!llvm.contains("try.multiply.infinity"));
}

#[test]
fn emits_private_infinity_function_and_aggregate_boundaries() {
    // TOPAL-NUM-INFINITY-001, TOPAL-NUM-INFINITY-ARITHMETIC-001,
    // TOPAL-COMPILER-INFINITY-001
    let source = include_str!("../../../../../examples/language/infinity-private-boundaries.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/infinity-private-boundaries.t").emit();

    for signature in [
        "define internal fastcc ptr @topal.fn.identity_2dint.0(ptr %arg0)",
        "define internal fastcc ptr @topal.fn.identity_2dnat.1(ptr %arg0)",
        "define internal fastcc ptr @topal.fn.identity_2drational.2(ptr %arg0)",
        "define internal fastcc { ptr, ptr } @topal.fn.return_2dpair.4({ ptr, ptr } %arg0)",
        "define internal fastcc { ptr, ptr, i32, i32 } @topal.fn.return_2drecord.5({ ptr, ptr, i32, i32 } %arg0)",
        "define internal fastcc ptr @topal.fn.make_2dnegative.6()",
        "define internal fastcc ptr @topal.fn.capture_2dpositive.7(ptr %arg0)",
        "define internal fastcc ptr @topal.fn.identity_2dint.8(ptr %arg0)",
    ] {
        assert!(
            llvm.contains(signature),
            "missing private signature {signature}"
        );
    }
    assert!(llvm.contains(
        "call fastcc ptr @topal.fn.identity_2dint.0(ptr @topal.runtime.int.positive.infinity)"
    ));
    assert!(llvm.contains("call fastcc { ptr, ptr } @topal.fn.return_2dpair.4"));
    assert!(llvm.contains(
        "call fastcc ptr @topal.fn.capture_2dpositive.7(ptr @topal.runtime.int.positive.infinity)"
    ));
    assert!(llvm.contains(
        "call fastcc ptr @topal.fn.identity_2dint.8(ptr @topal.runtime.int.positive.infinity)"
    ));
    assert!(llvm.contains("name: \"Int\""));
    assert!(llvm.contains("name: \"Nat\""));
    assert!(llvm.contains("name: \"Rational\""));
}

#[test]
fn emits_dynamic_infinity_multiplication_through_result_runtime_paths() {
    // TOPAL-NUM-INFINITY-ARITHMETIC-001, TOPAL-TYPE-RESULT-001,
    // TOPAL-COMPILER-INFINITY-001
    let source = include_str!("../../../../../examples/language/dynamic-infinity-results.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/dynamic-infinity-results.t").emit();

    assert_eq!(
        llvm.matches("call ptr @topal.runtime.int.try.multiply.infinity")
            .count(),
        2
    );
    assert_eq!(
        llvm.matches("call ptr @topal.runtime.rational.try.multiply.infinity")
            .count(),
        2
    );
    assert!(llvm.contains("call ptr @topal.runtime.result.failure(i32 3"));
    assert!(llvm.contains("%exactly.one.infinity = xor i1"));
    assert!(llvm.contains("br i1 %valid, label %validate.zero, label %invalid"));
    assert!(llvm.contains("name: \"Result (Int, lang arithmetic ArithmeticErrorCode)\""));
    assert!(llvm.contains("name: \"Result (Rational, lang arithmetic ArithmeticErrorCode)\""));
}

#[test]
fn emits_lexical_block_scope_metadata() {
    // TOPAL-EXEC-BLOCK-001, TOPAL-COMPILER-BLOCK-001
    let source = "use language (version is v0.1)\nvalue is 40\n{\n  value is 41\n  value + 1\n}\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/block.t").emit();
    assert!(llvm.contains("distinct !DILexicalBlock"));
    assert!(llvm.contains("DILocalVariable(name: \"value\""));
}

#[test]
fn emits_completed_as_a_retained_zero_data_result() {
    // TOPAL-EXEC-COMPLETED-001
    let source = "use language (version is v0.1)\nfinish is fn () -> Completed\n  result is Completed\n  result\nretain is fn (value : Completed) -> Completed\n  value\n(finish (), retain Completed, Completed = Completed)\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/completed.t").emit();
    assert!(llvm.contains("define internal fastcc i8 @topal.fn.finish.0"));
    assert!(llvm.contains("call fastcc i8 @topal.fn.finish.0"));
    assert!(llvm.contains("call fastcc i8 @topal.fn.retain.1(i8 0)"));
    assert!(llvm.contains("icmp eq i8 0, 0"));
    assert!(llvm.contains("ret i8 0"));
    assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Completed\""));
}

#[test]
fn emits_empty_effect_as_a_distinct_zero_data_scalar() {
    // TOPAL-EFFECT-EMPTY-001, TOPAL-EFFECT-IDENTITY-001,
    // TOPAL-EFFECT-BOUNDARY-001, TOPAL-EFFECT-PRODUCT-001
    for source in [
        include_str!("../../../../../examples/language/empty-effects.t"),
        include_str!("../../../../../examples/language/effect-classifier.t"),
        include_str!("../../../../../examples/language/effect-identity.t"),
        include_str!("../../../../../examples/language/effect-products.t"),
        include_str!("../../../../../examples/language/unit-effect-value.t"),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "empty-effects.t").emit();
        assert!(llvm.contains("name: \"Effect\""));
        assert!(llvm.contains("DIEnumerator(name: \"empty\", value: 0)"));
        assert!(!llvm.contains("topal.runtime.effect"));
    }

    let source = include_str!("../../../../../examples/language/effect-function-boundary.t");
    let program = analyze_for_compiler(source).unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "effect-function-boundary.t").emit();
    assert!(llvm.contains(&format!("define internal fastcc i8 @{symbol}(i8 %arg0)")));
    assert!(llvm.contains(&format!("call fastcc i8 @{symbol}(i8 0)")));
    assert!(!llvm.contains("topal.runtime.effect"));

    let equality = analyze_for_compiler(include_str!(
        "../../../../../examples/language/effect-identity.t"
    ))
    .unwrap();
    let llvm = Generator::new(&equality, "effect-identity.t").emit();
    assert!(llvm.contains("icmp eq i8 0, 0"));
}

#[test]
fn emits_recursive_private_tuple_function_results_and_debug_types() {
    // TOPAL-EXEC-COMPLETION-EFFECT-VALUE-001,
    // TOPAL-COMPILER-TUPLE-RESULT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/completion-effect-value.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "completion-effect-value.t").emit();
    assert!(llvm.contains(&format!("define internal fastcc {{ i8, i8 }} @{symbol}()")));
    assert!(llvm.contains("insertvalue { i8, i8 } poison, i8 0, 0"));
    assert!(llvm.contains("insertvalue { i8, i8 } %v0, i8 0, 1"));
    assert!(llvm.contains(&format!("call fastcc {{ i8, i8 }} @{symbol}()")));
    assert!(llvm.contains("extractvalue { i8, i8 }"));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"(Completed, Effect)\""));
    assert!(llvm.contains("name: \"_0\""));
    assert!(llvm.contains("name: \"_1\""));

    let nested = analyze_for_compiler(
            "use language (version is v0.1)\nmake is fn static () -> ((Int, Boolean), String)\n  ((42, true), \"Topal\")\nmake ()\n",
        )
        .unwrap();
    let llvm = Generator::new(&nested, "nested-tuple-result.t").emit();
    assert!(llvm.contains("fastcc { { ptr, i1 }, ptr }"));
    assert!(llvm.contains("insertvalue { ptr, i1 }"));
    assert!(llvm.contains("extractvalue { { ptr, i1 }, ptr }"));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"((Int, Boolean), String)\", file:"));
    assert!(llvm.contains("size: 192, align: 64"));
}

#[test]
fn emits_fieldwise_phi_nodes_for_tuple_decision_results() {
    // TOPAL-COMPILER-TUPLE-DECISION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/tuple-decision-results.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "tuple-decision-results.t").emit();
    for function in program
        .functions
        .iter()
        .filter(|function| function.source_name.starts_with("choose-"))
    {
        let definition = llvm
            .split_once(&format!("@{}(", function.symbol))
            .unwrap()
            .1
            .split_once("\n}\n")
            .unwrap()
            .0;
        assert_eq!(definition.matches("phi ptr").count(), 2, "{definition}");
        assert!(!definition.contains("phi {"), "{definition}");
    }

    let nested = analyze_for_compiler(
            "use language (version is v0.1)\nselect is fn (condition : Boolean) -> ((Int, Boolean), String)\n  condition\n    true then ((1, true), \"yes\")\n    false then ((0, false), \"no\")\nselect true\n",
        )
        .unwrap();
    let symbol = &nested.functions[0].symbol;
    let llvm = Generator::new(&nested, "nested-tuple-decision.t").emit();
    let definition = llvm
        .split_once(&format!("@{symbol}("))
        .unwrap()
        .1
        .split_once("\n}\n")
        .unwrap()
        .0;
    assert_eq!(definition.matches("phi ptr").count(), 2, "{definition}");
    assert_eq!(definition.matches("phi i1").count(), 1, "{definition}");
    assert!(!definition.contains("phi {"), "{definition}");
}

#[test]
fn emits_exact_private_tuple_parameter_prototypes_and_debug_shadows() {
    // TOPAL-COMPILER-TUPLE-PARAMETER-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/tuple-function-parameters.t"
    ))
    .unwrap();
    let symbol = |name: &str| {
        program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap()
            .symbol
            .as_str()
    };
    let llvm = Generator::new(&program, "tuple-function-parameters.t").emit();

    let retain = symbol("retain");
    assert!(llvm.contains(&format!(
        "define internal fastcc {{ {{ ptr, i1 }}, ptr }} @{retain}({{ {{ ptr, i1 }}, ptr }} %arg0)"
    )));
    assert!(llvm.contains(&format!(
        "call fastcc {{ {{ ptr, i1 }}, ptr }} @{retain}({{ {{ ptr, i1 }}, ptr }}"
    )));
    assert!(llvm.contains("extractvalue { { ptr, i1 }, ptr } %arg0, 0"));
    assert!(llvm.contains("store { { ptr, i1 }, ptr } %arg0"));
    assert!(llvm.contains("#dbg_declare(ptr"));

    let choose = symbol("choose");
    assert!(llvm.contains(&format!(
        "define internal fastcc {{ ptr, ptr }} @{choose}({{ ptr, ptr }} %arg0, i1 %arg1)"
    )));
    assert!(llvm.contains(&format!(
        "call fastcc {{ ptr, ptr }} @{choose}({{ ptr, ptr }}"
    )));

    let tuple_first = symbol("tuple-first");
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{tuple_first}({{ ptr, ptr }} %arg0)"
    )));
    let fields_first = symbol("fields-first");
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{fields_first}(ptr %arg0, ptr %arg1)"
    )));

    let discard = symbol("discard-pair");
    let discard_definition = llvm
        .split_once(&format!("@{discard}("))
        .unwrap()
        .1
        .split_once("\n}\n")
        .unwrap()
        .0;
    assert!(discard_definition.starts_with("{ i8, i8 } %arg0)"));
    assert!(!discard_definition.contains("extractvalue"));
    assert!(!llvm.contains("DILocalVariable(name: \"_\""));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"((Int, Boolean), String)\", file:"));
}

#[test]
fn emits_order_preserving_private_record_boundaries_and_debug_types() {
    // TOPAL-COMPILER-RECORD-BOUNDARY-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/record-function-boundaries.t"
    ))
    .unwrap();
    let symbol = |name: &str| {
        program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap()
            .symbol
            .as_str()
    };
    let llvm = Generator::new(&program, "record-function-boundaries.t").emit();
    let person_type = "{ i1, ptr, i32, i32 }";

    let retain = symbol("retain-person");
    assert!(llvm.contains(&format!(
        "define internal fastcc {person_type} @{retain}({person_type} %arg0)"
    )));
    assert!(llvm.contains(&format!(
        "call fastcc {person_type} @{retain}({person_type}"
    )));
    assert!(llvm.contains(&format!("extractvalue {person_type} %arg0, 0")));
    assert!(llvm.contains(&format!("extractvalue {person_type} %arg0, 3")));
    assert!(llvm.contains(&format!("store {person_type} %arg0")));

    for name in [
        "choose-person",
        "choose-ordered",
        "choose-comparison",
        "choose-enum",
        "choose-optional",
        "choose-result",
    ] {
        let choice_definition = llvm
            .split_once(&format!("@{}(", symbol(name)))
            .unwrap()
            .1
            .split_once("\n}\n")
            .unwrap()
            .0;
        assert_eq!(
            choice_definition.matches("phi i32").count(),
            2,
            "{name}: {choice_definition}"
        );
        assert!(!choice_definition.contains("phi {"), "{name}");
    }
    assert!(llvm.contains("switch i32"));

    assert!(llvm.contains("{ { i1, ptr, i32, i32 }, ptr, i32, i32 }"));
    assert!(
        llvm.contains("DW_TAG_structure_type, name: \"(active : Boolean, name : String)\", file:")
    );
    assert!(llvm.contains("name: \"active\""));
    assert!(llvm.contains("name: \"name\""));
    assert!(llvm.contains("#dbg_declare(ptr"));
    assert!(!llvm.contains("topal.runtime.record"));
}

#[test]
fn emits_fundamental_type_values_as_closed_private_tags() {
    // TOPAL-ABSTRACTION-TYPE-VALUE-001,
    // TOPAL-ABSTRACTION-TYPE-IDENTITY-001,
    // TOPAL-ABSTRACTION-TYPE-BOUNDARY-001
    for source in [
        include_str!("../../../../../examples/language/type-values.t"),
        include_str!("../../../../../examples/language/type-classifier.t"),
        include_str!("../../../../../examples/language/type-identity.t"),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "type-values.t").emit();
        assert!(!llvm.contains("topal.runtime.type"));
    }

    let classified = analyze_for_compiler(include_str!(
        "../../../../../examples/language/type-classifier.t"
    ))
    .unwrap();
    let llvm = Generator::new(&classified, "type-classifier.t").emit();
    assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Type\""));
    for (name, value) in [
        ("Boolean", 0),
        ("Int", 1),
        ("Nat", 2),
        ("Rational", 3),
        ("String", 4),
        ("Unit", 5),
        ("Scope", 6),
    ] {
        assert!(llvm.contains(&format!("DIEnumerator(name: \"{name}\", value: {value})")));
    }

    let boundary = analyze_for_compiler(include_str!(
        "../../../../../examples/language/type-function-boundary.t"
    ))
    .unwrap();
    let symbol = &boundary.functions[0].symbol;
    let llvm = Generator::new(&boundary, "type-function-boundary.t").emit();
    assert!(llvm.contains(&format!("define internal fastcc i32 @{symbol}(i32 %arg0)")));
    assert!(llvm.contains(&format!("call fastcc i32 @{symbol}(i32 1)")));

    let identity = analyze_for_compiler(include_str!(
        "../../../../../examples/language/type-identity.t"
    ))
    .unwrap();
    let llvm = Generator::new(&identity, "type-identity.t").emit();
    assert!(llvm.contains("icmp eq i32 1, 1"));
    assert!(llvm.contains("icmp eq i32 1, 4"));
}

#[test]
fn emits_named_constraint_values_as_closed_private_tags() {
    // TOPAL-ABSTRACTION-CONSTRAINT-CLASSIFIER-001,
    // TOPAL-TYPE-CONSTRAINT-001, TOPAL-COMPILER-CONSTRAINT-VALUE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/constraint-classifier.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "constraint-classifier.t").emit();
    assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Constraint\""));
    assert!(llvm.contains("DIEnumerator(name: \"<Constraint Positive>\", value: 0)"));
    assert!(llvm.contains("DIEnumerator(name: \"<Constraint rule>\", value: 1)"));
    assert!(llvm.contains("#dbg_value(i32 0"));
    assert!(llvm.contains("#dbg_value(i32 1"));
    assert!(!llvm.contains("topal.runtime.constraint"));
    assert!(!llvm.contains("define internal fastcc i1 @topal.constraint"));
}

#[test]
fn emits_constraint_validation_with_erased_base_storage_and_result_paths() {
    // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMPILER-CONSTRAINT-VALIDATE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/constraints-and-derived-capabilities.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "constraints-and-derived-capabilities.t").emit();
    assert!(llvm.contains("DIDerivedType(tag: DW_TAG_typedef, name: \"Positive\""));
    assert!(llvm.contains("constraint.accepted"));
    assert!(llvm.contains("constraint.rejected"));
    assert!(llvm.contains("constraint.merge"));
    assert!(llvm.contains("call ptr @topal.runtime.result.success"));
    assert!(llvm.contains("call ptr @topal.runtime.result.failure(i32 0"));
    assert!(llvm.contains("phi ptr"));
    assert!(llvm.contains("#dbg_declare(ptr"));
    assert!(!llvm.contains("topal.runtime.constraint"));
}

#[test]
fn emits_fundamental_constraint_bases_and_boxed_boolean_results() {
    // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMPILER-CONSTRAINT-FUNDAMENTAL-BASES-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/constraint-fundamental-bases.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "constraint-fundamental-bases.t").emit();
    for name in ["Pass", "Nonempty", "PositiveRational"] {
        assert!(
            llvm.contains(&format!(
                "DIDerivedType(tag: DW_TAG_typedef, name: \"{name}\""
            )),
            "{llvm}"
        );
    }
    assert!(llvm.contains("call ptr @topal.platform.allocate(i64 1)"));
    assert!(llvm.contains("store i1 %"));
    assert!(llvm.contains("load i1, ptr %"));
    assert!(llvm.contains("call ptr @topal.runtime.result.success"));
    assert!(llvm.contains("call ptr @topal.runtime.result.failure(i32 0"));
    assert!(!llvm.contains("topal.runtime.constraint"));
}

#[test]
fn erases_dependency_only_library_selection_and_preserves_nat_arithmetic() {
    // TOPAL-SYN-LIBRARY-001, TOPAL-LIB-DEPENDENCY-001,
    // TOPAL-COMPILER-LIBRARY-DEPENDENCY-001,
    // TOPAL-COMPILER-NAT-ARITHMETIC-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/data-transfer/packet-filter.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "packet-filter.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module defines the source entry point")
        .1;
    assert!(llvm.contains("call ptr @topal.runtime.int.add"));
    assert!(main.contains("result.project.error"));
    assert!(main.contains("ret void"));
    assert!(!llvm.contains("topal.runtime.library"));
    assert!(!llvm.contains("topal.runtime.namespace"));
}

#[test]
fn lowers_qualified_source_library_functions_to_direct_private_calls() {
    // TOPAL-LIB-SOURCE-001, TOPAL-COMPILER-LIBRARY-SOURCE-001
    let program = analyze_for_compiler_with_modules(
        include_str!("../../../../../examples/data-transfer/firewall.t"),
        &[
            CompilerSourceModule {
                identity: vec!["std".into(), "data".into(), "spans".into()],
                source_name: "library/std/data/spans.t".into(),
                source: include_str!("../../../../../library/std/data/spans.t").into(),
            },
            CompilerSourceModule {
                identity: vec!["std".into(), "network".into(), "addresses".into()],
                source_name: "library/std/network/addresses.t".into(),
                source: include_str!("../../../../../library/std/network/addresses.t").into(),
            },
        ],
    )
    .unwrap();
    let llvm = Generator::new(&program, "firewall.t").emit();
    for name in [
        "std_2edata_2espans_2espan_3f",
        "std_2edata_2espans_2espans_2doverlap_3f",
        "std_2enetwork_2eaddresses_2eipv4_3f",
        "std_2enetwork_2eaddresses_2eoctet_3f",
    ] {
        assert!(llvm.contains(name), "{name}: {llvm}");
    }
    assert!(!llvm.contains("topal.runtime.library"));
    assert!(!llvm.contains("topal.runtime.namespace"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn specializes_capability_generic_source_library_calls_directly() {
    // TOPAL-COMPILER-LIBRARY-GENERIC-001, TOPAL-LIB-SOURCE-001,
    // TOPAL-CAPABILITY-COMPOSE-001
    let program = analyze_for_compiler_with_modules(
            "use language (version is v0.1)\nuse library std (version is v0.1)\nminimum is std min\nminimum (4, 2)\n",
            &[CompilerSourceModule {
                identity: vec!["std".into()],
                source_name: "library/std/module.t".into(),
                source: include_str!("../../../../../library/std/module.t").into(),
            }],
        )
        .unwrap();
    let [minimum] = program.functions.as_slice() else {
        panic!("one selected generic function is specialized")
    };
    assert_eq!(minimum.parameters[0].value_type, CompilerType::Int);
    assert_eq!(minimum.parameters[1].value_type, CompilerType::Int);
    assert_eq!(minimum.result_type, CompilerType::Int);

    let llvm = Generator::new(&program, "generic-facade.t").emit();
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1)",
        minimum.symbol
    )));
    assert!(llvm.contains(&format!("call fastcc ptr @{}(ptr", minimum.symbol)));
    assert!(!llvm.contains("topal.runtime.generic"));
    assert!(!llvm.contains("topal.runtime.library"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn lowers_generic_optional_int_list_pair_as_one_private_pointer_payload() {
    // TOPAL-COMPILER-LIBRARY-OPTIONAL-AGGREGATE-001,
    // TOPAL-COMPILER-LIBRARY-GENERIC-001, TOPAL-TYPE-OPTIONAL-001
    let program = analyze_for_compiler_with_modules(
        include_str!("../../../../../tests/standard-library/transfer-queues.t"),
        &[
            CompilerSourceModule {
                identity: vec!["std".into()],
                source_name: "library/std/module.t".into(),
                source: include_str!("../../../../../library/std/module.t").into(),
            },
            CompilerSourceModule {
                identity: vec!["std".into(), "transfer".into(), "queues".into()],
                source_name: "library/std/transfer/queues.t".into(),
                source: include_str!("../../../../../library/std/transfer/queues.t").into(),
            },
        ],
    )
    .unwrap();
    let dequeue = program
        .functions
        .iter()
        .find(|function| function.source_name == "std.transfer.queues.dequeue")
        .unwrap();
    assert_eq!(
        dequeue.result_type,
        CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
            CompilerType::Int,
            CompilerType::List(Box::new(CompilerType::Int)),
        ])))
    );

    let llvm = Generator::new(&program, "transfer-queues.t").emit();
    assert!(llvm.contains(&format!("call fastcc ptr @{}(ptr", dequeue.symbol)));
    assert!(llvm.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(llvm.contains("call ptr @topal.runtime.optional.some(ptr"));
    assert!(llvm.contains("call ptr @topal.runtime.optional.payload(ptr"));
    assert!(llvm.contains("getelementptr i8, ptr"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn lowers_string_pair_lookup_fold_as_one_finite_private_loop() {
    // TOPAL-COMPILER-LIBRARY-STRING-PAIR-FOLD-001,
    // TOPAL-COLLECTION-FOLD-001, TOPAL-LIST-ENTRY-COUNT-001
    let program = analyze_for_compiler_with_modules(
        include_str!("../../../../../tests/standard-library/store-memory.t"),
        &[
            CompilerSourceModule {
                identity: vec!["std".into()],
                source_name: "library/std/module.t".into(),
                source: include_str!("../../../../../library/std/module.t").into(),
            },
            CompilerSourceModule {
                identity: vec!["std".into(), "store".into(), "memory".into()],
                source_name: "library/std/store/memory.t".into(),
                source: include_str!("../../../../../library/std/store/memory.t").into(),
            },
        ],
    )
    .unwrap();
    let lookup = program
        .functions
        .iter()
        .find(|function| function.source_name == "std.store.memory.lookup")
        .unwrap();
    let object_count = program
        .functions
        .iter()
        .find(|function| function.source_name == "std.store.memory.object-count")
        .unwrap();
    assert_eq!(
        lookup.result_type,
        CompilerType::Optional(Box::new(CompilerType::String))
    );
    assert_eq!(object_count.result_type, CompilerType::Nat);

    let llvm = Generator::new(&program, "store-memory.t").emit();
    let lookup_body = llvm
        .split_once(&format!("define internal fastcc ptr @{}(", lookup.symbol))
        .unwrap()
        .1;
    assert!(lookup_body.contains("list.fold.loop"));
    assert!(lookup_body.contains("getelementptr i8, ptr"));
    assert!(lookup_body.contains("i64 16"));
    assert!(lookup_body.contains("phi ptr"));
    assert!(llvm.contains(&format!("call fastcc ptr @{}(ptr", lookup.symbol)));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn lowers_nat_pair_gather_fold_as_one_finite_private_loop() {
    // TOPAL-COMPILER-LIBRARY-NAT-PAIR-FOLD-001,
    // TOPAL-COLLECTION-FOLD-001, TOPAL-NUM-NAT-001
    let program = analyze_for_compiler_with_modules(
        include_str!("../../../../../tests/standard-library/data-spans.t"),
        &[CompilerSourceModule {
            identity: vec!["std".into(), "data".into(), "spans".into()],
            source_name: "library/std/data/spans.t".into(),
            source: include_str!("../../../../../library/std/data/spans.t").into(),
        }],
    )
    .unwrap();
    let gathered_length = program
        .functions
        .iter()
        .find(|function| function.source_name == "std.data.spans.gathered-length")
        .unwrap();
    assert_eq!(gathered_length.result_type, CompilerType::Nat);
    assert_eq!(
        gathered_length.parameters[0].value_type,
        CompilerType::List(Box::new(CompilerType::Tuple(vec![
            CompilerType::Nat,
            CompilerType::Nat,
        ])))
    );

    let llvm = Generator::new(&program, "data-spans.t").emit();
    let function_body = llvm
        .split_once(&format!(
            "define internal fastcc ptr @{}(",
            gathered_length.symbol
        ))
        .unwrap()
        .1;
    assert!(function_body.contains("list.fold.loop"));
    assert!(function_body.contains("getelementptr i8, ptr"));
    assert!(function_body.contains("i64 16"));
    assert!(function_body.contains("phi ptr"));
    assert!(llvm.contains(&format!("call fastcc ptr @{}(ptr", gathered_length.symbol)));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn lowers_build_graph_string_list_folds_and_operations_directly() {
    // TOPAL-COMPILER-LIBRARY-STRING-LIST-FOLD-001,
    // TOPAL-COLLECTION-FOLD-001, TOPAL-LIST-APPEND-001,
    // TOPAL-LIST-CONTAINS-ENTRY-001
    let program = analyze_for_compiler_with_modules(
        include_str!("../../../../../tests/standard-library/build-graph.t"),
        &[CompilerSourceModule {
            identity: vec!["std".into(), "build".into(), "graph".into()],
            source_name: "library/std/build/graph.t".into(),
            source: include_str!("../../../../../library/std/build/graph.t").into(),
        }],
    )
    .unwrap();
    let selected = program
        .functions
        .iter()
        .find(|function| function.source_name == "std.build.graph.selected")
        .unwrap();
    assert_eq!(
        selected.result_type,
        CompilerType::List(Box::new(CompilerType::String))
    );

    let llvm = Generator::new(&program, "build-graph.t").emit();
    let selected_body = llvm
        .split_once(&format!("define internal fastcc ptr @{}(", selected.symbol))
        .unwrap()
        .1;
    assert!(selected_body.contains("list.fold.loop"));
    assert!(llvm.contains("@topal.runtime.list.string.contains.entry"));
    assert!(llvm.contains("@topal.runtime.list.string.concat"));
    assert!(llvm.contains("call i1 @topal.runtime.string.equal"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_root_namespace_identity_and_statically_qualified_call() {
    // TOPAL-COMPILER-ROOT-NAMESPACE-001, TOPAL-NAMESPACE-ROOT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/root-namespace.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "root-namespace.t").emit();
    assert!(llvm.contains(&llvm_bytes(b"root")));
    assert!(llvm.contains(&format!("call fastcc ptr @{symbol}(ptr")));
    assert!(!llvm.contains("topal.runtime.namespace"));
    assert!(!llvm.contains("topal.runtime.scope"));
}

#[test]
fn emits_use_namespace_as_the_existing_private_scope_identity() {
    // TOPAL-COMPILER-NAMESPACE-USE-001, TOPAL-NAMESPACE-USE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/use-namespace.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "use-namespace.t").emit();
    assert!(llvm.contains(&format!("call fastcc ptr @{symbol}(ptr")));
    assert!(llvm.contains("!DILocalVariable(name: \"current\""));
    assert!(!llvm.contains("topal.runtime.use"));
    assert!(!llvm.contains("topal.runtime.namespace"));
    assert!(!llvm.contains("topal.runtime.scope"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_function_namespace_aliases_as_direct_private_calls() {
    // TOPAL-COMPILER-NAMESPACE-FUNCTION-ALIAS-001,
    // TOPAL-NAMESPACE-ALIAS-001, TOPAL-NAMESPACE-OVERLOAD-001
    for source in [
        include_str!("../../../../../examples/language/namespace-alias.t"),
        include_str!("../../../../../examples/language/namespace-overloads.t"),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let symbols = program
            .functions
            .iter()
            .map(|function| function.symbol.clone())
            .collect::<Vec<_>>();
        let llvm = Generator::new(&program, "namespace-alias.t").emit();
        for symbol in symbols {
            assert!(llvm.lines().any(|line| {
                line.contains("call fastcc") && line.contains(&format!("@{symbol}("))
            }));
        }
        assert!(!llvm.contains("topal.runtime.namespace"));
        assert!(!llvm.contains("topal.runtime.scope"));
    }
}

#[test]
fn emits_namespace_qualified_generator_without_runtime_lookup() {
    // TOPAL-COMPILER-NAMESPACE-GENERATOR-001,
    // TOPAL-NAMESPACE-GENERATOR-001, TOPAL-GENERATOR-FOREACH-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/namespace-generator.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "namespace-generator.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert!(main.contains("#dbg_value(i32 0"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("name: \"Scope\""));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(llvm.contains("name: \"Character\""));
    assert!(!main.contains("topal.runtime.namespace"));
    assert!(!main.contains("topal.runtime.scope"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_namespace_data_members_as_stable_ssa_references() {
    // TOPAL-COMPILER-NAMESPACE-DATA-001, TOPAL-NAMESPACE-SNAPSHOT-001
    let program = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 40 + 2\napi is root\n{\n  answer is 0\n  (api answer, root answer, answer)\n}\n",
        )
        .unwrap();
    let llvm = Generator::new(&program, "namespace-data.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module defines the source entry point")
        .1;
    assert_eq!(
        main.matches("call ptr @topal.runtime.int.add(").count(),
        1,
        "the captured initializer must execute exactly once"
    );
    assert!(!llvm.contains("topal.runtime.namespace"));
    assert!(!llvm.contains("topal.runtime.scope"));

    for source in [
        include_str!("../../../../../examples/language/namespace-alias-chain.t"),
        include_str!("../../../../../examples/language/namespace-snapshot.t"),
        include_str!("../../../../../examples/language/scope-classifier.t"),
        include_str!("../../../../../examples/language/published-root-member.t"),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "namespace-data.t").emit();
        assert!(!llvm.contains("topal.runtime.namespace"));
        assert!(!llvm.contains("topal.runtime.scope"));
    }
}

#[test]
fn emits_scope_parameters_with_private_data_environment_arguments() {
    // TOPAL-COMPILER-NAMESPACE-BOUNDARY-001,
    // TOPAL-NAMESPACE-FUNCTION-BOUNDARY-001
    let source = "use language (version is v0.1)\nanswer is 40 + 2\nincrement is fn (value : Int) -> Int\n  value + 1\nread-answer is fn (api : Scope) -> Int\n  api answer\napply is fn (api : Scope, value : Int) -> Int\n  api increment value\nforward is fn (api : Scope) -> Int\n  read-answer api\n(read-answer root, apply root 41, forward root)\n";
    let program = analyze_for_compiler(source).unwrap();
    let forward = program
        .functions
        .iter()
        .find(|function| function.source_name == "forward")
        .unwrap();
    let llvm = Generator::new(&program, "scope-parameter.t").emit();
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(i32 %arg0, ptr %arg1)",
        forward.symbol
    )));
    assert!(llvm.lines().any(|line| {
        line.contains("call fastcc ptr") && line.contains("(i32 %arg0, ptr %arg1)")
    }));
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module defines the source entry point")
        .1;
    assert_eq!(
        main.matches("call ptr @topal.runtime.int.add(").count(),
        1,
        "the captured namespace initializer executes once"
    );
    assert!(llvm.contains("!DILocalVariable(name: \"api\", arg: 1"));
    assert!(llvm.contains("!DILocalVariable(name: \"api answer\", arg: 2"));
    assert!(!llvm.contains("topal.runtime.namespace"));
    assert!(!llvm.contains("topal.runtime.scope"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn forwards_live_root_data_as_exact_private_arguments() {
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-NAMESPACE-ROOT-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-root-data-forwarding.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "function-root-data-forwarding.t").emit();
    for name in ["read", "relay", "forward"] {
        let symbol = &program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap()
            .symbol;
        assert!(llvm.contains(&format!(
            "define internal fastcc {{ ptr, ptr, ptr }} @{symbol}(ptr %arg0, ptr %arg1, ptr %arg2)"
        )));
    }
    let read = &program
        .functions
        .iter()
        .find(|function| function.source_name == "read")
        .unwrap()
        .symbol;
    let relay = &program
        .functions
        .iter()
        .find(|function| function.source_name == "relay")
        .unwrap()
        .symbol;
    assert!(llvm.contains(&format!(
        "call fastcc {{ ptr, ptr, ptr }} @{read}(ptr %arg0, ptr %arg1, ptr %arg2)"
    )));
    assert!(llvm.contains(&format!(
        "call fastcc {{ ptr, ptr, ptr }} @{relay}(ptr %arg0, ptr %arg1, ptr %arg2)"
    )));
    assert_eq!(
        llvm.matches("!DILocalVariable(name: \"root label\", arg: 2")
            .count(),
        3
    );
    assert_eq!(
        llvm.matches("!DILocalVariable(name: \"root answer\", arg: 3")
            .count(),
        3
    );
    assert!(llvm.matches("alloca ptr, align 8").count() >= 6);
    assert!(!llvm.contains("topal.runtime.namespace"));
    assert!(!llvm.contains("topal.runtime.scope"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_defining_context_as_a_private_direct_capture_parameter() {
    // TOPAL-COMPILER-CONTEXT-CAPTURE-001, TOPAL-CONTEXT-SELECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/constructed-context.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "constructed-context.t").emit();
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{symbol}(ptr %arg0, ptr %arg1)"
    )));
    assert!(llvm.lines().any(|line| {
        line.contains("call fastcc ptr")
            && line.contains(&format!("@{symbol}(ptr @.topal.int.1, ptr @.topal.int.0)"))
    }));
    assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
    assert!(llvm.contains("!DILocalVariable(name: \"@ offset\", arg: 2"));
    assert!(!llvm.contains("topal.runtime.context"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));

    let computed = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 20 + 20\nadd-offset is fn (value : Int) -> Int\n  value + @ offset\nadd-offset 2\n",
        )
        .unwrap();
    let llvm = Generator::new(&computed, "computed-context.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module defines the source entry point")
        .1;
    assert_eq!(
        main.matches("call ptr @topal.runtime.int.add(").count(),
        1,
        "the defining-context initializer must execute exactly once"
    );
}

#[test]
fn forwards_defining_context_as_exact_private_arguments() {
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
    // TOPAL-CONTEXT-SELECT-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/defining-context-forwarding.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "defining-context-forwarding.t").emit();
    for name in ["read", "relay", "forward"] {
        let symbol = &program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap()
            .symbol;
        assert!(llvm.contains(&format!(
            "define internal fastcc {{ ptr, ptr, ptr }} @{symbol}(ptr %arg0, ptr %arg1, ptr %arg2)"
        )));
    }
    let read = &program
        .functions
        .iter()
        .find(|function| function.source_name == "read")
        .unwrap()
        .symbol;
    let relay = &program
        .functions
        .iter()
        .find(|function| function.source_name == "relay")
        .unwrap()
        .symbol;
    assert!(llvm.contains(&format!(
        "call fastcc {{ ptr, ptr, ptr }} @{read}(ptr %arg0, ptr %arg1, ptr %arg2)"
    )));
    assert!(llvm.contains(&format!(
        "call fastcc {{ ptr, ptr, ptr }} @{relay}(ptr %arg0, ptr %arg1, ptr %arg2)"
    )));
    assert_eq!(
        llvm.matches("!DILocalVariable(name: \"@ offset\", arg: 2")
            .count(),
        3
    );
    assert_eq!(
        llvm.matches("!DILocalVariable(name: \"@ label\", arg: 3")
            .count(),
        3
    );
    assert!(llvm.matches("alloca ptr, align 8").count() >= 6);
    assert!(!llvm.contains("topal.runtime.context"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn forwards_exact_scalar_environments_across_proven_recursion() {
    // TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
    // TOPAL-COMPILER-DEBUG-001
    let source = include_str!("../../../../../examples/language/recursive-scalar-environments.t")
        .replace("(cycle-even 3, cycle-odd 3)", "cycle-even 3");
    let program = analyze_for_compiler(&source).unwrap();
    let llvm = Generator::new(&program, "recursive-scalar-environments.t").emit();
    assert_eq!(program.functions.len(), 2);
    let even = program
        .functions
        .iter()
        .find(|function| function.source_name == "cycle-even")
        .unwrap();
    let odd = program
        .functions
        .iter()
        .find(|function| function.source_name == "cycle-odd")
        .unwrap();
    for function in [even, odd] {
        assert!(llvm.contains(&format!(
                "define internal fastcc {{ i1, ptr, ptr }} @{}(ptr %arg0, ptr %arg1, ptr %arg2) nounwind noinline",
                function.symbol
            )));
    }
    for (caller, callee) in [(even, odd), (odd, even)] {
        let definition = llvm
            .split_once(&format!("@{}(", caller.symbol))
            .expect("caller definition is emitted")
            .1;
        assert!(definition.contains(&format!(
            "call fastcc {{ i1, ptr, ptr }} @{}(ptr %v4, ptr %arg1, ptr %arg2)",
            callee.symbol
        )));
    }
    assert_eq!(
        llvm.matches("!DILocalVariable(name: \"@ captured\", arg: 2")
            .count(),
        2
    );
    assert_eq!(
        llvm.matches("!DILocalVariable(name: \"root live\", arg: 3")
            .count(),
        2
    );
    assert!(!llvm.contains("topal.runtime.context"));
    assert!(!llvm.contains("topal.runtime.namespace"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn erases_static_empty_effect_view_before_llvm_lowering() {
    // TOPAL-FUNCTION-EFFECT-BOUND-001, TOPAL-EFFECT-CONTAIN-001,
    // TOPAL-INTRO-STATIC-001, TOPAL-INTRO-VIEW-001,
    // TOPAL-COMPILER-FUNCTION-EMPTY-EFFECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-effect-bound.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "function-effect-bound.t").emit();
    assert!(llvm.contains(&format!("define internal fastcc ptr @{symbol}(ptr %arg0)")));
    assert!(
        llvm.lines().any(|line| {
            line.contains("call fastcc ptr") && line.contains(&format!("@{symbol}("))
        })
    );
    assert!(!llvm.contains("FunctionView"));
    assert!(!llvm.contains("signature"));
    assert!(!llvm.contains("topal.runtime.introspection"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn erases_static_introspection_and_lowers_numeric_version() {
    // TOPAL-INTRO-QUALIFIED-001, TOPAL-INTRO-STATIC-001,
    // TOPAL-INTRO-VIEW-001, TOPAL-INTRO-CONTEXT-001,
    // TOPAL-INTRO-RELATION-001, TOPAL-COMPILER-STATIC-INTROSPECTION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/static-introspection.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "static-introspection.t").emit();
    assert!(llvm.contains("alloca { ptr, ptr, ptr, ptr }, align 8"));
    assert!(llvm.contains("name: \"Version\""));
    assert!(llvm.contains("name: \"major\""));
    assert!(llvm.contains("name: \"minor\""));
    assert!(llvm.contains("name: \"patch\""));
    assert!(llvm.contains("name: \"build\""));
    assert!(!llvm.contains("integer-identity"));
    assert!(!llvm.contains("integer-view"));
    assert!(!llvm.contains("current-context"));
    assert!(!llvm.contains("topal.runtime.introspection"));
    assert!(!llvm.contains("topal.runtime.version"));
}

#[test]
fn lowers_the_lint_namespace_as_an_authority_free_scope_value() {
    // TOPAL-SYN-CONTEXT-001, TOPAL-LINT-VARIANT-001,
    // TOPAL-COMPILER-LINT-VARIANT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/lint-language-variant.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "lint-language-variant.t").emit();

    assert!(llvm.contains(&llvm_bytes(b"<namespace lang lint>")));
    assert!(!llvm.contains("topal.runtime.lint"));
    assert!(!llvm.contains("topal.runtime.namespace"));
    assert!(!llvm.contains("topal.runtime.scope"));
    assert!(!llvm.contains("call ptr %"));

    let debug_program = analyze_for_compiler(
            "use language (version is v0.1, features is (lint))\nlint-scope : Scope is lang lint\nlint-scope\n",
        )
        .unwrap();
    let debug_llvm = Generator::new(&debug_program, "lint-scope-debug.t").emit();
    assert!(debug_llvm.contains("!DIEnumerator(name: \"<namespace root>\", value: 0)"));
    assert!(debug_llvm.contains("!DIEnumerator(name: \"<namespace lang lint>\", value: 1)"));
}
