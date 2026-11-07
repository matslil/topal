use super::*;

const ACTION: &str = include_str!("../../../linux-kernel/kernel/policy/inode-mode-action.t");
const METADATA: &str = include_str!("../../../linux-kernel/kernel/support/inode-metadata.t");

fn inode_input(work: &Path, source: &str) -> SystemsSourceLibraryInput {
    let source_path = work.join("inode-mode-action.t");
    std::fs::write(&source_path, source).unwrap();
    std::fs::create_dir(work.join("support")).unwrap();
    std::fs::write(work.join("support/inode-metadata.t"), METADATA).unwrap();
    SystemsSourceLibraryInput {
        source: "policy/inode-mode-action.t".into(),
        source_name: source_path.to_string_lossy().into(),
        text: source.into(),
        modules: vec![crate::SystemsSourceModuleInput {
            source: "support/inode-metadata.t".into(),
            identity: vec!["support".into(), "inode-metadata".into()],
            text: METADATA.into(),
        }],
    }
}

fn inode_client(work: &Path, record: &SystemsRegionLibraryArtifact) -> std::path::PathBuf {
    let symbols = record.native_symbols().unwrap();
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/inode_mode_client.c");
    let executable = work.join("inode-mode-client");
    run(Command::new("cc")
        .args([
            "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2", "-g", "-no-pie",
        ])
        .arg(format!("-DINODE_EXPORT={}", symbols.export))
        .arg(format!("-DINODE_ALLOCATE={}", symbols.allocate))
        .arg(format!("-DINODE_ABORT={}", symbols.abort))
        .arg(format!(
            "-DINODE_SOURCE_LIMIT={}",
            record.allocation.numeric.maximum_bytes
        ))
        .arg(client)
        .arg(work.join("kernel-region.o"))
        .arg("-o")
        .arg(&executable))
    .unwrap();
    executable
}

#[test]
fn inode_mode_source_binds_an_independent_instance_and_complete_costs() {
    // TOPAL-SYSTEMS-INODE-MODE-ACTION-001.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let input = inode_input(&work, ACTION);
    let object =
        build_systems_region_instance_library(&input, "apply", 2, "inode-metadata", &tools, &work)
            .unwrap();
    assert_eq!(object.record.instance.as_deref(), Some("inode-metadata"));
    assert_eq!(object.record.scalar_count, Some(2));
    assert_eq!(object.record.allocation.maximum_view_nodes, 1);
    assert_eq!(
        object.record.schema,
        "topal.kernel-region-frame-source-object.x86_64/4"
    );
    assert_eq!(
        object.record.allocation.numeric.source_evaluation_bytes,
        12064
    );
    assert_eq!(object.record.allocation.numeric.bridge_bytes, 112);
    assert_eq!(object.record.allocation.numeric.header_bytes, 32);
    assert_eq!(object.record.allocation.numeric.maximum_bytes, 12208);
    assert_eq!(object.record.allocation.numeric.maximum_operand_bits, 65);
    assert_eq!(object.record.maximum_stack_bytes, 1296);
    assert_eq!(object.record.functions.len(), 47);
    assert_eq!(
        object.record.allocation.unsigned_runtime_sha256.as_deref(),
        Some(UNSIGNED_RUNTIME_SHA256)
    );
    assert!(object.record.allocation.numeric.maximum_bytes <= 65536);
    assert!(object.record.maximum_stack_bytes <= 4096);
    verify_systems_region_instance_library(
        &input,
        "apply",
        2,
        "inode-metadata",
        &tools,
        &temporary_work(),
        &object,
    )
    .unwrap();
    let executable = inode_client(&work, &object.record);
    let output = run(&mut Command::new(&executable)).unwrap();
    assert!(output.contains("INODE_NATIVE_PASS calls=36906"), "{output}");
    eprintln!("{output}");
    for case in 0..14 {
        let mut changed = object.clone();
        match case {
            0 => changed.record.instance = Some("file-position".into()),
            1 => changed.record.scalar_count = Some(1),
            2 => changed.record.allocation.unsigned_runtime_sha256 = None,
            3 => changed.record.allocation.unsigned_runtime_sha256 = Some("0".repeat(64)),
            4 => changed.record.allocation.numeric.maximum_bytes -= 8,
            5 => changed.record.maximum_stack_bytes -= 8,
            6 => changed.object[0] ^= 1,
            7 => changed.module_snapshot.push(0),
            8 => changed.llvm.push_str("; changed source\n"),
            9 => changed.retained_llvm.push_str("; changed retained input\n"),
            10 => changed.record.imported_hooks[0].push_str("_wrong_owner"),
            11 => changed.record.effects[0].write_owner = false,
            12 => changed.record.allocation.region_runtime_sha256 = "0".repeat(64),
            13 => {
                changed.record.schema = "topal.kernel-region-frame-source-object.x86_64/2".into();
                changed.record.allocation.unsigned_runtime_sha256 = None;
            }
            _ => unreachable!(),
        }
        assert!(
            verify_systems_region_instance_library(
                &input,
                "apply",
                2,
                "inode-metadata",
                &tools,
                &temporary_work(),
                &changed
            )
            .is_err(),
            "case {case}"
        );
    }
    eprintln!(
        "INODE_MODE_COMPONENT allocation={:?} stack={} functions={} work={}",
        object.record.allocation,
        object.record.maximum_stack_bytes,
        object.record.functions.len(),
        work.display()
    );
}

#[test]
fn inode_source_large_nat_facts_refuse_before_observing_inaccessible_owner_payload() {
    // TOPAL-SYSTEMS-INODE-MODE-ACTION-001.
    let tools = LlvmTools::discover(None).unwrap();
    for large in [
        "18446744073709551616",
        "57896044618658097711785492504343953926634992332820282019728792003956564819968",
    ] {
        for time in [false, true] {
            let work = temporary_work();
            let facts = if time {
                format!("region, permissions, seconds + {large}")
            } else {
                format!("region, permissions + {large}, seconds")
            };
            let source = ACTION.replace("region, permissions, seconds", &facts);
            let input = inode_input(&work, &source);
            let object = build_systems_region_instance_library(
                &input,
                "apply",
                2,
                "inode-metadata",
                &tools,
                &work,
            )
            .unwrap();
            let executable = inode_client(&work, &object.record);
            let output = run(Command::new(executable).arg("wide")).unwrap();
            assert!(output.contains("INODE_NATIVE_PASS calls=1"), "{output}");
        }
    }
}

#[test]
fn inode_mode_changed_source_fails_the_byte_oracle_and_false_keeps_earlier_stores() {
    // TOPAL-SYSTEMS-INODE-MODE-ACTION-001.
    let tools = LlvmTools::discover(None).unwrap();
    for false_after_store in [false, true] {
        let work = temporary_work();
        let mut input = inode_input(&work, ACTION);
        input.modules[0].text = if false_after_store {
            METADATA.replace("true then write-time (region, seconds)", "true then false")
        } else {
            METADATA.replace("value is mode)", "value is 0)")
        };
        let object = build_systems_region_instance_library(
            &input,
            "apply",
            2,
            "inode-metadata",
            &tools,
            &work,
        )
        .unwrap();
        let executable = inode_client(&work, &object.record);
        if false_after_store {
            let output = run(Command::new(executable).arg("false-after-mode")).unwrap();
            assert!(output.contains("INODE_NATIVE_PASS calls=1"), "{output}");
        } else {
            let error = run(&mut Command::new(executable)).unwrap_err();
            assert!(
                error.to_string().contains("incorrect source inode bytes"),
                "{error}"
            );
        }
    }
}

#[test]
fn inode_mode_debugger_observes_actual_source_facts_effects_and_caller_reclamation() {
    // TOPAL-SYSTEMS-INODE-MODE-ACTION-001.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let input = inode_input(&work, ACTION);
    let object =
        build_systems_region_instance_library(&input, "apply", 2, "inode-metadata", &tools, &work)
            .unwrap();
    let executable = inode_client(&work, &object.record);
    let bytes = std::fs::read(&executable).unwrap();
    let data = inspect_systems_region_read_only_data(&bytes, &object.record).unwrap();
    let config = work.join("inode-debug.json");
    std::fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!({"record": object.record, "data": data})).unwrap(),
    )
    .unwrap();
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/inode_mode_debug.py");
    let output = run(Command::new("gdb")
        .args(["--batch", "-nx"])
        .arg("-ex")
        .arg(format!("set $inode_config = \"{}\"", config.display()))
        .arg("-x")
        .arg(script)
        .arg(executable))
    .unwrap();
    assert!(
        output.contains("INODE_DEBUG_PASS calls=9 erased=9 payload-loads=48 payload-stores=64"),
        "{output}"
    );
    eprintln!("{output}");
}
