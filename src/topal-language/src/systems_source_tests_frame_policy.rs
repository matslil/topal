const FRAME_POLICY_DECLARATION: &str = "  frame-allocator-library is lang systems source-library (\n    source is \"memory/frame-metadata.t\",\n    export is frame-policy,\n    interface is physical-frame-policy-tail-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";

fn frame_policy_root() -> String {
    include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t").replacen(
        "lang systems artifact (\n",
        &format!("lang systems artifact (\n{FRAME_POLICY_DECLARATION}"),
        1,
    )
}

#[test]
fn frame_policy_slot_retains_its_obligation_without_changing_legacy_operations() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001.
    let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let plain = analyze_systems_for_compiler(
        include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t"),
        &target,
    )
    .unwrap();
    assert!(plain.frame_allocator_library.is_none());
    let mut selected = analyze_systems_for_compiler(&frame_policy_root(), &target).unwrap();
    let requirement = selected.frame_allocator_library.take().unwrap();
    assert_eq!(
        requirement.interface,
        CompilerSystemsSourceLibraryInterface::PhysicalFramePolicyTailV1
    );
    assert_eq!(requirement.source, "memory/frame-metadata.t");
    assert_eq!(requirement.export, "frame-policy");
    assert_eq!(
        requirement.interface.as_str(),
        "physical-frame-policy-tail-v1"
    );
    assert_eq!(requirement.runtime_limit_bytes, Some(65536));
    assert_eq!(requirement.stack_limit_bytes, 4096);
    assert!(requirement.state_limit_bytes.is_none());
    assert_eq!(selected, plain);
}

#[test]
fn frame_policy_slot_rejects_wrong_interface_duplicate_and_persistent_reserve() {
    let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let root = frame_policy_root();
    for declaration in [
        FRAME_POLICY_DECLARATION.replace(
            "physical-frame-policy-tail-v1",
            "borrowed-byte-region-tail-action-v1",
        ),
        FRAME_POLICY_DECLARATION.replace("frame-allocator-library is", "namespace-library is"),
        FRAME_POLICY_DECLARATION.replace("stack-limit-bytes is 4096", "stack-limit-bytes is 4095"),
        FRAME_POLICY_DECLARATION.replace("stack-limit-bytes is 4096", "stack-limit-bytes is 4097"),
        FRAME_POLICY_DECLARATION.replace(
            "runtime-limit-bytes is 65536",
            "runtime-limit-bytes is 65535",
        ),
        FRAME_POLICY_DECLARATION.replace(
            "runtime-limit-bytes is 65536",
            "runtime-limit-bytes is 65537",
        ),
        FRAME_POLICY_DECLARATION.replace(
            "runtime-limit-bytes is 65536",
            "state-limit-bytes is 65536, runtime-limit-bytes is 65536",
        ),
        FRAME_POLICY_DECLARATION.replace("memory/frame-metadata.t", "../frame-metadata.t"),
        format!("{FRAME_POLICY_DECLARATION}{FRAME_POLICY_DECLARATION}"),
    ] {
        assert!(
            analyze_systems_for_compiler(
                &root.replace(FRAME_POLICY_DECLARATION, &declaration),
                &target
            )
            .is_err(),
            "{declaration}"
        );
    }
}

#[test]
fn frame_policy_slot_requires_the_actual_described_memory_allocator_creation() {
    let mut root = frame_policy_root();
    let start = root.find("boot is fn").unwrap();
    let end = root.find("\ndebug-break-handler is fn").unwrap();
    root.replace_range(start..end,"boot is fn (context : BootstrapContext) -> BootstrapDisposition\n  context fatal \"no allocator\"\n");
    let error = analyze_systems_for_compiler(
        &root,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap_err();
    // The existing sealed boot protocol refuses this before the import check.
    assert_eq!(error.code, "E-SYSTEMS-BOOT-MEMORY");
}

#[test]
fn frame_policy_model_does_not_erase_or_rebind_the_context_obligation() {
    let program = analyze_systems_for_compiler(
        &frame_policy_root(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    validate_systems_program(&program).unwrap();
    for fault in 0..5 {
        let mut wrong = program.clone();
        match fault {
            0 => {
                wrong.frame_allocator_library.as_mut().unwrap().interface =
                    CompilerSystemsSourceLibraryInterface::BorrowedByteRegionTailActionV1
            }
            1 => {
                wrong.bootstrap.handler.operations.retain(|op| {
                    !matches!(op, CompilerSystemsOperation::CreateFrameAllocator { .. })
                })
            }
            2 => wrong
                .bootstrap
                .handler
                .operations
                .retain(|op| !matches!(op, CompilerSystemsOperation::DescribeBootMemory { .. })),
            _ => {
                let duplicate = wrong
                    .bootstrap
                    .handler
                    .operations
                    .iter()
                    .find(|op| {
                        if fault == 3 {
                            matches!(op, CompilerSystemsOperation::CreateFrameAllocator { .. })
                        } else {
                            matches!(op, CompilerSystemsOperation::DescribeBootMemory { .. })
                        }
                    })
                    .unwrap()
                    .clone();
                wrong.bootstrap.handler.operations.push(duplicate);
            }
        }
        assert_eq!(
            validate_systems_program(&wrong).unwrap_err().code,
            "E-SYSTEMS-FRAME-POLICY",
            "fault {fault}"
        );
    }
}
