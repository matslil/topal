fn emit_static_process_start(
    code: &mut X86FunctionEncoder,
    source_bound: bool,
    file_open_plan: bool,
    source_numeric: bool,
    source_image: bool,
    source_dispatch: bool,
    dispatch_policy: crate::systems_numeric_library::NumericPolicy,
    source_identity: bool,
    retained_identity: bool,
    source_inode: bool,
    frame_memory: bool,
    user_extent_probe: bool,
) -> Result<(), CompileError> {
    const OBJECT: u32 = 0x0e70;
    const TAG: u32 = 0x0fc8;
    const VALUE: u32 = 0x0fd0;
    const TERMINATOR: u32 = 0x0fd8;
    const CONTEXT_QWORDS: u32 = 21;

    // Preserve static root, metadata, payload, bootstrap root, and stack backing.
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    code.bytes(&[0x4c, 0x8b, 0x54, 0x24, 0x30]); // private context argument
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x41, 0xf7, 0xc2, 0x07, 0, 0, 0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4c, 0x89, 0xd7, 0x31, 0xc0, 0xb9]);
    code.bytes(&CONTEXT_QWORDS.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x80, 0xba]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fail");
    for offset in [224_u32, 232, 240, 248, 264, 272] {
        code.bytes(&[0x49, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fail");
    }
    for offset in (280_u32..=616).step_by(8) {
        code.bytes(&[0x49, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fail");
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
        code.bytes(&[0x49, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fail");
    }
    process_files::emit_entry_validation(code)?;
    process_runtime::time::emit_entry_validation(code)?;
    if source_bound {
        process_runtime::policy::emit_entry_validation(code)?;
    }
    if source_numeric || source_image || source_dispatch || source_identity {
        process_runtime::numeric::emit_entry_validation(
            code,
            source_numeric,
            source_image,
            source_dispatch,
            source_identity,
            retained_identity,
        )?;
    }
    code.bytes(&[0x49, 0x8b, 0x82]);
    code.bytes(&256_u32.to_le_bytes());
    code.bytes(&[0x4c, 0x39, 0xd0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, // payload
        0x48, 0x8b, 0x74, 0x24, 0x08, // leaf metadata
        0x48, 0x8b, 0x0c, 0x24, // static root
        0x4c, 0x8b, 0x44, 0x24, 0x18, // bootstrap root
        0x4c, 0x8b, 0x4c, 0x24, 0x20, // stack backing
    ]);

    emit_static_image_address_space_resource_validation(code, true)?;
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    emit_static_image_published_stack_leaf_verification(
        code,
        [
            "process-entry-h",
            "process-entry-x",
            "process-entry-r",
            "process-entry-w",
        ],
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        code,
        true,
        "process-entry-pd",
        "process-entry-pd-expected",
    )?;
    emit_static_image_stack_leaf_root_verification(code)?;
    emit_static_image_stack_leaf_entry_verification(code)?;
    emit_static_image_resident_program_header_verification(code)?;

    // The exact bounded heap reserve is 34 provider-owned zero data pages;
    // the fifth hierarchy frame is the independent high leaf table.
    code.bytes(&[0x4c, 0x8b, 0x54, 0x24, 0x30]);
    if !frame_memory {
        code.bytes(&[0x48, 0x8b, 0x44, 0x24, 0x38]);
        code.bytes(&[0x48, 0x85, 0xc0]);
        code.jump_if(0x84, "fail");
        code.bytes(&[0xa9, 0xff, 0x0f, 0, 0]);
        code.jump_if(0x85, "fail");
        code.bytes(&[
            0x48, 0x89, 0xc7, 0x31, 0xc0, 0xb9, 0x00, 0x44, 0, 0, 0xfc, 0xf3, 0x48, 0xaf,
        ]);
        code.jump_if(0x85, "fail");
    }
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x48, 0x83, 0xbe, 0xb8, 0x0f, 0, 0, 0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0x72, 0x70]);
    if frame_memory {
        code.bytes(&[0x31, 0xc0]);
    } else {
        code.bytes(&[0x48, 0x8b, 0x44, 0x24, 0x38]);
    }
    code.bytes(&[0x49, 0x89, 0x42, 0x78]);
    code.bytes(&[0x48, 0x8b, 0x44, 0x24, 0x40]);
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0xf7, 0xc0, 0x0f, 0, 0, 0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0x82]);
    code.bytes(&904_u32.to_le_bytes());
    code.bytes(&[0x4c, 0x8b, 0x4c, 0x24, 0x20]);
    code.bytes(&[0x4d, 0x89, 0x8a]);
    code.bytes(&912_u32.to_le_bytes());

    // Retain one kernel-private pristine copy of the writable ELF load. The
    // source was already populated from and checked against the digest-bound
    // boot resource; the copy remains outside the user address space and is
    // consumed only by a successful same-image execve replacement.
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x10, 0x48, 0x81, 0xc6]);
    code.bytes(&0x001e_5660_u32.to_le_bytes());
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x40]);
    code.bytes(&[0xb9]);
    code.bytes(&0x9728_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0xa4, 0x0f, 0xae, 0xf0]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x10, 0x48, 0x81, 0xc6]);
    code.bytes(&0x001e_5660_u32.to_le_bytes());
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x40]);
    code.bytes(&[0xb9]);
    code.bytes(&0x9728_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0xa6]);
    code.jump_if(0x85, "fail");

    // Preserve the opaque random object while reusing the exact deterministic
    // initial-stack verifier, then restore and verify its published ABI facts.
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    code.bytes(&[0x4c, 0x8b, 0x54, 0x24, 0x30]);
    code.bytes(&[0x49, 0x8b, 0x83]);
    code.bytes(&OBJECT.to_le_bytes());
    code.bytes(&[0x49, 0x89, 0x42, 0x20]);
    code.bytes(&[0x49, 0x8b, 0x83]);
    code.bytes(&(OBJECT + 8).to_le_bytes());
    code.bytes(&[0x49, 0x89, 0x42, 0x28]);
    for (offset, value) in [(TAG, 25_u64), (VALUE, 0x007f_fe70)] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x49, 0x39, 0x83]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fail");
    }
    code.bytes(&[0x49, 0x83, 0xbb]);
    code.bytes(&TERMINATOR.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fail");
    for offset in [OBJECT, OBJECT + 8, TAG, VALUE] {
        code.bytes(&[0x49, 0xc7, 0x83]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    emit_static_image_load_metadata_stack(code, true)?;
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    code.bytes(&[0x4c, 0x8b, 0x54, 0x24, 0x30]);
    for (context_offset, target_offset) in [(0x20_u8, OBJECT), (0x28, OBJECT + 8)] {
        code.bytes(&[0x49, 0x8b, 0x42, context_offset]);
        code.bytes(&[0x49, 0x89, 0x83]);
        code.bytes(&target_offset.to_le_bytes());
        code.bytes(&[0x49, 0xc7, 0x42, context_offset, 0, 0, 0, 0]);
    }
    for (offset, value) in [(TAG, 25_u64), (VALUE, 0x007f_fe70)] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x49, 0x89, 0x83]);
        code.bytes(&offset.to_le_bytes());
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);

    // Save all privileged state changed by this bounded entry.
    for (msr, context_offset) in [
        (0xc000_0102_u32, 56_u8),
        (0xc000_0100, 64),
        (0xc000_0080, 72),
        (0xc000_0081, 80),
        (0xc000_0082, 88),
        (0xc000_0084, 96),
    ] {
        code.bytes(&[0xb9]);
        code.bytes(&msr.to_le_bytes());
        code.bytes(&[0x0f, 0x32, 0x48, 0xc1, 0xe2, 0x20, 0x48, 0x09, 0xd0]);
        code.bytes(&[0x49, 0x89, 0x42, context_offset]);
    }
    code.bytes(&[0x48, 0x8b, 0x44, 0x24, 0x18]);
    code.bytes(&[0x49, 0x89, 0x42, 0x10]); // bootstrap CR3
    code.bytes(&[0x48, 0x8b, 0x04, 0x24]);
    code.bytes(&[0x49, 0x89, 0x42, 0x18]); // static CR3
    code.bytes(&[0x49, 0x89, 0x62, 0x08]); // continuation RSP

    code.bytes(&[0xb9]);
    code.bytes(&0xc000_0080_u32.to_le_bytes());
    code.bytes(&[0x0f, 0x32, 0x83, 0xc8, 0x01, 0x0f, 0x30]);
    emit_wrmsr_immediate(code, 0xc000_0081, 0x001b_0010_0000_0000);
    code.rip_lea(&[0x48, 0x8d, 0x05], "static-process-syscall");
    emit_wrmsr_register(code, 0xc000_0082, &[]);
    emit_wrmsr_immediate(code, 0xc000_0084, if frame_memory { 0x600 } else { 0x200 });
    emit_wrmsr_register(code, 0xc000_0102, &[0x4c, 0x89, 0xd0]);
    emit_wrmsr_immediate(code, 0xc000_0100, 0);

    // Retain the already verified resident-header variant so stack-relative
    // ABI observations can remain exact for both the clean fixture and the
    // pinned external executable without trusting a user-controlled pointer.
    code.bytes(&[0x41, 0xc6, 0x82]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x48, 0x8b, 0x44, 0x24, 0x10]);
    code.bytes(&[0x48, 0x83, 0x78, 0x28, 0]);
    code.jump_if(0x84, "static-process-header-variant-ready");
    code.bytes(&[0x41, 0xc6, 0x82]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[2]);
    code.bind("static-process-header-variant-ready")?;
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x20]);
    code.bytes(&[0x49, 0x8b, 0xba]);
    code.bytes(&904_u32.to_le_bytes());
    code.bytes(&[
        0x48, 0x81, 0xc7, 0, 0xa0, 0, 0, 0xb9, 0, 0x10, 0, 0, 0xfc, 0xf3, 0xa4,
    ]);
    // One fresh affine sealed-input owner advances across the exact command corpus.
    code.bytes(&[0x41, 0xc6, 0x82]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[1]);
    emit_static_process_save_caller_registers(code);

    // The bounded x86-64 process owns a fresh SIMD context. Preserve the
    // provider's CR4, enable only the architected OS SIMD controls, and
    // initialize x87, MXCSR, and every XMM register before entry.
    code.bytes(&[0x0f, 0x20, 0xc0, 0xa9, 0x0c, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fatal"); // CR0.EM and CR0.TS must both be clear
    if user_extent_probe || code.owned_user_extents {
        // Initialize the new guest profile before user entry, preserving every
        // other CR0 bit. A user transfer never disables write protection.
        code.bind("user-extent-wp-before")?;
        code.bytes(&[0x48, 0x0d, 0, 0, 1, 0, 0x0f, 0x22, 0xc0]);
        code.bind("user-extent-wp-ready")?;
    }
    code.bytes(&[0x0f, 0x20, 0xe0, 0x49, 0x89, 0x82, 0x18, 0x01, 0, 0]);
    code.bytes(&[0x48, 0x0d, 0x00, 0x06, 0, 0, 0x0f, 0x22, 0xe0]);
    code.bytes(&[0x41, 0xc7, 0x82, 0x20, 0x01, 0, 0, 0x80, 0x1f, 0, 0]);
    emit_static_process_clean_simd_state(code);
    code.bytes(&[0x41, 0xc6, 0x02, 0x01]);

    code.bytes(&[0x48, 0x8b, 0x04, 0x24, 0x0f, 0x22, 0xd8]);
    code.bytes(&[0x6a, 0x23]);
    code.bytes(&[0x41, 0x80, 0xba]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[2]);
    code.jump_if(0x84, "static-process-external-entry-stack");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x007f_ffa0_u64.to_le_bytes());
    code.jump("static-process-entry-stack-ready");
    code.bind("static-process-external-entry-stack")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x007f_fe90_u64.to_le_bytes());
    code.bind("static-process-entry-stack-ready")?;
    code.bytes(&[0x50, 0x6a, 0x02, 0x6a, 0x2b, 0x48, 0xb8]);
    code.bytes(&0x004a_6580_u64.to_le_bytes());
    code.bytes(&[0x50, 0x48, 0xcf]);

    code.bind("static-process-syscall")?;
    code.bytes(&[0x0f, 0x01, 0xf8]);
    emit_static_process_save_syscall_registers(code);
    // SYSCALL does not switch RSP. Never use the untrusted user stack for
    // provider calls, staging, random-generator work, or the replacement frame.
    // The saved user RSP is restored only by the checked SYSRET path.
    code.bytes(&[0x65, 0x48, 0x8b, 0x24, 0x25, 8, 0, 0, 0]);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0, 0, 0, 0, 1]);
    code.jump_if(0x85, "fatal");
    // Linux x86-64 selects a syscall with the low native 32-bit number.
    code.bytes(&[0x89, 0xc0]);
    if frame_memory {
        frame_memory::capture(code);
    }
    if source_dispatch {
        process_runtime::dispatch::emit_invoke(code, dispatch_policy)?;
    }
    process_files::emit_dispatch(code)?;
    if source_inode {
        process_runtime::inode_services::emit_dispatch(code)?;
    }
    process_runtime::signals::emit_dispatch(code)?;
    process_runtime::namespace::emit_dispatch(
        code,
        file_open_plan
            || matches!(
                dispatch_policy,
                crate::systems_numeric_library::NumericPolicy::LinuxDispatchV3
                    | crate::systems_numeric_library::NumericPolicy::LinuxDispatchV4
                    | crate::systems_numeric_library::NumericPolicy::LinuxDispatchV5
                    | crate::systems_numeric_library::NumericPolicy::LinuxDispatchV6
            ),
        dispatch_policy == crate::systems_numeric_library::NumericPolicy::LinuxDispatchV6,
    )?;
    process_runtime::realtime::emit_dispatch(code)?;
    process_runtime::credentials::emit_dispatch(code)?;
    process_runtime::observations::emit_dispatch(code)?;
    process_runtime::time::emit_dispatch(code)?;
    process_runtime::children::emit_dispatch(code)?;
    process_runtime::pipes::emit_dispatch(code)?;
    process_runtime::volatile_files::emit_dispatch(code)?;
    code.bytes(&[0x85, 0xc0]);
    code.jump_if(0x85, "static-process-non-read-syscall");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[22]);
    code.jump_if(0x82, "static-process-shell-sealed-echo-command-read");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[25]);
    code.jump_if(0x87, "static-process-shell-sealed-echo-command-read");
    code.jump("static-process-shell-bin-busybox-header-read");
    code.bind("static-process-non-read-syscall")?;
    code.bytes(&[0x83, 0xf8, 60]);
    code.jump_if(0x85, "runtime-non-exit-syscall");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&896_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-execve-true-exit");
    code.bind("runtime-non-exit-syscall")?;
    code.bytes(&[0x83, 0xf8, 7]);
    code.jump_if(0x84, "static-process-shell-bin-busybox-header-poll");
    code.bytes(&[0x83, 0xf8, 72]);
    code.jump_if(
        0x84,
        "static-process-shell-bin-busybox-explicit-close-fcntl",
    );
    code.bytes(&[0x83, 0xf8, 0x03]);
    code.jump_if(0x84, "static-process-shell-close");
    code.bytes(&[0x83, 0xf8, 0x05]);
    code.jump_if(0x84, "static-process-shell-console-fstat");
    code.bytes(&[0x3d, 0x01, 0x01, 0, 0]);
    code.jump_if(0x84, "static-process-shell-dev-tty-open-miss");
    code.bytes(&[0x3d, 0xd9, 0, 0, 0]);
    code.jump_if(0x84, "static-process-shell-root-glob-getdents");
    code.bytes(&[0x83, 0xf8, 0x3f]);
    code.jump_if(0x84, "static-process-shell-uname");
    code.bytes(&[0x83, 0xf8, 0x3b]);
    code.jump_if(0x84, "static-process-busybox-true-execve");
    code.bytes(&[0x83, 0xf8, 0x4f]);
    code.jump_if(0x84, "static-process-shell-getcwd");
    code.bytes(&[0x83, 0xf8, 0x50]);
    code.jump_if(0x84, "static-process-shell-chdir-root");
    code.bytes(&[0x83, 0xf8, 0x10]);
    code.jump_if(0x84, "static-process-shell-tcgets-miss");
    code.bytes(&[0x3d, 0x0c, 0, 0, 0]);
    code.jump_if(0x84, "static-process-brk");
    code.bytes(&[0x3d, 0x9e, 0, 0, 0]);
    code.jump_if(0x84, "static-process-set-fs");
    code.bytes(&[0x3d, 0xda, 0, 0, 0]);
    code.jump_if(0x84, "static-process-set-tid-address");
    code.bytes(&[0x3d, 0x11, 0x01, 0, 0]);
    code.jump_if(0x84, "static-process-set-robust-list");
    code.bytes(&[0x3d, 0x4e, 0x01, 0, 0]);
    code.jump_if(0x84, "static-process-rseq-fallback");
    code.bytes(&[0x3d, 0x2e, 0x01, 0, 0]);
    code.jump_if(0x84, "static-process-stack-limit");
    code.bytes(&[0x3d, 0x0b, 0x01, 0, 0]);
    code.jump_if(0x84, "static-process-readlinkat-fallback");
    code.bytes(&[0x3d, 0x3e, 0x01, 0, 0]);
    code.jump_if(0x84, "static-process-getrandom");
    code.bytes(&[0x3d, 0x0a, 0, 0, 0]);
    code.jump_if(0x84, "static-process-relro-mprotect");
    code.bytes(&[0x3d, 0x9d, 0, 0, 0]);
    code.jump_if(0x84, "static-process-initial-process-name");
    code.bytes(&[0x3d, 0x66, 0, 0, 0]);
    code.jump_if(0x84, "static-process-initial-getuid");
    code.bytes(&[0x3d, 0x6b, 0, 0, 0]);
    code.jump_if(0x84, "static-process-shell-geteuid");
    code.bytes(&[0x3d, 0x06, 0x01, 0, 0]);
    code.jump_if(0x84, "static-process-newfstatat");
    for number in [104_u8, 106, 105] {
        code.bytes(&[0x3d, number, 0, 0, 0]);
        code.jump_if(0x84, "static-process-initial-credential-tail");
    }
    code.bytes(&[0x3d, 0x01, 0, 0, 0]);
    code.jump_if(0x84, "static-process-applet-newline-write");
    code.bytes(&[0x3d, 0x27, 0, 0, 0]);
    code.jump_if(0x84, "static-process-shell-getpid");
    code.bytes(&[0x3d, 0x0d, 0, 0, 0]);
    code.jump_if(0x84, "static-process-shell-sigchld");
    code.bytes(&[0x3d, 0x6e, 0, 0, 0]);
    code.jump_if(0x84, "static-process-shell-getppid");
    code.bytes(&[0x3d, 0xe7, 0, 0, 0]);
    code.jump_if(0x84, "static-process-exit-group-selected");
    // TOPAL-SYSTEMS-STATIC-PROCESS-ENOSYS-001: no implemented arm selected.
    // Do not inspect unused arguments, advance startup or consume the owner.
    code.bytes(&[0x48, 0xc7, 0xc0, 0xda, 0xff, 0xff, 0xff]);
    emit_static_process_sysret(code);
    code.bind("static-process-exit-group-selected")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&896_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x84, "static-process-execve-true-exit");
    if code.owned_user_extents {
        code.bytes(&[0x48, 0x85, 0xff]);
        code.jump_if(0x85, "fatal");
        process_runtime::children::emit_require_initial_sole_process(code);
        process_runtime::file_descriptions::emit_require_active_table(code);
    }
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&416_u32.to_le_bytes());
    code.bytes(&[0x3c, 1]);
    code.jump_if(0x84, "static-process-shell-terminal-clean-fstat");
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
    code.bytes(&[u8::from(!code.owned_user_extents)]);
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
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[9]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[6]);
    code.jump_if(0x85, "fatal");
    if code.owned_user_extents {
        process_runtime::shell_input::emit_require_exit(code);
    } else {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&u64::from_le_bytes(*b"exit\n\0\0\0").to_le_bytes());
        code.bytes(&[0x48, 0x39, 0x04, 0x25]);
        code.bytes(&0x005f_8f60_u32.to_le_bytes());
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&0x005f_8f68_u32.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[27]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[u8::from(!code.owned_user_extents)]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.jump("static-process-shell-root-condition-terminal-clear");

    code.bind("static-process-execve-true-exit")?;
    process_runtime::children::emit_exit_dispatch(code)?;
    code.bytes(&[0x65, 0x8a, 0x04, 0x25, 104, 0, 0, 0]);
    code.bytes(&[0x3c, 11]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x3c, 15]);
    code.jump_if(0x87, "fatal");
    process_runtime::emit_exit_witness(code);
    code.call("child-parent-release");
    code.call("fd-terminal-release");
    code.bytes(&[0x31, 0xff]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 16]);
    for (offset, expected) in [(737_u32, 0_u8), (892, 5)] {
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fatal");
    }
    for offset in (736_u32..=895).step_by(8) {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    for offset in [168_u32, 176, 224, 232] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&896_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump("static-process-terminal-phase-ready");

    code.bind("static-process-shell-terminal-clean-fstat")?;
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
    code.bytes(&[u8::from(!code.owned_user_extents)]);
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
    code.bytes(&[1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[10]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[6]);
    code.jump_if(0x85, "fatal");
    if code.owned_user_extents {
        process_runtime::shell_input::emit_require_exit(code);
    } else {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&u64::from_le_bytes(*b"exit\n\0\0\0").to_le_bytes());
        code.bytes(&[0x48, 0x39, 0x04, 0x25]);
        code.bytes(&0x005f_8f60_u32.to_le_bytes());
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&0x005f_8f68_u32.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for offset in (744_u32..888).step_by(8) {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&616_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&736_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&737_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&738_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&739_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&740_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&741_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&742_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[0]);
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
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&743_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[27]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-root-condition-terminal-clear")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&889_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&891_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[5]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&889_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&891_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&888_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&889_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&890_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&891_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&892_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-shell-terminal-header-ready")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 32]);
    code.jump_if(0x85, "fatal");
    emit_static_process_command_buffer_validation(code);
    if code.owned_user_extents {
        // Descriptor initialization transferred these dispositions into the
        // process-owned table. Require consumed prelude slots; table teardown
        // retires the current registrations, including subsequent changes.
        for offset in [168_u32, 176, 224, 232] {
            code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
            code.bytes(&offset.to_le_bytes());
            code.bytes(&[0]);
            code.jump_if(0x85, "fatal");
        }
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&417_u32.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    } else {
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
        code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        code.bytes(&417_u32.to_le_bytes());
        code.bytes(&[1]);
        code.jump_if(0x85, "fatal");
    }
    if code.owned_user_extents {
        code.call("fd-terminal-release");
        process_runtime::shell_input::emit_retire(code);
    } else {
        // Erase the process-visible command before releasing its consumed owner.
        code.bytes(&[0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&0x005f_8f60_u32.to_le_bytes());
        code.bytes(&0_u32.to_le_bytes());
        code.bytes(&[0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&0x005f_8f68_u32.to_le_bytes());
        code.bytes(&0_u32.to_le_bytes());
        code.bytes(&[0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&0x005f_8f60_u32.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&0x005f_8f68_u32.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&418_u32.to_le_bytes());
    code.bytes(&[0]);
    for offset in [168_u32, 176, 224, 232] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    code.bytes(&417_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bind("static-process-terminal-phase-ready")?;
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
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&256_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&264_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x84, "fatal");
    for offset in [192_u32, 200, 208, 216] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    for offset in [184_u32, 256, 264] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    code.bind("static-process-terminal-thread-state")?;
    // The owned first image and runtime sole-mm exits retire these opaque
    // registrations with the address space.
    if !code.owned_user_extents {
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&896_u32.to_le_bytes());
        code.bytes(&[1]);
        code.jump_if(0x85, "static-process-legacy-thread-release");
    }
    for offset in [152_u32, 160] {
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    if !code.owned_user_extents {
        code.jump("static-process-terminal-thread-state-ready");
        code.bind("static-process-legacy-thread-release")?;
        code.bytes(&[0x48, 0x85, 0xff]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25, 0xa0, 0, 0, 0]);
        code.bytes(&[0x48, 0x3d, 0xe0, 0x76, 0x5f, 0x00]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x48, 0x81, 0x38, 0xe0, 0x76, 0x5f, 0x00]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x48, 0x83, 0x78, 0x08, 0xe0]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x48, 0x83, 0x78, 0x10, 0]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25, 0xa0, 0, 0, 0, 0, 0, 0, 0]);
        code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25, 0x98, 0, 0, 0]);
        code.bytes(&[0x48, 0x3d, 0xd0, 0x76, 0x5f, 0x00]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0xc7, 0x00, 0, 0, 0, 0, 0x83, 0x38, 0]);
        code.jump_if(0x85, "fatal");
        code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25, 0x98, 0, 0, 0, 0, 0, 0, 0]);
        code.bind("static-process-terminal-thread-state-ready")?;
    }
    code.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    code.bytes(&896_u32.to_le_bytes());
    code.bytes(&[0]);
    code.bytes(&[0x65, 0x48, 0x89, 0x24, 0x25, 0x30, 0, 0, 0]);
    code.bytes(&[
        0x65, 0x48, 0x8b, 0x04, 0x25, 0x10, 0, 0, 0, 0x0f, 0x22, 0xd8,
    ]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x24, 0x25, 0x08, 0, 0, 0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0, 0, 0, 0, 2]);
    emit_static_process_clear_syscall_registers(code);
    code.bytes(&[0x0f, 0x01, 0xf8]);
    code.jump("returned");

    code.bind("static-process-brk")?;
    if frame_memory {
        frame_memory::brk(code, user_extent_probe)?;
    }
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "static-process-brk-query");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 1]);
    code.jump_if(0x84, "static-process-brk-initial-growth");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 6]);
    code.jump_if(0x84, "static-process-brk-post-random-growth");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 7]);
    code.jump_if(0x84, "static-process-brk-post-random-page-alignment");
    code.jump("fatal");

    code.bind("static-process-brk-initial-growth")?;
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x005f_7d80_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x70, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x78, 0, 0, 0]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4c, 0x89, 0xc0, 0x48, 0x83, 0xc8, 0x07, 0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x09, 0xc8, 0x48, 0x89, 0x82, 0xb8, 0x0f, 0, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0, 0x48, 0xb8]);
    code.bytes(&0x005f_7000_u64.to_le_bytes());
    code.bytes(&[0x0f, 0x01, 0x38, 0x0f, 0xae, 0xf0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 2]);
    code.bytes(&[0x48, 0x89, 0xf8]);
    emit_static_process_sysret(code);

    code.bind("static-process-brk-post-random-growth")?;
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x0061_8d80_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x89, 0x0c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x70, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x78, 0, 0, 0]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc0, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");

    // The existing first heap leaf must still name data frame zero while every
    // other low-table heap slot and the private next table remain absent.
    code.bytes(&[0x48, 0x8b, 0x82, 0xb8, 0x0f, 0, 0, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4d, 0x89, 0xc2, 0x49, 0x83, 0xca, 0x07]);
    code.bytes(&[0x49, 0x0f, 0xba, 0xea, 0x3f, 0x4c, 0x39, 0xd0]);
    code.jump_if(0x85, "fatal");
    for offset in (0x0fc0_u32..=0x0ff8).step_by(8) {
        code.bytes(&[0x48, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fatal");
    }
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x49, 0x8d, 0xba, 0x00, 0x40, 0, 0, 0x31, 0xc0]);
    code.bytes(&[0xb9, 0xfe, 0x01, 0, 0]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");

    // Fill the heap prefix in the already-published high table, then fill the
    // remaining entries in the low table. The high stack leaf stays disjoint.
    code.bytes(&[0x4d, 0x8d, 0x90, 0x00, 0x90, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x48, 0x8d, 0xb8, 0x00, 0x40, 0, 0, 0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0xb9, 25, 0, 0, 0]);
    code.bind("static-process-brk-post-random-next-leaves")?;
    code.bytes(&[0x4c, 0x89, 0xd0, 0x4c, 0x09, 0xc8, 0x48, 0xab]);
    code.bytes(&[0x49, 0x81, 0xc2, 0, 0x10, 0, 0, 0xff, 0xc9]);
    code.jump_if(0x85, "static-process-brk-post-random-next-leaves");

    code.bytes(&[0x4d, 0x8d, 0x90, 0x00, 0x10, 0, 0]);
    code.bytes(&[0x48, 0x8d, 0xba, 0xc0, 0x0f, 0, 0, 0xb9, 8, 0, 0, 0]);
    code.bind("static-process-brk-post-random-low-leaves")?;
    code.bytes(&[0x4c, 0x89, 0xd0, 0x4c, 0x09, 0xc8, 0x48, 0xab]);
    code.bytes(&[0x49, 0x81, 0xc2, 0, 0x10, 0, 0, 0xff, 0xc9]);
    code.jump_if(0x85, "static-process-brk-post-random-low-leaves");

    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0x18, 0x20, 0, 0, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[
        0x49, 0x8d, 0x92, 0x00, 0x40, 0, 0, 0x48, 0x83, 0xca, 0x07, 0x48, 0x39, 0xd0,
    ]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x005f_8000_u64.to_le_bytes());
    code.bytes(&[0xb9, 33, 0, 0, 0]);
    code.bind("static-process-brk-post-random-invalidate")?;
    code.bytes(&[0x0f, 0x01, 0x38, 0x48, 0x05, 0, 0x10, 0, 0, 0xff, 0xc9]);
    code.jump_if(0x85, "static-process-brk-post-random-invalidate");
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 7]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0xbf]);
    code.bytes(&0x0061_8d80_u32.to_le_bytes());
    code.bytes(&[0xb8]);
    code.bytes(&0x0061_8d80_u32.to_le_bytes());
    emit_static_process_sysret(code);

    code.bind("static-process-brk-post-random-page-alignment")?;
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x89, 0x0c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());

    // Revalidate every retained heap leaf without publishing another mapping.
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x70, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x78, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc0, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc2, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x89, 0xc1]);
    code.bytes(&[0x48, 0x8d, 0xba, 0xb8, 0x0f, 0, 0, 0xb9, 9, 0, 0, 0]);
    code.bind("static-process-brk-page-alignment-low-leaves")?;
    code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xce, 0x48, 0x83, 0xce, 0x07]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xee, 0x3f, 0x48, 0x39, 0xf0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc1, 0, 0x10, 0, 0, 0x48, 0x83, 0xc7, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, "static-process-brk-page-alignment-low-leaves");

    code.bytes(&[0x49, 0x8d, 0xba, 0, 0x40, 0, 0, 0xb9, 25, 0, 0, 0]);
    code.bind("static-process-brk-page-alignment-high-leaves")?;
    code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xce, 0x48, 0x83, 0xce, 0x07]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xee, 0x3f, 0x48, 0x39, 0xf0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc1, 0, 0x10, 0, 0, 0x48, 0x83, 0xc7, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, "static-process-brk-page-alignment-high-leaves");
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&485_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");

    // Both retained stack leaves must remain adjacent, present, writable,
    // non-executable, and physically disjoint from the anonymous extent.
    code.bytes(&[0x49, 0x8b, 0x82, 0xf0, 0x4f, 0, 0, 0x48, 0x89, 0xc2]);
    code.bytes(&[0x48, 0x83, 0xe2, 0x9f, 0x48, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x89, 0xd6, 0x81, 0xe6, 0xff, 0x0f, 0, 0, 0x83, 0xfe, 7]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x48, 0x0f, 0xba, 0xf2, 0x3f, 0x48, 0x81, 0xe2, 0, 0xf0, 0xff, 0xff,
    ]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0]);
    code.bytes(&[
        0x48, 0x0f, 0xba, 0xf0, 0x3f, 0x48, 0x25, 0, 0xf0, 0xff, 0xff,
    ]);
    code.bytes(&[0x48, 0x81, 0xc2, 0x00, 0x10, 0, 0, 0x48, 0x39, 0xc2]);
    code.jump_if(0x85, "fatal");

    code.bytes(&[0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0, 0x48, 0x89, 0xc2]);
    code.bytes(&[0x48, 0x83, 0xe2, 0x9f, 0x48, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x89, 0xd6, 0x81, 0xe6, 0xff, 0x0f, 0, 0, 0x83, 0xfe, 7]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x48, 0x0f, 0xba, 0xf2, 0x3f, 0x48, 0x81, 0xe2, 0, 0xf0, 0xff, 0xff,
    ]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4c, 0x39, 0xc2]);
    code.jump_if(0x82, "static-process-brk-page-alignment-stack-disjoint");
    code.bytes(&[0x4c, 0x89, 0xc0, 0x48, 0x05, 0, 0x20, 0x02, 0]);
    code.bytes(&[0x48, 0x39, 0xc2]);
    code.jump_if(0x82, "fatal");
    code.bind("static-process-brk-page-alignment-stack-disjoint")?;

    // The reusable random service remains live and quiescent.
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x3c, 3]);
    code.jump_if(0x87, "fatal");
    for offset in [168_u32, 176, 224, 232, 240, 248] {
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
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 8]);
    code.bytes(&[0xbf]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    code.bytes(&[0xb8]);
    code.bytes(&0x0061_9000_u32.to_le_bytes());
    emit_static_process_sysret(code);

    code.bind("static-process-relro-mprotect")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x68, 0, 0, 0, 8]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x005e_5000_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xfe]);
    code.bytes(&0x0000_8000_u32.to_le_bytes());
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x83, 0xfa, 1]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0040_1000_u32.to_le_bytes());
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0058_c285_u32.to_le_bytes());
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x65, 0x48, 0x89, 0x0c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());

    // Recover the checked image leaf table through the active hierarchy. The
    // exact GNU RELRO leaves must retain their original affine frames and
    // staged RW/NX role; target-maintained accessed/dirty bits are ignored.
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc0, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x39, 0xc0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x8d, 0x88, 0, 0x10, 0, 0, 0x49, 0x83, 0xc9, 0x27]);
    code.bytes(&[0x4d, 0x39, 0x08]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x8d, 0x88, 0, 0x20, 0, 0, 0x49, 0x83, 0xc9, 0x27]);
    code.bytes(&[0x4d, 0x39, 0x88, 0, 0x10, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x8b, 0x88, 0x10, 0x20, 0, 0]);
    code.bytes(&[
        0x4c, 0x89, 0xc8, 0x48, 0x83, 0xe0, 0x9f, 0x48, 0x83, 0xc8, 0x27,
    ]);
    code.bytes(&[0x49, 0x39, 0x80, 0x10, 0x20, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x81, 0xe1, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4d, 0x8b, 0x11, 0x49, 0x81, 0xe2, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x49, 0x81, 0xc2, 0, 0x50, 0x1e, 0]);
    code.bytes(&[0x4d, 0x8d, 0x81, 0x28, 0x0f, 0, 0, 0xb9, 8, 0, 0, 0]);
    code.bind("static-process-relro-mprotect-leaves")?;
    code.bytes(&[0x49, 0x8b, 0x00, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xd2, 0x48, 0x83, 0xca, 7]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xea, 0x3f, 0x48, 0x39, 0xd0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4c, 0x89, 0xd0, 0x48, 0x83, 0xc8, 5]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xe8, 0x3f, 0x49, 0x89, 0x00]);
    code.bytes(&[
        0x49, 0x81, 0xc2, 0, 0x10, 0, 0, 0x49, 0x83, 0xc0, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, "static-process-relro-mprotect-leaves");

    // Publish the complete edit, invalidate exactly the changed interval, and
    // read back every RO/NX leaf before exposing success.
    code.bytes(&[0x0f, 0xae, 0xf0, 0x48, 0xb8]);
    code.bytes(&0x005e_5000_u64.to_le_bytes());
    code.bytes(&[0xb9, 8, 0, 0, 0]);
    code.bind("static-process-relro-mprotect-invalidate")?;
    code.bytes(&[0x0f, 0x01, 0x38, 0x48, 0x05, 0, 0x10, 0, 0, 0xff, 0xc9]);
    code.jump_if(0x85, "static-process-relro-mprotect-invalidate");
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[
        0x49, 0x81, 0xea, 0, 0x80, 0, 0, 0x49, 0x83, 0xe8, 0x40, 0xb9, 8, 0, 0, 0,
    ]);
    code.bind("static-process-relro-mprotect-readback")?;
    code.bytes(&[0x49, 0x8b, 0x00, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xd2, 0x48, 0x83, 0xca, 5]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xea, 0x3f, 0x48, 0x39, 0xd0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc2, 0, 0x10, 0, 0, 0x49, 0x83, 0xc0, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, "static-process-relro-mprotect-readback");

    // Revalidate all 34 retained heap leaves and the disjoint stack leaf.
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x70, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x78, 0, 0, 0]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x18, 0, 0, 0]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0xf7, 0xc2, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x41, 0xf7, 0xc0, 0xff, 0x0f, 0, 0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0x18, 0x20, 0, 0, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[
        0x49, 0x8d, 0xba, 0, 0x40, 0, 0, 0x48, 0x83, 0xcf, 7, 0x48, 0x39, 0xf8,
    ]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x4d, 0x89, 0xc1]);
    code.bytes(&[0x48, 0x8d, 0xba, 0xb8, 0x0f, 0, 0, 0xb9, 9, 0, 0, 0]);
    code.bind("static-process-relro-mprotect-low-heap")?;
    code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xce, 0x48, 0x83, 0xce, 7]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xee, 0x3f, 0x48, 0x39, 0xf0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc1, 0, 0x10, 0, 0, 0x48, 0x83, 0xc7, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, "static-process-relro-mprotect-low-heap");
    code.bytes(&[0x49, 0x8d, 0xba, 0, 0x40, 0, 0, 0xb9, 25, 0, 0, 0]);
    code.bind("static-process-relro-mprotect-high-heap")?;
    code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xce, 0x48, 0x83, 0xce, 7]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xee, 0x3f, 0x48, 0x39, 0xf0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc1, 0, 0x10, 0, 0, 0x48, 0x83, 0xc7, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, "static-process-relro-mprotect-high-heap");
    code.bytes(&[0x31, 0xc0, 0xb9]);
    code.bytes(&485_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0xf0, 0x4f, 0, 0, 0x48, 0x89, 0xc2]);
    code.bytes(&[0x48, 0x83, 0xe2, 0x9f, 0x48, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x89, 0xd6, 0x81, 0xe6, 0xff, 0x0f, 0, 0, 0x83, 0xfe, 7]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x48, 0x0f, 0xba, 0xf2, 0x3f, 0x48, 0x81, 0xe2, 0, 0xf0, 0xff, 0xff,
    ]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0]);
    code.bytes(&[
        0x48, 0x0f, 0xba, 0xf0, 0x3f, 0x48, 0x25, 0, 0xf0, 0xff, 0xff,
    ]);
    code.bytes(&[0x48, 0x81, 0xc2, 0x00, 0x10, 0, 0, 0x48, 0x39, 0xc2]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[0x49, 0x8b, 0x82, 0xf8, 0x4f, 0, 0, 0x48, 0x89, 0xc2]);
    code.bytes(&[0x48, 0x83, 0xe2, 0x9f, 0x48, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fatal");
    code.bytes(&[0x89, 0xd6, 0x81, 0xe6, 0xff, 0x0f, 0, 0, 0x83, 0xfe, 7]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x48, 0x0f, 0xba, 0xf2, 0x3f, 0x48, 0x81, 0xe2, 0, 0xf0, 0xff, 0xff,
    ]);
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fatal");
    code.bytes(&[0x4c, 0x89, 0xc8, 0x48, 0x2d, 0, 0x20, 0x02, 0]);
    code.bytes(&[0x48, 0x39, 0xc2]);
    code.jump_if(0x82, "static-process-relro-mprotect-stack-disjoint");
    code.bytes(&[0x4c, 0x39, 0xca]);
    code.jump_if(0x82, "fatal");
    code.bind("static-process-relro-mprotect-stack-disjoint")?;

    // The reusable random service remains live and quiescent.
    code.bytes(&[0x65, 0x8a, 0x04, 0x25]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[0x3c, 2]);
    code.jump_if(0x82, "fatal");
    code.bytes(&[0x3c, 3]);
    code.jump_if(0x87, "fatal");
    for offset in [168_u32, 176, 224, 232, 240, 248] {
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
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25]);
    code.bytes(&272_u32.to_le_bytes());
    code.bytes(&[0, 0, 0, 0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x68, 0, 0, 0, 9]);
    code.bytes(&[0x31, 0xc0]);
    emit_static_process_sysret(code);

    Ok(())
}
