fn static_image_stack_random_slot_authority(authorize: bool) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    emit_static_image_address_space_resource_validation(&mut code, true)?;
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if authorize {
            [
                "random-slot-authorize-h",
                "random-slot-authorize-x",
                "random-slot-authorize-r",
                "random-slot-authorize-w",
            ]
        } else {
            [
                "random-slot-revoke-h",
                "random-slot-revoke-x",
                "random-slot-revoke-r",
                "random-slot-revoke-w",
            ]
        },
        !authorize,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if authorize {
            "random-slot-authorize-pd"
        } else {
            "random-slot-revoke-pd"
        },
        if authorize {
            "random-slot-authorize-pd-e"
        } else {
            "random-slot-revoke-pd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    emit_static_image_load_metadata_stack(&mut code, true)?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_authorize_stack_random_slot() -> Result<Vec<u8>, CompileError> {
    static_image_stack_random_slot_authority(true)
}

fn static_image_revoke_stack_random_slot() -> Result<Vec<u8>, CompileError> {
    static_image_stack_random_slot_authority(false)
}

fn static_image_prepared_process_retention(retain: bool) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x51, 0x41, 0x50, 0x57, 0x56, 0x51]);
    emit_static_image_address_space_resource_validation(&mut code, true)?;
    code.bytes(&[
        0x48, 0x8b, 0x7c, 0x24, 0x10, 0x48, 0x8b, 0x74, 0x24, 0x08, 0x4c, 0x8b, 0x5c, 0x24, 0x20,
    ]);
    emit_static_image_published_stack_leaf_verification_with_relro(
        &mut code,
        if retain {
            [
                "prepared-retain-h",
                "prepared-retain-x",
                "prepared-retain-r",
                "prepared-retain-w",
            ]
        } else {
            [
                "prepared-restore-h",
                "prepared-restore-x",
                "prepared-restore-r",
                "prepared-restore-w",
            ]
        },
        !retain,
    )?;
    emit_static_image_address_space_hierarchy_verification_labeled(
        &mut code,
        true,
        if retain {
            "prepared-retain-pd"
        } else {
            "prepared-restore-pd"
        },
        if retain {
            "prepared-retain-pd-e"
        } else {
            "prepared-restore-pd-e"
        },
    )?;
    emit_static_image_stack_leaf_root_verification(&mut code)?;
    emit_static_image_stack_leaf_entry_verification(&mut code)?;
    emit_static_image_resident_program_header_verification(&mut code)?;
    emit_static_image_load_metadata_stack(&mut code, true)?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_retain_prepared_process() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_restore_prepared_process() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(false)
}

fn static_image_qualify_retained_disjoint_work() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_authorize_entropy_handoff() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_authorize_entropy_acquisition() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_device() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_transport() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_negotiation() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_queue() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_bus_master() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_dma_buffers() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_split_ring() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_dma_preparation() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_queue_addresses() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_queue_activation() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_request_completion() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_sample_capture() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_sample_quality() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_pool_lifecycle() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_qualify_entropy_generator_lifecycle() -> Result<Vec<u8>, CompileError> {
    static_image_prepared_process_retention(true)
}

fn static_image_deliver_entropy_initial_random() -> Result<Vec<u8>, CompileError> {
    const OBJECT: u32 = 0x0e70;
    const TAG: u32 = 0x0fc8;
    const VALUE: u32 = 0x0fd0;
    const TERMINATOR: u32 = 0x0fd8;
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc2, 0x07, 0, 0, 0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    for register_compare in [
        &[0x48, 0x39, 0xf9][..],
        &[0x48, 0x39, 0xf1][..],
        &[0x48, 0x39, 0xd1][..],
    ] {
        code.bytes(register_compare);
        code.jump_if(0x84, "fail");
    }
    for offset in (168_u32..=272).step_by(8) {
        code.bytes(&[0x48, 0x83, 0xba]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fail");
    }
    for offset in [OBJECT, OBJECT + 8, TAG, VALUE, TERMINATOR] {
        code.bytes(&[0x48, 0x83, 0xbf]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0]);
        code.jump_if(0x85, "fail");
    }
    for (source_offset, target_offset) in [(0_u8, OBJECT), (8, OBJECT + 8)] {
        if source_offset == 0 {
            code.bytes(&[0x48, 0x8b, 0x06]);
        } else {
            code.bytes(&[0x48, 0x8b, 0x46, source_offset]);
        }
        code.bytes(&[0x48, 0x89, 0x87]);
        code.bytes(&target_offset.to_le_bytes());
        code.bytes(&[0x48, 0x3b, 0x87]);
        code.bytes(&target_offset.to_le_bytes());
        code.jump_if(0x85, "fail_wipe");
    }
    for (offset, value) in [(TAG, 25_u64), (VALUE, 0x007f_fe70)] {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x48, 0x89, 0x87]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0x48, 0x39, 0x87]);
        code.bytes(&offset.to_le_bytes());
        code.jump_if(0x85, "fail_wipe");
    }
    for (source_offset, context_offset) in [(16_u8, 168_u32), (24, 176)] {
        code.bytes(&[0x48, 0x8b, 0x46, source_offset]);
        code.bytes(&[0x48, 0x89, 0x82]);
        code.bytes(&context_offset.to_le_bytes());
        code.bytes(&[0x48, 0x3b, 0x82]);
        code.bytes(&context_offset.to_le_bytes());
        code.jump_if(0x85, "fail_context_wipe");
    }
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x8b, 0x01]);
        } else {
            code.bytes(&[0x48, 0x8b, 0x41, offset]);
        }
        code.bytes(&[0x48, 0x89, 0x82]);
        code.bytes(&(192_u32 + u32::from(offset)).to_le_bytes());
        code.bytes(&[0x48, 0x3b, 0x82]);
        code.bytes(&(192_u32 + u32::from(offset)).to_le_bytes());
        code.jump_if(0x85, "fail_context_wipe");
    }
    code.bytes(&[0x48, 0x89, 0x92]);
    code.bytes(&256_u32.to_le_bytes());
    code.bytes(&[0x48, 0x39, 0x92]);
    code.bytes(&256_u32.to_le_bytes());
    code.jump_if(0x85, "fail_context_wipe");
    code.bytes(&[0xc6, 0x82]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[1]);
    code.bytes(&[0x80, 0xba]);
    code.bytes(&184_u32.to_le_bytes());
    code.bytes(&[1]);
    code.jump_if(0x85, "fail_context_wipe");
    code.bytes(&[0x41, 0xb8, 1, 0, 0, 0]);
    code.bind("wipe")?;
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0xc7, 0x06, 0, 0, 0, 0]);
            code.bytes(&[0x48, 0x83, 0x3e, 0]);
        } else {
            code.bytes(&[0x48, 0xc7, 0x46, offset, 0, 0, 0, 0]);
            code.bytes(&[0x48, 0x83, 0x7e, offset, 0]);
        }
        code.jump_if(0x85, "fail");
    }
    code.bytes(&[0x44, 0x89, 0xc0, 0xc3]);
    code.bind("fail_wipe")?;
    code.bytes(&[0x45, 0x31, 0xc0]);
    code.jump("wipe");
    code.bind("fail_context_wipe")?;
    for offset in (168_u32..=272).step_by(8) {
        code.bytes(&[0x48, 0xc7, 0x82]);
        code.bytes(&offset.to_le_bytes());
        code.bytes(&[0, 0, 0, 0]);
    }
    code.bytes(&[0x45, 0x31, 0xc0]);
    code.jump("wipe");
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}
