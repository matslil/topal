fn emit_static_process_active_stack_validation(code: &mut X86FunctionEncoder) {
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
}

fn emit_static_process_child_disposition_validation(code: &mut X86FunctionEncoder) {
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
}

fn emit_static_process_heap_validation(
    code: &mut X86FunctionEncoder,
    low_label: &'static str,
    high_label: &'static str,
) -> Result<(), CompileError> {
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
    code.bytes(&[0x4d, 0x89, 0xc1]);
    code.bytes(&[0x48, 0x8d, 0xba, 0xb8, 0x0f, 0, 0, 0xb9, 9, 0, 0, 0]);
    code.bind(low_label)?;
    code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xce, 0x48, 0x83, 0xce, 7]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xee, 0x3f, 0x48, 0x39, 0xf0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc1, 0, 0x10, 0, 0, 0x48, 0x83, 0xc7, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, low_label);
    code.bytes(&[0x49, 0x8d, 0xba, 0, 0x40, 0, 0, 0xb9, 25, 0, 0, 0]);
    code.bind(high_label)?;
    code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xce, 0x48, 0x83, 0xce, 7]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xee, 0x3f, 0x48, 0x39, 0xf0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc1, 0, 0x10, 0, 0, 0x48, 0x83, 0xc7, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, high_label);
    Ok(())
}

fn emit_static_process_low_heap_validation(
    code: &mut X86FunctionEncoder,
    low_label: &'static str,
) -> Result<(), CompileError> {
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
    code.bytes(&[0x4d, 0x89, 0xc1]);
    code.bytes(&[0x48, 0x8d, 0xba, 0xb8, 0x0f, 0, 0, 0xb9, 9, 0, 0, 0]);
    code.bind(low_label)?;
    code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xce, 0x48, 0x83, 0xce, 7]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xee, 0x3f, 0x48, 0x39, 0xf0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc1, 0, 0x10, 0, 0, 0x48, 0x83, 0xc7, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, low_label);
    Ok(())
}

fn emit_static_process_command_buffer_validation(code: &mut X86FunctionEncoder) {
    // The bounded command extent lies wholly in the second anonymous heap leaf.
    // Revalidate its affine frame identity and writable/NX permissions.
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
}

fn emit_static_process_shell_output_heap_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
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
    code.bytes(&[0x4d, 0x89, 0xc1]);
    code.bytes(&[0x48, 0x8d, 0xba, 0xb8, 0x0f, 0, 0, 0xb9, 9, 0, 0, 0]);
    code.bind("static-process-shell-echo-low-heap")?;
    code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xce, 0x48, 0x83, 0xce, 7]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xee, 0x3f, 0x48, 0x39, 0xf0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc1, 0, 0x10, 0, 0, 0x48, 0x83, 0xc7, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, "static-process-shell-echo-low-heap");
    code.bytes(&[0x49, 0x8d, 0xba, 0, 0x40, 0, 0, 0xb9, 25, 0, 0, 0]);
    code.bind("static-process-shell-echo-high-heap")?;
    code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x83, 0xe0, 0x9f]);
    code.bytes(&[0x4c, 0x89, 0xce, 0x48, 0x83, 0xce, 7]);
    code.bytes(&[0x48, 0x0f, 0xba, 0xee, 0x3f, 0x48, 0x39, 0xf0]);
    code.jump_if(0x85, "fatal");
    code.bytes(&[
        0x49, 0x81, 0xc1, 0, 0x10, 0, 0, 0x48, 0x83, 0xc7, 8, 0xff, 0xc9,
    ]);
    code.jump_if(0x85, "static-process-shell-echo-high-heap");
    Ok(())
}

fn static_image_process_enter(
    source_bound: bool,
    file_open_plan: bool,
    source_numeric: bool,
    source_image: bool,
    source_dispatch: bool,
    dispatch_policy: crate::systems_numeric_library::NumericPolicy,
    source_identity: bool,
    retained_identity: bool,
    structured_identity: bool,
    source_position: bool,
    source_page: bool,
    source_inode: bool,
    source_namespace: bool,
) -> Result<LinkableX86Function, CompileError> {
    let mut code = X86FunctionEncoder::default();
    emit_static_process_start(
        &mut code,
        source_bound,
        file_open_plan,
        source_numeric,
        source_image,
        source_dispatch,
        dispatch_policy,
        source_identity,
        retained_identity,
        source_inode,
    )?;
    emit_static_process_read_exec(&mut code, source_image)?;
    emit_static_process_files(&mut code)?;
    emit_static_process_output(&mut code)?;
    emit_static_process_completion(
        &mut code,
        source_bound,
        file_open_plan,
        source_numeric,
        source_image,
        source_dispatch,
        source_identity,
        retained_identity,
        structured_identity,
        source_position,
        source_page,
        source_inode,
        source_namespace,
    )?;
    code.finish_linkable()
}

include!("systems_process_entry_start.rs");
include!("systems_process_entry_read_exec.rs");
include!("systems_process_entry_files.rs");
include!("systems_process_entry_output.rs");
include!("systems_process_entry_completion.rs");

fn static_image_process_restore_entry() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 2]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0x7f, 104, 6]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x48, 0x3b, 0x47, 0x10]);
    code.jump_if(0x85, "fail");
    for offset in [8_u8, 32, 40, 48, 56, 64, 72, 80, 88, 96] {
        code.bytes(&[0x48, 0x83, 0x7f, offset, 0]);
        code.jump_if(0x85, "fail");
    }
    for offset in (112_u32..=616).step_by(8) {
        if offset < 128 {
            code.bytes(&[0x48, 0x83, 0x7f, u8::try_from(offset).unwrap(), 0]);
        } else {
            code.bytes(&[0x48, 0x83, 0xbf]);
            code.bytes(&offset.to_le_bytes());
            code.bytes(&[0]);
        }
        code.jump_if(0x85, "fail");
    }
    code.bytes(&[0x31, 0xc0, 0xb9, 78, 0, 0, 0, 0xfc, 0xf3, 0x48, 0xab]);
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_revoke_entropy_initial_random() -> Result<Vec<u8>, CompileError> {
    const OBJECT: u32 = 0x0e70;
    const TAG: u32 = 0x0fc8;
    const VALUE: u32 = 0x0fd0;
    const TERMINATOR: u32 = 0x0fd8;
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    for (offset, value) in [(TAG, 25_u64), (VALUE, 0x007f_fe70)] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x48, 0x39, 0x87]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fail");
    }
    code.bytes(&[0x48, 0x83, 0xbf]);
    code.bytes(&TERMINATOR.to_le_bytes());
    code.bytes(&[0]);
    code.jump_if(0x85, "fail");
    for offset in [OBJECT, OBJECT + 8, TAG, VALUE] {
        code.bytes(&[0x48, 0xc7, 0x87]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
        code.bytes(&[0x48, 0x83, 0xbf]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fail");
    }
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_revoke_entropy_acquisition() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(false)
}

fn static_image_revoke_entropy_handoff() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(false)
}
