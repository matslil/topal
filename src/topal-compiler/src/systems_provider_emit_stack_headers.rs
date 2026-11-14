fn emit_static_image_program_header_stack_verification(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    const ENTRY_OFFSET: u32 = 0x0f50;
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
    code.bytes(&490_u32.to_le_bytes());
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
        (ENTRY_OFFSET + 72, 3),
        (ENTRY_OFFSET + 80, 0x0040_0040),
        (ENTRY_OFFSET + 88, 4),
        (ENTRY_OFFSET + 96, 56),
        (ENTRY_OFFSET + 104, 5),
        (ENTRY_OFFSET + 112, 11),
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
        ENTRY_OFFSET + 120,
        ENTRY_OFFSET + 128,
        ENTRY_OFFSET + 136,
        0x0ff0,
        0x0ff8,
    ] {
        code.bytes(&[0x49, 0x83, 0xbb]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fail");
    }
    Ok(())
}

fn emit_static_image_resident_program_header_verification(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let image = qualified_x86_64_static_user_image_fixture();
    code.bytes(&[0x4c, 0x8b, 0x54, 0x24, 0x10]);
    for (offset, expected) in image[..64 + 11 * 56].iter().copied().enumerate() {
        if (40..48).contains(&offset) || (58..64).contains(&offset) {
            continue;
        }
        let offset = u32::try_from(offset)
            .map_err(|_| CompileError::Tool("resident program-header offset exceeds u32".into()))?;
        code.bytes(&[0x41, 0x80, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[expected]);
        code.jump_if(0x85, "fail");
    }

    // Select one complete coherent section-directory tuple without parsing
    // its non-loadable contents. The first program-header kind occupies the
    // final two bytes of both qword constants and remains independently
    // checked above.
    code.bytes(&[0x49, 0x8b, 0x42, 0x28]);
    code.bytes(&[0x45, 0x31, 0xc9, 0x41, 0xb8]);
    code.bytes(&0x001e_dea0_u32.to_le_bytes());
    code.bytes(&[0x48, 0x85, 0xc0, 0x4d, 0x0f, 0x45, 0xc8, 0x4c, 0x39, 0xc8]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x0001_0000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x49, 0xb8]);
    code.bytes(&0x0001_0016_0017_0040_u64.to_le_bytes());
    code.bytes(&[0x48, 0x85, 0xc0, 0x4d, 0x0f, 0x45, 0xc8]);
    code.bytes(&[0x4d, 0x39, 0x4a, 0x3a]);
    code.jump_if(0x85, "fail");
    Ok(())
}

fn emit_static_image_program_header_stack_words(code: &mut X86FunctionEncoder) {
    const ENTRY_OFFSET: u32 = 0x0f50;
    for (offset, value) in [
        (ENTRY_OFFSET, 2_u64),
        (ENTRY_OFFSET + 8, 0x007f_ffe0),
        (ENTRY_OFFSET + 16, 0x007f_ffe8),
        (ENTRY_OFFSET + 40, 6),
        (ENTRY_OFFSET + 48, 4096),
        (ENTRY_OFFSET + 56, 9),
        (ENTRY_OFFSET + 64, 0x004a_6580),
        (ENTRY_OFFSET + 72, 3),
        (ENTRY_OFFSET + 80, 0x0040_0040),
        (ENTRY_OFFSET + 88, 4),
        (ENTRY_OFFSET + 96, 56),
        (ENTRY_OFFSET + 104, 5),
        (ENTRY_OFFSET + 112, 11),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
    ] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x49, 0x89, 0x83]);
        code.bytes(&offset.to_le_bytes());
    }
}

fn static_image_stack_program_headers(add: bool) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    emit_static_image_address_space_resource_validation(&mut code, true)?;
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    let before = if add {
        [
            "ph-add-header",
            "ph-add-executable",
            "ph-add-relro",
            "ph-add-writable",
        ]
    } else {
        [
            "ph-remove-header",
            "ph-remove-executable",
            "ph-remove-relro",
            "ph-remove-writable",
        ]
    };
    emit_static_image_published_stack_leaf_verification_with_relro(&mut code, before, !add)?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "ph-add-low-pd"
        } else {
            "ph-remove-low-pd"
        },
        if add {
            "ph-add-low-pd-expected"
        } else {
            "ph-remove-low-pd-expected"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_entry_fact_stack_verification(&mut code)?;
    } else {
        emit_static_image_program_header_stack_verification(&mut code)?;
    }
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x02, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab]);
    if add {
        emit_static_image_program_header_stack_words(&mut code);
    } else {
        emit_static_image_entry_fact_stack_words(&mut code);
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    code.bytes(&[0x48, 0x8b, 0x7c, 0x24, 0x10]);
    code.bytes(&[0x48, 0x8b, 0x74, 0x24, 0x08]);
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    let after = if add {
        [
            "ph-add-after-header",
            "ph-add-after-executable",
            "ph-add-after-relro",
            "ph-add-after-writable",
        ]
    } else {
        [
            "ph-remove-after-header",
            "ph-remove-after-executable",
            "ph-remove-after-relro",
            "ph-remove-after-writable",
        ]
    };
    emit_static_image_published_stack_leaf_verification_with_relro(&mut code, after, !add)?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "ph-add-after-low-pd"
        } else {
            "ph-remove-after-low-pd"
        },
        if add {
            "ph-add-after-low-pd-expected"
        } else {
            "ph-remove-after-low-pd-expected"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_program_header_stack_verification(&mut code)?;
    } else {
        emit_static_image_entry_fact_stack_verification(&mut code)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_program_headers() -> Result<Vec<u8>, CompileError> {
    static_image_stack_program_headers(true)
}

fn static_image_remove_stack_program_headers() -> Result<Vec<u8>, CompileError> {
    static_image_stack_program_headers(false)
}

fn emit_static_image_execfn_stack(
    code: &mut X86FunctionEncoder,
    verify: bool,
) -> Result<(), CompileError> {
    const O: u32 = 0x0f40;
    let words = [
        (O, 2_u64),
        (O + 8, 0x007f_ffe0),
        (O + 16, 0x007f_ffe8),
        (O + 40, 6),
        (O + 48, 4096),
        (O + 56, 9),
        (O + 64, 0x004a_6580),
        (O + 72, 3),
        (O + 80, 0x0040_0040),
        (O + 88, 4),
        (O + 96, 56),
        (O + 104, 5),
        (O + 112, 11),
        (O + 120, 31),
        (O + 128, 0x007f_ffe0),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
    ];
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    if verify {
        code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
        code.bytes(&488_u32.to_le_bytes());
        code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
        code.jump_if(0x85, "fail");
    }
    for (offset, value) in words {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        if verify {
            code.bytes(&[0x49, 0x39, 0x83]);
            code.bytes(&offset.to_le_bytes());
            code.jump_if(0x85, "fail");
        } else {
            code.bytes(&[0x49, 0x89, 0x83]);
            code.bytes(&offset.to_le_bytes());
        }
    }
    if verify {
        for offset in [O + 24, O + 32, O + 136, O + 144, O + 152, 0x0ff0, 0x0ff8] {
            code.bytes(&[0x49, 0x83, 0xbb]);
            code.bytes(&offset.to_le_bytes());
            code.bytes(&[0]);
            code.jump_if(0x85, "fail");
        }
    }
    Ok(())
}

fn static_image_stack_executable_name(add: bool) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    emit_static_image_address_space_resource_validation(&mut code, true)?;
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "execfn-add-h",
                "execfn-add-x",
                "execfn-add-r",
                "execfn-add-w",
            ]
        } else {
            [
                "execfn-remove-h",
                "execfn-remove-x",
                "execfn-remove-r",
                "execfn-remove-w",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "execfn-add-pd"
        } else {
            "execfn-remove-pd"
        },
        if add {
            "execfn-add-pd-e"
        } else {
            "execfn-remove-pd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_program_header_stack_verification(&mut code)?;
    } else {
        emit_static_image_execfn_stack(&mut code, true)?;
    }
    code.bytes(&[
        0x4c, 0x8b, 0x5c, 0x24, 0x20, 0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc,
        0xf3, 0x48, 0xab,
    ]);
    if add {
        emit_static_image_execfn_stack(&mut code, false)?;
    } else {
        emit_static_image_program_header_stack_words(&mut code);
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "execfn-add-ah",
                "execfn-add-ax",
                "execfn-add-ar",
                "execfn-add-aw",
            ]
        } else {
            [
                "execfn-remove-ah",
                "execfn-remove-ax",
                "execfn-remove-ar",
                "execfn-remove-aw",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "execfn-add-apd"
        } else {
            "execfn-remove-apd"
        },
        if add {
            "execfn-add-apd-e"
        } else {
            "execfn-remove-apd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_execfn_stack(&mut code, true)?;
    } else {
        emit_static_image_program_header_stack_verification(&mut code)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_executable_name() -> Result<Vec<u8>, CompileError> {
    static_image_stack_executable_name(true)
}
fn static_image_remove_stack_executable_name() -> Result<Vec<u8>, CompileError> {
    static_image_stack_executable_name(false)
}
