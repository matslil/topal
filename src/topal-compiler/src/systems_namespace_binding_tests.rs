// Included in boot-adapter tests to exercise independent final admission.
fn namespace_test_root() -> String {
    let declaration = "  namespace-library is lang systems source-library (\n    source is \"policy/namespace-action.t\",\n    export is apply,\n    interface is borrowed-byte-region-instance-action-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";
    // The component certificate is deliberately V5/version 1. Each V6 test
    // selects its separate owning-adapter certificate explicitly below.
    let legacy = SOURCE.replace("static-linux-dispatch-v6", "static-linux-dispatch-v5");
    if legacy.contains("  namespace-library is") {
        legacy
    } else {
        legacy.replace(
            "  inode-mode-library is",
            &format!("{declaration}  inode-mode-library is"),
        )
    }
}

#[test]
fn v6_namespace_binding_requires_complete_owning_retirement_and_entry_costs() {
    // TOPAL-SYSTEMS-NAMESPACE-ABI-001/-ACTION-001. Artifact construction and
    // independent admission; runtime/physical/Linux qualification is separate.
    let source =
        namespace_test_root().replace("static-linux-dispatch-v5", "static-linux-dispatch-v6");
    let parent = temporary_parent();
    fs::create_dir_all(&parent).unwrap();
    fs::write(parent.join("namespace-root.t"), &source).unwrap();
    let (kernel, initial, user, artifact) = linked_kernel_source(&parent, &source);
    generate_x86_64_linux_boot_image(&kernel, &artifact, &initial, &user).unwrap();
    let runtime = artifact.namespace_runtime.as_ref().unwrap();
    assert_eq!(runtime.schema, "topal.namespace-owner-binding.x86_64/2");
    assert_eq!(
        runtime.invocation.schema,
        "topal.namespace-region-invocation.x86_64/2"
    );
    let retirement = runtime.admission_retirement.as_ref().unwrap();
    assert_eq!(
        (
            retirement.kernel_stack_context_offset,
            retirement.minimum_stack_pointer,
            retirement.maximum_stack_pointer,
            retirement.stack_alignment,
            retirement.retired_stack_bytes,
            retirement.owning_adapter_frame_bytes
        ),
        (8, 0x108000, 0x180000, 8, 4096, 8)
    );
    let halt = runtime
        .invocation
        .code
        .iter()
        .find(|code| code.symbol == crate::systems_provider_object::namespace_region::HALT)
        .unwrap();
    assert!(
        retirement.halt_instruction_offset > 0
            && retirement.halt_instruction_offset + 4 == halt.bytes
    );
    let entries = artifact.volatile_files.namespace_entries.as_ref().unwrap();
    let inode = artifact.inode_mode_runtime.as_ref().unwrap();
    assert_eq!(inode.schema, "topal.inode-region-invocation.x86_64/2");
    let admission = inode.namespace_admission.as_ref().unwrap();
    assert_eq!(
        (
            admission.metadata_offset,
            admission.metadata_bytes,
            admission.maximum_link_count,
            admission.validation_stack_bytes
        ),
        (0x43000, 4096, 4, 72)
    );
    assert_eq!(inode.combined_stack_bytes, 1544);
    assert_eq!(inode.code.len(), 5);
    for case in 0..7 {
        let mut wrong = artifact.clone();
        let inode = wrong.inode_mode_runtime.as_mut().unwrap();
        match case {
            0 => inode.schema = "topal.inode-region-invocation.x86_64/1".into(),
            1 => inode.namespace_admission = None,
            2 => {
                inode
                    .namespace_admission
                    .as_mut()
                    .unwrap()
                    .maximum_link_count = 1
            }
            3 => inode.namespace_admission.as_mut().unwrap().metadata_offset += 8,
            4 => inode.namespace_admission.as_mut().unwrap().metadata_bytes -= 8,
            5 => {
                inode
                    .namespace_admission
                    .as_mut()
                    .unwrap()
                    .validation_stack_bytes -= 8
            }
            _ => {
                inode.code.pop();
            }
        }
        assert!(
            generate_x86_64_linux_boot_image(&kernel, &wrong, &initial, &user).is_err(),
            "V6 inode admission mutation {case}"
        );
    }
    let mut null_inode = serde_json::to_value(&artifact).unwrap();
    null_inode["inode_mode_runtime"]["namespace_admission"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<SystemsArtifactProvenance>(null_inode).is_err());
    assert_eq!(
        (
            entries.metadata_lookup_stack_bytes,
            entries.admission_retired_stack_bytes,
            entries.namespace_caller_stack_bytes
        ),
        (104, 4096, 256)
    );
    assert_eq!(
        artifact
            .namespace_library
            .as_ref()
            .unwrap()
            .maximum_stack_bytes
            + entries.namespace_caller_stack_bytes,
        4080
    );
    for case in 0..16 {
        let mut wrong = artifact.clone();
        let owner = wrong.namespace_runtime.as_mut().unwrap();
        match case {
            0 => owner.schema = "topal.namespace-owner-binding.x86_64/1".into(),
            1 => owner.invocation.schema = "topal.namespace-region-invocation.x86_64/1".into(),
            2 => owner.admission_retirement = None,
            3 => {
                owner
                    .admission_retirement
                    .as_mut()
                    .unwrap()
                    .kernel_stack_context_offset += 8
            }
            4 => {
                owner
                    .admission_retirement
                    .as_mut()
                    .unwrap()
                    .minimum_stack_pointer -= 8
            }
            5 => {
                owner
                    .admission_retirement
                    .as_mut()
                    .unwrap()
                    .maximum_stack_pointer += 8
            }
            6 => owner.admission_retirement.as_mut().unwrap().stack_alignment = 1,
            7 => {
                owner
                    .admission_retirement
                    .as_mut()
                    .unwrap()
                    .retired_stack_bytes -= 8
            }
            8 => {
                owner
                    .admission_retirement
                    .as_mut()
                    .unwrap()
                    .owning_adapter_frame_bytes = 0
            }
            9 => {
                owner
                    .admission_retirement
                    .as_mut()
                    .unwrap()
                    .halt_instruction_offset += 1
            }
            10 => wrong.volatile_files.namespace_entries = None,
            11 => {
                wrong
                    .volatile_files
                    .namespace_entries
                    .as_mut()
                    .unwrap()
                    .metadata_lookup_stack_bytes = 96
            }
            12 => {
                wrong
                    .volatile_files
                    .namespace_entries
                    .as_mut()
                    .unwrap()
                    .admission_retired_stack_bytes -= 8
            }
            13 => {
                wrong
                    .volatile_files
                    .namespace_entries
                    .as_mut()
                    .unwrap()
                    .namespace_caller_stack_bytes -= 8
            }
            14 => wrong.kernel_library = None,
            _ => wrong.kernel_library.as_mut().unwrap().interface = "boot-open-decision-v1".into(),
        }
        assert!(
            generate_x86_64_linux_boot_image(&kernel, &wrong, &initial, &user).is_err(),
            "V6 binding mutation {case}"
        );
    }
    let mut null = serde_json::to_value(&artifact).unwrap();
    null["namespace_runtime"]["admission_retirement"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<SystemsArtifactProvenance>(null).is_err());
    publish_x86_64_linux_boot_image(&kernel, &artifact, &initial, &user, &parent.join("boot"))
        .unwrap();
    eprintln!(
        "NAMESPACE_V6_BINDING_PASS caller=256 combined=4080 lookup=104 admission-retired=4096 mutations=16 work={}",
        parent.display()
    );
}

#[test]
fn namespace_binding_requires_exact_owner_source_recovery_and_complete_stack() {
    // TOPAL-SYSTEMS-NAMESPACE-ACTION-001. Declaring a controller does not select
    // the later V6 Linux handlers or prove its physical ring0 loan/cleanup.
    let source = namespace_test_root();
    let parent = temporary_parent();
    fs::create_dir_all(&parent).unwrap();
    fs::write(parent.join("namespace-root.t"), &source).unwrap();
    let (kernel, initial, user, artifact) = linked_kernel_source(&parent, &source);
    generate_x86_64_linux_boot_image(&kernel, &artifact, &initial, &user).unwrap();
    let runtime = artifact.namespace_runtime.as_ref().unwrap();
    let binding = &runtime.invocation;
    let library = artifact.namespace_library.as_ref().unwrap();
    assert_eq!(runtime.schema, "topal.namespace-owner-binding.x86_64/1");
    assert!(runtime.admission_retirement.is_none());
    assert_eq!(binding.schema, "topal.namespace-region-invocation.x86_64/1");
    assert_eq!(binding.instance.as_deref(), Some("namespace-metadata"));
    assert_eq!(binding.scalar_count, Some(2));
    assert_eq!(
        (
            binding.maximum_source_stack_bytes,
            binding.live_frame_bytes,
            binding.caller_alignment_bytes,
            binding.combined_stack_bytes
        ),
        (3824, 184, 64, 4072)
    );
    assert_eq!(library.allocation.numeric.maximum_bytes, 57536);
    assert_eq!(library.functions.len(), 89);
    assert_eq!(binding.source_code.len(), library.functions.len());
    assert_eq!(binding.code.len(), 7);
    assert_eq!(
        (
            runtime.metadata_offset,
            runtime.metadata_bytes,
            runtime.staged_input_offset,
            runtime.staged_input_bytes,
            runtime.comparison_bytes,
            runtime.recovery_bytes,
            runtime.helper_stack_bytes
        ),
        (0x43000, 4096, 3072, 512, 4096, 32, 8)
    );
    assert_eq!(binding.storage.len(), 5);
    assert_eq!(runtime.private_bss_minimum_address, 0x800000);
    assert!(binding.storage.iter().all(|s| s.address >= 0x800000));
    for other in [
        &artifact.region_runtime.as_ref().unwrap().storage,
        &artifact.file_position_runtime.as_ref().unwrap().storage,
        &artifact.ram_page_pool_runtime.as_ref().unwrap().storage,
        &artifact.inode_mode_runtime.as_ref().unwrap().storage,
    ] {
        assert!(binding.storage.iter().all(|a| {
            other
                .iter()
                .all(|b| a.address + a.bytes <= b.address || b.address + b.bytes <= a.address)
        }));
    }
    for case in 0..26 {
        let mut wrong = artifact.clone();
        match case {
            0 => wrong.namespace_library = None,
            1 => wrong.namespace_runtime = None,
            2 => wrong.namespace_runtime.as_mut().unwrap().metadata_offset += 8,
            3 => wrong.namespace_runtime.as_mut().unwrap().metadata_bytes -= 1,
            4 => {
                wrong
                    .namespace_runtime
                    .as_mut()
                    .unwrap()
                    .staged_input_offset -= 8
            }
            5 => wrong.namespace_runtime.as_mut().unwrap().staged_input_bytes -= 1,
            6 => wrong.namespace_runtime.as_mut().unwrap().comparison_bytes -= 8,
            7 => wrong.namespace_runtime.as_mut().unwrap().recovery_bytes -= 8,
            8 => wrong.namespace_runtime.as_mut().unwrap().helper_stack_bytes = 0,
            9 => {
                wrong
                    .namespace_runtime
                    .as_mut()
                    .unwrap()
                    .invocation
                    .combined_stack_bytes -= 8
            }
            10 => {
                wrong
                    .namespace_runtime
                    .as_mut()
                    .unwrap()
                    .invocation
                    .live_frame_bytes -= 8
            }
            11 => {
                wrong
                    .namespace_runtime
                    .as_mut()
                    .unwrap()
                    .invocation
                    .instance = Some("inode-metadata".into())
            }
            12 => {
                wrong
                    .namespace_runtime
                    .as_mut()
                    .unwrap()
                    .invocation
                    .scalar_count = Some(1)
            }
            13 => {
                wrong
                    .namespace_library
                    .as_mut()
                    .unwrap()
                    .allocation
                    .numeric
                    .maximum_bytes -= 8
            }
            14 => {
                wrong
                    .namespace_library
                    .as_mut()
                    .unwrap()
                    .maximum_stack_bytes -= 8
            }
            15 => wrong.namespace_library.as_mut().unwrap().instance = Some("file-position".into()),
            16 => wrong.namespace_library.as_mut().unwrap().scalar_count = Some(1),
            17 => {
                wrong.namespace_runtime.as_mut().unwrap().invocation.storage[3].address =
                    artifact.file_position_runtime.as_ref().unwrap().storage[1].address
            }
            18 => wrong.namespace_runtime.as_mut().unwrap().invocation.storage[4].bytes = 24,
            19 => {
                wrong.namespace_runtime.as_mut().unwrap().invocation.code[0].sha256 = "0".repeat(64)
            }
            20 => {
                wrong
                    .namespace_runtime
                    .as_mut()
                    .unwrap()
                    .invocation
                    .source_code[0]
                    .sha256 = "0".repeat(64)
            }
            21 => {
                wrong
                    .namespace_runtime
                    .as_mut()
                    .unwrap()
                    .invocation
                    .source_data[0]
                    .sha256 = "0".repeat(64)
            }
            22 => {
                let input = wrong
                    .inputs
                    .iter()
                    .find(|i| i.identity.starts_with("kernel-namespace-source:"))
                    .unwrap()
                    .clone();
                wrong.inputs.push(input);
            }
            23 => wrong
                .outputs
                .retain(|i| i.identity != "namespace-library.modules.json"),
            24 => wrong.inode_mode_runtime = None,
            25 => {
                wrong
                    .namespace_runtime
                    .as_mut()
                    .unwrap()
                    .private_bss_minimum_address = 0x400000
            }
            _ => unreachable!(),
        }
        assert!(
            generate_x86_64_linux_boot_image(&kernel, &wrong, &initial, &user).is_err(),
            "namespace mutation {case}"
        );
    }
    use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
    let file = object::File::parse(kernel.as_slice()).unwrap();
    for bound in binding.code.iter().chain(&binding.source_code) {
        let symbol = file.symbol_by_name(&bound.symbol).unwrap();
        let section = file
            .section_by_index(symbol.section_index().unwrap())
            .unwrap();
        let (offset, _) = section.file_range().unwrap();
        let index = (offset + symbol.address() - section.address()) as usize;
        let mut changed = kernel.clone();
        changed[index] ^= 1;
        let mut evidence = artifact.clone();
        evidence
            .outputs
            .iter_mut()
            .find(|o| o.identity == SYSTEMS_KERNEL_FILE)
            .unwrap()
            .sha256 = sha256(&changed);
        assert!(
            generate_x86_64_linux_boot_image(&changed, &evidence, &initial, &user).is_err(),
            "namespace changed code {}",
            bound.symbol
        );
    }
    for key in ["namespace_library", "namespace_runtime"] {
        let mut json = serde_json::to_value(&artifact).unwrap();
        json[key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<SystemsArtifactProvenance>(json).is_err(),
            "explicit null {key}"
        );
    }
    publish_x86_64_linux_boot_image(&kernel, &artifact, &initial, &user, &parent.join("boot"))
        .unwrap();
    eprintln!(
        "NAMESPACE_BINDING_PASS stack=4072 source=3824 helper=8 native=7 source-native=89 private-storages=5 mutations=26 code-flips=96 work={}",
        parent.display()
    );
    // Retain the complete artifact/root/boot fixture for the distinct ring0
    // GDB controller tests; this test does not claim those observations.
}

#[test]
fn namespace_fault_sources_publish_with_complete_bounded_controller_bindings() {
    // TOPAL-SYSTEMS-NAMESPACE-ACTION-001. Actual fault source objects, not an
    // injected return value or a modified native controller. Physical fatal
    // behavior is a separate QEMU observation of these retained fixtures.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
    for (v6, wrong_association) in [(false, false), (false, true), (true, false), (true, true)] {
        let source = if v6 {
            namespace_test_root().replace("static-linux-dispatch-v5", "static-linux-dispatch-v6")
        } else {
            namespace_test_root()
        };
        let program = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let parent = temporary_parent();
        let private_root = parent.join("libraries");
        let mut input = crate::systems_library::load_source_library(
            program.namespace_library.as_ref().unwrap(),
            &root,
        )
        .unwrap();
        input.text = if wrong_association {
            "use language (version is v0.1, features is (systems))\nuse library support (version is v0.1)\ndamage is fn (region : KernelByteRegion) -> Boolean\n  region unsigned store (offset-bytes is 64, byte-count is 8, byte-order is little-endian, value is 999)\npub apply is fn ((region : KernelByteRegion, operation : Nat, seconds : Nat)) -> Boolean\n  changed is support namespace-metadata apply (region, operation, seconds)\n  changed\n    true then damage region\n    false then false\n".into()
        } else {
            input.text.replace(
                "  support namespace-metadata apply (region, operation, seconds)",
                "  _ is support namespace-metadata apply (region, operation, seconds)\n  false",
            )
        };
        let path = private_root.join(&input.source);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &input.text).unwrap();
        input.source_name = path.to_string_lossy().into();
        for module in &input.modules {
            let path = private_root.join(&module.source);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, &module.text).unwrap();
        }
        fs::write(parent.join("namespace-root.t"), &source).unwrap();
        let (kernel, initial, user, artifact) =
            linked_kernel_source_with_namespace(&parent, &source, Some(&input));
        let binding = &artifact.namespace_runtime.as_ref().unwrap().invocation;
        assert_eq!(
            binding.maximum_source_stack_bytes,
            if wrong_association { 3840 } else { 3824 }
        );
        assert_eq!(
            binding.combined_stack_bytes,
            if wrong_association { 4088 } else { 4072 }
        );
        let runtime = artifact.namespace_runtime.as_ref().unwrap();
        assert_eq!(runtime.admission_retirement.is_some(), v6);
        if v6 {
            let entry = artifact.volatile_files.namespace_entries.as_ref().unwrap();
            assert_eq!(
                binding.maximum_source_stack_bytes + entry.namespace_caller_stack_bytes,
                if wrong_association { 4096 } else { 4080 }
            );
            assert_eq!(runtime.schema, "topal.namespace-owner-binding.x86_64/2");
        } else {
            assert_eq!(runtime.schema, "topal.namespace-owner-binding.x86_64/1");
        }
        assert_eq!(
            artifact.namespace_library.as_ref().unwrap().source_sha256,
            sha256(input.text.as_bytes())
        );
        assert_eq!(
            artifact
                .namespace_library
                .as_ref()
                .unwrap()
                .allocation
                .numeric
                .maximum_bytes,
            57536
        );
        generate_x86_64_linux_boot_image(&kernel, &artifact, &initial, &user).unwrap();
        publish_x86_64_linux_boot_image(&kernel, &artifact, &initial, &user, &parent.join("boot"))
            .unwrap();
        eprintln!(
            "NAMESPACE_FAULT_BINDING_PASS v6={v6} wrong_association={wrong_association} combined={} work={}",
            binding.combined_stack_bytes,
            parent.display()
        );
    }
}
