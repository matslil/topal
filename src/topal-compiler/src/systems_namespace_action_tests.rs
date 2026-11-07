use super::*;

const BYTES: &str = include_str!("../../../linux-kernel/kernel/support/namespace-bytes.t");
const NAMESPACE: &str = include_str!("../../../linux-kernel/kernel/support/namespace-metadata.t");
const ACTION: &str = include_str!("../../../linux-kernel/kernel/policy/namespace-action.t");

#[path = "systems_namespace_oracle_tests.rs"]
mod native_oracle;

fn namespace_action_input(work: &Path) -> SystemsSourceLibraryInput {
    let path = work.join("namespace-action.t");
    std::fs::write(&path, ACTION).unwrap();
    std::fs::create_dir(work.join("support")).unwrap();
    let modules = [
        ("namespace-bytes", BYTES),
        ("namespace-metadata", NAMESPACE),
    ]
    .into_iter()
    .map(|(name, text)| {
        std::fs::write(work.join(format!("support/{name}.t")), text).unwrap();
        crate::SystemsSourceModuleInput {
            source: format!("support/{name}.t"),
            identity: vec!["support".into(), name.into()],
            text: text.into(),
        }
    })
    .collect();
    SystemsSourceLibraryInput {
        source: "policy/namespace-action.t".into(),
        source_name: path.to_string_lossy().into(),
        text: ACTION.into(),
        modules,
    }
}

#[test]
#[allow(clippy::too_many_lines)] // One exact source object ties provenance, costs, native effects and GDB evidence.
fn namespace_metadata_action_has_complete_bounds_and_exact_native_effects() {
    // TOPAL-SYSTEMS-NAMESPACE-ACTION-001; no selected native kernel claim.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let input = namespace_action_input(&work);
    let object = build_systems_region_instance_library(
        &input,
        "apply",
        2,
        "namespace-metadata",
        &tools,
        &work,
    )
    .unwrap();
    verify_systems_region_instance_library(
        &input,
        "apply",
        2,
        "namespace-metadata",
        &tools,
        &temporary_work(),
        &object,
    )
    .unwrap();
    for case in 0..4 {
        let mut candidate = object.clone();
        match case {
            0 => candidate.record.instance = Some("inode-metadata".into()),
            1 => candidate.record.allocation.numeric.maximum_bytes -= 8,
            2 => candidate.object[0] ^= 1,
            _ => candidate.module_snapshot.push(0),
        }
        assert!(
            verify_systems_region_instance_library(
                &input,
                "apply",
                2,
                "namespace-metadata",
                &tools,
                &temporary_work(),
                &candidate,
            )
            .is_err(),
            "namespace binding case {case}"
        );
        eprintln!("NAMESPACE_BINDING_REFUSAL_PASS case={case}");
    }
    eprintln!(
        "NAMESPACE_ACTION_COMPONENT temporary={} stack={} functions={} work={}",
        object.record.allocation.numeric.maximum_bytes,
        object.record.maximum_stack_bytes,
        object.record.functions.len(),
        work.display()
    );
    assert!(object.record.allocation.numeric.maximum_bytes <= 65536);
    assert!(object.record.maximum_stack_bytes <= 4096);
    std::fs::write(
        work.join("namespace-action.record.json"),
        serde_json::to_vec_pretty(&object.record).unwrap(),
    )
    .unwrap();
    let symbols = object.record.native_symbols().unwrap();
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/namespace_action_client.c");
    for debug in [false, true] {
        let name = if debug {
            "namespace-action-debug-client"
        } else {
            "namespace-action-client"
        };
        let executable = work.join(name);
        let mut compile = Command::new("cc");
        compile
            .args([
                "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2", "-g", "-no-pie",
            ])
            .arg(format!("-DNS_EXPORT={}", symbols.export))
            .arg(format!("-DNS_ALLOCATE={}", symbols.allocate))
            .arg(format!("-DNS_ABORT={}", symbols.abort))
            .arg(format!(
                "-DNS_LIMIT={}",
                object.record.allocation.numeric.maximum_bytes
            ))
            .arg(&client)
            .arg(work.join("kernel-region.o"))
            .arg("-o")
            .arg(&executable);
        if debug {
            compile.arg("-DNS_DEBUG=1");
        }
        run(&mut compile).unwrap();
        let output = if debug {
            let script = client.with_file_name("namespace_action_debug.py");
            run(Command::new("gdb")
                .args(["-q", "--batch", "-ex", "set debuginfod enabled off", "-x"])
                .arg(script)
                .arg(&executable))
            .unwrap()
        } else {
            run(&mut Command::new(&executable)).unwrap()
        };
        let expected = if debug {
            "NAMESPACE_ACTION_DEBUG_PASS returns=4 stores=121 staged-input-stores=0"
        } else {
            "NAMESPACE_ACTION_NATIVE_PASS calls=8594"
        };
        assert!(output.contains(expected), "{output}");
        std::fs::write(work.join(format!("{name}.log")), &output).unwrap();
        eprintln!("{output}");
    }
}

fn namespace_bytes_input(work: &Path) -> SystemsSourceLibraryInput {
    let text = r#"use language (version is v0.1, features is (systems))
use library support (version is v0.1)
check-zero is fn ((region : KernelByteRegion, operation : Nat)) -> Boolean
  operation = 3
    true then support namespace-bytes zero-name (region, 1536)
    false then false
check-clear is fn ((region : KernelByteRegion, operation : Nat)) -> Boolean
  operation = 2
    true then support namespace-bytes clear-name (region, 1536)
    false then check-zero (region, operation)
check-copy is fn ((region : KernelByteRegion, operation : Nat)) -> Boolean
  operation = 1
    true then support namespace-bytes copy-name (region, 1536, 3200)
    false then check-clear (region, operation)
select-probe is fn ((region : KernelByteRegion, operation : Nat, length : Nat)) -> Boolean
  operation = 0
    true then support namespace-bytes valid-input (region, length)
    false then check-copy (region, operation)
pub apply is fn ((region : KernelByteRegion, operation : Nat, length : Nat)) -> Boolean
  extent is region byte count
  extent >= 4096
    true then select-probe (region, operation, length)
    false then false
"#;
    let path = work.join("namespace-bytes-probe.t");
    std::fs::write(&path, text).unwrap();
    std::fs::create_dir(work.join("support")).unwrap();
    std::fs::write(work.join("support/namespace-bytes.t"), BYTES).unwrap();
    SystemsSourceLibraryInput {
        source: "policy/namespace-bytes-probe.t".into(),
        source_name: path.to_string_lossy().into(),
        text: text.into(),
        modules: vec![crate::SystemsSourceModuleInput {
            source: "support/namespace-bytes.t".into(),
            identity: vec!["support".into(), "namespace-bytes".into()],
            text: BYTES.into(),
        }],
    }
}

#[test]
fn qualified_namespace_source_can_fail_after_stores_or_return_a_wrong_association() {
    // TOPAL-SYSTEMS-NAMESPACE-ACTION-001. Witnesses why an owning controller
    // needs independent completion and fatal cleanup. This caller is not that
    // kernel controller and does not claim its private-stack reclamation.
    let tools = LlvmTools::discover(None).unwrap();
    for wrong_association in [false, true] {
        let work = temporary_work();
        let mut input = namespace_action_input(&work);
        input.text = if wrong_association {
            "use language (version is v0.1, features is (systems))\nuse library support (version is v0.1)\ndamage is fn (region : KernelByteRegion) -> Boolean\n  region unsigned store (offset-bytes is 64, byte-count is 8, byte-order is little-endian, value is 999)\npub apply is fn ((region : KernelByteRegion, operation : Nat, seconds : Nat)) -> Boolean\n  changed is support namespace-metadata apply (region, operation, seconds)\n  changed\n    true then damage region\n    false then false\n".into()
        } else {
            ACTION.replace(
                "  support namespace-metadata apply (region, operation, seconds)",
                "  _ is support namespace-metadata apply (region, operation, seconds)\n  false",
            )
        };
        std::fs::write(&input.source_name, &input.text).unwrap();
        let object = build_systems_region_instance_library(
            &input,
            "apply",
            2,
            "namespace-metadata",
            &tools,
            &work,
        )
        .unwrap();
        assert!(object.record.allocation.numeric.maximum_bytes <= 65536);
        assert!(object.record.maximum_stack_bytes <= 4096);
        let symbols = object.record.native_symbols().unwrap();
        let client = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../linux-kernel/labs/qemu/x86_64/namespace_action_client.c");
        let executable = work.join("namespace-fault-client");
        run(Command::new("cc")
            .args([
                "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2", "-g", "-no-pie",
            ])
            .arg(if wrong_association {
                "-DNS_WRONG_ASSOCIATION=1"
            } else {
                "-DNS_FALSE_RESULT=1"
            })
            .arg(format!("-DNS_EXPORT={}", symbols.export))
            .arg(format!("-DNS_ALLOCATE={}", symbols.allocate))
            .arg(format!("-DNS_ABORT={}", symbols.abort))
            .arg(format!(
                "-DNS_LIMIT={}",
                object.record.allocation.numeric.maximum_bytes
            ))
            .arg(client)
            .arg(work.join("kernel-region.o"))
            .arg("-o")
            .arg(&executable))
        .unwrap();
        let output = run(&mut Command::new(&executable)).unwrap();
        let marker = if wrong_association {
            "NAMESPACE_WRONG_ASSOCIATION_COMPONENT_PASS calls=8594"
        } else {
            "NAMESPACE_FALSE_AFTER_STORE_COMPONENT_PASS calls=8594"
        };
        assert!(output.contains(marker), "{output}");
        std::fs::write(work.join("namespace-fault-client.log"), &output).unwrap();
        std::fs::write(
            work.join("namespace-fault.record.json"),
            serde_json::to_vec_pretty(&object.record).unwrap(),
        )
        .unwrap();
        eprintln!(
            "{output}NAMESPACE_FAULT_COMPONENT wrong_association={wrong_association} stack={} temporary={} work={}",
            object.record.maximum_stack_bytes,
            object.record.allocation.numeric.maximum_bytes,
            work.display()
        );
    }
}

#[test]
fn namespace_fixed_byte_helpers_have_complete_source_bounds() {
    // TOPAL-SYSTEMS-NAMESPACE-ACTION-001; component only, no live namespace grant.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let input = namespace_bytes_input(&work);
    let object = build_systems_region_instance_library(
        &input,
        "apply",
        2,
        "namespace-bytes-probe",
        &tools,
        &work,
    )
    .unwrap();
    eprintln!(
        "NAMESPACE_BYTES_COMPONENT temporary={} stack={} functions={} work={}",
        object.record.allocation.numeric.maximum_bytes,
        object.record.maximum_stack_bytes,
        object.record.functions.len(),
        work.display()
    );
    assert!(object.record.allocation.numeric.maximum_bytes <= 65536);
    assert!(object.record.maximum_stack_bytes <= 4096);
    let symbols = object.record.native_symbols().unwrap();
    let executable = work.join("namespace-bytes-client");
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/namespace_bytes_client.c");
    run(Command::new("cc")
        .args([
            "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2", "-g", "-no-pie",
        ])
        .arg(format!("-DNS_EXPORT={}", symbols.export))
        .arg(format!("-DNS_ALLOCATE={}", symbols.allocate))
        .arg(format!("-DNS_ABORT={}", symbols.abort))
        .arg(format!(
            "-DNS_LIMIT={}",
            object.record.allocation.numeric.maximum_bytes
        ))
        .arg(client)
        .arg(work.join("kernel-region.o"))
        .arg("-o")
        .arg(&executable))
    .unwrap();
    let output = run(&mut Command::new(&executable)).unwrap();
    assert!(
        output.contains("NAMESPACE_BYTES_NATIVE_PASS calls=8440"),
        "{output}"
    );
    eprintln!("{output}");
}

#[test]
fn shared_region_code_still_charges_every_view_and_refuses_the_original_limit() {
    // TOPAL-SYSTEMS-NAMESPACE-ACTION-001; code sharing is not resource sharing.
    let tools = LlvmTools::discover(None).unwrap();
    for repetitions in [80, 256] {
        let work = temporary_work();
        let mut text = "use language (version is v0.1, features is (systems))\nmake-window is fn (region : KernelByteRegion) -> Boolean\n  selected : Optional KernelByteRegion is region byte window (offset-bytes is 0, byte-count is 1, rights is read-only)\n  selected\n    Some window then true\n    None then false\npub apply is fn ((region : KernelByteRegion, first : Nat, second : Nat)) -> Boolean\n".to_owned();
        for index in 0..repetitions {
            text.push_str(&format!("  viewed-{index} is make-window region\n"));
        }
        text.push_str("  true\n");
        let source = work.join("view-count.t");
        std::fs::write(&source, &text).unwrap();
        let input = SystemsSourceLibraryInput {
            source: "policy/view-count.t".into(),
            source_name: source.to_string_lossy().into(),
            text,
            modules: vec![],
        };
        let result = build_systems_region_instance_library(
            &input,
            "apply",
            2,
            "namespace-view-count-probe",
            &tools,
            &work,
        );
        if repetitions == 80 {
            let object = result.unwrap();
            assert_eq!(object.record.allocation.maximum_view_nodes, 81);
            assert_eq!(
                object.record.allocation.numeric.source_evaluation_bytes,
                6400
            );
            assert_eq!(object.record.allocation.numeric.maximum_bytes, 6544);
        } else {
            assert!(
                result
                    .err()
                    .expect("257 views must exceed the unchanged bound")
                    .to_string()
                    .contains("region source exceeds issued-view bound")
            );
        }
    }
}
