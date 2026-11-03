use super::*;

const SOURCE: &str = include_str!("../../../linux-kernel/labs/qemu/x86_64/region_source_probe.t");
const BYTES: &str = include_str!("../../../linux-kernel/kernel/support/bytes.t");

#[test]
fn instance_objects_bind_distinct_namespaces_hooks_counts_and_canonical_identity() {
    // TOPAL-SYSTEMS-REGION-INSTANCE-SOURCE-001, -BINDING-001.
    let tools = LlvmTools::discover(None).unwrap();
    let source = "use language (version is v0.1, features is (systems))\npub apply is fn ((region : KernelByteRegion, first : Nat, second : Nat, third : Nat)) -> Boolean\n  true\n";
    let mut roots = Vec::new();
    let mut records = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    for label in ["left-owner", "right-owner"] {
        let work = temporary_work();
        let input = input(&work, source);
        let object =
            build_systems_region_instance_library(&input, "apply", 3, label, &tools, &work)
                .unwrap();
        assert_eq!(
            object.record.schema,
            "topal.kernel-region-frame-source-object.x86_64/2"
        );
        assert_eq!(object.record.instance.as_deref(), Some(label));
        assert_eq!(object.record.scalar_count, Some(3));
        let symbols = object.record.native_symbols().unwrap();
        for symbol in object
            .record
            .functions
            .iter()
            .map(|f| f.symbol.as_str())
            .chain(
                object
                    .record
                    .read_only_data
                    .iter()
                    .map(|d| d.symbol.as_str()),
            )
            .chain(object.record.imported_hooks.iter().map(String::as_str))
        {
            assert!(
                names.insert(symbol.to_owned()),
                "duplicate instance-owned name {symbol}"
            );
            assert!(
                symbol.starts_with(&symbols.prefix)
                    || symbol == symbols.export
                    || symbol == symbols.allocate
                    || symbol == symbols.abort
            );
        }
        verify_systems_region_instance_library(
            &input,
            "apply",
            3,
            label,
            &tools,
            &temporary_work(),
            &object,
        )
        .unwrap();
        assert!(
            verify_systems_region_framed_action_library(
                &input,
                "apply",
                3,
                &tools,
                &temporary_work(),
                &object
            )
            .is_err()
        );
        let mut changed = object.clone();
        changed.record.instance = None;
        assert!(
            verify_systems_region_instance_library(
                &input,
                "apply",
                3,
                label,
                &tools,
                &temporary_work(),
                &changed
            )
            .is_err()
        );
        let mut json = serde_json::to_value(&object.record).unwrap();
        assert_eq!(
            serde_json::from_value::<SystemsRegionLibraryArtifact>(json.clone()).unwrap(),
            object.record
        );
        json["instance"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<SystemsRegionLibraryArtifact>(json).is_err());
        records.push(object.record.clone());
        roots.push(work);
    }
    // Namespace registration is one facet, not an ownership certificate. Use
    // deliberately minimal ELF objects to isolate unselected-symbol refusal.
    for (name, accepted) in [
        ("topal.region-instance.left-owner.extra", true),
        ("topal.region-instance.right-owner.extra", true),
        ("topal.region-instance.foreign-owner.extra", false),
        ("topal_kernel_region_frame_left_owner_apply_v1", true),
        ("topal_kernel_region_frame_right_owner_allocate_v1", true),
        ("topal_kernel_region_frame_foreign_owner_abort_v1", false),
        ("topal_kernel_region_frame_apply_v1", false),
    ] {
        let mut file = object::write::Object::new(
            object::BinaryFormat::Elf,
            object::Architecture::X86_64,
            object::Endianness::Little,
        );
        let section = file.add_section(Vec::new(), b".text".to_vec(), object::SectionKind::Text);
        let value = file.append_section_data(section, &[0xc3], 1);
        file.add_symbol(object::write::Symbol {
            name: name.as_bytes().to_vec(),
            value,
            size: 1,
            kind: object::SymbolKind::Text,
            scope: object::SymbolScope::Linkage,
            weak: false,
            section: object::write::SymbolSection::Section(section),
            flags: object::SymbolFlags::None,
        });
        let bytes = file.write().unwrap();
        assert_eq!(
            validate_instance_namespaces(&bytes, &[Some(&records[0]), Some(&records[1])]).is_ok(),
            accepted,
            "{name}"
        );
        assert!(
            validate_instance_namespaces(&bytes, &[None, None]).is_err(),
            "{name}"
        );
    }
    let combined = temporary_work().join("both.bc");
    let linked = Command::new(tools.directory.join("llvm-link"))
        .arg(roots[0].join("kernel-region.bc"))
        .arg(roots[1].join("kernel-region.bc"))
        .arg("-o")
        .arg(&combined)
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let verified = Command::new(tools.directory.join("opt"))
        .args(["-passes=verify", "-disable-output"])
        .arg(&combined)
        .output()
        .unwrap();
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    for label in [
        "",
        "Root",
        "0root",
        "root.region",
        "root_region",
        "root--region",
        "root-",
        "root/region",
        "région",
    ] {
        let work = temporary_work();
        assert!(
            build_systems_region_instance_library(
                &input(&work, source),
                "apply",
                3,
                label,
                &tools,
                &work
            )
            .is_err(),
            "{label}"
        );
        assert!(!work.join("kernel-region.o").exists());
    }
    let profile = CompilerKernelRegionProfile::FramedAction { scalar_count: 0 };
    let empty_work = temporary_work();
    let empty_source = "use language (version is v0.1, features is (systems))\npub apply is fn (region : KernelByteRegion) -> Boolean\n  true\n";
    let empty_input = input(&empty_work, empty_source);
    let empty = build_systems_region_instance_library(
        &empty_input,
        "apply",
        0,
        "empty-owner",
        &tools,
        &empty_work,
    )
    .unwrap();
    assert!(empty.record.read_only_data.is_empty());
    assert_eq!(empty.record.scalar_count, Some(0));
    verify_systems_region_instance_library(
        &empty_input,
        "apply",
        0,
        "empty-owner",
        &tools,
        &temporary_work(),
        &empty,
    )
    .unwrap();
    let root = RegionNativeSymbols::for_profile(profile, Some(ROOT_INSTANCE)).unwrap();
    assert_eq!(root.prefix, ROOT_FRAME_PREFIX);
    assert_eq!(
        (
            root.export.as_str(),
            root.allocate.as_str(),
            root.abort.as_str()
        ),
        (ROOT_FRAME_EXPORT, ROOT_FRAME_ALLOCATE, ROOT_FRAME_ABORT)
    );
    let file = RegionNativeSymbols::for_profile(profile, Some(FILE_INSTANCE)).unwrap();
    assert_eq!(file.prefix, FILE_FRAME_PREFIX);
    assert_eq!(
        (
            file.export.as_str(),
            file.allocate.as_str(),
            file.abort.as_str()
        ),
        (FILE_FRAME_EXPORT, FILE_FRAME_ALLOCATE, FILE_FRAME_ABORT)
    );
    assert!(RegionNativeSymbols::for_profile(profile, Some(&"r".repeat(48))).is_ok());
    assert!(RegionNativeSymbols::for_profile(profile, Some(&"r".repeat(49))).is_err());
}

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

#[test]
fn framed_region_source_objects_bind_exact_scalar_counts_and_complete_costs() {
    // TOPAL-SYSTEMS-REGION-FRAME-SOURCE-001, -FRAME-ABI-001.
    let tools = LlvmTools::discover(None).unwrap();
    for scalar_count in [0, 1, 2, 3, 16] {
        let parameters = (0..scalar_count)
            .map(|i| format!(", fact{i} : Nat"))
            .collect::<String>();
        let fields = format!("region : KernelByteRegion{parameters}");
        let operands = if scalar_count == 0 {
            fields
        } else {
            format!("({fields})")
        };
        let source = format!(
            "use language (version is v0.1, features is (systems))\npub apply is fn ({operands}) -> Boolean\n  true\n"
        );
        let work = temporary_work();
        let input = input(&work, &source);
        let object = build_systems_region_framed_action_library(
            &input,
            "apply",
            scalar_count,
            &tools,
            &work,
        )
        .unwrap();
        assert_eq!(object.record.scalar_count, Some(scalar_count));
        let encoded = serde_json::to_value(&object.record).unwrap();
        let decoded: SystemsRegionLibraryArtifact =
            serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(decoded, object.record);
        for field in ["scalar_count", "adapter_family_sha256"] {
            let mut changed = encoded.clone();
            if field == "scalar_count" {
                changed[field] = serde_json::Value::Null;
            } else {
                changed["allocation"][field] = serde_json::Value::Null;
            }
            assert!(serde_json::from_value::<SystemsRegionLibraryArtifact>(changed).is_err());
        }
        assert_eq!(
            object.record.schema,
            "topal.kernel-region-frame-source-object.x86_64/1"
        );
        assert_eq!(object.record.native_export, FRAME_EXPORT);
        assert_eq!(
            object.record.native_abi,
            "topal.kernel-region-frame-control-view64-u64frame-bool.x86_64-sysv/1"
        );
        assert_eq!(object.record.imported_hooks, [FRAME_ALLOCATE, FRAME_ABORT]);
        assert_eq!(
            object.record.allocation.numeric.bridge_bytes,
            64 + 24 * u64::from(scalar_count)
        );
        assert_eq!(
            object.record.allocation.numeric.maximum_bytes,
            96 + 24 * u64::from(scalar_count)
        );
        assert_eq!(object.record.read_only_data.is_empty(), scalar_count == 0);
        assert!(object.record.maximum_stack_bytes <= 4096);
        eprintln!(
            "frame count={scalar_count} selected-adapter={} family={:?}",
            object.record.allocation.adapter_sha256, object.record.allocation.adapter_family_sha256
        );
        assert_eq!(
            object.record.allocation.adapter_family_sha256,
            Some(crate::codegen::region_frame_adapter_family_sha256())
        );
        verify_systems_region_framed_action_library(
            &input,
            "apply",
            scalar_count,
            &tools,
            &temporary_work(),
            &object,
        )
        .unwrap();
        assert!(
            verify_systems_region_framed_action_library(
                &input,
                "apply",
                scalar_count + 1,
                &tools,
                &temporary_work(),
                &object
            )
            .is_err()
        );
        let mut changed = object.clone();
        changed.record.scalar_count = None;
        assert!(
            verify_systems_region_framed_action_library(
                &input,
                "apply",
                scalar_count,
                &tools,
                &temporary_work(),
                &changed
            )
            .is_err()
        );
    }
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
        (
            include_str!("../../../linux-kernel/kernel/policy/bootstrap-byte-copy.t"),
            "root-copy",
        ),
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
    assert!(json.get("scalar_count").is_none());
    assert!(json["allocation"].get("adapter_family_sha256").is_none());
    for field in ["scalar_count", "adapter_family_sha256"] {
        let mut changed = json.clone();
        if field == "scalar_count" {
            changed[field] = serde_json::Value::Null;
        } else {
            changed["allocation"][field] = serde_json::Value::Null;
        }
        assert!(serde_json::from_value::<SystemsRegionLibraryArtifact>(changed).is_err());
    }
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
