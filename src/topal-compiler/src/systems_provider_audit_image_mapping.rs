fn audit_provider_image_mapping(file: &object::File<'_>, data: &[u8]) {
    let copy_payload = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_COPY_PAYLOAD_SYMBOL);
    assert!(copy_payload.len() > 8_000);
    assert_eq!(&copy_payload[..2], &[0x57, 0x56]);
    assert_eq!(
        copy_payload
            .windows(3)
            .filter(|bytes| *bytes == [0xfc, 0xf3, 0xa4])
            .count(),
        4,
        "copy lowering must contain exactly four bounded source-to-payload copies"
    );
    assert_eq!(
        copy_payload
            .windows(3)
            .filter(|bytes| *bytes == [0xfc, 0xf3, 0xa6])
            .count(),
        4,
        "copy lowering must compare all four copied intervals"
    );
    assert_eq!(
        copy_payload
            .windows(3)
            .filter(|bytes| *bytes == [0xfc, 0xf3, 0xae])
            .count(),
        4,
        "copy lowering must verify all four preserved-zero intervals"
    );
    for length in [0x6e0_u32, 0x18_b285, 0x5_7040, 0x9728] {
        let mut instruction = vec![0xb9];
        instruction.extend_from_slice(&length.to_le_bytes());
        assert_eq!(
            copy_payload
                .windows(instruction.len())
                .filter(|bytes| *bytes == instruction)
                .count(),
            2,
            "each file interval must be copied and compared exactly once"
        );
    }
    for length in [0x920_u32, 0xd7b, 0x1620, 0x8278] {
        let mut instruction = vec![0xb9];
        instruction.extend_from_slice(&length.to_le_bytes());
        assert_eq!(
            copy_payload
                .windows(instruction.len())
                .filter(|bytes| *bytes == instruction)
                .count(),
            1,
            "each preserved-zero interval must be checked exactly once"
        );
    }
    assert!(
        copy_payload
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x81, 0xc3, 0xf4, 0x30, 0x00, 0x00]),
        "copy lowering must derive the static member at archive offset 12532"
    );
    assert!(
        copy_payload
            .windows(3)
            .filter(|bytes| *bytes == [0x41, 0x80, 0xb8])
            .count()
            >= 653,
        "copy lowering must revalidate every approved ELF identity and program-header byte"
    );
    assert_eq!(
        copy_payload
            .windows(10)
            .filter(|bytes| {
                *bytes == [0x49, 0xb9, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]
            })
            .count(),
        2,
        "the exact live leaves must be verified before and after copying"
    );
    assert_eq!(
        copy_payload
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0xae, 0xf0])
            .count(),
        1,
        "the populated bytes must be published before verification"
    );

    let seal_permissions = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_SEAL_PERMISSIONS_SYMBOL);
    assert!(seal_permissions.len() > 8_000);
    assert_eq!(&seal_permissions[..2], &[0x57, 0x56]);
    assert_eq!(
        seal_permissions
            .windows(3)
            .filter(|bytes| *bytes == [0xfc, 0xf3, 0xa6])
            .count(),
        4,
        "permission sealing must revalidate all four populated intervals"
    );
    assert_eq!(
        seal_permissions
            .windows(3)
            .filter(|bytes| *bytes == [0xfc, 0xf3, 0xae])
            .count(),
        4,
        "permission sealing must revalidate all four preserved-zero intervals"
    );
    for (flags, count) in [
        (0x8000_0000_0000_0005_u64, 1_u32),
        (0x0000_0000_0000_0005_u64, 396),
        (0x8000_0000_0000_0005_u64, 88),
        (0x8000_0000_0000_0007_u64, 18),
    ] {
        let mut instruction = vec![0x49, 0xb9];
        instruction.extend_from_slice(&flags.to_le_bytes());
        instruction.push(0xb9);
        instruction.extend_from_slice(&count.to_le_bytes());
        assert_eq!(
            seal_permissions
                .windows(instruction.len())
                .filter(|bytes| *bytes == instruction)
                .count(),
            2,
            "each final role must appear once for publication and once for readback"
        );
    }
    assert_eq!(
        seal_permissions
            .windows(2)
            .filter(|bytes| *bytes == [0x48, 0xab])
            .count(),
        4,
        "permission sealing must publish exactly four contiguous role runs"
    );
    assert_eq!(
        seal_permissions
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0xae, 0xf0])
            .count(),
        1,
        "the complete inactive leaf rewrite must be published once"
    );
    assert_eq!(
        seal_permissions
            .windows(4)
            .filter(|bytes| *bytes == [0xfc, 0xf3, 0x48, 0xaf])
            .count(),
        2,
        "the nine absent leaves must be verified before and after sealing"
    );
    assert!(
        seal_permissions
            .windows(3)
            .filter(|bytes| *bytes == [0x41, 0x80, 0xb8])
            .count()
            >= 653,
        "permission sealing must revalidate approved ELF metadata"
    );

    let attach = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_ATTACH_ADDRESS_SPACE_SYMBOL);
    let activate = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_ACTIVATE_ADDRESS_SPACE_SYMBOL,
    );
    let restore = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_RESTORE_ADDRESS_SPACE_SYMBOL);
    let authorize_entry = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTRY_SYMBOL);
    let authorize_stack_slot =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_SLOT_SYMBOL);
    let revoke_stack_slot = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_SLOT_SYMBOL);
    let revoke_entry = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTRY_SYMBOL);
    let detach = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_DETACH_ADDRESS_SPACE_SYMBOL);
    assert_eq!(&attach[..3], &[0x57, 0x56, 0x51]);
    assert_eq!(&detach[..3], &[0x57, 0x56, 0x51]);
    for (lowering, role, relro_sealed) in [
        (&attach, "attachment", false),
        (&detach, "detachment", true),
    ] {
        assert_eq!(
            lowering
                .windows(3)
                .filter(|bytes| *bytes == [0x0f, 0x20, 0xd8])
                .count(),
            2,
            "{role} must prove the hierarchy inactive before and after its mutation"
        );
        assert_eq!(
            lowering
                .windows(9)
                .filter(|bytes| {
                    *bytes == [0xb9, 0x00, 0x0a, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab]
                })
                .count(),
            1,
            "{role} must clear the complete five-frame hierarchy"
        );
        assert!(
            lowering
                .windows(9)
                .any(|bytes| { bytes == [0xb9, 0x00, 0x0a, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xaf] }),
            "{role} must read back the complete cleared hierarchy"
        );
        for (flags, count) in [
            (0x8000_0000_0000_0005_u64, 1_u32),
            (0x0000_0000_0000_0005_u64, 396),
            (
                0x8000_0000_0000_0005_u64,
                if relro_sealed { 96 } else { 88 },
            ),
            (
                0x8000_0000_0000_0007_u64,
                if relro_sealed { 10 } else { 18 },
            ),
        ] {
            let mut instruction = vec![0x49, 0xb9];
            instruction.extend_from_slice(&flags.to_le_bytes());
            instruction.push(0xb9);
            instruction.extend_from_slice(&count.to_le_bytes());
            assert_eq!(
                lowering
                    .windows(instruction.len())
                    .filter(|bytes| *bytes == instruction)
                    .count(),
                2,
                "{role} must verify every sealed leaf role before and after mutation"
            );
        }
    }
    for qualified_mmio_leaf in [0x0000_0000_fec0_00fb_u64, 0x0000_0000_fee0_00fb] {
        let bytes = qualified_mmio_leaf.to_le_bytes();
        assert_eq!(
            attach
                .windows(bytes.len())
                .filter(|candidate| *candidate == bytes)
                .count(),
            2,
            "attachment must publish and read back each exact MMIO leaf"
        );
        assert_eq!(
            detach
                .windows(bytes.len())
                .filter(|candidate| *candidate == bytes)
                .count(),
            1,
            "detachment must read back each exact MMIO leaf before clearing"
        );
    }
    assert!(
        attach
            .windows(8)
            .any(|bytes| bytes == [0x49, 0x89, 0x84, 0xcb, 0x00, 0x20, 0x00, 0x00]),
        "attachment must construct all 512 low-directory entries"
    );
    assert_eq!(
        attach
            .windows(6)
            .filter(|bytes| *bytes == [0x48, 0x0d, 0xe3, 0x00, 0x00, 0x00])
            .count(),
        2,
        "attachment must publish and verify pre-accessed/pre-dirty identity leaves"
    );
    assert!(
        !detach
            .windows(8)
            .any(|bytes| bytes == [0x49, 0x89, 0x84, 0xcb, 0x00, 0x20, 0x00, 0x00]),
        "detachment must verify rather than reconstruct the low directory"
    );
    for (lowering, role, relro_sealed) in [
        (&activate, "activation", false),
        (&restore, "restoration", false),
    ] {
        assert_eq!(&lowering[..5], &[0x41, 0x50, 0x57, 0x56, 0x51]);
        assert!(
            !lowering
                .windows(9)
                .any(|bytes| bytes == [0xb9, 0x00, 0x0a, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab]),
            "{role} must not mutate the attached hierarchy"
        );
        for (flags, count) in [
            (0x8000_0000_0000_0005_u64, 1_u32),
            (0x0000_0000_0000_0005_u64, 396),
            (
                0x8000_0000_0000_0005_u64,
                if relro_sealed { 96 } else { 88 },
            ),
            (
                0x8000_0000_0000_0007_u64,
                if relro_sealed { 10 } else { 18 },
            ),
        ] {
            let mut instruction = vec![0x49, 0xb9];
            instruction.extend_from_slice(&flags.to_le_bytes());
            instruction.push(0xb9);
            instruction.extend_from_slice(&count.to_le_bytes());
            assert_eq!(
                lowering
                    .windows(instruction.len())
                    .filter(|bytes| *bytes == instruction)
                    .count(),
                1,
                "{role} must verify every sealed leaf role exactly once"
            );
        }
    }
    assert_eq!(
        activate
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0x22, 0xd9])
            .count(),
        1,
        "activation must perform exactly one static-root CR3 write"
    );
    assert_eq!(
        restore
            .windows(4)
            .filter(|bytes| *bytes == [0x41, 0x0f, 0x22, 0xd8])
            .count(),
        1,
        "restoration must perform exactly one bootstrap-root CR3 write"
    );
    assert_eq!(
        activate
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0x20, 0xd8])
            .count(),
        4,
        "activation must validate both roots and read back its CR3 handoff"
    );
    assert_eq!(
        restore
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0x20, 0xd8])
            .count(),
        3,
        "restoration must validate the active root, read back the bootstrap root, and revalidate static-root inactivity"
    );
    let restore_switch = restore
        .windows(4)
        .position(|bytes| bytes == [0x41, 0x0f, 0x22, 0xd8])
        .unwrap();
    let restore_leaf_validation = restore
        .windows(10)
        .position(|bytes| bytes == [0x49, 0xb9, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80])
        .unwrap();
    assert!(
        restore_switch < restore_leaf_validation,
        "restoration must restore identity translation before dereferencing construction resources"
    );
    assert_ne!(
        authorize_entry, revoke_entry,
        "entry authorization and revocation must validate distinct staged and sealed RELRO roles"
    );
    assert_eq!(&authorize_entry[..5], &[0x41, 0x50, 0x57, 0x56, 0x51]);
    assert_eq!(
        authorize_entry
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0x20, 0xd8])
            .count(),
        3,
        "entry authority must prove both root states and the exact active bootstrap root"
    );
    assert!(
        !authorize_entry
            .windows(3)
            .any(|bytes| bytes[..2] == [0x0f, 0x22]),
        "entry authority must not modify a translation root"
    );
    assert!(
        authorize_entry
            .windows(8)
            .any(|bytes| bytes == 0x004a_6580_u64.to_le_bytes())
            && authorize_entry
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x2d, 0x00, 0x00, 0x40, 0x00, 0x48])
            && authorize_entry
                .windows(6)
                .any(|bytes| bytes == [0x3d, 0x80, 0x65, 0x0a, 0x00, 0x0f]),
        "entry authority must retain the exact ELF entry and executable-span offset"
    );
    assert!(
        authorize_entry
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x39, 0x83, 0x30, 0x05, 0x00, 0x00])
            && authorize_entry
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x05, 0x00, 0x60, 0x0a, 0x00, 0x48]),
        "entry authority must bind leaf ordinal 166 to payload page offset 0xa6000"
    );
    assert!(
        !authorize_entry
            .windows(9)
            .any(|bytes| { bytes == [0xb9, 0x00, 0x0a, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab] }),
        "entry authority must not mutate the attached hierarchy"
    );
    assert_ne!(
        authorize_stack_slot, revoke_stack_slot,
        "stack-slot authorization and revocation must validate distinct staged and sealed RELRO roles"
    );
    assert_eq!(&authorize_stack_slot[..5], &[0x41, 0x50, 0x57, 0x56, 0x51]);
    assert!(
        authorize_stack_slot
            .windows(8)
            .any(|bytes| bytes == 0x004a_6580_u64.to_le_bytes())
            && authorize_stack_slot
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x39, 0x83, 0x30, 0x05, 0x00, 0x00]),
        "stack-slot authority must retain exact entry validation"
    );
    assert!(
        authorize_stack_slot
            .windows(8)
            .any(|bytes| bytes == [0x49, 0x83, 0xbb, 0xf0, 0x0f, 0x00, 0x00, 0x00])
            && authorize_stack_slot
                .windows(8)
                .any(|bytes| bytes == [0x49, 0x83, 0xbb, 0xf8, 0x0f, 0x00, 0x00, 0x00]),
        "stack-slot authority must prove guard ordinal 510 and slot ordinal 511 absent"
    );
    assert!(
        authorize_stack_slot
            .windows(8)
            .any(|bytes| bytes == 0x0060_0000_u64.to_le_bytes()),
        "stack-slot authority must retain the provider-private aligned initial top"
    );
    assert_eq!(
        authorize_stack_slot
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0x20, 0xd8])
            .count(),
        3,
        "stack-slot authority must prove both root states and the active bootstrap root"
    );
    assert!(
        !authorize_stack_slot
            .windows(3)
            .any(|bytes| bytes[..2] == [0x0f, 0x22])
            && !authorize_stack_slot
                .windows(9)
                .any(|bytes| { bytes == [0xb9, 0x00, 0x0a, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab] }),
        "stack-slot authority must not modify a root or hierarchy byte"
    );

    let revoke = file
        .symbol_by_name(X86_SYSTEMS_STATIC_IMAGE_REVOKE_SYMBOL)
        .unwrap();
    let start = usize::try_from(revoke.address()).unwrap();
    let end = start + usize::try_from(revoke.size()).unwrap();
    let revoke = &data[start..end];
    for (flags, count) in [
        (0x8000_0000_0000_0005_u64, 1_u32),
        (0x0000_0000_0000_0005_u64, 396),
        (0x8000_0000_0000_0005_u64, 96),
        (0x8000_0000_0000_0007_u64, 10),
    ] {
        let mut instruction = vec![0x49, 0xb9];
        instruction.extend_from_slice(&flags.to_le_bytes());
        instruction.push(0xb9);
        instruction.extend_from_slice(&count.to_le_bytes());
        assert_eq!(
            revoke
                .windows(instruction.len())
                .filter(|bytes| *bytes == instruction)
                .count(),
            1,
            "revocation must first verify every sealed leaf role"
        );
    }
    assert_eq!(
        revoke
            .windows(5)
            .filter(|bytes| *bytes == [0xb9, 0x00, 0x02, 0x00, 0x00])
            .count(),
        2,
        "revocation must clear and then verify all 512 entries"
    );
    assert!(
        revoke
            .windows(4)
            .any(|bytes| bytes == [0xfc, 0xf3, 0x48, 0xab])
    );
    assert_eq!(
        revoke
            .windows(4)
            .filter(|bytes| *bytes == [0xfc, 0xf3, 0x48, 0xaf])
            .count(),
        2
    );
    assert_eq!(
        revoke
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0xae, 0xf0])
            .count(),
        1
    );

    let translation_selector = file
        .symbol_by_name(X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL)
        .unwrap();
    let translation_selector_start = usize::try_from(translation_selector.address()).unwrap();
    let translation_selector_end =
        translation_selector_start + usize::try_from(translation_selector.size()).unwrap();
    let translation_selector = &data[translation_selector_start..translation_selector_end];
    assert!(
        translation_selector
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x81, 0xc2, 0x00, 0x50, 0x00, 0x00]),
        "translation backing selection must reserve five complete pages"
    );
    assert!(
        translation_selector
            .windows(6)
            .any(|bytes| { bytes == [0x48, 0x3d, 0x00, 0xb0, 0xff, 0x3f] })
    );

    let builder = file
        .symbol_by_name(X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL)
        .unwrap();
    let builder_start = usize::try_from(builder.address()).unwrap();
    let builder_end = builder_start + usize::try_from(builder.size()).unwrap();
    let builder = &data[builder_start..builder_end];
    assert!(
        builder
            .windows(4)
            .any(|bytes| bytes == [0xfc, 0xf3, 0x48, 0xab])
    );
    assert!(
        builder
            .windows(6)
            .any(|bytes| bytes == [0x81, 0xf9, 0x00, 0x02, 0x00, 0x00])
    );
    assert!(builder.windows(3).any(|bytes| bytes == [0x0f, 0xae, 0xf0]));
    assert!(
        builder
            .windows(8)
            .any(|bytes| { bytes == 0x0000_0000_fec0_009b_u64.to_le_bytes() })
    );
    assert!(
        builder
            .windows(8)
            .any(|bytes| { bytes == 0x0000_0000_fee0_009b_u64.to_le_bytes() })
    );
    assert!(
        builder
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x80, 0x18, 0x10, 0x00, 0x00]),
        "replacement translation must publish the local-APIC directory through PDPT[3]"
    );
    assert!(
        !builder
            .windows(8)
            .any(|bytes| bytes == 0x0060_0007_u64.to_le_bytes())
    );
    assert!(
        !builder
            .windows(8)
            .any(|bytes| bytes == 0x8000_0000_0060_1007_u64.to_le_bytes())
    );
}
