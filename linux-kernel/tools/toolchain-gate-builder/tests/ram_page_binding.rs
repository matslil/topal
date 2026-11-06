use object::{Object, ObjectSection, ObjectSymbol};
use sha2::{Digest, Sha256};
use std::path::Path;
use topal_compiler::*;
use topal_language::compiler::{CompilerSystemsTargetSelection, analyze_systems_for_compiler};

#[test]
fn real_ram_page_owner_binds_four_disjoint_instances_and_rejects_tampering() {
    // TOPAL-SYSTEMS-REGION-PAGE-{OWNER,ALLOCATE,RELEASE}-001.
    let text = include_str!("../../../kernel/arch/x86_64/toolchain-gate.t");
    let declaration = "  ram-page-pool-library is lang systems source-library (\n    source is \"memory/ram-page-pool.t\",\n    export is apply,\n    interface is borrowed-byte-region-instance-action-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";
    let text = if text.contains("  ram-page-pool-library is") {
        text.to_owned()
    } else {
        text.replace(
            "  region-library is",
            &(declaration.to_owned() + "  region-library is"),
        )
    };
    let program = analyze_systems_for_compiler(
        &text,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kernel");
    let load = |requirement: &Option<
        topal_language::compiler::CompilerSystemsSourceLibraryRequirement,
    >| {
        requirement.as_ref().map(|r| {
            // Independent immutable source closure for each actual owning slot.
            let helpers: &[&str] = match r.source.as_str() {
                "memory/ram-page-pool.t" => &["support/page-slots.t"],
                "policy/file-position.t" => &["support/word-bytes.t"],
                "policy/inode-mode-action.t" => &["support/inode-metadata.t"],
                "policy/bootstrap-byte-copy-frame.t" => {
                    &["policy/bootstrap-byte-copy.t", "support/bytes.t"]
                }
                "policy/file-open-access.t" => &[],
                "policy/relative-sleep-deadline.t" => &["support/clock.t"],
                "policy/static-elf64-header.t" => &["support/elf.t"],
                "arch/x86_64/policy/static-linux-dispatch-v4.t"
                | "arch/x86_64/policy/static-linux-dispatch-v5.t" => &["support/syscall.t"],
                "policy/structured-process-identity.t" => {
                    &["support/identity-history.t", "support/process-identity.t"]
                }
                other => panic!("unqualified fixture source {other}"),
            };
            SystemsSourceLibraryInput {
                source: r.source.clone(),
                source_name: root.join(&r.source).to_string_lossy().into(),
                text: std::fs::read_to_string(root.join(&r.source)).unwrap(),
                modules: helpers
                    .iter()
                    .map(|path| SystemsSourceModuleInput {
                        source: (*path).into(),
                        identity: path
                            .strip_suffix(".t")
                            .unwrap()
                            .split('/')
                            .map(str::to_owned)
                            .collect(),
                        text: std::fs::read_to_string(root.join(path)).unwrap(),
                    })
                    .collect(),
            }
        })
    };
    let policy = load(&program.kernel_library);
    let time = load(&program.time_library);
    let image = load(&program.image_library);
    let dispatch = load(&program.dispatch_library);
    let identity = load(&program.process_identity_library);
    let region = load(&program.region_library);
    let position = load(&program.file_position_library);
    let page = load(&program.ram_page_pool_library);
    let inode = load(&program.inode_mode_library);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent = std::env::temp_dir().join(format!(
        "topal-ram-page-binding-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&parent).unwrap();
    let tools = LlvmTools::discover(None).unwrap();
    let libraries = SystemsKernelLibraries {
        inode_mode: inode.as_ref(),
        ram_page_pool: page.as_ref(),
        region: region.as_ref(),
        file_position: position.as_ref(),
        policy: policy.as_ref(),
        time: time.as_ref(),
        image: image.as_ref(),
        dispatch: dispatch.as_ref(),
        process_identity: identity.as_ref(),
    };
    let mut missing = libraries;
    missing.ram_page_pool = None;
    assert!(
        publish_x86_64_systems_artifact_with_kernel_libraries(
            &program,
            &tools,
            None,
            None,
            missing,
            &parent.join("missing")
        )
        .is_err()
    );
    assert!(!parent.join("missing").exists());
    let artifact = publish_x86_64_systems_artifact_with_kernel_libraries(
        &program,
        &tools,
        None,
        Some(&qualified_x86_64_static_user_image_fixture()),
        libraries,
        &parent.join("kernel"),
    )
    .unwrap();
    let binding = artifact.record.ram_page_pool_runtime.as_ref().unwrap();
    assert_eq!(binding.instance.as_deref(), Some("ram-page-pool"));
    assert_eq!(binding.scalar_count, Some(2));
    assert_eq!(binding.schema, "topal.ram-page-region-invocation.x86_64/1");
    assert_eq!(
        (binding.map_offset, binding.map_bytes, binding.slots),
        (0x43018, 8, 8)
    );
    assert_eq!(binding.maximum_source_stack_bytes, 1152);
    assert_eq!(binding.combined_stack_bytes, 1400);
    assert_eq!(binding.live_frame_bytes, 184);
    assert_eq!(binding.storage.len(), 3);
    assert_eq!(binding.code.len(), 4);
    for other in [
        &artifact.record.inode_mode_runtime.as_ref().unwrap().storage,
        &artifact.record.region_runtime.as_ref().unwrap().storage,
        &artifact
            .record
            .file_position_runtime
            .as_ref()
            .unwrap()
            .storage,
    ] {
        assert!(binding.storage.iter().all(|page| {
            other.iter().all(|other| {
                page.address + page.bytes <= other.address
                    || other.address + other.bytes <= page.address
            })
        }));
    }
    let kernel = std::fs::read(&artifact.kernel).unwrap();
    let initial = std::fs::read(&artifact.initial_user_image).unwrap();
    let user = std::fs::read(artifact.static_user_image.as_ref().unwrap()).unwrap();
    generate_x86_64_linux_boot_image(&kernel, &artifact.record, &initial, &user).unwrap();
    for mode in 0..12 {
        let mut changed = artifact.record.clone();
        match mode {
            0 => {
                changed
                    .ram_page_pool_runtime
                    .as_mut()
                    .unwrap()
                    .combined_stack_bytes -= 8
            }
            1 => changed.ram_page_pool_runtime.as_mut().unwrap().map_offset += 1,
            2 => {
                changed.ram_page_pool_library.as_mut().unwrap().instance =
                    Some("file-position".into())
            }
            3 => changed.ram_page_pool_library.as_mut().unwrap().scalar_count = Some(1),
            4 => changed.ram_page_pool_runtime.as_mut().unwrap().storage[0].address += 8,
            5 => {
                changed.ram_page_pool_runtime.as_mut().unwrap().source_code[0].sha256 =
                    "00".repeat(32)
            }
            6 => {
                changed.ram_page_pool_runtime.as_mut().unwrap().source_data[0].sha256 =
                    "00".repeat(32)
            }
            7 => changed.ram_page_pool_runtime = None,
            8 => changed.ram_page_pool_library = None,
            9 => changed.volatile_files.page_allocation_map_offset = None,
            10 => {
                let mut entry = changed.inputs[0].clone();
                entry.identity = "kernel-ram-page-pool-source:foreign.t".into();
                changed.inputs.push(entry);
            }
            11 => {
                let mut entry = changed.outputs[0].clone();
                entry.identity = "ram-page-pool-library.foreign".into();
                changed.outputs.push(entry);
            }
            _ => unreachable!(),
        }
        assert!(
            generate_x86_64_linux_boot_image(&kernel, &changed, &initial, &user).is_err(),
            "mode {mode}"
        );
    }
    let encoded = serde_json::to_value(&artifact.record).unwrap();
    for field in ["ram_page_pool_library", "ram_page_pool_runtime"] {
        let mut null = encoded.clone();
        null[field] = serde_json::Value::Null;
        assert!(serde_json::from_value::<SystemsArtifactProvenance>(null).is_err());
    }
    // The exact linked source/controller instruction bytes, not just their
    // symbol names, must match the artifact's owning invocation certificate.
    let file = object::File::parse(kernel.as_slice()).unwrap();
    for symbol in [
        "topal_x86_systems_ram_page_apply_v1",
        "topal_kernel_region_frame_ram_page_pool_apply_v1",
    ] {
        let function = file.symbol_by_name(symbol).unwrap();
        let section = file
            .section_by_index(function.section_index().unwrap())
            .unwrap();
        let offset = section.file_range().unwrap().0 + function.address() - section.address();
        let mut changed = kernel.clone();
        changed[offset as usize] ^= 1;
        let mut rebound = artifact.record.clone();
        rebound
            .outputs
            .iter_mut()
            .find(|entry| entry.identity == SYSTEMS_KERNEL_FILE)
            .unwrap()
            .sha256 = format!("{:x}", Sha256::digest(&changed));
        assert!(generate_x86_64_linux_boot_image(&changed, &rebound, &initial, &user).is_err());
    }
    eprintln!("RAM page bound artifact: {}", artifact.directory.display());
    // Source absence retains its explicit native allocator and canonical
    // metadata omissions; another source instance grants no map authority.
    let mut absent = program.clone();
    absent.ram_page_pool_library = None;
    let legacy = publish_x86_64_systems_artifact_with_kernel_libraries(
        &absent,
        &tools,
        None,
        Some(&qualified_x86_64_static_user_image_fixture()),
        missing,
        &parent.join("native-only"),
    )
    .unwrap();
    assert!(legacy.record.ram_page_pool_library.is_none());
    assert!(legacy.record.ram_page_pool_runtime.is_none());
    assert_eq!(
        legacy.record.volatile_files.schema,
        "topal.volatile-file.x86_64/5"
    );
    assert!(
        legacy
            .record
            .volatile_files
            .page_allocation_map_offset
            .is_none()
    );
    let encoded = serde_json::to_value(&legacy.record).unwrap();
    assert!(encoded.get("ram_page_pool_library").is_none());
    assert!(encoded.get("ram_page_pool_runtime").is_none());
    let bytes = std::fs::read(&legacy.kernel).unwrap();
    let native = object::File::parse(bytes.as_slice()).unwrap();
    assert!(
        !native.symbols().any(|symbol| symbol
            .name()
            .is_ok_and(|name| name.starts_with("topal_x86_systems_ram_page_")
                || name.starts_with("topal.region-instance.ram-page-pool.")
                || name.starts_with("topal_kernel_region_frame_ram_page_pool_")))
    );
    generate_x86_64_linux_boot_image(&bytes, &legacy.record, &initial, &user).unwrap();
}
