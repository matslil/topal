fn audit_provider_process_entry(file: &object::File<'_>) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let process_enter = symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL);
    let process_restore =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY_SYMBOL);
    assert!(
        process_enter
            .windows(8)
            .any(|window| window == 0x004a_6580_u64.to_le_bytes()),
        "static process entry must derive the sealed ELF entry"
    );
    assert!(
        process_enter
            .windows(8)
            .any(|window| window == 0x007f_fe90_u64.to_le_bytes()),
        "static process entry must derive the external executable's final initial stack"
    );
    assert!(
        process_enter
            .windows(8)
            .any(|window| window == 0x007f_ffa0_u64.to_le_bytes()),
        "static process entry must retain the clean-room fixture's established stack"
    );
    assert!(
        process_enter
            .windows(8)
            .any(|window| window == [0x41, 0x80, 0xba, 0xa0, 0x01, 0, 0, 2]),
        "stack selection must be gated by the verified private header variant"
    );
    assert!(
        process_enter
            .windows(2)
            .any(|window| window == [0x48, 0xcf]),
        "static process entry must cross the native privilege boundary"
    );
    let process_fixture_image = qualified_x86_64_static_user_image_fixture();
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-BUSYBOX-TRUE-EXECVE-001:
    // retained pristine bytes and the full heap reserve must have distinct,
    // sufficiently sized provider storage, beyond the user mappings.
    let snapshot = file
        .symbol_by_name(X86_SYSTEMS_STATIC_PROCESS_EXEC_SNAPSHOT_SYMBOL)
        .unwrap();
    let heap = file
        .symbol_by_name(X86_SYSTEMS_STATIC_PROCESS_BRK_RESERVE_SYMBOL)
        .unwrap();
    assert_eq!(snapshot.size(), 0xb000);
    assert_eq!(heap.size(), 34 * 4096);
    assert!(snapshot.address() >= heap.address() + heap.size());
    assert_eq!(
        &process_fixture_image[0x0a_6580..0x0a_6585],
        &[0xe9, 0x7b, 0xfb, 0xff, 0xff]
    );
    let process_fixture = &process_fixture_image[0x0a_6100..0x0a_6580];
    let shell_continuation = &process_fixture_image[0x0a_7000..0x0a_a000];
    for sequence in [
        &[0xb8, 0x0c, 0, 0, 0, 0x31, 0xff, 0x0f, 0x05][..],
        &[
            0xbf, 0x02, 0x10, 0, 0, 0xbe, 0x00, 0x74, 0x5f, 0, 0xb8, 0x9e, 0, 0, 0,
        ][..],
        &[0xbf, 0xd0, 0x76, 0x5f, 0, 0xb8, 0xda, 0, 0, 0][..],
        &[0xbf, 0xe0, 0x76, 0x5f, 0, 0xbe, 0x18, 0, 0, 0][..],
        &[0xb8, 0x11, 0x01, 0, 0, 0x0f, 0x05][..],
        &[
            0xbf, 0x00, 0x73, 0x5f, 0, 0xbe, 0x21, 0, 0, 0, 0x31, 0xd2, 0x41, 0xba, 0x53, 0x30,
            0x05, 0x53, 0xb8, 0x4e, 0x01, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 0xda,
        ][..],
        &[
            0x48, 0x81, 0xec, 0x40, 0x01, 0, 0, 0x31, 0xff, 0xbe, 3, 0, 0, 0, 0x31, 0xd2, 0x49,
            0x89, 0xe2, 0xb8, 0x2e, 0x01, 0, 0, 0x0f, 0x05,
        ][..],
        &[
            0xbf, 0x9c, 0xff, 0xff, 0xff, 0xbe, 0xda, 0xfd, 0x5b, 0, 0xba, 0, 0xe0, 0x5f, 0, 0x41,
            0xba, 0, 0x10, 0, 0, 0xb8, 0x0b, 0x01, 0, 0, 0x0f, 0x05,
        ][..],
        &[
            0xbf, 0x68, 0x5e, 0x5f, 0, 0xbe, 8, 0, 0, 0, 0xba, 1, 0, 0, 0, 0xb8, 0x3e, 0x01, 0, 0,
            0x0f, 0x05,
        ][..],
        &[
            0xbf, 0x70, 0x5e, 0x5f, 0, 0xbe, 8, 0, 0, 0, 0xba, 1, 0, 0, 0, 0xb8, 0x3e, 0x01, 0, 0,
            0x0f, 0x05,
        ][..],
        &[
            0xb8, 0x0c, 0, 0, 0, 0x31, 0xff, 0x0f, 0x05, 0x48, 0x3d, 0x80, 0x7d, 0x5f, 0, 0x75,
            0xfe,
        ][..],
        &[
            0xbf, 0x80, 0x8d, 0x61, 0, 0xb8, 0x0c, 0, 0, 0, 0x0f, 0x05, 0x48, 0x3d, 0x80, 0x8d,
            0x61, 0, 0x75, 0xfe,
        ][..],
        &[
            0xbf, 0x00, 0x90, 0x61, 0, 0xb8, 0x0c, 0, 0, 0, 0x0f, 0x05, 0x48, 0x3d, 0x00, 0x90,
            0x61, 0, 0x75, 0xfe,
        ][..],
        &[
            0xbf, 0x00, 0x50, 0x5e, 0, 0xbe, 0x00, 0x80, 0, 0, 0xba, 1, 0, 0, 0, 0xb8, 0x0a, 0, 0,
            0, 0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe,
        ][..],
        &[
            0x48, 0x81, 0xec, 0x40, 0x01, 0, 0, 0xbf, 0x10, 0, 0, 0, 0x48, 0x89, 0xe6, 0xb8, 0x9d,
            0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe,
        ][..],
        &[
            0xb8, 0x66, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe,
        ][..],
        &[
            0x48, 0x81, 0xec, 0x20, 0x02, 0, 0, 0xbf, 0x9c, 0xff, 0xff, 0xff, 0xbe, 0x00, 0x69,
            0x4a, 0x00, 0x48, 0x89, 0xe2, 0x45, 0x31, 0xd2, 0xb8, 0x06, 0x01, 0, 0, 0x0f, 0x05,
            0x48, 0x83, 0xf8, 0xfe,
        ][..],
    ] {
        assert!(
            process_fixture
                .windows(sequence.len())
                .any(|window| window == sequence)
        );
    }
    for number in [104_u8, 106, 105] {
        assert!(
            process_fixture.windows(12).any(|window| window
                == [
                    0xb8, number, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe
                ]),
            "clean process fixture must require every credential-tail result"
        );
    }
    for number in [106_u8, 105] {
        assert!(
            process_fixture.windows(14).any(|window| window
                == [
                    0x31, 0xff, 0xb8, number, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe
                ]),
            "clean process fixture must select only the sealed zero credential"
        );
    }
    assert!(
        process_fixture.windows(27).any(|window| window
            == [
                0x48, 0x81, 0xec, 0x40, 0x02, 0, 0, 0xc6, 0x04, 0x24, 0x0a, 0xbf, 1, 0, 0, 0, 0x48,
                0x89, 0xe6, 0xba, 1, 0, 0, 0, 0xb8, 1, 0
            ])
            && process_fixture.windows(12).any(|window| window
                == [
                    0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 1, 0x75, 0xfe, 0xc6, 0x04
                ]),
        "clean process fixture must stage and require its exact one-byte stdout result"
    );
    assert!(
        process_fixture.windows(13).any(|window| window
            == [
                0xb8, 0x27, 0, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 1, 0x75, 0xfe
            ]),
        "clean process fixture must require sealed PID one"
    );
    assert!(
        process_fixture.windows(19).any(|window| window
            == [
                0xbf, 17, 0, 0, 0, 0x48, 0x89, 0xe6, 0x31, 0xd2, 0x41, 0xba, 8, 0, 0, 0, 0xb8, 13,
                0
            ])
            && process_fixture.windows(10).any(
                |window| window == [0x48, 0xb8, 0xff, 0xff, 0xff, 0x7f, 0xfe, 0xff, 0xff, 0xff]
            )
            && process_fixture
                .windows(9)
                .any(|window| window == [0x48, 0xc7, 0x44, 0x24, 0x08, 0, 0, 0, 4]),
        "clean process fixture must install the exact child disposition"
    );
    assert!(
        process_fixture.windows(12).any(|window| window
            == [
                0xb8, 0x6e, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe
            ]),
        "clean process fixture must require the sealed no-parent result"
    );
    assert!(
        process_fixture
            .windows(5)
            .any(|window| window == [0xe9, 0x7a, 0x0b, 0, 0])
            && shell_continuation.windows(15).any(|window| {
                window
                    == [
                        0x31, 0xff, 0xbe, 0x01, 0x54, 0, 0, 0x48, 0x89, 0xe2, 0xb8, 0x10, 0, 0, 0,
                    ]
            }),
        "clean process fixture must enter its sealed island and issue exact TCGETS"
    );
    assert!(
        process_enter
            .windows(11)
            .any(|window| { window == [0x65, 0x80, 0x3c, 0x25, 0x78, 0x03, 0, 0, 25, 0x0f, 0x87] })
            && shell_continuation
                .windows(10)
                .any(|window| window == [0x48, 0x89, 0xe7, 0xb8, 0x3f, 0, 0, 0, 0x0f, 0x05])
            && shell_continuation.windows(15).any(|window| {
                window
                    == [
                        0xbf, 0x60, 0x94, 0x5f, 0, 0xbe, 0, 0x10, 0, 0, 0xb8, 0x4f, 0, 0, 0,
                    ]
            }),
        "clean process fixture must request deterministic identity and sealed root cwd"
    );
    for signal in [2_u8, 3, 15] {
        assert!(
            shell_continuation
                .windows(10)
                .any(|window| { window == [0xbf, signal, 0, 0, 0, 0x31, 0xf6, 0x48, 0x89, 0xe2] }),
            "clean process fixture must query every exact default disposition"
        );
    }
    assert!(
        shell_continuation
            .windows(14)
            .any(|window| window == [0xbf, 1, 0, 0, 0, 0x48, 0x89, 0xe6, 0xb8, 5, 0, 0, 0, 0x0f])
            && shell_continuation
                .windows(10)
                .any(|window| window == [0x48, 0x81, 0x7c, 0x24, 24, 0x80, 0x21, 0, 0, 0x75])
            && shell_continuation
                .windows(10)
                .any(|window| window == [0x48, 0x81, 0x7c, 0x24, 40, 0, 0x44, 0, 0, 0x75])
            && shell_continuation
                .windows(10)
                .any(|window| window == [0x48, 0x81, 0x7c, 0x24, 56, 0, 0x10, 0, 0, 0x75]),
        "clean process fixture must validate the complete bounded console stat"
    );
    for offset in (472_u32..616).step_by(8) {
        assert!(
            process_enter.windows(10).any(|window| {
                window[..5] == [0x65, 0x48, 0x83, 0x3c, 0x25]
                    && window[5..9] == offset.to_le_bytes()
                    && window[9] == 0
            }),
            "static shell console fstat must inspect every private staging qword"
        );
    }
    for (offset, value) in [
        (480_u32, 1_u32),
        (488, 1),
        (496, 0x2180),
        (512, 0x4400),
        (528, 4096),
    ] {
        assert!(
            process_enter.windows(13).any(|window| {
                window[..5] == [0x65, 0x48, 0xc7, 0x04, 0x25]
                    && window[5..9] == offset.to_le_bytes()
                    && window[9..13] == value.to_le_bytes()
            }),
            "static shell console fstat must stage each nonzero native field"
        );
    }
    assert!(
        process_enter
            .windows(9)
            .any(|window| { window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0x02, 0, 0, 1] }),
        "static shell console fstat must publish its private observation"
    );
    assert_eq!(
        &process_fixture_image[0x0a_6912..0x0a_691b],
        b"/dev/tty\0",
        "clean process fixture must retain the exact immutable /dev/tty candidate"
    );
    assert!(
        shell_continuation.windows(31).any(|window| window
            == [
                0xbf, 0x9c, 0xff, 0xff, 0xff, 0xbe, 0x12, 0x69, 0x4a, 0, 0xba, 2, 0, 0, 0, 0x45,
                0x31, 0xd2, 0xb8, 1, 1, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 0xfe, 0x75, 0xfe,
            ]),
        "clean process fixture must issue exact /dev/tty openat and require ENOENT"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x12, 0x69, 0x4a, 0])
            && process_enter.windows(10).any(
                |window| window == [0x48, 0xb8, b'/', b'd', b'e', b'v', b'/', b't', b't', b'y']
            )
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x69, 0x02, 0, 0, 1]),
        "static shell /dev/tty open miss must validate and publish the exact observation"
    );
    let standard_terminal_probes = [
        0xc6, 0x04, 0x24, 0x6d, 0xbf, 2, 0, 0, 0, 0xbe, 0x01, 0x54, 0, 0, 0x48, 0x89, 0xe2, 0xb8,
        0x10, 0, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 0xe7, 0x75, 0xfe, 0x80, 0x3c, 0x24, 0x6d,
        0x75, 0xfe, 0xbf, 1, 0, 0, 0, 0xbe, 0x01, 0x54, 0, 0, 0x48, 0x89, 0xe2, 0xb8, 0x10, 0, 0,
        0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 0xe7, 0x75, 0xfe, 0x80, 0x3c, 0x24, 0x6d, 0x75, 0xfe,
        0x31, 0xff, 0xbe, 0x01, 0x54, 0, 0, 0x48, 0x89, 0xe2, 0xb8, 0x10, 0, 0, 0, 0x0f, 0x05,
        0x48, 0x83, 0xf8, 0xe7, 0x75, 0xfe, 0x80, 0x3c, 0x24, 0x6d, 0x75, 0xfe, 0xc6, 0x04, 0x24,
        0, 0x80, 0x3c, 0x24, 0, 0x75, 0xfe,
    ];
    assert!(
        shell_continuation
            .windows(standard_terminal_probes.len())
            .any(|window| window == standard_terminal_probes),
        "clean process fixture must prove ordered stderr, stdout, and stdin TCGETS misses"
    );
    for observation in [0x0000_0101_u32, 0x0001_0101, 0x0002_0101] {
        assert!(
            process_enter.windows(6).any(|window| {
                window[..2] == [0x48, 0x3d] && window[2..6] == observation.to_le_bytes()
            }),
            "static shell terminal probes must require every exact predecessor observation"
        );
    }
    assert!(
        process_enter
            .windows(8)
            .any(|window| window == [0x65, 0xfe, 0x04, 0x25, 0x6a, 0x02, 0, 0]),
        "static shell terminal probes must publish their ordered private count"
    );
    assert!(
        shell_continuation.windows(28).any(|window| window
            == [
                0xc7, 0x04, 0x24, b's', b'h', b':', b' ', 0xbf, 2, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba,
                4, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0x0f, 0x05, 0x48,
            ])
            && shell_continuation.windows(24).any(|window| window
                == [
                    0xbf, 2, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba, 40, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0x0f,
                    0x05, 0x48, 0x83, 0xf8, 40,
                ])
            && shell_continuation.windows(28).any(|window| window
                == [
                    0xc6, 0x04, 0x24, b'\n', 0xbf, 2, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba, 1, 0, 0, 0,
                    0xb8, 1, 0, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 1,
                ]),
        "clean process fixture must issue every exact terminal-warning segment"
    );
    for (register, expected) in [
        (0xc0_u8, *b"can't ac"),
        (0xc1, *b"cess tty"),
        (0xc2, *b"; job co"),
        (0xc3, *b"ntrol tu"),
        (0xc4, *b"rned off"),
    ] {
        let mut comparison = vec![0x48, 0xb8];
        comparison.extend_from_slice(&expected);
        comparison.extend_from_slice(&[0x49, 0x39, register]);
        assert!(
            process_enter
                .windows(comparison.len())
                .any(|window| window == comparison.as_slice()),
            "static shell warning must stage and validate every eight-byte phrase word"
        );
    }
    for observation in [0x0003_0101_u32, 0x0103_0101, 0x0203_0101] {
        assert!(
            process_enter.windows(6).any(|window| {
                window[..2] == [0x48, 0x3d] && window[2..6] == observation.to_le_bytes()
            }),
            "static shell warning must require every exact predecessor observation"
        );
    }
    assert!(
        process_enter
            .windows(8)
            .any(|window| window == [0x65, 0xfe, 0x04, 0x25, 0x6b, 0x02, 0, 0]),
        "static shell warning must publish its ordered private count"
    );
    assert_eq!(
        &process_fixture_image[0x0a_6920..0x0a_6935],
        b"/var/spool/mail/root\0",
        "clean process fixture must retain the exact immutable MAILPATH candidate"
    );
    assert!(
        shell_continuation.windows(31).any(|window| window
            == [
                0xbf, 0x9c, 0xff, 0xff, 0xff, 0xbe, 0x20, 0x69, 0x4a, 0, 0x48, 0x89, 0xe2, 0x45,
                0x31, 0xd2, 0xb8, 0x06, 0x01, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 0xfe, 0x75, 0xfe,
                0x48, 0xb8,
            ])
            && shell_continuation
                .windows(12)
                .filter(|window| {
                    *window == [0xb9, 18, 0, 0, 0, 0x48, 0x89, 0xe7, 0xfc, 0xf3, 0x48, 0xaf]
                })
                .count()
                == 9,
        "clean process fixture must issue the exact MAILPATH call and prove every complete startup, root, named-child, opened-child, initial-file, executable, and nonempty metadata candidate"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x20, 0x69, 0x4a, 0])
            && process_enter.windows(10).any(
                |window| window == [0x48, 0xb8, b'/', b'v', b'a', b'r', b'/', b's', b'p', b'o']
            )
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x6c, 0x02, 0, 0, 1]),
        "static shell MAILPATH miss must validate and publish the exact observation"
    );
    let complete_mailpath_observation = [
        0x48, 0xb8, 0x01, 0x01, 0x03, 0x03, 0x01, 0, 0, 0, 0x65, 0x48, 0x39, 0x04, 0x25, 0x68,
        0x02, 0, 0,
    ];
    assert_eq!(
        process_enter
            .windows(complete_mailpath_observation.len())
            .filter(|window| *window == complete_mailpath_observation)
            .count(),
        2,
        "banner dispatch and handling must require the complete MAILPATH observation"
    );
    assert!(
        shell_continuation.windows(26).any(|window| window
            == [
                0xbf, 1, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba, 109, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0x0f,
                0x05, 0x48, 0x83, 0xf8, 109, 0x75, 0xfe,
            ])
            && shell_continuation
                .windows(10)
                .any(|window| window
                    == [0x48, 0xb8, b'\n', b'\n', b'B', b'u', b's', b'y', b'B', b'o'])
            && shell_continuation.windows(15).any(|window| window
                == [
                    0x48, 0xb8, b'd', b's', b'.', b'\n', b'\n', 0, 0, 0, 0x48, 0x89, 0x44, 0x24,
                    0x68,
                ]),
        "clean process fixture must stage and issue the exact 109-byte shell banner"
    );
    assert!(
        process_enter.windows(20).any(|window| window
            == [
                0x45, 0x31, 0xd2, 0x42, 0x8a, 0x04, 0x16, 0x65, 0x41, 0x88, 0x82, 0x70, 0x02, 0, 0,
                0x49, 0xff, 0xc2, 0x49, 0x83,
            ])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x6d, 0x02, 0, 0, 1]),
        "static shell banner must stage completely and publish its private completion fact"
    );
    const EXPECTED_SHELL_BANNER: &[u8; 109] =
            b"\n\nBusyBox v1.37.0 (Debian 1:1.37.0-6+b9) built-in shell (ash)\nEnter 'help' for a list of built-in commands.\n\n";
    let mut padded_banner = [0_u8; 112];
    padded_banner[..EXPECTED_SHELL_BANNER.len()].copy_from_slice(EXPECTED_SHELL_BANNER);
    for (index, chunk) in padded_banner.chunks_exact(8).enumerate() {
        let mut comparison = vec![0x48, 0xb8];
        comparison.extend_from_slice(chunk);
        comparison.extend_from_slice(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        comparison.extend_from_slice(&(624_u32 + (index as u32 * 8)).to_le_bytes());
        assert!(
            process_enter
                .windows(comparison.len())
                .any(|window| window == comparison.as_slice()),
            "static shell banner must validate every private staging word"
        );
    }
    let complete_banner_observation = [
        0x48, 0xb8, 0x01, 0x01, 0x03, 0x03, 0x01, 0x01, 0, 0, 0x65, 0x48, 0x39, 0x04, 0x25, 0x68,
        0x02, 0, 0,
    ];
    assert_eq!(
        process_enter
            .windows(complete_banner_observation.len())
            .filter(|window| *window == complete_banner_observation)
            .count(),
        2,
        "post-banner probe dispatch and handling must require the complete banner observation"
    );
    assert!(
        shell_continuation.windows(43).any(|window| window
            == [
                0xc6, 0x04, 0x24, 0x6d, 0x31, 0xff, 0xbe, 0x01, 0x54, 0, 0, 0x48, 0x89, 0xe2, 0xb8,
                0x10, 0, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 0xe7, 0x75, 0xfe, 0x80, 0x3c, 0x24,
                0x6d, 0x75, 0xfe, 0xc6, 0x04, 0x24, 0, 0x80, 0x3c, 0x24, 0, 0x75, 0xfe,
            ]),
        "clean process fixture must issue and erase the exact post-banner stdin TCGETS miss"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x6e, 0x02, 0, 0, 1]),
        "post-banner terminal probe must publish its private completion fact"
    );
    let complete_post_banner_probe_observation = [
        0x48, 0xb8, 0x01, 0x01, 0x03, 0x03, 0x01, 0x01, 0x01, 0, 0x65, 0x48, 0x39, 0x04, 0x25,
        0x68, 0x02, 0, 0,
    ];
    assert_eq!(
        process_enter
            .windows(complete_post_banner_probe_observation.len())
            .filter(|window| *window == complete_post_banner_probe_observation)
            .count(),
        1,
        "post-banner getcwd handling must require the complete terminal-probe observation"
    );
    assert!(
        shell_continuation.windows(66).any(|window| window
            == [
                0x66, 0xc7, 0x04, 0x25, 0x60, 0x94, 0x5f, 0, 0xa5, 0xa5, 0xbf, 0x60, 0x94, 0x5f, 0,
                0xbe, 0, 0x10, 0, 0, 0xb8, 0x4f, 0, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 2, 0x75,
                0xfe, 0x66, 0x81, 0x3c, 0x25, 0x60, 0x94, 0x5f, 0, 0x2f, 0, 0x75, 0xfe, 0x66, 0xc7,
                0x04, 0x25, 0x60, 0x94, 0x5f, 0, 0, 0, 0x66, 0x83, 0x3c, 0x25, 0x60, 0x94, 0x5f, 0,
                0, 0x75, 0xfe,
            ]),
        "clean process fixture must issue, validate, erase, and prove the post-banner root getcwd"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x6f, 0x02, 0, 0, 1])
            && process_enter
                .windows(10)
                .any(|window| window == [0x66, 0xc7, 0x04, 0x25, 0x60, 0x94, 0x5f, 0, 0x2f, 0]),
        "post-banner getcwd must publish the sealed root and its private completion fact"
    );
    let complete_post_banner_getcwd_observation = [
        0x48, 0xb8, 0x01, 0x01, 0x03, 0x03, 0x01, 0x01, 0x01, 0x01, 0x65, 0x48, 0x39, 0x04, 0x25,
        0x68, 0x02, 0, 0,
    ];
    assert_eq!(
        process_enter
            .windows(complete_post_banner_getcwd_observation.len())
            .filter(|window| *window == complete_post_banner_getcwd_observation)
            .count(),
        20,
        "effective identity, profile open, profile metadata, profile EOF, profile close, all prompts, command-time profile dispatches, sealed input publications including clean replacement validation, and rollback must require the complete post-banner getcwd observation"
    );
    assert!(
        shell_continuation.windows(12).any(|window| window
            == [
                0xb8, 0x6b, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe
            ])
            && process_enter
                .windows(5)
                .any(|window| window == [0x3d, 0x6b, 0, 0, 0]),
        "clean fixture and dispatcher must issue the exact post-banner geteuid"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe0, 0x02, 0, 0, 1])
            && process_enter
                .windows(9)
                .filter(|window| { *window == [0x65, 0x80, 0x3c, 0x25, 0xe0, 0x02, 0, 0, 1] })
                .count()
                == 9
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe0, 0x02, 0, 0, 0]),
        "post-banner effective identity must publish, gate successors, and clear its private fact"
    );
    assert_eq!(
        &process_fixture_image[0x0a_6938..0x0a_6945],
        b"/etc/profile\0",
        "clean process fixture must retain the exact immutable system-profile path"
    );
    assert!(
        shell_continuation.windows(28).any(|window| window
            == [
                0xbf, 0x9c, 0xff, 0xff, 0xff, 0xbe, 0x38, 0x69, 0x4a, 0, 0x31, 0xd2, 0x45, 0x31,
                0xd2, 0xb8, 0x01, 0x01, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 3, 0x75, 0xfe,
            ]),
        "clean process fixture must issue the exact read-only system-profile open"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe1, 0x02, 0, 0, 1])
            && process_enter
                .windows(9)
                .filter(|window| { *window == [0x65, 0x80, 0x3c, 0x25, 0xe1, 0x02, 0, 0, 1] })
                .count()
                == 14
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe1, 0x02, 0, 0, 0]),
        "profile and busybox opens must publish and gate predecessor observations before transferring the reopened description to the reusable table"
    );
    assert!(
        shell_continuation.windows(20).any(|window| window
            == [
                0xbf, 3, 0, 0, 0, 0x48, 0x89, 0xe6, 0xb8, 5, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0,
                0x75, 0xfe,
            ]),
        "clean process fixture must issue and validate exact descriptor-three profile fstat"
    );
    assert!(
        shell_continuation
            .windows(11)
            .any(|window| window == [0x48, 0x81, 0x7c, 0x24, 24, 0x24, 0x81, 0, 0, 0x75, 0xfe])
            && shell_continuation
                .windows(11)
                .any(|window| window == [0x48, 0x81, 0x7c, 0x24, 56, 0, 0x10, 0, 0, 0x75, 0xfe]),
        "clean process fixture must validate exact immutable regular-file mode and block size"
    );
    assert!(
        process_enter
            .windows(13)
            .any(|window| window == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xf0, 0x02, 0, 0, 2, 0, 0, 0])
            && process_enter.windows(13).any(|window| window
                == [
                    0x65, 0x48, 0xc7, 0x04, 0x25, 0, 0x03, 0, 0, 0x24, 0x81, 0, 0
                ])
            && process_enter.windows(13).any(|window| window
                == [
                    0x65, 0x48, 0xc7, 0x04, 0x25, 0x20, 0x03, 0, 0, 0, 0x10, 0, 0
                ]),
        "profile fstat must privately stage exact identity, mode, and block size"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe2, 0x02, 0, 0, 1])
            && process_enter
                .windows(9)
                .filter(|window| { *window == [0x65, 0x80, 0x3c, 0x25, 0xe2, 0x02, 0, 0, 1] })
                .count()
                == 7
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe2, 0x02, 0, 0, 0]),
        "profile fstat must publish, gate successors, and clear its private completion fact"
    );
    assert!(
        shell_continuation.windows(22).any(|window| window
            == [
                0xbf, 3, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba, 128, 0, 0, 0, 0x31, 0xc0, 0x0f, 0x05,
                0x48, 0x85, 0xc0, 0x75, 0xfe,
            ]),
        "clean process fixture must issue exact descriptor-three profile EOF read"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe3, 0x02, 0, 0, 1])
            && process_enter
                .windows(9)
                .filter(|window| { *window == [0x65, 0x80, 0x3c, 0x25, 0xe3, 0x02, 0, 0, 1] })
                .count()
                == 6
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe3, 0x02, 0, 0, 0]),
        "profile EOF must publish, gate successors, and clear its private completion fact"
    );
    assert!(
        shell_continuation.windows(17).any(|window| window
            == [
                0xbf, 3, 0, 0, 0, 0xb8, 3, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe,
            ]),
        "clean process fixture must explicitly close descriptor three"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe1, 0x02, 0, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0x80, 0x3c, 0x25, 0xe1, 0x02, 0, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe4, 0x02, 0, 0, 1])
            && process_enter
                .windows(9)
                .filter(|window| { *window == [0x65, 0x80, 0x3c, 0x25, 0xe4, 0x02, 0, 0, 1] })
                .count()
                == 5
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe4, 0x02, 0, 0, 0]),
        "profile close must consume the live description, publish closed history, gate successors, and clear that history"
    );
    assert!(
        shell_continuation.windows(48).any(|window| window
            == [
                0xc7, 0x04, 0x24, 0x2f, 0x20, 0x23, 0x20, 0xbf, 2, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba,
                4, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 4, 0x75, 0xfe, 0x48,
                0xc7, 0x04, 0x24, 0, 0, 0, 0, 0x48, 0x83, 0x3c, 0x24, 0, 0x75, 0xfe,
            ]),
        "clean process fixture must publish and erase the exact root prompt"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe5, 0x02, 0, 0, 1])
            && process_enter
                .windows(9)
                .filter(|window| { *window == [0x65, 0x80, 0x3c, 0x25, 0xe5, 0x02, 0, 0, 1] })
                .count()
                == 4
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe5, 0x02, 0, 0, 0])
            && process_enter.windows(12).any(|window| window
                == [
                    0x65, 0x81, 0x3c, 0x25, 0xe8, 0x02, 0, 0, 0x2f, 0x20, 0x23, 0x20,
                ]),
        "prompt handling must validate private staging, publish completion, gate successors, and clear the fact"
    );
    assert_eq!(
        shell_continuation
            .windows(8)
            .filter(|window| { *window == [0x31, 0xff, 0xbe, 0x60, 0x8f, 0x5f, 0, 0xba] })
            .count(),
        5,
        "clean process fixture must issue all five reads into the same checked command buffer"
    );
    assert!(
        shell_continuation
            .windows(10)
            .any(|window| window == [0x48, 0xb8, b'e', b'c', b'h', b'o', b' ', b't', b'o', b'p'])
            && shell_continuation
                .windows(10)
                .any(|window| window == [0x48, 0xb8, b'a', b'l', b'\n', 0, 0, 0, 0, 0])
            && shell_continuation
                .windows(10)
                .any(|window| window == [0x48, 0xb8, b'c', b'd', b' ', b'/', b'\n', 0, 0, 0])
            && shell_continuation.windows(17).any(|window| window
                == [
                    0xbf, 0x60, 0x94, 0x5f, 0, 0xb8, 0x50, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0,
                    0x75, 0xfe,
                ])
            && shell_continuation.windows(10).any(
                |window| window == [0x48, 0xb8, b'e', b'c', b'h', b'o', b' ', b'/', b'*', b';']
            )
            && shell_continuation
                .windows(5)
                .any(|window| window == [0xb8, 0xd9, 0, 0, 0])
            && shell_continuation.windows(19).any(|window| window
                == [
                    0xbf, 1, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba, 5, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0x0f,
                ])
            && shell_continuation.windows(25).any(|window| window
                == [
                    0xbf, 0x9c, 0xff, 0xff, 0xff, 0xbe, 0x60, 0x94, 0x5f, 0, 0x48, 0x89, 0xe2,
                    0x45, 0x31, 0xd2, 0xb8, 0x06, 0x01, 0, 0, 0x0f, 0x05, 0x48, 0x85,
                ])
            && shell_continuation.windows(19).any(|window| window
                == [
                    0xbf, 1, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba, 13, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0x0f,
                ])
            && shell_continuation.windows(10).any(
                |window| window == [0x48, 0xb8, b'/', b'b', b'i', b'n', b'/', b'b', b'u', b's']
            )
            && shell_continuation
                .windows(10)
                .any(|window| window == [0x48, 0xb8, b'y', b'b', b'o', b'x', b'\n', 0, 0, 0])
            && shell_continuation.windows(10).any(
                |window| window == [0x48, 0xb8, b'b', b'u', b's', b'y', b'b', b'o', b'x', b'-']
            )
            && shell_continuation
                .windows(10)
                .any(|window| window == [0x48, 0xb8, b'f', b'i', b'l', b'e', b'\n', 0, 0, 0])
            && shell_continuation.windows(10).any(|window| {
                window == [0x48, 0xb8, b'e', b'x', b'e', b'c', b' ', b'3', b'<', b'/']
            })
            && shell_continuation.windows(10).any(|window| {
                window == [0x48, 0xb8, b'b', b'i', b'n', b'/', b'b', b'u', b's', b'y']
            })
            && shell_continuation.windows(10).any(|window| {
                window == [0x48, 0xb8, b'b', b'o', b'x', b';', b'p', b'w', b'd', b'\n']
            })
            && shell_continuation.windows(12).any(|window| window
                == [
                    0x66, 0x81, 0x3c, 0x25, 0x60, 0x94, 0x5f, 0, b'/', b'\n', 0x75, 0xfe
                ])
            && shell_continuation.windows(19).any(|window| window
                == [
                    0xbf, 1, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba, 6, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0x0f
                ])
            && shell_continuation.windows(10).any(
                |window| window == [0x48, 0xb8, b'e', b'x', b'e', b'c', b' ', b'/', b'b', b'i']
            )
            && shell_continuation.windows(11).any(
                |window| window == [0x48, 0x83, 0x3c, 0x25, 0x78, 0x8f, 0x5f, 0, 0, 0x75, 0xfe]
            )
            && shell_continuation
                .windows(9)
                .any(|window| window == [0xb8, 0x3b, 0, 0, 0, 0x0f, 0x05, 0x0f, 0x0b]),
        "clean process fixture must consume the shell corpus, refill the exec command, and issue nonreturning image replacement"
    );
    assert_eq!(
        process_enter
            .windows(3)
            .filter(|window| *window == [0x0f, 0x22, 0xd8])
            .count(),
        4,
        "entry, private pair exchange, child parent restoration and terminal return each reload CR3 once"
    );
    assert!(
        process_enter
            .windows(8)
            .any(|window| window == [0x0f, 0x20, 0xc0, 0xa9, 0x0c, 0, 0, 0]),
        "process entry must reject unavailable x87/SIMD state"
    );
    assert!(
        process_enter
            .windows(10)
            .any(|window| { window == [0x0f, 0x20, 0xe0, 0x49, 0x89, 0x82, 0x18, 0x01, 0, 0] })
            && process_enter
                .windows(9)
                .any(|window| window == [0x48, 0x0d, 0, 0x06, 0, 0, 0x0f, 0x22, 0xe0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x49, 0x8b, 0x82, 0x18, 0x01, 0, 0])
            && process_enter
                .windows(3)
                .filter(|window| *window == [0x0f, 0x22, 0xe0])
                .count()
                == 2,
        "process entry must retain, enable, and exactly restore CR4 SIMD controls"
    );
    assert_eq!(
        process_enter
            .windows(10)
            .filter(|window| { *window == [0xdb, 0xe3, 0x41, 0x0f, 0xae, 0x92, 0x20, 0x01, 0, 0] })
            .count(),
        3,
        "entry, exec replacement, and return must initialize and scrub x87 state"
    );
    for (rex, register, offset) in STATIC_PROCESS_SYSCALL_REGISTER_SLOTS {
        let mut save = vec![0x65, rex, 0x89, register, 0x25];
        save.extend_from_slice(&offset.to_le_bytes());
        let mut restore = vec![0x65, rex, 0x8b, register, 0x25];
        restore.extend_from_slice(&offset.to_le_bytes());
        assert_eq!(
            process_enter
                .windows(save.len())
                .filter(|window| *window == save)
                .count(),
            1,
            "each Linux syscall-boundary register must be saved exactly once"
        );
        assert!(
            process_enter
                .windows(restore.len())
                .any(|window| window == restore),
            "each Linux syscall-boundary register must be restored before SYSRETQ"
        );
    }
    (process_enter, process_restore, process_fixture_image)
}
