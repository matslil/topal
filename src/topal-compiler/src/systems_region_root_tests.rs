use crate::*;
use object::{Object, ObjectSection, ObjectSymbol};
use std::path::{Path, PathBuf};
use topal_language::compiler::{
    CompilerSystemsProgram, CompilerSystemsTargetSelection, analyze_systems_for_compiler,
};

const ROOT: &str = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");

fn program() -> CompilerSystemsProgram {
    analyze_systems_for_compiler(ROOT, &CompilerSystemsTargetSelection::initial_x86_64_qemu())
        .unwrap()
}
fn work() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("topal-root-region-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    path
}
fn load(
    requirement: &Option<topal_language::compiler::CompilerSystemsSourceLibraryRequirement>,
    root: &Path,
) -> Option<SystemsSourceLibraryInput> {
    requirement
        .as_ref()
        .map(|requirement| crate::systems_library::load_source_library(requirement, root).unwrap())
}

#[test]
fn root_region_publishes_exact_native_ownership_and_rejects_code_and_bound_tampering() {
    // TOPAL-SYSTEMS-REGION-ROOT-BINDING-001, -CALL-001, -RECLAIM-001.
    let program = program();
    let tools = LlvmTools::discover(None).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/kernel")
        .canonicalize()
        .unwrap();
    let policy = load(&program.kernel_library, &root);
    let region = load(&program.region_library, &root);
    let position = load(&program.file_position_library, &root);
    let page = load(&program.ram_page_pool_library, &root);
    let inode = load(&program.inode_mode_library, &root);
    let time = load(&program.time_library, &root);
    let image = load(&program.image_library, &root);
    let dispatch = load(&program.dispatch_library, &root);
    let identity = load(&program.process_identity_library, &root);
    let parent = work();
    let artifact = publish_x86_64_systems_artifact_with_kernel_libraries(
        &program,
        &tools,
        None,
        Some(&qualified_x86_64_static_user_image_fixture()),
        SystemsKernelLibraries {
            inode_mode: inode.as_ref(),
            ram_page_pool: page.as_ref(),
            file_position: position.as_ref(),
            region: region.as_ref(),
            policy: policy.as_ref(),
            time: time.as_ref(),
            image: image.as_ref(),
            dispatch: dispatch.as_ref(),
            process_identity: identity.as_ref(),
        },
        &parent.join("kernel"),
    )
    .unwrap();
    eprintln!("root region artifact: {}", artifact.directory.display());
    let binding = artifact.record.region_runtime.as_ref().unwrap();
    let source_record = artifact.record.region_library.as_ref().unwrap();
    assert_eq!(source_record.instance.as_deref(), Some("root-region"));
    assert_eq!(source_record.scalar_count, Some(0));
    assert_eq!(binding.schema, "topal.root-region-invocation.x86_64/3");
    assert_eq!(binding.instance.as_deref(), Some("root-region"));
    assert_eq!(binding.scalar_count, Some(0));
    assert_eq!(binding.live_frame_bytes, Some(56));
    assert_eq!(
        binding.combined_stack_bytes,
        source_record.maximum_stack_bytes + 56 + 32
    );
    let field = artifact.record.file_position_runtime.as_ref().unwrap();
    assert!(binding.storage.iter().all(|root| {
        field.storage.iter().all(|field| {
            root.address + root.bytes <= field.address
                || field.address + field.bytes <= root.address
        })
    }));
    assert!(binding.combined_stack_bytes <= 4096);
    assert_eq!(binding.caller_alignment_bytes, 32);
    assert_eq!(binding.imported_hook_stack_bytes, 8);
    assert_eq!(binding.storage.len(), 3);
    assert_eq!(binding.code.len(), 4);
    assert_eq!(
        binding.source_code.len(),
        artifact
            .record
            .region_library
            .as_ref()
            .unwrap()
            .functions
            .len()
    );
    assert!(
        artifact
            .record
            .semantic_trace
            .iter()
            .any(|step| step.contains("runtime-guard;offset=1;expected=90"))
    );
    assert!(artifact.record.semantic_trace.iter().any(|step| {
        step.contains("runtime-guard;required-source-boolean=true;instance=root-region")
    }));
    let kernel = std::fs::read(&artifact.kernel).unwrap();
    let initial = std::fs::read(&artifact.initial_user_image).unwrap();
    let user = std::fs::read(artifact.static_user_image.as_ref().unwrap()).unwrap();
    let encoded = serde_json::to_value(&artifact.record).unwrap();
    for (context, fields) in [
        ("region_library", &["instance", "scalar_count"][..]),
        (
            "region_runtime",
            &["instance", "scalar_count", "live_frame_bytes"][..],
        ),
        ("file_position_library", &["instance", "scalar_count"][..]),
        ("file_position_runtime", &["instance", "scalar_count"][..]),
    ] {
        for field in fields {
            for missing in [false, true] {
                let mut value = encoded.clone();
                if missing {
                    value[context].as_object_mut().unwrap().remove(*field);
                } else {
                    value[context][*field] = serde_json::Value::Null;
                }
                match serde_json::from_value::<SystemsArtifactProvenance>(value) {
                    Err(_) => {}
                    Ok(changed) => assert!(
                        generate_x86_64_linux_boot_image(&kernel, &changed, &initial, &user)
                            .is_err(),
                        "accepted {context}.{field} missing={missing}"
                    ),
                }
            }
        }
    }
    for mutation in 0..10 {
        let mut changed = artifact.record.clone();
        match mutation {
            0 => changed.region_library.as_mut().unwrap().instance = Some("file-position".into()),
            1 => changed.region_runtime.as_mut().unwrap().instance = Some("file-position".into()),
            2 => {
                changed.file_position_library.as_mut().unwrap().instance =
                    Some("root-region".into())
            }
            3 => {
                changed.file_position_runtime.as_mut().unwrap().instance =
                    Some("root-region".into())
            }
            4 => changed.region_runtime.as_mut().unwrap().live_frame_bytes = Some(48),
            5 => changed.region_library.as_mut().unwrap().scalar_count = Some(2),
            6 => changed.region_runtime.as_mut().unwrap().scalar_count = Some(2),
            7 => changed.file_position_library.as_mut().unwrap().scalar_count = Some(0),
            8 => changed.file_position_runtime.as_mut().unwrap().scalar_count = Some(0),
            9 => {
                changed.region_library.as_mut().unwrap().imported_hooks = changed
                    .file_position_library
                    .as_ref()
                    .unwrap()
                    .imported_hooks
                    .clone()
            }
            _ => unreachable!(),
        }
        assert!(
            generate_x86_64_linux_boot_image(&kernel, &changed, &initial, &user).is_err(),
            "accepted cross-instance mutation {mutation}"
        );
    }
    let boot =
        generate_x86_64_linux_boot_image(&kernel, &artifact.record, &initial, &user).unwrap();
    std::fs::write(parent.join("bzImage"), boot.bytes).unwrap();
    std::fs::write(parent.join("initramfs.cpio"), boot.boot_resource).unwrap();
    let mut wrong = artifact.record.clone();
    wrong.region_runtime.as_mut().unwrap().combined_stack_bytes -= 8;
    let error = generate_x86_64_linux_boot_image(&kernel, &wrong, &initial, &user)
        .err()
        .unwrap();
    assert!(
        error
            .to_string()
            .contains("root runtime or combined bounds mismatch"),
        "{error}"
    );
    let mut wrong = artifact.record.clone();
    wrong
        .region_library
        .as_mut()
        .unwrap()
        .imported_hook_stack_bytes = 0;
    assert!(generate_x86_64_linux_boot_image(&kernel, &wrong, &initial, &user).is_err());
    for mutation in 0..6 {
        let mut wrong = artifact.record.clone();
        let library = wrong.region_library.as_mut().unwrap();
        match mutation {
            0 => library.allocation.numeric.source_evaluation_bytes -= 1,
            1 => library.functions[0].maximum_call_path_bytes -= 1,
            2 => library.functions.push(library.functions[0].clone()),
            3 => library.maximum_stack_bytes -= 8,
            4 => library.allocation.maximum_view_nodes = 0,
            _ => library.platform_hooks_sha256 = "0".repeat(64),
        }
        assert!(
            generate_x86_64_linux_boot_image(&kernel, &wrong, &initial, &user).is_err(),
            "accepted region tamper {mutation}"
        );
    }
    let mut invalid = region.as_ref().unwrap().clone();
    invalid.text = invalid
        .text
        .replace("-> Boolean\n", "-> Boolean\n  : Effects ()\n");
    let destination = parent.join("invalid-source");
    let error = publish_x86_64_systems_artifact_with_kernel_libraries(
        &program,
        &tools,
        None,
        Some(&qualified_x86_64_static_user_image_fixture()),
        SystemsKernelLibraries {
            inode_mode: inode.as_ref(),
            ram_page_pool: page.as_ref(),
            file_position: position.as_ref(),
            region: Some(&invalid),
            policy: policy.as_ref(),
            time: time.as_ref(),
            image: image.as_ref(),
            dispatch: dispatch.as_ref(),
            process_identity: identity.as_ref(),
        },
        &destination,
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("effect"), "{error}");
    assert!(
        !destination.exists(),
        "invalid source published a partial artifact"
    );
    let file = object::File::parse(kernel.as_slice()).unwrap();
    let helper = artifact
        .record
        .region_library
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| function.symbol.contains("copy_2dbyte"))
        .unwrap();
    let symbol = file.symbol_by_name(&helper.symbol).unwrap();
    let section = file
        .section_by_index(symbol.section_index().unwrap())
        .unwrap();
    let offset = section.file_range().unwrap().0 + symbol.address() - section.address();
    let mut changed = kernel.clone();
    changed[offset as usize] ^= 1;
    let mut wrong = artifact.record.clone();
    wrong
        .outputs
        .iter_mut()
        .find(|output| output.identity == SYSTEMS_KERNEL_FILE)
        .unwrap()
        .sha256 = crate::artifact::sha256(&changed);
    let error = generate_x86_64_linux_boot_image(&changed, &wrong, &initial, &user)
        .err()
        .unwrap();
    assert!(
        error
            .to_string()
            .contains("root runtime or combined bounds mismatch"),
        "{error}"
    );
    let mut wrong = artifact.record.clone();
    wrong.region_runtime.as_mut().unwrap().source_code[0].sha256 = "0".repeat(64);
    assert!(generate_x86_64_linux_boot_image(&kernel, &wrong, &initial, &user).is_err());
    let symbol = file
        .symbol_by_name(
            &artifact
                .record
                .region_library
                .as_ref()
                .unwrap()
                .imported_hooks[0],
        )
        .unwrap();
    let section = file
        .section_by_index(symbol.section_index().unwrap())
        .unwrap();
    let offset = section.file_range().unwrap().0 + symbol.address() - section.address();
    let mut changed = kernel.clone();
    changed[offset as usize] ^= 1;
    let mut wrong = artifact.record.clone();
    wrong
        .outputs
        .iter_mut()
        .find(|output| output.identity == SYSTEMS_KERNEL_FILE)
        .unwrap()
        .sha256 = crate::artifact::sha256(&changed);
    let error = generate_x86_64_linux_boot_image(&changed, &wrong, &initial, &user)
        .err()
        .unwrap();
    assert!(
        error
            .to_string()
            .contains("linked native root template differs"),
        "{error}"
    );
}
