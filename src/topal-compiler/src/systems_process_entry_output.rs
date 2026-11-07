fn emit_static_process_output(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    code.bind("static-process-applet-newline-write")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&896_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x84, "runtime-console-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[3]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-explicit-close-write",
    );
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 26]);
    code.jump_if(0x85, "static-process-applet-write-after-interactive");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-prompt");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0001_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-banner");
    code.jump("static-process-shell-terminal-warning");
    code.bind("static-process-applet-write-after-interactive")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 31]);
    code.jump_if(0x84, "static-process-shell-phase-31-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 30]);
    code.jump_if(0x84, "static-process-shell-phase-30-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 29]);
    code.jump_if(0x84, "static-process-shell-phase-29-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 28]);
    code.jump_if(0x84, "static-process-shell-phase-28-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 27]);
    code.jump_if(0x84, "static-process-shell-echo-write");
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");

    // Revalidate the active hierarchy before touching either sealed candidate.
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc2, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4c, 0x39, 0xd0]);
    code.jump_if(0x85, "fatal");

    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x85, "static-process-applet-newline-external-candidate");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 15]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fd60_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0, 0x48, 0x89, 0xc2]);
    code.bytes(&[0x48, 0x83, 0xe2, 0x9f, 0x48, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x89, 0xd7, 0x81, 0xe7, 0xff, 0x0f, 0, 0, 0x83, 0xff, 7]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-applet-newline-candidate-ready");

    code.bind("static-process-applet-newline-external-candidate")?;
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 11]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_8450_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x70, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x78, 0, 0, 0]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0xf7, 0xc2, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc0, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x8b, 0x82, 0xc0, 0x0f, 0, 0, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4d, 0x8d, 0x88, 0, 0x10, 0, 0, 0x49, 0x83, 0xc9, 7]);
    code.bytes(&[0x49, 0x0f, 0xba, 0xe9, 0x3f, 0x4c, 0x39, 0xc8]);
    code.jump_if(0x85, "fatal");

    code.bind("static-process-applet-newline-candidate-ready")?;
    code.bytes(&[0x44, 0x8a, 0x06, 0x41, 0x80, 0xf8, 0x0a]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-applet-newline-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(0x84, "static-process-applet-newline-uart-ready");
    code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x44, 0x88, 0xc0, 0xee]);
    code.bytes(&[0x45, 0x31, 0xc0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 16]);
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-explicit-close-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 14]);
    code.jump_if(0x85, "fatal");
    for (offset, expected) in [
        (417_u32, 1_u8),
        (418, 4),
        (737, 0),
        (743, 4),
        (888, 27),
        (889, 0),
        (890, 0),
        (891, 0),
        (892, 3),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-explicit-close-write-clean",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 14]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0060_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_low_heap_validation(
        code,
        "static-process-shell-bin-busybox-explicit-close-write-low-heap",
    )?;
    code.jump("static-process-shell-bin-busybox-explicit-close-write-profile-ready");
    code.bind("static-process-shell-bin-busybox-explicit-close-write-clean")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 14]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-bin-busybox-explicit-close-write-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    for offset in [744_u32, 752] {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-explicit-close-write-copy")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x16]);
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 14]);
    code.jump_if(
        0x85,
        "static-process-shell-bin-busybox-explicit-close-write-copy",
    );
    for (offset, expected) in [
        (744_u32, u64::from_le_bytes(*b"busybox-")),
        (752, u64::from_le_bytes(*b"close\n\0\0")),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-explicit-close-write-output")?;
    code.bind("static-process-shell-bin-busybox-explicit-close-write-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-explicit-close-write-uart-ready",
    );
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x65, 0x41, 0x8a, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0xee, 0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 14]);
    code.jump_if(
        0x85,
        "static-process-shell-bin-busybox-explicit-close-write-output",
    );
    for offset in [744_u32, 752] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[4]);
    code.bytes(&[0xb8, 14, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-terminal-warning")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0003_0101_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-warning-length-four");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0103_0101_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-warning-length-forty");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0203_0101_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xb9, 1, 0, 0, 0]);
    code.jump("static-process-shell-warning-candidate");
    code.bind("static-process-shell-warning-length-four")?;
    code.bytes(&[0x41, 0xb9, 4, 0, 0, 0]);
    code.jump("static-process-shell-warning-candidate");
    code.bind("static-process-shell-warning-length-forty")?;
    code.bytes(&[0x41, 0xb9, 40, 0, 0, 0]);
    code.bind("static-process-shell-warning-candidate")?;
    code.bytes(&[0x48, 0x83, 0xff, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4c, 0x39, 0xca]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x89, 0xf0, 0x49, 0x01, 0xd0]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0003_0101_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-warning-four");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0103_0101_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-warning-forty");
    code.jump("static-process-shell-warning-newline");

    code.bind("static-process-shell-warning-four")?;
    code.bytes(&[0x44, 0x8b, 0x06, 0x41, 0x81, 0xf8]);
    code.bytes(&u32::from_le_bytes(*b"sh: ").to_le_bytes());
    code.jump_if(0x85, "fatal");
    for (index, ready) in [
        "static-process-shell-warning-four-uart-0",
        "static-process-shell-warning-four-uart-1",
        "static-process-shell-warning-four-uart-2",
        "static-process-shell-warning-four-uart-3",
    ]
    .into_iter()
    .enumerate()
    {
        code.bind(ready)?;
        code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
        code.jump_if(0x84, ready);
        code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x44, 0x89, 0xc0, 0xee]);
        if index != 3 {
            code.bytes(&[0x49, 0xc1, 0xe8, 8]);
        }
    }
    code.bytes(&[0x45, 0x31, 0xc0]);
    code.bytes(&[0x65, 0xfe, 0x04, 0x25]);
    code.bytes(&619_u32.to_le_bytes());
    code.bytes(&[0xb8, 4, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-warning-forty")?;
    code.bytes(&[0x4c, 0x8b, 0x06, 0x4c, 0x8b, 0x4e, 8, 0x4c, 0x8b, 0x56, 16]);
    code.bytes(&[0x4c, 0x8b, 0x5e, 24, 0x4c, 0x8b, 0x66, 32]);
    for (register, expected) in [
        (0xc0_u8, u64::from_le_bytes(*b"can't ac")),
        (0xc1, u64::from_le_bytes(*b"cess tty")),
        (0xc2, u64::from_le_bytes(*b"; job co")),
        (0xc3, u64::from_le_bytes(*b"ntrol tu")),
        (0xc4, u64::from_le_bytes(*b"rned off")),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x49, 0x39, register]);
        code.jump_if(0x85, "fatal");
    }
    for (move_register, shift_register, ready_labels) in [
        (
            0xc0_u8,
            0xe8_u8,
            [
                "static-process-shell-warning-forty-r8-uart-0",
                "static-process-shell-warning-forty-r8-uart-1",
                "static-process-shell-warning-forty-r8-uart-2",
                "static-process-shell-warning-forty-r8-uart-3",
                "static-process-shell-warning-forty-r8-uart-4",
                "static-process-shell-warning-forty-r8-uart-5",
                "static-process-shell-warning-forty-r8-uart-6",
                "static-process-shell-warning-forty-r8-uart-7",
            ],
        ),
        (
            0xc8,
            0xe9,
            [
                "static-process-shell-warning-forty-r9-uart-0",
                "static-process-shell-warning-forty-r9-uart-1",
                "static-process-shell-warning-forty-r9-uart-2",
                "static-process-shell-warning-forty-r9-uart-3",
                "static-process-shell-warning-forty-r9-uart-4",
                "static-process-shell-warning-forty-r9-uart-5",
                "static-process-shell-warning-forty-r9-uart-6",
                "static-process-shell-warning-forty-r9-uart-7",
            ],
        ),
        (
            0xd0,
            0xea,
            [
                "static-process-shell-warning-forty-r10-uart-0",
                "static-process-shell-warning-forty-r10-uart-1",
                "static-process-shell-warning-forty-r10-uart-2",
                "static-process-shell-warning-forty-r10-uart-3",
                "static-process-shell-warning-forty-r10-uart-4",
                "static-process-shell-warning-forty-r10-uart-5",
                "static-process-shell-warning-forty-r10-uart-6",
                "static-process-shell-warning-forty-r10-uart-7",
            ],
        ),
        (
            0xd8,
            0xeb,
            [
                "static-process-shell-warning-forty-r11-uart-0",
                "static-process-shell-warning-forty-r11-uart-1",
                "static-process-shell-warning-forty-r11-uart-2",
                "static-process-shell-warning-forty-r11-uart-3",
                "static-process-shell-warning-forty-r11-uart-4",
                "static-process-shell-warning-forty-r11-uart-5",
                "static-process-shell-warning-forty-r11-uart-6",
                "static-process-shell-warning-forty-r11-uart-7",
            ],
        ),
        (
            0xe0,
            0xec,
            [
                "static-process-shell-warning-forty-r12-uart-0",
                "static-process-shell-warning-forty-r12-uart-1",
                "static-process-shell-warning-forty-r12-uart-2",
                "static-process-shell-warning-forty-r12-uart-3",
                "static-process-shell-warning-forty-r12-uart-4",
                "static-process-shell-warning-forty-r12-uart-5",
                "static-process-shell-warning-forty-r12-uart-6",
                "static-process-shell-warning-forty-r12-uart-7",
            ],
        ),
    ] {
        for (index, ready) in ready_labels.into_iter().enumerate() {
            code.bind(ready)?;
            code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
            code.jump_if(0x84, ready);
            code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x44, 0x89, move_register, 0xee]);
            if index != 7 {
                code.bytes(&[0x49, 0xc1, shift_register, 8]);
            }
        }
    }
    code.bytes(&[
        0x45, 0x31, 0xc0, 0x45, 0x31, 0xc9, 0x45, 0x31, 0xd2, 0x45, 0x31, 0xdb, 0x45, 0x31, 0xe4,
    ]);
    code.bytes(&[0x65, 0xfe, 0x04, 0x25]);
    code.bytes(&619_u32.to_le_bytes());
    code.bytes(&[0xb8, 40, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-warning-newline")?;
    code.bytes(&[0x44, 0x8a, 0x06, 0x41, 0x80, 0xf8, 0x0a]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-warning-newline-uart")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(0x84, "static-process-shell-warning-newline-uart");
    code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x44, 0x88, 0xc0, 0xee]);
    code.bytes(&[0x45, 0x31, 0xc0]);
    code.bytes(&[0x65, 0xfe, 0x04, 0x25]);
    code.bytes(&619_u32.to_le_bytes());
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-prompt")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 26]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    for (offset, expected) in [
        (736_u32, 1_u8),
        (737, 0),
        (738, 1),
        (739, 1),
        (740, 1),
        (741, 0),
        (742, 0),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x48, 0x83, 0xff, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 4]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");

    // Copy and validate the complete application segment before publication.
    code.bytes(&[0x8b, 0x06]);
    code.bytes(&[0x65, 0x89, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x65, 0x81, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&u32::from_le_bytes(*b"/ # ").to_le_bytes());
    code.jump_if(0x85, "fatal");
    for (index, ready) in [
        "static-process-shell-prompt-uart-0",
        "static-process-shell-prompt-uart-1",
        "static-process-shell-prompt-uart-2",
        "static-process-shell-prompt-uart-3",
    ]
    .into_iter()
    .enumerate()
    {
        code.bind(ready)?;
        code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
        code.jump_if(0x84, ready);
        code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x65, 0x8a, 0x04, 0x25]);
        code.bytes(&(744_u32 + u32::try_from(index).expect("bounded prompt index")).to_le_bytes());
        code.bytes(&[0xee]);
    }
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&741_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0xb8, 4, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-phase-28-write")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-refill-prompt");

    code.bind("static-process-shell-phase-29-write")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x84, "static-process-shell-third-prompt");
    code.jump("fatal");

    code.bind("static-process-shell-phase-30-write")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[26]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-header-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[21]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-open-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[12]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[14]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[19]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-nonempty-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[17]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-executable-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[8]);
    code.jump_if(0x84, "static-process-shell-pwd-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[6]);
    code.jump_if(0x84, "static-process-shell-pwd-write");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[4]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x84, "static-process-shell-root-condition-write");
    code.jump("static-process-shell-fourth-prompt");

    code.bind("static-process-shell-phase-31-write")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[9]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-fourth-prompt");

    code.bind("static-process-shell-refill-prompt")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    for (offset, expected) in [
        (736_u32, 1_u8),
        (737, 0),
        (738, 1),
        (739, 1),
        (740, 1),
        (741, 1),
        (742, 0),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"echo top").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x04, 0x25]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"al\n\0\0\0\0\0").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x04, 0x25]);
    code.bytes(&0x005f_8f68_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 4]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x8b, 0x06]);
    code.bytes(&[0x65, 0x89, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x65, 0x81, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&u32::from_le_bytes(*b"/ # ").to_le_bytes());
    code.jump_if(0x85, "fatal");
    for (index, ready) in [
        "static-process-shell-refill-prompt-uart-0",
        "static-process-shell-refill-prompt-uart-1",
        "static-process-shell-refill-prompt-uart-2",
        "static-process-shell-refill-prompt-uart-3",
    ]
    .into_iter()
    .enumerate()
    {
        code.bind(ready)?;
        code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
        code.jump_if(0x84, ready);
        code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x65, 0x8a, 0x04, 0x25]);
        code.bytes(
            &(744_u32 + u32::try_from(index).expect("bounded refill prompt index")).to_le_bytes(),
        );
        code.bytes(&[0xee]);
    }
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0xb8, 4, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-root-condition-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 5]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-root-condition-write-clean");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 5]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_heap_validation(
        code,
        "static-process-shell-root-condition-write-low-heap",
        "static-process-shell-root-condition-write-high-heap",
    )?;
    code.jump("static-process-shell-root-condition-write-profile-ready");
    code.bind("static-process-shell-root-condition-write-clean")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 5]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-root-condition-write-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[4]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[4]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    // Stage all five application bytes before comparison or publication.
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-root-condition-write-copy")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x16]);
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 5]);
    code.jump_if(0x85, "static-process-shell-root-condition-write-copy");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"/bin\n\0\0\0").to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-root-condition-write-output")?;
    code.bind("static-process-shell-root-condition-write-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(0x84, "static-process-shell-root-condition-write-uart-ready");
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x65, 0x41, 0x8a, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0xee, 0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 5]);
    code.jump_if(0x85, "static-process-shell-root-condition-write-output");

    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[6]);
    code.bytes(&[0xb8, 5, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-empty-bin-glob-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 13]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-write-clean");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 13]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_heap_validation(
        code,
        "static-process-shell-empty-bin-glob-write-low-heap",
        "static-process-shell-empty-bin-glob-write-high-heap",
    )?;
    code.jump("static-process-shell-empty-bin-glob-write-profile-ready");
    code.bind("static-process-shell-empty-bin-glob-write-clean")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 13]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-empty-bin-glob-write-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 4)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    for offset in (744_u32..760).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-empty-bin-glob-write-copy")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x16]);
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 13]);
    code.jump_if(0x85, "static-process-shell-empty-bin-glob-write-copy");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[14]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-condition-write-value",
    );
    for (offset, expected) in [
        (744_u32, u64::from_le_bytes(*b"/bin/bus")),
        (752, u64::from_le_bytes(*b"ybox\n\0\0\0")),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.jump("static-process-shell-empty-bin-glob-write-value-ready");
    code.bind("static-process-shell-bin-busybox-condition-write-value")?;
    for (offset, expected) in [
        (744_u32, u64::from_le_bytes(*b"busybox-")),
        (752, u64::from_le_bytes(*b"file\n\0\0\0")),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bind("static-process-shell-empty-bin-glob-write-value-ready")?;
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-empty-bin-glob-write-output")?;
    code.bind("static-process-shell-empty-bin-glob-write-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-write-uart-ready");
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x65, 0x41, 0x8a, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0xee, 0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 13]);
    code.jump_if(0x85, "static-process-shell-empty-bin-glob-write-output");
    for offset in (744_u32..760).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[14]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-condition-write-complete",
    );
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[13]);
    code.jump("static-process-shell-empty-bin-glob-write-complete");
    code.bind("static-process-shell-bin-busybox-condition-write-complete")?;
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[15]);
    code.bind("static-process-shell-empty-bin-glob-write-complete")?;
    code.bytes(&[0xb8, 13, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-executable-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 19]);
    code.jump_if(0x85, "fatal");
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 4), (888, 17)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-executable-write-clean",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 19]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_heap_validation(
        code,
        "static-process-shell-bin-busybox-executable-write-low-heap",
        "static-process-shell-bin-busybox-executable-write-high-heap",
    )?;
    code.jump("static-process-shell-bin-busybox-executable-write-profile-ready");
    code.bind("static-process-shell-bin-busybox-executable-write-clean")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 19]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-bin-busybox-executable-write-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    for offset in (744_u32..768).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-executable-write-copy")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x16]);
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 19]);
    code.jump_if(
        0x85,
        "static-process-shell-bin-busybox-executable-write-copy",
    );
    for (offset, expected) in [
        (744_u32, u64::from_le_bytes(*b"busybox-")),
        (752, u64::from_le_bytes(*b"executab")),
        (760, u64::from_le_bytes(*b"le\n\0\0\0\0\0")),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-executable-write-output")?;
    code.bind("static-process-shell-bin-busybox-executable-write-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-executable-write-uart-ready",
    );
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x65, 0x41, 0x8a, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0xee, 0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 19]);
    code.jump_if(
        0x85,
        "static-process-shell-bin-busybox-executable-write-output",
    );
    for offset in (744_u32..768).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[18]);
    code.bytes(&[0xb8, 19, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-nonempty-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 17]);
    code.jump_if(0x85, "fatal");
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 4), (888, 19)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-nonempty-write-clean",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 17]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_heap_validation(
        code,
        "static-process-shell-bin-busybox-nonempty-write-low-heap",
        "static-process-shell-bin-busybox-nonempty-write-high-heap",
    )?;
    code.jump("static-process-shell-bin-busybox-nonempty-write-profile-ready");
    code.bind("static-process-shell-bin-busybox-nonempty-write-clean")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 17]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-bin-busybox-nonempty-write-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    for offset in (744_u32..768).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-nonempty-write-copy")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x16]);
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 17]);
    code.jump_if(0x85, "static-process-shell-bin-busybox-nonempty-write-copy");
    for (offset, expected) in [
        (744_u32, u64::from_le_bytes(*b"busybox-")),
        (752, u64::from_le_bytes(*b"nonempty")),
        (760, u64::from_le_bytes(*b"\n\0\0\0\0\0\0\0")),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-nonempty-write-output")?;
    code.bind("static-process-shell-bin-busybox-nonempty-write-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-nonempty-write-uart-ready",
    );
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x65, 0x41, 0x8a, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0xee, 0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 17]);
    code.jump_if(
        0x85,
        "static-process-shell-bin-busybox-nonempty-write-output",
    );
    for offset in (744_u32..768).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[20]);
    code.bytes(&[0xb8, 17, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-open-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 13]);
    code.jump_if(0x85, "fatal");
    for (offset, expected) in [
        (417_u32, 1_u8),
        (418, 4),
        (737, 1),
        (743, 4),
        (888, 21),
        (889, 0),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-open-write-clean");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 13]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0060_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_low_heap_validation(
        code,
        "static-process-shell-bin-busybox-open-write-low-heap",
    )?;
    code.jump("static-process-shell-bin-busybox-open-write-profile-ready");
    code.bind("static-process-shell-bin-busybox-open-write-clean")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 13]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-bin-busybox-open-write-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    for offset in (744_u32..760).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-open-write-copy")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x16]);
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 13]);
    code.jump_if(0x85, "static-process-shell-bin-busybox-open-write-copy");
    for (offset, expected) in [
        (744_u32, u64::from_le_bytes(*b"busybox-")),
        (752, u64::from_le_bytes(*b"open\n\0\0\0")),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-open-write-output")?;
    code.bind("static-process-shell-bin-busybox-open-write-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-open-write-uart-ready",
    );
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x65, 0x41, 0x8a, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0xee, 0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 13]);
    code.jump_if(0x85, "static-process-shell-bin-busybox-open-write-output");
    for offset in (744_u32..760).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[22]);
    code.bytes(&[0xb8, 13, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-header-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 13]);
    code.jump_if(0x85, "fatal");
    for (offset, expected) in [
        (417_u32, 1_u8),
        (418, 4),
        (737, 1),
        (743, 4),
        (888, 26),
        (889, 4),
        (891, 0),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-header-write-clean");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 13]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0060_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_low_heap_validation(
        code,
        "static-process-shell-bin-busybox-header-write-low-heap",
    )?;
    code.jump("static-process-shell-bin-busybox-header-write-profile-ready");
    code.bind("static-process-shell-bin-busybox-header-write-clean")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 13]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-bin-busybox-header-write-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    for offset in (744_u32..760).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-header-write-copy")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x16]);
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 13]);
    code.jump_if(0x85, "static-process-shell-bin-busybox-header-write-copy");
    for (offset, expected) in [
        (744_u32, u64::from_le_bytes(*b"busybox-")),
        (752, u64::from_le_bytes(*b"read\n\0\0\0")),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-bin-busybox-header-write-output")?;
    code.bind("static-process-shell-bin-busybox-header-write-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-header-write-uart-ready",
    );
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x65, 0x41, 0x8a, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0xee, 0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 13]);
    code.jump_if(0x85, "static-process-shell-bin-busybox-header-write-output");
    for offset in (744_u32..760).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[27]);
    code.bytes(&[0xb8, 13, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-named-root-condition-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 14]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-named-root-condition-write-clean",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 14]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_heap_validation(
        code,
        "static-process-shell-named-root-condition-write-low-heap",
        "static-process-shell-named-root-condition-write-high-heap",
    )?;
    code.jump("static-process-shell-named-root-condition-write-profile-ready");
    code.bind("static-process-shell-named-root-condition-write-clean")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 14]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-named-root-condition-write-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 4), (888, 7)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    for offset in [744_u32, 752] {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-named-root-condition-write-copy")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x16]);
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 14]);
    code.jump_if(0x85, "static-process-shell-named-root-condition-write-copy");
    for (offset, expected) in [
        (744_u32, u64::from_le_bytes(*b"bin-dire")),
        (752, u64::from_le_bytes(*b"ctory\n\0\0")),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-named-root-condition-write-output")?;
    code.bind("static-process-shell-named-root-condition-write-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(
        0x84,
        "static-process-shell-named-root-condition-write-uart-ready",
    );
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x65, 0x41, 0x8a, 0x82]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0xee, 0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 14]);
    code.jump_if(
        0x85,
        "static-process-shell-named-root-condition-write-output",
    );
    for offset in [744_u32, 752] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[8]);
    code.bytes(&[0xb8, 14, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-pwd-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-pwd-write-clean");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[8]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 2]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.jump("static-process-shell-pwd-write-profile-ready");
    code.bind("static-process-shell-pwd-write-clean")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[6]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_9460_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-pwd-write-profile-ready")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-shell-pwd-write-low-heap",
        "static-process-shell-pwd-write-high-heap",
    )?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x66, 0x8b, 0x06]);
    code.bytes(&[0x65, 0x66, 0x89, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x65, 0x66, 0x81, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&u16::from_le_bytes(*b"/\n").to_le_bytes());
    code.jump_if(0x85, "fatal");
    for (index, ready) in [
        "static-process-shell-pwd-write-uart-0",
        "static-process-shell-pwd-write-uart-1",
    ]
    .into_iter()
    .enumerate()
    {
        code.bind(ready)?;
        code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
        code.jump_if(0x84, ready);
        code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x65, 0x8a, 0x04, 0x25]);
        code.bytes(&(744_u32 + u32::try_from(index).expect("bounded pwd index")).to_le_bytes());
        code.bytes(&[0xee]);
    }
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[9]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 31]);
    code.bytes(&[0xb8, 2, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-third-prompt")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 4]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x8b, 0x06]);
    code.bytes(&[0x65, 0x89, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x65, 0x81, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&u32::from_le_bytes(*b"/ # ").to_le_bytes());
    code.jump_if(0x85, "fatal");
    for (index, ready) in [
        "static-process-shell-third-prompt-uart-0",
        "static-process-shell-third-prompt-uart-1",
        "static-process-shell-third-prompt-uart-2",
        "static-process-shell-third-prompt-uart-3",
    ]
    .into_iter()
    .enumerate()
    {
        code.bind(ready)?;
        code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
        code.jump_if(0x84, ready);
        code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x65, 0x8a, 0x04, 0x25]);
        code.bytes(
            &(744_u32 + u32::try_from(index).expect("bounded third prompt index")).to_le_bytes(),
        );
        code.bytes(&[0xee]);
    }
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[2]);
    code.bytes(&[0xb8, 4, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-fourth-prompt")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[27]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[4]);
    code.jump_if(0x84, "static-process-shell-fourth-prompt-after-test");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[9]);
    code.jump_if(0x85, "fatal");
    for (offset, expected) in [(737_u32, 1_u8), (892, 5)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xb1, 10]);
    code.jump("static-process-shell-fourth-prompt-profile-ready");
    code.bind("static-process-shell-fourth-prompt-after-test")?;
    for (offset, expected) in [(737_u32, 0_u8), (892, 4)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[4]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xb1, 5]);
    code.bind("static-process-shell-fourth-prompt-profile-ready")?;
    code.bytes(&[0x48, 0x83, 0xff, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 4]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x8b, 0x06]);
    code.bytes(&[0x65, 0x89, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x65, 0x81, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&u32::from_le_bytes(*b"/ # ").to_le_bytes());
    code.jump_if(0x85, "fatal");
    for (index, ready) in [
        "static-process-shell-fourth-prompt-uart-0",
        "static-process-shell-fourth-prompt-uart-1",
        "static-process-shell-fourth-prompt-uart-2",
        "static-process-shell-fourth-prompt-uart-3",
    ]
    .into_iter()
    .enumerate()
    {
        code.bind(ready)?;
        code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
        code.jump_if(0x84, ready);
        code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x65, 0x8a, 0x04, 0x25]);
        code.bytes(
            &(744_u32 + u32::try_from(index).expect("bounded fourth prompt index")).to_le_bytes(),
        );
        code.bytes(&[0xee]);
    }
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x44, 0x88, 0x0c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[0xb8, 4, 0, 0, 0]);
    emit_static_process_sysret(code);

    Ok(())
}
