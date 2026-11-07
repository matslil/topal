fn emit_static_image_platform_stack(
    code: &mut X86FunctionEncoder,
    verify: bool,
) -> Result<(), CompileError> {
    const O: u32 = 0x0ed0;
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
        (O + 232, 15),
        (O + 240, 0x007f_fff0),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
        (0x0ff0, u64::from_le_bytes(*b"x86_64\0\0")),
    ];
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    if verify {
        code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
        code.bytes(&474_u32.to_le_bytes());
        code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
        code.jump_if(0x85, "fail");
    }
    for (offset, value) in words {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        if verify {
            code.bytes(&[0x49, 0x39, 0x83]);
        } else {
            code.bytes(&[0x49, 0x89, 0x83]);
        }
        code.bytes(&offset.to_le_bytes());
        if verify {
            code.jump_if(0x85, "fail");
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
            O + 248,
            O + 256,
            O + 264,
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

fn static_image_stack_platform(add: bool) -> Result<Vec<u8>, CompileError> {
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
                "platform-add-h",
                "platform-add-x",
                "platform-add-r",
                "platform-add-w",
            ]
        } else {
            [
                "platform-remove-h",
                "platform-remove-x",
                "platform-remove-r",
                "platform-remove-w",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "platform-add-pd"
        } else {
            "platform-remove-pd"
        },
        if add {
            "platform-add-pd-e"
        } else {
            "platform-remove-pd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_clock_tick_rate_stack(&mut code, true)?;
    } else {
        emit_static_image_platform_stack(&mut code, true)?;
    }
    code.bytes(&[
        0x4c, 0x8b, 0x5c, 0x24, 0x20, 0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9, 0, 2, 0, 0, 0xfc, 0xf3,
        0x48, 0xab,
    ]);
    if add {
        emit_static_image_platform_stack(&mut code, false)?;
    } else {
        emit_static_image_clock_tick_rate_stack(&mut code, false)?;
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "platform-add-ah",
                "platform-add-ax",
                "platform-add-ar",
                "platform-add-aw",
            ]
        } else {
            [
                "platform-remove-ah",
                "platform-remove-ax",
                "platform-remove-ar",
                "platform-remove-aw",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "platform-add-apd"
        } else {
            "platform-remove-apd"
        },
        if add {
            "platform-add-apd-e"
        } else {
            "platform-remove-apd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_platform_stack(&mut code, true)?;
    } else {
        emit_static_image_clock_tick_rate_stack(&mut code, true)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_platform() -> Result<Vec<u8>, CompileError> {
    static_image_stack_platform(true)
}

fn static_image_remove_stack_platform() -> Result<Vec<u8>, CompileError> {
    static_image_stack_platform(false)
}

fn emit_static_image_hardware_capabilities_stack(
    code: &mut X86FunctionEncoder,
    verify: bool,
) -> Result<(), CompileError> {
    const O: u32 = 0x0eb0;
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
        (O + 232, 15),
        (O + 240, 0x007f_fff0),
        (O + 248, 16),
        (O + 256, 0x178b_fbfd),
        (O + 264, 26),
        (O + 272, 0),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
        (0x0ff0, u64::from_le_bytes(*b"x86_64\0\0")),
    ];
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    if verify {
        code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
        code.bytes(&470_u32.to_le_bytes());
        code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
        code.jump_if(0x85, "fail");
    }
    for (offset, value) in words {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        if verify {
            code.bytes(&[0x49, 0x39, 0x83]);
        } else {
            code.bytes(&[0x49, 0x89, 0x83]);
        }
        code.bytes(&offset.to_le_bytes());
        if verify {
            code.jump_if(0x85, "fail");
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
            O + 280,
            O + 288,
            O + 296,
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

fn static_image_stack_hardware_capabilities(add: bool) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    emit_static_image_address_space_resource_validation(&mut code, true)?;
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            ["hwcap-add-h", "hwcap-add-x", "hwcap-add-r", "hwcap-add-w"]
        } else {
            [
                "hwcap-remove-h",
                "hwcap-remove-x",
                "hwcap-remove-r",
                "hwcap-remove-w",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "hwcap-add-pd"
        } else {
            "hwcap-remove-pd"
        },
        if add {
            "hwcap-add-pd-e"
        } else {
            "hwcap-remove-pd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_platform_stack(&mut code, true)?;
    } else {
        emit_static_image_hardware_capabilities_stack(&mut code, true)?;
    }
    code.bytes(&[
        0x4c, 0x8b, 0x5c, 0x24, 0x20, 0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9, 0, 2, 0, 0, 0xfc, 0xf3,
        0x48, 0xab,
    ]);
    if add {
        emit_static_image_hardware_capabilities_stack(&mut code, false)?;
    } else {
        emit_static_image_platform_stack(&mut code, false)?;
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "hwcap-add-ah",
                "hwcap-add-ax",
                "hwcap-add-ar",
                "hwcap-add-aw",
            ]
        } else {
            [
                "hwcap-remove-ah",
                "hwcap-remove-ax",
                "hwcap-remove-ar",
                "hwcap-remove-aw",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "hwcap-add-apd"
        } else {
            "hwcap-remove-apd"
        },
        if add {
            "hwcap-add-apd-e"
        } else {
            "hwcap-remove-apd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_hardware_capabilities_stack(&mut code, true)?;
    } else {
        emit_static_image_platform_stack(&mut code, true)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_hardware_capabilities() -> Result<Vec<u8>, CompileError> {
    static_image_stack_hardware_capabilities(true)
}

fn static_image_remove_stack_hardware_capabilities() -> Result<Vec<u8>, CompileError> {
    static_image_stack_hardware_capabilities(false)
}

fn emit_static_image_load_metadata_stack(
    code: &mut X86FunctionEncoder,
    verify: bool,
) -> Result<(), CompileError> {
    const O: u32 = 0x0e90;
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
        (O + 232, 15),
        (O + 240, 0x007f_fff0),
        (O + 248, 16),
        (O + 256, 0x178b_fbfd),
        (O + 264, 26),
        (O + 272, 0),
        (O + 280, 7),
        (O + 288, 0),
        (O + 296, 8),
        (O + 304, 0),
        (0x0fe0, 0x0078_6f62_7973_7562),
        (0x0fe8, 0x0000_0000_0000_6873),
        (0x0ff0, u64::from_le_bytes(*b"x86_64\0\0")),
    ];
    code.bytes(&[0x4c, 0x8b, 0x5c, 0x24, 0x20]);
    if verify {
        code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9]);
        code.bytes(&466_u32.to_le_bytes());
        code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
        code.jump_if(0x85, "fail");
    }
    for (offset, value) in words {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        if verify {
            code.bytes(&[0x49, 0x39, 0x83]);
        } else {
            code.bytes(&[0x49, 0x89, 0x83]);
        }
        code.bytes(&offset.to_le_bytes());
        if verify {
            code.jump_if(0x85, "fail");
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
            O + 312,
            O + 320,
            O + 328,
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

fn static_image_stack_load_metadata(add: bool) -> Result<Vec<u8>, CompileError> {
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
                "load-metadata-add-h",
                "load-metadata-add-x",
                "load-metadata-add-r",
                "load-metadata-add-w",
            ]
        } else {
            [
                "load-metadata-remove-h",
                "load-metadata-remove-x",
                "load-metadata-remove-r",
                "load-metadata-remove-w",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "load-metadata-add-pd"
        } else {
            "load-metadata-remove-pd"
        },
        if add {
            "load-metadata-add-pd-e"
        } else {
            "load-metadata-remove-pd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_hardware_capabilities_stack(&mut code, true)?;
    } else {
        emit_static_image_load_metadata_stack(&mut code, true)?;
    }
    code.bytes(&[
        0x4c, 0x8b, 0x5c, 0x24, 0x20, 0x4c, 0x89, 0xdf, 0x31, 0xc0, 0xb9, 0, 2, 0, 0, 0xfc, 0xf3,
        0x48, 0xab,
    ]);
    if add {
        emit_static_image_load_metadata_stack(&mut code, false)?;
    } else {
        emit_static_image_hardware_capabilities_stack(&mut code, false)?;
    }
    code.bytes(&[0x0f, 0xae, 0xf8]);
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if add {
            [
                "load-metadata-add-ah",
                "load-metadata-add-ax",
                "load-metadata-add-ar",
                "load-metadata-add-aw",
            ]
        } else {
            [
                "load-metadata-remove-ah",
                "load-metadata-remove-ax",
                "load-metadata-remove-ar",
                "load-metadata-remove-aw",
            ]
        },
        !add,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if add {
            "load-metadata-add-apd"
        } else {
            "load-metadata-remove-apd"
        },
        if add {
            "load-metadata-add-apd-e"
        } else {
            "load-metadata-remove-apd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    if add {
        emit_static_image_load_metadata_stack(&mut code, true)?;
    } else {
        emit_static_image_hardware_capabilities_stack(&mut code, true)?;
    }
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_add_stack_load_metadata() -> Result<Vec<u8>, CompileError> {
    static_image_stack_load_metadata(true)
}

fn static_image_remove_stack_load_metadata() -> Result<Vec<u8>, CompileError> {
    static_image_stack_load_metadata(false)
}
