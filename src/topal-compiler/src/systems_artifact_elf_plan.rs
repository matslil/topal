fn elf_u16(image: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        image[offset..offset + 2]
            .try_into()
            .expect("validated ELF header extent"),
    )
}

fn elf_u32(image: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        image[offset..offset + 4]
            .try_into()
            .expect("validated ELF header extent"),
    )
}

fn checked_elf_interval(
    start: u64,
    size: u64,
    description: &str,
) -> Result<StaticElfInterval, CompileError> {
    let end = start.checked_add(size).ok_or_else(|| {
        CompileError::Tool(format!(
            "bounded static ELF {description} interval overflows"
        ))
    })?;
    Ok(StaticElfInterval { start, end })
}

fn page_floor(value: u64) -> u64 {
    value & !(STATIC_ELF_PAGE_SIZE - 1)
}

fn page_ceil(value: u64, description: &str) -> Result<u64, CompileError> {
    value
        .checked_add(STATIC_ELF_PAGE_SIZE - 1)
        .map(page_floor)
        .ok_or_else(|| {
            CompileError::Tool(format!(
                "bounded static ELF {description} page interval overflows"
            ))
        })
}

#[allow(clippy::too_many_lines)] // Keeps one fail-closed validation transaction auditable.
fn derive_x86_64_static_elf_load_plan(image: &[u8]) -> Result<StaticElfLoadPlan, CompileError> {
    if image.len() < 64 {
        return Err(CompileError::Tool(
            "bounded static ELF requires a complete 64-byte ELF header".into(),
        ));
    }
    if image[0..4] != [0x7f, b'E', b'L', b'F']
        || image[4] != 2
        || image[5] != 1
        || image[6] != 1
        || elf_u16(image, 16) != 2
        || elf_u16(image, 18) != 62
        || elf_u32(image, 20) != 1
        || elf_u16(image, 52) != 64
        || elf_u16(image, 54) != 56
    {
        return Err(CompileError::Tool(
            "bounded static ELF requires current little-endian ELF64 ET_EXEC/EM_X86_64 identity and canonical header sizes"
                .into(),
        ));
    }

    let entry = image_u64(image, 24);
    let program_header_offset = image_u64(image, 32);
    let program_header_count = elf_u16(image, 56);
    if program_header_count == 0 || program_header_count > STATIC_ELF_MAX_PROGRAM_HEADERS {
        return Err(CompileError::Tool(format!(
            "bounded static ELF program-header count must be 1..={STATIC_ELF_MAX_PROGRAM_HEADERS}"
        )));
    }
    let table_bytes = u64::from(program_header_count)
        .checked_mul(56)
        .ok_or_else(|| CompileError::Tool("bounded static ELF table size overflows".into()))?;
    let table = checked_elf_interval(program_header_offset, table_bytes, "program-header table")?;
    if table.start < 64 || table.end > image.len() as u64 {
        return Err(CompileError::Tool(
            "bounded static ELF program-header table lies outside the complete input".into(),
        ));
    }

    let roles = [
        StaticElfLoadRole::Headers,
        StaticElfLoadRole::Executable,
        StaticElfLoadRole::ImmutableData,
        StaticElfLoadRole::WritableData,
    ];
    let expected_flags = [4_u32, 5, 4, 6];
    let mut segments = Vec::new();
    let mut tls = None;
    let mut relro_pages = None;
    let mut stack_executable = None;

    for index in 0..usize::from(program_header_count) {
        let offset = usize::try_from(program_header_offset).map_err(|_| {
            CompileError::Tool("bounded static ELF table offset exceeds host limits".into())
        })? + index * 56;
        let kind = elf_u32(image, offset);
        let flags = elf_u32(image, offset + 4);
        let file_offset = image_u64(image, offset + 8);
        let virtual_address = image_u64(image, offset + 16);
        let file_size = image_u64(image, offset + 32);
        let memory_size = image_u64(image, offset + 40);
        let alignment = image_u64(image, offset + 48);

        match kind {
            1 => {
                let segment_index = segments.len();
                if segment_index >= roles.len() || flags != expected_flags[segment_index] {
                    return Err(CompileError::Tool(
                        "bounded static ELF requires exactly ordered R, R/X, R, and R/W load roles without W+X"
                            .into(),
                    ));
                }
                if memory_size == 0 || file_size > memory_size {
                    return Err(CompileError::Tool(
                        "bounded static ELF load sizes require 0 < p_memsz and p_filesz <= p_memsz"
                            .into(),
                    ));
                }
                if alignment < STATIC_ELF_PAGE_SIZE || !alignment.is_power_of_two() {
                    return Err(CompileError::Tool(
                        "bounded static ELF load alignment must be a power of two at least 4096"
                            .into(),
                    ));
                }
                if file_offset % STATIC_ELF_PAGE_SIZE != virtual_address % STATIC_ELF_PAGE_SIZE {
                    return Err(CompileError::Tool(
                        "bounded static ELF load file and virtual page offsets are incongruent"
                            .into(),
                    ));
                }
                let file = checked_elf_interval(file_offset, file_size, "load file")?;
                let memory = checked_elf_interval(virtual_address, memory_size, "load memory")?;
                if file.end > image.len() as u64
                    || memory.start < STATIC_ELF_PAGE_SIZE
                    || memory.end > STATIC_ELF_MAX_USER_ADDRESS
                {
                    return Err(CompileError::Tool(
                        "bounded static ELF load interval lies outside its file or user envelope"
                            .into(),
                    ));
                }
                let pages = StaticElfInterval {
                    start: page_floor(memory.start),
                    end: page_ceil(memory.end, "load memory")?,
                };
                if segments
                    .last()
                    .is_some_and(|prior: &StaticElfLoadSegment| prior.pages.end > pages.start)
                {
                    return Err(CompileError::Tool(
                        "bounded static ELF load page intervals overlap or are unordered".into(),
                    ));
                }
                segments.push(StaticElfLoadSegment {
                    role: roles[segment_index],
                    file,
                    memory,
                    pages,
                    alignment,
                });
            }
            2 | 3 => {
                return Err(CompileError::Tool(
                    "bounded static ELF does not admit PT_DYNAMIC or PT_INTERP".into(),
                ));
            }
            7 => {
                if tls.is_some() || memory_size == 0 || file_size > memory_size {
                    return Err(CompileError::Tool(
                        "bounded static ELF admits at most one nonempty valid PT_TLS".into(),
                    ));
                }
                if alignment == 0 || !alignment.is_power_of_two() {
                    return Err(CompileError::Tool(
                        "bounded static ELF TLS alignment must be a nonzero power of two".into(),
                    ));
                }
                let file = checked_elf_interval(file_offset, file_size, "TLS file")?;
                let memory = checked_elf_interval(virtual_address, memory_size, "TLS memory")?;
                if file.end > image.len() as u64 {
                    return Err(CompileError::Tool(
                        "bounded static ELF TLS file interval exceeds the complete input".into(),
                    ));
                }
                tls = Some(StaticElfTlsPlan {
                    file,
                    memory,
                    alignment,
                });
            }
            0x6474_e551 => {
                if stack_executable.replace(flags & 1 != 0).is_some() {
                    return Err(CompileError::Tool(
                        "bounded static ELF requires exactly one PT_GNU_STACK".into(),
                    ));
                }
            }
            0x6474_e552 => {
                if relro_pages.is_some() || memory_size == 0 {
                    return Err(CompileError::Tool(
                        "bounded static ELF admits at most one nonempty PT_GNU_RELRO".into(),
                    ));
                }
                let memory = checked_elf_interval(virtual_address, memory_size, "RELRO memory")?;
                relro_pages = Some(StaticElfInterval {
                    start: page_floor(memory.start),
                    end: page_ceil(memory.end, "RELRO memory")?,
                });
            }
            _ => {}
        }
    }

    if segments.len() != roles.len() {
        return Err(CompileError::Tool(
            "bounded static ELF requires exactly four PT_LOAD entries".into(),
        ));
    }
    let executable = segments[1];
    if entry < executable.memory.start || entry >= executable.memory.end {
        return Err(CompileError::Tool(
            "bounded static ELF entry lies outside the R/X load interval".into(),
        ));
    }
    let writable = segments[3];
    if let Some(tls_plan) = tls
        && (tls_plan.file.start < writable.file.start
            || tls_plan.file.end > writable.file.end
            || tls_plan.memory.start < writable.memory.start
            || tls_plan.memory.end > writable.memory.end)
    {
        return Err(CompileError::Tool(
            "bounded static ELF TLS intervals lie outside the R/W load role".into(),
        ));
    }
    if let Some(relro) = relro_pages
        && (relro.start < writable.pages.start
            || relro.end > writable.pages.end
            || relro.start == relro.end)
    {
        return Err(CompileError::Tool(
            "bounded static ELF RELRO pages lie outside the R/W load role".into(),
        ));
    }
    let Some(stack_executable) = stack_executable else {
        return Err(CompileError::Tool(
            "bounded static ELF requires exactly one PT_GNU_STACK".into(),
        ));
    };
    if stack_executable {
        return Err(CompileError::Tool(
            "bounded static ELF requires a non-executable stack".into(),
        ));
    }

    let pages = StaticElfInterval {
        start: segments[0].pages.start,
        end: segments[3].pages.end,
    };
    let page_count = (pages.end - pages.start) / STATIC_ELF_PAGE_SIZE;
    if page_count == 0
        || page_count > 512
        || page_floor(pages.start) / STATIC_ELF_LEAF_WINDOW_SIZE
            != (pages.end - 1) / STATIC_ELF_LEAF_WINDOW_SIZE
    {
        return Err(CompileError::Tool(
            "bounded static ELF load span exceeds one 512-leaf 2 MiB window".into(),
        ));
    }

    Ok(StaticElfLoadPlan {
        identity: StaticElfIdentity {
            class: image[4],
            data_encoding: image[5],
            version: image[6],
            file_type: elf_u16(image, 16),
            machine: elf_u16(image, 18),
        },
        entry,
        segments,
        pages,
        page_count,
        tls,
        relro_pages,
        stack_executable,
    })
}

#[allow(clippy::too_many_lines)] // Keeps one inherited-plan validation transaction auditable.
fn derive_x86_64_static_elf_page_plan(
    load: &StaticElfLoadPlan,
) -> Result<StaticElfPagePlan, CompileError> {
    if load.segments.len() != 4
        || load.page_count == 0
        || load.page_count > 512
        || !load.pages.start.is_multiple_of(STATIC_ELF_PAGE_SIZE)
        || !load.pages.end.is_multiple_of(STATIC_ELF_PAGE_SIZE)
        || load.pages.end <= load.pages.start
        || (load.pages.end - load.pages.start) / STATIC_ELF_PAGE_SIZE != load.page_count
    {
        return Err(CompileError::Tool(
            "bounded static ELF page realization has an inconsistent inherited span".into(),
        ));
    }

    let expected_roles = [
        StaticElfLoadRole::Headers,
        StaticElfLoadRole::Executable,
        StaticElfLoadRole::ImmutableData,
        StaticElfLoadRole::WritableData,
    ];
    let mut next_page = load.pages.start;
    for (segment, expected_role) in load.segments.iter().zip(expected_roles) {
        if segment.role != expected_role
            || segment.pages.start != next_page
            || !segment.pages.start.is_multiple_of(STATIC_ELF_PAGE_SIZE)
            || !segment.pages.end.is_multiple_of(STATIC_ELF_PAGE_SIZE)
            || segment.pages.end <= segment.pages.start
            || segment.memory.start < segment.pages.start
            || segment.memory.end > segment.pages.end
            || segment.file.end < segment.file.start
            || segment.memory.end < segment.memory.start
            || segment.file.end - segment.file.start > segment.memory.end - segment.memory.start
        {
            return Err(CompileError::Tool(
                "bounded static ELF page realization requires four exact adjacent inherited roles"
                    .into(),
            ));
        }
        next_page = segment.pages.end;
    }
    if next_page != load.pages.end {
        return Err(CompileError::Tool(
            "bounded static ELF page realization does not cover the complete inherited span".into(),
        ));
    }

    let writable = load.segments[3];
    if let Some(relro) = load.relro_pages
        && (!relro.start.is_multiple_of(STATIC_ELF_PAGE_SIZE)
            || !relro.end.is_multiple_of(STATIC_ELF_PAGE_SIZE)
            || relro.start >= relro.end
            || relro.start < writable.pages.start
            || relro.end > writable.pages.end)
    {
        return Err(CompileError::Tool(
            "bounded static ELF page realization has invalid inherited RELRO pages".into(),
        ));
    }

    let mut pages = Vec::with_capacity(usize::try_from(load.page_count).map_err(|_| {
        CompileError::Tool("bounded static ELF page count exceeds host limits".into())
    })?);
    for segment in &load.segments {
        let mut virtual_address = segment.pages.start;
        while virtual_address < segment.pages.end {
            let entry_permission = match segment.role {
                StaticElfLoadRole::Executable => StaticElfPagePermission::ReadOnlyExecute,
                StaticElfLoadRole::WritableData => StaticElfPagePermission::ReadWriteNoExecute,
                StaticElfLoadRole::Headers | StaticElfLoadRole::ImmutableData => {
                    StaticElfPagePermission::ReadOnlyNoExecute
                }
            };
            pages.push(StaticElfPageRecord {
                virtual_address,
                frame_ordinal: (virtual_address - load.pages.start) / STATIC_ELF_PAGE_SIZE,
                source_role: segment.role,
                initial_permission: StaticElfPagePermission::ReadWriteNoExecute,
                entry_permission,
            });
            virtual_address += STATIC_ELF_PAGE_SIZE;
        }
    }
    let derived_page_count = u64::try_from(pages.len()).map_err(|_| {
        CompileError::Tool("bounded static ELF realized page count exceeds u64".into())
    })?;
    if derived_page_count != load.page_count
        || pages.iter().enumerate().any(|(index, page)| {
            let ordinal = u64::try_from(index).expect("bounded to 512 page records");
            page.frame_ordinal != ordinal
                || page.virtual_address != load.pages.start + ordinal * STATIC_ELF_PAGE_SIZE
        })
    {
        return Err(CompileError::Tool(
            "bounded static ELF page realization has incomplete page ordinals".into(),
        ));
    }

    let mut construction = Vec::with_capacity(5);
    construction.push(StaticElfConstructionOperation::Zero(load.pages));
    for segment in &load.segments {
        let copy_size = segment.file.end - segment.file.start;
        let destination =
            checked_elf_interval(segment.memory.start, copy_size, "copy destination")?;
        if destination.end > segment.memory.end {
            return Err(CompileError::Tool(
                "bounded static ELF copy exceeds its inherited memory role".into(),
            ));
        }
        construction.push(StaticElfConstructionOperation::Copy(
            StaticElfCopyOperation {
                file: segment.file,
                destination,
            },
        ));
    }

    Ok(StaticElfPagePlan {
        frame_count: load.page_count,
        alignment_frames: 1,
        construction,
        pages,
        entry: load.entry,
        tls: load.tls,
        relro_pages: load.relro_pages,
    })
}

fn validate_x86_64_static_user_image(image: &[u8]) -> Result<(), CompileError> {
    const EXPECTED_FILE_BYTES: usize = 2_024_544;
    const EXPECTED_ENTRY: u64 = 0x004a_6580;
    let load = derive_x86_64_static_elf_load_plan(image)?;
    let pages = derive_x86_64_static_elf_page_plan(&load)?;
    let expected_segments = [
        StaticElfLoadSegment {
            role: StaticElfLoadRole::Headers,
            file: StaticElfInterval {
                start: 0,
                end: 0x6e0,
            },
            memory: StaticElfInterval {
                start: 0x0040_0000,
                end: 0x0040_06e0,
            },
            pages: StaticElfInterval {
                start: 0x0040_0000,
                end: 0x0040_1000,
            },
            alignment: 0x1000,
        },
        StaticElfLoadSegment {
            role: StaticElfLoadRole::Executable,
            file: StaticElfInterval {
                start: 0x1000,
                end: 0x18_c285,
            },
            memory: StaticElfInterval {
                start: 0x0040_1000,
                end: 0x0058_c285,
            },
            pages: StaticElfInterval {
                start: 0x0040_1000,
                end: 0x0058_d000,
            },
            alignment: 0x1000,
        },
        StaticElfLoadSegment {
            role: StaticElfLoadRole::ImmutableData,
            file: StaticElfInterval {
                start: 0x18_d000,
                end: 0x1e_4040,
            },
            memory: StaticElfInterval {
                start: 0x0058_d000,
                end: 0x005e_4040,
            },
            pages: StaticElfInterval {
                start: 0x0058_d000,
                end: 0x005e_5000,
            },
            alignment: 0x1000,
        },
        StaticElfLoadSegment {
            role: StaticElfLoadRole::WritableData,
            file: StaticElfInterval {
                start: 0x1e_4660,
                end: 0x1e_dd88,
            },
            memory: StaticElfInterval {
                start: 0x005e_5660,
                end: 0x005f_61c8,
            },
            pages: StaticElfInterval {
                start: 0x005e_5000,
                end: 0x005f_7000,
            },
            alignment: 0x1000,
        },
    ];
    let expected_tls = Some(StaticElfTlsPlan {
        file: StaticElfInterval {
            start: 0x1e_4660,
            end: 0x1e_46a0,
        },
        memory: StaticElfInterval {
            start: 0x005e_5660,
            end: 0x005e_56e2,
        },
        alignment: 8,
    });
    let expected_relro = Some(StaticElfInterval {
        start: 0x005e_5000,
        end: 0x005e_d000,
    });
    let expected_program_headers = [
        (1, 4, 0, 0x0040_0000, 0x6e0, 0x6e0, 0x1000),
        (1, 5, 0x1000, 0x0040_1000, 0x18_b285, 0x18_b285, 0x1000),
        (1, 4, 0x18_d000, 0x0058_d000, 0x5_7040, 0x5_7040, 0x1000),
        (1, 6, 0x1e_4660, 0x005e_5660, 0x9728, 0x1_0b68, 0x1000),
        (4, 4, 0x2a8, 0x0040_02a8, 0x20, 0x20, 8),
        (4, 4, 0x2c8, 0x0040_02c8, 0x24, 0x24, 4),
        (4, 4, 0x1e_4020, 0x005e_4020, 0x20, 0x20, 4),
        (7, 4, 0x1e_4660, 0x005e_5660, 0x40, 0x82, 8),
        (0x6474_e553, 4, 0x2a8, 0x0040_02a8, 0x20, 0x20, 8),
        (0x6474_e551, 6, 0, 0, 0, 0, 0x10),
        (0x6474_e552, 4, 0x1e_4660, 0x005e_5660, 0x79a0, 0x79a0, 1),
    ];
    let exact_program_headers = expected_program_headers.iter().enumerate().all(
        |(index, &(kind, flags, offset, virtual_address, file_size, memory_size, alignment))| {
            let header = 64 + index * 56;
            elf_u32(image, header) == kind
                && elf_u32(image, header + 4) == flags
                && image_u64(image, header + 8) == offset
                && image_u64(image, header + 16) == virtual_address
                && image_u64(image, header + 24) == virtual_address
                && image_u64(image, header + 32) == file_size
                && image_u64(image, header + 40) == memory_size
                && image_u64(image, header + 48) == alignment
        },
    );
    let section_header_shape = match (
        image_u64(image, 40),
        elf_u16(image, 58),
        elf_u16(image, 60),
        elf_u16(image, 62),
    ) {
        (0, 0, 0, 0) => true,
        (0x1e_dea0, 64, 23, 22) => {
            let directory_bytes = 64_usize.checked_mul(23);
            let directory_end =
                directory_bytes.and_then(|bytes| 0x1e_dea0_usize.checked_add(bytes));
            directory_end == Some(image.len())
        }
        _ => false,
    };

    if image.len() != EXPECTED_FILE_BYTES
        || image_u64(image, 32) != 64
        || elf_u16(image, 56) != 11
        || !section_header_shape
        || !exact_program_headers
        || load.entry != EXPECTED_ENTRY
        || load.segments.as_slice() != expected_segments
        || load.pages
            != (StaticElfInterval {
                start: 0x0040_0000,
                end: 0x005f_7000,
            })
        || load.page_count != 503
        || load.tls != expected_tls
        || load.relro_pages != expected_relro
        || load.stack_executable
        || pages.frame_count != 503
        || pages.entry != EXPECTED_ENTRY
    {
        return Err(CompileError::Tool(
            "static user image does not match the approved 503-page x86-64 shape".into(),
        ));
    }
    Ok(())
}

fn validate_x86_64_initial_user_image(image: &[u8]) -> Result<(), CompileError> {
    let reference = initial_user_elf_file()?;
    if image.len() != reference.len() {
        if let Ok(load_plan) = derive_x86_64_static_elf_load_plan(image) {
            let page_plan = derive_x86_64_static_elf_page_plan(&load_plan)?;
            return Err(CompileError::Tool(format!(
                "bounded static ELF page plan covers one {}-frame affine extent at entry {:#x}, but this image is not runtime-selectable until its mapping lifecycle is implemented",
                page_plan.frame_count, page_plan.entry
            )));
        }
        return Err(CompileError::Tool(format!(
            "initial user image has {} bytes; the qualified x86-64 profile requires {}",
            image.len(),
            reference.len()
        )));
    }

    let variable_header_ranges = [
        INITIAL_USER_ENTRY_ELF_OFFSET..INITIAL_USER_ENTRY_ELF_OFFSET + 8,
        INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET..INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET + 8,
        INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET..INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET + 8,
        INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET..INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET + 8,
        INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET..INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET + 8,
    ];
    for offset in 0..256 {
        if variable_header_ranges
            .iter()
            .any(|range| range.contains(&offset))
        {
            continue;
        }
        if image[offset] != reference[offset] {
            return Err(CompileError::Tool(format!(
                "initial user image changes immutable x86-64 ELF metadata at byte {offset}"
            )));
        }
    }

    let code_file_size = image_u64(image, INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET);
    let code_memory_size = image_u64(image, INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET);
    if code_file_size != code_memory_size
        || !(4360..=8192).contains(&code_file_size)
        || code_file_size % 8 != 0
    {
        return Err(CompileError::Tool(
            "initial user executable extent must be equal file/memory sizes of 4360..8192 bytes aligned to 8"
                .into(),
        ));
    }

    let entry = image_u64(image, INITIAL_USER_ENTRY_ELF_OFFSET);
    let code_end = INITIAL_USER_CODE_ADDRESS
        .checked_add(code_file_size)
        .ok_or_else(|| CompileError::Tool("initial user executable extent overflows".into()))?;
    if entry < INITIAL_USER_CODE_ADDRESS + 4096 + 256 || entry >= code_end {
        return Err(CompileError::Tool(
            "initial user entry must lie inside the executable payload extent".into(),
        ));
    }

    let data_file_size = image_u64(image, INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET);
    let data_memory_size = image_u64(image, INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET);
    if !(8..=INITIAL_USER_DATA_CAPACITY as u64).contains(&data_file_size)
        || data_file_size % 8 != 0
        || data_memory_size < data_file_size
        || data_memory_size > INITIAL_USER_DATA_CAPACITY as u64
        || data_memory_size % 8 != 0
    {
        return Err(CompileError::Tool(
            "initial user data extent requires an 8-byte-aligned file size of 8..4096 and memory size between it and 4096"
                .into(),
        ));
    }
    if INITIAL_USER_DATA_FILE_OFFSET + INITIAL_USER_DATA_CAPACITY != image.len() {
        return Err(CompileError::Tool(
            "initial user data segment is outside the qualified file envelope".into(),
        ));
    }
    Ok(())
}

fn image_u64(image: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        image[offset..offset + 8]
            .try_into()
            .expect("validated ELF extent"),
    )
}
