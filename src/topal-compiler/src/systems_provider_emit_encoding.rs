fn append_translation_functions(
    object: &mut Object<'_>,
    text: object::write::SectionId,
) -> Result<(), CompileError> {
    for (name, encoded) in [
        (
            X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
            translation_backing_selector()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
            translation_space_builder()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
            translation_space_activator()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL,
            translation_edit_begin()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
            translation_edit_map()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
            translation_edit_unmap()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL,
            translation_edit_commit()?,
        ),
    ] {
        append_encoded_function(object, text, name, &encoded);
    }
    Ok(())
}

fn append_function(
    object: &mut Object<'_>,
    section: object::write::SectionId,
    name: &str,
    instructions: &[X86Instruction],
) {
    let mut encoded = Vec::new();
    for instruction in instructions {
        instruction.encode(&mut encoded);
    }
    let offset = object.append_section_data(section, &encoded, 16);
    object.add_symbol(Symbol {
        name: name.as_bytes().to_vec(),
        value: offset,
        size: encoded.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
}

fn append_encoded_function(
    object: &mut Object<'_>,
    section: object::write::SectionId,
    name: &str,
    encoded: &[u8],
) {
    let offset = object.append_section_data(section, encoded, 16);
    object.add_symbol(Symbol {
        name: name.as_bytes().to_vec(),
        value: offset,
        size: encoded.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
}

fn append_linkable_function(
    object: &mut Object<'_>,
    section: object::write::SectionId,
    name: &str,
    encoded: &LinkableX86Function,
) -> Result<(), CompileError> {
    append_encoded_function(object, section, name, &encoded.bytes);
    let start = object
        .symbol(object.symbol_id(name.as_bytes()).unwrap())
        .value;
    for (offset, target) in &encoded.calls {
        let symbol = object.symbol_id(target.as_bytes()).unwrap_or_else(|| {
            object.add_symbol(Symbol {
                name: target.as_bytes().to_vec(),
                value: 0,
                size: 0,
                kind: SymbolKind::Text,
                scope: SymbolScope::Unknown,
                weak: false,
                section: SymbolSection::Undefined,
                flags: SymbolFlags::None,
            })
        });
        object
            .add_relocation(
                section,
                object::write::Relocation {
                    offset: start + offset,
                    symbol,
                    addend: -4,
                    flags: object::RelocationFlags::Generic {
                        kind: object::RelocationKind::PltRelative,
                        encoding: object::RelocationEncoding::X86Branch,
                        size: 32,
                    },
                },
            )
            .map_err(|e| CompileError::Tool(format!("native service call relocation: {e}")))?;
    }
    Ok(())
}

fn emit_initial_boot_resource_handoff_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let expected = generate_initial_boot_resource(
        &initial_user_elf_file()?,
        &qualified_x86_64_static_user_image_fixture(),
    )?;
    let expected_size = u32::try_from(expected.len())
        .map_err(|_| CompileError::Tool("initial boot resource exceeds u32".into()))?;
    code.bytes(&[0x83, 0xbe, 0xc0, 0x00, 0x00, 0x00, 0x00]); // ext_ramdisk_image
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xbe, 0xc4, 0x00, 0x00, 0x00, 0x00]); // ext_ramdisk_size
    code.jump_if(0x85, "fail");
    code.bytes(&[0x44, 0x8b, 0x86, 0x18, 0x02, 0x00, 0x00]); // resource base
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x8b, 0x8e, 0x1c, 0x02, 0x00, 0x00]); // resource size
    code.bytes(&[0x81, 0xf9]);
    code.bytes(&expected_size.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4d, 0x89, 0xc1, 0x49, 0x01, 0xc9]); // resource end
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x00, 0x00, 0x40]); // initial map limit
    code.jump_if(0x87, "fail");
    code.bytes(&[0x41, 0x81, 0x38, 0x30, 0x37, 0x30, 0x37]); // `0707`
    code.jump_if(0x85, "fail");
    code.bytes(&[0x66, 0x41, 0x81, 0x78, 0x04, 0x30, 0x31]); // `01`
    code.jump_if(0x85, "fail");
    Ok(())
}

fn boot_memory_validator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xf6]); // test rsi, rsi
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0xf0, 0xff, 0x00]); // cmp rsi, 0x00fff000
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xbe, 0x50, 0x02, 0x00, 0x00, 0x00]); // setup_data == 0
    code.jump_if(0x85, "fail");
    emit_initial_boot_resource_handoff_validation(&mut code)?;
    code.bytes(&[0x0f, 0xb6, 0x8e, 0xe8, 0x01, 0x00, 0x00]); // e820_entries
    code.bytes(&[0x85, 0xc9]); // test ecx, ecx
    code.jump_if(0x84, "fail");
    code.bytes(&[0x81, 0xf9, 0x80, 0x00, 0x00, 0x00]); // cmp ecx, 128
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x8d, 0x96, 0xd0, 0x02, 0x00, 0x00]); // first E820 entry
    code.bytes(&[0x45, 0x31, 0xc0]); // xor r8d, r8d

    code.bind("loop")?;
    code.bytes(&[0x48, 0x8b, 0x02]); // base
    code.bytes(&[0x4c, 0x8b, 0x4a, 0x08]); // length
    code.bytes(&[0x44, 0x8b, 0x52, 0x10]); // type
    code.bytes(&[0x4d, 0x85, 0xc9]); // test length
    code.jump_if(0x84, "zero-length");
    code.bytes(&[0x49, 0x89, 0xc3]); // end = base
    code.bytes(&[0x4d, 0x01, 0xcb]); // end += length
    code.jump_if(0x82, "fail");
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]); // E820 RAM
    code.jump_if(0x85, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0x00, 0x00, 0x01]); // base >= 16 MiB
    code.jump_if(0x83, "base-ready");
    code.bytes(&[0xb8, 0x00, 0x00, 0x00, 0x01]); // candidate = 16 MiB
    code.bind("base-ready")?;
    code.bytes(&[0x48, 0x05, 0xff, 0x0f, 0x00, 0x00]); // align candidate up
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x81, 0xe3, 0x00, 0xf0, 0xff, 0xff]); // align end down
    code.bytes(&[0x4c, 0x39, 0xd8]); // cmp candidate, end
    code.jump_if(0x83, "next");
    code.bytes(&[0x41, 0xb0, 0x01]); // one allocatable page exists
    code.jump("next");

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail"); // empty RAM is malformed

    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]); // next 20-byte entry
    code.bytes(&[0xff, 0xc9]); // dec ecx
    code.jump_if(0x85, "loop");
    code.bytes(&[0x45, 0x84, 0xc0]); // test r8b, r8b
    code.jump_if(0x84, "fail");
    code.bytes(&[0xb0, 0x01, 0xc3]); // success

    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]); // failure
    code.finish()
}

fn physical_frame_selector() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x54]); // preserve r12
    code.bytes(&[0x41, 0x55]); // preserve r13
    code.bytes(&[0x49, 0x89, 0xcc]); // retain optional excluded base
    code.bytes(&[0x4d, 0x89, 0xc5]); // retain optional excluded end
    code.bytes(&[0x4d, 0x85, 0xe4]);
    code.jump_if(0x85, "exclusion-present");
    code.bytes(&[0x4d, 0x85, 0xed]); // absent base requires absent end
    code.jump_if(0x85, "fail");
    code.jump("count-admitted");
    code.bind("exclusion-present")?;
    code.bytes(&[0x4d, 0x85, 0xed]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4d, 0x39, 0xec]); // excluded end > excluded base
    code.jump_if(0x83, "fail");
    code.bytes(&[0x41, 0xf7, 0xc4, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0xf7, 0xc5, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]); // exclusion admits one or two frames
    code.jump_if(0x84, "count-admitted");
    code.bytes(&[0x83, 0xfa, 0x02]);
    code.jump_if(0x85, "fail");
    code.bind("count-admitted")?;
    code.bytes(&[0x83, 0xfa, 0x01]); // frame count == 1
    code.jump_if(0x84, "count-ready");
    code.bytes(&[0x83, 0xfa, 0x02]); // frame count == 2
    code.jump_if(0x84, "count-ready");
    code.bytes(&[0x83, 0xfa, 0x03]); // frame count == 3
    code.jump_if(0x84, "count-ready");
    code.bytes(&[0x83, 0xfa, 0x05]); // frame count == 5
    code.jump_if(0x84, "count-ready");
    code.bytes(&[0x83, 0xfa, 34]); // post-random anonymous growth extent
    code.jump_if(0x84, "count-ready");
    code.bytes(&[0x81, 0xfa, 0xf7, 0x01, 0x00, 0x00]); // frame count == 503
    code.jump_if(0x85, "fail");
    code.bind("count-ready")?;
    code.bytes(&[0x48, 0xc1, 0xe2, 0x0c]); // byte count = frames * 4096
    code.bytes(&[0x49, 0x89, 0xd0]); // retain byte count in r8
    code.bytes(&[0x48, 0x85, 0xff]); // test private allocation floor
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // test rsi, rsi
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0xf0, 0xff, 0x00]); // cmp rsi, 0x00fff000
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xbe, 0x50, 0x02, 0x00, 0x00, 0x00]); // setup_data == 0
    code.jump_if(0x85, "fail");
    emit_initial_boot_resource_handoff_validation(&mut code)?;
    code.bytes(&[0x49, 0x89, 0xd0]); // restore byte count after handoff validation
    code.bytes(&[0x0f, 0xb6, 0x8e, 0xe8, 0x01, 0x00, 0x00]); // e820_entries
    code.bytes(&[0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x81, 0xf9, 0x80, 0x00, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x8d, 0x96, 0xd0, 0x02, 0x00, 0x00]); // first E820 entry

    code.bind("loop")?;
    code.bytes(&[0x48, 0x8b, 0x02]); // candidate = base
    code.bytes(&[0x4c, 0x8b, 0x4a, 0x08]); // length
    code.bytes(&[0x44, 0x8b, 0x52, 0x10]); // type
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "zero-length");
    code.bytes(&[0x49, 0x89, 0xc3]); // end = base
    code.bytes(&[0x4d, 0x01, 0xcb]); // end += length
    code.jump_if(0x82, "fail");
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]); // E820 RAM
    code.jump_if(0x85, "next");
    code.bytes(&[0x48, 0x39, 0xf8]); // base >= private floor
    code.jump_if(0x83, "base-ready");
    code.bytes(&[0x48, 0x89, 0xf8]); // candidate = private floor
    code.bind("base-ready")?;
    code.bytes(&[0x48, 0x05, 0xff, 0x0f, 0x00, 0x00]); // align candidate up
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x81, 0xe3, 0x00, 0xf0, 0xff, 0xff]); // align end down
    code.bytes(&[0x49, 0x89, 0xc2]); // extent end = candidate
    code.bytes(&[0x4d, 0x01, 0xc2]); // extent end += byte count
    code.jump_if(0x82, "fail");
    code.bytes(&[0x4d, 0x85, 0xe4]); // optional live user backing exclusion
    code.jump_if(0x84, "extent-ready");
    code.bytes(&[0x4d, 0x39, 0xe2]); // extent end <= excluded base
    code.jump_if(0x86, "extent-ready");
    code.bytes(&[0x4c, 0x39, 0xe8]); // candidate >= excluded end
    code.jump_if(0x83, "extent-ready");
    code.bytes(&[0x4c, 0x89, 0xe8]); // advance candidate past exclusion
    code.bytes(&[0x49, 0x89, 0xc2]);
    code.bytes(&[0x4d, 0x01, 0xc2]);
    code.jump_if(0x82, "fail");
    code.bind("extent-ready")?;
    code.bytes(&[0x44, 0x8b, 0x8e, 0x18, 0x02, 0x00, 0x00]); // resource base
    code.bytes(&[0x44, 0x8b, 0x9e, 0x1c, 0x02, 0x00, 0x00]); // resource size
    code.bytes(&[0x4d, 0x01, 0xcb]); // resource end
    code.bytes(&[0x4d, 0x39, 0xca]); // extent end <= resource base
    code.jump_if(0x86, "resource-disjoint");
    code.bytes(&[0x4c, 0x39, 0xd8]); // candidate >= resource end
    code.jump_if(0x83, "resource-disjoint");
    code.jump("next");
    code.bind("resource-disjoint")?;
    code.bytes(&[0x4d, 0x39, 0xda]); // extent end <= RAM end
    code.jump_if(0x87, "next");
    code.bytes(&[0x49, 0x81, 0xfa, 0x00, 0x00, 0x00, 0x40]); // end <= 1 GiB
    code.jump_if(0x87, "next");
    code.bytes(&[0x41, 0x5d, 0x41, 0x5c, 0xc3]); // restore r13/r12 and return physical base

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail");

    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "loop");

    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0x41, 0x5d, 0x41, 0x5c, 0xc3]);
    code.finish()
}
