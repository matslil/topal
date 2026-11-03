use std::path::Path;
use std::process::Command;
use topal_compiler::*;
const ACTION_EXPORT: &str = "topal_kernel_region_action_apply_v1";
const ACTION_PREFIX: &str = "topal.region.action.";
const ACTION_ALLOCATE: &str = "topal_kernel_region_action_allocate_v1";
const ACTION_ABORT: &str = "topal_kernel_region_action_abort_v1";
const EXPORT: &str = "topal_kernel_region_apply_v1";
const ALLOCATE: &str = "topal_kernel_region_allocate_v1";

fn run(command: &mut Command) -> Result<String, String> {
    let output = command.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }
    String::from_utf8(output.stdout).map_err(|e| e.to_string())
}

const SOURCE: &str = include_str!("../../../kernel/policy/file-position.t");
const WORD: &str = include_str!("../../../kernel/support/word-bytes.t");

fn work() -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "topal-region-action-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&path).unwrap();
    path
}
fn input(work: &Path) -> SystemsSourceLibraryInput {
    std::fs::write(work.join("file-position.t"), SOURCE).unwrap();
    std::fs::create_dir(work.join("support")).unwrap();
    std::fs::write(work.join("support/word-bytes.t"), WORD).unwrap();
    SystemsSourceLibraryInput {
        source: "policy/file-position.t".into(),
        source_name: work.join("file-position.t").to_string_lossy().into(),
        text: SOURCE.into(),
        modules: vec![SystemsSourceModuleInput {
            source: "support/word-bytes.t".into(),
            identity: vec!["support".into(), "word-bytes".into()],
            text: WORD.into(),
        }],
    }
}
fn client(work: &Path, limit: u64, unsigned: bool) -> std::path::PathBuf {
    let client =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../labs/qemu/x86_64/region_action_client.c");
    let executable = work.join("region-action-client");
    run(Command::new("cc")
        .args([
            "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2", "-g", "-no-pie",
        ])
        .arg(format!("-DREGION_SOURCE_LIMIT_BYTES={limit}"))
        .arg(if unsigned {
            "-DACTION_WORD_LIMIT=UINT64_MAX"
        } else {
            "-DACTION_WORD_LIMIT=INT64_MAX"
        })
        .arg(client)
        .arg(work.join("kernel-region.o"))
        .arg("-o")
        .arg(&executable))
    .unwrap();
    executable
}

#[test]
fn region_action_executes_real_nat_addition_byte_codec_and_boolean_refusals() {
    // TOPAL-SYSTEMS-REGION-ACTION-SOURCE-001; standalone evidence, not a VFS loan.
    let tools = LlvmTools::discover(None).unwrap();
    let work = work();
    let input = input(&work);
    let unit = build_systems_region_action_library(&input, "advance", &tools, &work).unwrap();
    assert_eq!(
        unit.record.declared_entry,
        "(KernelByteRegion, Nat, Nat) -> Boolean"
    );
    assert_eq!(unit.record.native_export, ACTION_EXPORT);
    assert_eq!(unit.record.allocation.numeric.bridge_bytes, 112);
    assert_eq!(unit.record.imported_hooks, [ACTION_ALLOCATE, ACTION_ABORT]);
    assert!(!unit.record.read_only_data.is_empty());
    assert!(
        unit.record
            .read_only_data
            .iter()
            .all(|d| d.symbol.starts_with(ACTION_PREFIX) && d.alignment == 8 && d.bytes > 0)
    );
    assert!(
        unit.record
            .functions
            .iter()
            .all(|f| f.symbol.starts_with(ACTION_PREFIX) || f.symbol == ACTION_EXPORT)
    );
    assert!(unit.record.maximum_stack_bytes <= 4096);
    assert!(!unit.retained_llvm.contains("@topal.kernel.region."));
    assert!(!unit.retained_llvm.contains("@topal_kernel_region_apply_v1"));
    assert!(
        !unit
            .retained_llvm
            .contains("@topal_kernel_region_allocate_v1")
    );
    let output = run(&mut Command::new(client(
        &work,
        unit.record.allocation.numeric.maximum_bytes,
        false,
    )))
    .unwrap();
    assert!(
        output.contains("REGION_ACTION_OK calls=168 rejected=12"),
        "{output}"
    );
    eprintln!(
        "{}: allocation {:?}; stack {}; {output}",
        work.display(),
        unit.record.allocation,
        unit.record.maximum_stack_bytes
    );
    let entry = unit
        .record
        .functions
        .iter()
        .find(|f| f.symbol.starts_with(ACTION_PREFIX) && f.symbol.contains("advance"))
        .unwrap();
    let codec = unit
        .record
        .functions
        .iter()
        .find(|f| f.symbol.starts_with(ACTION_PREFIX) && f.symbol.contains("store_2du64_2dle"))
        .unwrap();
    let debug =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../labs/qemu/x86_64/region_action_debug.py");
    let output = run(Command::new("gdb")
        .args(["--batch", "-nx"])
        .arg("-ex")
        .arg(format!("set $region_action_entry = \"{}\"", entry.symbol))
        .arg("-ex")
        .arg(format!("set $region_action_codec = \"{}\"", codec.symbol))
        .arg("-x")
        .arg(debug)
        .arg(work.join("region-action-client")))
    .unwrap();
    assert!(
        output.contains("REGION_ACTION_DEBUG_OK returns=168 entries=168 codecs=98"),
        "{output}"
    );
}

#[test]
fn region_action_binding_reproduction_rejects_changed_code_costs_and_profile() {
    // TOPAL-SYSTEMS-REGION-ACTION-SOURCE-001.
    let tools = LlvmTools::discover(None).unwrap();
    let directory = work();
    let input = input(&directory);
    let unit = build_systems_region_action_library(&input, "advance", &tools, &directory).unwrap();
    verify_systems_region_action_library(&input, "advance", &tools, &work(), &unit).unwrap();
    assert!(verify_systems_region_library(&input, "advance", &tools, &work(), &unit).is_err());
    let encoded = serde_json::to_value(&unit.record).unwrap();
    assert!(encoded.get("scalar_count").is_none());
    assert!(encoded["allocation"].get("adapter_family_sha256").is_none());
    let decoded: SystemsRegionLibraryArtifact = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(decoded, unit.record);
    for field in ["scalar_count", "adapter_family_sha256"] {
        let mut changed = encoded.clone();
        if field == "scalar_count" {
            changed[field] = serde_json::Value::Null;
        } else {
            changed["allocation"][field] = serde_json::Value::Null;
        }
        assert!(serde_json::from_value::<SystemsRegionLibraryArtifact>(changed).is_err());
    }
    for mode in 0..8 {
        let mut changed = unit.clone();
        match mode {
            0 => changed.record.allocation.numeric.bridge_bytes -= 24,
            1 => changed.record.allocation.numeric.maximum_bytes -= 8,
            2 => changed.record.maximum_stack_bytes -= 8,
            3 => changed.record.native_export = EXPORT.into(),
            4 => changed.record.imported_hooks[0] = ALLOCATE.into(),
            5 => changed.object[0] ^= 1,
            6 => changed.record.allocation.adapter_sha256 = "0".repeat(64),
            7 => changed.record.read_only_data[0].sha256 = "0".repeat(64),
            _ => unreachable!(),
        }
        assert!(
            verify_systems_region_action_library(&input, "advance", &tools, &work(), &changed)
                .is_err(),
            "case {mode}"
        );
    }
}

#[test]
fn region_action_changed_ordinary_body_fails_independent_byte_oracle() {
    // TOPAL-SYSTEMS-REGION-ACTION-POSITION-001; typing does not prove the VFS contract.
    let tools = LlvmTools::discover(None).unwrap();
    let work = work();
    let mut input = input(&work);
    input.modules[0].text = WORD.replace(
        "offset-bytes is 7, value is byte",
        "offset-bytes is 7, value is 0",
    );
    let unit = build_systems_region_action_library(&input, "advance", &tools, &work).unwrap();
    let error = run(&mut Command::new(client(
        &work,
        unit.record.allocation.numeric.maximum_bytes,
        false,
    )))
    .unwrap_err();
    assert!(
        error.to_string().contains("incorrect action result/bytes"),
        "{error}"
    );
}

#[test]
fn region_action_transports_full_u64_nat_inputs_without_truncating_65_bit_sum() {
    // TOPAL-SYSTEMS-REGION-ACTION-SOURCE-001: full scalar domain and reusable codec.
    let tools = LlvmTools::discover(None).unwrap();
    let work = work();
    let mut input = input(&work);
    input.text = SOURCE.replace("9223372036854775807", "18446744073709551615");
    let unit = build_systems_region_action_library(&input, "advance", &tools, &work).unwrap();
    let output = run(&mut Command::new(client(
        &work,
        unit.record.allocation.numeric.maximum_bytes,
        true,
    )))
    .unwrap();
    assert!(
        output.contains("REGION_ACTION_OK calls=168 rejected=12"),
        "{output}"
    );
}

#[test]
fn region_action_private_symbols_coexist_with_the_frozen_unit_profile() {
    // TOPAL-SYSTEMS-REGION-ACTION-SOURCE-001: link closure, not field ownership.
    let tools = LlvmTools::discover(None).unwrap();
    let action_work = work();
    let action_input = input(&action_work);
    let action =
        build_systems_region_action_library(&action_input, "advance", &tools, &action_work)
            .unwrap();
    let unit_work = work();
    let unit_input = SystemsSourceLibraryInput {
        source: "policy/unit.t".into(), source_name: "unit.t".into(),
        text: "use language (version is v0.1, features is (systems))\npub apply is fn (region : KernelByteRegion) -> Unit\n  copied is region byte store (offset-bytes is 1, value is 90)\n  ()\n".into(), modules: Vec::new(),
    };
    let unit = build_systems_region_library(&unit_input, "apply", &tools, &unit_work).unwrap();
    let globals = |llvm: &str| {
        llvm.lines()
            .filter_map(|line| {
                line.strip_prefix('@')
                    .and_then(|line| line.split_once(" = "))
                    .map(|(name, _)| name.to_owned())
            })
            .filter(|name| name != "llvm.used")
            .collect::<std::collections::BTreeSet<_>>()
    };
    let action_globals = globals(&action.retained_llvm);
    assert!(
        action_globals
            .iter()
            .all(|name| name.starts_with(ACTION_PREFIX))
    );
    assert!(action_globals.is_disjoint(&globals(&unit.retained_llvm)));
    let mut symbols = unit
        .record
        .functions
        .iter()
        .map(|f| f.symbol.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        action
            .record
            .functions
            .iter()
            .all(|f| symbols.insert(&f.symbol))
    );
    let linked = action_work.join("combined.bc");
    run(Command::new(tools.directory.join("llvm-link"))
        .arg(action_work.join("kernel-region.bc"))
        .arg(unit_work.join("kernel-region.bc"))
        .arg("-o")
        .arg(&linked))
    .unwrap();
    run(Command::new(tools.directory.join("opt"))
        .args(["-passes=verify", "-disable-output"])
        .arg(linked))
    .unwrap();
}

#[test]
fn region_action_native_service_calls_are_explicit_and_missing_snapshots_refuse() {
    // TOPAL-SYSTEMS-REGION-ACTION-OWNER-001, -RETURN-001, -POSITION-001.
    // Structural provider evidence only; a declaration supplies no source snapshot.
    // Actual source/link/controller/boot binding is checked by a separate test.
    let source = include_str!("../../../kernel/arch/x86_64/toolchain-gate.t");
    let start = source.find("  file-position-library is").unwrap();
    let end = start + source[start..].find("  ),\n").unwrap() + 5;
    let legacy = source[start..end].replace(
        "borrowed-byte-region-instance-action-v1",
        "borrowed-byte-region-action-v1",
    );
    let source = format!("{}{}{}", &source[..start], legacy, &source[end..]);
    let program = topal_language::compiler::analyze_systems_for_compiler(
        &source,
        &topal_language::compiler::CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    assert_eq!(
        program
            .file_position_library
            .as_ref()
            .unwrap()
            .interface
            .as_str(),
        "borrowed-byte-region-action-v1"
    );
    let object = generate_x86_64_systems_provider_object(&program).unwrap();
    assert_eq!(
        object.plan.file_position_library_interface,
        Some("borrowed-byte-region-action-v1")
    );
    let tools = LlvmTools::discover(None).unwrap();
    let directory = work();
    let provider = directory.join("provider.o");
    std::fs::write(&provider, object.bytes).unwrap();
    let relocations = run(Command::new(tools.directory.join("llvm-objdump"))
        .arg("-r")
        .arg(&provider))
    .unwrap();
    let native_calls = |symbol: &str| {
        relocations
            .lines()
            .filter(|line| line.contains("R_X86_64_PLT32") && line.contains(symbol))
            .count()
    };
    assert_eq!(native_calls("topal_x86_systems_file_position_apply_v1"), 3);
    assert_eq!(
        native_calls("topal_x86_systems_file_position_finalize_v1"),
        1
    );
    assert_eq!(native_calls("topal_kernel_region_action_apply_v1"), 1);
    let destination = directory.join("refused");
    let error = publish_x86_64_systems_artifact_with_kernel_libraries(
        &program,
        &tools,
        None,
        None,
        SystemsKernelLibraries::default(),
        &destination,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("declaration requires its matching explicit action source snapshot"),
        "{error}"
    );
    assert!(!destination.exists());
}

#[test]
fn region_action_linked_constants_reject_tampering_independently_of_function_bytes() {
    // TOPAL-SYSTEMS-REGION-ACTION-SOURCE-001: source constants are executable input.
    let tools = LlvmTools::discover(None).unwrap();
    let directory = work();
    let input = input(&directory);
    let unit = build_systems_region_action_library(&input, "advance", &tools, &directory).unwrap();
    let executable = client(
        &directory,
        unit.record.allocation.numeric.maximum_bytes,
        false,
    );
    let bytes = std::fs::read(&executable).unwrap();
    let bindings = inspect_systems_region_action_data(&bytes, &unit.record).unwrap();
    assert_eq!(bindings.len(), unit.record.read_only_data.len());
    // Independently walk ELF64 section addresses/file offsets; alter one bound
    // constant byte without changing any executable instruction or symbol.
    let u16_at =
        |offset: usize| u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap()) as usize;
    let u64_at = |offset: usize| u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
    let headers = u64_at(40) as usize;
    let stride = u16_at(58);
    let count = u16_at(60);
    let address = bindings[0].address;
    let offset = (0..count)
        .find_map(|index| {
            let header = headers + index * stride;
            let start = u64_at(header + 16);
            let size = u64_at(header + 32);
            (address >= start && address - start < size)
                .then(|| (u64_at(header + 24) + address - start) as usize)
        })
        .unwrap();
    let mut changed = bytes.clone();
    changed[offset] ^= 1;
    let error = inspect_systems_region_action_data(&changed, &unit.record).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("immutable region data bytes changed"),
        "{error}"
    );
    for mode in 0..6 {
        let mut changed = unit.record.clone();
        match mode {
            0 => changed.read_only_data.clear(),
            1 => changed.read_only_data[0].alignment = 1,
            2 => changed.read_only_data[0].bytes += 8,
            3 => changed.read_only_data[0].symbol = "topal.region.action.missing".into(),
            4 => changed.scalar_count = Some(2),
            5 => changed.allocation.adapter_family_sha256 = Some("0".repeat(64)),
            _ => unreachable!(),
        }
        assert!(
            inspect_systems_region_action_data(&bytes, &changed).is_err(),
            "case {mode}"
        );
    }
}

#[test]
fn framed_region_kernel_publication_binds_actual_caller_storage_code_and_constants() {
    // TOPAL-SYSTEMS-REGION-ACTION-SOURCE-001, -OWNER-001, -RETURN-001, -POSITION-001.
    // TOPAL-SYSTEMS-REGION-FRAME-SOURCE-001, -FRAME-ABI-001.
    // Structural linked/kernel packaging evidence; VM execution remains separate.
    use topal_language::compiler::{INITIAL_SYSTEMS_BOARD, INITIAL_SYSTEMS_TARGET};
    let directory = work();
    let kernel_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../kernel")
        .canonicalize()
        .unwrap();
    let root = include_str!("../../../kernel/arch/x86_64/toolchain-gate.t");
    let user = directory.join("user.elf");
    std::fs::write(&user, qualified_x86_64_static_user_image_fixture()).unwrap();
    let options = CompileOptions {
        source_name: directory.join("root.t").to_string_lossy().into(),
        output: directory.join("linked"),
        emit: Emit::Executable,
        llvm_tools: None,
        library_root: kernel_root,
        target: TargetSelection {
            target: Some(INITIAL_SYSTEMS_TARGET.into()),
            board: Some(INITIAL_SYSTEMS_BOARD.into()),
            ..Default::default()
        },
        optimization: Default::default(),
        standard_library: None,
        initial_user_image: None,
        static_user_image: Some(user),
    };
    let artifact = publish_systems_source(root, &options).unwrap();
    let library = artifact.record.file_position_library.as_ref().unwrap();
    let binding = artifact.record.file_position_runtime.as_ref().unwrap();
    let encoded = serde_json::to_value(&artifact.record).unwrap();
    let decoded: SystemsArtifactProvenance = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(
        decoded.file_position_runtime.as_ref().unwrap().scalar_count,
        Some(2)
    );
    for target in ["source", "family", "runtime", "unit-source", "unit-family"] {
        let mut changed = encoded.clone();
        match target {
            "source" => changed["file_position_library"]["scalar_count"] = serde_json::Value::Null,
            "family" => {
                changed["file_position_library"]["allocation"]["adapter_family_sha256"] =
                    serde_json::Value::Null
            }
            "runtime" => changed["file_position_runtime"]["scalar_count"] = serde_json::Value::Null,
            "unit-source" => changed["region_library"]["scalar_count"] = serde_json::Value::Null,
            "unit-family" => {
                changed["region_library"]["allocation"]["adapter_family_sha256"] =
                    serde_json::Value::Null
            }
            _ => unreachable!(),
        }
        assert!(
            serde_json::from_value::<SystemsArtifactProvenance>(changed).is_err(),
            "{target}"
        );
    }
    assert_eq!(
        library.schema,
        "topal.kernel-region-frame-source-object.x86_64/2"
    );
    assert_eq!(library.scalar_count, Some(2));
    assert_eq!(
        binding.schema,
        "topal.file-position-region-invocation.x86_64/3"
    );
    assert_eq!(binding.scalar_count, Some(2));
    assert_eq!(library.instance.as_deref(), Some("file-position"));
    assert_eq!(binding.instance.as_deref(), Some("file-position"));
    assert_eq!(
        binding.combined_stack_bytes,
        library.maximum_stack_bytes + 88 + 32
    );
    assert!(binding.combined_stack_bytes <= 4096);
    assert_eq!(binding.storage.len(), 3);
    assert_eq!(binding.code.len(), 4);
    assert_eq!(binding.source_data.len(), library.read_only_data.len());
    assert_eq!(
        binding.controller_code.symbol,
        X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL
    );
    let kernel = std::fs::read(&artifact.kernel).unwrap();
    let initial = std::fs::read(&artifact.initial_user_image).unwrap();
    let user = std::fs::read(artifact.static_user_image.as_ref().unwrap()).unwrap();
    let boot =
        generate_x86_64_linux_boot_image(&kernel, &artifact.record, &initial, &user).unwrap();
    std::fs::write(directory.join("bzImage"), boot.bytes).unwrap();
    std::fs::write(directory.join("initramfs.cpio"), boot.boot_resource).unwrap();
    eprintln!("action kernel candidate: {}", directory.display());
    // Correcting the outer image digest cannot legitimize changed constants,
    // ordinary instructions, caller code or source invocation machinery.
    for (case, address) in [
        ("constant", binding.source_data[0].address),
        ("ordinary-code", binding.source_code[0].address),
        ("caller-code", binding.controller_code.address),
        ("native-code", binding.code[0].address),
    ] {
        let mut changed_kernel = kernel.clone();
        let u16_at = |offset: usize| {
            u16::from_le_bytes(kernel[offset..offset + 2].try_into().unwrap()) as usize
        };
        let u64_at =
            |offset: usize| u64::from_le_bytes(kernel[offset..offset + 8].try_into().unwrap());
        let headers = u64_at(40) as usize;
        let stride = u16_at(58);
        let offset = (0..u16_at(60))
            .find_map(|index| {
                let header = headers + index * stride;
                let start = u64_at(header + 16);
                let size = u64_at(header + 32);
                (address >= start && address - start < size)
                    .then(|| (u64_at(header + 24) + address - start) as usize)
            })
            .unwrap();
        changed_kernel[offset] ^= 1;
        let path = directory.join(format!("changed-{case}.elf"));
        std::fs::write(&path, &changed_kernel).unwrap();
        let digest = run(Command::new("sha256sum").arg(path))
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .to_owned();
        let mut changed = artifact.record.clone();
        changed
            .outputs
            .iter_mut()
            .find(|o| o.identity == SYSTEMS_KERNEL_FILE)
            .unwrap()
            .sha256 = digest;
        let error = generate_x86_64_linux_boot_image(&changed_kernel, &changed, &initial, &user)
            .unwrap_err();
        assert!(
            error.to_string().contains("E-SYSTEMS-REGION-ACTION")
                || error.to_string().contains("E-SYSTEMS-REGION-LIBRARY"),
            "{case}: {error}"
        );
    }
    for mode in 0..15 {
        let mut changed = artifact.record.clone();
        match mode {
            0 => {
                changed
                    .file_position_runtime
                    .as_mut()
                    .unwrap()
                    .combined_stack_bytes -= 8
            }
            1 => {
                changed
                    .file_position_library
                    .as_mut()
                    .unwrap()
                    .allocation
                    .numeric
                    .bridge_bytes -= 24
            }
            2 => {
                changed
                    .file_position_runtime
                    .as_mut()
                    .unwrap()
                    .controller_code
                    .sha256 = "0".repeat(64)
            }
            3 => {
                changed.file_position_runtime.as_mut().unwrap().source_data[0].sha256 =
                    "0".repeat(64)
            }
            4 => {
                changed
                    .file_position_runtime
                    .as_mut()
                    .unwrap()
                    .controller_owner
                    .address += 4096
            }
            5 => changed.file_position_library = None,
            6 => changed.file_position_runtime = None,
            7 => changed.file_position_library.as_mut().unwrap().scalar_count = Some(3),
            8 => changed.file_position_library.as_mut().unwrap().scalar_count = None,
            9 => changed.file_position_runtime.as_mut().unwrap().scalar_count = Some(1),
            10 => {
                changed
                    .file_position_library
                    .as_mut()
                    .unwrap()
                    .allocation
                    .adapter_family_sha256 = None
            }
            11 => {
                changed
                    .file_position_library
                    .as_mut()
                    .unwrap()
                    .allocation
                    .adapter_sha256 = "0".repeat(64)
            }
            12 => {
                changed.file_position_library.as_mut().unwrap().native_abi =
                    "topal.kernel-region-action-control-view64-u64x2-bool.x86_64-sysv/1".into()
            }
            13 => {
                changed.file_position_runtime.as_mut().unwrap().schema =
                    "topal.file-position-region-invocation.x86_64/1".into()
            }
            14 => {
                changed.file_position_runtime.as_mut().unwrap().interface =
                    "borrowed-byte-region-action-v1".into()
            }
            _ => unreachable!(),
        }
        assert!(
            generate_x86_64_linux_boot_image(&kernel, &changed, &initial, &user).is_err(),
            "case {mode}"
        );
    }
    // A separate named root allocation must not alias an action arena, even
    // when the attacker changes only ELF metadata and corrects the outer hash.
    let read32 = |at: usize| u32::from_le_bytes(kernel[at..at + 4].try_into().unwrap()) as usize;
    let read64 = |at: usize| u64::from_le_bytes(kernel[at..at + 8].try_into().unwrap()) as usize;
    let headers = read64(40);
    let stride = u16::from_le_bytes(kernel[58..60].try_into().unwrap()) as usize;
    let count = u16::from_le_bytes(kernel[60..62].try_into().unwrap()) as usize;
    let mut value_offset = None;
    for index in 0..count {
        let header = headers + index * stride;
        if read32(header + 4) != 2 {
            continue;
        }
        let strings = headers + read32(header + 40) * stride;
        let strings_start = read64(strings + 24);
        let start = read64(header + 24);
        let end = start + read64(header + 32);
        for entry in (start..end).step_by(read64(header + 56)) {
            let name = strings_start + read32(entry);
            let name_end = kernel[name..].iter().position(|b| *b == 0).unwrap() + name;
            if &kernel[name..name_end] == b"topal_x86_systems_region_arena" {
                assert!(value_offset.replace(entry + 8).is_none());
            }
        }
    }
    let mut aliased = kernel.clone();
    let offset = value_offset.unwrap();
    aliased[offset..offset + 8].copy_from_slice(&binding.storage[0].address.to_le_bytes());
    let path = directory.join("aliased-root-action-arena.elf");
    std::fs::write(&path, &aliased).unwrap();
    let digest = run(Command::new("sha256sum").arg(path))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    let mut changed = artifact.record.clone();
    changed
        .outputs
        .iter_mut()
        .find(|o| o.identity == SYSTEMS_KERNEL_FILE)
        .unwrap()
        .sha256 = digest;
    let error = generate_x86_64_linux_boot_image(&aliased, &changed, &initial, &user).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("action storage aliases another named allocation"),
        "{error}"
    );
}
