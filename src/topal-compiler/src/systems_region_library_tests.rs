use super::*;

const SOURCE: &str = include_str!("../../../linux-kernel/labs/qemu/x86_64/region_source_probe.t");
const BYTES: &str = include_str!("../../../linux-kernel/kernel/support/bytes.t");

fn temporary_work() -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "topal-region-source-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&path).unwrap();
    path
}
fn input(work: &Path, source: &str) -> SystemsSourceLibraryInput {
    let source_path = work.join("region-source.t");
    std::fs::write(&source_path, source).unwrap();
    let modules = if source.contains("use library support") {
        std::fs::create_dir(work.join("support")).unwrap();
        std::fs::write(work.join("support/bytes.t"), BYTES).unwrap();
        vec![crate::SystemsSourceModuleInput {
            source: "support/bytes.t".into(),
            identity: vec!["support".into(), "bytes".into()],
            text: BYTES.into(),
        }]
    } else {
        Vec::new()
    };
    SystemsSourceLibraryInput {
        source: "policy/region-source.t".into(),
        source_name: source_path.to_string_lossy().into(),
        text: source.into(),
        modules,
    }
}

fn native_client(work: &Path, limit: u64) -> std::path::PathBuf {
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/region_source_client.c");
    let executable = work.join("region-source-client");
    run(Command::new("cc")
        .args([
            "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2", "-g", "-no-pie",
        ])
        .arg(format!("-DREGION_SOURCE_LIMIT_BYTES={limit}"))
        .arg(&client)
        .arg(work.join("kernel-region.o"))
        .arg("-o")
        .arg(&executable))
    .unwrap();
    executable
}

#[test]
fn emits_checked_region_source_and_actual_native_helper_frames() {
    // TOPAL-SYSTEMS-REGION-SOURCE-ENTRY-001, -OBJECT-001.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let input = input(&work, SOURCE);
    let unit = build_systems_region_library(&input, "apply", &tools, &work).unwrap();
    eprintln!("region source work: {}", work.display());
    eprintln!(
        "region allocation: {:?}; stack: {}",
        unit.record.allocation, unit.record.maximum_stack_bytes
    );
    assert_eq!(unit.record.declared_entry, "(KernelByteRegion) -> Unit");
    assert_eq!(unit.record.allocation.maximum_view_nodes, 2);
    assert!(unit.record.maximum_stack_bytes <= 4096);
    assert!(
        unit.record
            .effects
            .iter()
            .all(|effect| effect.read_owner && effect.write_owner)
    );
    assert!(unit.llvm.contains("call fastcc void @topal.region."));
    assert!(unit.llvm.contains("name: \"KernelByteRegion\""));
    assert!(!unit.llvm.contains("@_start"));
    assert!(
        unit.record
            .functions
            .iter()
            .any(|function| function.symbol.starts_with(PREFIX))
    );
    let executable = native_client(&work, unit.record.allocation.numeric.maximum_bytes);
    let output = run(&mut Command::new(&executable)).unwrap();
    assert!(
        output.contains("REGION_SOURCE_OK calls=18 rejected=12"),
        "{output}"
    );
    eprintln!("{output}");
    let helper = unit
        .record
        .functions
        .iter()
        .find(|function| function.symbol.starts_with(PREFIX) && function.symbol.contains("copy"))
        .unwrap();
    let debug = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/region_source_debug.py");
    let output = run(Command::new("gdb")
        .args(["--batch", "-nx"])
        .arg("-ex")
        .arg(format!("set $region_helper = \"{}\"", helper.symbol))
        .arg("-x")
        .arg(debug)
        .arg(&executable))
    .unwrap();
    assert!(
        output.contains("REGION_SOURCE_DEBUG_OK returns=18 helpers=3"),
        "{output}"
    );
    eprintln!("{output}");
}

#[test]
fn region_source_checks_dynamic_refusals_rights_and_argument_effect_order() {
    // TOPAL-SYSTEMS-REGION-SOURCE-MEMBERS-001, -CLOSURE-001, -OBJECT-001.
    let tools = LlvmTools::discover(None).unwrap();
    let readonly = SOURCE.replace("rights is read-write", "rights is read-only");
    for (source, mode) in [
        (readonly.as_str(), "readonly-view"),
        (
            include_str!("../../../linux-kernel/labs/qemu/x86_64/region_source_boundaries.t"),
            "boundaries",
        ),
        (
            include_str!("../../../linux-kernel/labs/qemu/x86_64/region_source_order.t"),
            "argument-order",
        ),
    ] {
        let work = temporary_work();
        let input = input(&work, source);
        let unit = build_systems_region_library(&input, "apply", &tools, &work).unwrap();
        let executable = native_client(&work, unit.record.allocation.numeric.maximum_bytes);
        let output = run(Command::new(&executable).arg(mode)).unwrap();
        assert!(
            output.contains("REGION_SOURCE_OK calls=18 rejected=12"),
            "{mode}: {output}"
        );
        eprintln!(
            "{mode}: {}: {:?}; stack {}; {output}",
            work.display(),
            unit.record.allocation,
            unit.record.maximum_stack_bytes
        );
    }
}

#[test]
fn isolated_region_modules_reject_caller_scope_and_changed_helpers_fail_the_byte_oracle() {
    // TOPAL-SYSTEMS-REGION-SOURCE-ENTRY-001, -OBJECT-001.
    let tools = LlvmTools::discover(None).unwrap();
    for mode in 0..3 {
        let work = temporary_work();
        let mut input = input(&work, SOURCE);
        match mode {
            0 => input.modules[0].text = BYTES.replace(", features is (systems)", ""),
            1 => input.modules[0].text = BYTES.replace("pub copy-byte", "copy-byte"),
            2 => {
                input.text.push_str(
                    "\nprivate-root is fn (region : KernelByteRegion) -> Boolean\n  true\n",
                );
                input.modules[0].text = "use language (version is v0.1, features is (systems))\npub copy-byte is fn ((region : KernelByteRegion, source : Nat, destination : Nat)) -> Boolean\n  private-root region\n".into();
            }
            _ => unreachable!(),
        }
        let error = build_systems_region_library(&input, "apply", &tools, &work)
            .err()
            .expect("invalid module accepted");
        if mode == 2 {
            assert!(
                error.to_string().contains("private caller scope"),
                "{error}"
            );
        }
    }
    let work = temporary_work();
    let mut input = input(&work, SOURCE);
    input.modules[0].text = BYTES.replace("value is value", "value is 255");
    let unit = build_systems_region_library(&input, "apply", &tools, &work).unwrap();
    let executable = native_client(&work, unit.record.allocation.numeric.maximum_bytes);
    let error = run(&mut Command::new(&executable)).unwrap_err();
    assert!(
        error.to_string().contains("incorrect source byte"),
        "{error}"
    );
}

#[test]
fn region_certificate_reproduction_rejects_undercharge_wrong_protocol_and_changed_objects() {
    // TOPAL-SYSTEMS-REGION-SOURCE-OBJECT-001.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let input = input(&work, SOURCE);
    let unit = build_systems_region_library(&input, "apply", &tools, &work).unwrap();
    verify_systems_region_library(&input, "apply", &tools, &temporary_work(), &unit).unwrap();
    for mode in 0..9 {
        let mut changed = unit.clone();
        match mode {
            0 => changed.record.allocation.numeric.maximum_bytes -= 8,
            1 => changed.record.allocation.maximum_view_nodes -= 1,
            2 => changed.record.maximum_stack_bytes -= 8,
            3 => changed.record.native_abi.push_str("-unchecked"),
            4 => changed.record.effects[0].write_owner = false,
            5 => changed.object[0] ^= 1,
            6 => changed.retained_llvm.push_str("; changed\n"),
            7 => changed.module_snapshot.push(0),
            8 => changed.record.allocation.region_runtime_sha256 = "0".repeat(64),
            _ => unreachable!(),
        }
        assert!(
            verify_systems_region_library(&input, "apply", &tools, &temporary_work(), &changed)
                .is_err(),
            "case {mode}"
        );
    }
    let mut json = serde_json::to_value(&unit.record).unwrap();
    json["unchecked"] = true.into();
    assert!(serde_json::from_value::<SystemsRegionLibraryArtifact>(json).is_err());
}

#[test]
fn region_source_bounds_reject_temporary_view_and_numeric_overflow_before_object_emission() {
    // TOPAL-SYSTEMS-REGION-SOURCE-OBJECT-001.
    let tools = LlvmTools::discover(None).unwrap();
    for mode in 0..3 {
        let mut source = "use language (version is v0.1, features is (systems))\npub apply is fn (region : KernelByteRegion) -> Unit\n".to_owned();
        let count = match mode {
            0 => 1700,
            1 => 256,
            _ => 1,
        };
        for index in 0..count {
            let operation = match mode {
                0 => "region byte load (offset-bytes is 0)",
                1 => "region byte window (offset-bytes is 0, byte-count is 0, rights is read-only)",
                _ => {
                    "region byte load (offset-bytes is 115792089237316195423570985008687907853269984665640564039457584007913129639936)"
                }
            };
            source.push_str(&format!("  result{index} is {operation}\n"));
        }
        source.push_str("  ()\n");
        let work = temporary_work();
        let input = input(&work, &source);
        let error = build_systems_region_library(&input, "apply", &tools, &work)
            .err()
            .expect("unbounded object accepted");
        let expected = match mode {
            0 => "allocation exceeds",
            1 => "issued-view bound",
            _ => "operand bit bound",
        };
        assert!(error.to_string().contains(expected), "case {mode}: {error}");
        assert!(!work.join("kernel-region.o").exists());
    }
}
