fn emit_static_image_resource_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    code.bytes(&[0x48, 0x85, 0xff]); // payload base
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // metadata base
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xf7, 0xc6, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x81, 0xfa, 0xf7, 0x01, 0x00, 0x00]); // exact 503 leaves
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0xf8]); // payload end
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x70, 0x1f, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x49, 0x89, 0xf1]); // metadata end
    code.bytes(&[0x49, 0x81, 0xc1, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x49, 0x39, 0xf9]); // metadata end <= payload base
    code.jump_if(0x86, "resources-disjoint");
    code.bytes(&[0x4c, 0x39, 0xc6]); // metadata base >= payload end
    code.jump_if(0x82, "fail");
    code.bind("resources-disjoint")
}

fn emit_static_image_leaf_verification(
    code: &mut X86FunctionEncoder,
    loop_label: &'static str,
) -> Result<(), CompileError> {
    code.bytes(&[0x49, 0x89, 0xfa]); // payload cursor
    code.bytes(&[0x48, 0x89, 0xf7]); // metadata cursor
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes()); // present/user/write/NX
    code.bytes(&[0xb9, 0xf7, 0x01, 0x00, 0x00]);
    code.bind(loop_label)?;
    code.bytes(&[0x4c, 0x89, 0xd0]);
    code.bytes(&[0x4c, 0x09, 0xc8]);
    code.bytes(&[0x48, 0x39, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xc7, 0x08]);
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, loop_label);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x09, 0x00, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]); // nine entries remain absent
    code.jump_if(0x85, "fail");
    Ok(())
}

fn emit_static_image_sealed_leaf_verification_with_relro(
    code: &mut X86FunctionEncoder,
    labels: [&'static str; 4],
    relro_sealed: bool,
) -> Result<(), CompileError> {
    fn emit_role(
        code: &mut X86FunctionEncoder,
        label: &'static str,
        count: u32,
        flags: u64,
    ) -> Result<(), CompileError> {
        code.bytes(&[0x49, 0xb9]);
        code.bytes(&flags.to_le_bytes());
        code.bytes(&[0xb9]);
        code.bytes(&count.to_le_bytes());
        code.bind(label)?;
        code.bytes(&[0x4c, 0x89, 0xd0]);
        code.bytes(&[0x4c, 0x09, 0xc8]);
        code.bytes(&[0x48, 0x39, 0x07]);
        code.jump_if(0x85, "fail");
        code.bytes(&[0x48, 0x83, 0xc7, 0x08]);
        code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
        code.bytes(&[0xff, 0xc9]);
        code.jump_if(0x85, label);
        Ok(())
    }

    code.bytes(&[0x49, 0x89, 0xfa]); // payload cursor
    code.bytes(&[0x48, 0x89, 0xf7]); // metadata cursor
    emit_role(code, labels[0], 1, 0x8000_0000_0000_0005)?;
    emit_role(code, labels[1], 396, 0x0000_0000_0000_0005)?;
    emit_role(
        code,
        labels[2],
        if relro_sealed { 96 } else { 88 },
        0x8000_0000_0000_0005,
    )?;
    emit_role(
        code,
        labels[3],
        if relro_sealed { 10 } else { 18 },
        0x8000_0000_0000_0007,
    )?;
    code.bytes(&[0x31, 0xc0, 0xb9, 0x09, 0x00, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fail");
    Ok(())
}

fn emit_static_image_sealed_leaf_verification(
    code: &mut X86FunctionEncoder,
    labels: [&'static str; 4],
) -> Result<(), CompileError> {
    emit_static_image_sealed_leaf_verification_with_relro(code, labels, false)
}

fn emit_static_image_post_relro_leaf_verification(
    code: &mut X86FunctionEncoder,
    labels: [&'static str; 4],
) -> Result<(), CompileError> {
    emit_static_image_sealed_leaf_verification_with_relro(code, labels, true)
}

fn static_image_prepare_initial_leaves() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    emit_static_image_resource_validation(&mut code)?;
    code.bytes(&[0x49, 0x89, 0xfa]); // payload cursor
    code.bytes(&[0x49, 0x89, 0xf3]); // retain metadata base
    code.bytes(&[0x48, 0x89, 0xf7]);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x02, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // zero all 512 entries
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x89, 0xdf]);
    code.bytes(&[0xb9, 0xf7, 0x01, 0x00, 0x00]);
    code.bind("construct-leaves")?;
    code.bytes(&[0x4c, 0x89, 0xd0]);
    code.bytes(&[0x4c, 0x09, 0xc8]);
    code.bytes(&[0x48, 0xab]);
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "construct-leaves");
    code.bytes(&[0x0f, 0xae, 0xf0]); // publish inactive leaves
    code.bytes(&[0x49, 0x81, 0xea, 0x00, 0x70, 0x1f, 0x00]);
    code.bytes(&[0x4c, 0x89, 0xd7]); // restore payload base
    code.bytes(&[0x4c, 0x89, 0xde]); // restore metadata base
    emit_static_image_leaf_verification(&mut code, "verify-leaves")?;
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_zero_payload() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    emit_static_image_resource_validation(&mut code)?;
    emit_static_image_leaf_verification(&mut code, "verify-leaves-before-zero")?;
    code.bytes(&[0x49, 0x81, 0xea, 0x00, 0x70, 0x1f, 0x00]); // payload base
    code.bytes(&[0x4c, 0x89, 0xd7]);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0xee, 0x03, 0x00]); // 503 * 4096 / 8 qwords
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // zero complete payload
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x4c, 0x89, 0xd7]);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0xee, 0x03, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]); // verify complete payload
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4c, 0x89, 0xd7]);
    emit_static_image_leaf_verification(&mut code, "verify-leaves-after-zero")?;
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_zero_stack_backing() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // nonzero backing base
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]); // page aligned
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0xb0, 0xff, 0x3f]); // complete frame bounded
    code.jump_if(0x87, "fail");
    code.bytes(&[0x49, 0x89, 0xf8]); // retain upper-page anchor
    code.bytes(&[0x48, 0x81, 0xef, 0x00, 0x10, 0x00, 0x00]); // extent base
    code.bytes(&[
        0xfc, 0x31, 0xc0, 0xb9, 0x00, 0x04, 0x00, 0x00, 0xf3, 0x48, 0xab,
    ]);
    code.bytes(&[0x0f, 0xae, 0xf8]); // sfence
    code.bytes(&[0x49, 0x8d, 0xb8, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x31, 0xc0, 0xb9, 0x00, 0x04, 0x00, 0x00, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn emit_static_image_published_stack_leaf_verification_with_relro(
    code: &mut X86FunctionEncoder,
    labels: [&'static str; 4],
    relro_sealed: bool,
) -> Result<(), CompileError> {
    fn emit_role(
        code: &mut X86FunctionEncoder,
        label: &'static str,
        count: u32,
        flags: u64,
    ) -> Result<(), CompileError> {
        code.bytes(&[0x49, 0xb9]);
        code.bytes(&flags.to_le_bytes());
        code.bytes(&[0xb9]);
        code.bytes(&count.to_le_bytes());
        code.bind(label)?;
        code.bytes(&[0x4c, 0x89, 0xd0]);
        code.bytes(&[0x4c, 0x09, 0xc8]);
        code.bytes(&[0x48, 0x39, 0x07]);
        code.jump_if(0x85, "fail");
        code.bytes(&[0x48, 0x83, 0xc7, 0x08]);
        code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
        code.bytes(&[0xff, 0xc9]);
        code.jump_if(0x85, label);
        Ok(())
    }

    code.bytes(&[0x49, 0x89, 0xfa]); // payload cursor
    code.bytes(&[0x48, 0x89, 0xf7]); // metadata cursor
    emit_role(code, labels[0], 1, 0x8000_0000_0000_0005)?;
    emit_role(code, labels[1], 396, 0x0000_0000_0000_0005)?;
    emit_role(
        code,
        labels[2],
        if relro_sealed { 96 } else { 88 },
        0x8000_0000_0000_0005,
    )?;
    emit_role(
        code,
        labels[3],
        if relro_sealed { 10 } else { 18 },
        0x8000_0000_0000_0007,
    )?;
    code.bytes(&[0x31, 0xc0, 0xb9, 0x09, 0x00, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]); // remaining low-table entries absent
    code.jump_if(0x85, "fail");
    code.bytes(&[
        0x48, 0x8b, 0x3c, 0x24, 0x48, 0x81, 0xc7, 0x00, 0x40, 0x00, 0x00,
    ]);
    code.bytes(&[0x31, 0xc0, 0xb9, 0xfe, 0x01, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]); // high-table prefix below the stack
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0xb8]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x89, 0xd8, 0x48, 0x2d, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x09, 0xc0]); // lower backing plus RW/NX user flags
    code.bytes(&[0x48, 0x39, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xc7, 0x08]);
    code.bytes(&[0x4c, 0x89, 0xd8, 0x4c, 0x09, 0xc0]); // upper backing leaf
    code.bytes(&[0x48, 0x39, 0x07]);
    code.jump_if(0x85, "fail");
    Ok(())
}

fn emit_static_image_published_stack_leaf_verification(
    code: &mut X86FunctionEncoder,
    labels: [&'static str; 4],
) -> Result<(), CompileError> {
    emit_static_image_published_stack_leaf_verification_with_relro(code, labels, false)
}

fn emit_static_image_stack_leaf_root_verification(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    code.bytes(&[0x4c, 0x8b, 0x44, 0x24, 0x18]); // qualified bootstrap root
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x41, 0xf7, 0xc0, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0xb0, 0xff, 0x3f]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x4c, 0x8b, 0x1c, 0x24, 0x4d, 0x39, 0xd8]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x0f, 0x20, 0xd8, 0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4c, 0x39, 0xc0]);
    code.jump_if(0x85, "fail");
    Ok(())
}

fn emit_static_image_stack_leaf_entry_verification(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    code.bytes(&[0x4c, 0x8b, 0x54, 0x24, 0x10]); // payload base
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x004a_6580_u64.to_le_bytes());
    code.bytes(&[0x49, 0x39, 0x42, 0x18]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x42, 0x18]);
    code.bytes(&[0x48, 0x2d, 0x00, 0x00, 0x40, 0x00]);
    code.bytes(&[0x48, 0x3d, 0x80, 0x65, 0x0a, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x89, 0xd0, 0x48, 0x05, 0x00, 0x60, 0x0a, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc8, 0x05]);
    code.bytes(&[0x49, 0x39, 0x83, 0x30, 0x05, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    Ok(())
}

fn emit_static_image_stack_backing_zero_verification(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]); // backing base
    code.bytes(&[0x4d, 0x85, 0xdb]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0xf7, 0xc3, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x81, 0xfb, 0x00, 0xb0, 0xff, 0x3f]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x49, 0x8d, 0xbb, 0x00, 0xf0, 0xff, 0xff, 0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x04, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fail");
    Ok(())
}

fn static_image_stack_leaf(publish: bool) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    emit_static_image_address_space_resource_validation(&mut code, true)?;
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    if publish {
        emit_static_image_sealed_leaf_verification_with_relro(
            &mut code,
            [
                "stack-publish-before-header",
                "stack-publish-before-executable",
                "stack-publish-before-relro",
                "stack-publish-before-writable",
            ],
            false,
        )?;
    } else {
        code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
        emit_static_image_published_stack_leaf_verification_with_relro(
            &mut code,
            [
                "stack-revoke-before-header",
                "stack-revoke-before-executable",
                "stack-revoke-before-relro",
                "stack-revoke-before-writable",
            ],
            true,
        )?;
    }
    emit_static_image_address_space_hierarchy_verification_labeled_with_stack(
        &mut code,
        true,
        "verify-stack-leaf-hierarchy-before-low-pd",
        "verify-stack-leaf-hierarchy-before-low-pd-expected",
        !publish,
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_stack_backing_zero_verification(&mut code)?;

    code.bytes(&[0x48, 0x8b, 0x34, 0x24]); // hierarchy base
    if publish {
        code.bytes(&[0x4c, 0x89, 0xd8, 0x48, 0x2d, 0x00, 0x10, 0x00, 0x00]);
        code.bytes(&[0x49, 0xb8]);
        code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
        code.bytes(&[0x4c, 0x09, 0xc0]);
        code.bytes(&[0x48, 0x89, 0x86, 0xf0, 0x4f, 0x00, 0x00]);
        code.bytes(&[0x4c, 0x89, 0xd8, 0x4c, 0x09, 0xc0]);
        code.bytes(&[0x48, 0x89, 0x86, 0xf8, 0x4f, 0x00, 0x00]);
    } else {
        code.bytes(&[
            0x48, 0xc7, 0x86, 0xf0, 0x4f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        code.bytes(&[
            0x48, 0xc7, 0x86, 0xf8, 0x4f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
    }
    code.bytes(&[0x0f, 0xae, 0xf8]); // sfence before complete readback

    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    if publish {
        code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
        emit_static_image_published_stack_leaf_verification_with_relro(
            &mut code,
            [
                "stack-publish-after-header",
                "stack-publish-after-executable",
                "stack-publish-after-relro",
                "stack-publish-after-writable",
            ],
            false,
        )?;
    } else {
        emit_static_image_sealed_leaf_verification_with_relro(
            &mut code,
            [
                "stack-revoke-after-header",
                "stack-revoke-after-executable",
                "stack-revoke-after-relro",
                "stack-revoke-after-writable",
            ],
            true,
        )?;
    }
    emit_static_image_address_space_hierarchy_verification_labeled_with_stack(
        &mut code,
        true,
        "verify-stack-leaf-hierarchy-after-low-pd",
        "verify-stack-leaf-hierarchy-after-low-pd-expected",
        publish,
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_stack_backing_zero_verification(&mut code)?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_publish_stack_leaf() -> Result<Vec<u8>, CompileError> {
    static_image_stack_leaf(true)
}

fn static_image_revoke_stack_leaf() -> Result<Vec<u8>, CompileError> {
    static_image_stack_leaf(false)
}

fn emit_static_image_initialized_stack_entry_verification(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    const STACK_ENTRY_OFFSET: u32 = 0x0fa0;
    const BUSYBOX_STRING_OFFSET: u32 = 0x0fe0;
    const SHELL_STRING_OFFSET: u32 = 0x0fe8;
    const BUSYBOX_POINTER: u64 = 0x007f_ffe0;
    const SHELL_POINTER: u64 = 0x007f_ffe8;
    const BUSYBOX_BYTES: u64 = 0x0078_6f62_7973_7562;
    const SHELL_BYTES: u64 = 0x0000_0000_0000_6873;

    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]); // backing base
    code.bytes(&[0x4d, 0x85, 0xdb]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0xf7, 0xc3, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0]);
    code.bytes(&[0xb9]);
    code.bytes(&500_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]); // zero prefix through qword 499
    code.jump_if(0x85, "fail");

    code.bytes(&[0x49, 0x83, 0xbb]);
    code.bytes(&STACK_ENTRY_OFFSET.to_le_bytes());
    code.bytes(&[0x02]);
    code.jump_if(0x85, "fail");
    for (offset, expected) in [
        (STACK_ENTRY_OFFSET + 8, BUSYBOX_POINTER),
        (STACK_ENTRY_OFFSET + 16, SHELL_POINTER),
        (BUSYBOX_STRING_OFFSET, BUSYBOX_BYTES),
        (SHELL_STRING_OFFSET, SHELL_BYTES),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x49, 0x39, 0x83]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fail");
    }
    for offset in [
        STACK_ENTRY_OFFSET + 24,
        STACK_ENTRY_OFFSET + 32,
        STACK_ENTRY_OFFSET + 40,
        STACK_ENTRY_OFFSET + 48,
        STACK_ENTRY_OFFSET + 56,
        SHELL_STRING_OFFSET + 8,
        SHELL_STRING_OFFSET + 16,
    ] {
        code.bytes(&[0x49, 0x83, 0xbb]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0x00]);
        code.jump_if(0x85, "fail");
    }
    Ok(())
}

fn static_image_stack_entry(initialize: bool) -> Result<Vec<u8>, CompileError> {
    const STACK_ENTRY_OFFSET: u32 = 0x0fa0;
    const BUSYBOX_STRING_OFFSET: u32 = 0x0fe0;
    const SHELL_STRING_OFFSET: u32 = 0x0fe8;

    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    emit_static_image_address_space_resource_validation(&mut code, true)?;
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if initialize {
            [
                "stack-entry-init-before-header",
                "stack-entry-init-before-executable",
                "stack-entry-init-before-relro",
                "stack-entry-init-before-writable",
            ]
        } else {
            [
                "stack-entry-clear-before-header",
                "stack-entry-clear-before-executable",
                "stack-entry-clear-before-relro",
                "stack-entry-clear-before-writable",
            ]
        },
        !initialize,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if initialize {
            "verify-stack-entry-init-before-low-pd"
        } else {
            "verify-stack-entry-clear-before-low-pd"
        },
        if initialize {
            "verify-stack-entry-init-before-low-pd-expected"
        } else {
            "verify-stack-entry-clear-before-low-pd-expected"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    if initialize {
        emit_static_image_stack_backing_zero_verification(&mut code)?;
    } else {
        emit_static_image_initialized_stack_entry_verification(&mut code)?;
    }

    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    if initialize {
        code.bytes(&[0x49, 0xc7, 0x83]);
        code.bytes(&STACK_ENTRY_OFFSET.to_le_bytes());
        code.bytes(&2_u32.to_le_bytes());
        for (offset, value) in [
            (STACK_ENTRY_OFFSET + 8, 0x007f_ffe0_u64),
            (STACK_ENTRY_OFFSET + 16, 0x007f_ffe8_u64),
            (BUSYBOX_STRING_OFFSET, 0x0078_6f62_7973_7562_u64),
            (SHELL_STRING_OFFSET, 0x0000_0000_0000_6873_u64),
        ] {
            code.bytes(&[0x48, 0xb8]);
            code.bytes(&value.to_le_bytes());
            code.bytes(&[0x49, 0x89, 0x83]);
            code.bytes(&offset.to_le_bytes());
        }
    } else {
        code.bytes(&[0x49, 0x8d, 0xbb, 0x00, 0xf0, 0xff, 0xff, 0x31, 0xc0]);
        code.bytes(&[0xb9, 0x00, 0x04, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab]);
    }
    code.bytes(&[0x0f, 0xae, 0xf8]); // sfence before complete readback

    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if initialize {
            [
                "stack-entry-init-after-header",
                "stack-entry-init-after-executable",
                "stack-entry-init-after-relro",
                "stack-entry-init-after-writable",
            ]
        } else {
            [
                "stack-entry-clear-after-header",
                "stack-entry-clear-after-executable",
                "stack-entry-clear-after-relro",
                "stack-entry-clear-after-writable",
            ]
        },
        !initialize,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if initialize {
            "verify-stack-entry-init-after-low-pd"
        } else {
            "verify-stack-entry-clear-after-low-pd"
        },
        if initialize {
            "verify-stack-entry-init-after-low-pd-expected"
        } else {
            "verify-stack-entry-clear-after-low-pd-expected"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    if initialize {
        emit_static_image_initialized_stack_entry_verification(&mut code)?;
    } else {
        emit_static_image_stack_backing_zero_verification(&mut code)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_initialize_stack_entry() -> Result<Vec<u8>, CompileError> {
    static_image_stack_entry(true)
}

fn static_image_clear_stack_entry() -> Result<Vec<u8>, CompileError> {
    static_image_stack_entry(false)
}

fn emit_static_image_pagesz_stack_entry_verification(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    const ENTRY_OFFSET: u32 = 0x0f90;
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
    code.bytes(&498_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fail");
    for (offset, expected) in [
        (ENTRY_OFFSET, 2_u64),
        (ENTRY_OFFSET + 8, 0x007f_ffe0),
        (ENTRY_OFFSET + 16, 0x007f_ffe8),
        (ENTRY_OFFSET + 40, 6),
        (ENTRY_OFFSET + 48, 4096),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x49, 0x39, 0x83]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fail");
    }
    for offset in [
        ENTRY_OFFSET + 24,
        ENTRY_OFFSET + 32,
        ENTRY_OFFSET + 56,
        ENTRY_OFFSET + 64,
        ENTRY_OFFSET + 72,
        0x0ff0,
        0x0ff8,
    ] {
        code.bytes(&[0x49, 0x83, 0xbb]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0x00]);
        code.jump_if(0x85, "fail");
    }
    Ok(())
}

fn emit_static_image_stack_entry_words(code: &mut X86FunctionEncoder, pagesz: bool) {
    let entry_offset = if pagesz { 0x0f90_u32 } else { 0x0fa0_u32 };
    for (offset, value) in [
        (entry_offset, 2_u64),
        (entry_offset + 8, 0x007f_ffe0),
        (entry_offset + 16, 0x007f_ffe8),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x49, 0x89, 0x83]);
        code.bytes(&offset.to_le_bytes());
    }
    if pagesz {
        for (offset, value) in [(entry_offset + 40, 6_u64), (entry_offset + 48, 4096)] {
            code.bytes(&[0x48, 0xb8]);
            code.bytes(&value.to_le_bytes());
            code.bytes(&[0x49, 0x89, 0x83]);
            code.bytes(&offset.to_le_bytes());
        }
    }
    for (offset, value) in [
        (0x0fe0_u32, 0x0078_6f62_7973_7562_u64),
        (0x0fe8, 0x0000_0000_0000_6873),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x49, 0x89, 0x83]);
        code.bytes(&offset.to_le_bytes());
    }
}

fn static_image_stack_pagesz(add: bool) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    emit_static_image_address_space_resource_validation(&mut code, true)?;
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "stack-pagesz-add-before-header",
                "stack-pagesz-add-before-executable",
                "stack-pagesz-add-before-relro",
                "stack-pagesz-add-before-writable",
            ]
        } else {
            [
                "stack-pagesz-remove-before-header",
                "stack-pagesz-remove-before-executable",
                "stack-pagesz-remove-before-relro",
                "stack-pagesz-remove-before-writable",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "verify-stack-pagesz-add-before-low-pd"
        } else {
            "verify-stack-pagesz-remove-before-low-pd"
        },
        if add {
            "verify-stack-pagesz-add-before-low-pd-expected"
        } else {
            "verify-stack-pagesz-remove-before-low-pd-expected"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    if add {
        emit_static_image_initialized_stack_entry_verification(&mut code)?;
    } else {
        emit_static_image_pagesz_stack_entry_verification(&mut code)?;
    }

    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x02, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab]);
    emit_static_image_stack_entry_words(&mut code, add);
    code.bytes(&[0x0f, 0xae, 0xf8]);

    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "stack-pagesz-add-after-header",
                "stack-pagesz-add-after-executable",
                "stack-pagesz-add-after-relro",
                "stack-pagesz-add-after-writable",
            ]
        } else {
            [
                "stack-pagesz-remove-after-header",
                "stack-pagesz-remove-after-executable",
                "stack-pagesz-remove-after-relro",
                "stack-pagesz-remove-after-writable",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "verify-stack-pagesz-add-after-low-pd"
        } else {
            "verify-stack-pagesz-remove-after-low-pd"
        },
        if add {
            "verify-stack-pagesz-add-after-low-pd-expected"
        } else {
            "verify-stack-pagesz-remove-after-low-pd-expected"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    if add {
        emit_static_image_pagesz_stack_entry_verification(&mut code)?;
    } else {
        emit_static_image_initialized_stack_entry_verification(&mut code)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_pagesz() -> Result<Vec<u8>, CompileError> {
    static_image_stack_pagesz(true)
}

fn static_image_remove_stack_pagesz() -> Result<Vec<u8>, CompileError> {
    static_image_stack_pagesz(false)
}

fn emit_static_image_entry_fact_stack_verification(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    const ENTRY_OFFSET: u32 = 0x0f80;
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
    code.bytes(&496_u32.to_le_bytes());
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    code.jump_if(0x85, "fail");
    for (offset, expected) in [
        (ENTRY_OFFSET, 2_u64),
        (ENTRY_OFFSET + 8, 0x007f_ffe0),
        (ENTRY_OFFSET + 16, 0x007f_ffe8),
        (ENTRY_OFFSET + 40, 6),
        (ENTRY_OFFSET + 48, 4096),
        (ENTRY_OFFSET + 56, 9),
        (ENTRY_OFFSET + 64, 0x004a_6580),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&expected.to_le_bytes());
        code.bytes(&[0x49, 0x39, 0x83]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fail");
    }
    for offset in [
        ENTRY_OFFSET + 24,
        ENTRY_OFFSET + 32,
        ENTRY_OFFSET + 72,
        ENTRY_OFFSET + 80,
        ENTRY_OFFSET + 88,
        0x0ff0,
        0x0ff8,
    ] {
        code.bytes(&[0x49, 0x83, 0xbb]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0x00]);
        code.jump_if(0x85, "fail");
    }
    Ok(())
}

fn emit_static_image_entry_fact_stack_words(code: &mut X86FunctionEncoder) {
    const ENTRY_OFFSET: u32 = 0x0f80;
    for (offset, value) in [
        (ENTRY_OFFSET, 2_u64),
        (ENTRY_OFFSET + 8, 0x007f_ffe0),
        (ENTRY_OFFSET + 16, 0x007f_ffe8),
        (ENTRY_OFFSET + 40, 6),
        (ENTRY_OFFSET + 48, 4096),
        (ENTRY_OFFSET + 56, 9),
        (ENTRY_OFFSET + 64, 0x004a_6580),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x49, 0x89, 0x83]);
        code.bytes(&offset.to_le_bytes());
    }
}

fn static_image_stack_entry_fact(add: bool) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    emit_static_image_address_space_resource_validation(&mut code, true)?;
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "stack-entry-fact-add-before-header",
                "stack-entry-fact-add-before-executable",
                "stack-entry-fact-add-before-relro",
                "stack-entry-fact-add-before-writable",
            ]
        } else {
            [
                "stack-entry-fact-remove-before-header",
                "stack-entry-fact-remove-before-executable",
                "stack-entry-fact-remove-before-relro",
                "stack-entry-fact-remove-before-writable",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "verify-stack-entry-fact-add-before-low-pd"
        } else {
            "verify-stack-entry-fact-remove-before-low-pd"
        },
        if add {
            "verify-stack-entry-fact-add-before-low-pd-expected"
        } else {
            "verify-stack-entry-fact-remove-before-low-pd-expected"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    if add {
        emit_static_image_pagesz_stack_entry_verification(&mut code)?;
    } else {
        emit_static_image_entry_fact_stack_verification(&mut code)?;
    }

    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x02, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab]);
    if add {
        emit_static_image_entry_fact_stack_words(&mut code);
    } else {
        emit_static_image_stack_entry_words(&mut code, true);
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);

    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "stack-entry-fact-add-after-header",
                "stack-entry-fact-add-after-executable",
                "stack-entry-fact-add-after-relro",
                "stack-entry-fact-add-after-writable",
            ]
        } else {
            [
                "stack-entry-fact-remove-after-header",
                "stack-entry-fact-remove-after-executable",
                "stack-entry-fact-remove-after-relro",
                "stack-entry-fact-remove-after-writable",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "verify-stack-entry-fact-add-after-low-pd"
        } else {
            "verify-stack-entry-fact-remove-after-low-pd"
        },
        if add {
            "verify-stack-entry-fact-add-after-low-pd-expected"
        } else {
            "verify-stack-entry-fact-remove-after-low-pd-expected"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    if add {
        emit_static_image_entry_fact_stack_verification(&mut code)?;
    } else {
        emit_static_image_pagesz_stack_entry_verification(&mut code)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_entry_fact() -> Result<Vec<u8>, CompileError> {
    static_image_stack_entry_fact(true)
}

fn static_image_remove_stack_entry_fact() -> Result<Vec<u8>, CompileError> {
    static_image_stack_entry_fact(false)
}
