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
    let kernel = std::fs::read(&artifact.kernel).unwrap();
    let initial = std::fs::read(&artifact.initial_user_image).unwrap();
    let user = std::fs::read(artifact.static_user_image.as_ref().unwrap()).unwrap();
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
        .replace("-> Unit\n", "-> Unit\n  : Effects ()\n");
    let destination = parent.join("invalid-source");
    let error = publish_x86_64_systems_artifact_with_kernel_libraries(
        &program,
        &tools,
        None,
        Some(&qualified_x86_64_static_user_image_fixture()),
        SystemsKernelLibraries {
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
        .symbol_by_name(crate::systems_region_library::ALLOCATE)
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
