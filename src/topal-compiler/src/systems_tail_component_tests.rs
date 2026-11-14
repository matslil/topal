use super::*;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

fn work() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "tail-step-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
fn clearer() -> (CompilerProgram, String) {
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
    (program, entry)
}
fn verify(component: &SystemsTailStepComponent) {
    let work = work();
    let path = work.join("step.ll");
    fs::write(&path, &component.llvm).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let output = Command::new(tools.directory.join("opt"))
        .args(["-passes=verify", "-disable-output"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(tools.directory.join("llc"))
        .args(["-filetype=obj", "-O2"])
        .arg(&path)
        .arg("-o")
        .arg(work.join("step.o"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn actual_source_step_preserves_order_and_binds_a_distinct_runtime() {
    // TOPAL-SYSTEMS-TAIL-STEP-001. Compiler component only; no kernel grant.
    let (program, entry) = clearer();
    let component = emit_systems_tail_step_component(&program, "clear-prefix.t", &entry).unwrap();
    verify(&component);
    assert!(
        component
            .qualification
            .contains("kernel execution unqualified")
    );
    assert_ne!(
        component.region_runtime_sha256,
        component.storage.region_runtime_sha256
    );
    assert!(include_str!("runtime/kernel_regions.ll").contains("%count, 65536"));
    assert!(component.llvm.contains("%count, 18446744073709551615"));
    assert!(
        !component
            .llvm
            .contains(&format!("@topal.tail.component.{entry}("))
    );
    assert!(component.llvm.contains("clear-prefix [tail step]"));
    assert!(component.llvm.contains("@topal_tail_helper_enter_v1"));
    assert!(
        component
            .readonly_roots
            .iter()
            .any(|r| r.symbol.ends_with("runtime.int.zero") && r.bytes == 16)
    );
    let root = component
        .llvm
        .split("define i64 @topal_tail_step_component_v1")
        .nth(1)
        .unwrap();
    let arguments_done = root.find("call fastcc").unwrap();
    let first_transfer = root.find("call void @topal_tail_stage_number_v1").unwrap();
    assert!(arguments_done < first_transfer);
    assert_eq!(
        root.matches("call void @topal_tail_stage_number_v1")
            .count(),
        3
    );
}

#[test]
fn pure_signed_source_lowers_through_the_same_step_component() {
    let source = "use language (version is v0.1)\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat(remaining - 1, total - 3)\nrepeat(4, 0)\n";
    let program = topal_language::analyze_for_compiler(source).unwrap();
    let topal_language::compiler::CompilerExpressionKind::Call { symbol, .. } =
        &program.main.result.kind
    else {
        panic!()
    };
    verify(&emit_systems_tail_step_component(&program, "signed.t", symbol).unwrap());
}

fn bindings(component: &SystemsTailStepComponent, kind: u64) -> String {
    use std::fmt::Write as _;
    let plan = &component.storage.source_plan;
    let root = plan
        .functions
        .iter()
        .find(|f| f.symbol == plan.entry_symbol)
        .unwrap();
    let mut header = format!(
        "#include <stdint.h>\n#define TAIL_PARAMETERS {}\n#define TAIL_FUNCTIONS {}\n#define TAIL_DEPTH {}\n#define TAIL_ENTRY_ID {}\n#define TAIL_BANK_BYTES {}\n#define TAIL_WORK_BYTES {}\n#define TAIL_OPERAND_BITS {}\n#define TAIL_MEASURE_PARAMETER {}\n#define TAIL_TEST_KIND {kind}\n#define TAIL_FRAMES_PER_STEP UINT64_C({})\n#define TAIL_LOANS_PER_STEP UINT64_C({})\n#define TAIL_BASE_BOUND UINT64_C({})\n#define TAIL_MIN_DECREMENT UINT64_C({})\n",
        root.parameter_classifiers.len(),
        component.functions.len(),
        plan.maximum_one_step_function_depth,
        component.entry_identity,
        component.storage.bank_bytes,
        component.storage.source_step_bytes.max(8),
        component.storage.maximum_operand_bits,
        plan.measure_parameter,
        component.storage.source_frames_per_step,
        component.storage.source_region_loans_per_step,
        plan.base_bound.parse::<u64>().unwrap_or(u64::MAX),
        plan.minimum_decrement.parse::<u64>().unwrap_or(u64::MAX)
    );
    header.push_str("typedef struct { uint64_t offset, bytes, bits; unsigned kind; } TailRoot;\nstatic const TailRoot TailRoots[TAIL_PARAMETERS] = {\n");
    let mut mask = 0u64;
    for (index, classifier) in root.parameter_classifiers.iter().enumerate() {
        if classifier == "KernelByteRegion" {
            header.push_str("{0,0,0,4},\n");
            continue;
        }
        let bound = component
            .storage
            .values
            .iter()
            .find(|v| v.parameter == index)
            .unwrap();
        let kind = match classifier.as_str() {
            "Nat" => 1,
            "Int" => 2,
            "Boolean" => 3,
            _ => panic!("scalar"),
        };
        let _ = writeln!(
            header,
            "{{{},{},{},{kind}}},",
            bound.bank_offset, bound.bank_slot_bytes, bound.maximum_bits
        );
        mask |= 1u64 << index;
    }
    let _ = writeln!(header, "}};\n#define TAIL_SCALAR_MASK UINT64_C({mask})");
    header.push_str("static const unsigned char TailHasRegion[TAIL_FUNCTIONS+1] = {\n");
    for function in &component.functions {
        let _ = writeln!(
            header,
            "[{}]={},",
            function.identity,
            u8::from(function.has_region)
        );
    }
    header.push_str(
        "};\nstatic const unsigned char TailCalls[TAIL_FUNCTIONS+1][TAIL_FUNCTIONS+1] = {\n",
    );
    for function in &component.functions {
        let _ = write!(header, "[{}]={{", function.identity);
        for callee in &function.callees {
            let _ = write!(header, "[{callee}]=1,");
        }
        header.push_str("},\n");
    }
    header.push_str("};\n");
    header
}

fn native(component: &SystemsTailStepComponent, kind: u64) -> String {
    native_boundary(component, kind, false)
}

fn native_boundary(component: &SystemsTailStepComponent, kind: u64, boundary: bool) -> String {
    native_boundary_variant(component, kind, boundary, false)
}

fn native_boundary_variant(
    component: &SystemsTailStepComponent,
    kind: u64,
    boundary: bool,
    bad_outcome: bool,
) -> String {
    native_boundary_fixture(component, kind, boundary, bad_outcome, false)
}

fn native_boundary_fixture(
    component: &SystemsTailStepComponent,
    kind: u64,
    boundary: bool,
    bad_outcome: bool,
    controller_failure: bool,
) -> String {
    native_provider_fixture(
        component,
        kind,
        boundary,
        bad_outcome,
        controller_failure,
        false,
        0,
    )
}

fn native_provider_fixture(
    component: &SystemsTailStepComponent,
    kind: u64,
    boundary: bool,
    bad_outcome: bool,
    controller_failure: bool,
    native_hooks: bool,
    source_fault: u8,
) -> String {
    let work = work();
    fs::write(work.join("step.ll"), &component.llvm).unwrap();
    let mut header = bindings(component, kind);
    let mut runtime = if native_hooks {
        Some(crate::systems_tail_runtime::runtime_for_component(component.clone()).unwrap())
    } else {
        None
    };
    if (9..=10).contains(&source_fault) {
        let runtime = runtime.as_mut().unwrap();
        inject_late_cleanup_failure(&mut runtime.boundary, u8::from(source_fault == 10));
        runtime.boundary_sha256 = format!("{:x}", Sha256::digest(&runtime.boundary));
        runtime.qualification =
            "explicit late target cleanup failure fixture; no kernel qualification".into();
    }
    if source_fault >= 11 {
        let runtime = runtime.as_mut().unwrap();
        match source_fault {
            18 => runtime.llvm = runtime.llvm.replacen("call void @p.copy(ptr %output, ptr %result, i64 56)", "call void @p.copy(ptr %output, ptr %result, i64 8)\n  call void @topal_tail_abort_v1()", 1),
            19 => inject_late_cleanup_failure(&mut runtime.boundary, 2),
            20 => { let offset = runtime.llvm.rfind("ret i64 0").unwrap(); runtime.llvm.replace_range(offset..offset + "ret i64 0".len(), "ret i64 7"); },
            21 => { let anchor = "%context = call ptr @p.context_role(ptr %provided, ptr @topal_tail_complete_body_v1)"; runtime.llvm = runtime.llvm.replacen(anchor, &format!("{anchor}\n  call void @topal_tail_step_begin_v1(ptr %context)"), 1); },
            _ => (),
        }
        runtime.llvm_sha256 = format!("{:x}", Sha256::digest(runtime.llvm.as_bytes()));
        runtime.boundary_sha256 = format!("{:x}", Sha256::digest(&runtime.boundary));
        runtime.qualification =
            "explicit completion failure fixture; no kernel qualification".into();
    }
    if let Some(runtime) = &runtime {
        assert!(boundary);
        header.push_str(&format!(
            "\n#define TAIL_NATIVE_HOOKS 1\n#define TAIL_NATIVE_CONTEXT_BYTES {}\n",
            runtime.layout.context_bytes
        ));
        for (key, value) in &runtime.layout.fields {
            header.push_str(&format!("#define TAIL_NATIVE_{key} {value}\n"));
        }
        fs::write(work.join("runtime.ll"), &runtime.llvm).unwrap();
        let mut metadata = serde_json::to_value(runtime).unwrap();
        metadata.as_object_mut().unwrap().remove("llvm");
        metadata.as_object_mut().unwrap().remove("boundary");
        metadata["source_step"]
            .as_object_mut()
            .unwrap()
            .remove("llvm");
        fs::write(
            work.join("runtime.json"),
            serde_json::to_vec_pretty(&metadata).unwrap(),
        )
        .unwrap();
    }
    if boundary {
        header.push_str("\n#define TAIL_BOUNDARY_TEST 1\n#define TAIL_TEMP_BYTES 65528\n#define TAIL_PRIVATE_OFFSET 64\n");
        fs::write(
            work.join("boundary.o"),
            runtime.as_ref().map_or_else(
                || crate::emit_systems_tail_boundary_component().unwrap(),
                |r| r.boundary.clone(),
            ),
        )
        .unwrap();
    }
    if bad_outcome {
        header.push_str("\n#define TAIL_BAD_OUTCOME_TEST 1\n");
    }
    if (1..=10).contains(&source_fault) {
        assert!(native_hooks);
        header.push_str(&format!(
            "\n#define TAIL_NATIVE_SOURCE_FAULT_TEST {source_fault}\n"
        ));
    }
    if source_fault >= 11 {
        header.push_str(&format!(
            "\n#define TAIL_NATIVE_COMPLETION_FAULT_TEST {source_fault}\n"
        ));
    }
    if controller_failure {
        assert!(boundary && !bad_outcome);
        header.push_str("\n#define TAIL_CONTROLLER_FAULT_TEST 1\n");
    }
    fs::write(work.join("tail_step_bindings.h"), header).unwrap();
    let mut metadata = serde_json::to_value(component).unwrap();
    metadata.as_object_mut().unwrap().remove("llvm");
    metadata["test_kind"] = serde_json::json!(kind);
    metadata["private_stack_boundary"] = serde_json::json!(boundary);
    metadata["controller_failure_fixture"] = serde_json::json!(controller_failure);
    metadata["native_provider_hooks"] = serde_json::json!(native_hooks);
    metadata["native_source_fault_fixture"] =
        serde_json::json!(if source_fault < 9 { source_fault } else { 0 });
    metadata["native_boundary_fault_fixture"] =
        serde_json::json!(if (9..=10).contains(&source_fault) {
            source_fault - 8
        } else {
            0
        });
    metadata["native_completion_fault_fixture"] =
        serde_json::json!(source_fault.saturating_sub(10));
    fs::write(
        work.join("component.json"),
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let mut llc = Command::new(tools.directory.join("llc"));
    llc.args(["-filetype=obj", "-O2"]);
    if native_hooks {
        llc.args([
            "-mcpu=x86-64",
            "-mattr=-mmx,-sse,-sse2,-avx,-avx2",
            "--frame-pointer=all",
            "-stack-size-section",
            "-function-sections",
            "--disable-tail-calls",
        ]);
    }
    llc.arg(work.join("step.ll"))
        .arg("-o")
        .arg(work.join("step.o"));
    let output = execute(&work, "llc", &mut llc);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let labs = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64");
    let mut cc = Command::new("cc");
    cc.args([
        "-std=gnu11",
        "-O2",
        "-g",
        "-fno-omit-frame-pointer",
        "-fno-strict-aliasing",
        "-fno-pie",
        "-no-pie",
        "-Wall",
        "-Wextra",
    ])
    .arg("-I")
    .arg(&work)
    .arg("-I")
    .arg(&labs)
    .arg(labs.join("tail_step_component_client.c"))
    .arg(work.join("step.o"))
    .arg("-o")
    .arg(work.join("client"));
    if boundary {
        cc.args(["-mgeneral-regs-only", "-mno-red-zone", "-fno-builtin"])
            .arg(work.join("boundary.o"));
    }
    if native_hooks {
        let output = execute(
            &work,
            "runtime-llc",
            Command::new(tools.directory.join("llc"))
                .args([
                    "-filetype=obj",
                    "-O2",
                    "-mcpu=x86-64",
                    "-mattr=-mmx,-sse,-sse2,-avx,-avx2",
                    "--frame-pointer=all",
                    "-stack-size-section",
                    "-function-sections",
                    "--disable-tail-calls",
                ])
                .arg(work.join("runtime.ll"))
                .arg("-o")
                .arg(work.join("runtime.o")),
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let evidence = crate::systems_tail_runtime::inspection::inspect(
            &fs::read(work.join("step.o")).unwrap(),
            &fs::read(work.join("runtime.o")).unwrap(),
        )
        .unwrap();
        fs::write(
            work.join("native-inspection.json"),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .unwrap();
        for (label, object) in [
            ("source-disassembly", "step.o"),
            ("runtime-disassembly", "runtime.o"),
        ] {
            let output = execute(
                &work,
                label,
                Command::new(tools.directory.join("llvm-objdump"))
                    .args(["-d", "--no-show-raw-insn"])
                    .arg(work.join(object)),
            );
            assert!(output.status.success());
            crate::systems_library::inspect_instructions(
                &String::from_utf8(output.stdout).unwrap(),
            )
            .unwrap();
        }
        cc.arg(work.join("runtime.o"));
    }
    let output = execute(&work, "cc", &mut cc);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = execute(&work, "native", &mut Command::new(work.join("client")));
    assert!(
        output.status.success(),
        "status {}\nstdout {}\nstderr {}\nwork {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        work.display()
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TAIL_STEP_NATIVE_PASS"), "{stdout}");
    println!("{stdout}");
    if boundary && (kind == 1 || (native_hooks && kind == 8)) {
        let output = execute(
            &work,
            "gdb",
            Command::new("gdb")
                .args([
                    "--nx",
                    "--batch",
                    "-ex",
                    "set pagination off",
                    "-ex",
                    "set confirm off",
                    "-ex",
                    &format!("set $tail_bad_outcome={}", u8::from(bad_outcome)),
                    "-ex",
                    &format!(
                        "set $tail_controller_failure={}",
                        u8::from(controller_failure)
                    ),
                    "-ex",
                    &format!("set $tail_native_hooks={}", u8::from(native_hooks)),
                    "-ex",
                    &format!(
                        "set $tail_source_fault={}",
                        if source_fault < 11 { source_fault } else { 0 }
                    ),
                    "-ex",
                    &format!(
                        "set $tail_completion_fault={}",
                        if source_fault >= 11 { source_fault } else { 0 }
                    ),
                ])
                .arg(work.join("client"))
                .arg("-x")
                .arg(labs.join("tail_boundary_inspect.py")),
        );
        assert!(
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains("TAIL_BOUNDARY_GDB_PASS"),
            "GDB {}\n{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    stdout
}

fn inject_late_cleanup_failure(boundary: &mut [u8], role: u8) {
    use object::{Object, ObjectSection, ObjectSymbol};
    let file = object::File::parse(&*boundary).unwrap();
    let (entry, marker) = if role == 2 {
        (
            "topal_x86_tail_complete_call_v1",
            "topal_x86_tail_complete_reclaimed_v1",
        )
    } else if role == 1 {
        (
            "topal_x86_tail_checkpoint_call_v1",
            "topal_x86_tail_checkpoint_reclaimed_v1",
        )
    } else {
        (
            "topal_x86_tail_boundary_call_v1",
            "topal_x86_tail_boundary_return_reclaimed_v1",
        )
    };
    let symbol = |name| {
        file.symbols()
            .find(|s| s.name().ok() == Some(name))
            .unwrap()
    };
    let entry = symbol(entry);
    let section = file
        .section_by_index(entry.section_index().unwrap())
        .unwrap();
    let file_base = section.file_range().unwrap().0 as usize;
    let offset = |address: u64| file_base + (address - section.address()) as usize;
    let start = offset(entry.address());
    let normal = offset(symbol(marker).address());
    let jump = start
        + boundary[start..normal]
            .windows(2)
            .rposition(|b| b == [0x0f, 0x85])
            .unwrap();
    let displacement = i32::from_le_bytes(boundary[jump + 2..jump + 6].try_into().unwrap());
    assert_eq!(
        (jump as i64 + 6 + i64::from(displacement)) as usize,
        normal + 1
    );
    // Invert only the final header readback branch. It now takes the existing
    // failure path with a completely zero header; no source behavior is edited.
    boundary[jump + 1] = 0x84;
}

#[test]
fn native_late_cleanup_failure_retains_history_after_header_erasure() {
    // TOPAL-SYSTEMS-TAIL-CHECKPOINT-001: target failure after successful erase.
    let (program, entry) = clearer();
    let component = emit_systems_tail_step_component(&program, "clear-prefix.t", &entry).unwrap();
    for fault in 9..=10 {
        assert!(
            native_provider_fixture(&component, 1, true, false, false, true, fault)
                .contains("fatal=1")
        );
    }
}

#[test]
fn provider_completion_reclaims_output_and_invocation_on_every_failure() {
    // TOPAL-SYSTEMS-TAIL-COMPLETION-001: complete and partial output retirement.
    let (program, entry) = clearer();
    let component = emit_systems_tail_step_component(&program, "clear-prefix.t", &entry).unwrap();
    for fault in 11..=24 {
        let output = native_provider_fixture(&component, 1, true, false, false, true, fault);
        assert!(
            output.contains("cases=1 calls=5 checkpoints=5 fatal=1"),
            "{output}"
        );
        assert!(output.contains("TAIL_COMPLETION_PASS returns=0"));
    }
}

#[test]
fn provider_owned_hooks_execute_actual_clearer_and_controller_recovery() {
    // TOPAL-SYSTEMS-TAIL-RUNTIME-001. Actual freestanding hooks, no C source callbacks.
    let (program, entry) = clearer();
    let component = emit_systems_tail_step_component(&program, "clear-prefix.t", &entry).unwrap();
    let output = native_provider_fixture(&component, 1, true, false, false, true, 0);
    assert!(output.contains("cases=22") && output.contains("fatal=0"));
    let output = native_provider_fixture(&component, 1, true, false, true, true, 0);
    assert!(output.contains("cases=26 calls=26 checkpoints=1 fatal=26"));
    let mut bad = component.clone();
    let end = bad.llvm.rfind("ret i64 1").unwrap();
    bad.llvm
        .replace_range(end..end + "ret i64 1".len(), "ret i64 7");
    bad.llvm_sha256 = format!("{:x}", Sha256::digest(bad.llvm.as_bytes()));
    bad.qualification="explicit invalid machine-outcome fixture; no source correspondence or kernel qualification".into();
    assert!(native_provider_fixture(&bad, 1, true, true, false, true, 0).contains("fatal=1"));
}

#[test]
fn provider_hooks_refuse_forged_history_and_number_before_dereference() {
    let (program, entry) = clearer();
    let component = emit_systems_tail_step_component(&program, "clear-prefix.t", &entry).unwrap();
    for fault in 1..=8 {
        let mut fixture = component.clone();
        if [1, 3, 4, 5].contains(&fault) {
            let runtime =
                crate::systems_tail_runtime::runtime_for_component(component.clone()).unwrap();
            let call = "call void @topal_tail_step_begin_v1(ptr %tail.context)";
            assert_eq!(fixture.llvm.matches(call).count(), 1);
            let injected = if fault == 5 {
                "  %test.allocate = call ptr @topal_tail_allocate_v1(i64 65537)".into()
            } else {
                let field = if fault == 1 {
                    "HISTORY"
                } else if fault == 3 {
                    "FRAME_END"
                } else {
                    "LOAN_END"
                };
                let value = if fault == 1 {
                    "ptr inttoptr (i64 1 to ptr)"
                } else {
                    "i64 2"
                };
                format!(
                    "  %test.field = getelementptr i8, ptr %tail.context, i64 {}\n  store {value}, ptr %test.field, align 8",
                    runtime.layout.fields[field]
                )
            };
            fixture.llvm = fixture
                .llvm
                .replacen(call, &format!("{call}\n{injected}"), 1);
        } else {
            let start = fixture
                .llvm
                .find("call void @topal_tail_stage_number_v1(")
                .unwrap();
            let pointer = start + fixture.llvm[start..].find(", ptr ").unwrap() + 6;
            let end = pointer + fixture.llvm[pointer..].find(')').unwrap();
            let original = fixture.llvm[pointer..end].to_owned();
            if fault == 6 {
                let line_end = start + fixture.llvm[start..].find('\n').unwrap();
                let line = fixture.llvm[start..line_end].to_owned();
                fixture.llvm.insert_str(line_end, &format!("\n  {line}"));
            } else {
                let replacement = if fault == 2 {
                    "inttoptr (i64 1 to ptr)"
                } else {
                    "%test.source"
                };
                fixture.llvm.replace_range(pointer..end, replacement);
                if fault >= 7 {
                    let (base, offset) = if fault == 7 {
                        (original, 8)
                    } else {
                        let runtime =
                            crate::systems_tail_runtime::runtime_for_component(component.clone())
                                .unwrap();
                        (
                            "%tail.context".into(),
                            runtime.layout.fields["BANKS"] + component.storage.bank_bytes,
                        )
                    };
                    fixture.llvm.insert_str(
                        start,
                        &format!("%test.source = getelementptr i8, ptr {base}, i64 {offset}\n  "),
                    );
                }
            }
        }
        fixture.llvm_sha256 = format!("{:x}", Sha256::digest(fixture.llvm.as_bytes()));
        fixture.qualification="explicit invalid-context/value native fault fixture; no source correspondence or kernel qualification".into();
        assert!(
            native_provider_fixture(&fixture, 1, true, false, false, true, fault)
                .contains("fatal=1")
        );
    }
}

#[test]
fn provider_runtime_rebuilds_complete_layout_and_refuses_actual_context_overflow() {
    let (program, entry) = clearer();
    let runtime =
        crate::emit_systems_tail_runtime_component(&program, "clear-prefix.t", &entry).unwrap();
    assert_eq!(runtime.layout.context_bytes, 1248);
    assert_eq!(runtime.layout.minimum_temporary_bytes, 1320);
    assert_eq!(runtime.layout.history_bytes, 40);
    let mut oversized = runtime.source_step.clone();
    oversized.storage.source_step_bytes = 65536;
    assert!(
        crate::systems_tail_runtime::runtime_for_component(oversized)
            .unwrap_err()
            .to_string()
            .contains("actual context")
    );
    assert_eq!(
        runtime.llvm_sha256,
        format!("{:x}", Sha256::digest(runtime.llvm.as_bytes()))
    );
    assert!(
        runtime
            .qualification
            .contains("kernel execution unqualified")
    );
}

#[test]
fn private_stack_boundary_reclaims_between_step_controller_failures() {
    // TOPAL-SYSTEMS-TAIL-BOUNDARY-001: actual source stores precede controller
    // failure/cancellation; inactive recovery is not an extent authority.
    let (program, entry) = clearer();
    let component = emit_systems_tail_step_component(&program, "clear-prefix.t", &entry).unwrap();
    let output = native_boundary_fixture(&component, 1, true, false, true);
    assert!(output.contains("cases=5 calls=5 checkpoints=1 fatal=5"));
    assert!(output.contains("TAIL_CONTROLLER_FAILURE_PASS admitted=5 invalid=8"));
}

fn execute(work: &std::path::Path, label: &str, command: &mut Command) -> std::process::Output {
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|arg| arg.to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let output = command.output().unwrap();
    fs::write(work.join(format!("{label}.stdout.log")), &output.stdout).unwrap();
    fs::write(work.join(format!("{label}.stderr.log")), &output.stderr).unwrap();
    fs::write(
        work.join(format!("{label}.command.json")),
        serde_json::to_vec_pretty(
            &serde_json::json!({"argv":argv,"exit_code":output.status.code()}),
        )
        .unwrap(),
    )
    .unwrap();
    output
}

#[test]
fn actual_native_clearer_reuses_banks_beyond_the_old_extent_and_preserves_fatal_effects() {
    let (program, entry) = clearer();
    let component = emit_systems_tail_step_component(&program, "clear-prefix.t", &entry).unwrap();
    assert!(native(&component, 1).contains("fatal=15"));
}

#[test]
fn actual_native_signed_source_preserves_values_after_each_retirement() {
    let source = "use language (version is v0.1)\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat(remaining - 1, total - 3)\nrepeat(4, 0)\n";
    let program = topal_language::analyze_for_compiler(source).unwrap();
    let topal_language::compiler::CompilerExpressionKind::Call { symbol, .. } =
        &program.main.result.kind
    else {
        panic!()
    };
    native(
        &emit_systems_tail_step_component(&program, "signed.t", symbol).unwrap(),
        0,
    );
}

#[test]
fn actual_native_boolean_state_is_transferred_as_ordinary_values() {
    let source = "use language (version is v0.1)\nrepeat is fn (remaining : Nat, enabled : Boolean) -> Boolean : Decreases remaining\n  remaining\n    <= 0 then enabled\n    otherwise repeat(remaining - 1, enabled)\nrepeat(4, true)\n";
    let program = topal_language::analyze_for_compiler(source).unwrap();
    let topal_language::compiler::CompilerExpressionKind::Call { symbol, .. } =
        &program.main.result.kind
    else {
        panic!()
    };
    native(
        &emit_systems_tail_step_component(&program, "boolean.t", symbol).unwrap(),
        3,
    );
}

#[test]
fn actual_native_optional_reads_reuse_work_without_fabricating_writes() {
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
    native(
        &emit_systems_tail_step_component(&program, "read-prefix.t", &entry.symbol).unwrap(),
        2,
    );
}

#[test]
fn actual_native_readonly_multilimb_constants_are_copied_before_retirement() {
    for (sign, kind) in [("", 4), ("-", 5)] {
        let constant = (num_bigint::BigInt::from(1) << 159usize).to_string();
        let source = format!(
            "use language (version is v0.1)\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat(remaining - 1, {sign}{constant})\nrepeat(4, 0)\n"
        );
        let program = topal_language::analyze_for_compiler(&source).unwrap();
        let topal_language::compiler::CompilerExpressionKind::Call { symbol, .. } =
            &program.main.result.kind
        else {
            panic!()
        };
        native(
            &emit_systems_tail_step_component(&program, "multilimb.t", symbol).unwrap(),
            kind,
        );
    }
}

#[test]
fn actual_native_aliased_successor_roots_receive_independent_bank_copies() {
    let source = "use language (version is v0.1, features is (systems))\npub repeat is fn ((region : KernelByteRegion, remaining : Nat, first : Nat, second : Nat)) -> Boolean : Decreases remaining\n  remaining\n    <= 0 then first = second\n    otherwise repeat(region, remaining - 1, first, first)\n";
    let program =
        topal_language::compiler::analyze_kernel_region_tail_source(source, &[], "repeat", 3)
            .unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "repeat")
        .unwrap();
    native(
        &emit_systems_tail_step_component(&program, "aliases.t", &entry.symbol).unwrap(),
        6,
    );
}

#[test]
fn actual_native_unit_terminal_has_a_private_outcome_without_a_source_value() {
    let source = "use language (version is v0.1)\nrepeat is fn (remaining : Nat) -> Unit : Decreases remaining\n  remaining\n    <= 0 then ()\n    otherwise repeat(remaining - 1)\nrepeat(4)\n";
    let program = topal_language::analyze_for_compiler(source).unwrap();
    let topal_language::compiler::CompilerExpressionKind::Call { symbol, .. } =
        &program.main.result.kind
    else {
        panic!()
    };
    native(
        &emit_systems_tail_step_component(&program, "unit.t", symbol).unwrap(),
        7,
    );
}

#[test]
fn actual_private_stack_boundary_reclaims_all_steps_and_admitted_source_failures() {
    // TOPAL-SYSTEMS-TAIL-BOUNDARY-001: real target object, guarded source stack.
    let (program, entry) = clearer();
    let component = emit_systems_tail_step_component(&program, "clear-prefix.t", &entry).unwrap();
    assert!(native_boundary(&component, 1, true).contains("fatal=15"));
}

#[test]
fn private_stack_boundary_does_not_resume_after_an_invalid_machine_outcome() {
    // Explicit implementation-fault fixture: NOT a source-correspondence record.
    let (program, entry) = clearer();
    let mut component =
        emit_systems_tail_step_component(&program, "clear-prefix.t", &entry).unwrap();
    let position = component.llvm.rfind("ret i64 1").unwrap();
    component
        .llvm
        .replace_range(position..position + "ret i64 1".len(), "ret i64 7");
    component.llvm_sha256 = format!("{:x}", Sha256::digest(component.llvm.as_bytes()));
    component.qualification = "injected invalid-outcome test fixture; no source correspondence or kernel execution qualification".into();
    assert!(native_boundary_variant(&component, 1, true, true).contains("fatal=1"));
}

#[test]
fn private_stack_boundary_preserves_scalar_roots_and_readonly_owners() {
    scalar_boundary_variants(false);
}

#[test]
fn provider_hooks_preserve_all_scalar_roots_and_readonly_owners() {
    scalar_boundary_variants(true);
}

#[test]
fn provider_checkpoint_handles_nonunit_descent_and_unreachable_large_edges() {
    // TOPAL-SYSTEMS-TAIL-CHECKPOINT-001: native measure is word-bounded by descent.
    for (base, decrement) in [("3", "2"), ("36893488147419103232", "36893488147419103232")] {
        let source = format!(
            "use language (version is v0.1)\nrepeat is fn (remaining : Nat) -> Boolean : Decreases remaining\n  remaining\n    <= {base} then true\n    otherwise repeat(remaining - {decrement})\nrepeat(4)\n"
        );
        let program = topal_language::analyze_for_compiler(&source).unwrap();
        let topal_language::compiler::CompilerExpressionKind::Call { symbol, .. } =
            &program.main.result.kind
        else {
            panic!()
        };
        let component = emit_systems_tail_step_component(&program, "nonunit.t", symbol).unwrap();
        if base != "3" {
            assert_eq!(component.storage.source_step_bytes, 0);
            let runtime =
                crate::systems_tail_runtime::runtime_for_component(component.clone()).unwrap();
            assert_eq!(
                runtime.layout.fields["STARTS"] - runtime.layout.fields["WORK"],
                8
            );
            assert!(runtime.llvm.contains("[0 x i64] []"));
        }
        assert!(
            native_provider_fixture(&component, 9, true, false, false, true, 0).contains("fatal=0")
        );
    }
}

#[test]
fn provider_checkpoint_retires_helpers_with_stack_passed_arguments() {
    // TOPAL-SYSTEMS-TAIL-CHECKPOINT-001: complete helper argument and spill path.
    let source = "use language (version is v0.1, features is (systems))\nsame is fn ((a : Nat, b : Nat, c : Nat, d : Nat, e : Nat, f : Nat, g : Nat)) -> Boolean\n  (a + b + c + d + e + f + g) = (a * 7)\npub repeat is fn ((region : KernelByteRegion, remaining : Nat, a : Nat, b : Nat, c : Nat, d : Nat, e : Nat, f : Nat, g : Nat)) -> Boolean : Decreases remaining\n  remaining\n    <= 0 then same(a, b, c, d, e, f, g)\n    otherwise repeat(region, remaining - 1, a, b, c, d, e, f, g)\n";
    let program =
        topal_language::compiler::analyze_kernel_region_tail_source(source, &[], "repeat", 8)
            .unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "repeat")
        .unwrap();
    let component =
        emit_systems_tail_step_component(&program, "stack-arguments.t", &entry.symbol).unwrap();
    assert!(
        native_provider_fixture(&component, 8, true, false, false, true, 0).contains("fatal=0")
    );
}

fn scalar_boundary_variants(native_hooks: bool) {
    let variants = [
        ("repeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat(remaining - 1, total - 3)\nrepeat(4, 0)\n".to_string(), 0),
        ("repeat is fn (remaining : Nat, enabled : Boolean) -> Boolean : Decreases remaining\n  remaining\n    <= 0 then enabled\n    otherwise repeat(remaining - 1, enabled)\nrepeat(4, true)\n".to_string(), 3),
        ("repeat is fn (remaining : Nat) -> Unit : Decreases remaining\n  remaining\n    <= 0 then ()\n    otherwise repeat(remaining - 1)\nrepeat(4)\n".to_string(), 7),
    ];
    for (body, kind) in variants {
        let program = topal_language::analyze_for_compiler(&format!(
            "use language (version is v0.1)\n{body}"
        ))
        .unwrap();
        let topal_language::compiler::CompilerExpressionKind::Call { symbol, .. } =
            &program.main.result.kind
        else {
            panic!()
        };
        native_provider_fixture(
            &emit_systems_tail_step_component(&program, "private-scalars.t", symbol).unwrap(),
            kind,
            true,
            false,
            false,
            native_hooks,
            0,
        );
    }
    for (sign, kind) in [("", 4), ("-", 5)] {
        let number = (num_bigint::BigInt::from(1) << 159usize).to_string();
        let source = format!(
            "use language (version is v0.1)\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat(remaining - 1, {sign}{number})\nrepeat(4, 0)\n"
        );
        let program = topal_language::analyze_for_compiler(&source).unwrap();
        let topal_language::compiler::CompilerExpressionKind::Call { symbol, .. } =
            &program.main.result.kind
        else {
            panic!()
        };
        native_provider_fixture(
            &emit_systems_tail_step_component(&program, "private-wide.t", symbol).unwrap(),
            kind,
            true,
            false,
            false,
            native_hooks,
            0,
        );
    }
    let read = include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t").replace(
        "region byte store (offset-bytes is offset, value is 0)\n    true then 1\n    false then 0",
        "region byte load (offset-bytes is offset)\n    Some value then 1\n    None then 0",
    );
    let program =
        topal_language::compiler::analyze_kernel_region_tail_source(&read, &[], "clear-prefix", 3)
            .unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "clear-prefix")
        .unwrap();
    native_provider_fixture(
        &emit_systems_tail_step_component(&program, "private-read.t", &entry.symbol).unwrap(),
        2,
        true,
        false,
        false,
        native_hooks,
        0,
    );
    let alias = "use language (version is v0.1, features is (systems))\npub repeat is fn ((region : KernelByteRegion, remaining : Nat, first : Nat, second : Nat)) -> Boolean : Decreases remaining\n  remaining\n    <= 0 then first = second\n    otherwise repeat(region, remaining - 1, first, first)\n";
    let program =
        topal_language::compiler::analyze_kernel_region_tail_source(alias, &[], "repeat", 3)
            .unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "repeat")
        .unwrap();
    native_provider_fixture(
        &emit_systems_tail_step_component(&program, "private-aliases.t", &entry.symbol).unwrap(),
        6,
        true,
        false,
        false,
        native_hooks,
        0,
    );
}

#[test]
fn defensive_ir_tail_control_flow_uses_the_selected_branch_and_storage_identity() {
    // Defensive checked-IR structure tests; no claim that mutated source bytes
    // attest these graphs. Wrong branch selection changes -3 into +1.
    use topal_language::compiler::{CompilerBinary as Op, CompilerExpressionKind as K};
    let source = "use language (version is v0.1)\nrepeat is fn (remaining : Nat, total : Int) -> Int : Decreases remaining\n  remaining\n    <= 0 then total\n    otherwise repeat(remaining - 1, total - 3)\nrepeat(4, 0)\n";
    let original = topal_language::analyze_for_compiler(source).unwrap();
    let K::Call { symbol, .. } = &original.main.result.kind else {
        panic!()
    };
    for scenario in 0..7 {
        let mut program = original.clone();
        let function = program
            .functions
            .iter_mut()
            .find(|f| f.symbol == *symbol)
            .unwrap();
        let K::OrderedComparisonDecision {
            otherwise, rules, ..
        } = &mut function.body.result.kind
        else {
            panic!()
        };
        let correct = otherwise.as_ref().clone();
        let mut wrong = correct.clone();
        let K::Call { arguments, .. } = &mut wrong.kind else {
            panic!()
        };
        let K::Binary { right, .. } = &mut arguments[1].kind else {
            panic!()
        };
        right.kind = K::Int((-1).into());
        let mut predicate = correct.clone();
        predicate.value_type = CompilerType::Boolean;
        let selected = scenario % 2 == 0;
        if scenario < 2 {
            predicate.kind = K::Boolean(selected);
            otherwise.kind = K::BooleanDecision {
                subject: Box::new(predicate),
                when_true: Box::new(if selected {
                    correct.clone()
                } else {
                    wrong.clone()
                }),
                when_false: Box::new(if selected { wrong } else { correct }),
            };
        } else if scenario < 4 {
            predicate.value_type = CompilerType::Int;
            predicate.kind = K::Int(if selected { 1.into() } else { 2.into() });
            let mut rule = rules[0].clone();
            rule.operation = Op::Equal;
            rule.operand.kind = K::Int(1.into());
            rule.action = if selected {
                correct.clone()
            } else {
                wrong.clone()
            };
            otherwise.kind = K::OrderedComparisonDecision {
                subject: Box::new(predicate),
                rules: vec![rule],
                otherwise: Box::new(if selected { wrong } else { correct }),
            };
        } else if scenario < 6 {
            predicate.value_type = CompilerType::Optional(Box::new(CompilerType::Int));
            let mut number = correct.clone();
            number.value_type = CompilerType::Int;
            number.kind = K::Int(0.into());
            predicate.kind = if selected {
                K::OptionalSome(Box::new(number))
            } else {
                K::OptionalNone
            };
            otherwise.kind = K::OptionalDecision {
                subject: Box::new(predicate),
                some_binding: Some(("picked".into(), correct.span)),
                some_action: Box::new(if selected {
                    correct.clone()
                } else {
                    wrong.clone()
                }),
                none_action: Box::new(if selected { wrong } else { correct }),
            };
        } else {
            let mut next = correct.clone();
            let K::Call { arguments, .. } = &mut next.kind else {
                panic!()
            };
            let value = arguments[1].clone();
            arguments[1].kind = K::Local("next.total".into());
            otherwise.kind = K::Block(Box::new(topal_language::compiler::CompilerBlock {
                statements: vec![topal_language::compiler::CompilerStatement::Binding(
                    topal_language::compiler::CompilerBinding {
                        name: "total".into(),
                        storage_name: "next.total".into(),
                        span: value.span,
                        value,
                    },
                )],
                result: next,
            }));
        }
        native(
            &emit_systems_tail_step_component(&program, "defensive-ir.t", symbol).unwrap(),
            0,
        );
    }
}
