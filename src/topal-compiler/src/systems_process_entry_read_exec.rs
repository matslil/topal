fn emit_static_process_read_exec(
    code: &mut X86FunctionEncoder,
    source_image: bool,
    frame_memory: bool,
) -> Result<(), CompileError> {
    code.bind("static-process-shell-getppid")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 18]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
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
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25]);
    code.bytes(&168_u32.to_le_bytes());
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x65, 0x48, 0x81, 0x3c, 0x25]);
    code.bytes(&176_u32.to_le_bytes());
    code.bytes(&0x0400_0000_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x4c, 0x8b, 0x1c, 0x25]);
    code.bytes(&224_u32.to_le_bytes());
    code.bytes(&[0x49, 0x81, 0xfb]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xfb]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0xffff_fffe_7fff_ffff_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&232_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 19]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-uname")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 19]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x89, 0xf8, 0x49, 0x81, 0xc0]);
    code.bytes(&390_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.rip_lea(&[0x4c, 0x8d, 0x05], "static-process-shell-uts-data");
    code.bytes(&[0x45, 0x31, 0xc9]);
    code.bind("static-process-shell-uname-copy")?;
    code.bytes(&[0x43, 0x8a, 0x04, 0x08, 0x42, 0x88, 0x04, 0x0f]);
    code.bytes(&[0x49, 0xff, 0xc1, 0x49, 0x81, 0xf9]);
    code.bytes(&390_u32.to_le_bytes());
    code.jump_if(0x85, "static-process-shell-uname-copy");
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 20]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-getcwd")?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25, 0x68, 0, 0, 0]);
    code.bytes(&[0x3c, 30]);
    code.jump_if(0x84, "static-process-shell-pwd-getcwd");
    code.bytes(&[0x3c, 20]);
    code.jump_if(0x84, "static-process-shell-initial-getcwd");
    code.bytes(&[0x3c, 26]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0001_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x005f_9460_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&4096_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-shell-post-banner-getcwd-low-heap",
        "static-process-shell-post-banner-getcwd-high-heap",
    )?;
    code.bytes(&[0x66, 0xc7, 0x04, 0x25]);
    code.bytes(&0x005f_9460_u32.to_le_bytes());
    code.bytes(&[0x2f, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&623_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0xb8, 2, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-chdir-root")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 28]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-chdir-root-clean");
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
    code.jump("static-process-shell-chdir-root-profile-ready");
    code.bind("static-process-shell-chdir-root-clean")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-chdir-root-profile-ready")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf8, 0x48, 0x83, 0xc0, 2]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-shell-chdir-root-low-heap",
        "static-process-shell-chdir-root-high-heap",
    )?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x3c, 0x25]);
    code.bytes(&312_u32.to_le_bytes());
    code.bytes(&[0x66, 0x81, 0x3f]);
    code.bytes(&u16::from_le_bytes(*b"/\0").to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 29]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-pwd-getcwd")?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
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
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x005f_9460_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&4096_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-shell-pwd-getcwd-low-heap",
        "static-process-shell-pwd-getcwd-high-heap",
    )?;
    code.bytes(&[0x66, 0xc7, 0x04, 0x25]);
    code.bytes(&0x005f_9460_u32.to_le_bytes());
    code.bytes(&[0x2f, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[6]);
    code.bytes(&[0xb8, 2, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-initial-getcwd")?;
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x005f_9460_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&4096_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-shell-getcwd-low-heap",
        "static-process-shell-getcwd-high-heap",
    )?;
    code.bytes(&[0x66, 0xc7, 0x04, 0x25]);
    code.bytes(&0x005f_9460_u32.to_le_bytes());
    code.bytes(&[0x2f, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 21]);
    code.bytes(&[0xb8, 2, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-tcgets-miss")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[22]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-header-ioctl-miss");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[26]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-header-ioctl-miss");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25, 0x68, 0, 0, 0]);
    code.bytes(&[0x3c, 30]);
    code.jump_if(0x84, "static-process-shell-pwd-tcgets-miss");
    code.bytes(&[0x3c, 21]);
    code.jump_if(0x84, "static-process-shell-initial-tcgets-miss");
    code.bytes(&[0x3c, 26]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-post-banner-tcgets-miss");
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0000_0101_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-stderr-tcgets-miss");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0001_0101_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-stdout-tcgets-miss");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0002_0101_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-standard-tcgets-candidate");
    code.bind("static-process-shell-stderr-tcgets-miss")?;
    code.bytes(&[0x48, 0x83, 0xff, 2]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-standard-tcgets-candidate");
    code.bind("static-process-shell-stdout-tcgets-miss")?;
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-standard-tcgets-candidate")?;
    code.bytes(&[0x81, 0xfe, 0x01, 0x54, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x83, 0xc0, 36]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
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
    code.bytes(&[0x65, 0xfe, 0x04, 0x25]);
    code.bytes(&618_u32.to_le_bytes());
    code.bytes(&[0x48, 0xc7, 0xc0, 0xe7, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-post-banner-tcgets-miss")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0xfe, 0x01, 0x54, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_fba0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&622_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x48, 0xc7, 0xc0, 0xe7, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-initial-tcgets-miss")?;
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0xfe, 0x01, 0x54, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x83, 0xc0, 36]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 22]);
    code.bytes(&[0x48, 0xc7, 0xc0, 0xe7, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-pwd-tcgets-miss")?;
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
    code.bytes(&[7]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x81, 0xfe, 0x01, 0x54, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_f1a0_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[8]);
    code.bytes(&[0x48, 0xc7, 0xc0, 0xe7, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-header-poll")?;
    for (offset, expected) in [
        (416_u32, 2_u8),
        (417, 1),
        (418, 4),
        (737, 1),
        (743, 4),
        (890, 3),
        (891, 0),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[0x3c, 22]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x3c, 25]);
    code.jump_if(0x87, "fatal");
    code.bytes(&[0x2c, 22]);
    code.bytes(&[0x65, 0x38, 0x04, 0x25]);
    code.bytes(&889_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfe, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x89, 0xf8, 0x49, 0x83, 0xc0, 8]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x48, 0x8b, 0x3c, 0x25]);
    code.bytes(&312_u32.to_le_bytes());
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&889_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-header-poll-first");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0001_0001_0000_0003_u64.to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x07]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-bin-busybox-header-poll-ready");
    code.bind("static-process-shell-bin-busybox-header-poll-first")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0001_0000_0003_u64.to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x07]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-bin-busybox-header-poll-ready")?;
    code.bytes(&[0x66, 0xc7, 0x47, 6, 1, 0]);
    code.bytes(&[0x0f, 0xae, 0xf8]);
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0001_0001_0000_0003_u64.to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x07]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&891_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-explicit-close-fcntl")?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-explicit-close-fcntl-profile-ready",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-bin-busybox-explicit-close-fcntl-profile-ready")?;
    for (offset, expected) in [
        (417_u32, 1_u8),
        (418, 4),
        (737, 1),
        (743, 4),
        (888, 27),
        (889, 4),
        (891, 0),
        (892, 0),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x406_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 10]);
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0xb8, 10, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-header-ioctl-miss")?;
    for (offset, expected) in [(416_u32, 2_u8), (417, 1), (418, 4), (737, 1), (743, 4)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    for (index, (state, request)) in [
        (22_u8, 0x5401_u32),
        (22, 0x5401),
        (22, 0x5402),
        (26, 0x5401),
        (26, 0x5402),
    ]
    .into_iter()
    .enumerate()
    {
        let next = match index {
            0 => "static-process-shell-bin-busybox-header-ioctl-0",
            1 => "static-process-shell-bin-busybox-header-ioctl-1",
            2 => "static-process-shell-bin-busybox-header-ioctl-2",
            3 => "static-process-shell-bin-busybox-header-ioctl-3",
            4 => "static-process-shell-bin-busybox-header-ioctl-4",
            _ => unreachable!("bounded header ioctl index"),
        };
        code.bytes(&[
            0x3c,
            u8::try_from(index).expect("bounded header ioctl index"),
        ]);
        code.jump_if(0x85, next);
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&888_u32.to_le_bytes());
        code.bytes(&[state]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x81, 0xfe]);
        code.bytes(&request.to_le_bytes());
        code.jump_if(0x85, "fatal");
        code.jump("static-process-shell-bin-busybox-header-ioctl-ready");
        code.bind(next)?;
    }
    code.jump("fatal");
    code.bind("static-process-shell-bin-busybox-header-ioctl-ready")?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25]);
    code.bytes(&328_u32.to_le_bytes());
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x83, 0xc0, 36]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0xfe, 0x04, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[0x48, 0xc7, 0xc0, 0xe7, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-signal-tail")?;
    code.bytes(&[0x49, 0x83, 0xfa, 8]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25, 0x68, 0, 0, 0]);
    code.bytes(&[0x3c, 22]);
    code.jump_if(0x84, "static-process-shell-query-interrupt");
    code.bytes(&[0x3c, 23]);
    code.jump_if(0x84, "static-process-shell-query-quit");
    code.bytes(&[0x3c, 24]);
    code.jump_if(0x84, "static-process-shell-install-quit-ignore");
    code.bytes(&[0x3c, 25]);
    code.jump_if(0x84, "static-process-shell-query-termination");
    code.jump("fatal");

    code.bind("static-process-shell-query-interrupt")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 2, 0x41, 0xb1, 23]);
    code.jump_if(0x84, "static-process-shell-query-default-common");
    code.jump("fatal");
    code.bind("static-process-shell-query-quit")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 3, 0x41, 0xb1, 24]);
    code.jump_if(0x84, "static-process-shell-query-default-common");
    code.jump("fatal");
    code.bind("static-process-shell-query-termination")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 15, 0x41, 0xb1, 26]);
    code.jump_if(0x84, "static-process-shell-query-default-common");
    code.jump("fatal");

    code.bind("static-process-shell-query-default-common")?;
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x83, 0xc0, 32]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[
        0x31, 0xc0, 0x48, 0x89, 0x02, 0x48, 0x89, 0x42, 8, 0x48, 0x89, 0x42, 16, 0x48, 0x89, 0x42,
        24,
    ]);
    code.bytes(&[0x65, 0x44, 0x88, 0x0c, 0x25, 0x68, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-install-quit-ignore")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x89, 0xf0, 0x49, 0x83, 0xc0, 32]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x49, 0x81, 0xf8]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x48, 0x83, 0x3e, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0x7e, 8]);
    code.bytes(&0x0400_0000_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x8b, 0x46, 16]);
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0xffff_fffe_7fff_ffff_u64.to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x46, 24]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 25]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-sealed-echo-command-read")?;
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x84, "static-process-shell-profile-read");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 31]);
    code.jump_if(0x84, "static-process-shell-final-command-read");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 30]);
    code.jump_if(0x84, "static-process-shell-pwd-command-read");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 29]);
    code.jump_if(0x84, "static-process-shell-root-test-command-read");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 28]);
    code.jump_if(0x84, "static-process-shell-refill-command-read");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 26]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&1023_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-read-clean-fstat");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&736_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&738_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&739_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&740_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&741_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.jump("static-process-shell-read-fstat-ready");
    code.bind("static-process-shell-read-clean-fstat")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&736_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&738_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&739_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&740_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&741_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bind("static-process-shell-read-fstat-ready")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    // Publish the exact first command and seal its unused tail.
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"echo top").to_le_bytes());
    code.bytes(&[0x48, 0xa3]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"al\n\0\0\0\0\0").to_le_bytes());
    code.bytes(&[0x48, 0xa3]);
    code.bytes(&0x005f_8f68_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
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
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[2]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 27]);
    code.bytes(&[0xb8, 11, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-refill-command-read")?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-refill-command-read-variant-ready",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-refill-command-read-variant-ready")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x84, "static-process-shell-refill-command-read-clean-facts");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    for offset in 736_u32..=742 {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.jump("static-process-shell-refill-command-read-facts-ready");
    code.bind("static-process-shell-refill-command-read-clean-facts")?;
    for (offset, expected) in [
        (736_u32, 1_u8),
        (737, 0),
        (738, 1),
        (739, 1),
        (740, 1),
        (741, 1),
        (742, 1),
    ] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bind("static-process-shell-refill-command-read-facts-ready")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&1023_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
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
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0101_0101_0303_0101_u64.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x39, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.jump_if(0x85, "static-process-shell-refill-command-read-publish");
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
    code.bind("static-process-shell-refill-command-read-publish")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"cd /\n\0\0\0").to_le_bytes());
    code.bytes(&[0x48, 0xa3]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&0x005f_8f68_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"cd /\n\0\0\0").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x04, 0x25]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&0x005f_8f68_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[3]);
    code.bytes(&[0xb8, 5, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-root-test-command-read")?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-root-test-command-read-clean");
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
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-root-test-command-read-profile-ready");
    code.bind("static-process-shell-root-test-command-read-clean")?;
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
    code.bytes(&[2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"cd /\n\0\0\0").to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x04, 0x25]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&0x005f_8f68_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-root-test-command-read-profile-ready")?;
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 8]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&1023_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[3]);
    code.jump_if(0x85, "fatal");
    for (index, chunk) in [
        *b"echo /*;",
        *b" test -d",
        *b" /bin &&",
        *b" echo /b",
        *b"in/*; te",
        *b"st -f /b",
        *b"in/busyb",
        *b"ox && ec",
        *b"ho busyb",
        *b"ox-file;",
        *b" test -x",
        *b" /bin/bu",
        *b"sybox &&",
        *b" echo bu",
        *b"sybox-ex",
        *b"ecutable",
        *b"; test -",
        *b"s /bin/b",
        *b"usybox &",
        *b"& echo b",
        *b"usybox-n",
        *b"onempty;",
        *b" exec 3<",
        *b"/bin/bus",
        *b"ybox; ec",
        *b"ho busyb",
        *b"ox-open;",
        *b" IFS= re",
        *b"ad -r -n",
        *b" 4 -u 3 ",
        *b"_ && ech",
        *b"o busybo",
        *b"x-read; ",
        *b"exec 3<&",
        *b"-; echo ",
        *b"busybox-",
        *b"close\n\0\0",
    ]
    .into_iter()
    .enumerate()
    {
        let offset = 0x005f_8f60_u32
            + u32::try_from(index * 8).expect("bounded root condition command offset");
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&u64::from_le_bytes(chunk).to_le_bytes());
        code.bytes(&[0x48, 0xa3]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    for (index, chunk) in [
        *b"echo /*;",
        *b" test -d",
        *b" /bin &&",
        *b" echo /b",
        *b"in/*; te",
        *b"st -f /b",
        *b"in/busyb",
        *b"ox && ec",
        *b"ho busyb",
        *b"ox-file;",
        *b" test -x",
        *b" /bin/bu",
        *b"sybox &&",
        *b" echo bu",
        *b"sybox-ex",
        *b"ecutable",
        *b"; test -",
        *b"s /bin/b",
        *b"usybox &",
        *b"& echo b",
        *b"usybox-n",
        *b"onempty;",
        *b" exec 3<",
        *b"/bin/bus",
        *b"ybox; ec",
        *b"ho busyb",
        *b"ox-open;",
        *b" IFS= re",
        *b"ad -r -n",
        *b" 4 -u 3 ",
        *b"_ && ech",
        *b"o busybo",
        *b"x-read; ",
        *b"exec 3<&",
        *b"-; echo ",
        *b"busybox-",
        *b"close\n\0\0",
    ]
    .into_iter()
    .enumerate()
    {
        let offset = 0x005f_8f60_u32
            + u32::try_from(index * 8).expect("bounded root condition command offset");
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&u64::from_le_bytes(chunk).to_le_bytes());
        code.bytes(&[0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[3]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[4]);
    code.bytes(&[0xb8, 0x26, 1, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-bin-busybox-header-read")?;
    code.bytes(&[0x48, 0x83, 0xff, 3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 1]);
    code.jump_if(0x85, "fatal");
    for (offset, expected) in [(417_u32, 1_u8), (418, 4), (737, 1), (743, 4)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[0x3c, 22]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x3c, 25]);
    code.jump_if(0x87, "fatal");
    code.bytes(&[0x2c, 22]);
    code.bytes(&[0x65, 0x38, 0x04, 0x25]);
    code.bytes(&889_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-header-read-clean");
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[3]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&891_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_7000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 1]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0060_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_low_heap_validation(
        code,
        "static-process-shell-bin-busybox-header-read-low-heap",
    )?;
    code.jump("static-process-shell-bin-busybox-header-read-profile-ready");
    code.bind("static-process-shell-bin-busybox-header-read-clean")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&891_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x007f_e000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc0, 1]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0080_0000_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    code.bind("static-process-shell-bin-busybox-header-read-profile-ready")?;
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&889_u32.to_le_bytes());
    let header_read_labels = [
        "static-process-shell-bin-busybox-header-read-byte-0",
        "static-process-shell-bin-busybox-header-read-byte-1",
        "static-process-shell-bin-busybox-header-read-byte-2",
        "static-process-shell-bin-busybox-header-read-byte-3",
    ];
    for (index, label) in header_read_labels.into_iter().enumerate() {
        code.bytes(&[
            0x3c,
            u8::try_from(index).expect("bounded header byte index"),
        ]);
        code.jump_if(0x84, label);
    }
    code.jump("fatal");
    for (label, byte) in header_read_labels
        .into_iter()
        .zip([0x7f_u8, b'E', b'L', b'F'])
    {
        code.bind(label)?;
        code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
        code.bytes(&744_u32.to_le_bytes());
        code.bytes(&[byte]);
        code.jump("static-process-shell-bin-busybox-header-read-byte-ready");
    }
    code.bind("static-process-shell-bin-busybox-header-read-byte-ready")?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&320_u32.to_le_bytes());
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0x88, 0x06, 0x0f, 0xae, 0xf8, 0x38, 0x06]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&744_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&891_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&891_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xfe, 0x04, 0x25]);
    code.bytes(&889_u32.to_le_bytes());
    code.bytes(&[0x65, 0xfe, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-pwd-command-read")?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-pwd-command-read-clean");
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
    code.bytes(&[4]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-pwd-command-read-profile-ready");
    code.bind("static-process-shell-pwd-command-read-clean")?;
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
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    for (index, chunk) in [
        *b"echo /*;",
        *b" test -d",
        *b" /bin &&",
        *b" echo /b",
        *b"in/*; te",
        *b"st -f /b",
        *b"in/busyb",
        *b"ox && ec",
        *b"ho busyb",
        *b"ox-file;",
        *b" test -x",
        *b" /bin/bu",
        *b"sybox &&",
        *b" echo bu",
        *b"sybox-ex",
        *b"ecutable",
        *b"; test -",
        *b"s /bin/b",
        *b"usybox &",
        *b"& echo b",
        *b"usybox-n",
        *b"onempty;",
        *b" exec 3<",
        *b"/bin/bus",
        *b"ybox; ec",
        *b"ho busyb",
        *b"ox-open;",
        *b" IFS= re",
        *b"ad -r -n",
        *b" 4 -u 3 ",
        *b"_ && ech",
        *b"o busybo",
        *b"x-read; ",
        *b"exec 3<&",
        *b"-; echo ",
        *b"busybox-",
        *b"close\n\0\0",
    ]
    .into_iter()
    .enumerate()
    {
        let offset = 0x005f_8f60_u32
            + u32::try_from(index * 8).expect("bounded consumed root condition offset");
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&u64::from_le_bytes(chunk).to_le_bytes());
        code.bytes(&[0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bind("static-process-shell-pwd-command-read-profile-ready")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[27]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[4]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&24_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xfa]);
    code.bytes(&1023_u32.to_le_bytes());
    code.jump_if(0x87, "fatal");
    emit_static_process_active_stack_validation(code);
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
    for offset in (0x005f_8f60_u32..0x005f_8f88).step_by(8) {
        code.bytes(&[0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    for (index, chunk) in [*b"exec 3</", *b"bin/busy", *b"box;pwd\n"]
        .into_iter()
        .enumerate()
    {
        let offset =
            0x005f_8f60_u32 + u32::try_from(index * 8).expect("bounded reopen command offset");
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&u64::from_le_bytes(chunk).to_le_bytes());
        code.bytes(&[0x48, 0xa3]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    for (index, chunk) in [*b"exec 3</", *b"bin/busy", *b"box;pwd\n"]
        .into_iter()
        .enumerate()
    {
        let offset =
            0x005f_8f60_u32 + u32::try_from(index * 8).expect("bounded reopen command offset");
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&u64::from_le_bytes(chunk).to_le_bytes());
        code.bytes(&[0x48, 0x39, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&0x005f_8f78_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[5]);
    code.bytes(&[0xb8, 24, 0, 0, 0]);
    emit_static_process_sysret(code);

    code.bind("static-process-shell-final-command-read")?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(
        0x84,
        "static-process-shell-final-command-read-variant-ready",
    );
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-final-command-read-variant-ready")?;
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x005f_8f60_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    emit_static_process_active_stack_validation(code);
    emit_static_process_command_buffer_validation(code);
    emit_static_process_child_disposition_validation(code);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[0x3c, 10]);
    code.jump_if(0x84, "static-process-shell-final-command-read-facts-ready");
    code.bytes(&[0x3c, 9]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-final-command-read-facts-ready")?;
    process_runtime::emit_command_input(code)?;
    code.bytes(&[0x50]);
    code.call("fd-table-initialize");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[6]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 32]);
    code.bytes(&[0x58]);
    emit_static_process_sysret(code);

    code.bind("static-process-busybox-true-execve")?;
    code.bytes(&[0x57, 0x56, 0x52, 0x51]);
    emit_static_process_active_stack_validation(code);
    emit_static_process_heap_validation(
        code,
        "static-process-execve-low-heap",
        "static-process-execve-high-heap",
    )?;
    code.bytes(&[0x59, 0x5a, 0x5e, 0x5f]);
    process_runtime::emit_exec_generation_validation(code)?;
    for (offset, expected) in [(737_u32, 0_u8), (892, 5)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    process_runtime::emit_exec_arguments(code)?;
    if source_image {
        process_runtime::header::emit_invoke(code)?;
        code.bytes(&[0x48, 0x85, 0xc0]);
        code.jump_if(0x84, "static-process-source-header-refused");
        code.jump("static-process-source-header-accepted");
        code.bind("static-process-source-header-refused")?;
        process_runtime::header::emit_refusal(code);
        code.bind("static-process-source-header-accepted")?;
    }
    code.call("fd-exec-filter");

    // Restore all writable image pages before copying the sealed snapshot.
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25]);
    code.bytes(&112_u32.to_le_bytes());
    for offset in (0x0f28_u32..=0x0fb0).step_by(8) {
        code.bytes(&[0x48, 0x8b, 0x82]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff, 0x49, 0xb9]);
        code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
        code.bytes(&[0x4c, 0x09, 0xc8, 0x48, 0x89, 0x82]);
        code.bytes(&offset.to_le_bytes());
    }
    code.bytes(&[0x0f, 0xae, 0xf0, 0x48, 0xb8]);
    code.bytes(&0x005e_5000_u64.to_le_bytes());
    code.bytes(&[0xb9, 18, 0, 0, 0]);
    code.bind("static-process-execve-invalidate-image")?;
    code.bytes(&[0x0f, 0x01, 0x38, 0x48, 0x05, 0, 0x10, 0, 0, 0xff, 0xc9]);
    code.jump_if(0x85, "static-process-execve-invalidate-image");
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&904_u32.to_le_bytes());
    code.bytes(&[0x48, 0xbf]);
    code.bytes(&0x005e_5660_u64.to_le_bytes());
    code.bytes(&[0xb9]);
    code.bytes(&0x9728_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0xa4]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&904_u32.to_le_bytes());
    code.bytes(&[0x48, 0xbf]);
    code.bytes(&0x005e_5660_u64.to_le_bytes());
    code.bytes(&[0xb9]);
    code.bytes(&0x9728_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0xa6]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0xbf]);
    code.bytes(&0x005e_ed88_u64.to_le_bytes());
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&0x8278_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0xaa]);
    code.bytes(&[0x48, 0xbf]);
    code.bytes(&0x005e_ed88_u64.to_le_bytes());
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&0x8278_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0xae]);
    code.jump_if(0x85, "fatal");

    // Erase and revoke every anonymous heap mapping from the old image.
    if frame_memory {
        code.bytes(&[0x31, 0xf6]);
        frame_memory::call(code, 7, "frame-mm-exec-return")?;
        code.bytes(&[0x48, 0x83, 0xf8, 1]);
        code.jump_if(0x85, "fatal");
    } else {
        code.bytes(&[0x48, 0xbf]);
        code.bytes(&0x005f_7000_u64.to_le_bytes());
        code.bytes(&[0x31, 0xc0, 0xb9]);
        code.bytes(&(34_u32 * 4096 / 8).to_le_bytes());
        code.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
        code.bytes(&[0x48, 0xbf]);
        code.bytes(&0x005f_7000_u64.to_le_bytes());
        code.bytes(&[0x31, 0xc0, 0xb9]);
        code.bytes(&(34_u32 * 4096 / 8).to_le_bytes());
        code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25]);
        code.bytes(&112_u32.to_le_bytes());
        for offset in (0x0fb8_u32..=0x0ff8).step_by(8) {
            code.bytes(&[0x48, 0xc7, 0x82]);
            code.bytes(&offset.to_le_bytes());
            code.bytes(&[0, 0, 0, 0]);
        }
        code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
        for offset in (0x4000_u32..0x40c8).step_by(8) {
            code.bytes(&[0x48, 0xc7, 0x82]);
            code.bytes(&offset.to_le_bytes());
            code.bytes(&[0, 0, 0, 0]);
        }
        code.bytes(&[0x0f, 0xae, 0xf0, 0x48, 0xb8]);
        code.bytes(&0x005f_7000_u64.to_le_bytes());
        code.bytes(&[0xb9, 34, 0, 0, 0]);
        code.bind("static-process-execve-invalidate-heap")?;
        code.bytes(&[0x0f, 0x01, 0x38, 0x48, 0x05, 0, 0x10, 0, 0, 0xff, 0xc9]);
        code.jump_if(0x85, "static-process-execve-invalidate-heap");
    }

    // Build a fresh Linux x86-64 initial stack for the replacement image.
    code.bytes(&[0x48, 0xbf]);
    code.bytes(&0x007f_e000_u64.to_le_bytes());
    code.bytes(&[0x31, 0xc0, 0xb9, 0x00, 0x04, 0, 0, 0xfc, 0xf3, 0x48, 0xab]);
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&u64::from_le_bytes(*b"x86_64\0\0").to_le_bytes());
    code.bytes(&[0x48, 0xa3]);
    code.bytes(&process_runtime::PLATFORM_BASE.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x8b, 0x3c, 0x25]);
    code.bytes(&256_u32.to_le_bytes());
    code.bytes(&[0x48, 0x8d, 0xb7]);
    code.bytes(&224_u32.to_le_bytes());
    code.bytes(&[0x48, 0x81, 0xc7]);
    code.bytes(&192_u32.to_le_bytes());
    code.call("static-process-getrandom-extract");
    code.bytes(&[0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x34, 0x25]);
    code.bytes(&256_u32.to_le_bytes());
    for (source_offset, destination) in [
        (224_u32, process_runtime::RANDOM_BASE),
        (232, process_runtime::RANDOM_BASE + 8),
    ] {
        code.bytes(&[0x48, 0x8b, 0x86]);
        code.bytes(&source_offset.to_le_bytes());
        code.bytes(&[0x48, 0xa3]);
        code.bytes(&u64::from(destination).to_le_bytes());
    }
    for offset in [224_u32, 232, 240, 248] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    code.bytes(&[0x65, 0x48, 0xff, 0x04, 0x25]);
    code.bytes(&264_u32.to_le_bytes());
    process_runtime::emit_exec_stack(code)?;

    // Reset process-local state while retaining the process identity, cwd,
    // standard descriptors, ignored disposition, and live descriptor three.
    for offset in [
        process_runtime::policy::POLICY_CONTEXT,
        process_runtime::numeric::CODE_CONTEXT,
        process_runtime::numeric::ARENA_CONTEXT,
    ] {
        code.bytes(&[0x49, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for offset in (296_u32..=408).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    for offset in 418_u32..424 {
        code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
    }
    for offset in (472_u32..=728).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    for offset in (744_u32..=887).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    for offset in [104_u32, 128, 136, 144, 152, 160, 168, 176] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&896_u32.to_le_bytes());
    code.bytes(&[1]);
    emit_wrmsr_immediate(code, 0xc000_0100, 0);
    // The unused new-image stack prefix must still be zero: in particular no
    // kernel return address or random-generator scratch may have landed there.
    code.bytes(&[0x48, 0xbf]);
    code.bytes(&0x007f_e000_u64.to_le_bytes());
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&((process_runtime::ENTRY as u32 - 0x007f_e000) / 8).to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");
    process_runtime::emit_exec_commit_witness(code);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25]);
    code.bytes(&256_u32.to_le_bytes());
    emit_static_process_clean_simd_state(code);
    emit_static_process_clear_syscall_registers(code);
    code.bytes(&[0x6a, 0x23, 0x48, 0xb8]);
    code.bytes(&process_runtime::ENTRY.to_le_bytes());
    code.bytes(&[0x50, 0x6a, 0x02, 0x6a, 0x2b, 0x48, 0xb8]);
    code.bytes(&0x004a_6580_u64.to_le_bytes());
    code.bytes(&[0x50]);
    code.bytes(&[
        0x31, 0xc0, 0x31, 0xdb, 0x31, 0xc9, 0x31, 0xd2, 0x31, 0xf6, 0x31, 0xff, 0x31, 0xed, 0x45,
        0x31, 0xc0, 0x45, 0x31, 0xc9, 0x45, 0x31, 0xd2, 0x45, 0x31, 0xdb, 0x45, 0x31, 0xe4, 0x45,
        0x31, 0xed, 0x45, 0x31, 0xf6, 0x45, 0x31, 0xff, 0x0f, 0x01, 0xf8, 0x48, 0xcf,
    ]);

    Ok(())
}
