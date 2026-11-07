fn emit_static_image_credential_stack(
    code: &mut X86FunctionEncoder,
    verify: bool,
) -> Result<(), CompileError> {
    const O: u32 = 0x0f00;
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
        (O + 136, 11),
        (O + 152, 12),
        (O + 168, 13),
        (O + 184, 14),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
    ];
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    if verify {
        code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
        code.bytes(&480_u32.to_le_bytes());
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
        for offset in [
            O + 24,
            O + 32,
            O + 144,
            O + 160,
            O + 176,
            O + 192,
            O + 200,
            O + 208,
            O + 216,
            0x0ff0,
            0x0ff8,
        ] {
            code.bytes(&[0x49, 0x83, 0xbb]);
            code.bytes(&offset.to_le_bytes());
            code.bytes(&[0]);
            code.jump_if(0x85, "fail");
        }
    }
    Ok(())
}

fn static_image_stack_credentials(add: bool) -> Result<Vec<u8>, CompileError> {
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
                "credentials-add-h",
                "credentials-add-x",
                "credentials-add-r",
                "credentials-add-w",
            ]
        } else {
            [
                "credentials-remove-h",
                "credentials-remove-x",
                "credentials-remove-r",
                "credentials-remove-w",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "credentials-add-pd"
        } else {
            "credentials-remove-pd"
        },
        if add {
            "credentials-add-pd-e"
        } else {
            "credentials-remove-pd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_execfn_stack(&mut code, true)?;
    } else {
        emit_static_image_credential_stack(&mut code, true)?;
    }
    code.bytes(&[
        0x4c, 0x8b, 0x5c, 0x24, 0x20, 0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc,
        0xf3, 0x48, 0xab,
    ]);
    if add {
        emit_static_image_credential_stack(&mut code, false)?;
    } else {
        emit_static_image_execfn_stack(&mut code, false)?;
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "credentials-add-ah",
                "credentials-add-ax",
                "credentials-add-ar",
                "credentials-add-aw",
            ]
        } else {
            [
                "credentials-remove-ah",
                "credentials-remove-ax",
                "credentials-remove-ar",
                "credentials-remove-aw",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "credentials-add-apd"
        } else {
            "credentials-remove-apd"
        },
        if add {
            "credentials-add-apd-e"
        } else {
            "credentials-remove-apd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_credential_stack(&mut code, true)?;
    } else {
        emit_static_image_execfn_stack(&mut code, true)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_credentials() -> Result<Vec<u8>, CompileError> {
    static_image_stack_credentials(true)
}

fn static_image_remove_stack_credentials() -> Result<Vec<u8>, CompileError> {
    static_image_stack_credentials(false)
}

fn emit_static_image_secure_mode_stack(
    code: &mut X86FunctionEncoder,
    verify: bool,
) -> Result<(), CompileError> {
    const O: u32 = 0x0ef0;
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
        (O + 136, 11),
        (O + 152, 12),
        (O + 168, 13),
        (O + 184, 14),
        (O + 200, 23),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
    ];
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    if verify {
        code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
        code.bytes(&478_u32.to_le_bytes());
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
        for offset in [
            O + 24,
            O + 32,
            O + 144,
            O + 160,
            O + 176,
            O + 192,
            O + 208,
            O + 216,
            O + 224,
            O + 232,
            0x0ff0,
            0x0ff8,
        ] {
            code.bytes(&[0x49, 0x83, 0xbb]);
            code.bytes(&offset.to_le_bytes());
            code.bytes(&[0]);
            code.jump_if(0x85, "fail");
        }
    }
    Ok(())
}

fn static_image_stack_secure_mode(add: bool) -> Result<Vec<u8>, CompileError> {
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
                "secure-add-h",
                "secure-add-x",
                "secure-add-r",
                "secure-add-w",
            ]
        } else {
            [
                "secure-remove-h",
                "secure-remove-x",
                "secure-remove-r",
                "secure-remove-w",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "secure-add-pd"
        } else {
            "secure-remove-pd"
        },
        if add {
            "secure-add-pd-e"
        } else {
            "secure-remove-pd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_credential_stack(&mut code, true)?;
    } else {
        emit_static_image_secure_mode_stack(&mut code, true)?;
    }
    code.bytes(&[
        0x4c, 0x8b, 0x5c, 0x24, 0x20, 0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc,
        0xf3, 0x48, 0xab,
    ]);
    if add {
        emit_static_image_secure_mode_stack(&mut code, false)?;
    } else {
        emit_static_image_credential_stack(&mut code, false)?;
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "secure-add-ah",
                "secure-add-ax",
                "secure-add-ar",
                "secure-add-aw",
            ]
        } else {
            [
                "secure-remove-ah",
                "secure-remove-ax",
                "secure-remove-ar",
                "secure-remove-aw",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "secure-add-apd"
        } else {
            "secure-remove-apd"
        },
        if add {
            "secure-add-apd-e"
        } else {
            "secure-remove-apd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_secure_mode_stack(&mut code, true)?;
    } else {
        emit_static_image_credential_stack(&mut code, true)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_secure_mode() -> Result<Vec<u8>, CompileError> {
    static_image_stack_secure_mode(true)
}

fn static_image_remove_stack_secure_mode() -> Result<Vec<u8>, CompileError> {
    static_image_stack_secure_mode(false)
}

fn emit_static_image_clock_tick_rate_stack(
    code: &mut X86FunctionEncoder,
    verify: bool,
) -> Result<(), CompileError> {
    const O: u32 = 0x0ee0;
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
        (O + 136, 11),
        (O + 152, 12),
        (O + 168, 13),
        (O + 184, 14),
        (O + 200, 23),
        (O + 216, 17),
        (O + 224, 100),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
    ];
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    if verify {
        code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
        code.bytes(&476_u32.to_le_bytes());
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
        for offset in [
            O + 24,
            O + 32,
            O + 144,
            O + 160,
            O + 176,
            O + 192,
            O + 208,
            O + 232,
            O + 240,
            O + 248,
            0x0ff0,
            0x0ff8,
        ] {
            code.bytes(&[0x49, 0x83, 0xbb]);
            code.bytes(&offset.to_le_bytes());
            code.bytes(&[0]);
            code.jump_if(0x85, "fail");
        }
    }
    Ok(())
}

fn static_image_stack_clock_tick_rate(add: bool) -> Result<Vec<u8>, CompileError> {
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
                "clktck-add-h",
                "clktck-add-x",
                "clktck-add-r",
                "clktck-add-w",
            ]
        } else {
            [
                "clktck-remove-h",
                "clktck-remove-x",
                "clktck-remove-r",
                "clktck-remove-w",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "clktck-add-pd"
        } else {
            "clktck-remove-pd"
        },
        if add {
            "clktck-add-pd-e"
        } else {
            "clktck-remove-pd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_secure_mode_stack(&mut code, true)?;
    } else {
        emit_static_image_clock_tick_rate_stack(&mut code, true)?;
    }
    code.bytes(&[
        0x4c, 0x8b, 0x5c, 0x24, 0x20, 0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9, 0x00, 0x02, 0, 0, 0xfc,
        0xf3, 0x48, 0xab,
    ]);
    if add {
        emit_static_image_clock_tick_rate_stack(&mut code, false)?;
    } else {
        emit_static_image_secure_mode_stack(&mut code, false)?;
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "clktck-add-ah",
                "clktck-add-ax",
                "clktck-add-ar",
                "clktck-add-aw",
            ]
        } else {
            [
                "clktck-remove-ah",
                "clktck-remove-ax",
                "clktck-remove-ar",
                "clktck-remove-aw",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "clktck-add-apd"
        } else {
            "clktck-remove-apd"
        },
        if add {
            "clktck-add-apd-e"
        } else {
            "clktck-remove-apd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_clock_tick_rate_stack(&mut code, true)?;
    } else {
        emit_static_image_secure_mode_stack(&mut code, true)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_clock_tick_rate() -> Result<Vec<u8>, CompileError> {
    static_image_stack_clock_tick_rate(true)
}

fn static_image_remove_stack_clock_tick_rate() -> Result<Vec<u8>, CompileError> {
    static_image_stack_clock_tick_rate(false)
}
