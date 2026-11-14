use super::*;

#[test]
fn frame_artifact_publishes_complete_sources_and_reconstructs_at_boot() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: actual public application
    // service and boot adapter, retaining complete source/native provenance.
    let work = super::tests::work("artifact");
    let root = crate::systems_frame_target::root_tests::root();
    let fixture = work.join("static-elf-abi-fixture.elf");
    fs::write(
        &fixture,
        crate::systems_frame_target::memory_tests::fixture(),
    )
    .unwrap();
    let published = crate::publish_systems_source(
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
            optimization: Default::default(),
            standard_library: None,
            initial_user_image: None,
            static_user_image: Some(fixture),
        },
    )
    .unwrap();
    let record = &published.record;
    let binding = record.frame_root.as_ref().unwrap();
    assert_eq!(binding.root_source, root);
    assert_eq!(binding.objects.len(), 20);
    assert_eq!(binding.other_sources.len(), 10);
    let kernel = fs::read(&published.kernel).unwrap();
    let initial = fs::read(&published.initial_user_image).unwrap();
    let image = fs::read(published.static_user_image.as_ref().unwrap()).unwrap();
    let boot = crate::generate_x86_64_linux_boot_image(&kernel, record, &initial, &image).unwrap();
    fs::write(work.join("bzImage"), &boot.bytes).unwrap();
    fs::write(work.join("initramfs.cpio"), &boot.boot_resource).unwrap();
    fs::write(
        work.join("boot-provenance.json"),
        serde_json::to_vec_pretty(&boot.provenance).unwrap(),
    )
    .unwrap();
    fs::write(work.join("kernel.elf"), &kernel).unwrap();
    fs::copy(
        &published.debug,
        work.join(published.debug.file_name().unwrap()),
    )
    .unwrap();
    let parsed = object::File::parse(kernel.as_slice()).unwrap();
    let bytes = parsed
        .symbol_by_name("topal_x86_frame_root_tokens_v1")
        .unwrap()
        .size();
    fs::write(
        work.join("root-tokens.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"bytes":bytes})).unwrap(),
    )
    .unwrap();
    let observer = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/frame_memory_inspect.py");
    crate::systems_tail_owner::run_tail_native_fixture(
        &work,
        "guest",
        Command::new("python3").arg(observer).arg(&work),
    );
    crate::systems_frame_target::memory_tests::inspect_faults(&work);
}

#[test]
fn frame_artifact_boot_refuses_erased_stale_or_duplicated_provenance() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: exercise the public boot gate,
    // including a changed native image with a correctly recomputed outer hash.
    let work = super::tests::work("artifact-refusals");
    let root = crate::systems_frame_target::root_tests::root();
    let input = crate::systems_frame_target::root_tests::input();
    let tools = LlvmTools::discover(None).unwrap();
    let program = crate::systems_frame_policy_binding::check(&root, &input)
        .unwrap()
        .program;
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
    let sources = [
        ("policy", &program.kernel_library),
        ("time", &program.time_library),
        ("image", &program.image_library),
        ("dispatch", &program.dispatch_library),
        ("process_identity", &program.process_identity_library),
        ("region", &program.region_library),
        ("file_position", &program.file_position_library),
        ("ram_page_pool", &program.ram_page_pool_library),
        ("inode_mode", &program.inode_mode_library),
        ("namespace", &program.namespace_library),
    ]
    .into_iter()
    .map(|(role, r)| {
        (
            role,
            crate::systems_library::load_source_library(r.as_ref().unwrap(), &directory).unwrap(),
        )
    })
    .collect::<BTreeMap<_, _>>();
    let libraries = crate::SystemsKernelLibraries {
        policy: sources.get("policy"),
        time: sources.get("time"),
        image: sources.get("image"),
        dispatch: sources.get("dispatch"),
        process_identity: sources.get("process_identity"),
        region: sources.get("region"),
        file_position: sources.get("file_position"),
        ram_page_pool: sources.get("ram_page_pool"),
        inode_mode: sources.get("inode_mode"),
        namespace: sources.get("namespace"),
    };
    let fixture = crate::systems_frame_target::memory_tests::fixture();
    let published = crate::publish_x86_64_systems_frame_artifact(
        &root,
        &input,
        &tools,
        None,
        Some(&fixture),
        libraries,
        &work.join("artifact"),
    )
    .unwrap();
    let kernel = fs::read(&published.kernel).unwrap();
    let initial = fs::read(&published.initial_user_image).unwrap();
    let record = published.record;
    let mut observations = Vec::new();
    for case in [
        "missing-binding",
        "undercharged-caller",
        "missing-library",
        "wrong-library-object",
        "changed-source",
        "extra-frame-input",
        "duplicate-input",
        "duplicate-output",
        "wrong-frame-output",
        "missing-other-owner",
        "detached-other-source",
        "orphan-tail-input",
        "changed-native-rehashed",
    ] {
        let mut changed = record.clone();
        let mut bytes = kernel.clone();
        match case {
            "missing-binding" => changed.frame_root = None,
            "undercharged-caller" => {
                changed
                    .frame_root
                    .as_mut()
                    .unwrap()
                    .bounds
                    .combined_with_syscall_caller -= 8
            }
            "missing-library" => {
                changed
                    .frame_root
                    .as_mut()
                    .unwrap()
                    .other_sources
                    .remove("library-time");
            }
            "wrong-library-object" => {
                changed
                    .frame_root
                    .as_mut()
                    .unwrap()
                    .other_original_objects
                    .insert("library-region".into(), "0".repeat(64));
            }
            "changed-source" => changed
                .frame_root
                .as_mut()
                .unwrap()
                .library
                .text
                .push_str("\n# changed snapshot\n"),
            "extra-frame-input" => changed.inputs.push(crate::DigestEntry {
                identity: "frame-unselected".into(),
                sha256: "0".repeat(64),
            }),
            "duplicate-input" => changed.inputs.push(
                changed
                    .inputs
                    .iter()
                    .find(|d| d.identity == "frame-root-source")
                    .unwrap()
                    .clone(),
            ),
            "duplicate-output" => changed.outputs.push(
                changed
                    .outputs
                    .iter()
                    .find(|d| d.identity == "frame-root.binding.json")
                    .unwrap()
                    .clone(),
            ),
            "wrong-frame-output" => {
                changed
                    .outputs
                    .iter_mut()
                    .find(|d| d.identity == "frame-provider.o")
                    .unwrap()
                    .sha256 = "0".repeat(64)
            }
            "missing-other-owner" => changed.region_runtime = None,
            "detached-other-source" => {
                let library = changed.region_library.as_mut().unwrap();
                library.source_sha256 = "0".repeat(64);
                let identity = format!("kernel-region-source:{}", library.source);
                changed
                    .inputs
                    .iter_mut()
                    .find(|d| d.identity == identity)
                    .unwrap()
                    .sha256 = library.source_sha256.clone();
                changed
                    .outputs
                    .iter_mut()
                    .find(|d| d.identity == "region-library.t")
                    .unwrap()
                    .sha256 = library.source_sha256.clone();
            }
            "orphan-tail-input" => changed.inputs.push(crate::DigestEntry {
                identity: "tail-root-source".into(),
                sha256: "0".repeat(64),
            }),
            "changed-native-rehashed" => {
                use object::ObjectSegment;
                let parsed = object::File::parse(kernel.as_slice()).unwrap();
                let placement = record
                    .frame_root
                    .as_ref()
                    .unwrap()
                    .linked
                    .iter()
                    .find(|p| p.marker.contains("frame-controller") && p.flags & 4 != 0)
                    .unwrap();
                let segment = parsed
                    .segments()
                    .find(|s| {
                        s.address() <= placement.address
                            && placement.address < s.address() + s.size()
                    })
                    .unwrap();
                let at = segment.file_range().0 + placement.address - segment.address();
                bytes[at as usize] ^= 1;
                changed
                    .outputs
                    .iter_mut()
                    .find(|d| d.identity == crate::SYSTEMS_KERNEL_FILE)
                    .unwrap()
                    .sha256 = crate::artifact::sha256(&bytes);
            }
            _ => unreachable!(),
        }
        let destination = work.join(case);
        let error = crate::publish_x86_64_linux_boot_image(
            &bytes,
            &changed,
            &initial,
            &fixture,
            &destination,
        )
        .unwrap_err()
        .to_string();
        if case == "detached-other-source" {
            assert!(
                error.contains("other source/artifact snapshot association"),
                "{error}"
            );
        }
        assert!(!destination.exists(), "published refused case {case}");
        observations.push(serde_json::json!({"case":case,"error":error}));
    }
    let mut encoded = serde_json::to_value(&record).unwrap();
    encoded["frame_root"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<crate::SystemsArtifactProvenance>(encoded).is_err());
    // All ten independently typed legacy records must name exactly the source
    // and module identities consumed by fresh reconstruction.
    for slot in [
        "kernel_library",
        "time_library",
        "image_library",
        "dispatch_library",
        "process_identity_library",
        "region_library",
        "file_position_library",
        "ram_page_pool_library",
        "inode_mode_library",
        "namespace_library",
    ] {
        for field in ["source_sha256", "source_name", "module_snapshot_sha256"] {
            let mut encoded = serde_json::to_value(&record).unwrap();
            encoded[slot][field] = serde_json::Value::String("detached".into());
            let changed =
                serde_json::from_value::<crate::SystemsArtifactProvenance>(encoded).unwrap();
            assert!(
                super::sources::validate(record.frame_root.as_ref().unwrap(), &changed).is_err(),
                "{slot}:{field}"
            );
        }
    }
    // An explicit source-bearing API still requires every selected input and
    // refuses an unrelated frame source before publishing any destination.
    for (label, source, selected) in [
        (
            "missing-source-slot",
            input.clone(),
            crate::SystemsKernelLibraries {
                time: None,
                ..libraries
            },
        ),
        (
            "wrong-frame-slot",
            crate::SystemsSourceLibraryInput {
                source: "memory/other.t".into(),
                ..input.clone()
            },
            libraries,
        ),
    ] {
        let destination = work.join(label);
        let error = crate::publish_x86_64_systems_frame_artifact(
            &root,
            &source,
            &tools,
            None,
            Some(&fixture),
            selected,
            &destination,
        )
        .unwrap_err()
        .to_string();
        assert!(!destination.exists());
        observations.push(serde_json::json!({"case":label,"error":error}));
    }
    fs::write(
        work.join("refusals.json"),
        serde_json::to_vec_pretty(&observations).unwrap(),
    )
    .unwrap();
}
