// Included in the boot adapter tests to exercise its independent final admission.
#[test]
fn inode_binding_requires_exact_actual_owner_geometry_costs_and_source() {
    // TOPAL-SYSTEMS-INODE-MODE-ACTION-001; TOPAL-SYSTEMS-LINUX-DISPATCH-V5-001.
    let parent = temporary_parent();
    let (kernel, initial, user, artifact) = linked_kernel_source(&parent, SOURCE);
    generate_x86_64_linux_boot_image(&kernel, &artifact, &initial, &user).unwrap();
    let binding = artifact.inode_mode_runtime.as_ref().unwrap();
    let library = artifact.inode_mode_library.as_ref().unwrap();
    assert_eq!(binding.schema, "topal.inode-region-invocation.x86_64/1");
    assert_eq!(binding.instance.as_deref(), Some("inode-metadata"));
    assert_eq!(binding.scalar_count, Some(2));
    assert_eq!(
        (
            binding.maximum_source_stack_bytes,
            binding.live_frame_bytes,
            binding.caller_alignment_bytes,
            binding.combined_stack_bytes
        ),
        (1296, 184, 64, 1544)
    );
    assert_eq!(library.allocation.numeric.maximum_bytes, 12208);
    assert_eq!(
        (
            binding.directory_metadata_offset,
            binding.file_object_offset,
            binding.file_metadata_offset,
            binding.pipe_pool_offset,
            binding.pipe_metadata_offset,
            binding.pipe_creation_time_offset,
            binding.metadata_bytes
        ),
        (0x43020, 0x43100, 24, 0x3e000, 64, 96, 32)
    );
    assert_eq!(binding.source_code.len(), library.functions.len());
    for other in [
        &artifact.region_runtime.as_ref().unwrap().storage,
        &artifact.file_position_runtime.as_ref().unwrap().storage,
        &artifact.ram_page_pool_runtime.as_ref().unwrap().storage,
    ] {
        assert!(binding.storage.iter().all(|a| {
            other
                .iter()
                .all(|b| a.address + a.bytes <= b.address || b.address + b.bytes <= a.address)
        }));
    }
    for case in 0..24 {
        let mut wrong = artifact.clone();
        match case {
            0 => wrong.inode_mode_library = None,
            1 => wrong.inode_mode_runtime = None,
            2 => {
                wrong
                    .inode_mode_runtime
                    .as_mut()
                    .unwrap()
                    .directory_metadata_offset -= 8
            }
            3 => {
                wrong
                    .inode_mode_runtime
                    .as_mut()
                    .unwrap()
                    .file_metadata_offset += 8
            }
            4 => {
                wrong
                    .inode_mode_runtime
                    .as_mut()
                    .unwrap()
                    .pipe_creation_time_offset = 64
            }
            5 => wrong.inode_mode_runtime.as_mut().unwrap().metadata_bytes = 24,
            6 => {
                wrong
                    .inode_mode_runtime
                    .as_mut()
                    .unwrap()
                    .combined_stack_bytes -= 8
            }
            7 => {
                wrong
                    .inode_mode_runtime
                    .as_mut()
                    .unwrap()
                    .maximum_source_stack_bytes -= 8
            }
            8 => wrong.inode_mode_runtime.as_mut().unwrap().instance = Some("file-position".into()),
            9 => wrong.inode_mode_runtime.as_mut().unwrap().scalar_count = Some(1),
            10 => wrong.inode_mode_runtime.as_mut().unwrap().storage[0].address += 8,
            11 => {
                wrong
                    .inode_mode_runtime
                    .as_mut()
                    .unwrap()
                    .controller_owner
                    .bytes -= 8
            }
            12 => wrong.inode_mode_runtime.as_mut().unwrap().source_code[0].sha256 = "0".repeat(64),
            13 => {
                wrong
                    .inode_mode_library
                    .as_mut()
                    .unwrap()
                    .allocation
                    .numeric
                    .source_evaluation_bytes -= 8
            }
            14 => {
                wrong
                    .inode_mode_library
                    .as_mut()
                    .unwrap()
                    .maximum_stack_bytes -= 8
            }
            15 => {
                wrong.inode_mode_library.as_mut().unwrap().schema =
                    "topal.kernel-region-frame-source-object.x86_64/2".into()
            }
            16 => {
                wrong.inode_mode_library.as_mut().unwrap().instance = Some("ram-page-pool".into())
            }
            17 => {
                wrong
                    .inode_mode_library
                    .as_mut()
                    .unwrap()
                    .allocation
                    .unsigned_runtime_sha256 = None
            }
            18 => wrong
                .outputs
                .retain(|o| o.identity != "inode-mode-library.o"),
            19 => wrong
                .inputs
                .retain(|o| !o.identity.starts_with("kernel-inode-mode-source-module:")),
            20 => wrong.volatile_files.directory_metadata_offset = 8,
            21 => wrong.volatile_files.pipe_metadata_offset = 72,
            22 => wrong.volatile_files.schema = "topal.volatile-file.x86_64/3".into(),
            _ => {
                wrong
                    .inode_mode_runtime
                    .as_mut()
                    .unwrap()
                    .retired_stack_reserve_bytes -= 8
            }
        }
        assert!(
            generate_x86_64_linux_boot_image(&kernel, &wrong, &initial, &user).is_err(),
            "accepted inode tamper {case}"
        );
    }
    let elf = object::File::parse(kernel.as_slice()).unwrap();
    use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
    for name in binding
        .code
        .iter()
        .chain(&binding.source_code)
        .map(|c| &c.symbol)
    {
        let symbol = elf.symbol_by_name(name).unwrap();
        let section = elf
            .section_by_index(symbol.section_index().unwrap())
            .unwrap();
        let (offset, _) = section.file_range().unwrap();
        let at = (offset + symbol.address() - section.address()) as usize;
        let mut changed = kernel.clone();
        changed[at] ^= 1;
        assert!(
            generate_x86_64_linux_boot_image(&changed, &artifact, &initial, &user).is_err(),
            "accepted changed {name}"
        );
    }
    let value = serde_json::to_value(&artifact).unwrap();
    for key in ["inode_mode_library", "inode_mode_runtime"] {
        let mut wrong = value.clone();
        wrong[key] = serde_json::Value::Null;
        assert!(serde_json::from_value::<SystemsArtifactProvenance>(wrong).is_err());
    }
    fs::remove_dir_all(parent).unwrap();
}
