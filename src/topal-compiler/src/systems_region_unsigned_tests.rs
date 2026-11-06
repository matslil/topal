use super::*;

const UNSIGNED_SOURCE: &str =
    include_str!("../../../linux-kernel/labs/qemu/x86_64/region_unsigned_probe.t");

fn unsigned_source(width: u8, big: bool) -> String {
    UNSIGNED_SOURCE
        .replace("byte-count is 8", &format!("byte-count is {width}"))
        .replace(
            "little-endian",
            if big { "big-endian" } else { "little-endian" },
        )
}

fn unsigned_client(work: &Path, limit: u64, width: u8, big: bool) -> std::path::PathBuf {
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/region_unsigned_client.c");
    let executable = work.join("region-unsigned-client");
    run(Command::new("cc")
        .args([
            "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2", "-g", "-no-pie",
        ])
        .arg(format!("-DFIELD_SOURCE_LIMIT={limit}"))
        .arg(format!("-DFIELD_WIDTH={width}"))
        .arg(format!("-DFIELD_BIG_ENDIAN={}", u8::from(big)))
        .arg(client)
        .arg(work.join("kernel-region.o"))
        .arg("-o")
        .arg(&executable))
    .unwrap();
    executable
}

#[test]
fn unsigned_fields_execute_all_formats_and_whole_preflight_on_guarded_payloads() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-001, -QUALIFY-001.
    let tools = LlvmTools::discover(None).unwrap();
    for width in 1..=8 {
        for big in [false, true] {
            let work = temporary_work();
            let source = unsigned_source(width, big);
            let input = input(&work, &source);
            let unit =
                build_systems_region_framed_action_library(&input, "apply", 3, &tools, &work)
                    .unwrap();
            assert_eq!(
                unit.record.schema,
                "topal.kernel-region-frame-source-object.x86_64/3"
            );
            assert_eq!(
                unit.record.allocation.unsigned_runtime_sha256.as_deref(),
                Some(UNSIGNED_RUNTIME_SHA256)
            );
            assert_eq!(unit.record.allocation.maximum_view_nodes, 2);
            assert!(unit.record.maximum_stack_bytes <= 4096);
            assert!(unit.record.allocation.numeric.maximum_bytes < 65536);
            let executable = unsigned_client(
                &work,
                unit.record.allocation.numeric.maximum_bytes,
                width,
                big,
            );
            let output = run(&mut Command::new(executable)).unwrap();
            assert!(output.contains("UNSIGNED_NATIVE_PASS"), "{output}");
            eprintln!(
                "{output}allocation={} stack={} work={}",
                unit.record.allocation.numeric.maximum_bytes,
                unit.record.maximum_stack_bytes,
                work.display()
            );
        }
    }
}

#[test]
fn unsigned_load_store_costs_and_optional_runtime_presence_are_exactly_bound() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-QUALIFY-001.
    let tools = LlvmTools::discover(None).unwrap();
    for (operation, cost) in [
        (
            "region unsigned store (offset-bytes is offset, byte-count is 8, byte-order is little-endian, value is value)",
            0,
        ),
        (
            "region unsigned load (offset-bytes is offset, byte-count is 8, byte-order is little-endian)\n    Some actual then actual = value\n    None then false",
            40,
        ),
    ] {
        let source = format!(
            "use language (version is v0.1, features is (systems))\npub apply is fn ((region : KernelByteRegion, offset : Nat, value : Nat)) -> Boolean\n  {operation}\n"
        );
        let work = temporary_work();
        let input = input(&work, &source);
        let object = build_systems_region_action_library(&input, "apply", &tools, &work).unwrap();
        assert_eq!(
            object.record.schema,
            "topal.kernel-region-action-source-object.x86_64/2"
        );
        assert_eq!(
            object.record.allocation.numeric.source_evaluation_bytes,
            cost
        );
        assert_eq!(object.record.allocation.numeric.maximum_bytes, 144 + cost);
        assert_eq!(object.record.allocation.maximum_view_nodes, 1);
        assert_eq!(object.record.allocation.numeric.maximum_operand_bits, 64);
        verify_systems_region_action_library(&input, "apply", &tools, &temporary_work(), &object)
            .unwrap();
        for mode in 0..10 {
            let mut changed = object.clone();
            match mode {
                0 => changed.record.allocation.unsigned_runtime_sha256 = None,
                1 => changed.record.allocation.unsigned_runtime_sha256 = Some("0".repeat(64)),
                2 => {
                    changed.record.schema =
                        "topal.kernel-region-action-source-object.x86_64/1".into()
                }
                3 => changed.record.allocation.numeric.maximum_bytes -= 8,
                4 => changed.object[0] ^= 1,
                5 => changed.llvm.push_str("; changed runtime\n"),
                6 => changed.retained_llvm.push_str("; changed\n"),
                7 => changed.module_snapshot.push(0),
                8 => changed.record.maximum_stack_bytes -= 8,
                9 => {
                    changed.record.allocation.numeric.source_evaluation_bytes =
                        cost.saturating_sub(8)
                }
                _ => unreachable!(),
            }
            if mode == 9 && cost == 0 {
                continue;
            }
            assert!(
                verify_systems_region_action_library(
                    &input,
                    "apply",
                    &tools,
                    &temporary_work(),
                    &changed
                )
                .is_err(),
                "case {mode}"
            );
        }
        let mut json = serde_json::to_value(&object.record).unwrap();
        json["allocation"]["unsigned_runtime_sha256"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<SystemsRegionLibraryArtifact>(json).is_err());
    }
    let work = temporary_work();
    let byte = input(&work, SOURCE);
    let object = build_systems_region_library(&byte, "apply", &tools, &work).unwrap();
    assert!(object.record.allocation.unsigned_runtime_sha256.is_none());
    let json = serde_json::to_value(&object.record).unwrap();
    assert!(json["allocation"].get("unsigned_runtime_sha256").is_none());
    assert!(!object.llvm.contains("@topal.kernel.region.unsigned."));
    let mut orphan = object.clone();
    orphan.record.allocation.unsigned_runtime_sha256 = Some(UNSIGNED_RUNTIME_SHA256.into());
    assert!(orphan.record.native_symbols().is_err());
    orphan.record.schema = "topal.kernel-region-source-object.x86_64/4".into();
    assert!(orphan.record.native_symbols().is_err());
    assert!(
        verify_systems_region_library(&byte, "apply", &tools, &temporary_work(), &orphan).is_err()
    );
}

#[test]
fn unsigned_native_debugger_observes_values_payload_access_and_complete_erasure() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-QUALIFY-001.
    let tools = LlvmTools::discover(None).unwrap();
    for big in [false, true] {
        let work = temporary_work();
        let input = input(&work, &unsigned_source(8, big));
        let unit =
            build_systems_region_framed_action_library(&input, "apply", 3, &tools, &work).unwrap();
        let executable =
            unsigned_client(&work, unit.record.allocation.numeric.maximum_bytes, 8, big);
        let bytes = std::fs::read(&executable).unwrap();
        let data = inspect_systems_region_read_only_data(&bytes, &unit.record).unwrap();
        let config = work.join("unsigned-debug.json");
        std::fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({"record": unit.record, "data": data})).unwrap(),
        )
        .unwrap();
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../linux-kernel/labs/qemu/x86_64/region_unsigned_debug.py");
        let output = run(Command::new("gdb")
            .args(["--batch", "-nx"])
            .arg("-ex")
            .arg(format!("set $unsigned_config = \"{}\"", config.display()))
            .arg("-x")
            .arg(script)
            .arg(executable))
        .unwrap();
        assert!(
            output.contains(
                "UNSIGNED_DEBUG_PASS calls=30 erased=30 payload-loads=8 payload-stores=8"
            ),
            "{output}"
        );
        eprintln!("big={big}: {output}");
    }
}

#[test]
fn unsigned_unit_instance_and_unselected_runtime_inputs_keep_exact_profile_binding() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-QUALIFY-001.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let source = "use language (version is v0.1, features is (systems))\npub apply is fn (region : KernelByteRegion) -> Unit\n  observed is region unsigned load (offset-bytes is 0, byte-count is 1, byte-order is big-endian)\n  ()\n";
    let input = input(&work, source);
    let unit = build_systems_region_library(&input, "apply", &tools, &work).unwrap();
    assert_eq!(
        unit.record.schema,
        "topal.kernel-region-source-object.x86_64/4"
    );
    assert_eq!(unit.record.allocation.numeric.source_evaluation_bytes, 40);
    verify_systems_region_library(&input, "apply", &tools, &temporary_work(), &unit).unwrap();
    let mut changed = unit.record.clone();
    changed.allocation.unsigned_runtime_sha256 = None;
    assert!(changed.native_symbols().is_err());
    changed = unit.record.clone();
    changed.schema = "topal.kernel-region-source-object.x86_64/3".into();
    assert!(changed.native_symbols().is_err());
    changed.allocation.unsigned_runtime_sha256 = None;
    assert!(changed.native_symbols().is_err());
    let source = "use language (version is v0.1, features is (systems))\npub apply is fn ((region : KernelByteRegion, offset : Nat, value : Nat)) -> Boolean\n  false\n    true then region unsigned store (offset-bytes is offset, byte-count is 7, byte-order is big-endian, value is value)\n    false then true\n";
    let work = temporary_work();
    let input = super::input(&work, source);
    let instance =
        build_systems_region_instance_library(&input, "apply", 2, "unsigned-owner", &tools, &work)
            .unwrap();
    assert_eq!(
        instance.record.schema,
        "topal.kernel-region-frame-source-object.x86_64/4"
    );
    assert_eq!(instance.record.instance.as_deref(), Some("unsigned-owner"));
    assert!(instance.record.allocation.unsigned_runtime_sha256.is_some());
    assert!(instance.llvm.contains(".primitive.unsigned.store"));
    verify_systems_region_instance_library(
        &input,
        "apply",
        2,
        "unsigned-owner",
        &tools,
        &temporary_work(),
        &instance,
    )
    .unwrap();
    let mut changed = instance.record.clone();
    changed.allocation.unsigned_runtime_sha256 = None;
    assert!(changed.native_symbols().is_err());
    changed = instance.record.clone();
    changed.schema = "topal.kernel-region-frame-source-object.x86_64/2".into();
    assert!(changed.native_symbols().is_err());
}

#[test]
fn unsigned_runtime_fact_evaluation_precedes_native_role_assignment() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-001, -QUALIFY-001.
    let tools = LlvmTools::discover(None).unwrap();
    let source = include_str!("../../../linux-kernel/labs/qemu/x86_64/region_source_order.t")
        .replace("stored is region byte store (value is read-before-write region, offset-bytes is change-source region)",
            "stored is region unsigned store (value is read-before-write region, byte-order is little-endian, offset-bytes is change-source region, byte-count is 1)");
    let work = temporary_work();
    let input = input(&work, &source);
    let unit = build_systems_region_library(&input, "apply", &tools, &work).unwrap();
    let executable = native_client(&work, unit.record.allocation.numeric.maximum_bytes);
    let output = run(Command::new(executable).arg("argument-order")).unwrap();
    assert!(
        output.contains("REGION_SOURCE_OK calls=18 rejected=12"),
        "{output}"
    );
}
