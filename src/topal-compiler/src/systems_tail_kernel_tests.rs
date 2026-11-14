use super::*;
use crate::{LlvmTools, SystemsKernelLibraries};
use std::{fs, path::Path, process::Command};
use topal_language::compiler::{CompilerSystemsTargetSelection, analyze_systems_for_compiler};

const ROOT: &str = include_str!("../../../linux-kernel/labs/qemu/x86_64/tail-kernel-root.t");

fn run(work: &Path, label: &str, command: &mut Command) {
    let result = command.output().unwrap();
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|s| s.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    fs::write(
        work.join(format!("{label}.command.json")),
        serde_json::to_vec_pretty(&serde_json::json!({
            "argv": argv, "exit_code": result.status.code()
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(work.join(format!("{label}.stdout.log")), &result.stdout).unwrap();
    fs::write(work.join(format!("{label}.stderr.log")), &result.stderr).unwrap();
    assert!(
        result.status.success(),
        "{label} in {}: {}\n{}\n{}",
        work.display(),
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn typed_root_tail_component_runs_in_privileged_guest_with_retirement() {
    // TOPAL-SYSTEMS-TAIL-KERNEL-001. Unpublished experiment: no artifact grant.
    let program =
        analyze_systems_for_compiler(ROOT, &CompilerSystemsTargetSelection::initial_x86_64_qemu())
            .unwrap();
    let source = include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t");
    let body =
        topal_language::compiler::analyze_kernel_region_tail_source(source, &[], "clear-prefix", 3)
            .unwrap();
    let entry = &body
        .functions
        .iter()
        .find(|f| f.source_name == "clear-prefix")
        .unwrap()
        .symbol;
    let owner = crate::emit_systems_tail_owner_component(&body, "algorithms/clear-prefix.t", entry)
        .unwrap();
    let tools = LlvmTools::discover(None).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = std::env::temp_dir().join(format!("tail-kernel-{}-{nonce}", std::process::id()));
    fs::create_dir(&work).unwrap();
    eprintln!("unpublished tail kernel fixture: {}", work.display());
    let trace = topal_semantics::model_systems_transitions(&program).unwrap();
    assert!(trace.iter().any(|step| matches!(
        step,
        topal_semantics::SystemsTransition::ResumeBootstrapRegionAfterLibrary {
            require_true: true
        }
    )));
    fs::write(work.join("root-model.txt"), format!("{trace:#?}\n")).unwrap();
    fs::write(
        work.join("owner-component.json"),
        serde_json::to_vec_pretty(&owner).unwrap(),
    )
    .unwrap();
    fs::write(work.join("root.t"), ROOT).unwrap();
    fs::write(work.join("source.t"), source).unwrap();
    fs::write(
        work.join("root.o"),
        crate::systems_artifact::generate_root_object(&program).unwrap(),
    )
    .unwrap();
    fs::write(
        work.join("provider.o"),
        crate::generate_x86_64_systems_provider_object(&program)
            .unwrap()
            .bytes,
    )
    .unwrap();
    fs::write(work.join("source.ll"), &owner.runtime.source_step.llvm).unwrap();
    fs::write(work.join("runtime.ll"), &owner.runtime.llvm).unwrap();
    fs::write(work.join("owner.ll"), &owner.llvm).unwrap();
    fs::write(work.join("boundary.o"), &owner.runtime.boundary).unwrap();
    for name in ["source", "runtime", "owner"] {
        run(
            &work,
            &format!("{name}-verify"),
            Command::new(tools.directory.join("opt"))
                .args(["-passes=verify", "-disable-output"])
                .arg(work.join(format!("{name}.ll"))),
        );
        run(
            &work,
            &format!("{name}-llc"),
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
                .arg(work.join(format!("{name}.ll")))
                .arg("-o")
                .arg(work.join(format!("{name}.o"))),
        );
        run(
            &work,
            &format!("{name}-disassembly"),
            Command::new(tools.directory.join("llvm-objdump"))
                .args(["-d", "--no-show-raw-insn"])
                .arg(work.join(format!("{name}.o"))),
        );
        crate::systems_library::inspect_instructions(
            &fs::read_to_string(work.join(format!("{name}-disassembly.stdout.log"))).unwrap(),
        )
        .unwrap();
    }
    let inner = crate::systems_tail_runtime::inspection::inspect(
        &fs::read(work.join("source.o")).unwrap(),
        &fs::read(work.join("runtime.o")).unwrap(),
    )
    .unwrap();
    let outer = crate::systems_tail_owner::inspection::inspect(
        &fs::read(work.join("owner.o")).unwrap(),
        &inner,
    )
    .unwrap();
    fs::write(
        work.join("inner-inspection.json"),
        serde_json::to_vec_pretty(&inner).unwrap(),
    )
    .unwrap();
    fs::write(
        work.join("owner-inspection.json"),
        serde_json::to_vec_pretty(&outer).unwrap(),
    )
    .unwrap();
    let owner_stack = outer["owner_stack_bytes"].as_u64().unwrap();
    let combined = outer["combined_normal_stack_bytes"].as_u64().unwrap();
    fs::write(
        work.join("kernel-owner.o"),
        component_object(65536, owner_stack, combined).unwrap(),
    )
    .unwrap();
    let mut link = Command::new(tools.directory.join("rust-lld"));
    link.args([
        "-flavor",
        "gnu",
        "-m",
        "elf_x86_64",
        "-static",
        "--no-dynamic-linker",
        "--build-id=none",
        "--entry",
        crate::X86_SYSTEMS_KERNEL_ENTRY,
        "--section-start=.bss=0x800000",
        "-z",
        "max-page-size=0x1000",
    ]);
    for name in [
        "root",
        "provider",
        "source",
        "runtime",
        "owner",
        "boundary",
        "kernel-owner",
    ] {
        link.arg(work.join(format!("{name}.o")));
    }
    link.arg("-o").arg(work.join("kernel.elf"));
    run(&work, "link", &mut link);
    let kernel = fs::read(work.join("kernel.elf")).unwrap();
    let boot = crate::systems_boot_image::unpublished_component_fixture(&kernel).unwrap();
    fs::write(work.join("bzImage"), boot).unwrap();
    let resource = crate::systems_boot_resource::generate_initial_boot_resource(
        &crate::systems_provider_object::initial_user_elf_file().unwrap(),
        &crate::qualified_x86_64_static_user_image_fixture(),
    )
    .unwrap();
    fs::write(work.join("initramfs.cpio"), resource).unwrap();
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/tail_kernel_inspect.py");
    for mode in [
        "normal",
        "source_abort",
        "false_result",
        "bad_key",
        "busy",
        "interrupts",
        "bad_stack",
        "generation_exhaustion",
        "frame_exhaustion",
        "dirty_private",
    ] {
        run(
            &work,
            &format!("guest-{mode}"),
            Command::new("python3")
                .arg(&script)
                .arg(&work)
                .arg("--mode")
                .arg(mode)
                .arg("--retired-bytes")
                .arg(
                    (FRAME_BYTES + owner_stack)
                        .div_ceil(8)
                        .checked_mul(8)
                        .unwrap()
                        .to_string(),
                ),
        );
        assert!(
            fs::read_to_string(work.join(format!("guest-{mode}.stdout.log")))
                .unwrap()
                .contains("TAIL_KERNEL_PASS")
        );
    }
    println!(
        "TAIL_KERNEL_PASS work={} owner_stack={owner_stack} combined={combined}",
        work.display()
    );
}

#[test]
fn tail_root_profile_is_explicit_and_canonical_publication_requires_binding() {
    // TOPAL-SYSTEMS-TAIL-KERNEL-001: never borrow a legacy acyclic certificate.
    let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let program = analyze_systems_for_compiler(ROOT, &target).unwrap();
    let provider = crate::generate_x86_64_systems_provider_object(&program).unwrap();
    assert!(
        provider
            .bytes
            .windows(b"region-history-bytes=40".len())
            .any(|w| w == b"region-history-bytes=40")
    );
    assert!(
        provider
            .bytes
            .windows(b"region-kernel-qualification=requires-reconstructed-source-native-caller-binding".len())
            .any(|w| w == b"region-kernel-qualification=requires-reconstructed-source-native-caller-binding")
    );
    for text in [
        ROOT.replace("stack-limit-bytes is 4096", "stack-limit-bytes is 8192"),
        ROOT.replace(
            "runtime-limit-bytes is 65536",
            "runtime-limit-bytes is 131072",
        ),
        ROOT.replace("byte-count is 64", "byte-count is 63"),
        ROOT.replace("  region-library is", "  ram-page-pool-library is"),
    ] {
        assert!(analyze_systems_for_compiler(&text, &target).is_err());
    }
    let tools = LlvmTools::discover(None).unwrap();
    let destination =
        std::env::temp_dir().join(format!("tail-no-publication-{}", std::process::id()));
    let error = crate::publish_x86_64_systems_artifact_with_kernel_libraries(
        &program,
        &tools,
        None,
        None,
        SystemsKernelLibraries {
            policy: None,
            time: None,
            image: None,
            dispatch: None,
            process_identity: None,
            region: None,
            file_position: None,
            ram_page_pool: None,
            inode_mode: None,
            namespace: None,
        },
        &destination,
    )
    .err()
    .unwrap();
    assert!(
        error
            .to_string()
            .contains("complete kernel tail owner source/native/caller binding"),
        "{error}"
    );
    assert!(!destination.exists());
    for (storage, own, combined) in [
        (4159, 240, 688),
        (65536, 4097, 688),
        (65536, 240, 4096),
        (65536, 4096, 688),
        (65536, 0, 688),
        (65536, 239, 688),
        (65536, 240, 687),
    ] {
        assert!(component_object(storage, own, combined).is_err());
    }
}
