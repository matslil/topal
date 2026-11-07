fn audit_provider_bootstrap<'data>(file: &object::File<'data>) -> &'data [u8] {
    let validator = file.symbol_by_name(X86_SYSTEMS_BOOT_MEMORY_SYMBOL).unwrap();
    assert!(validator.size() > 100);
    let section = file
        .section_by_index(validator.section_index().unwrap())
        .unwrap();
    let data = section.data().unwrap();
    let start = usize::try_from(validator.address()).unwrap();
    let end = start + usize::try_from(validator.size()).unwrap();
    assert_eq!(&data[end - 3..end], &[0x31, 0xc0, 0xc3]);
    let selector = file
        .symbol_by_name(X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL)
        .unwrap();
    assert!(selector.size() > 100);
    let selector_start = usize::try_from(selector.address()).unwrap();
    let selector_end = selector_start + usize::try_from(selector.size()).unwrap();
    let selector = &data[selector_start..selector_end];
    assert_ne!(selector, &data[start..end]);
    assert!(
        selector
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x81, 0xfa, 0x00, 0x00, 0x00, 0x40]),
        "selector must constrain the complete extent to bootstrap identity mappings"
    );
    assert!(selector.windows(3).any(|bytes| bytes == [0x83, 0xfa, 0x01]));
    assert!(selector.windows(3).any(|bytes| bytes == [0x83, 0xfa, 0x03]));
    assert!(
        selector.windows(3).any(|bytes| bytes == [0x83, 0xfa, 0x05]),
        "selector must admit the exact five-frame static ELF hierarchy extent"
    );
    assert!(
        selector
            .windows(6)
            .any(|bytes| bytes == [0x81, 0xfa, 0xf7, 0x01, 0x00, 0x00]),
        "selector must admit the exact 503-frame static ELF extent"
    );
    assert!(
        selector
            .windows(4)
            .any(|bytes| bytes == [0x48, 0xc1, 0xe2, 0x0c]),
        "selector must convert the admitted frame count to a byte extent"
    );
    assert!(
        selector.windows(3).any(|bytes| bytes == [0x4d, 0x39, 0xda]),
        "selector must require the complete extent to fit its E820 RAM run"
    );
    assert_eq!(&selector[..7], &[0x41, 0x54, 0x41, 0x55, 0x49, 0x89, 0xcc]);
    assert!(
        selector
            .windows(7)
            .any(|bytes| bytes == [0x41, 0xf7, 0xc5, 0xff, 0x0f, 0x00, 0x00]),
        "selector must validate the supplied complete exclusion end"
    );
    assert!(
        selector.windows(3).any(|bytes| bytes == [0x4d, 0x39, 0xe2])
            && selector.windows(3).any(|bytes| bytes == [0x4c, 0x39, 0xe8])
            && selector.windows(3).any(|bytes| bytes == [0x4c, 0x89, 0xe8]),
        "selector must detect overlap and advance beyond the complete live exclusion"
    );
    assert!(
        selector.windows(3).any(|bytes| bytes == [0x48, 0x39, 0xf8]),
        "selector must apply its provider-private allocation floor"
    );
    assert_eq!(
        selector
            .windows(5)
            .filter(|bytes| *bytes == [0x41, 0x5d, 0x41, 0x5c, 0xc3])
            .count(),
        2,
        "success and failure must both restore r13 and r12"
    );
    assert_eq!(
        &selector[selector.len() - 7..],
        &[0x31, 0xc0, 0x41, 0x5d, 0x41, 0x5c, 0xc3]
    );

    let prepare = file
        .symbol_by_name(X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL)
        .unwrap();
    let start = usize::try_from(prepare.address()).unwrap();
    let end = start + usize::try_from(prepare.size()).unwrap();
    let prepare = &data[start..end];
    assert!(
        prepare
            .windows(10)
            .any(|bytes| bytes == [0x49, 0xb9, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]),
        "initial leaves must be present, user-readable, writable, and NX"
    );
    assert_eq!(
        prepare
            .windows(5)
            .filter(|bytes| *bytes == [0xb9, 0xf7, 0x01, 0x00, 0x00])
            .count(),
        2,
        "construction and verification must each cover exactly 503 leaves"
    );
    assert!(
        prepare
            .windows(5)
            .any(|bytes| bytes == [0xb9, 0x09, 0x00, 0x00, 0x00])
    );
    assert!(
        prepare
            .windows(5)
            .any(|bytes| bytes == [0xb9, 0x00, 0x02, 0x00, 0x00])
    );
    assert!(
        prepare
            .windows(4)
            .any(|bytes| bytes == [0xfc, 0xf3, 0x48, 0xab])
    );
    assert!(
        prepare
            .windows(4)
            .any(|bytes| bytes == [0xfc, 0xf3, 0x48, 0xaf])
    );
    assert_eq!(
        prepare
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0xae, 0xf0])
            .count(),
        1
    );

    let zero_payload = file
        .symbol_by_name(X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL)
        .unwrap();
    let start = usize::try_from(zero_payload.address()).unwrap();
    let end = start + usize::try_from(zero_payload.size()).unwrap();
    let zero_payload = &data[start..end];
    assert_eq!(
        zero_payload
            .windows(5)
            .filter(|bytes| *bytes == [0xb9, 0x00, 0xee, 0x03, 0x00])
            .count(),
        2,
        "payload zeroing and readback must each cover 503 complete pages"
    );
    assert_eq!(
        zero_payload
            .windows(10)
            .filter(|bytes| {
                *bytes == [0x49, 0xb9, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]
            })
            .count(),
        2,
        "the exact live leaves must be verified before and after zeroing"
    );
    assert_eq!(
        zero_payload
            .windows(4)
            .filter(|bytes| *bytes == [0xfc, 0xf3, 0x48, 0xab])
            .count(),
        1
    );
    assert_eq!(
        zero_payload
            .windows(4)
            .filter(|bytes| *bytes == [0xfc, 0xf3, 0x48, 0xaf])
            .count(),
        3
    );
    assert_eq!(
        zero_payload
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0xae, 0xf0])
            .count(),
        1
    );

    let zero_stack_backing =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_ZERO_STACK_BACKING_SYMBOL);
    assert!(
        zero_stack_backing
            .windows(7)
            .any(|bytes| bytes == [0x48, 0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00])
            && zero_stack_backing
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xff, 0x00, 0xb0, 0xff, 0x3f]),
        "stack-backing zeroing must validate page alignment and the complete bounded extent"
    );
    assert_eq!(
        zero_stack_backing
            .windows(11)
            .filter(|bytes| {
                *bytes
                    == [
                        0xfc, 0x31, 0xc0, 0xb9, 0x00, 0x04, 0x00, 0x00, 0xf3, 0x48, 0xab,
                    ]
            })
            .count(),
        1,
        "stack-backing zeroing must clear exactly 1024 forward qwords"
    );
    assert_eq!(
        zero_stack_backing
            .windows(10)
            .filter(|bytes| {
                *bytes == [0x31, 0xc0, 0xb9, 0x00, 0x04, 0x00, 0x00, 0xf3, 0x48, 0xaf]
            })
            .count(),
        1,
        "stack-backing zeroing must read back exactly 1024 forward qwords"
    );
    assert_eq!(
        zero_stack_backing
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0xae, 0xf8])
            .count(),
        1,
        "stack-backing zeroing must order the complete clear before readback"
    );

    let publish_stack_leaf =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_PUBLISH_STACK_LEAF_SYMBOL);
    let revoke_stack_leaf = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_LEAF_SYMBOL);
    assert!(
        publish_stack_leaf
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x89, 0x86, 0xf0, 0x4f, 0x00, 0x00])
            && publish_stack_leaf
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x89, 0x86, 0xf8, 0x4f, 0x00, 0x00]),
        "stack-leaf publication must write adjacent high-table leaf ordinals 510 and 511"
    );
    assert!(
        [0xf0_u8, 0xf8].into_iter().all(|offset| {
            revoke_stack_leaf.windows(11).any(|bytes| {
                bytes
                    == [
                        0x48, 0xc7, 0x86, offset, 0x4f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    ]
            })
        }),
        "stack-leaf revocation must erase adjacent high-table leaf ordinals 510 and 511"
    );
    for bytes in [publish_stack_leaf.as_slice(), revoke_stack_leaf.as_slice()] {
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each inactive leaf transition must fence before readback"
        );
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x04, 0x00, 0x00, 0xfc, 0xf3, 0x48]
                })
                .count(),
            2,
            "each transition must verify both zero backing pages before and after publication"
        );
        assert_eq!(
            bytes
                .windows(8)
                .filter(|window| { *window == [0xb9, 0x08, 0x00, 0x00, 0x00, 0xfc, 0xf3, 0x48] })
                .count(),
            2,
            "both hierarchy MMIO-tail readbacks must be complete"
        );
        assert_eq!(
            bytes
                .windows(8)
                .filter(|window| { *window == [0xb9, 0x09, 0x00, 0x00, 0x00, 0xfc, 0xf3, 0x48] })
                .count(),
            2,
            "the nine absent low-table leaves must be read back before and after the transition"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive stack-leaf transitions must not issue active-address invalidation"
        );
    }
    assert_eq!(
        publish_stack_leaf
            .windows(10)
            .filter(|bytes| {
                *bytes == [0x49, 0xb8, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]
            })
            .count(),
        2,
        "publication must construct and read back the exact RW/NX user leaf"
    );
    assert_eq!(
        revoke_stack_leaf
            .windows(10)
            .filter(|bytes| {
                *bytes == [0x49, 0xb8, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]
            })
            .count(),
        1,
        "revocation must validate the exact RW/NX user leaf before erasing it"
    );

    let initialize_stack_entry = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_INITIALIZE_STACK_ENTRY_SYMBOL,
    );
    let clear_stack_entry = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_CLEAR_STACK_ENTRY_SYMBOL);
    for expected in [
        0x007f_ffe0_u64,
        0x007f_ffe8,
        0x0078_6f62_7973_7562,
        0x0000_0000_0000_6873,
    ] {
        assert!(
            initialize_stack_entry
                .windows(8)
                .any(|bytes| bytes == expected.to_le_bytes()),
            "stack-entry initialization must carry every sealed pointer and string fact"
        );
    }
    assert!(
        initialize_stack_entry
            .windows(7)
            .any(|bytes| { bytes == [0x49, 0xc7, 0x83, 0xa0, 0x0f, 0x00, 0x00] }),
        "stack-entry initialization must place argc at the aligned entry offset"
    );
    assert_eq!(
        clear_stack_entry
            .windows(10)
            .filter(|bytes| {
                *bytes == [0x31, 0xc0, 0xb9, 0x00, 0x04, 0x00, 0x00, 0xfc, 0xf3, 0x48]
            })
            .count(),
        2,
        "stack-entry clearing must clear and then read back exactly 1024 qwords"
    );
    for bytes in [
        initialize_stack_entry.as_slice(),
        clear_stack_entry.as_slice(),
    ] {
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each stack-entry transition must fence before readback"
        );
        assert!(
            bytes
                .windows(10)
                .any(|window| window == [0x49, 0xb8, 0x07, 0, 0, 0, 0, 0, 0, 0x80]),
            "each stack-entry transition must preserve the exact RW/NX user leaf"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive stack-entry transitions must not issue active-address invalidation"
        );
    }

    let add_stack_pagesz = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PAGESZ_SYMBOL);
    let remove_stack_pagesz =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PAGESZ_SYMBOL);
    for bytes in [add_stack_pagesz.as_slice(), remove_stack_pagesz.as_slice()] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0x00, 0x00, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each AT_PAGESZ transition must clear the complete frame before reconstruction"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each AT_PAGESZ transition must fence before complete readback"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive AT_PAGESZ transitions must not issue active-address invalidation"
        );
    }
    for expected in [6_u64, 4096] {
        assert!(
            add_stack_pagesz
                .windows(8)
                .any(|bytes| bytes == expected.to_le_bytes()),
            "AT_PAGESZ addition must encode the exact native tag and page value"
        );
    }
    assert!(
        add_stack_pagesz
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0x90, 0x0f, 0x00, 0x00]),
        "AT_PAGESZ addition must reconstruct the aligned extended entry position"
    );
    assert!(
        remove_stack_pagesz
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0xa0, 0x0f, 0x00, 0x00]),
        "AT_PAGESZ removal must restore the aligned minimal entry position"
    );

    let add_stack_entry_fact =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_ENTRY_FACT_SYMBOL);
    let remove_stack_entry_fact = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_ENTRY_FACT_SYMBOL,
    );
    for bytes in [
        add_stack_entry_fact.as_slice(),
        remove_stack_entry_fact.as_slice(),
    ] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0x00, 0x00, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each AT_ENTRY transition must clear the complete frame before reconstruction"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each AT_ENTRY transition must fence before complete readback"
        );
        assert!(
            bytes
                .windows(10)
                .any(|window| window == [0x48, 0xb8, 0x80, 0x65, 0x4a, 0, 0, 0, 0, 0]),
            "each AT_ENTRY transition must revalidate the retained ELF entry"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive AT_ENTRY transitions must not issue active-address invalidation"
        );
    }
    for expected in [6_u64, 4096, 9, 0x004a_6580] {
        assert!(
            add_stack_entry_fact
                .windows(8)
                .any(|bytes| bytes == expected.to_le_bytes()),
            "AT_ENTRY addition must encode the prior page fact and exact authorized entry"
        );
    }
    assert!(
        add_stack_entry_fact
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0x80, 0x0f, 0x00, 0x00]),
        "AT_ENTRY addition must reconstruct the aligned extended entry position"
    );
    assert!(
        remove_stack_entry_fact
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0x90, 0x0f, 0x00, 0x00]),
        "AT_ENTRY removal must restore the aligned AT_PAGESZ entry position"
    );

    let add_program_headers = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PROGRAM_HEADERS_SYMBOL,
    );
    let remove_program_headers = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PROGRAM_HEADERS_SYMBOL,
    );
    for bytes in [
        add_program_headers.as_slice(),
        remove_program_headers.as_slice(),
    ] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each program-header transition must clear the complete frame"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each program-header transition must fence once"
        );
        assert!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x41, 0x80, 0xba])
                .count()
                == 1332,
            "each transition must verify every invariant resident ELF header and program-table byte before and after reconstruction"
        );
        assert_eq!(
            bytes
                .windows(4)
                .filter(|window| *window == [0x4d, 0x0f, 0x45, 0xc8])
                .count(),
            4,
            "each resident-header check must select one coherent section-directory tuple"
        );
        for expected in [0x0001_0000_0000_0000_u64, 0x0001_0016_0017_0040_u64] {
            assert!(
                bytes
                    .windows(8)
                    .any(|window| window == expected.to_le_bytes()),
                "each resident-header check must encode both complete approved tuples"
            );
        }
    }
    for expected in [3_u64, 0x0040_0040, 4, 56, 5, 11] {
        assert!(
            add_program_headers
                .windows(8)
                .any(|bytes| bytes == expected.to_le_bytes()),
            "program-header addition must encode the exact coherent native group"
        );
    }
    assert!(
        add_program_headers
            .windows(7)
            .any(|bytes| { bytes == [0x49, 0x89, 0x83, 0x50, 0x0f, 0x00, 0x00] })
    );
    assert!(
        remove_program_headers
            .windows(7)
            .any(|bytes| { bytes == [0x49, 0x89, 0x83, 0x80, 0x0f, 0x00, 0x00] })
    );

    let add_executable_name = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_EXECUTABLE_NAME_SYMBOL,
    );
    let remove_executable_name = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_EXECUTABLE_NAME_SYMBOL,
    );
    for bytes in [
        add_executable_name.as_slice(),
        remove_executable_name.as_slice(),
    ] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each AT_EXECFN transition must clear the complete frame"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each AT_EXECFN transition must fence once"
        );
        assert!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x41, 0x80, 0xba])
                .count()
                == 1332,
            "each transition must verify every invariant resident ELF header and program-table byte before and after reconstruction"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 31_u64.to_le_bytes()),
            "each transition must encode or verify the exact AT_EXECFN tag"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 0x007f_ffe0_u64.to_le_bytes()),
            "each transition must encode or verify the argv[0] pointer alias"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive AT_EXECFN transitions must not invalidate an active address"
        );
    }
    assert!(
        add_executable_name
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0x40, 0x0f, 0x00, 0x00]),
        "AT_EXECFN addition must reconstruct the extended entry position"
    );
    assert!(
        remove_executable_name
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0x50, 0x0f, 0x00, 0x00]),
        "AT_EXECFN removal must restore the program-header entry position"
    );

    let add_credentials =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CREDENTIALS_SYMBOL);
    let remove_credentials = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CREDENTIALS_SYMBOL,
    );
    for bytes in [add_credentials.as_slice(), remove_credentials.as_slice()] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each credential transition must clear the complete frame"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each credential transition must fence once"
        );
        assert!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x41, 0x80, 0xba])
                .count()
                == 1332,
            "each transition must verify every invariant resident ELF header and program-table byte before and after reconstruction"
        );
        for tag in [11_u64, 12, 13, 14] {
            assert!(
                bytes.windows(8).any(|window| window == tag.to_le_bytes()),
                "each transition must encode or verify every credential tag"
            );
        }
        assert!(
            bytes
                .windows(8)
                .any(|window| window == [0x49, 0x83, 0xbb, 0x90, 0x0f, 0, 0, 0]),
            "credential values must be read back as the sealed zero identity"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive credential transitions must not invalidate an active address"
        );
    }
    assert!(
        add_credentials
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0x00, 0x0f, 0x00, 0x00]),
        "credential addition must reconstruct the extended entry position"
    );
    assert!(
        remove_credentials
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0x40, 0x0f, 0x00, 0x00]),
        "credential removal must restore the AT_EXECFN entry position"
    );

    let add_secure_mode =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_SECURE_MODE_SYMBOL);
    let remove_secure_mode = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_SECURE_MODE_SYMBOL,
    );
    for bytes in [add_secure_mode.as_slice(), remove_secure_mode.as_slice()] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each AT_SECURE transition must clear the complete frame"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each AT_SECURE transition must fence once"
        );
        assert!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x41, 0x80, 0xba])
                .count()
                == 1332,
            "each transition must verify every invariant resident ELF header and program-table byte before and after reconstruction"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 23_u64.to_le_bytes()),
            "each transition must encode or verify native AT_SECURE"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == [0x49, 0x83, 0xbb, 0xc0, 0x0f, 0, 0, 0]),
            "AT_SECURE must be read back as the sealed zero result"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive AT_SECURE transitions must not invalidate an active address"
        );
    }
    assert!(
        add_secure_mode
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0xf0, 0x0e, 0x00, 0x00]),
        "AT_SECURE addition must reconstruct the extended entry position"
    );
    assert!(
        remove_secure_mode
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0x00, 0x0f, 0x00, 0x00]),
        "AT_SECURE removal must restore the credential entry position"
    );

    let add_clock_tick_rate = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CLOCK_TICK_RATE_SYMBOL,
    );
    let remove_clock_tick_rate = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CLOCK_TICK_RATE_SYMBOL,
    );
    for bytes in [
        add_clock_tick_rate.as_slice(),
        remove_clock_tick_rate.as_slice(),
    ] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each AT_CLKTCK transition must clear the complete frame"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each AT_CLKTCK transition must fence once"
        );
        assert!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x41, 0x80, 0xba])
                .count()
                == 1332,
            "each transition must verify every invariant resident ELF header and program-table byte before and after reconstruction"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 17_u64.to_le_bytes()),
            "each transition must encode or verify native AT_CLKTCK"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 100_u64.to_le_bytes()),
            "AT_CLKTCK must carry the sealed Linux USER_HZ value"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive AT_CLKTCK transitions must not invalidate an active address"
        );
    }
    assert!(
        add_clock_tick_rate
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0xe0, 0x0e, 0x00, 0x00]),
        "AT_CLKTCK addition must reconstruct the extended entry position"
    );
    assert!(
        remove_clock_tick_rate
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0xf0, 0x0e, 0x00, 0x00]),
        "AT_CLKTCK removal must restore the AT_SECURE entry position"
    );

    let add_platform = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PLATFORM_SYMBOL);
    let remove_platform =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PLATFORM_SYMBOL);
    for bytes in [add_platform.as_slice(), remove_platform.as_slice()] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each AT_PLATFORM transition must clear the complete frame"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each AT_PLATFORM transition must fence once"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 15_u64.to_le_bytes()),
            "each transition must encode or verify native AT_PLATFORM"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 0x007f_fff0_u64.to_le_bytes()),
            "AT_PLATFORM must point to the bounded platform object"
        );
        assert!(
            bytes.windows(8).any(|window| window == b"x86_64\0\0"),
            "AT_PLATFORM must carry the complete native platform string"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive AT_PLATFORM transitions must not invalidate an active address"
        );
    }
    assert!(
        add_platform
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0xd0, 0x0e, 0x00, 0x00]),
        "AT_PLATFORM addition must reconstruct the extended entry position"
    );
    assert!(
        remove_platform
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0xe0, 0x0e, 0x00, 0x00]),
        "AT_PLATFORM removal must restore the AT_CLKTCK entry position"
    );

    let add_hardware_capabilities = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_HARDWARE_CAPABILITIES_SYMBOL,
    );
    let remove_hardware_capabilities = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_HARDWARE_CAPABILITIES_SYMBOL,
    );
    for bytes in [
        add_hardware_capabilities.as_slice(),
        remove_hardware_capabilities.as_slice(),
    ] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each hardware-capability transition must clear the complete frame"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each hardware-capability transition must fence once"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 16_u64.to_le_bytes()),
            "each transition must encode or verify native AT_HWCAP"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 0x178b_fbfd_u64.to_le_bytes()),
            "AT_HWCAP must carry the pinned qemu64-v1 capability mask"
        );
        assert!(
            bytes
                .windows(8)
                .any(|window| window == 26_u64.to_le_bytes()),
            "each transition must encode or verify native AT_HWCAP2"
        );
        assert!(
            bytes.windows(17).any(|window| {
                window[..10] == [0x48, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0]
                    && window[10] == 0x49
                    && matches!(window[11], 0x39 | 0x89)
                    && window[12..] == [0x83, 0xc0, 0x0f, 0, 0]
            }),
            "AT_HWCAP2 must carry an explicit zero value before AT_NULL"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive hardware-capability transitions must not invalidate an active address"
        );
    }
    assert!(
        add_hardware_capabilities
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0xb0, 0x0e, 0x00, 0x00]),
        "hardware-capability addition must enlarge the aligned block downward"
    );
    assert!(
        remove_hardware_capabilities
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0xd0, 0x0e, 0x00, 0x00]),
        "hardware-capability removal must restore the platform entry position"
    );

    let add_load_metadata = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_LOAD_METADATA_SYMBOL,
    );
    let remove_load_metadata = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_LOAD_METADATA_SYMBOL,
    );
    for bytes in [
        add_load_metadata.as_slice(),
        remove_load_metadata.as_slice(),
    ] {
        assert_eq!(
            bytes
                .windows(10)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc, 0xf3, 0x48]
                })
                .count(),
            1,
            "each load-metadata transition must clear the complete frame"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x0f, 0xae, 0xf8])
                .count(),
            1,
            "each load-metadata transition must fence once"
        );
        for (tag, value_offset) in [(7_u64, 0x0fb0_u32), (8_u64, 0x0fc0_u32)] {
            assert!(
                bytes.windows(8).any(|window| window == tag.to_le_bytes()),
                "each transition must encode or verify both load-metadata tags"
            );
            assert!(
                bytes.windows(17).any(|window| {
                    window[..10] == [0x48, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0]
                        && window[10] == 0x49
                        && matches!(window[11], 0x39 | 0x89)
                        && window[12] == 0x83
                        && window[13..] == value_offset.to_le_bytes()
                }),
                "each load-metadata fact must carry an explicit zero value"
            );
        }
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive load-metadata transitions must not invalidate an active address"
        );
    }
    assert!(
        add_load_metadata
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0x90, 0x0e, 0x00, 0x00]),
        "load-metadata addition must enlarge the aligned block downward"
    );
    assert!(
        remove_load_metadata
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x89, 0x83, 0xb0, 0x0e, 0x00, 0x00]),
        "load-metadata removal must restore the hardware-capability entry position"
    );

    data
}
