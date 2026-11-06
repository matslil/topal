fn build_linked_outputs(
    tools: &LlvmTools,
    work: &Path,
    stage: &Path,
) -> Result<LinkedOutputs, CompileError> {
    let unstripped_path = work.join("kernel.unstripped");
    let kernel_path = stage.join(SYSTEMS_KERNEL_FILE);
    let debug_path = stage.join(SYSTEMS_DEBUG_FILE);
    let map_path = stage.join(SYSTEMS_MAP_FILE);

    run_tool(
        Command::new(tools.tool_path("rust-lld"))
            .current_dir(work)
            .arg("-flavor")
            .arg("gnu")
            .arg("-m")
            .arg("elf_x86_64")
            .arg("-static")
            .arg("--no-dynamic-linker")
            .arg("--build-id=none")
            .arg("--entry")
            .arg(X86_SYSTEMS_KERNEL_ENTRY)
            .arg("-z")
            .arg("max-page-size=0x1000")
            .arg(format!("-Map=../{SYSTEMS_MAP_FILE}"))
            .arg("root.o")
            .arg("provider.o")
            .args(
                work.join("inode-mode/kernel-region.o")
                    .is_file().then_some("inode-mode/kernel-region.o"),
            )
            .args(
                work.join("ram-page-pool/kernel-region.o")
                    .is_file()
                    .then_some("ram-page-pool/kernel-region.o"),
            )
            .args(
                work.join("file-position/kernel-region.o")
                    .is_file()
                    .then_some("file-position/kernel-region.o"),
            )
            .args(
                work.join("kernel-region.o")
                    .is_file()
                    .then_some("kernel-region.o"),
            )
            .args(
                work.join("time-library.o")
                    .is_file()
                    .then_some("time-library.o"),
            )
            .args(
                work.join("image-library.o")
                    .is_file()
                    .then_some("image-library.o"),
            )
            .args(
                work.join("process-identity-library.o")
                    .is_file()
                    .then_some("process-identity-library.o"),
            )
            .args(
                work.join("dispatch-library.o")
                    .is_file()
                    .then_some("dispatch-library.o"),
            )
            .args(
                work.join("kernel-library.o")
                    .is_file()
                    .then_some("kernel-library.o"),
            )
            .arg("-o")
            .arg("kernel.unstripped"),
    )?;

    let objcopy = tools.tool_path("llvm-objcopy");
    if !objcopy.is_file() {
        return Err(CompileError::Tool(format!(
            "LLVM tool `{}` is missing; systems artifact publication requires llvm-objcopy",
            objcopy.display()
        )));
    }
    run_tool(
        Command::new(&objcopy)
            .arg("--only-keep-debug")
            .arg("--remove-section=.debug_names")
            .arg(&unstripped_path)
            .arg(&debug_path),
    )?;
    run_tool(
        Command::new(&objcopy)
            .current_dir(stage)
            .arg("--strip-debug")
            .arg(format!("--add-gnu-debuglink={SYSTEMS_DEBUG_FILE}"))
            .arg(&unstripped_path)
            .arg(&kernel_path),
    )?;

    let kernel = fs::read(&kernel_path).map_err(io_error("read linked kernel", &kernel_path))?;
    let debug = fs::read(&debug_path).map_err(io_error("read debug artifact", &debug_path))?;
    let map = fs::read(&map_path).map_err(io_error("read link map", &map_path))?;
    Ok(LinkedOutputs { kernel, debug, map })
}

#[allow(clippy::too_many_arguments)] // Every independently hashed artifact input stays explicit.
fn artifact_provenance(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    plan: &crate::X86SystemsProviderPlan,
    provider_object: &[u8],
    root_object: &[u8],
    kernel: &[u8],
    debug: &[u8],
    map: &[u8],
    initial_user_image: &[u8],
    static_user_image: Option<&[u8]>,
    placements: Vec<SystemsArtifactPlacement>,
) -> Result<SystemsArtifactProvenance, CompileError> {
    let mut inputs = vec![
        DigestEntry {
            identity: X86_SYSTEMS_PROVIDER_OBJECT_REVISION.into(),
            sha256: sha256(provider_object),
        },
        DigestEntry {
            identity: X86_SYSTEMS_ROOT_OBJECT_REVISION.into(),
            sha256: sha256(root_object),
        },
    ];
    inputs.sort_by(|left, right| left.identity.cmp(&right.identity));
    let mut outputs = vec![
        DigestEntry {
            identity: SYSTEMS_DEBUG_FILE.into(),
            sha256: sha256(debug),
        },
        DigestEntry {
            identity: SYSTEMS_KERNEL_FILE.into(),
            sha256: sha256(kernel),
        },
        DigestEntry {
            identity: SYSTEMS_MAP_FILE.into(),
            sha256: sha256(map),
        },
        DigestEntry {
            identity: SYSTEMS_INITIAL_USER_IMAGE_FILE.into(),
            sha256: sha256(initial_user_image),
        },
    ];
    if let Some(image) = static_user_image {
        outputs.push(DigestEntry {
            identity: SYSTEMS_STATIC_USER_IMAGE_FILE.into(),
            sha256: sha256(image),
        });
    }
    outputs.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(SystemsArtifactProvenance {
        schema: X86_SYSTEMS_ARTIFACT_REVISION.into(),
        provider: X86_SYSTEMS_PROVIDER_REVISION.into(),
        provider_object: X86_SYSTEMS_PROVIDER_OBJECT_REVISION.into(),
        root_object: X86_SYSTEMS_ROOT_OBJECT_REVISION.into(),
        target: plan.target.into(),
        board: plan.board.into(),
        profile: plan.profile.into(),
        machine_cpu: plan.machine_cpu_model.into(),
        codegen_cpu: plan.codegen_cpu.into(),
        platform_abi: X86_SYSTEMS_PLATFORM_ABI.into(),
        data_layout: plan.data_layout.into(),
        object_format: plan.object_format.into(),
        relocation_model: plan.relocation_model.into(),
        code_model: plan.code_model.into(),
        llvm_version: tools.version.clone(),
        entry_symbol: X86_SYSTEMS_KERNEL_ENTRY.into(),
        exception_entries: vec![
            X86_SYSTEMS_DEBUG_BREAK_ENTRY.into(),
            X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY.into(),
            X86_SYSTEMS_DEADLINE_ENTRY.into(),
            X86_SYSTEMS_USER_COPY_FAULT_SYMBOL.into(),
        ],
        bootstrap_storage_capacity: plan.bootstrap_placement.capacity_bytes,
        bootstrap_storage_alignment: plan.bootstrap_placement.alignment_bytes,
        placements,
        semantic_trace: semantic_trace(program)?,
        kernel_library: None,
        region_library: None,
        file_position_library: None,
        file_position_runtime: None,
        ram_page_pool_library: None,
        ram_page_pool_runtime: None,
        inode_mode_library: None,
        inode_mode_runtime: None,
        region_runtime: None,
        time_library: None,
        image_library: None,
        dispatch_library: None,
        process_identity_library: None,
        volatile_files: crate::systems_provider_object::volatile_file_profile(
            program.ram_page_pool_library.is_some(),
            program.inode_mode_library.is_some(),
        ),
        inputs,
        outputs,
    })
}

fn inspect_linked_kernel(
    kernel: &[u8],
    debug: &[u8],
    map: &[u8],
    storage_capacity: u64,
    requires_bootstrap_memory: bool,
) -> Result<(), CompileError> {
    let file = object::File::parse(kernel)
        .map_err(|error| CompileError::Tool(format!("cannot parse linked systems ELF: {error}")))?;
    inspect_kernel_elf(&file, storage_capacity, requires_bootstrap_memory)?;
    inspect_debug_and_map(debug, map)
}

fn inspect_kernel_elf(
    file: &object::File<'_>,
    storage_capacity: u64,
    requires_bootstrap_memory: bool,
) -> Result<(), CompileError> {
    if file.format() != BinaryFormat::Elf
        || file.architecture() != Architecture::X86_64
        || !file.is_little_endian()
        || file.kind() != object::ObjectKind::Executable
    {
        return Err(CompileError::Tool(
            "linked systems artifact is not a little-endian executable ELF64 x86-64 image".into(),
        ));
    }
    let entry = file
        .symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY)
        .ok_or_else(|| {
            CompileError::Tool(format!(
                "linked systems artifact omits `{X86_SYSTEMS_KERNEL_ENTRY}`"
            ))
        })?;
    if entry.address() == 0 || entry.address() != file.entry() {
        return Err(CompileError::Tool(
            "linked systems ELF entry does not select its generated bootstrap root".into(),
        ));
    }
    inspect_bootstrap_reservation_floor(file)?;
    if requires_bootstrap_memory {
        inspect_bootstrap_memory_instructions(file)?;
    }
    if file
        .symbols()
        .filter(object::ObjectSymbol::is_undefined)
        .count()
        != 0
        || file
            .sections()
            .flat_map(|section| section.relocations())
            .count()
            != 0
    {
        return Err(CompileError::Tool(
            "linked systems ELF retains an undefined dependency or relocation".into(),
        ));
    }
    for forbidden in [
        ".interp",
        ".dynamic",
        ".dynsym",
        ".plt",
        ".got.plt",
        ".eh_frame",
        ".eh_frame_hdr",
        ".gcc_except_table",
    ] {
        if file.section_by_name(forbidden).is_some() {
            return Err(CompileError::Tool(format!(
                "linked systems ELF contains forbidden hosted section `{forbidden}`"
            )));
        }
    }
    for required in REQUIRED_LINKED_TEXT_SYMBOLS {
        let symbol = file.symbol_by_name(required).ok_or_else(|| {
            CompileError::Tool(format!(
                "linked systems ELF omits generated symbol `{required}`"
            ))
        })?;
        if symbol.kind() != SymbolKind::Text || symbol.address() == 0 {
            return Err(CompileError::Tool(format!(
                "linked systems symbol `{required}` lacks executable placement"
            )));
        }
    }
    let storage = file
        .symbol_by_name("topal_bootstrap_storage")
        .ok_or_else(|| {
            CompileError::Tool("linked systems ELF omits generated bootstrap storage".into())
        })?;
    let storage_section = storage
        .section_index()
        .and_then(|index| file.section_by_index(index).ok())
        .ok_or_else(|| {
            CompileError::Tool("linked bootstrap storage has no allocated section".into())
        })?;
    if storage_section.kind() != SectionKind::UninitializedData
        || storage.size() != storage_capacity
        || storage.address() % 4096 != 0
    {
        return Err(CompileError::Tool(
            "linked systems ELF violates checked bootstrap-storage placement".into(),
        ));
    }
    if file.section_by_name(".gnu_debuglink").is_none() {
        return Err(CompileError::Tool(
            "linked systems ELF omits its canonical debug companion link".into(),
        ));
    }
    Ok(())
}

fn inspect_bootstrap_reservation_floor(file: &object::File<'_>) -> Result<(), CompileError> {
    for segment in file.segments() {
        let end = segment
            .address()
            .checked_add(segment.size())
            .ok_or_else(|| CompileError::Tool("linked systems segment range overflows".into()))?;
        if end > X86_SYSTEMS_ALLOCATABLE_FLOOR {
            return Err(CompileError::Tool(format!(
                "linked systems segment extends beyond reserved bootstrap floor {X86_SYSTEMS_ALLOCATABLE_FLOOR:#x}"
            )));
        }
    }
    Ok(())
}

fn inspect_bootstrap_memory_instructions(file: &object::File<'_>) -> Result<(), CompileError> {
    let entry = file
        .symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY)
        .ok_or_else(|| CompileError::Tool("generated bootstrap root is absent".into()))?;
    let section = entry
        .section_index()
        .and_then(|index| file.section_by_index(index).ok())
        .ok_or_else(|| {
            CompileError::Tool("generated bootstrap root has no executable section".into())
        })?;
    let section_data = section.data().map_err(|error| {
        CompileError::Tool(format!(
            "cannot inspect generated bootstrap root bytes: {error}"
        ))
    })?;
    let start =
        usize::try_from(entry.address().saturating_sub(section.address())).map_err(|_| {
            CompileError::Tool("generated bootstrap root offset exceeds host range".into())
        })?;
    let size = usize::try_from(entry.size()).map_err(|_| {
        CompileError::Tool("generated bootstrap root size exceeds host range".into())
    })?;
    let end = start
        .checked_add(size)
        .ok_or_else(|| CompileError::Tool("generated bootstrap root range overflows".into()))?;
    let root = section_data.get(start..end).ok_or_else(|| {
        CompileError::Tool("generated bootstrap root range exceeds its section".into())
    })?;
    let retains_store = root
        .windows(7)
        .any(|bytes| bytes[..2] == [0xc6, 0x05] && bytes[6] == 90);
    let retains_load_and_check = root
        .windows(15)
        .any(|bytes| bytes[..3] == [0x0f, 0xb6, 0x05] && bytes[7..11] == [0x3c, 90, 0x0f, 0x85]);
    if !retains_store || !retains_load_and_check {
        return Err(CompileError::Tool(
            "linked bootstrap root omits its checked byte store/load path".into(),
        ));
    }
    Ok(())
}

fn inspect_debug_and_map(debug: &[u8], map: &[u8]) -> Result<(), CompileError> {
    let debug_file = object::File::parse(debug).map_err(|error| {
        CompileError::Tool(format!("cannot parse systems debug artifact: {error}"))
    })?;
    if debug_file.format() != BinaryFormat::Elf || debug_file.architecture() != Architecture::X86_64
    {
        return Err(CompileError::Tool(
            "systems debug artifact does not match the kernel ELF architecture".into(),
        ));
    }
    if debug_file
        .symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY)
        .is_none()
    {
        return Err(CompileError::Tool(
            "systems debug artifact omits the generated bootstrap symbol".into(),
        ));
    }
    let map = std::str::from_utf8(map)
        .map_err(|error| CompileError::Tool(format!("systems link map is not UTF-8: {error}")))?;
    for symbol in [
        X86_SYSTEMS_KERNEL_ENTRY,
        X86_SYSTEMS_DEBUG_BREAK_ENTRY,
        X86_SYSTEMS_KERNEL_THREAD_ENTRY,
        X86_SYSTEMS_COMPLETION_THREAD_ENTRY,
        X86_SYSTEMS_TERMINAL_THREAD_ENTRY,
        X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
        X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL,
        X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
        X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
        X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL,
        X86_SYSTEMS_CRITICAL_ENTER_SYMBOL,
        X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
        X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL,
        X86_SYSTEMS_CONTEXT_CREATE_SYMBOL,
        X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL,
        X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL,
        X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL,
        X86_SYSTEMS_USER_COPY_FAULT_SYMBOL,
        "topal_x86_systems_uart16550_write",
        "topal_bootstrap_storage",
    ] {
        if !map.contains(symbol) {
            return Err(CompileError::Tool(format!(
                "systems link map omits generated symbol `{symbol}`"
            )));
        }
    }
    for input_section in [
        X86_SYSTEMS_ROOT_TEXT_SECTION,
        X86_SYSTEMS_PROVIDER_TEXT_SECTION,
        X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION,
    ] {
        if !map.contains(input_section) {
            return Err(CompileError::Tool(format!(
                "systems link map omits semantic input section `{input_section}`"
            )));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)] // The closed semantic-to-symbol placement ledger stays auditable together.
fn linked_placements(kernel: &[u8]) -> Result<Vec<SystemsArtifactPlacement>, CompileError> {
    let file = object::File::parse(kernel)
        .map_err(|error| CompileError::Tool(format!("cannot parse linked systems ELF: {error}")))?;
    let mut placements = Vec::new();
    for (semantic_identity, symbol_name) in [
        ("topal.systems.entry.bootstrap/1", X86_SYSTEMS_KERNEL_ENTRY),
        (
            "topal.systems.entry.synchronous.debug-break/1",
            X86_SYSTEMS_DEBUG_BREAK_ENTRY,
        ),
        (
            "topal.systems.entry.external.local-notification/1",
            X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
        ),
        (
            "topal.systems.entry.resumed.kernel-thread/1",
            X86_SYSTEMS_KERNEL_THREAD_ENTRY,
        ),
        (
            "topal.systems.entry.resumed.completion-thread/1",
            X86_SYSTEMS_COMPLETION_THREAD_ENTRY,
        ),
        (
            "topal.systems.entry.resumed.terminal-thread/1",
            X86_SYSTEMS_TERMINAL_THREAD_ENTRY,
        ),
        (SYSTEMS_BOOT_MEMORY_DESCRIBE, X86_SYSTEMS_BOOT_MEMORY_SYMBOL),
        (SYSTEMS_FRAMES_ALLOCATE, X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL),
        (SYSTEMS_KERNEL_MAP, X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL),
        (SYSTEMS_KERNEL_MAPPING_STORE_BYTE, X86_SYSTEMS_KERNEL_ENTRY),
        (SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, X86_SYSTEMS_KERNEL_ENTRY),
        (SYSTEMS_KERNEL_UNMAP, X86_SYSTEMS_KERNEL_ENTRY),
        (
            SYSTEMS_TRANSLATION_BEGIN,
            X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_COMMIT,
            X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_ACTIVATE,
            X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_EDIT_BEGIN,
            X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_EDIT_MAP,
            X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_EDIT_UNMAP,
            X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_EDIT_COMMIT,
            X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL,
        ),
        (SYSTEMS_CRITICAL_ENTER, X86_SYSTEMS_CRITICAL_ENTER_SYMBOL),
        (
            SYSTEMS_CRITICAL_RESTORE,
            X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
        ),
        (SYSTEMS_ATOMIC_WORD_CREATE, X86_SYSTEMS_ATOMIC_CREATE_SYMBOL),
        (
            SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
            X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
        ),
        (SYSTEMS_ATOMIC_LOAD, X86_SYSTEMS_ATOMIC_LOAD_SYMBOL),
        (SYSTEMS_ATOMIC_END, X86_SYSTEMS_KERNEL_ENTRY),
        (
            SYSTEMS_LOCAL_NOTIFICATION_SEND,
            X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL,
        ),
        (
            SYSTEMS_LOCAL_NOTIFICATION_WAIT,
            X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL,
        ),
        (
            SYSTEMS_LOCAL_NOTIFICATION_COMPLETE,
            X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL,
        ),
        (
            SYSTEMS_MONOTONIC_CLOCK_NOW,
            X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL,
        ),
        (SYSTEMS_DEADLINE_AFTER, X86_SYSTEMS_DEADLINE_AFTER_SYMBOL),
        (SYSTEMS_DEADLINE_ARM, X86_SYSTEMS_DEADLINE_ARM_SYMBOL),
        (SYSTEMS_DEADLINE_WAIT, X86_SYSTEMS_DEADLINE_WAIT_SYMBOL),
        (
            SYSTEMS_DEADLINE_COMPLETE,
            X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL,
        ),
        (
            SYSTEMS_RESUME_DEADLINE,
            "topal_x86_systems_interrupt_return",
        ),
        (
            SYSTEMS_KERNEL_CONTEXT_CREATE,
            X86_SYSTEMS_CONTEXT_CREATE_SYMBOL,
        ),
        (
            SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL,
        ),
        (
            SYSTEMS_KERNEL_CONTEXT_RETIRE,
            X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL,
        ),
        (
            SYSTEMS_KERNEL_CONTEXT_RECLAIM,
            X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
            X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING,
            X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_MAP,
            X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_STORE_BYTE,
            X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
            X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_LOAD_BYTE,
            X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
            X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA,
            X86_SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA,
            X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS,
            X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
            X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
            X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL,
        ),
        (
            SYSTEMS_USER_CONTEXT_PREPARE,
            X86_SYSTEMS_USER_PREPARE_SYMBOL,
        ),
        (SYSTEMS_USER_CONTEXT_ENTER, X86_SYSTEMS_USER_ENTER_SYMBOL),
        (
            SYSTEMS_USER_CONTEXT_COMPLETE,
            X86_SYSTEMS_USER_COMPLETE_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_RECLAIM,
            X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL,
        ),
        (SYSTEMS_USER_COPY_TO, X86_SYSTEMS_USER_SYSCALL_SYMBOL),
        (SYSTEMS_USER_COPY_FROM, X86_SYSTEMS_USER_SYSCALL_SYMBOL),
        (
            SYSTEMS_USER_SYSCALL_DISPATCH,
            X86_SYSTEMS_USER_SYSCALL_SYMBOL,
        ),
        (SYSTEMS_CONSOLE_WRITE, "topal_x86_systems_uart16550_write"),
        (SYSTEMS_DEBUG_BREAK, "topal_x86_systems_debug_break"),
        (
            SYSTEMS_RESUME_DEBUG_BREAK,
            "topal_x86_systems_interrupt_return",
        ),
        (
            SYSTEMS_RESUME_LOCAL_NOTIFICATION,
            "topal_x86_systems_interrupt_return",
        ),
        (SYSTEMS_FATAL, "topal_x86_systems_fatal"),
        (
            SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
            "topal_bootstrap_storage",
        ),
        (
            SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
            "topal_bootstrap_storage",
        ),
        (
            SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            "topal_bootstrap_storage",
        ),
    ] {
        let symbol = file.symbol_by_name(symbol_name).ok_or_else(|| {
            CompileError::Tool(format!(
                "linked systems ELF omits placement symbol `{symbol_name}`"
            ))
        })?;
        let section = symbol
            .section_index()
            .and_then(|index| file.section_by_index(index).ok())
            .ok_or_else(|| {
                CompileError::Tool(format!(
                    "linked systems placement symbol `{symbol_name}` has no section"
                ))
            })?;
        placements.push(SystemsArtifactPlacement {
            semantic_identity: semantic_identity.into(),
            symbol: symbol_name.into(),
            section: section.name().unwrap_or("<unnamed>").into(),
            address: symbol.address(),
            size: symbol.size(),
            alignment: section.align(),
        });
    }
    placements.sort_by(|left, right| {
        left.semantic_identity
            .cmp(&right.semantic_identity)
            .then_with(|| left.symbol.cmp(&right.symbol))
    });
    Ok(placements)
}
