fn emit_static_process_files(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    code.bind("static-process-shell-console-fstat")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[8]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-fstat");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 29]);
    code.jump_if(0x84, "static-process-shell-root-glob-fstat");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 30]);
    code.jump_if(0x84, "static-process-shell-pwd-console-fstat");
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x84, "static-process-shell-profile-fstat");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 26]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-console-fstat-candidate-ready");

    code.bind("static-process-shell-pwd-console-fstat")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[2]);
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
    code.bytes(&[4]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_f240_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_command_buffer_validation(code);
    code.bind("static-process-shell-console-fstat-candidate-ready")?;
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");

    // Build the complete native value in provider-private staging before the
    // exact checked user candidate is changed.
    for offset in (472_u32..616).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for (offset, value) in [
        (480_u32, 1_u32),
        (488, 1),
        (496, 0x2180),
        (512, 0x4400),
        (528, 4096),
    ] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&value.to_le_bytes());
    }
    for (index, staging_offset) in (472_u32..616).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded native stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x89, 0x46, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x89, 0x86]);
            code.bytes(&user_offset.to_le_bytes());
        }
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    for (index, staging_offset) in (472_u32..616).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded native stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x39, 0x06]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x39, 0x46, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x39, 0x86]);
            code.bytes(&user_offset.to_le_bytes());
        }
        code.jump_if(0x85, "fatal");
    }
    for offset in (472_u32..616).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 30]);
    code.jump_if(0x84, "static-process-shell-pwd-console-fstat-complete");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump("static-process-shell-console-fstat-complete");
    code.bind("static-process-shell-pwd-console-fstat-complete")?;
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[7]);
    code.bind("static-process-shell-console-fstat-complete")?;
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-dev-tty-open-miss")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[4]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-reopen");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[20]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-open");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[7]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-open");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 29]);
    code.jump_if(0x84, "static-process-shell-root-glob-open");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-profile-open");
    code.bind("static-process-shell-dev-tty-open-miss-exact")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 26]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x81, 0x3c, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&1_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x004a_6912_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"/dev/tty").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x80, 0x7e, 8, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&617_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x48, 0xc7, 0xc0, 0xfe, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-profile-open")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 26]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&736_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x004a_6938_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"/etc/pro").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0x7e, 8]);
    code.bytes(&u32::from_le_bytes(*b"file").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x80, 0x7e, 12, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0xb8, 3, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-open")?;
    for (offset, expected) in [
        (417_u32, 1_u8),
        (418, 4),
        (737, 0),
        (743, 4),
        (888, 20),
        (889, 0),
        (890, 0),
        (891, 0),
        (892, 0),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-open-clean");
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
        "static-process-shell-bin-busybox-open-low-heap",
    )?;
    code.jump("static-process-shell-bin-busybox-open-profile-ready");
    code.bind("static-process-shell-bin-busybox-open-clean")?;
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_9460_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-bin-busybox-open-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"/bin/bus").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0x7e, 8]);
    code.bytes(&u32::from_le_bytes(*b"ybox").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x80, 0x7e, 12, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[21]);
    code.bytes(&[0xb8, 3, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-reopen")?;
    for (offset, expected) in [
        (417_u32, 1_u8),
        (418, 5),
        (737, 0),
        (888, 27),
        (889, 0),
        (890, 0),
        (891, 0),
        (892, 4),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-reopen-clean");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[4]);
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
        "static-process-shell-bin-busybox-reopen-low-heap",
    )?;
    code.jump("static-process-shell-bin-busybox-reopen-profile-ready");
    code.bind("static-process-shell-bin-busybox-reopen-clean")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_9460_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-bin-busybox-reopen-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"/bin/bus").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0x7e, 8]);
    code.bytes(&u32::from_le_bytes(*b"ybox").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x80, 0x7e, 12, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[5]);
    code.bytes(&[0xb8, 3, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-root-glob-open")?;
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 3), (888, 0)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-root-glob-open-profile-ready");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-root-glob-open-profile-ready")?;
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x0009_0800_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 2]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-shell-root-glob-open-low-heap",
        "static-process-shell-root-glob-open-high-heap",
    )?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x66, 0x81, 0x3e]);
    code.bytes(&u16::from_le_bytes(*b"/\0").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0xb8, 3, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-empty-bin-glob-open")?;
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 4), (888, 7)] {
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
        "static-process-shell-empty-bin-glob-open-profile-ready",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-empty-bin-glob-open-profile-ready")?;
    code.bytes(&[0x81, 0xff, 0x9c, 0xff, 0xff, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x0009_0800_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 6]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-shell-empty-bin-glob-open-low-heap",
        "static-process-shell-empty-bin-glob-open-high-heap",
    )?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x81, 0x3e]);
    code.bytes(&u32::from_le_bytes(*b"/bin").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x66, 0x81, 0x7e, 4]);
    code.bytes(&u16::from_le_bytes(*b"/\0").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[8]);
    code.bytes(&[0xb8, 3, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-root-glob-fstat")?;
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 3), (888, 1)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-root-glob-fstat-profile-ready");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-root-glob-fstat-profile-ready")?;
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x05]);
    code.bytes(&144_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for (offset, value) in [(752_u32, 3_u32), (760, 3), (768, 0x41ed), (800, 4096)] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&value.to_le_bytes());
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    for (index, staging_offset) in (744_u32..888).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded root directory stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x89, 0x46, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x89, 0x86]);
            code.bytes(&user_offset.to_le_bytes());
        }
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    for (index, staging_offset) in (744_u32..888).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded root directory stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x39, 0x06]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x39, 0x46, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x39, 0x86]);
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
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[2]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-empty-bin-glob-fstat")?;
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 4), (888, 8)] {
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
        "static-process-shell-empty-bin-glob-fstat-profile-ready",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-empty-bin-glob-fstat-profile-ready")?;
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x05]);
    code.bytes(&144_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for (offset, value) in [(752_u32, 4_u32), (760, 2), (768, 0x41ed), (800, 4096)] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&value.to_le_bytes());
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    for (index, staging_offset) in (744_u32..888).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded bin directory stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x89, 0x46, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x89, 0x86]);
            code.bytes(&user_offset.to_le_bytes());
        }
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    for (index, staging_offset) in (744_u32..888).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded bin directory stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x39, 0x06]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x39, 0x46, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x39, 0x86]);
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
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[9]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-profile-fstat")?;
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
    for (offset, expected) in [(736_u32, 1_u8), (737, 1), (738, 0)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");

    // Derive the complete native regular-file value from the retained
    // description and stage it privately before changing user memory.
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for (offset, value) in [(752_u32, 2_u32), (760, 1), (768, 0x8124), (800, 4096)] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&value.to_le_bytes());
    }
    for (index, staging_offset) in (744_u32..888).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded profile stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x89, 0x46, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x89, 0x86]);
            code.bytes(&user_offset.to_le_bytes());
        }
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    for (index, staging_offset) in (744_u32..888).step_by(8).enumerate() {
        let user_offset = u32::try_from(index * 8).expect("bounded profile stat offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x39, 0x06]);
        } else if user_offset <= 0x7f {
            code.bytes(&[0x48, 0x39, 0x46, user_offset as u8]);
        } else {
            code.bytes(&[0x48, 0x39, 0x86]);
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
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&738_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-profile-read")?;
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
    for (offset, expected) in [(736_u32, 1_u8), (737, 1), (738, 1), (739, 0)] {
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
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&128_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&739_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-root-glob-getdents")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[9]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-getdents");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[10]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-getdents");
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 3)] {
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
        "static-process-shell-root-glob-getdents-profile-ready",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-root-glob-getdents-profile-ready")?;
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25]);
    code.bytes(&328_u32.to_le_bytes());
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-root-glob-getdents-clean");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&32_768_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x05]);
    code.bytes(&32_768_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_heap_validation(
        code,
        "static-process-shell-root-glob-getdents-low-heap",
        "static-process-shell-root-glob-getdents-high-heap",
    )?;
    code.jump("static-process-shell-root-glob-getdents-candidate-ready");
    code.bind("static-process-shell-root-glob-getdents-clean")?;
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&512_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x05]);
    code.bytes(&512_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-root-glob-getdents-candidate-ready")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[2]);
    code.jump_if(0x84, "static-process-shell-root-glob-getdents-first");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[4]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-root-glob-getdents-first")?;
    for offset in (744_u32..816).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for (offset, value) in [
        (744_u32, 3_u64),
        (752, 1),
        (760, 0x0000_0000_2e04_0018),
        (768, 3),
        (776, 2),
        (784, 0x0000_002e_2e04_0018),
        (792, 4),
        (800, 3),
        (808, 0x0000_6e69_6204_0018),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    for (index, staging_offset) in (744_u32..816).step_by(8).enumerate() {
        let user_offset = u8::try_from(index * 8).expect("bounded root dirent offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else {
            code.bytes(&[0x48, 0x89, 0x46, user_offset]);
        }
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    for offset in (744_u32..816).step_by(8) {
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
    code.bytes(&[3]);
    code.bytes(&[0xb8, 72, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-empty-bin-glob-getdents")?;
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
        "static-process-shell-empty-bin-glob-getdents-profile-ready",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-empty-bin-glob-getdents-profile-ready")?;
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25]);
    code.bytes(&328_u32.to_le_bytes());
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-getdents-clean");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&32_768_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x05]);
    code.bytes(&32_768_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_heap_validation(
        code,
        "static-process-shell-empty-bin-glob-getdents-low-heap",
        "static-process-shell-empty-bin-glob-getdents-high-heap",
    )?;
    code.jump("static-process-shell-empty-bin-glob-getdents-candidate-ready");
    code.bind("static-process-shell-empty-bin-glob-getdents-clean")?;
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&512_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x05]);
    code.bytes(&512_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-empty-bin-glob-getdents-candidate-ready")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[9]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-getdents-first");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[10]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[11]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-empty-bin-glob-getdents-first")?;
    for offset in (744_u32..824).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for (offset, value) in [
        (744_u32, 4_u64),
        (752, 1),
        (760, 0x0000_0000_2e04_0018),
        (768, 3),
        (776, 2),
        (784, 0x0000_002e_2e04_0018),
        (792, 5),
        (800, 3),
        (808, 0x6279_7375_6208_0020),
        (816, 0x0000_0000_0000_786f),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
    }
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    for (index, staging_offset) in (744_u32..824).step_by(8).enumerate() {
        let user_offset = u8::try_from(index * 8).expect("bounded bin dirent offset");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
        code.bytes(&staging_offset.to_le_bytes());
        if user_offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else {
            code.bytes(&[0x48, 0x89, 0x46, user_offset]);
        }
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    for offset in (744_u32..824).step_by(8) {
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
    code.bytes(&[10]);
    code.bytes(&[0xb8, 80, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-close")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[27]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-explicit-close");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[11]);
    code.jump_if(0x84, "static-process-shell-empty-bin-glob-close");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 29]);
    code.jump_if(0x84, "static-process-shell-root-glob-close");
    code.jump("static-process-shell-profile-close");

    code.bind("static-process-shell-bin-busybox-explicit-close")?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-explicit-close-profile-ready",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-bin-busybox-explicit-close-profile-ready")?;
    for (offset, expected) in [
        (417_u32, 1_u8),
        (418, 4),
        (737, 1),
        (743, 4),
        (888, 27),
        (889, 4),
        (891, 0),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-explicit-close-three",
    );
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 10]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-explicit-close-last-clean",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-bin-busybox-explicit-close-last-ready");
    code.bind("static-process-shell-bin-busybox-explicit-close-last-clean")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-bin-busybox-explicit-close-last-ready")?;
    for offset in [737_u32, 889, 890, 891] {
        code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[3]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);
    code.bind("static-process-shell-bin-busybox-explicit-close-three")?;
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[2]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-root-glob-close")?;
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 3), (888, 4)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-root-glob-close-profile-ready");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-root-glob-close-profile-ready")?;
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[5]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[4]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 30]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-empty-bin-glob-close")?;
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (743, 4), (888, 11)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[12]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-profile-close")?;
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
        (737, 1),
        (738, 1),
        (739, 1),
        (740, 0),
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
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&740_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    Ok(())
}
