#[test]
fn requires_affine_exhaustive_boot_memory_refinement() {
    // TOPAL-SYSTEMS-BOOT-MEMORY-001, TOPAL-SYSTEMS-AUTHORITY-001.
    for (source, code) in [
        (
            SOURCE.replace(
                "context boot describe memory",
                "context console write \"skipped refinement\"",
            ),
            "E-SYSTEMS-BOOT-MEMORY",
        ),
        (
            SOURCE.replace(
                "restored console write \"TOPAL_KERNEL_MEMORY_DESCRIBED\"",
                "context console write \"TOPAL_KERNEL_MEMORY_DESCRIBED\"",
            ),
            "E-SYSTEMS-OPERATION",
        ),
        (
            SOURCE.replace(
                "Error failure then failure fatal \"boot memory description failed\"",
                "Error failure then failure resume",
            ),
            "E-SYSTEMS-DISPOSITION",
        ),
    ] {
        assert_eq!(
            analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            code
        );
    }
}

#[test]
fn requires_affine_physical_frame_allocation_and_release() {
    // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-AUTHORITY-001.
    for (source, code, expected) in [
        (
            SOURCE.replace(
                "described memory create frame allocator",
                "context memory create frame allocator",
            ),
            "E-SYSTEMS-FRAMES",
            "live memory-described context",
        ),
        (
            SOURCE.replacen("frame-count is 1", "frame-count is 4", 1),
            "E-SYSTEMS-FRAMES",
            "only one, two, three, five, 34, or exactly 503 frames",
        ),
        (
            SOURCE.replace("frame-count is 503", "frame-count is 502"),
            "E-SYSTEMS-FRAMES",
            "only one, two, three, five, 34, or exactly 503 frames",
        ),
        (
            SOURCE.replace("frame-count is 503", "frame-count is 504"),
            "E-SYSTEMS-FRAMES",
            "only one, two, three, five, 34, or exactly 503 frames",
        ),
        (
            SOURCE.replacen("frame-count is 1", "frame-count is 3", 1),
            "E-SYSTEMS-FRAMES",
            "initial kernel-mapping probe requires exactly one frame",
        ),
        (
            SOURCE.replace("alignment-frames is 1", "alignment-frames is 2"),
            "E-SYSTEMS-FRAMES",
            "unit alignment",
        ),
        (
            SOURCE.replace(
                "memory frames release frames",
                "memory frames release other",
            ),
            "E-SYSTEMS-FRAMES",
            "affine extent",
        ),
        (
            SOURCE.replace(
                "failure fatal \"frame allocator creation failed\"",
                "failure resume",
            ),
            "E-SYSTEMS-DISPOSITION",
            "not a disposition",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert_eq!(error.code, code);
        assert!(error.message.contains(expected), "{}", error.message);
    }
}

#[test]
fn requires_exact_static_elf_mapping_resource_ownership() {
    // TOPAL-SYSTEMS-STATIC-ELF-MAPPING-RESOURCES-001.
    let metadata_request = "frame-count is 1,\n                                                                        alignment-frames is 1\n                                                                      )\n                                                                        Ok static-elf-leaf-table-frame";
    let metadata_request_two = "frame-count is 2,\n                                                                        alignment-frames is 1\n                                                                      )\n                                                                        Ok static-elf-leaf-table-frame";
    for (source, code, expected) in [
        (
            SOURCE.replace(metadata_request, metadata_request_two),
            "E-SYSTEMS-TRANSLATION-EDIT",
            "exactly one frame",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_OK",
                "TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_BAD",
            ),
            "E-SYSTEMS-TRANSLATION-EDIT",
            "exact marker",
        ),
        (
            SOURCE.replace(
                "unmapped frames release returned-static-elf-metadata",
                "unmapped frames release other-static-elf-metadata",
            ),
            "E-SYSTEMS-FRAMES",
            "affine extent",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert_eq!(error.code, code);
        assert!(error.message.contains(expected), "{}", error.message);
    }
}

#[test]
fn requires_exact_static_elf_initial_leaf_construction() {
    // TOPAL-SYSTEMS-STATIC-ELF-LEAVES-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "layout is bounded-static-elf-initial-rw-nx",
                    "layout is other-static-elf-layout",
                ),
                "exact payload, metadata, and initial RW/NX layout",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_OK",
                    "TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replacen("resource is metadata", "resource is payload", 1),
                "exact construction and role",
            ),
            (
                SOURCE.replace(
                    "owner is detached-static-elf-construction,\n                                                                                resource is payload",
                    "owner is other-static-elf-construction,\n                                                                                resource is payload",
                ),
                "exact construction and role",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-TRANSLATION-EDIT");
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_complete_static_elf_payload_zeroing() {
    // TOPAL-SYSTEMS-STATIC-ELF-ZERO-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is static-elf-construction,\n                                                                                extent is complete",
                    "owner is other-static-elf-construction,\n                                                                                extent is complete",
                ),
                "exact live construction and complete extent",
            ),
            (
                SOURCE.replace("extent is complete", "extent is partial"),
                "exact live construction and complete extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replacen(
                    "owner is zeroed-static-elf-construction",
                    "owner is static-elf-construction",
                    1,
                ),
                "exact zeroed construction",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-TRANSLATION-EDIT");
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_payload_copy() {
    // TOPAL-SYSTEMS-STATIC-ELF-COPY-001.
    for (source, expected) in [
            (
                SOURCE.replacen(
                    "owner is zeroed-static-elf-construction",
                    "owner is static-elf-construction",
                    1,
                ),
                "exact zeroed construction",
            ),
            (
                SOURCE.replace(
                    "source is retained-static-user-elf",
                    "source is other-resource",
                ),
                "retained static ELF",
            ),
            (
                SOURCE.replace(
                    "plan is bounded-static-elf-four-loads",
                    "plan is other-plan",
                ),
                "four-load plan",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replacen(
                    "owner is detached-static-elf-construction,\n                                                                                resource is metadata",
                    "owner is populated-static-elf-construction,\n                                                                                resource is metadata",
                    1,
                ),
                "exact construction and role",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-TRANSLATION-EDIT");
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_final_permissions() {
    // TOPAL-SYSTEMS-STATIC-ELF-PERMISSIONS-001.
    for (source, expected) in [
            (
                SOURCE.replacen(
                    "owner is populated-static-elf-construction",
                    "owner is zeroed-static-elf-construction",
                    1,
                ),
                "exact populated construction",
            ),
            (
                SOURCE.replace("plan is bounded-static-elf-wx-relro", "plan is other-plan"),
                "W^X/RELRO plan",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_OK",
                    "TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replacen(
                    "owner is detached-static-elf-construction,\n                                                                                resource is metadata",
                    "owner is populated-static-elf-construction,\n                                                                                resource is metadata",
                    1,
                ),
                "exact construction and role",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-TRANSLATION-EDIT");
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_hierarchy_extent() {
    // TOPAL-SYSTEMS-STATIC-ELF-HIERARCHY-EXTENT-001.
    for (source, code, expected) in [
            (
                SOURCE.replacen(
                    "owner is sealed-static-elf-construction,\n                                                                                frame-count is 5",
                    "owner is populated-static-elf-construction,\n                                                                                frame-count is 5",
                    1,
                ),
                "E-SYSTEMS-TRANSLATION-EDIT",
                "exact sealed construction",
            ),
            (
                SOURCE.replacen("frame-count is 5", "frame-count is 34", 1),
                "E-SYSTEMS-FRAMES",
                "frame-allocation slice",
            ),
            (
                SOURCE.replacen(
                    "owner is sealed-static-elf-construction,\n                                                                                frame-count is 5,\n                                                                                alignment-frames is 1",
                    "owner is sealed-static-elf-construction,\n                                                                                frame-count is 5,\n                                                                                alignment-frames is 2",
                    1,
                ),
                "E-SYSTEMS-TRANSLATION-EDIT",
                "five unit-aligned frames",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK",
                    "TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_BAD",
                ),
                "E-SYSTEMS-TRANSLATION-EDIT",
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "unmapped console write \"TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK\"\n                                                                              attached-static-elf-construction is unmapped static image construction attach address space",
                    "unmapped frames release static-elf-hierarchy-frames\n                                                                              attached-static-elf-construction is unmapped static image construction attach address space",
                ),
                "E-SYSTEMS-OPERATION",
                "not admitted",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(
                error.code, code,
                "{expected}: {}",
                error.message
            );
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_address_space_attachment() {
    // TOPAL-SYSTEMS-STATIC-ELF-ADDRESS-SPACE-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "layout is bounded-static-elf-inactive-address-space",
                    "layout is other-static-elf-address-space",
                ),
                "inactive layout",
            ),
            (
                SOURCE.replace(
                    "hierarchy is static-elf-hierarchy-frames",
                    "hierarchy is other-static-elf-hierarchy",
                ),
                "hierarchy extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is detached-static-elf-construction,\n                                                                                resource is hierarchy",
                    "owner is attached-static-elf-construction,\n                                                                                resource is hierarchy",
                ),
                "exact construction and role",
            ),
            (
                SOURCE.replace(
                    "unmapped frames release returned-static-elf-hierarchy",
                    "unmapped frames release static-elf-hierarchy-frames",
                ),
                "affine extent",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_address_space_activation_and_restoration() {
    // TOPAL-SYSTEMS-STATIC-ELF-ACTIVATION-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is attached-static-elf-construction,\n                                                                                policy is bounded-static-elf-kernel-continuation",
                    "owner is sealed-static-elf-construction,\n                                                                                policy is bounded-static-elf-kernel-continuation",
                ),
                "exact owner and continuation policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-static-elf-kernel-continuation",
                    "policy is unbounded-static-elf-kernel-continuation",
                ),
                "exact owner and continuation policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is active-static-elf-construction,\n                                                                                policy is bounded-bootstrap-kernel-continuation",
                    "owner is attached-static-elf-construction,\n                                                                                policy is bounded-bootstrap-kernel-continuation",
                ),
                "exact owner and continuation policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-bootstrap-kernel-continuation",
                    "policy is unbounded-bootstrap-kernel-continuation",
                ),
                "exact owner and continuation policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_entry_authorization_and_revocation() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTRY-AUTHORITY-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is restored-static-elf-construction,\n                                                                                policy is validated-static-elf-entry",
                    "owner is active-static-elf-construction,\n                                                                                policy is validated-static-elf-entry",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "policy is validated-static-elf-entry",
                    "policy is unchecked-static-elf-entry",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is stack-slot-restored-static-elf-construction,\n                                                                                policy is bounded-entry-qualification-rollback",
                    "owner is restored-static-elf-construction,\n                                                                                policy is bounded-entry-qualification-rollback",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-entry-qualification-rollback",
                    "policy is unbounded-entry-qualification-rollback",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is entry-restored-static-elf-construction\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK\"",
                    "owner is restored-static-elf-construction\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK\"",
                ),
                "exact restored construction",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_stack_slot_authorization_and_revocation() {
    // TOPAL-SYSTEMS-STATIC-ELF-STACK-SLOT-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is entry-authorized-static-elf-construction,\n                                                                                policy is bounded-linux-initial-stack-slot",
                    "owner is restored-static-elf-construction,\n                                                                                policy is bounded-linux-initial-stack-slot",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-linux-initial-stack-slot",
                    "policy is unguarded-linux-initial-stack-slot",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is zeroed-stack-backing-restored-construction,\n                                                                                policy is bounded-stack-slot-qualification-rollback",
                    "owner is entry-authorized-static-elf-construction,\n                                                                                policy is bounded-stack-slot-qualification-rollback",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-slot-qualification-rollback",
                    "policy is unbounded-stack-slot-qualification-rollback",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is stack-slot-restored-static-elf-construction,\n                                                                                policy is bounded-entry-qualification-rollback",
                    "owner is entry-authorized-static-elf-construction,\n                                                                                policy is bounded-entry-qualification-rollback",
                ),
                "exact owner and policy",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_stack_backing_reservation_and_release() {
    // TOPAL-SYSTEMS-STATIC-ELF-STACK-BACKING-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is stack-slot-authorized-static-elf-construction,\n                                                                                frame-count is 2,",
                    "owner is entry-authorized-static-elf-construction,\n                                                                                frame-count is 2,",
                ),
                "exact slot-authorized construction",
            ),
            (
                SOURCE.replace(
                    "static-elf-stack-backing is unmapped static image construction reserve stack backing (\n                                                                                owner is stack-slot-authorized-static-elf-construction,\n                                                                                frame-count is 2,",
                    "static-elf-stack-backing is unmapped static image construction reserve stack backing (\n                                                                                owner is stack-slot-authorized-static-elf-construction,\n                                                                                frame-count is 1,",
                ),
                "two unit-aligned frames",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "unmapped frames release static-elf-stack-backing",
                    "unmapped frames release static-elf-hierarchy-frames",
                ),
                "live allocator and affine extent",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_stack_backing_zeroing() {
    // TOPAL-SYSTEMS-STATIC-ELF-STACK-ZERO-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is stack-slot-authorized-static-elf-construction,\n                                                                                backing is static-elf-stack-backing,",
                    "owner is entry-authorized-static-elf-construction,\n                                                                                backing is static-elf-stack-backing,",
                ),
                "exact owner, backing, and complete extent",
            ),
            (
                SOURCE.replace(
                    "backing is static-elf-stack-backing,\n                                                                                extent is complete",
                    "backing is static-elf-hierarchy-frames,\n                                                                                extent is complete",
                ),
                "exact owner, backing, and complete extent",
            ),
            (
                SOURCE.replace(
                    "extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK\"",
                    "extent is partial\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK\"",
                ),
                "exact owner, backing, and complete extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is zeroed-stack-backing-restored-construction,\n                                                                                policy is bounded-stack-slot-qualification-rollback",
                    "owner is stack-slot-authorized-static-elf-construction,\n                                                                                policy is bounded-stack-slot-qualification-rollback",
                ),
                "exact owner and policy",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_stack_leaf_publication_and_revocation() {
    // TOPAL-SYSTEMS-STATIC-ELF-STACK-LEAF-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is zeroed-stack-backing-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                access is writable-non-executable,",
                    "owner is stack-slot-authorized-static-elf-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                access is writable-non-executable,",
                ),
                "exact owner, backing, RW/NX access, and complete extent",
            ),
            (
                SOURCE.replace(
                    "backing is static-elf-stack-backing,\n                                                                                access is writable-non-executable,",
                    "backing is static-elf-hierarchy-frames,\n                                                                                access is writable-non-executable,",
                ),
                "exact owner, backing, RW/NX access, and complete extent",
            ),
            (
                SOURCE.replace(
                    "access is writable-non-executable,\n                                                                                extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK\"",
                    "access is writable-executable,\n                                                                                extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK\"",
                ),
                "exact owner, backing, RW/NX access, and complete extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is published-stack-leaf-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-leaf-qualification-rollback",
                    "owner is zeroed-stack-backing-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-leaf-qualification-rollback",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-leaf-qualification-rollback",
                    "policy is retained-stack-leaf",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_stack_entry_initialization_and_clearing() {
    // TOPAL-SYSTEMS-STATIC-ELF-STACK-ENTRY-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is published-stack-leaf-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                profile is bounded-static-elf-busybox-sh,",
                    "owner is zeroed-stack-backing-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                profile is bounded-static-elf-busybox-sh,",
                ),
                "exact owner, backing, sealed profile, and complete extent",
            ),
            (
                SOURCE.replace(
                    "profile is bounded-static-elf-busybox-sh,\n                                                                                extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK\"",
                    "profile is unbounded-static-elf,\n                                                                                extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK\"",
                ),
                "exact owner, backing, sealed profile, and complete extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is initialized-stack-entry-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-entry-qualification-rollback",
                    "owner is published-stack-leaf-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-entry-qualification-rollback",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-entry-qualification-rollback",
                    "policy is retained-stack-entry",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_stack_pagesz_addition_and_removal() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-PAGESZ-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is initialized-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is provider-base-page,",
                    "owner is published-stack-leaf-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is provider-base-page,",
                ),
                "exact owner, backing, provider page value, and policy",
            ),
            (
                SOURCE.replace("value is provider-base-page", "value is source-page"),
                "exact owner, backing, provider page value, and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_OK",
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is pagesz-stack-entry-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-pagesz-qualification-rollback",
                    "owner is initialized-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-pagesz-qualification-rollback",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-pagesz-qualification-rollback",
                    "policy is retained-stack-pagesz",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_stack_entry_fact_addition_and_removal() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-ENTRY-001.
    for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is pagesz-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is authorized-static-elf-entry,",
                    "owner is initialized-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is authorized-static-elf-entry,",
                ),
                "exact owner, backing, authorized entry value, and policy",
            ),
            (
                SOURCE.replace(
                    "value is authorized-static-elf-entry",
                    "value is source-entry-address",
                ),
                "exact owner, backing, authorized entry value, and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_OK",
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is entry-auxv-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-entry-fact-qualification-rollback",
                    "owner is pagesz-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-entry-fact-qualification-rollback",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-entry-fact-qualification-rollback",
                    "policy is retained-stack-entry-fact",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_stack_program_header_group() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-PHDR-001.
    for source in [
            SOURCE.replace(
                "value is resident-static-elf-program-headers",
                "value is source-program-headers",
            ),
            SOURCE.replace(
                "owner is program-header-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-program-headers-qualification-rollback",
                "owner is entry-auxv-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-program-headers-qualification-rollback",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("program-header") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_stack_executable_name_fact() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-EXECFN-001.
    for source in [
            SOURCE.replace(
                "owner is program-header-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-executable-name,",
                "owner is entry-auxv-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-executable-name,",
            ),
            SOURCE.replace(
                "value is initial-executable-name",
                "value is source-executable-name",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_BAD",
            ),
            SOURCE.replace(
                "owner is executable-name-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-executable-name-qualification-rollback",
                "owner is program-header-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-executable-name-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-executable-name-qualification-rollback",
                "retained-stack-executable-name",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("executable-name")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_stack_credential_group() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-CREDENTIALS-001.
    for source in [
            SOURCE.replace(
                "owner is executable-name-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-process-credentials,",
                "owner is program-header-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-process-credentials,",
            ),
            SOURCE.replace(
                "value is initial-process-credentials",
                "value is source-process-credentials",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_BAD",
            ),
            SOURCE.replace(
                "owner is credential-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-credentials-qualification-rollback",
                "owner is executable-name-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-credentials-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-credentials-qualification-rollback",
                "retained-stack-credentials",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("credential") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_stack_secure_mode_fact() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-SECURE-001.
    for source in [
            SOURCE.replace(
                "owner is credential-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-execution-security,",
                "owner is executable-name-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-execution-security,",
            ),
            SOURCE.replace(
                "value is initial-execution-security",
                "value is source-execution-security",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_BAD",
            ),
            SOURCE.replace(
                "owner is secure-mode-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-secure-mode-qualification-rollback",
                "owner is credential-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-secure-mode-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-secure-mode-qualification-rollback",
                "retained-stack-secure-mode",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("secure-mode") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_stack_clock_tick_rate_fact() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-CLKTCK-001.
    for source in [
            SOURCE.replace(
                "owner is secure-mode-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is linux-user-hz,",
                "owner is credential-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is linux-user-hz,",
            ),
            SOURCE.replace("value is linux-user-hz", "value is hardware-timer-rate"),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_BAD",
            ),
            SOURCE.replace(
                "owner is clock-tick-rate-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-clock-tick-rate-qualification-rollback",
                "owner is secure-mode-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-clock-tick-rate-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-clock-tick-rate-qualification-rollback",
                "retained-stack-clock-tick-rate",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("clock-tick-rate")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_stack_platform_fact() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-PLATFORM-001.
    for source in [
            SOURCE.replace(
                "owner is clock-tick-rate-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is native-userspace-platform,",
                "owner is secure-mode-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is native-userspace-platform,",
            ),
            SOURCE.replace("value is native-userspace-platform", "value is source-platform-string"),
            SOURCE.replace("TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_OK", "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_BAD"),
            SOURCE.replace(
                "owner is platform-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-platform-qualification-rollback",
                "owner is clock-tick-rate-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-platform-qualification-rollback",
            ),
            SOURCE.replace("bounded-stack-platform-qualification-rollback", "retained-stack-platform"),
            SOURCE.replace("TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_OK", "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_BAD"),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("platform") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_stack_hardware_capabilities() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-HWCAP-001.
    for source in [
            SOURCE.replace(
                "owner is platform-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is native-userspace-hardware-capabilities,",
                "owner is clock-tick-rate-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is native-userspace-hardware-capabilities,",
            ),
            SOURCE.replace(
                "value is native-userspace-hardware-capabilities",
                "value is source-hardware-capability-mask",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_BAD",
            ),
            SOURCE.replace(
                "owner is hardware-capabilities-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-hardware-capabilities-qualification-rollback",
                "owner is platform-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-hardware-capabilities-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-hardware-capabilities-qualification-rollback",
                "retained-stack-hardware-capabilities",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("hardware-capability")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_stack_load_metadata() {
    // TOPAL-SYSTEMS-STATIC-ELF-AUXV-LOAD-METADATA-001.
    for source in [
            SOURCE.replace(
                "owner is hardware-capabilities-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is static-executable-load-metadata,",
                "owner is platform-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is static-executable-load-metadata,",
            ),
            SOURCE.replace(
                "value is static-executable-load-metadata",
                "value is source-loader-base",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_BAD",
            ),
            SOURCE.replace(
                "owner is load-metadata-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-load-metadata-qualification-rollback",
                "owner is hardware-capabilities-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-load-metadata-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-load-metadata-qualification-rollback",
                "retained-stack-load-metadata",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("load-metadata")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_stack_random_slot_authority() {
    // TOPAL-SYSTEMS-STATIC-ELF-RANDOM-SLOT-001.
    for source in [
            SOURCE.replace(
                "owner is load-metadata-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-auxv-random-object,",
                "owner is hardware-capabilities-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-auxv-random-object,",
            ),
            SOURCE.replace("value is initial-auxv-random-object", "value is source-random-buffer"),
            SOURCE.replace("extent-bytes is 16", "extent-bytes is 8"),
            SOURCE.replace(
                "bounded-linux-auxv-random-before-null",
                "source-selected-random-placement",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_OK",
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_BAD",
            ),
            SOURCE.replace(
                "owner is random-slot-authorized-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-random-slot-qualification-rollback",
                "owner is load-metadata-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-random-slot-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-random-slot-qualification-rollback",
                "retained-stack-random-slot",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("random-slot")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_prepared_process_retention() {
    // TOPAL-SYSTEMS-STATIC-ELF-PREPARED-RETENTION-001.
    for source in [
            SOURCE.replace(
                "owner is random-slot-authorized-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is bounded-static-elf-initial-process,",
                "owner is load-metadata-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is bounded-static-elf-initial-process,",
            ),
            SOURCE.replace("value is bounded-static-elf-initial-process", "value is unsealed-process"),
            SOURCE.replace("bounded-static-elf-pre-entropy-retention", "unbounded-process-retention"),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_BAD",
            ),
            SOURCE.replace(
                "owner is retained-static-process-after-entropy-rollback,\n                                                                                policy is bounded-static-elf-retention-qualification-rollback",
                "owner is random-slot-authorized-stack-construction,\n                                                                                policy is bounded-static-elf-retention-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-static-elf-retention-qualification-rollback",
                "discard-retained-process",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("prepared-process")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_retained_disjoint_work() {
    // TOPAL-SYSTEMS-STATIC-ELF-RETAINED-DISJOINT-WORK-001.
    for source in [
            SOURCE.replace(
                "owner is retained-static-process,\n                                                                                frame-count is 1,",
                "owner is random-slot-authorized-stack-construction,\n                                                                                frame-count is 1,",
            ),
            SOURCE.replace(
                "owner is retained-static-process,\n                                                                                frame-count is 1,\n                                                                                alignment-frames is 1,",
                "owner is retained-static-process,\n                                                                                frame-count is 2,\n                                                                                alignment-frames is 1,",
            ),
            SOURCE.replace(
                "bounded-static-elf-retained-disjoint-work",
                "overlapping-retained-work",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_OK",
                "TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("retained disjoint")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_entropy_handoff() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-HANDOFF-001.
    for source in [
            SOURCE.replace(
                "owner is retained-static-process-after-disjoint-work,\n                                                                                process is bounded-static-elf-initial-process,",
                "owner is retained-static-process,\n                                                                                process is bounded-static-elf-initial-process,",
            ),
            SOURCE.replace("process is bounded-static-elf-initial-process", "process is unrelated-process"),
            SOURCE.replace("bounded-static-elf-initial-random-delivery", "loader-owned-random-delivery"),
            SOURCE.replace("TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_OK", "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_BAD"),
            SOURCE.replace("bounded-static-elf-entropy-handoff-qualification-rollback", "discard-entropy-pending-process"),
            SOURCE.replace("TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_OK", "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_BAD"),
        ] {
            let error = analyze_systems_for_compiler(&source, &CompilerSystemsTargetSelection::initial_x86_64_qemu()).unwrap_err();
            assert!(error.message.contains("entropy handoff") || error.message.contains("exact marker"), "{}", error.message);
        }
}

#[test]
fn requires_exact_static_elf_entropy_acquisition_authority() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-ACQUISITION-AUTHORITY-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-pending-static-process,",
            "owner is retained-static-process,",
        ),
        SOURCE.replace(
            "output is initial-entropy-output",
            "output is unrelated-output",
        ),
        SOURCE.replace(
            "generation is generation-two",
            "generation is generation-one",
        ),
        SOURCE.replace(
            "bounded-static-elf-initial-entropy-acquisition",
            "loader-owned-entropy-acquisition",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_BAD",
        ),
        SOURCE.replace(
            "bounded-static-elf-entropy-acquisition-qualification-rollback",
            "discard-entropy-acquisition",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy acquisition") || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_entropy_device_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-DEVICE-COEXISTENCE-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "device is initial-virtio-rng-device",
            "device is unrelated-device",
        ),
        SOURCE.replace(
            "bounded-static-elf-entropy-device-coexistence",
            "synthetic-entropy-device",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy-device") || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_entropy_transport_discovery() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-TRANSPORT-DISCOVERY-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process-after-device,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "transport is initial-virtio-rng-pci-transport",
            "transport is unrelated-transport",
        ),
        SOURCE.replace(
            "bounded-static-elf-entropy-transport-discovery",
            "synthetic-entropy-transport",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy-transport") || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_entropy_transport_negotiation() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-TRANSPORT-NEGOTIATION-001.
    for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-transport,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "transport is initial-virtio-rng-pci-transport,\n                                                                                policy is bounded-static-elf-entropy-transport-negotiation",
                "transport is unrelated-transport,\n                                                                                policy is bounded-static-elf-entropy-transport-negotiation",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-transport-negotiation",
                "policy is synthetic-entropy-negotiation",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-transport negotiation")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_entropy_queue_discovery() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-QUEUE-DISCOVERY-001.
    for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-negotiation,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "transport is initial-virtio-rng-pci-transport,\n                                                                                queue is initial-virtio-rng-queue-zero,",
                "transport is unrelated-transport,\n                                                                                queue is initial-virtio-rng-queue-zero,",
            ),
            SOURCE.replace(
                "queue is initial-virtio-rng-queue-zero,",
                "queue is unrelated-queue,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-queue-discovery",
                "policy is synthetic-entropy-queue",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-queue")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_entropy_bus_master_qualification() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-BUS-MASTER-001.
    for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-queue,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "device is initial-virtio-rng-device,\n                                                                                policy is bounded-static-elf-entropy-bus-master",
                "device is unrelated-device,\n                                                                                policy is bounded-static-elf-entropy-bus-master",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-bus-master",
                "policy is synthetic-entropy-bus-master",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy bus-master")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_entropy_dma_buffer_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-DMA-BUFFER-COEXISTENCE-001.
    for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-bus-master,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "device is initial-virtio-rng-device,\n                                                                                domain is initial-dma-domain,",
                "device is unrelated-device,\n                                                                                domain is initial-dma-domain,",
            ),
            SOURCE.replace("domain is initial-dma-domain,", "domain is unrelated-domain,"),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-dma-buffer-coexistence",
                "policy is synthetic-entropy-dma-buffers",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy DMA-buffer")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_entropy_split_ring_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-SPLIT-RING-COEXISTENCE-001.
    for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-dma-buffers,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "device is initial-virtio-rng-device,\n                                                                                domain is initial-dma-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,",
                "device is unrelated-device,\n                                                                                domain is initial-dma-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,",
            ),
            SOURCE.replace(
                "domain is initial-dma-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,\n                                                                                policy is bounded-static-elf-entropy-split-ring-coexistence",
                "domain is unrelated-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,\n                                                                                policy is bounded-static-elf-entropy-split-ring-coexistence",
            ),
            SOURCE.replace(
                "domain is initial-dma-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,\n                                                                                policy is bounded-static-elf-entropy-split-ring-coexistence",
                "domain is initial-dma-domain,\n                                                                                queue is unrelated-queue,\n                                                                                policy is bounded-static-elf-entropy-split-ring-coexistence",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-split-ring-coexistence",
                "policy is synthetic-entropy-split-ring",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy split-ring")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_entropy_dma_preparation_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-DMA-PREPARATION-COEXISTENCE-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process-after-split-ring,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "policy is bounded-static-elf-entropy-dma-preparation-coexistence",
            "policy is synthetic-entropy-dma-preparation",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy DMA-preparation")
                || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_entropy_queue_address_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-QUEUE-ADDRESS-COEXISTENCE-001.
    for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-dma-preparation,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "transport is initial-virtio-rng-pci-transport,\n                                                                                policy is bounded-static-elf-entropy-queue-address-coexistence",
                "transport is unrelated-transport,\n                                                                                policy is bounded-static-elf-entropy-queue-address-coexistence",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-queue-address-coexistence",
                "policy is synthetic-entropy-queue-addresses",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy queue-address")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn requires_exact_static_elf_entropy_queue_activation_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-QUEUE-ACTIVATION-COEXISTENCE-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process-after-queue-addresses,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "policy is bounded-static-elf-entropy-empty-queue-activation-coexistence",
            "policy is synthetic-entropy-queue-activation",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy queue-activation")
                || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}
