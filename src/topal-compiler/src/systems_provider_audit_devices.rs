fn audit_provider_devices(file: &object::File<'_>) {
    for (name, expected) in [
        (
            "topal_x86_systems_uart16550_write",
            &[
                0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20, 0x74, 0xfb, 0x66, 0xba, 0xf8, 0x03, 0x40,
                0x88, 0xf8, 0xee, 0xc3,
            ][..],
        ),
        (
            X86_SYSTEMS_PCI_CONFIGURATION_READ_DWORD_SYMBOL,
            &[
                0x66, 0xba, 0xf8, 0x0c, 0x89, 0xf8, 0xef, 0x66, 0xba, 0xfc, 0x0c, 0xed, 0x39, 0xf0,
                0x0f, 0x94, 0xc0, 0x0f, 0xb6, 0xc0, 0xc3,
            ],
        ),
        ("topal_x86_systems_debug_break", &[0xcc, 0xc3]),
        ("topal_x86_systems_interrupt_return", &[0x48, 0xcf]),
        ("topal_x86_systems_fatal", &[0xfa, 0xf4, 0xeb, 0xfd]),
        (X86_SYSTEMS_CRITICAL_ENTER_SYMBOL, &[0x9c, 0x58, 0xfa, 0xc3]),
        (
            X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
            &[
                0x40, 0xf6, 0xc7, 0x02, 0x74, 0x16, 0xf7, 0xc7, 0x00, 0x02, 0x00, 0x00, 0x74, 0x07,
                0xfb, 0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3, 0xfa, 0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3,
                0x31, 0xc0, 0xc3,
            ],
        ),
        (
            X86_SYSTEMS_ATOMIC_CREATE_SYMBOL,
            &[
                0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, 0x75, 0x09, 0x48, 0x89, 0x37, 0xb8, 0x01,
                0x00, 0x00, 0x00, 0xc3, 0x31, 0xc0, 0xc3,
            ],
        ),
        (
            X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
            &[
                0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, 0x75, 0x0f, 0x48, 0x89, 0xf0, 0xf0, 0x48,
                0x0f, 0xb1, 0x17, 0x0f, 0x94, 0xc0, 0x0f, 0xb6, 0xc0, 0xc3, 0x31, 0xc0, 0xc3,
            ],
        ),
        (
            X86_SYSTEMS_ATOMIC_LOAD_SYMBOL,
            &[
                0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, 0x75, 0x04, 0x48, 0x8b, 0x07, 0xc3, 0x31,
                0xc0, 0xc3,
            ],
        ),
    ] {
        let symbol = file.symbol_by_name(name).unwrap();
        assert_eq!(symbol.size(), expected.len() as u64);
        let section = file
            .section_by_index(symbol.section_index().unwrap())
            .unwrap();
        let data = section.data().unwrap();
        let start = usize::try_from(symbol.address()).unwrap();
        assert_eq!(&data[start..start + expected.len()], expected);
    }
    for (name, read) in [
        (X86_SYSTEMS_PCI_REGION_READ_SYMBOL, true),
        (X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL, true),
        (X86_SYSTEMS_PCI_REGION_WRITE_SYMBOL, false),
    ] {
        let symbol = file.symbol_by_name(name).unwrap();
        let section = file
            .section_by_index(symbol.section_index().unwrap())
            .unwrap();
        let data = section.data().unwrap();
        let start = usize::try_from(symbol.address()).unwrap();
        let end = start + usize::try_from(symbol.size()).unwrap();
        let bytes = &data[start..end];
        assert_eq!(
            bytes
                .windows(5)
                .filter(|window| *window == [0x66, 0xba, 0xf8, 0x0c, 0xef])
                .count(),
            7,
            "gateway access must select BAR, offset, width, data, and three rollback fields"
        );
        assert_eq!(
            bytes
                .windows(4)
                .filter(|window| *window == [0x66, 0xba, 0xfc, 0x0c])
                .count(),
            7,
            "gateway access must address BAR, offset, width, data, and three rollback fields"
        );
        assert_eq!(
            bytes
                .windows(6)
                .filter(|window| *window == [0x31, 0xc0, 0xef, 0xed, 0x85, 0xc0])
                .count(),
            3,
            "gateway access must restore and verify the discovered zero BAR, offset, and length fields"
        );
        if read {
            assert!(
                bytes
                    .windows(4)
                    .any(|window| window == [0xed, 0x44, 0x39, 0xc0])
            );
            assert!(
                bytes
                    .windows(4)
                    .any(|window| window == [0xec, 0x44, 0x38, 0xc0])
            );
            assert!(
                bytes
                    .windows(6)
                    .any(|window| window == [0x66, 0xed, 0x66, 0x44, 0x39, 0xc0])
            );
        } else {
            assert!(
                bytes
                    .windows(4)
                    .any(|window| window == [0x44, 0x89, 0xc0, 0xef])
            );
            assert!(
                bytes
                    .windows(4)
                    .any(|window| window == [0x44, 0x89, 0xc0, 0xee])
            );
            assert!(
                bytes
                    .windows(5)
                    .any(|window| window == [0x44, 0x89, 0xc0, 0x66, 0xef])
            );
        }
    }
    let eventual = file
        .symbol_by_name(X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL)
        .unwrap();
    let section = file
        .section_by_index(eventual.section_index().unwrap())
        .unwrap();
    let data = section.data().unwrap();
    let start = usize::try_from(eventual.address()).unwrap();
    let end = start + usize::try_from(eventual.size()).unwrap();
    let bytes = &data[start..end];
    assert_eq!(
        bytes
            .windows(6)
            .filter(|window| *window == [0x41, 0xbb, 0x40, 0x42, 0x0f, 0x00])
            .count(),
        1,
        "eventual read must install the exact 1,000,000-attempt bound"
    );
    assert_eq!(
        bytes
            .windows(3)
            .filter(|window| *window == [0x41, 0xff, 0xcb])
            .count(),
        3,
        "each width-specific eventual-read path must consume one attempt per mismatch"
    );
    let update = file
        .symbol_by_name(X86_SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD_SYMBOL)
        .unwrap();
    let section = file
        .section_by_index(update.section_index().unwrap())
        .unwrap();
    let data = section.data().unwrap();
    let start = usize::try_from(update.address()).unwrap();
    let end = start + usize::try_from(update.size()).unwrap();
    let bytes = &data[start..end];
    assert_eq!(
        bytes
            .windows(5)
            .filter(|window| *window == [0x66, 0xba, 0xf8, 0x0c, 0xef])
            .count(),
        1,
        "masked word update must select configuration address once"
    );
    assert_eq!(
        bytes
            .windows(2)
            .filter(|window| *window == [0x66, 0xed])
            .count(),
        2,
        "masked word update must read pre-state and post-state exactly"
    );
    assert_eq!(
        bytes
            .windows(2)
            .filter(|window| *window == [0x66, 0xef])
            .count(),
        1,
        "masked word update must write exactly one command word"
    );
    assert!(
        bytes
            .windows(4)
            .any(|window| window == [0x66, 0x44, 0x01, 0xc2])
    );
    let dma_create = file
        .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_CREATE_SYMBOL)
        .unwrap();
    let section = file
        .section_by_index(dma_create.section_index().unwrap())
        .unwrap();
    let data = section.data().unwrap();
    let start = usize::try_from(dma_create.address()).unwrap();
    let end = start + usize::try_from(dma_create.size()).unwrap();
    let bytes = &data[start..end];
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xff, 0x01]));
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xff, 0x02]));
    assert_eq!(bytes.iter().filter(|byte| **byte == 0xc3).count(), 2);
    assert!(!bytes.iter().any(|byte| matches!(*byte, 0xec..=0xef)));
    let bind = file
        .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_BIND_REFERENCE_SYMBOL)
        .unwrap();
    let start = usize::try_from(bind.address()).unwrap();
    let end = start + usize::try_from(bind.size()).unwrap();
    let bytes = &data[start..end];
    assert!(
        bytes
            .windows(4)
            .any(|window| window == [0x48, 0x89, 0x14, 0x37])
    );
    assert!(
        bytes
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0xf8, 0x0f, 0x00, 0x00])
    );
    let store = file
        .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_STORE_INTEGER_SYMBOL)
        .unwrap();
    let start = usize::try_from(store.address()).unwrap();
    let end = start + usize::try_from(store.size()).unwrap();
    let bytes = &data[start..end];
    assert!(
        bytes
            .windows(4)
            .any(|window| window == [0x66, 0x89, 0x0c, 0x37])
    );
    assert!(bytes.windows(3).any(|window| window == [0x89, 0x0c, 0x37]));
    assert!(
        bytes
            .windows(4)
            .any(|window| window == [0x48, 0x89, 0x0c, 0x37])
    );
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xfa, 0x08]));
    let load = file
        .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL)
        .unwrap();
    let start = usize::try_from(load.address()).unwrap();
    let end = start + usize::try_from(load.size()).unwrap();
    let bytes = &data[start..end];
    assert!(
        bytes
            .windows(7)
            .any(|window| window == [0x48, 0x8b, 0x04, 0x37, 0x48, 0x39, 0xc8])
    );
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xfa, 0x08]));
    let prepare = file
        .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_PREPARE_DEVICE_SYMBOL)
        .unwrap();
    let start = usize::try_from(prepare.address()).unwrap();
    let end = start + usize::try_from(prepare.size()).unwrap();
    let bytes = &data[start..end];
    assert!(
        bytes
            .windows(7)
            .any(|window| window == [0x48, 0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00])
    );
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xfe, 0x02]));
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xfe, 0x01]));
    assert!(
        !bytes
            .windows(4)
            .any(|window| window == [0x48, 0x89, 0x14, 0x37])
    );
    let complete = file
        .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_COMPLETE_CPU_SYMBOL)
        .unwrap();
    let start = usize::try_from(complete.address()).unwrap();
    let end = start + usize::try_from(complete.size()).unwrap();
    let bytes = &data[start..end];
    assert!(
        bytes
            .windows(7)
            .any(|window| window == [0x48, 0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00])
    );
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xfe, 0x01]));
    assert!(
        !bytes
            .windows(4)
            .any(|window| window == [0x48, 0x89, 0x14, 0x37])
    );
    let load = file
        .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL)
        .unwrap();
    let start = usize::try_from(load.address()).unwrap();
    let end = start + usize::try_from(load.size()).unwrap();
    let bytes = &data[start..end];
    assert!(
        bytes
            .windows(6)
            .any(|window| window == [0x0f, 0xb7, 0x04, 0x37, 0x39, 0xc8])
    );
    assert!(
        bytes
            .windows(5)
            .any(|window| window == [0x8b, 0x04, 0x37, 0x39, 0xc8])
    );
    let capture = file
        .symbol_by_name(X86_SYSTEMS_ENTROPY_SAMPLE_CAPTURE_SYMBOL)
        .unwrap();
    let start = usize::try_from(capture.address()).unwrap();
    let end = start + usize::try_from(capture.size()).unwrap();
    let bytes = &data[start..end];
    for validation in [
        &[0x83, 0xfa, 0x01][..],
        &[0x83, 0xf9, 0x02],
        &[0x41, 0x83, 0xf8, 0x20],
        &[0x45, 0x85, 0xc9],
    ] {
        assert!(
            bytes
                .windows(validation.len())
                .any(|window| window == validation)
        );
    }
    for (load, store, verify) in [
        (
            &[0x48, 0x8b, 0x07][..],
            &[0x48, 0x89, 0x06][..],
            &[0x48, 0x3b, 0x06][..],
        ),
        (
            &[0x48, 0x8b, 0x47, 8][..],
            &[0x48, 0x89, 0x46, 8][..],
            &[0x48, 0x3b, 0x46, 8][..],
        ),
        (
            &[0x48, 0x8b, 0x47, 16][..],
            &[0x48, 0x89, 0x46, 16][..],
            &[0x48, 0x3b, 0x46, 16][..],
        ),
        (
            &[0x48, 0x8b, 0x47, 24][..],
            &[0x48, 0x89, 0x46, 24][..],
            &[0x48, 0x3b, 0x46, 24][..],
        ),
    ] {
        for sequence in [load, store, verify] {
            assert_eq!(
                bytes
                    .windows(sequence.len())
                    .filter(|window| *window == sequence)
                    .count(),
                1,
                "capture must load, copy, and verify each qword exactly once"
            );
        }
    }
    let mix = file
        .symbol_by_name(X86_SYSTEMS_ENTROPY_POOL_MIX_SYMBOL)
        .unwrap();
    let start = usize::try_from(mix.address()).unwrap();
    let end = start + usize::try_from(mix.size()).unwrap();
    let bytes = &data[start..end];
    assert!(bytes.len() > 5_000, "scalar BLAKE2s lowering is complete");
    assert!(
        bytes
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xec, 0x80, 0, 0, 0])
    );
    for rotation in [16_u8, 12, 8, 7] {
        assert_eq!(
            bytes
                .windows(4)
                .filter(|window| *window == [0x41, 0xc1, 0xc8, rotation])
                .count()
                + bytes
                    .windows(3)
                    .filter(|window| *window == [0xc1, 0xc9, rotation])
                    .count(),
            80,
            "each BLAKE2s G half-round uses the selected rotation"
        );
    }
    for value in [0x6b08_e647_u32, 0x7ef3_b6ff, 0x1cae_9f75] {
        assert!(bytes.windows(4).any(|window| window == value.to_le_bytes()));
    }
    for offset in [0_u8, 8, 16, 24] {
        assert!(
            bytes
                .windows(4)
                .any(|window| window == [0x49, 0x89, 0x42, offset]),
            "the consumed sample qword is wiped"
        );
        assert!(
            bytes
                .windows(4)
                .any(|window| window == [0x49, 0x0b, 0x42, offset]),
            "the consumed sample qword wipe is verified"
        );
    }
    #[cfg(all(target_arch = "x86_64", target_family = "unix"))]
    check_entropy_mix_against_independent_reference(bytes);
    let initialize = file
        .symbol_by_name(X86_SYSTEMS_ENTROPY_GENERATOR_INITIALIZE_SYMBOL)
        .unwrap();
    let start = usize::try_from(initialize.address()).unwrap();
    let end = start + usize::try_from(initialize.size()).unwrap();
    let bytes = &data[start..end];
    for offset in [0_u8, 8, 16, 24] {
        let load = if offset == 0 {
            vec![0x48, 0x8b, 0x07]
        } else {
            vec![0x48, 0x8b, 0x47, offset]
        };
        let publish = if offset == 0 {
            vec![0x48, 0x89, 0x06]
        } else {
            vec![0x48, 0x89, 0x46, offset]
        };
        let wipe = if offset == 0 {
            vec![0x48, 0x89, 0x07]
        } else {
            vec![0x48, 0x89, 0x47, offset]
        };
        for sequence in [load, publish, wipe] {
            assert!(
                bytes
                    .windows(sequence.len())
                    .any(|window| window == sequence)
            );
        }
    }
    let extract = file
        .symbol_by_name(X86_SYSTEMS_ENTROPY_GENERATOR_EXTRACT_SYMBOL)
        .unwrap();
    let start = usize::try_from(extract.address()).unwrap();
    let end = start + usize::try_from(extract.size()).unwrap();
    let bytes = &data[start..end];
    assert!(bytes.len() > 4_000, "scalar ChaCha20 lowering is complete");
    for value in [0x6170_7865_u32, 0x3320_646e, 0x7962_2d32, 0x6b20_6574] {
        assert!(bytes.windows(4).any(|window| window == value.to_le_bytes()));
    }
    for rotation in [16_u8, 8] {
        assert_eq!(
            bytes
                .windows(4)
                .filter(|window| *window == [0x41, 0xc1, 0xc0, rotation])
                .count(),
            80
        );
    }
    for rotation in [12_u8, 7] {
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0xc1, 0xc1, rotation])
                .count(),
            80
        );
    }
    for offset in [0_u8, 4, 8, 12, 16, 20, 24, 28] {
        assert!(
            bytes
                .windows(4)
                .any(|window| window == [0x41, 0x89, 0x42, offset]),
            "the successor key word is published"
        );
        assert!(
            bytes
                .windows(4)
                .any(|window| window == [0x41, 0x89, 0x43, offset]),
            "the opaque output word is published"
        );
    }
    #[cfg(all(target_arch = "x86_64", target_family = "unix"))]
    check_entropy_extract_against_independent_reference(bytes);
    let publish = file
        .symbol_by_name(X86_SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE_SYMBOL)
        .unwrap();
    let start = usize::try_from(publish.address()).unwrap();
    let end = start + usize::try_from(publish.size()).unwrap();
    let bytes = &data[start..end];
    assert!(
        bytes
            .windows(6)
            .any(|window| window == [0x81, 0xff, 0x84, 0x28, 0x00, 0x80])
    );
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xfa, 0x20]));
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xfa, 0x28]));
    assert!(bytes.windows(3).any(|window| window == [0x83, 0xfa, 0x30]));
    assert_eq!(
        bytes
            .windows(4)
            .filter(|window| *window == [0x66, 0xba, 0xfc, 0x0c])
            .count(),
        6
    );
    assert!(
        bytes
            .windows(5)
            .any(|window| window == [0x48, 0xc1, 0xe8, 0x20, 0xef])
    );
}
