use super::*;
use object::{Object, ObjectSection, ObjectSymbol};

const ROOT: &str = include_str!("../../../linux-kernel/labs/qemu/x86_64/tail-kernel-root.t");
fn source() -> Source {
    Source {
        source: "algorithms/clear-prefix.t".into(),
        source_name: "algorithms/clear-prefix.t".into(),
        text: include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t").into(),
        modules: Vec::new(),
    }
}
fn work() -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = std::env::temp_dir().join(format!(
        "tail-reconstruction-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&work).unwrap();
    work
}
#[test]
fn fresh_source_objects_bind_complete_relocated_kernel_inputs() {
    // TOPAL-SYSTEMS-TAIL-BINDING-001: fresh objects supply comparison inputs.
    let work = work();
    eprintln!("tail reconstruction fixture: {}", work.display());
    let tools = LlvmTools::discover(None).unwrap();
    let mut rebuilt = rebuild(ROOT, &source(), &tools, &work).unwrap();
    let mut link = Command::new(tools.tool_path("rust-lld"));
    link.args([
        "-flavor",
        "gnu",
        "-m",
        "elf_x86_64",
        "-static",
        "--no-dynamic-linker",
        "--build-id=none",
        "--section-start=.bss=0x800000",
        "--entry",
        crate::X86_SYSTEMS_KERNEL_ENTRY,
        "-z",
        "max-page-size=0x1000",
    ]);
    for object in &rebuilt.record.objects {
        link.arg(work.join(format!("{}.o", object.role)));
    }
    link.arg("-o").arg(work.join("kernel.elf"));
    run(&work, "link", &mut link).unwrap();
    let bytes = fs::read(work.join("kernel.elf")).unwrap();
    rebuilt.bind(&bytes).unwrap();
    assert_eq!(rebuilt.record.bounds.combined_with_caller, 824);
    assert_eq!(rebuilt.record.objects.len(), 7);
    assert_eq!(
        rebuilt.record.linked.len(),
        rebuilt
            .record
            .objects
            .iter()
            .map(|o| o.sections.len())
            .sum::<usize>()
    );
    assert_eq!(rebuilt.program.bootstrap_storage.capacity_bytes, 65536);
    assert!(rebuilt.record.linked.iter().any(|s| s.flags == 6));
    assert!(rebuilt.record.linked.iter().any(|s| s.flags == 2));
    assert!(rebuilt.record.linked.iter().any(|s| s.flags == 3));
    fs::write(
        work.join("reconstructed-binding.json"),
        serde_json::to_vec_pretty(&rebuilt.record).unwrap(),
    )
    .unwrap();
    let file = object::File::parse(bytes.as_slice()).unwrap();
    for name in [
        "topal_tail_step_component_v1",
        "topal_tail_owner_invoke_v1",
        "topal_x86_systems_region_library_apply_v1",
        crate::X86_SYSTEMS_KERNEL_ENTRY,
    ] {
        let symbol = file.symbol_by_name(name).unwrap();
        let section = file
            .section_by_index(symbol.section_index().unwrap())
            .unwrap();
        let offset = section.file_range().unwrap().0 + symbol.address() - section.address();
        let mut changed = bytes.clone();
        changed[offset as usize] ^= 1;
        assert!(
            rebuilt.bind(&changed).is_err(),
            "accepted changed code {name}"
        );
    }
    // Anonymous readonly sections are included, even without data symbols.
    let data = rebuilt
        .record
        .linked
        .iter()
        .find(|s| s.marker.starts_with("topal.tail-inputs.source.") && s.flags == 2)
        .unwrap();
    let begin = file
        .symbol_by_name(&format!("{}.begin", data.marker))
        .unwrap();
    let section = file
        .section_by_index(begin.section_index().unwrap())
        .unwrap();
    let offset = section.file_range().unwrap().0 + data.address - section.address();
    let mut changed = bytes.clone();
    changed[offset as usize] ^= 1;
    assert!(
        rebuilt.bind(&changed).is_err(),
        "accepted changed anonymous constants"
    );
    println!(
        "TAIL_RECONSTRUCTION_PASS work={} sections={}",
        work.display(),
        rebuilt.record.linked.len()
    );
}

#[test]
fn reconstruction_requires_real_root_and_complete_source_selection() {
    // TOPAL-SYSTEMS-TAIL-BINDING-001: never consume external native/range metadata.
    let tools = LlvmTools::discover(None).unwrap();
    let work = work();
    for mode in 0..5 {
        let mut root = ROOT.to_owned();
        let mut input = source();
        match mode {
            0 => input.source = "../outside.t".into(),
            1 => input.source = "algorithms/wrong.t".into(),
            2 => root = root.replace("byte-count is 64", "byte-count is 63"),
            3 => {
                root = root.replace(
                    "borrowed-byte-region-tail-action-v1",
                    "borrowed-byte-region-instance-action-v1",
                )
            }
            4 => input.modules.push(SystemsSourceModuleInput {
                source: "support/unused.t".into(),
                identity: vec!["support".into(), "unused".into()],
                text: "use language (version is v0.1)\npub identity is fn (n : Nat) -> Nat\n  n\n"
                    .into(),
            }),
            _ => unreachable!(),
        }
        assert!(
            rebuild(&root, &input, &tools, &work).is_err(),
            "mode {mode}"
        );
    }
}

#[test]
fn source_bearing_publication_reconstructs_at_boot_and_executes_kernel() {
    // TOPAL-SYSTEMS-TAIL-BINDING-001, TOPAL-SYSTEMS-TAIL-KERNEL-001.
    let tools = LlvmTools::discover(None).unwrap();
    let work = work();
    eprintln!("canonical tail artifact fixture: {}", work.display());
    let input = source().input();
    let artifact = crate::publish_x86_64_systems_tail_artifact(
        ROOT,
        &tools,
        None,
        Some(&crate::qualified_x86_64_static_user_image_fixture()),
        crate::SystemsKernelLibraries {
            region: Some(&input),
            ..Default::default()
        },
        &work.join("artifact"),
    )
    .unwrap();
    assert!(artifact.record.region_library.is_none() && artifact.record.region_runtime.is_none());
    let binding = artifact.record.tail_root.as_ref().unwrap();
    assert_eq!(binding.root_source, ROOT);
    assert_eq!(binding.bounds.combined_with_caller, 824);
    let kernel = fs::read(&artifact.kernel).unwrap();
    let initial = fs::read(&artifact.initial_user_image).unwrap();
    let user = fs::read(artifact.static_user_image.as_ref().unwrap()).unwrap();
    let boot = crate::generate_x86_64_linux_boot_image(&kernel, &artifact.record, &initial, &user)
        .unwrap();
    fs::write(work.join("kernel.elf"), &kernel).unwrap();
    fs::copy(&artifact.debug, work.join(crate::SYSTEMS_DEBUG_FILE)).unwrap();
    fs::write(work.join("bzImage"), boot.bytes).unwrap();
    fs::write(work.join("initramfs.cpio"), boot.boot_resource).unwrap();
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
            &format!("canonical-guest-{mode}"),
            Command::new("python3")
                .arg(&script)
                .arg(&work)
                .arg("--mode")
                .arg(mode)
                .arg("--retired-bytes")
                .arg(binding.bounds.retired_controller.to_string()),
        )
        .unwrap();
    }
    for mode in 0..14 {
        let mut changed = artifact.record.clone();
        match mode {
            0 => changed.tail_root = None,
            1 => changed.tail_root.as_mut().unwrap().bounds.combined -= 8,
            2 => changed.tail_root.as_mut().unwrap().objects[0].sha256 = "0".repeat(64),
            3 => changed.tail_root.as_mut().unwrap().component_sha256 = "0".repeat(64),
            4 => {
                changed.tail_root.as_mut().unwrap().root_source =
                    ROOT.replace("byte-count is 64", "byte-count is 63")
            }
            5 => changed
                .tail_root
                .as_mut()
                .unwrap()
                .linked
                .pop()
                .map(|_| ())
                .unwrap(),
            6 => changed.outputs.push(
                changed
                    .outputs
                    .iter()
                    .find(|o| o.identity == "tail-source.o")
                    .unwrap()
                    .clone(),
            ),
            7 => changed.tail_root.as_mut().unwrap().llvm_version = "0.0.0".into(),
            8 => changed
                .semantic_trace
                .push("forged caller permission".into()),
            9 => {
                changed
                    .inputs
                    .iter_mut()
                    .find(|i| i.identity == crate::X86_SYSTEMS_ROOT_OBJECT_REVISION)
                    .unwrap()
                    .sha256 = "0".repeat(64)
            }
            10 => {
                let value = changed.tail_root.as_ref().unwrap().objects[0].clone();
                changed.tail_root.as_mut().unwrap().objects.push(value);
            }
            11 => changed.platform_abi = "forged abi".into(),
            12 => changed.inputs.push(crate::DigestEntry {
                identity: "kernel-region-source:orphan.t".into(),
                sha256: "0".repeat(64),
            }),
            13 => changed.outputs.push(crate::DigestEntry {
                identity: "region-library.orphan.o".into(),
                sha256: "0".repeat(64),
            }),
            _ => unreachable!(),
        }
        fs::write(
            work.join(format!("tamper-{mode}.json")),
            serde_json::to_vec_pretty(&changed).unwrap(),
        )
        .unwrap();
        let error = crate::generate_x86_64_linux_boot_image(&kernel, &changed, &initial, &user)
            .err()
            .unwrap_or_else(|| panic!("accepted mutation {mode}"));
        fs::write(
            work.join(format!("tamper-{mode}.error.txt")),
            error.to_string(),
        )
        .unwrap();
    }
    let mut encoded = serde_json::to_value(&artifact.record).unwrap();
    encoded["tail_root"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<crate::SystemsArtifactProvenance>(encoded).is_err());
    let mut modified = binding.library.clone();
    assert!(modified.text.contains("value is 0"));
    modified.text = modified.text.replace("value is 0", "value is 1");
    let mut forged = rebuild(ROOT, &modified, &tools, &work.join("forged-source"))
        .unwrap()
        .record;
    forged.linked = binding.linked.clone();
    let mut changed = artifact.record.clone();
    changed.tail_root = Some(forged);
    let error = crate::generate_x86_64_linux_boot_image(&kernel, &changed, &initial, &user)
        .err()
        .unwrap();
    assert!(
        error.to_string().starts_with("E-SYSTEMS-TAIL-LINK:"),
        "{error}"
    );
    fs::write(work.join("forged-source.error.txt"), error.to_string()).unwrap();
    reject_changed_kernel(&kernel, &artifact.record, &initial, &user, &work);
    println!("TAIL_PUBLICATION_PASS work={}", work.display());
}

fn reject_changed_kernel(
    kernel: &[u8],
    record: &crate::SystemsArtifactProvenance,
    initial: &[u8],
    user: &[u8],
    work: &Path,
) {
    let file = object::File::parse(kernel).unwrap();
    let phoff = u64::from_le_bytes(kernel[32..40].try_into().unwrap()) as usize;
    let stride = u16::from_le_bytes(kernel[54..56].try_into().unwrap()) as usize;
    let count = u16::from_le_bytes(kernel[56..58].try_into().unwrap()) as usize;
    for mode in [
        "source-code",
        "anonymous-constant",
        "entry",
        "load-projection",
        "bss-shadow",
    ] {
        let mut bytes = kernel.to_vec();
        match mode {
            "source-code" => {
                let symbol = file.symbol_by_name("topal_tail_step_component_v1").unwrap();
                let section = file
                    .section_by_index(symbol.section_index().unwrap())
                    .unwrap();
                let offset = section.file_range().unwrap().0 + symbol.address() - section.address();
                bytes[offset as usize] ^= 1;
            }
            "anonymous-constant" => {
                let input = record
                    .tail_root
                    .as_ref()
                    .unwrap()
                    .linked
                    .iter()
                    .find(|s| s.marker.starts_with("topal.tail-inputs.source.") && s.flags == 2)
                    .unwrap();
                let symbol = file
                    .symbol_by_name(&format!("{}.begin", input.marker))
                    .unwrap();
                let section = file
                    .section_by_index(symbol.section_index().unwrap())
                    .unwrap();
                let offset = section.file_range().unwrap().0 + input.address - section.address();
                bytes[offset as usize] ^= 1;
            }
            "entry" => bytes[24..32].copy_from_slice(&(file.entry() + 1).to_le_bytes()),
            "load-projection" | "bss-shadow" => {
                let flags = if mode == "load-projection" { 5 } else { 6 };
                let ph = (0..count)
                    .map(|i| phoff + i * stride)
                    .find(|p| {
                        u32::from_le_bytes(kernel[*p..*p + 4].try_into().unwrap()) == 1
                            && u32::from_le_bytes(kernel[*p + 4..*p + 8].try_into().unwrap())
                                == flags
                    })
                    .unwrap();
                if mode == "load-projection" {
                    let offset = u64::from_le_bytes(kernel[ph + 8..ph + 16].try_into().unwrap());
                    bytes[ph + 8..ph + 16].copy_from_slice(&(offset + 8).to_le_bytes());
                } else {
                    // Preserve SHT_NOBITS, but map nonzero ELF bytes over its beginning.
                    bytes[ph + 8..ph + 16].copy_from_slice(&0u64.to_le_bytes());
                    bytes[ph + 32..ph + 40].copy_from_slice(&8u64.to_le_bytes());
                }
            }
            _ => unreachable!(),
        }
        let mut changed = record.clone();
        // Recompute the outer digest: rejection must come from actual reconstruction.
        changed
            .outputs
            .iter_mut()
            .find(|d| d.identity == crate::SYSTEMS_KERNEL_FILE)
            .unwrap()
            .sha256 = crate::artifact::sha256(&bytes);
        let error = crate::generate_x86_64_linux_boot_image(&bytes, &changed, initial, user)
            .err()
            .unwrap_or_else(|| panic!("accepted changed kernel {mode}"));
        let message = error.to_string();
        assert!(message.starts_with("E-SYSTEMS-TAIL-"), "{mode}: {message}");
        if mode == "load-projection" {
            assert!(message.contains("load bytes disagree"), "{message}");
        }
        if mode == "bss-shadow" {
            assert!(message.contains("shadows zero storage"), "{message}");
        }
        fs::write(work.join(format!("kernel-tamper-{mode}.elf")), bytes).unwrap();
        fs::write(
            work.join(format!("kernel-tamper-{mode}.error.txt")),
            message,
        )
        .unwrap();
    }
}

#[test]
fn tail_publication_keeps_all_selected_kernel_source_libraries() {
    // TOPAL-SYSTEMS-TAIL-BINDING-001: the tail root coexists with every real source slot.
    let original = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");
    let old = "source is \"policy/bootstrap-byte-copy-frame.t\",\n    export is apply,\n    interface is borrowed-byte-region-instance-action-v1";
    assert_eq!(original.matches(old).count(), 1);
    let root=original.replace(old,"source is \"algorithms/clear-prefix.t\",\n    export is clear-prefix,\n    interface is borrowed-byte-region-tail-action-v1").replace("observed = 90","observed = 0");
    let work = work();
    eprintln!("all-library tail artifact fixture: {}", work.display());
    let user = work.join("busybox.elf");
    fs::write(&user, crate::qualified_x86_64_static_user_image_fixture()).unwrap();
    let artifact = crate::publish_systems_source(
        &root,
        &crate::CompileOptions {
            source_name: "arch/x86_64/toolchain-gate.t".into(),
            output: work.join("artifact"),
            emit: crate::Emit::Executable,
            llvm_tools: None,
            library_root: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
            target: crate::TargetSelection {
                target: Some("x86_64-unknown-none".into()),
                board: Some("topal-qemu-pc-q35-10.2".into()),
                ..Default::default()
            },
            optimization: crate::OptimizationRequest::default(),
            standard_library: None,
            initial_user_image: None,
            static_user_image: Some(user),
        },
    )
    .unwrap();
    assert!(artifact.record.tail_root.is_some());
    assert!(
        artifact.record.kernel_library.is_some()
            && artifact.record.time_library.is_some()
            && artifact.record.image_library.is_some()
            && artifact.record.dispatch_library.is_some()
            && artifact.record.process_identity_library.is_some()
            && artifact.record.namespace_library.is_some()
            && artifact.record.inode_mode_library.is_some()
            && artifact.record.ram_page_pool_library.is_some()
            && artifact.record.file_position_library.is_some()
    );
    let kernel = fs::read(&artifact.kernel).unwrap();
    let boot = crate::generate_x86_64_linux_boot_image(
        &kernel,
        &artifact.record,
        &fs::read(&artifact.initial_user_image).unwrap(),
        &fs::read(artifact.static_user_image.as_ref().unwrap()).unwrap(),
    )
    .unwrap();
    fs::write(work.join("kernel.elf"), kernel).unwrap();
    fs::copy(&artifact.debug, work.join(crate::SYSTEMS_DEBUG_FILE)).unwrap();
    fs::write(work.join("bzImage"), boot.bytes).unwrap();
    fs::write(work.join("initramfs.cpio"), boot.boot_resource).unwrap();
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/tail_kernel_inspect.py");
    run(
        &work,
        "all-library-guest-normal",
        Command::new("python3")
            .arg(&script)
            .arg(&work)
            .args(["--mode", "normal", "--retired-bytes"])
            .arg(
                artifact
                    .record
                    .tail_root
                    .as_ref()
                    .unwrap()
                    .bounds
                    .retired_controller
                    .to_string(),
            ),
    )
    .unwrap();
    println!("TAIL_ALL_LIBRARIES_PASS work={}", work.display());
}

#[test]
fn reconstructed_count_source_observes_full_owned_extent_and_isolated_modules() {
    // TOPAL-SYSTEMS-TAIL-BOUNDS-001: native process owner, not a larger kernel grant.
    let root = ROOT
        .replace("algorithms/clear-prefix.t", "algorithms/count-probe.t")
        .replace("export is clear-prefix", "export is inspect");
    let helper =
        "extent is fn (region : KernelByteRegion) -> Nat : Effects ()\n  region byte count\n";
    let body = "pub inspect is fn ((region : KernelByteRegion, remaining : Nat, expected : Nat, state : Nat)) -> Boolean : Decreases remaining\n  remaining\n    <= 0 then (extent region) = expected\n    otherwise inspect(region, remaining - 1, expected, state)\n";
    let tools = LlvmTools::discover(None).unwrap();
    for imported in [false, true] {
        let work = work();
        eprintln!(
            "native count fixture: {} imported={imported}",
            work.display()
        );
        let mut text = "use language (version is v0.1, features is (systems))\n".to_owned();
        let mut modules = Vec::new();
        if imported {
            text.push_str("use library support (version is v0.1)\n");
            text.push_str(&body.replace("extent region", "support extent count region"));
            modules.push(SystemsSourceModuleInput {source:"support/extent.t".into(),
                identity:vec!["support".into(),"extent".into()],
                text:"use language (version is v0.1, features is (systems))\npub count is fn (region : KernelByteRegion) -> Nat : Effects ()\n  region byte count\n".into()});
        } else {
            text.push_str(helper);
            text.push_str(body);
        }
        let input = Source {
            source: "algorithms/count-probe.t".into(),
            source_name: "algorithms/count-probe.t".into(),
            text,
            modules,
        };
        let rebuilt = rebuild(&root, &input, &tools, &work).unwrap();
        fs::write(
            work.join("count-inputs.json"),
            serde_json::to_vec_pretty(&rebuilt.record).unwrap(),
        )
        .unwrap();
        let client = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../linux-kernel/labs/qemu/x86_64/tail_count_client.c");
        let mut cc = Command::new("cc");
        cc.args([
            "-std=gnu11",
            "-O2",
            "-g",
            "-fno-omit-frame-pointer",
            "-fno-strict-aliasing",
            "-fno-pie",
            "-no-pie",
            "-Wall",
            "-Wextra",
            "-mgeneral-regs-only",
            "-mno-red-zone",
            "-fno-builtin",
        ]);
        cc.arg(client);
        for role in ["source", "runtime", "owner", "boundary"] {
            cc.arg(work.join(format!("{role}.o")));
        }
        cc.arg("-o").arg(work.join("count-client"));
        run(&work, "count-cc", &mut cc).unwrap();
        run(
            &work,
            "count-native",
            &mut Command::new(work.join("count-client")),
        )
        .unwrap();
        let output = fs::read_to_string(work.join("count-native.stdout.log")).unwrap();
        assert_eq!(
            output,
            "TAIL_COUNT_PASS cases=56 sizes=7 max_extent=262144\n"
        );
        println!(
            "TAIL_COUNT_RECONSTRUCTION_PASS imported={imported} work={} {output}",
            work.display()
        );
    }
}
