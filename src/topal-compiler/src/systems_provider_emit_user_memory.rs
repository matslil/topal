fn user_prepare_initial_address_space() -> Result<Vec<u8>, CompileError> {
    let mut probe = initial_user_probe()?;
    let static_user_image = qualified_x86_64_static_user_image_fixture();
    let expected_boot_resource =
        generate_initial_boot_resource(&initial_user_elf_file()?, &static_user_image)?;
    probe.resize(probe.len().next_multiple_of(8), 0x90);
    if probe.len() > 4096 {
        return Err(CompileError::Tool(
            "initial user probe exceeds its sealed executable page".into(),
        ));
    }
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xd2]); // allocator-owned backing base
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc2, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0xd0, 0xff, 0x3f]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x49, 0x89, 0xd2]); // retain backing identity in r10
    code.bytes(&[0x80, 0x3f, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // retained Linux boot_params
    code.jump_if(0x84, "fail");
    code.bytes(&[0x83, 0xbe, 0xc0, 0x00, 0x00, 0x00, 0x00]); // ext_ramdisk_image
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xbe, 0xc4, 0x00, 0x00, 0x00, 0x00]); // ext_ramdisk_size
    code.jump_if(0x85, "fail");
    code.bytes(&[0x44, 0x8b, 0x86, 0x18, 0x02, 0x00, 0x00]); // ramdisk_image
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x8b, 0x8e, 0x1c, 0x02, 0x00, 0x00]); // ramdisk_size
    code.bytes(&[0x81, 0xf9]);
    code.bytes(
        &u32::try_from(expected_boot_resource.len())
            .map_err(|_| CompileError::Tool("initial boot resource exceeds u32".into()))?
            .to_le_bytes(),
    );
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4c, 0x89, 0xc0, 0x48, 0x01, 0xc8]); // resource end
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x00, 0x00, 0x40]); // within initial identity map
    code.jump_if(0x87, "fail");
    let image_end =
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + initial_user_elf_file()?.len();
    let static_payload_offset = X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET as usize;
    let static_metadata_end = static_payload_offset + 64 + 11 * 56;
    let static_image_end = static_payload_offset + static_user_image.len();
    let metadata_offsets = (0..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize)
        .chain(
            X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + 256,
        )
        .chain(image_end..static_payload_offset)
        .chain(static_payload_offset..static_metadata_end)
        .chain(static_image_end..expected_boot_resource.len())
        .filter(|offset| {
            let code_size_fields = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                    + INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET
                    + 8;
            let entry_field = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_ENTRY_ELF_OFFSET
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                    + INITIAL_USER_ENTRY_ELF_OFFSET
                    + 8;
            let data_size_fields = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                    + INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET
                    + 8;
            let static_section_offset = static_payload_offset + 40..static_payload_offset + 48;
            let static_section_shape = static_payload_offset + 58..static_payload_offset + 64;
            !entry_field.contains(offset)
                && !code_size_fields.contains(offset)
                && !data_size_fields.contains(offset)
                && !static_section_offset.contains(offset)
                && !static_section_shape.contains(offset)
        });
    for offset in metadata_offsets {
        let expected = expected_boot_resource[offset];
        code.bytes(&[0x41, 0x80, 0xb8]); // cmp byte ptr [r8 + displacement], immediate
        code.bytes(
            &u32::try_from(offset)
                .map_err(|_| CompileError::Tool("boot resource offset exceeds u32".into()))?
                .to_le_bytes(),
        );
        code.bytes(&[expected]);
        code.jump_if(0x85, "fail");
    }
    let static_section_offset = u32::try_from(static_payload_offset + 40)
        .map_err(|_| CompileError::Tool("static section offset exceeds u32".into()))?;
    let static_section_shape = u32::try_from(static_payload_offset + 58)
        .map_err(|_| CompileError::Tool("static section shape exceeds u32".into()))?;
    code.bytes(&[0x49, 0x8b, 0x80]);
    code.bytes(&static_section_offset.to_le_bytes());
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x84, "static-resource-clean-section-shape");
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x001e_dea0_u32.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0001_0016_0017_0040_u64.to_le_bytes());
    code.jump("static-resource-section-shape-ready");
    code.bind("static-resource-clean-section-shape")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0001_0000_0000_0000_u64.to_le_bytes());
    code.bind("static-resource-section-shape-ready")?;
    code.bytes(&[0x49, 0x39, 0x80]);
    code.bytes(&static_section_shape.to_le_bytes());
    code.jump_if(0x85, "fail");
    let code_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external code-size offset exceeds u32".into()))?;
    let code_memory_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
            + INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external memory-size offset exceeds u32".into()))?;
    code.bytes(&[0x49, 0x8b, 0x80]); // executable p_filesz
    code.bytes(&code_file_size_offset.to_le_bytes());
    code.bytes(&[0x49, 0x3b, 0x80]); // p_filesz == p_memsz
    code.bytes(&code_memory_size_offset.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x3d, 0x08, 0x11, 0x00, 0x00]); // includes second-page entry qword
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x20, 0x00, 0x00]); // two base pages
    code.jump_if(0x87, "fail");
    code.bytes(&[0xa8, 0x07]); // qword complete
    code.jump_if(0x85, "fail");
    let entry_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_ENTRY_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external entry offset exceeds u32".into()))?;
    code.bytes(&[0x4d, 0x8b, 0x98]); // ELF e_entry
    code.bytes(&entry_offset.to_le_bytes());
    code.bytes(&[0x49, 0x81, 0xfb]);
    code.bytes(&(INITIAL_USER_CODE_ADDRESS as u32 + 4096 + 256).to_le_bytes());
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x05]);
    code.bytes(&(INITIAL_USER_CODE_ADDRESS as u32).to_le_bytes());
    code.bytes(&[0x49, 0x39, 0xc3]);
    code.jump_if(0x83, "fail");
    let data_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external data-size offset exceeds u32".into()))?;
    let data_memory_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
            + INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external BSS-size offset exceeds u32".into()))?;
    code.bytes(&[0x49, 0x8b, 0x80]); // data p_filesz
    code.bytes(&data_file_size_offset.to_le_bytes());
    code.bytes(&[0x48, 0x83, 0xf8, 0x08]); // at least one initialized qword
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x10, 0x00, 0x00]); // one file page
    code.jump_if(0x87, "fail");
    code.bytes(&[0xa8, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x90]); // data p_memsz
    code.bytes(&data_memory_size_offset.to_le_bytes());
    code.bytes(&[0x48, 0x39, 0xd0]); // p_filesz <= p_memsz
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0xf6, 0xc2, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0xf9]); // retain state in r9
    code.bytes(&[0x4c, 0x89, 0x57, 0x68]); // retain backing identity
    code.bytes(&[0x4c, 0x89, 0x9f, 0x88, 0x00, 0x00, 0x00]); // selected entry
    code.bytes(&[0x4d, 0x89, 0xd0]); // publish through supervisor identity mapping
    for (index, chunk) in probe.chunks_exact(8).enumerate() {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(chunk);
        let displacement = u32::try_from(index * 8)
            .map_err(|_| CompileError::Tool("user probe displacement exceeds u32".into()))?;
        code.bytes(&[0x49, 0x89, 0x80]);
        code.bytes(&displacement.to_le_bytes());
    }
    for (index, chunk) in initial_user_stack_bytes().chunks_exact(8).enumerate() {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(chunk);
        let displacement = u32::try_from(0x1eb0 + index * 8).map_err(|_| {
            CompileError::Tool("initial user stack displacement exceeds u32".into())
        })?;
        code.bytes(&[0x49, 0x89, 0x82]);
        code.bytes(&displacement.to_le_bytes());
    }
    code.bytes(&[0x4d, 0x8b, 0x99, 0x88, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4d, 0x89, 0x9a, 0xe8, 0x1e, 0x00, 0x00]); // runtime AT_ENTRY
    code.bytes(&[0x4d, 0x8d, 0x9a, 0x00, 0x20, 0x00, 0x00]); // leaf table
    code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x02, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab]); // clear frame
    code.bytes(&[0x4c, 0x89, 0xcf]); // restore state
    code.bytes(&[0x4c, 0x89, 0xd2, 0x48, 0x83, 0xca, 0x05]); // RX code leaf
    code.bytes(&[0x49, 0x89, 0x13]);
    code.bytes(&[0x49, 0x8d, 0x92, 0x00, 0x10, 0x00, 0x00]); // stack frame
    code.bytes(&[0x48, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x48, 0x09, 0xca, 0x49, 0x89, 0x53, 0x08]); // RW/NX stack leaf
    code.bytes(&[0x0f, 0x20, 0xd8]); // active root from CR3
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4d, 0x89, 0xda, 0x49, 0x83, 0xca, 0x07]);
    code.bytes(&[0x4c, 0x89, 0x90, 0x18, 0x20, 0x00, 0x00]); // publish user PT
    code.bytes(&[0x0f, 0xae, 0xf0]); // publish before invalidation
    for address in [0x0060_0000_u64, 0x0060_1000, 0x0060_2000] {
        code.bytes(&[0x49, 0xb8]);
        code.bytes(&address.to_le_bytes());
        code.bytes(&[0x41, 0x0f, 0x01, 0x38]); // invlpg [r8]
    }

    code.bytes(&[0xc6, 0x87, 0xc0, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0xc6, 0x87, 0xd0, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xd8, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[
        0x48, 0xc7, 0x87, 0xe0, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x00,
    ]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xe8, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x01]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_verify_initial_mapping() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc2, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0x3f, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0x89, 0xd0]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x84, "code-extent-end");
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x10, 0x00, 0x00]);
    code.jump("extent-end-ready");
    code.bind("code-extent-end")?;
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x20, 0x00, 0x00]);
    code.bind("extent-end-ready")?;
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x89, 0xc9]);
    code.bytes(&[0x49, 0x81, 0xc1, 0x00, 0x30, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x39, 0xc8]);
    code.jump_if(0x86, "disjoint");
    code.bytes(&[0x4c, 0x39, 0xca]);
    code.jump_if(0x83, "disjoint");
    code.jump("fail");

    code.bind("disjoint")?;
    code.bytes(&[0x48, 0x81, 0xc1, 0x00, 0x20, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0x79, 0x10, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0xd0]);
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x4d, 0x09, 0xc8, 0x4c, 0x89, 0x41, 0x10]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_2000_u64.to_le_bytes());
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x41, 0xc6, 0x02, 0x6d]);
    code.bytes(&[0x41, 0x80, 0x3a, 0x6d]);
    code.jump_if(0x85, "cleanup-fail");
    code.bytes(&[0x48, 0xc7, 0x41, 0x10, 0, 0, 0, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0, 0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x0b]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);

    code.bind("cleanup-fail")?;
    code.bytes(&[0x48, 0xc7, 0x41, 0x10, 0, 0, 0, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0, 0x41, 0x0f, 0x01, 0x3a]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_map_initial_frame() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc2, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0x3f, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x84, "generation-one");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x0e]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 0x11, 0x00, 0x00, 0x00]);
    code.jump("generation-accepted");
    code.bind("generation-one")?;
    code.bytes(&[0xb8, 0x04, 0x00, 0x00, 0x00]);
    code.bind("generation-accepted")?;
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x84, "check-generation-one-frame");
    code.bytes(&[0x48, 0x83, 0xbf, 0x80, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x3b, 0x57, 0x78]);
    code.jump_if(0x84, "fail");
    code.jump("frame-slot-clear");
    code.bind("check-generation-one-frame")?;
    code.bytes(&[0x48, 0x83, 0x7f, 0x78, 0x00]);
    code.jump_if(0x85, "fail");
    code.bind("frame-slot-clear")?;
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0x89, 0xd0]);
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x89, 0xc9]);
    code.bytes(&[0x49, 0x81, 0xc1, 0x00, 0x30, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x39, 0xc8]);
    code.jump_if(0x86, "disjoint");
    code.bytes(&[0x4c, 0x39, 0xca]);
    code.jump_if(0x83, "disjoint");
    code.jump("fail");

    code.bind("disjoint")?;
    code.bytes(&[0x48, 0x81, 0xc1, 0x00, 0x20, 0x00, 0x00]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x84, "pte-slot-selected");
    code.bytes(&[0x48, 0x83, 0xc1, 0x10]);
    code.bind("pte-slot-selected")?;
    code.bytes(&[0x48, 0x83, 0x79, 0x18, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x85, "leaf-slots-clear");
    code.bytes(&[0x48, 0x83, 0x79, 0x20, 0x00]);
    code.jump_if(0x85, "fail");
    code.bind("leaf-slots-clear")?;
    code.bytes(&[0x49, 0x89, 0xd0]);
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x4d, 0x09, 0xc8, 0x4c, 0x89, 0x41, 0x18]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x85, "leaves-published");
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x89, 0x41, 0x20]);
    code.bind("leaves-published")?;
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x84, "publish-generation-one");
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_5000_u64.to_le_bytes());
    code.bytes(&[0x48, 0x89, 0x97, 0x80, 0x00, 0x00, 0x00]);
    code.jump("publish-mapping");
    code.bind("publish-generation-one")?;
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_3000_u64.to_le_bytes());
    code.bytes(&[0x48, 0x89, 0x57, 0x78]);
    code.bind("publish-mapping")?;
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x85, "mapping-invalidated");
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bind("mapping-invalidated")?;
    code.bytes(&[0x88, 0x87, 0x70, 0x00, 0x00, 0x00]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);

    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_store_initial_mapping_byte() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x11]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // retained Linux boot_params
    code.jump_if(0x84, "fail");
    emit_initial_boot_resource_handoff_validation(&mut code)?;
    let data_payload_offset = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET
        .checked_add(
            u32::try_from(INITIAL_USER_DATA_FILE_OFFSET)
                .map_err(|_| CompileError::Tool("external user data offset exceeds u32".into()))?,
        )
        .ok_or_else(|| CompileError::Tool("external user data offset overflow".into()))?;
    let data_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external data-size offset exceeds u32".into()))?;
    let data_memory_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
            + INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external BSS-size offset exceeds u32".into()))?;
    code.bytes(&[0x49, 0x8b, 0x88]); // runtime p_filesz
    code.bytes(&data_file_size_offset.to_le_bytes());
    code.bytes(&[0x4d, 0x8b, 0x98]); // runtime p_memsz
    code.bytes(&data_memory_size_offset.to_le_bytes());
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    code.bytes(&[0x4c, 0x01, 0xd8]); // initial break = data base + p_memsz
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x89, 0x87, 0x90, 0x00, 0x00, 0x00]); // current break
    code.bytes(&[0x48, 0x89, 0x87, 0x98, 0x00, 0x00, 0x00]); // initial break
    code.bytes(&[0x41, 0xbb, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0x49, 0x29, 0xcb]); // zero bytes = page size - p_filesz
    code.bytes(&[0x49, 0x81, 0xc0]);
    code.bytes(&data_payload_offset.to_le_bytes());
    code.bytes(&[0x49, 0xba]);
    code.bytes(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    code.bind("data-copy-loop")?;
    code.bytes(&[0x49, 0x8b, 0x00]);
    code.bytes(&[0x49, 0x89, 0x02]);
    code.bytes(&[0x49, 0x83, 0xc0, 0x08]);
    code.bytes(&[0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x48, 0x83, 0xe9, 0x08]);
    code.jump_if(0x85, "data-copy-loop");
    code.bytes(&[0x4d, 0x85, 0xdb]);
    code.jump_if(0x84, "data-complete");
    code.bind("bss-zero-loop")?;
    code.bytes(&[0x49, 0xc7, 0x02, 0, 0, 0, 0]);
    code.bytes(&[0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x49, 0x83, 0xeb, 0x08]);
    code.jump_if(0x85, "bss-zero-loop");
    code.bind("data-complete")?;
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x12]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

struct InitialUserEntryPayload {
    bytes: Vec<u8>,
    continuation_address: u64,
    uname_return_address: u64,
    nanosleep_return_address: u64,
}

fn initial_user_stack_bytes() -> [u8; 336] {
    let mut bytes = [0_u8; 336];
    bytes[0..8].copy_from_slice(&1_u64.to_le_bytes());
    bytes[8..16].copy_from_slice(&0x0060_1fc8_u64.to_le_bytes());
    bytes[32..40].copy_from_slice(&6_u64.to_le_bytes());
    bytes[40..48].copy_from_slice(&4096_u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&9_u64.to_le_bytes());
    bytes[56..64].copy_from_slice(&0_u64.to_le_bytes());
    bytes[64..72].copy_from_slice(&31_u64.to_le_bytes());
    bytes[72..80].copy_from_slice(&0x0060_1fc8_u64.to_le_bytes());
    for (offset, tag) in [(80, 11_u64), (96, 12), (112, 13), (128, 14)] {
        bytes[offset..offset + 8].copy_from_slice(&tag.to_le_bytes());
    }
    bytes[144..152].copy_from_slice(&3_u64.to_le_bytes());
    bytes[152..160].copy_from_slice(&INITIAL_USER_PROGRAM_HEADER_ADDRESS.to_le_bytes());
    bytes[160..168].copy_from_slice(&4_u64.to_le_bytes());
    bytes[168..176].copy_from_slice(&56_u64.to_le_bytes());
    bytes[176..184].copy_from_slice(&5_u64.to_le_bytes());
    bytes[184..192].copy_from_slice(&3_u64.to_le_bytes());
    bytes[192..200].copy_from_slice(&23_u64.to_le_bytes());
    bytes[208..216].copy_from_slice(&17_u64.to_le_bytes());
    bytes[216..224].copy_from_slice(&100_u64.to_le_bytes());
    bytes[224..232].copy_from_slice(&15_u64.to_le_bytes());
    bytes[232..240].copy_from_slice(&0x0060_1fc0_u64.to_le_bytes());
    bytes[240..248].copy_from_slice(&25_u64.to_le_bytes());
    bytes[248..256].copy_from_slice(&0x0060_1fe0_u64.to_le_bytes());
    bytes[272..279].copy_from_slice(b"x86_64\0");
    bytes[280..291].copy_from_slice(b"topal-probe");
    bytes
}

fn initial_user_elf_prefix(image_size: u64) -> [u8; 256] {
    let mut bytes = [0_u8; 256];
    bytes[0..16].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    bytes[16..18].copy_from_slice(&2_u16.to_le_bytes()); // ET_EXEC
    bytes[18..20].copy_from_slice(&62_u16.to_le_bytes()); // EM_X86_64
    bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
    bytes[24..32].copy_from_slice(&INITIAL_USER_ENTRY_ADDRESS.to_le_bytes());
    bytes[32..40].copy_from_slice(&64_u64.to_le_bytes());
    bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
    bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
    bytes[56..58].copy_from_slice(&3_u16.to_le_bytes());
    bytes[58..60].copy_from_slice(&64_u16.to_le_bytes());

    bytes[64..68].copy_from_slice(&1_u32.to_le_bytes()); // PT_LOAD
    bytes[68..72].copy_from_slice(&5_u32.to_le_bytes()); // PF_R | PF_X
    bytes[80..88].copy_from_slice(&INITIAL_USER_CODE_ADDRESS.to_le_bytes());
    bytes[88..96].copy_from_slice(&INITIAL_USER_CODE_ADDRESS.to_le_bytes());
    bytes[96..104].copy_from_slice(&image_size.to_le_bytes());
    bytes[104..112].copy_from_slice(&image_size.to_le_bytes());
    bytes[112..120].copy_from_slice(&4096_u64.to_le_bytes());

    bytes[120..124].copy_from_slice(&1_u32.to_le_bytes()); // PT_LOAD data
    bytes[124..128].copy_from_slice(&6_u32.to_le_bytes()); // PF_R | PF_W
    bytes[128..136].copy_from_slice(&(INITIAL_USER_DATA_FILE_OFFSET as u64).to_le_bytes());
    bytes[136..144].copy_from_slice(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    bytes[144..152].copy_from_slice(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    bytes[152..160].copy_from_slice(&8_u64.to_le_bytes());
    bytes[160..168].copy_from_slice(&16_u64.to_le_bytes());
    bytes[168..176].copy_from_slice(&4096_u64.to_le_bytes());

    bytes[176..180].copy_from_slice(&0x6474_e551_u32.to_le_bytes()); // PT_GNU_STACK
    bytes[180..184].copy_from_slice(&6_u32.to_le_bytes()); // PF_R | PF_W
    bytes[224..232].copy_from_slice(&16_u64.to_le_bytes());
    bytes
}

pub(crate) fn initial_user_code_image() -> Result<Vec<u8>, CompileError> {
    let payload = initial_user_entry_payload()?;
    let image_size = (4096_u64 + 256)
        .checked_add(u64::try_from(payload.bytes.len()).unwrap_or(u64::MAX))
        .ok_or_else(|| CompileError::Tool("constructed ELF image size overflow".into()))?;
    let mut bytes = initial_user_elf_prefix(image_size).to_vec();
    bytes.resize(4096 + 256, 0x90);
    bytes.extend(payload.bytes);
    if bytes.len() > INITIAL_USER_CODE_CAPACITY {
        return Err(CompileError::Tool(
            "constructed ELF image exceeds its sealed two-page executable extent".into(),
        ));
    }
    Ok(bytes)
}

pub(crate) fn initial_user_elf_file() -> Result<Vec<u8>, CompileError> {
    let mut bytes = initial_user_code_image()?;
    bytes.resize(
        INITIAL_USER_DATA_FILE_OFFSET + INITIAL_USER_DATA_CAPACITY,
        0,
    );
    bytes[INITIAL_USER_DATA_FILE_OFFSET] = 0x6d;
    Ok(bytes)
}
