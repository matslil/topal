fn emit_static_process_completion(
    code: &mut X86FunctionEncoder,
    source_bound: bool,
    file_open_plan: bool,
    source_numeric: bool,
    source_image: bool,
    source_dispatch: bool,
    source_identity: bool,
    retained_identity: bool,
    structured_identity: bool,
    source_position: bool,
) -> Result<(), CompileError> {
    code.bind("static-process-shell-banner")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0001_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 109]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");

    const SHELL_BANNER: &[u8; 109] =
        b"\n\nBusyBox v1.37.0 (Debian 1:1.37.0-6+b9) built-in shell (ash)\nEnter 'help' for a list of built-in commands.\n\n";
    let mut padded_shell_banner = [0_u8; 112];
    padded_shell_banner[..SHELL_BANNER.len()].copy_from_slice(SHELL_BANNER);
    for offset in (624_u32..736).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }

    // Copy the complete user payload into provider-private staging before
    // validating or publishing any byte.
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-banner-copy")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x16]);
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&624_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 109]);
    code.jump_if(0x85, "static-process-shell-banner-copy");
    for (index, chunk) in padded_shell_banner.chunks_exact(8).enumerate() {
        let expected = u64::from_le_bytes(chunk.try_into().expect("eight-byte banner word"));
        let offset = 624_u32 + u32::try_from(index * 8).expect("bounded banner offset");
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }

    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-banner-output")?;
    code.bind("static-process-shell-banner-uart-ready")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(0x84, "static-process-shell-banner-uart-ready");
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x65, 0x41, 0x8a, 0x82]);
    code.bytes(&624_u32.to_le_bytes());
    code.bytes(&[0xee, 0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 109]);
    code.jump_if(0x85, "static-process-shell-banner-output");

    code.bytes(&[0x31, 0xc0, 0x45, 0x31, 0xd2]);
    code.bind("static-process-shell-banner-erase")?;
    code.bytes(&[0x65, 0x41, 0x88, 0x82]);
    code.bytes(&624_u32.to_le_bytes());
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 112]);
    code.jump_if(0x85, "static-process-shell-banner-erase");
    for offset in (624_u32..736).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&621_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0xb8, 109, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-echo-write")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 6]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x85, "static-process-shell-echo-external-candidate");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-echo-candidate-ready");
    code.bind("static-process-shell-echo-external-candidate")?;
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 6]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_shell_output_heap_validation(code)?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bind("static-process-shell-echo-candidate-ready")?;
    // Stage and validate all six bytes before making output observable.
    code.bytes(&[0x44, 0x8b, 0x06, 0x44, 0x0f, 0xb7, 0x4e, 4]);
    code.bytes(&[0x41, 0x81, 0xf8]);
    code.bytes(&u32::from_le_bytes(*b"topa").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0x81, 0xf9]);
    code.bytes(&0x0000_0a6c_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    for (index, ready) in [
        "static-process-shell-echo-uart-low-0",
        "static-process-shell-echo-uart-low-1",
        "static-process-shell-echo-uart-low-2",
        "static-process-shell-echo-uart-low-3",
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
    for (index, ready) in [
        "static-process-shell-echo-uart-high-0",
        "static-process-shell-echo-uart-high-1",
    ]
    .into_iter()
    .enumerate()
    {
        code.bind(ready)?;
        code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
        code.jump_if(0x84, ready);
        code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0x44, 0x89, 0xc8, 0xee]);
        if index == 0 {
            code.bytes(&[0x49, 0xc1, 0xe9, 8]);
        }
    }
    code.bytes(&[0x45, 0x31, 0xc0, 0x45, 0x31, 0xc9]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 28]);
    code.bytes(&[0xb8, 6, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-getpid")?;
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x85, "static-process-shell-getpid-external");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 16]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-getpid-phase-ready");
    code.bind("static-process-shell-getpid-external")?;
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 11]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-getpid-phase-ready")?;
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4c, 0x39, 0xd0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0, 0x48, 0x89, 0xc2]);
    code.bytes(&[0x48, 0x83, 0xe2, 0x9f, 0x48, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x89, 0xd6, 0x81, 0xe6, 0xff, 0x0f, 0, 0, 0x83, 0xfe, 7]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 17]);
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-sigchld")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 17]);
    code.jump_if(0x84, "static-process-shell-sigchld-install");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[6]);
    code.jump_if(0x84, "static-process-execve-sigchld-reset");
    code.jump("static-process-shell-signal-tail");
    code.bind("static-process-execve-sigchld-reset")?;
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x84, "static-process-execve-signal-number-ready");
    code.bytes(&[0x48, 0x83, 0xff, 17]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-execve-signal-number-ready")?;
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x83, 0xfa, 8]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_ffe0_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x83, 0x3e, 0]);
        } else if offset == 8 {
            code.bytes(&[0x48, 0x81, 0x7e, 8, 0, 0, 0, 4]);
        } else if offset == 16 {
            code.bytes(&[0x48, 0x81, 0x7e, 16]);
            code.bytes(&0x004a_9740_u32.to_le_bytes());
        } else {
            code.bytes(&[0x48, 0xb8]);
            code.bytes(&0xffff_fffe_7fff_ffff_u64.to_le_bytes());
            code.bytes(&[0x48, 0x39, 0x46, 24]);
        }
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "static-process-execve-reset-child-disposition");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump("static-process-execve-reset-disposition-ready");
    code.bind("static-process-execve-reset-child-disposition")?;
    for offset in [168_u32, 176] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    code.bind("static-process-execve-reset-disposition-ready")?;
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);
    code.bind("static-process-shell-sigchld-install")?;
    code.bytes(&[0x48, 0x83, 0xff, 17]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x83, 0xfa, 8]);
    code.jump_if(0x85, "fatal");
    for offset in [168_u32, 176, 224, 232] {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");

    // Revalidate the active hierarchy and exact writable, NX stack leaf before
    // copying the complete native request into provider registers.
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25]);
    code.bytes(&24_u32.to_le_bytes());
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc2, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4c, 0x39, 0xd0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0x18, 0x20, 0, 0, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[
        0x4d, 0x8d, 0x82, 0, 0x40, 0, 0, 0x49, 0x83, 0xc8, 7, 0x4c, 0x39, 0xc0,
    ]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_ffe0_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bytes(&[
        0x4c, 0x8b, 0x06, 0x4c, 0x8b, 0x4e, 0x08, 0x4c, 0x8b, 0x5e, 0x10, 0x4c, 0x8b, 0x66, 0x18,
    ]);
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x49, 0x81, 0xf9]);
    code.bytes(&0x0400_0000_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x81, 0xfb]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xfb]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0xffff_fffe_7fff_ffff_u64.to_le_bytes());
    code.bytes(&[0x49, 0x39, 0xc4]);
    code.jump_if(0x85, "fatal");
    for (register, offset) in [(0x04_u8, 168_u32), (0x0c, 176), (0x1c, 224), (0x24, 232)] {
        code.bytes(&[0x65, 0x4c, 0x89, register, 0x25]);
        code.bytes(&offset.to_le_bytes());
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 18]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-initial-process-name")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 9]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 0x10]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&896_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "runtime-initial-process-name-extent");
    process_runtime::emit_stack_output_extent(code, false);
    code.jump("static-process-name-buffer-ready");
    code.bind("runtime-initial-process-name-extent")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fe60_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-name-buffer-ready");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fd50_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-name-buffer-ready");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fd58_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bind("static-process-name-buffer-ready")?;
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");

    // Validate the active hierarchy and the exact writable, NX stack leaf.
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc2, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4c, 0x39, 0xd0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0x18, 0x20, 0, 0, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[
        0x4d, 0x8d, 0x82, 0, 0x40, 0, 0, 0x49, 0x83, 0xc8, 7, 0x4c, 0x39, 0xc0,
    ]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0, 0x48, 0x89, 0xc2]);
    code.bytes(&[0x48, 0x83, 0xe2, 0x9f, 0x48, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x89, 0xd7, 0x81, 0xe7, 0xff, 0x0f, 0, 0, 0x83, 0xff, 7]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x48, 0x0f, 0xba, 0xf2, 0x3f, 0x48, 0x81, 0xe2, 0, 0xf0, 0xff, 0xff,
    ]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");

    // The reusable random service remains live and quiescent.
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x3c, 3]);
    code.jump_if(0x87, "fatal");
    for offset in [168_u32, 176, 224, 232, 240, 248, 272] {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for offset in [256_u32, 264] {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x84, "fatal");
    }

    // Publish and read back the complete immutable native field.
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0078_6f62_7973_7562_u64.to_le_bytes());
    code.bytes(&[0x48, 0x89, 0x06, 0x48, 0xc7, 0x46, 0x08, 0, 0, 0, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0, 0x48, 0x39, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0x7e, 0x08, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 10]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-initial-getuid")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 10]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");

    // Validate the active hierarchy and recover the exact retained stack
    // frame without consulting or retaining any native argument register.
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc2, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4c, 0x39, 0xd0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0x18, 0x20, 0, 0, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[
        0x4d, 0x8d, 0x82, 0, 0x40, 0, 0, 0x49, 0x83, 0xc8, 7, 0x4c, 0x39, 0xc0,
    ]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0, 0x48, 0x89, 0xc2]);
    code.bytes(&[0x48, 0x83, 0xe2, 0x9f, 0x48, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x89, 0xd6, 0x81, 0xe6, 0xff, 0x0f, 0, 0, 0x83, 0xfe, 7]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xe2, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0xba]);
    code.bytes(&0x007f_f000_u32.to_le_bytes());

    // The sealed real/effective user/group facts must remain complete and
    // coherent with the zero-valued real UID returned below.
    process_runtime::emit_credential_base(
        code,
        "static-process-credential-stack-generation-ready",
    )?;
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
        code.bytes(&[0x48, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[value]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 11]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-geteuid")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[16]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-executable-geteuid");
    code.jump("static-process-shell-post-banner-geteuid");

    code.bind("static-process-shell-bin-busybox-executable-geteuid")?;
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 4)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[17]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-post-banner-geteuid")?;
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
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&736_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0xba]);
    code.bytes(&0x007f_f000_u32.to_le_bytes());
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
        code.bytes(&[0x48, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[value]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&736_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-newfstatat")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[6]);
    code.jump_if(0x84, "static-process-shell-named-root-directory-stat");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[13]);
    code.jump_if(0x84, "static-process-shell-named-root-directory-stat");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[15]);
    code.jump_if(0x84, "static-process-shell-named-root-directory-stat");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[18]);
    code.jump_if(0x84, "static-process-shell-named-root-directory-stat");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 29]);
    code.jump_if(0x84, "static-process-shell-root-directory-stat");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 11]);
    code.jump_if(0x84, "static-process-initial-config-stat-miss");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 26]);
    code.jump_if(0x84, "static-process-shell-mailpath-stat-miss");
    code.jump("fatal");

    code.bind("static-process-shell-named-root-directory-stat")?;
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 4)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-named-root-directory-stat-profile-ready",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-named-root-directory-stat-profile-ready")?;
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[6]);
    code.jump_if(0x84, "static-process-shell-named-root-directory-stat-path");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[13]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-stat-path");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[15]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-stat-path");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[18]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-bin-busybox-stat-path")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 13]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"/bin/bus").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0x7e, 8]);
    code.bytes(&u32::from_le_bytes(*b"ybox").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x80, 0x7e, 12, 0]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-named-root-directory-stat-path-ready");
    code.bind("static-process-shell-named-root-directory-stat-path")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 5]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bytes(&[0x81, 0x3e]);
    code.bytes(&u32::from_le_bytes(*b"/bin").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x80, 0x7e, 4, 0]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-named-root-directory-stat-path-ready")?;
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x81, 0xc0]);
    code.bytes(&144_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-shell-named-root-directory-stat-low-heap",
        "static-process-shell-named-root-directory-stat-high-heap",
    )?;
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[13]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-newfstatat-values");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[15]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-newfstatat-values");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[18]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-newfstatat-values");
    for (offset, value) in [(752_u32, 4_u32), (760, 2), (768, 0x41ed), (800, 4096)] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&value.to_le_bytes());
    }
    code.jump("static-process-shell-named-root-newfstatat-values-ready");
    code.bind("static-process-shell-bin-busybox-newfstatat-values")?;
    for (offset, value) in [
        (752_u32, 5_u32),
        (760, 1),
        (768, 0x816d),
        (792, 2_024_544),
        (800, 4096),
        (808, 3_955),
    ] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&value.to_le_bytes());
    }
    code.bind("static-process-shell-named-root-newfstatat-values-ready")?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25]);
    code.bytes(&328_u32.to_le_bytes());
    for (index, staging_offset) in (744_u32..888).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded named directory stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x89, 0x02]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x89, 0x42, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x89, 0x82]);
            code.bytes(&user_offset.to_le_bytes());
        }
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    for (index, staging_offset) in (744_u32..888).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded named directory stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x39, 0x02]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x39, 0x42, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x39, 0x82]);
            code.bytes(&user_offset.to_le_bytes());
        }
        code.jump_if(0x85, "fatal");
    }
    for offset in (744_u32..888).step_by(8) {
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
    code.bytes(&[13]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-stat-complete");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[15]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-repeat-stat-complete",
    );
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[18]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-nonempty-stat-complete",
    );
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[7]);
    code.jump("static-process-shell-named-root-stat-complete");
    code.bind("static-process-shell-bin-busybox-stat-complete")?;
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[14]);
    code.jump("static-process-shell-named-root-stat-complete");
    code.bind("static-process-shell-bin-busybox-repeat-stat-complete")?;
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[16]);
    code.jump("static-process-shell-named-root-stat-complete");
    code.bind("static-process-shell-bin-busybox-nonempty-stat-complete")?;
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[19]);
    code.bind("static-process-shell-named-root-stat-complete")?;
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-root-directory-stat")?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-root-directory-stat-clean");
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
    code.jump("static-process-shell-root-directory-stat-profile-ready");
    code.bind("static-process-shell-root-directory-stat-clean")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-root-directory-stat-profile-ready")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[4]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 9]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x81, 0xc0]);
    code.bytes(&144_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-shell-root-directory-stat-low-heap",
        "static-process-shell-root-directory-stat-high-heap",
    )?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"/missing").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x80, 0x7e, 8, 0]);
    code.jump_if(0x85, "fatal");
    // Negative lookup publishes no metadata: prove all private transfer
    // staging remains empty before recording the missing-path observation.
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[4]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 30]);
    code.bytes(&[0x48, 0xc7, 0xc0, 0xfe, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-mailpath-stat-miss")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x81, 0x3c, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&0x0303_0101_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x004a_6920_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"/var/spo").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"ol/mail/").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x46, 8]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0x7e, 16]);
    code.bytes(&u32::from_le_bytes(*b"root").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x80, 0x7e, 20, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&620_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&620_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x48, 0xc7, 0xc0, 0xfe, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-initial-config-stat-miss")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 11]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]); // 32-bit AT_FDCWD = -100
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]); // flags = 0
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_ff70_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");

    // Bind each resident-header form to its exact immutable pathname candidate.
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x84, "static-process-clean-config-path");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005c_0aa7_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.jump("static-process-config-path-ready");
    code.bind("static-process-clean-config-path")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x004a_6900_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bind("static-process-config-path-ready")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x7375_622f_6374_652f_u64.to_le_bytes()); // "/etc/bus"
    code.bytes(&[0x48, 0x39, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x6e6f_632e_786f_6279_u64.to_le_bytes()); // "ybox.con"
    code.bytes(&[0x48, 0x39, 0x46, 8]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x66, 0x83, 0x7e, 0x10, 0x66]); // "f\0"
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 12]);
    code.bytes(&[0x48, 0xc7, 0xc0, 0xfe, 0xff, 0xff, 0xff]); // -ENOENT
    emit_static_process_sysret(code);

    code.bind("static-process-initial-credential-tail")?;
    code.bytes(&[0x3d, 104, 0, 0, 0]);
    code.jump_if(0x84, "static-process-initial-getgid");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x3d, 106, 0, 0, 0]);
    code.jump_if(0x84, "static-process-initial-setgid");
    code.bytes(&[0x3d, 105, 0, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 14]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xb1, 15]);
    code.jump("static-process-initial-credential-tail-common");
    code.bind("static-process-initial-getgid")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 12]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xb1, 13]);
    code.jump("static-process-initial-credential-tail-common");
    code.bind("static-process-initial-setgid")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 13]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xb1, 14]);
    code.bind("static-process-initial-credential-tail-common")?;
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");

    // Revalidate the active hierarchy, retained stack, and complete coherent
    // credential group before observing or idempotently selecting a view.
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc2, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4c, 0x39, 0xd0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0x18, 0x20, 0, 0, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[
        0x4d, 0x8d, 0x82, 0, 0x40, 0, 0, 0x49, 0x83, 0xc8, 7, 0x4c, 0x39, 0xc0,
    ]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0, 0x48, 0x89, 0xc2]);
    code.bytes(&[0x48, 0x83, 0xe2, 0x9f, 0x48, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x89, 0xd6, 0x81, 0xe6, 0xff, 0x0f, 0, 0, 0x83, 0xfe, 7]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x0f, 0xba, 0xf2, 0x3f]);
    code.bytes(&[0x48, 0x81, 0xe2, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    process_runtime::emit_credential_base(code, "static-process-tail-stack-generation-ready")?;
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
        code.bytes(&[0x48, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[value]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x44, 0x88, 0x0c, 0x25, 0x68, 0, 0, 0]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-brk-query")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 0]);
    code.jump_if(0x84, "static-process-brk-initial-query");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 6]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0xb8]);
    code.bytes(&0x005f_7d80_u32.to_le_bytes());
    emit_static_process_sysret(code);

    code.bind("static-process-brk-initial-query")?;
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 1]);
    code.bytes(&[0xb8]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    emit_static_process_sysret(code);

    code.bind("static-process-set-fs")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x0000_1002_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7400_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x89, 0x0c, 0x25, 0x88, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x89, 0x14, 0x25, 0x90, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x70, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x78, 0, 0, 0]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4c, 0x89, 0xc0, 0x48, 0x83, 0xc8, 0x07, 0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x09, 0xc8, 0x4c, 0x8b, 0x8a, 0xb8, 0x0f, 0, 0]);
    code.bytes(&[0x49, 0x83, 0xe1, 0x9f, 0x49, 0x39, 0xc1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x89, 0x34, 0x25, 0x80, 0, 0, 0]);
    code.bytes(&[0x89, 0xf0, 0x48, 0x89, 0xf2, 0x48, 0xc1, 0xea, 0x20, 0xb9]);
    code.bytes(&0xc000_0100_u32.to_le_bytes());
    code.bytes(&[0x0f, 0x30]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x90, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25, 0x88, 0, 0, 0]);
    for context_offset in [136_u8, 144] {
        code.bytes(&[
            0x65,
            0x48,
            0xc7,
            0x04,
            0x25,
            context_offset,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]);
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 3]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-set-tid-address")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x005f_76d0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x70, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x78, 0, 0, 0]);
    code.bytes(&[0x4c, 0x89, 0xc0, 0x48, 0x83, 0xc8, 0x07, 0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x09, 0xc8, 0x4c, 0x8b, 0x8a, 0xb8, 0x0f, 0, 0]);
    code.bytes(&[0x49, 0x83, 0xe1, 0x9f, 0x49, 0x39, 0xc1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x89, 0x3c, 0x25, 0x98, 0, 0, 0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 4]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
    code.bytes(&process_files::TABLE_CONTEXT.to_le_bytes());
    code.bytes(&[0x48, 0x8b, 0x40, 56]);
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "static-process-tid-known");
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    code.bind("static-process-tid-known")?;
    emit_static_process_sysret(code);

    code.bind("static-process-set-robust-list")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 4]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x005f_76e0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfe, 0x18]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x70, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x78, 0, 0, 0]);
    code.bytes(&[0x4c, 0x89, 0xc0, 0x48, 0x83, 0xc8, 0x07, 0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x09, 0xc8, 0x4c, 0x8b, 0x8a, 0xb8, 0x0f, 0, 0]);
    code.bytes(&[0x49, 0x83, 0xe1, 0x9f, 0x49, 0x39, 0xc1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x89, 0x3c, 0x25, 0xa0, 0, 0, 0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 5]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-rseq-fallback")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xc7, 0xc0, 0xda, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-stack-limit")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfe, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&896_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "runtime-stack-limit-extent");
    process_runtime::emit_stack_output_extent(code, true);
    code.jump("static-process-stack-limit-output-ready");
    code.bind("runtime-stack-limit-extent")?;
    code.bytes(&[0x49, 0x81, 0xfa]);
    code.bytes(&0x007f_fe60_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-stack-limit-output-ready");
    code.bytes(&[0x49, 0x81, 0xfa]);
    code.bytes(&0x007f_fd50_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-stack-limit-output-ready");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x81, 0xfa]);
    code.bytes(&0x007f_fe00_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bind("static-process-stack-limit-output-ready")?;
    code.bytes(&[0x49, 0xc7, 0x02, 0, 0, 0x80, 0]);
    code.bytes(&[0x49, 0xc7, 0x42, 0x08, 0xff, 0xff, 0xff, 0xff]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 6]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-readlinkat-fallback")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 6]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xc7, 0xc0, 0xda, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-getrandom")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 6]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x83, 0xe0, 0xf8]);
    code.jump_if(0x85, "static-process-getrandom-einval");
    code.bytes(&[
        0x49, 0x89, 0xd0, 0x49, 0x83, 0xe0, 0x06, 0x49, 0x83, 0xf8, 0x06,
    ]);
    code.jump_if(0x84, "static-process-getrandom-einval");
    code.bytes(&[0x48, 0x83, 0xfe, 16]);
    code.jump_if(0x87, "static-process-getrandom-einval");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "static-process-getrandom-zero");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x005e_5660_u32.to_le_bytes());
    code.jump_if(0x82, "static-process-getrandom-efault");
    code.bytes(&[0x49, 0x89, 0xf8, 0x49, 0x01, 0xf0]);
    code.jump_if(0x82, "static-process-getrandom-efault");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x005f_61c8_u32.to_le_bytes());
    code.jump_if(0x87, "static-process-getrandom-efault");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-getrandom-initial");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x3c, 3]);
    code.jump_if(0x87, "fatal");

    // Later calls extract on the suspended privileged stack, preserving every
    // native SYSRETQ input across the provider-local ChaCha20 transition.
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x89, 0x24, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x8b, 0x24, 0x25, 0x08, 0, 0, 0]);
    code.bytes(&[0x41, 0x53, 0x51, 0x57, 0x56]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x3c, 0x25]);
    code.bytes(&256_u32.to_le_bytes());
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x48, 0x8d, 0xb7]);
    code.bytes(&224_u32.to_le_bytes());
    code.bytes(&[0x48, 0x81, 0xc7]);
    code.bytes(&192_u32.to_le_bytes());
    code.call("static-process-getrandom-extract");
    code.bytes(&[0x5e, 0x5f, 0x59, 0x41, 0x5b]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x24, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x65, 0x4c, 0x8b, 0x0c, 0x25]);
    code.bytes(&256_u32.to_le_bytes());
    code.bytes(&[0x49, 0x81, 0xc1]);
    code.bytes(&224_u32.to_le_bytes());
    code.jump("static-process-getrandom-copy");

    code.bind("static-process-getrandom-initial")?;
    code.bytes(&[0x65, 0x4c, 0x8b, 0x0c, 0x25]);
    code.bytes(&256_u32.to_le_bytes());
    code.bytes(&[0x49, 0x81, 0xc1]);
    code.bytes(&168_u32.to_le_bytes());

    code.bind("static-process-getrandom-copy")?;
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("static-process-getrandom-copy-loop")?;
    code.bytes(&[0x49, 0x39, 0xf2]);
    code.jump_if(0x84, "static-process-getrandom-copy-complete");
    code.bytes(&[0x43, 0x8a, 0x04, 0x11, 0x42, 0x88, 0x04, 0x17]);
    code.bytes(&[0x42, 0x3a, 0x04, 0x17]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.jump("static-process-getrandom-copy-loop");
    code.bind("static-process-getrandom-copy-complete")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x84, "static-process-getrandom-wipe-initial");
    for offset in [224_u32, 232, 240, 248] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[3]);
    code.jump("static-process-getrandom-account");
    code.bind("static-process-getrandom-wipe-initial")?;
    for offset in [168_u32, 176] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[2]);
    code.bind("static-process-getrandom-account")?;
    code.bytes(&[0x65, 0x48, 0xff, 0x04, 0x25]);
    code.bytes(&264_u32.to_le_bytes());
    code.bytes(&[0x48, 0x89, 0xf0]);
    code.jump("static-process-getrandom-return");
    code.bind("static-process-getrandom-zero")?;
    code.bytes(&[0x31, 0xc0]);
    code.jump("static-process-getrandom-return");
    code.bind("static-process-getrandom-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("static-process-getrandom-return");
    code.bind("static-process-getrandom-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.bind("static-process-getrandom-return")?;
    emit_static_process_sysret(code);

    code.bind("returned")?;
    code.bytes(&[0x4c, 0x8b, 0x54, 0x24, 0x30]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    for offset in (0x0f28_u32..0x0f68).step_by(8) {
        code.bytes(&[0x48, 0x83, 0xa6]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0xfd]);
    }
    code.bytes(&[0x49, 0x8b, 0xb2]);
    code.bytes(&904_u32.to_le_bytes());
    code.bytes(&[0x48, 0x81, 0xc6, 0, 0xa0, 0, 0]);
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x20, 0xb9, 0, 0x10, 0, 0, 0xfc, 0xf3, 0xa4,
    ]);
    code.bytes(&[0x49, 0x8b, 0xba]);
    code.bytes(&904_u32.to_le_bytes());
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&(0xb000_u32 / 8).to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab, 0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0x8b, 0xba]);
    code.bytes(&904_u32.to_le_bytes());
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&(0xb000_u32 / 8).to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0xba]);
    code.bytes(&process_files::TABLE_CONTEXT.to_le_bytes());
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&((process_runtime::children::STORAGE_BYTES / 8) as u32).to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");
    if source_numeric || source_image || source_dispatch || source_identity {
        process_runtime::numeric::emit_terminal_readback(code)?;
    }
    if retained_identity {
        process_runtime::retained::emit_release(code);
    }
    for offset in [
        896_u32,
        904,
        912,
        process_runtime::AUX_CONTEXT,
        process_files::TABLE_CONTEXT,
        process_files::ARCHIVE_CONTEXT,
        process_runtime::time::CLOCK_CONTEXT,
        process_runtime::policy::POLICY_CONTEXT,
        process_runtime::numeric::CODE_CONTEXT,
        process_runtime::numeric::ARENA_CONTEXT,
        process_runtime::header::CODE_CONTEXT,
        process_runtime::dispatch::CODE_CONTEXT,
        process_runtime::identity::CODE_CONTEXT,
        process_runtime::retained::STORE_CONTEXT,
    ] {
        code.bytes(&[0x49, 0xc7, 0x82]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x49, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x41, 0x80, 0x3a, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0x80, 0x7a, 0x68, 11]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x41, 0x80, 0x7a, 0x68, 32]);
    code.jump_if(0x87, "fatal");
    for offset in (168_u32..=272).step_by(8) {
        code.bytes(&[0x49, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for offset in (296_u32..=408).step_by(8) {
        code.bytes(&[0x49, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for offset in (472_u32..=616).step_by(8) {
        code.bytes(&[0x49, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x41, 0x80, 0xba]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x41, 0x80, 0xba]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[2]);
    code.jump_if(0x87, "fatal");
    code.bytes(&[0x49, 0xc7, 0x82]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x3b, 0x42, 0x10]);
    code.jump_if(0x85, "fatal");

    // Scrub the process-owned SIMD state and restore the exact prior CR4
    // before releasing the private context.
    emit_static_process_clean_simd_state(code);
    code.bytes(&[0x49, 0x8b, 0x82, 0x18, 0x01, 0, 0]);
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x0f, 0x22, 0xe0]);
    for context_offset in [280_u32, 288] {
        code.bytes(&[0x49, 0xc7, 0x82]);
        code.bytes(&context_offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    for (msr, context_offset) in [
        (0xc000_0102_u32, 56_u8),
        (0xc000_0100, 64),
        (0xc000_0080, 72),
        (0xc000_0081, 80),
        (0xc000_0082, 88),
        (0xc000_0084, 96),
    ] {
        code.bytes(&[0x49, 0x8b, 0x42, context_offset]);
        emit_wrmsr_register(code, msr, &[]);
        code.bytes(&[0x49, 0xc7, 0x42, context_offset, 0, 0, 0, 0]);
    }
    for context_offset in [8_u8, 48] {
        code.bytes(&[0x49, 0xc7, 0x42, context_offset, 0, 0, 0, 0]);
    }
    // A reusable user FS base is an opaque value within the qualified native limit.
    code.bytes(&[0x49, 0x8b, 0x82, 0x80, 0, 0, 0, 0x49, 0xbb]);
    code.bytes(&0x0000_8000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x39, 0xd8]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x49, 0xc7, 0x82, 0x80, 0, 0, 0, 0, 0, 0, 0]);
    for context_offset in [128_u32, 136, 144, 152, 160] {
        code.bytes(&[0x49, 0x83, 0xba]);
        code.bytes(&context_offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x49, 0x8b, 0x72, 0x70]);
    code.bytes(&[0x48, 0x8d, 0xbe, 0xb8, 0x0f, 0, 0, 0x31, 0xc0]);
    code.bytes(&[0xb9, 9, 0, 0, 0, 0xfc, 0xf3, 0x48, 0xab]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x48, 0x8d, 0xbe, 0xb8, 0x0f, 0, 0, 0x31, 0xc0]);
    code.bytes(&[0xb9, 9, 0, 0, 0, 0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x42, 0x18]);
    code.bytes(&[0x48, 0x8d, 0xb8, 0x00, 0x40, 0, 0, 0x31, 0xc0]);
    code.bytes(&[0xb9, 25, 0, 0, 0, 0xfc, 0xf3, 0x48, 0xab]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0x8b, 0x42, 0x18]);
    code.bytes(&[0x48, 0x8d, 0xb8, 0x00, 0x40, 0, 0, 0x31, 0xc0]);
    code.bytes(&[0xb9, 25, 0, 0, 0, 0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x8b, 0x7a, 0x78, 0x31, 0xc0, 0xb9, 0x00, 0x44, 0, 0, 0xfc, 0xf3, 0x48, 0xab,
    ]);
    code.bytes(&[
        0x49, 0x8b, 0x7a, 0x78, 0x31, 0xc0, 0xb9, 0x00, 0x44, 0, 0, 0xfc, 0xf3, 0x48, 0xaf,
    ]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xc6, 0x42, 0x68, 6]);
    for context_offset in [112_u8, 120] {
        code.bytes(&[0x49, 0xc7, 0x42, context_offset, 0, 0, 0, 0]);
    }

    // BusyBox uses the bounded zero prefix below the immutable initial stack
    // for ordinary call frames and stack-local syscall outputs. Restore that
    // prefix before returning the affine completed-process owner.
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x20, 0x48, 0x81, 0xef]);
    code.bytes(&0x1000_u32.to_le_bytes());
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&512_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab, 0x0f, 0xae, 0xf0]);
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x20, 0x48, 0x81, 0xef]);
    code.bytes(&0x1000_u32.to_le_bytes());
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&512_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x20, 0x31, 0xc0]);
    code.bytes(&[0xb9]);
    code.bytes(&466_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab, 0x0f, 0xae, 0xf0]);
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x20, 0x31, 0xc0]);
    code.bytes(&[0xb9]);
    code.bytes(&466_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");

    // Hardware may set Accessed on any leaf touched by the selected external
    // executable and Dirty on writable leaves. Normalize those two
    // provider-private status bits across the complete bounded leaf table so
    // the completed owner retains the exact sealed mapping for qualification.
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    for offset in (0_u32..=0x0fb0).step_by(8) {
        code.bytes(&[0x48, 0x83, 0xa6]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0x9f]);
        code.bytes(&[0x48, 0xf7, 0x86]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&0x60_u32.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    for offset in [0x4ff0_u32, 0x4ff8] {
        code.bytes(&[0x48, 0x8b, 0x04, 0x24]);
        code.bytes(&[0x48, 0x83, 0xa0]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0x9f]);
        code.bytes(&[0x48, 0xf7, 0x80]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&0x60_u32.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    emit_static_process_restore_caller_registers(code);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);

    code.bind("fatal")?;
    code.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.bind("static-process-shell-uts-data")?;
    code.bytes(&initial_utsname_bytes());
    code.bind("static-process-getrandom-extract")?;
    code.bytes(&entropy_extract_chacha20_generator()?);
    process_runtime::emit_console_write(code)?;
    process_runtime::emit_extent_subroutine(code)?;
    process_files::emit_initialize(code)?;
    process_files::emit_lifetime_services(code)?;
    process_files::emit_transfer_services(code, source_position)?;
    process_files::emit_reclamation(code, source_position)?;
    process_runtime::signals::emit_services(code)?;
    process_runtime::namespace::emit_services(code, source_bound, file_open_plan)?;
    process_runtime::realtime::emit_services(code)?;
    process_runtime::credentials::emit_services(code)?;
    process_runtime::observations::emit_services(code)?;
    process_runtime::time::emit_services(code, source_numeric)?;
    process_runtime::children::emit_services(
        code,
        source_identity,
        retained_identity,
        structured_identity,
    )?;
    process_runtime::pipes::emit_services(code)?;
    process_runtime::volatile_files::emit_services(code, file_open_plan, source_position)?;
    process_runtime::environment::emit_copy(code)?;
    Ok(())
}
