fn audit_provider_process_syscalls(
    process_enter: &[u8],
    process_restore: &[u8],
    shell_continuation: &[u8],
) {
    let mut kernel_stack_entry = vec![0x0f, 0x01, 0xf8];
    for (rex, register, offset) in STATIC_PROCESS_SYSCALL_REGISTER_SLOTS {
        kernel_stack_entry.extend_from_slice(&[0x65, rex, 0x89, register, 0x25]);
        kernel_stack_entry.extend_from_slice(&offset.to_le_bytes());
    }
    kernel_stack_entry.extend_from_slice(&[0x65, 0x48, 0x8b, 0x24, 0x25, 8, 0, 0, 0]);
    assert!(
        process_enter
            .windows(kernel_stack_entry.len())
            .any(|bytes| bytes == kernel_stack_entry),
        "syscall admission must save the complete user context and switch to retained kernel RSP before provider work"
    );
    for (rex, register, offset) in STATIC_PROCESS_CALLER_REGISTER_SLOTS {
        let mut save = vec![rex, 0x89, register];
        save.extend_from_slice(&offset.to_le_bytes());
        let mut restore = vec![rex, 0x8b, register];
        restore.extend_from_slice(&offset.to_le_bytes());
        assert_eq!(
            process_enter
                .windows(save.len())
                .filter(|window| *window == save)
                .count(),
            1,
            "each provider callee-saved register must be retained exactly once"
        );
        assert_eq!(
            process_enter
                .windows(restore.len())
                .filter(|window| *window == restore)
                .count(),
            1,
            "each provider callee-saved register must be restored exactly once"
        );
    }
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0xe7, 0, 0, 0]),
        "the static dispatcher must accept terminal exit_group"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x0c, 0, 0, 0]),
        "the static dispatcher must accept the initial brk query"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x9e, 0, 0, 0]),
        "the static dispatcher must accept initial x86 FS-base installation"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0xda, 0, 0, 0]),
        "the static dispatcher must accept initial clear-child-TID registration"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x4e, 0x01, 0, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0xc7, 0xc0, 0xda, 0xff, 0xff, 0xff]),
        "the static dispatcher must return exact ENOSYS for the initial rseq probe"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x2e, 0x01, 0, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x49, 0xc7, 0x02, 0, 0, 0x80, 0])
            && process_enter
                .windows(8)
                .any(|window| window == [0x49, 0xc7, 0x42, 8, 0xff, 0xff, 0xff, 0xff]),
        "the static dispatcher must publish the exact immutable stack-limit pair"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x0b, 0x01, 0, 0]),
        "the static dispatcher must select the readlinkat unsupported-operation path"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x3e, 0x01, 0, 0]),
        "the static dispatcher must select the bounded getrandom path"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x0a, 0, 0, 0]),
        "the static dispatcher must select the exact RELRO mprotect path"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x9d, 0, 0, 0]),
        "the static dispatcher must select the initial process-name path"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x66, 0, 0, 0]),
        "the static dispatcher must select the initial getuid path"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0x06, 0x01, 0, 0]),
        "the static dispatcher must select the initial configuration-stat miss path"
    );
    for number in [104_u8, 106, 105] {
        assert!(
            process_enter
                .windows(5)
                .any(|window| window == [0x3d, number, 0, 0, 0]),
            "the static dispatcher must select every credential-tail operation"
        );
    }
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 1, 0, 0, 0]),
        "the static dispatcher must select the applet newline write"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 39, 0, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 17]),
        "the static dispatcher must return sealed PID one and advance its phase"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 13, 0, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 18])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0x4c, 0x89, 0x04, 0x25, 0xa8, 0, 0, 0]),
        "the static dispatcher must retain the child disposition and advance its phase"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 110, 0, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 19]),
        "the static dispatcher must return no parent and advance its phase"
    );
    assert!(
        process_enter
            .windows(3)
            .any(|window| window == [0x83, 0xf8, 0x3f])
            && process_enter
                .windows(3)
                .any(|window| window == [0x83, 0xf8, 0x4f])
            && process_enter
                .windows(7)
                .any(|window| window == [0x49, 0x81, 0xc0, 0x86, 0x01, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 20]),
        "shell identity publication must dispatch uname and copy all 390 bytes"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xff, 0x60, 0x94, 0x5f, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0x81, 0xfe, 0, 0x10, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 21]),
        "shell current directory must bind the exact heap output and sealed root result"
    );
    for phase in [23_u8, 24, 26] {
        assert!(
            process_enter
                .windows(3)
                .any(|window| window == [0x41, 0xb1, phase]),
            "shell default queries must select every ordered successor phase"
        );
    }
    for phase in [25_u8, 27] {
        assert!(
            process_enter
                .windows(9)
                .any(|window| { window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, phase] }),
            "shell stateful transitions must publish their exact phase"
        );
    }
    assert!(
        process_enter.windows(17).any(|window| {
            window
                == [
                    0x31, 0xc0, 0x48, 0x89, 0x02, 0x48, 0x89, 0x42, 8, 0x48, 0x89, 0x42, 16, 0x48,
                    0x89, 0x42, 24,
                ]
        }) && process_enter
            .windows(9)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xa1, 0x01, 0, 0, 1]),
        "signal queries must publish complete defaults and retain ignored quit"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x60, 0x8f, 0x5f, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0x81, 0xfa, 0xff, 0x03, 0, 0]),
        "sealed command input must require the exact candidate and count"
    );
    assert!(
        process_enter
            .windows(3)
            .any(|window| window == [0x48, 0x85, 0xff]),
        "credential setters must require the sealed zero identity argument"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xec, 0x80, 0, 0, 0]),
        "repeated static getrandom must use bounded privileged extraction scratch"
    );
    assert!(
        process_enter
            .windows(4)
            .any(|window| window == [0x00, 0x70, 0x5f, 0x00]),
        "the static dispatcher must derive the page-rounded initial break"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xff, 0x00, 0x90, 0x61, 0x00])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 8])
            && process_enter.windows(10).any(
                |window| window == [0xbf, 0x00, 0x90, 0x61, 0x00, 0xb8, 0x00, 0x90, 0x61, 0x00]
            ),
        "page-alignment growth must require and publish the exact byte-granular break"
    );
    assert!(
        process_enter
            .windows(3)
            .any(|window| window == [0x83, 0xf8, 0x10])
            && process_enter
                .windows(6)
                .any(|window| window == [0x81, 0xfe, 0x01, 0x54, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 22]),
        "shell terminal probing must bind exact TCGETS absence and advance its phase"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 32])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0x48, 0x39, 0x04, 0x25, 0xe8, 0, 0, 0])
            && process_enter
                .windows(10)
                .any(|window| { window == [0x65, 0xc6, 0x04, 0x25, 0xa1, 0x01, 0, 0, 0, 0x65] }),
        "shell completion must require output-complete phase and clear both retained dispositions"
    );
    assert!(
        process_enter
            .windows(3)
            .any(|window| window == [0x83, 0xf8, 0x50])
            && process_enter
                .windows(5)
                .any(|window| window == [0x66, 0x81, 0x3f, b'/', 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe7, 0x02, 0, 0, 1])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 29]),
        "root chdir must dispatch syscall 80, validate the exact pathname, publish completion, and advance to the root-test phase"
    );
    assert!(
        process_enter
            .windows(5)
            .any(|window| window == [0x3d, 0xd9, 0, 0, 0])
            && process_enter.windows(10).any(
                |window| window == [0x48, 0xb8, b'e', b'c', b'h', b'o', b' ', b'/', b'*', b';']
            )
            && process_enter
                .windows(6)
                .any(|window| window == [0x81, 0xff, 0x9c, 0xff, 0xff, 0xff])
            && process_enter
                .windows(6)
                .any(|window| window == [0x48, 0x81, 0xfa, 0, 8, 9])
            && process_enter.windows(13).any(|window| {
                window == [0x65, 0x48, 0xc7, 0x04, 0x25, 0, 3, 0, 0, 0xed, 0x41, 0, 0]
            })
            && process_enter.windows(13).any(|window| {
                window == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xf8, 2, 0, 0, 3, 0, 0, 0]
            })
            && process_enter
                .windows(10)
                .any(|window| { window == [0x48, 0xb8, 0x18, 0, 4, b'b', b'i', b'n', 0, 0] })
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, 1])
            && [2_u8, 3, 4, 5, 6].into_iter().all(|state| {
                process_enter
                    .windows(9)
                    .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, state])
            })
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 30]),
        "root glob must publish the exact command, open and coherently describe the exact root directory, publish the named child record, close it, and advance to selected output"
    );
    assert!(
        process_enter
            .windows(10)
            .any(|window| { window == [0x48, 0xb8, b'/', b'b', b'i', b'n', b'\n', 0, 0, 0] })
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, 6])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0x80, 0x3c, 0x25, 0x78, 0x03, 0, 0, 6])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, 0]),
        "root-glob enumeration must publish the exact named match, gate pwd on its private observation, and terminally clear it"
    );
    assert!(
        process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b' ', b't', b'e', b's', b't', b' ', b'-', b'd']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b' ', b'/', b'b', b'i', b'n', b' ', b'&', b'&']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b' ', b'e', b'c', b'h', b'o', b' ', b'/', b'b']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'i', b'n', b'/', b'*', b';', b' ', b't', b'e']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b's', b't', b' ', b'-', b'f', b' ', b'/', b'b']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'o', b'x', b'-', b'f', b'i', b'l', b'e', b';']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b' ', b't', b'e', b's', b't', b' ', b'-', b'x']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b's', b'y', b'b', b'o', b'x', b'-', b'e', b'x']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'e', b'c', b'u', b't', b'a', b'b', b'l', b'e']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b';', b' ', b't', b'e', b's', b't', b' ', b'-']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b's', b' ', b'/', b'b', b'i', b'n', b'/', b'b']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'u', b's', b'y', b'b', b'o', b'x', b' ', b'&']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'&', b' ', b'e', b'c', b'h', b'o', b' ', b'b']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'u', b's', b'y', b'b', b'o', b'x', b'-', b'n']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'o', b'n', b'e', b'm', b'p', b't', b'y', b';']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b' ', b'e', b'x', b'e', b'c', b' ', b'3', b'<']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'/', b'b', b'i', b'n', b'/', b'b', b'u', b's']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'y', b'b', b'o', b'x', b';', b' ', b'e', b'c']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'h', b'o', b' ', b'b', b'u', b's', b'y', b'b']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'o', b'x', b'-', b'o', b'p', b'e', b'n', b';']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b' ', b'I', b'F', b'S', b'=', b' ', b'r', b'e']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'a', b'd', b' ', b'-', b'r', b' ', b'-', b'n']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b' ', b'4', b' ', b'-', b'u', b' ', b'3', b' ']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'_', b' ', b'&', b'&', b' ', b'e', b'c', b'h']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'o', b' ', b'b', b'u', b's', b'y', b'b', b'o']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'x', b'-', b'r', b'e', b'a', b'd', b';', b' ']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'e', b'x', b'e', b'c', b' ', b'3', b'<', b'&']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'-', b';', b' ', b'e', b'c', b'h', b'o', b' ']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'b', b'u', b's', b'y', b'b', b'o', b'x', b'-']
        }) && process_enter
            .windows(10)
            .any(|window| { window == [0x48, 0xb8, b'c', b'l', b'o', b's', b'e', b'\n', 0, 0] })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'e', b'x', b'e', b'c', b' ', b'3', b'<', b'/']
            })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'b', b'i', b'n', b'/', b'b', b'u', b's', b'y']
            })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'b', b'o', b'x', b';', b'p', b'w', b'd', b'\n']
            })
            && process_enter
                .windows(5)
                .any(|window| window == [0xb8, 0x26, 1, 0, 0])
            && process_enter
                .windows(5)
                .any(|window| window == [0xb8, 24, 0, 0, 0]),
        "bin-busybox reopen must preserve the exact 294-byte predecessor command and publish the exact 24-byte reopen-plus-pwd command"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| window == [0x65, 0x80, 0x3c, 0x25, 0x78, 0x03, 0, 0, 6])
            && process_enter
                .windows(6)
                .any(|window| window == [0x81, 0x3e, b'/', b'b', b'i', b'n'])
            && process_enter.windows(13).any(|window| {
                window == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xf0, 0x02, 0, 0, 4, 0, 0, 0]
            })
            && process_enter.windows(13).any(|window| {
                window == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xf8, 0x02, 0, 0, 2, 0, 0, 0]
            })
            && process_enter.windows(13).any(|window| {
                window
                    == [
                        0x65, 0x48, 0xc7, 0x04, 0x25, 0, 0x03, 0, 0, 0xed, 0x41, 0, 0,
                    ]
            })
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, 7]),
        "named root stat must require the enumeration fact, match /bin exactly, publish complete directory metadata, and advance its private lifecycle"
    );
    assert!(
        [8_u8, 9, 10, 11, 12, 13].into_iter().all(|state| {
            process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, state])
        }) && process_enter
            .windows(6)
            .any(|window| window == [0x48, 0x81, 0xfa, 0, 8, 9])
            && process_enter
                .windows(5)
                .any(|window| window == [0xb8, 80, 0, 0, 0])
            && process_enter
                .windows(10)
                .any(|window| { window == [0x48, 0xb8, 0x20, 0, 8, b'b', b'u', b's', b'y', b'b'] })
            && process_enter
                .windows(10)
                .any(|window| { window == [0x48, 0xb8, b'o', b'x', 0, 0, 0, 0, 0, 0] })
            && process_enter
                .windows(4)
                .any(|window| window == [0x48, 0x83, 0xfa, 13])
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'/', b'b', b'i', b'n', b'/', b'b', b'u', b's']
            })
            && process_enter
                .windows(10)
                .any(|window| { window == [0x48, 0xb8, b'y', b'b', b'o', b'x', b'\n', 0, 0, 0] })
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0x80, 0x3c, 0x25, 0x78, 0x03, 0, 0, 13]),
        "bin-busybox glob must open and describe the named directory, publish the regular-file child record, observe EOF, close, and emit the exact match"
    );
    assert!(
        process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'/', b'b', b'i', b'n', b'/', b'b', b'u', b's']
        }) && process_enter.windows(13).any(|window| {
            window == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xf0, 0x02, 0, 0, 5, 0, 0, 0]
        }) && process_enter.windows(13).any(|window| {
            window == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xf8, 0x02, 0, 0, 1, 0, 0, 0]
        }) && process_enter.windows(13).any(|window| {
            window
                == [
                    0x65, 0x48, 0xc7, 0x04, 0x25, 0, 0x03, 0, 0, 0x6d, 0x81, 0, 0,
                ]
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'b', b'u', b's', b'y', b'b', b'o', b'x', b'-']
        }) && [14_u8, 15].into_iter().all(|state| {
            process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, state])
        }),
        "bin-busybox file condition must match the exact pathname, publish complete regular-file metadata, emit the selected output, and advance through lifecycle state fifteen"
    );
    assert!(
        shell_continuation
            .windows(10)
            .any(|window| { window == [0xb8, 107, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0] })
            && process_enter
                .windows(5)
                .any(|window| window == [0x3d, 107, 0, 0, 0])
            && [16_u8, 17, 18].into_iter().all(|state| {
                process_enter
                    .windows(9)
                    .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, state])
            })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'b', b'u', b's', b'y', b'b', b'o', b'x', b'-']
            })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'e', b'x', b'e', b'c', b'u', b't', b'a', b'b']
            })
            && process_enter
                .windows(10)
                .any(|window| { window == [0x48, 0xb8, b'l', b'e', b'\n', 0, 0, 0, 0, 0] })
            && process_enter
                .windows(4)
                .any(|window| window == [0x48, 0x83, 0xfa, 19]),
        "bin-busybox executable condition must repeat complete metadata, observe effective UID zero, emit all nineteen selected bytes, and advance through lifecycle state eighteen"
    );
    assert!(
        [19_u8, 20].into_iter().all(|state| {
            process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, state])
        }) && process_enter.windows(13).any(|window| {
            window
                == [
                    0x65, 0x48, 0xc7, 0x04, 0x25, 0x18, 0x03, 0, 0, 0x60, 0xe4, 0x1e, 0,
                ]
        }) && process_enter.windows(13).any(|window| {
            window
                == [
                    0x65, 0x48, 0xc7, 0x04, 0x25, 0x28, 0x03, 0, 0, 0x73, 0x0f, 0, 0,
                ]
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'n', b'o', b'n', b'e', b'm', b'p', b't', b'y']
        }) && process_enter
            .windows(4)
            .any(|window| window == [0x48, 0x83, 0xfa, 17]),
        "bin-busybox nonempty condition must repeat complete positive-size metadata, emit all seventeen selected bytes, and advance through lifecycle state twenty"
    );
    assert!(
        [21_u8, 22, 27].into_iter().all(|state| {
            process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x78, 0x03, 0, 0, state])
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'/', b'b', b'i', b'n', b'/', b'b', b'u', b's']
        }) && process_enter.windows(10).any(|window| {
            window == [0x48, 0xb8, b'b', b'u', b's', b'y', b'b', b'o', b'x', b'-']
        }) && process_enter
            .windows(10)
            .any(|window| { window == [0x48, 0xb8, b'o', b'p', b'e', b'n', b'\n', 0, 0, 0] })
            && process_enter
                .windows(6)
                .filter(|window| **window == [0x48, 0x3d, 0, 0, 0x60, 0])
                .count()
                >= 2
            && shell_continuation.windows(28).any(|window| window
                == [
                    0xbf, 0x9c, 0xff, 0xff, 0xff, 0xbe, 0x60, 0x94, 0x5f, 0, 0x31, 0xd2, 0x45,
                    0x31, 0xd2, 0xb8, 0x01, 0x01, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 3, 0x75,
                    0xfe,
                ])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe1, 0x02, 0, 0, 1])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe1, 0x02, 0, 0, 0]),
        "bin-busybox read-only open must validate the exact pathname and arguments, bind descriptor three, emit all thirteen selected bytes, and hand the live description to the bounded read and explicit-close sequence"
    );
    assert!(
        shell_continuation
            .windows(10)
            .filter(|window| {
                **window == [0xbf, 3, 0, 0, 0, 0x48, 0x8d, 0x74, 0x24, 0]
                    || **window == [0xbf, 3, 0, 0, 0, 0x48, 0x8d, 0x74, 0x24, 1]
                    || **window == [0xbf, 3, 0, 0, 0, 0x48, 0x8d, 0x74, 0x24, 2]
                    || **window == [0xbf, 3, 0, 0, 0, 0x48, 0x8d, 0x74, 0x24, 3]
            })
            .count()
            == 4
            && [0x7f_u8, b'E', b'L', b'F']
                .into_iter()
                .enumerate()
                .all(|(offset, byte)| {
                    shell_continuation.windows(7).any(|window| {
                        window
                            == [
                                0x80,
                                0x7c,
                                0x24,
                                u8::try_from(offset).expect("bounded ELF header offset"),
                                byte,
                                0x75,
                                0xfe,
                            ]
                    })
                })
            && process_enter
                .windows(8)
                .any(|window| window == [0x65, 0xfe, 0x04, 0x25, 0x79, 0x03, 0, 0])
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'b', b'u', b's', b'y', b'b', b'o', b'x', b'-']
            })
            && process_enter
                .windows(10)
                .any(|window| { window == [0x48, 0xb8, b'r', b'e', b'a', b'd', b'\n', 0, 0, 0] })
            && process_enter
                .windows(9)
                .any(|window| { window == [0x65, 0x80, 0x3c, 0x25, 0x79, 0x03, 0, 0, 4] })
            && process_enter
                .windows(9)
                .any(|window| { window == [0x65, 0xc6, 0x04, 0x25, 0x79, 0x03, 0, 0, 0] })
            && process_enter
                .windows(5)
                .any(|window| window == [0x83, 0xf8, 7, 0x0f, 0x84])
            && [0x5401_u32, 0x5402].into_iter().all(|request| {
                let bytes = request.to_le_bytes();
                process_enter
                    .windows(6)
                    .any(|window| window == [0x81, 0xfe, bytes[0], bytes[1], bytes[2], bytes[3]])
            })
            && process_enter
                .windows(8)
                .any(|window| window == [0x65, 0xfe, 0x04, 0x25, 0x7a, 0x03, 0, 0])
            && [0x0000_0001_0000_0003_u64, 0x0001_0001_0000_0003_u64,]
                .into_iter()
                .all(|pollfd| {
                    let bytes = pollfd.to_le_bytes();
                    process_enter.windows(10).any(|window| {
                        window
                            == [
                                0x48, 0xb8, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4],
                                bytes[5], bytes[6], bytes[7],
                            ]
                    })
                })
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x7b, 0x03, 0, 0, 1])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x7b, 0x03, 0, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x7a, 0x03, 0, 0, 0]),
        "bin-busybox header read must dispatch only lifecycle states twenty-two through twenty-five, model the exact external ENOTTY and POLLIN sequence, issue four one-byte descriptor reads, validate the ELF identity, emit the exact selected output, and hand the live description to explicit close"
    );
    assert!(
        shell_continuation.windows(28).any(|window| window
            == [
                0xbf, 3, 0, 0, 0, 0xbe, 6, 4, 0, 0, 0xba, 10, 0, 0, 0, 0xb8, 72, 0, 0, 0, 0x0f,
                0x05, 0x48, 0x83, 0xf8, 10, 0x75, 0xfe,
            ])
            && [3_u8, 10].into_iter().all(|descriptor| {
                shell_continuation.windows(17).any(|window| {
                    window
                        == [
                            0xbf, descriptor, 0, 0, 0, 0xb8, 3, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85,
                            0xc0, 0x75, 0xfe,
                        ]
                })
            })
            && shell_continuation.windows(10).any(|window| {
                window == [0x48, 0xb8, b'b', b'u', b's', b'y', b'b', b'o', b'x', b'-']
            })
            && shell_continuation.windows(10).any(|window| {
                window == [0x48, 0xb8, b'c', b'l', b'o', b's', b'e', b'\n', 0, 0]
            })
            && shell_continuation.windows(19).any(|window| {
                window
                    == [
                        0xbf, 1, 0, 0, 0, 0x48, 0x89, 0xe6, 0xba, 14, 0, 0, 0, 0xb8, 1, 0, 0, 0,
                        0x0f,
                    ]
            })
            && process_enter
                .windows(5)
                .any(|window| window == [0x83, 0xf8, 0x48, 0x0f, 0x84])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0x81, 0xfe, 6, 4, 0, 0])
            && process_enter
                .windows(4)
                .any(|window| window == [0x48, 0x83, 0xfa, 10])
            && [1_u8, 2, 3, 4, 0].into_iter().all(|state| {
                process_enter
                    .windows(9)
                    .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x7c, 0x03, 0, 0, state])
            })
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe1, 0x02, 0, 0, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x79, 0x03, 0, 0, 0]),
        "bin-busybox explicit close must duplicate descriptor three to close-on-exec descriptor ten, close both references in order, destroy the shared description only at the last close, emit the exact selected output, and hand absence to reopen"
    );
    assert!(
        shell_continuation
            .windows(28)
            .filter(|window| {
                **window
                    == [
                        0xbf, 0x9c, 0xff, 0xff, 0xff, 0xbe, 0x60, 0x94, 0x5f, 0, 0x31, 0xd2, 0x45,
                        0x31, 0xd2, 0xb8, 0x01, 0x01, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, 3, 0x75,
                        0xfe,
                    ]
            })
            .count()
            == 2
            && [5_u8, 0].into_iter().all(|state| {
                process_enter
                    .windows(9)
                    .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x7c, 0x03, 0, 0, state])
            })
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe1, 0x02, 0, 0, 1]),
        "bin-busybox reopen must reuse descriptor three for a fresh offset-zero description, retain it through pwd, and revoke it at teardown"
    );
    assert!(
        !shell_continuation
            .windows(5)
            .any(|window| window == [0xb8, 0xb7, 0x01, 0, 0])
            && !process_enter
                .windows(5)
                .any(|window| window == [0x3d, 0xb7, 0x01, 0, 0]),
        "bin-busybox executable condition must not substitute an unobserved faccessat2 syscall"
    );
    assert!(
        process_enter
            .windows(9)
            .any(|window| { window == [0x65, 0x80, 0x3c, 0x25, 0xa2, 0x01, 0, 0, 1] })
            && process_enter
                .windows(9)
                .any(|window| { window == [0x65, 0xc6, 0x04, 0x25, 0xa2, 0x01, 0, 0, 2] })
            && process_enter
                .windows(9)
                .any(|window| { window == [0x65, 0xc6, 0x04, 0x25, 0xa2, 0x01, 0, 0, 3] })
            && process_enter
                .windows(9)
                .any(|window| { window == [0x65, 0xc6, 0x04, 0x25, 0xa2, 0x01, 0, 0, 4] })
            && process_enter
                .windows(9)
                .any(|window| { window == [0x65, 0xc6, 0x04, 0x25, 0xa2, 0x01, 0, 0, 5] })
            && process_enter
                .windows(9)
                .any(|window| { window == [0x65, 0xc6, 0x04, 0x25, 0xa2, 0x01, 0, 0, 6] })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'e', b'c', b'h', b'o', b' ', b't', b'o', b'p']
            })
            && process_enter
                .windows(10)
                .any(|window| { window == [0x48, 0xb8, b'a', b'l', b'\n', 0, 0, 0, 0, 0] })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'e', b'c', b'h', b'o', b' ', b'/', b'*', b';']
            })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'e', b'x', b'e', b'c', b' ', b'3', b'<', b'/']
            })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'b', b'i', b'n', b'/', b'b', b'u', b's', b'y']
            })
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, b'b', b'o', b'x', b';', b'p', b'w', b'd', b'\n']
            })
            && process_enter
                .windows(10)
                .any(|window| { window == [0x48, 0xb8, b'e', b'x', b'i', b't', b'\n', 0, 0, 0] })
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xa2, 0x01, 0, 0, 6])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 32]),
        "shell command reads must require fresh input ownership, publish echo, cd root, root glob, reopen plus pwd, then exit, advance ownership six times, and reach the exact terminal phase"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x40, 0xf2, 0x7f, 0])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe7, 0x02, 0, 0, 7])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0x81, 0xfa, 0xa0, 0xf1, 0x7f, 0])
            && process_enter
                .windows(6)
                .any(|window| window == [0x81, 0xfe, 0x01, 0x54, 0, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0xc7, 0xc0, 0xe7, 0xff, 0xff, 0xff])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0xe7, 0x02, 0, 0, 8]),
        "external pwd must bind complete stdout metadata, exact TCGETS absence, and their ordered private facts"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x60, 0x94, 0x5f, 0]),
        "clean pwd output must bind its exact readable candidate"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x00, 0x70, 0x5f, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 2])
            && process_enter
                .windows(6)
                .any(|window| window == [0x48, 0x3d, 0x00, 0x90, 0x61, 0]),
        "external pwd output must bind its complete candidate to the checked readable heap"
    );
    assert!(
        process_enter
            .windows(11)
            .any(|window| window == [0x65, 0x66, 0x81, 0x3c, 0x25, 0xe8, 0x02, 0, 0, 0x2f, 0x0a]),
        "pwd output must stage exact root plus newline"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0xa0, 0xfb, 0x7f, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0x81, 0xfe, 0x00, 0x70, 0x5f, 0])
            && process_enter
                .windows(6)
                .any(|window| window == [0x48, 0x3d, 0x00, 0x90, 0x61, 0])
            && process_enter
                .windows(8)
                .any(|window| window == [0x44, 0x8b, 0x06, 0x44, 0x0f, 0xb7, 0x4e, 4])
            && process_enter
                .windows(9)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 28]),
        "shell echo output must bind the clean candidate and external heap extent, validate backing, stage six bytes, and advance phase"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x60, 0xfd, 0x7f, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0x81, 0xfe, 0x50, 0x84, 0x5f, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0x8b, 0x82, 0xc0, 0x0f, 0, 0]),
        "newline output must bind each profile to its exact readable candidate backing"
    );
    for (candidate, header, phase) in [
        ([0x48, 0x81, 0xfe, 0x60, 0xfd, 0x7f, 0], 1_u8, 15_u8),
        ([0x48, 0x81, 0xfe, 0x50, 0x84, 0x5f, 0], 2_u8, 11_u8),
    ] {
        let candidate_position = process_enter
            .windows(candidate.len())
            .position(|window| window == candidate)
            .expect("profile-bound newline candidate");
        let prefix = &process_enter[candidate_position.saturating_sub(64)..candidate_position];
        assert!(
            prefix.windows(2).any(|window| window == [0x3c, header])
                && prefix
                    .windows(9)
                    .any(|window| { window == [0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, phase] }),
            "each newline candidate must require its exact header and predecessor phase"
        );
    }
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x44, 0x8a, 0x06, 0x41, 0x80, 0xf8, 0x0a])
            && process_enter
                .windows(8)
                .any(|window| window == [0x66, 0xba, 0xf8, 0x03, 0x44, 0x88, 0xc0, 0xee])
            && process_enter
                .windows(3)
                .any(|window| window == [0x45, 0x31, 0xc0]),
        "newline output must stage, verify, publish, and clear exactly one line feed"
    );
    let alignment_start = process_enter
        .windows(7)
        .position(|window| window == [0x48, 0x81, 0xff, 0x00, 0x90, 0x61, 0x00])
        .expect("alignment handler start");
    let alignment_end = alignment_start
        + process_enter[alignment_start..]
            .windows(10)
            .position(|window| {
                window == [0xbf, 0x00, 0x90, 0x61, 0x00, 0xb8, 0x00, 0x90, 0x61, 0x00]
            })
            .expect("alignment handler response")
        + 10;
    let alignment_handler = &process_enter[alignment_start..alignment_end];
    assert!(
        alignment_handler
            .windows(3)
            .any(|window| window == [0x4d, 0x89, 0xc1])
            && alignment_handler
                .windows(3)
                .all(|window| window != [0x4d, 0x89, 0xc3]),
        "alignment leaf validation must preserve the SYSRETQ RFLAGS register"
    );
    for count in [9_u32, 25, 485] {
        let mut instruction = vec![0xb9];
        instruction.extend_from_slice(&count.to_le_bytes());
        assert!(
            process_enter
                .windows(instruction.len())
                .any(|window| window == instruction),
            "page-alignment growth must revalidate every retained heap-leaf interval"
        );
    }
    let relro_start = process_enter
        .windows(7)
        .position(|window| window == [0x48, 0x81, 0xff, 0x00, 0x50, 0x5e, 0x00])
        .expect("RELRO mprotect handler start");
    let relro_end = relro_start
        + process_enter[relro_start..]
            .windows(11)
            .position(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 9, 0x31, 0xc0])
            .expect("RELRO mprotect state publication")
        + 11;
    let relro_handler = &process_enter[relro_start..relro_end];
    assert!(
        relro_handler
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x00, 0x80, 0x00, 0x00])
            && relro_handler
                .windows(4)
                .any(|window| window == [0x48, 0x83, 0xfa, 1]),
        "RELRO mprotect must require the exact eight-page read-only request"
    );
    assert!(
        relro_handler
            .windows(7)
            .any(|window| window == [0x4d, 0x8d, 0x81, 0x28, 0x0f, 0, 0])
            && relro_handler
                .windows(5)
                .any(|window| window == [0xb9, 8, 0, 0, 0]),
        "RELRO mprotect must validate all eight checked-role leaves"
    );
    assert!(
        relro_handler
            .windows(3)
            .any(|window| window == [0x4d, 0x89, 0xc1])
            && relro_handler
                .windows(7)
                .filter(|window| *window == [0x49, 0x8d, 0xba, 0, 0x40, 0, 0])
                .count()
                >= 2
            && relro_handler
                .windows(7)
                .all(|window| window != [0x49, 0x8d, 0xb8, 0, 0x40, 0, 0]),
        "RELRO validation must keep anonymous-frame and high-table identities distinct"
    );
    assert!(
        relro_handler
            .windows(3)
            .any(|window| window == [0x49, 0x89, 0x00])
            && relro_handler
                .windows(3)
                .any(|window| window == [0x0f, 0x01, 0x38])
            && relro_handler
                .windows(7)
                .any(|window| window == [0x49, 0x81, 0xea, 0, 0x80, 0, 0])
            && relro_handler
                .windows(4)
                .any(|window| window == [0x49, 0x83, 0xe8, 0x40]),
        "RELRO sealing must publish, invalidate, rewind, and read back the exact interval"
    );
    assert!(
        relro_handler
            .windows(3)
            .all(|window| window != [0x4d, 0x89, 0xc3]),
        "RELRO validation must preserve the SYSRETQ RFLAGS register"
    );
    let process_name_start = process_enter
        .windows(4)
        .position(|window| window == [0x48, 0x83, 0xff, 0x10])
        .expect("initial process-name handler start");
    let process_name_end = process_name_start
        + process_enter[process_name_start..]
            .windows(11)
            .position(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 10, 0x31, 0xc0])
            .expect("initial process-name state publication")
        + 11;
    let process_name_handler = &process_enter[process_name_start..process_name_end];
    assert!(
        process_name_handler
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x60, 0xfe, 0x7f, 0])
            && process_name_handler.windows(10).any(|window| {
                window == [0x48, 0xb8, 0x62, 0x75, 0x73, 0x79, 0x62, 0x6f, 0x78, 0]
            }),
        "initial process-name query must publish the exact complete sealed field"
    );
    assert!(
        process_name_handler
            .windows(7)
            .any(|window| window == [0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0])
            && process_name_handler
                .windows(3)
                .all(|window| window != [0x4d, 0x89, 0xc3]),
        "initial process-name output must require the writable stack leaf and preserve RFLAGS"
    );
    let getuid_start = process_enter
        .windows(9)
        .position(|w| w == [0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 10])
        .expect("initial getuid phase guard");
    let getuid_end = getuid_start
        + process_enter[getuid_start..]
            .windows(3)
            .position(|w| w == [0x48, 0x0f, 0x07])
            .expect("initial getuid native return")
        + 3;
    let getuid_handler = &process_enter[getuid_start..getuid_end];
    for (offset, value) in [
        (0x0f18_u32, 11_u8),
        (0x0f20, 0),
        (0x0f28, 12),
        (0x0f30, 0),
        (0x0f38, 13),
        (0x0f40, 0),
        (0x0f48, 14),
        (0x0f50, 0),
    ] {
        assert!(
            getuid_handler.windows(8).any(|window| {
                window[..3] == [0x48, 0x83, 0xba]
                    && window[3..7] == offset.to_le_bytes()
                    && window[7] == value
            }),
            "initial getuid must revalidate every sealed credential word"
        );
    }
    assert!(
        getuid_handler
            .windows(11)
            .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 11, 0x31, 0xc0])
            && getuid_handler
                .windows(3)
                .all(|window| window != [0x4d, 0x89, 0xc3]),
        "initial getuid must publish its phase, return zero, and preserve RFLAGS"
    );
    assert!(
        process_enter
            .windows(7)
            .any(|window| window == [0x48, 0x81, 0xfe, 0x00, 0x69, 0x4a, 0])
            && process_enter
                .windows(7)
                .any(|window| window == [0x48, 0x81, 0xfe, 0xa7, 0x0a, 0x5c, 0])
            && process_enter.windows(10).any(|window| {
                window == [0x48, 0xb8, 0x2f, 0x65, 0x74, 0x63, 0x2f, 0x62, 0x75, 0x73]
            })
            && process_enter
                .windows(11)
                .any(|window| window == [0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 12, 0x48, 0xc7]),
        "initial configuration-stat miss must bind both immutable paths and publish absence phase"
    );
    let credential_tail_start = process_enter
        .windows(14)
        .position(|w| w[..7] == [0x3d, 104, 0, 0, 0, 0x0f, 0x84] && w[11..] == [0x48, 0x85, 0xff])
        .expect("credential-tail dispatch guard");
    let credential_tail_end = credential_tail_start
        + process_enter[credential_tail_start..]
            .windows(3)
            .position(|w| w == [0x48, 0x0f, 0x07])
            .expect("credential-tail native return")
        + 3;
    let credential_tail_handler = &process_enter[credential_tail_start..credential_tail_end];
    for phase in [13_u8, 14, 15] {
        assert!(
            credential_tail_handler
                .windows(3)
                .any(|window| window == [0x41, 0xb1, phase]),
            "credential-tail operations must publish each exact ordered phase"
        );
    }
    assert!(
        credential_tail_handler
            .windows(9)
            .any(|window| window == [0x65, 0x44, 0x88, 0x0c, 0x25, 0x68, 0, 0, 0])
            && credential_tail_handler
                .windows(3)
                .all(|window| window != [0x4d, 0x89, 0xc3]),
        "credential-tail operations must publish only private phase and preserve RFLAGS"
    );
    assert!(
        process_enter
            .windows(3)
            .any(|window| window == [0x48, 0x0f, 0x07]),
        "the nonterminal query must return through native SYSRETQ"
    );
    for offset in (0_u32..=0x0fb0).step_by(8) {
        assert!(
            process_enter.windows(8).any(|window| {
                window[..3] == [0x48, 0x83, 0xa6]
                    && window[3..7] == offset.to_le_bytes()
                    && window[7] == 0x9f
            }),
            "terminal return must normalize the complete bounded image leaf table"
        );
    }
    assert!(
        [0xf0_u8, 0xf8].into_iter().all(|offset| {
            process_enter
                .windows(8)
                .any(|window| window == [0x48, 0x83, 0xa0, offset, 0x4f, 0, 0, 0x9f])
        }),
        "terminal return must normalize both high-table stack-leaf hardware status fields"
    );
    assert!(
        process_enter.windows(15).any(|window| {
            window
                == [
                    0x48, 0x8b, 0x7c, 0x24, 0x20, 0x48, 0x81, 0xef, 0, 0x10, 0, 0, 0x31, 0xc0, 0xb9,
                ]
        }) && process_enter
            .windows(9)
            .any(|window| { window == [0xb9, 0, 0x02, 0, 0, 0xfc, 0xf3, 0x48, 0xab] })
            && process_enter
                .windows(9)
                .any(|window| { window == [0xb9, 0, 0x02, 0, 0, 0xfc, 0xf3, 0x48, 0xaf] }),
        "terminal return must zero and verify the complete lower stack page"
    );
    assert!(
        process_enter.windows(11).any(|window| {
            window == [0xb9, 0xd2, 0x01, 0, 0, 0xfc, 0xf3, 0x48, 0xab, 0x0f, 0xae]
        }) && process_enter
            .windows(9)
            .any(|window| { window == [0xb9, 0xd2, 0x01, 0, 0, 0xfc, 0xf3, 0x48, 0xaf] }),
        "terminal return must zero and verify all 466 mutable stack-prefix qwords"
    );
    assert!(
        process_restore
            .windows(12)
            .any(|window| window == [0x31, 0xc0, 0xb9, 78, 0, 0, 0, 0xfc, 0xf3, 0x48, 0xab, 0xb8]),
        "qualification restoration must clear the complete private context"
    );
}
