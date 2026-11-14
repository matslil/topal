use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
use topal_language::compiler::{
    CompilerSystemsTargetSelection, SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
    SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, SYSTEMS_DEADLINE_AFTER, SYSTEMS_DEADLINE_ARM,
    SYSTEMS_DEADLINE_COMPLETE, SYSTEMS_FRAME_ALLOCATOR_CREATE, SYSTEMS_FRAMES_RELEASE,
    SYSTEMS_KERNEL_MAP, SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
    SYSTEMS_KERNEL_UNMAP, analyze_systems_for_compiler,
};

use super::*;

const SOURCE: &str = include_str!("../../../linux-kernel/labs/qemu/x86_64/pre-frame-kernel-root.t");

fn program() -> CompilerSystemsProgram {
    let mut program = analyze_systems_for_compiler(
        SOURCE,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    // The existing structural corpus explicitly exercises the retained
    // closed profile; source-bound publication is covered independently.
    program.region_library = None;
    program.file_position_library = None;
    program.ram_page_pool_library = None;
    program.inode_mode_library = None;
    program.namespace_library = None;
    program
        .bootstrap
        .handler
        .effects
        .retain(|effect| effect != topal_language::compiler::SYSTEMS_REGION_LIBRARY_APPLY);
    program.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::ApplyBootstrapRegionLibrary
        )
    });
    for operation in &mut program.bootstrap.handler.operations {
        if let CompilerSystemsOperation::BootstrapLoadByteEquals { offset_bytes, .. } = operation {
            *offset_bytes = 0;
        }
    }
    program.kernel_library = None;
    program.time_library = None;
    program.image_library = None;
    program.dispatch_library = None;
    program.process_identity_library = None;
    program
}

#[test]
fn declared_library_requires_checked_snapshot_before_publication() {
    // TOPAL-SYSTEMS-SOURCE-LIBRARY-001, TOPAL-SYSTEMS-KERNEL-POLICY-BINDING-001
    let program = analyze_systems_for_compiler(
        SOURCE,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let tools = LlvmTools::discover(None).unwrap();
    let parent = std::env::temp_dir().join(format!(
        "topal-source-policy-publication-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&parent).unwrap();
    let destination = parent.join("missing");
    assert!(publish_x86_64_systems_artifact(&program, &tools, &destination).is_err());
    assert!(!destination.exists());
    let mut source = crate::systems_library::load_source_library(
        program.kernel_library.as_ref().unwrap(),
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
    )
    .unwrap();
    source.text = source
        .text
        .replace("pub file-open-decision", "file-open-decision");
    let time = crate::systems_library::load_source_library(
        program.time_library.as_ref().unwrap(),
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
    )
    .unwrap();
    let image = crate::systems_library::load_source_library(
        program.image_library.as_ref().unwrap(),
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
    )
    .unwrap();
    let destination = parent.join("invalid");
    let process_identity = crate::systems_library::load_source_library(
        program.process_identity_library.as_ref().unwrap(),
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
    )
    .unwrap();
    let dispatch = crate::systems_library::load_source_library(
        program.dispatch_library.as_ref().unwrap(),
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
    )
    .unwrap();
    assert!(
        publish_x86_64_systems_artifact_with_kernel_libraries(
            &program,
            &tools,
            None,
            None,
            SystemsKernelLibraries {
                namespace: None,
                inode_mode: None,
                ram_page_pool: None,
                file_position: None,
                region: None,
                policy: Some(&source),
                time: Some(&time),
                image: Some(&image),
                dispatch: Some(&dispatch),
                process_identity: Some(&process_identity),
            },
            &destination
        )
        .is_err()
    );
    assert!(!destination.exists());
    assert_eq!(std::fs::read_dir(&parent).unwrap().count(), 0);
    std::fs::remove_dir_all(parent).unwrap();
}

fn put_u16(image: &mut [u8], offset: usize, value: u16) {
    image[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(image: &mut [u8], offset: usize, value: u32) {
    image[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(image: &mut [u8], offset: usize, value: u64) {
    image[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn busybox_shaped_static_elf() -> Vec<u8> {
    qualified_x86_64_static_user_image_fixture()
}

#[test]
fn derives_the_pinned_busybox_shaped_static_elf_plan() {
    // TOPAL-SYSTEMS-STATIC-ELF-PLAN-001, TOPAL-COMP-SYSTEMS-ARTIFACT-001.
    let image = busybox_shaped_static_elf();
    let plan = derive_x86_64_static_elf_load_plan(&image).unwrap();
    assert_eq!(
        plan.identity,
        StaticElfIdentity {
            class: 2,
            data_encoding: 1,
            version: 1,
            file_type: 2,
            machine: 62,
        }
    );
    assert_eq!(plan.entry, 0x004a_6580);
    assert_eq!(plan.page_count, 503);
    assert_eq!(plan.pages.start, 0x0040_0000);
    assert_eq!(plan.pages.end, 0x005f_7000);
    assert_eq!(
        plan.segments
            .iter()
            .map(|segment| segment.role)
            .collect::<Vec<_>>(),
        vec![
            StaticElfLoadRole::Headers,
            StaticElfLoadRole::Executable,
            StaticElfLoadRole::ImmutableData,
            StaticElfLoadRole::WritableData,
        ]
    );
    assert_eq!(plan.segments[1].pages.start, 0x0040_1000);
    assert_eq!(plan.segments[1].pages.end, 0x0058_d000);
    assert_eq!(plan.segments[3].memory.end, 0x005f_61c8);
    assert_eq!(
        plan.tls,
        Some(StaticElfTlsPlan {
            file: StaticElfInterval {
                start: 0x1e_4660,
                end: 0x1e_46a0,
            },
            memory: StaticElfInterval {
                start: 0x005e_5660,
                end: 0x005e_56e2,
            },
            alignment: 8,
        })
    );
    assert_eq!(
        plan.relro_pages,
        Some(StaticElfInterval {
            start: 0x005e_5000,
            end: 0x005e_d000,
        })
    );
    assert!(!plan.stack_executable);
    let error = validate_x86_64_initial_user_image(&image).unwrap_err();
    assert!(error.to_string().contains("503-frame affine extent"));
    assert!(error.to_string().contains("not runtime-selectable"));
}

#[test]
fn static_user_image_validation_requires_the_exact_approved_shape() {
    // TOPAL-SYSTEMS-STATIC-ELF-ARTIFACT-001,
    // TOPAL-SYSTEMS-STATIC-ELF-RESIDENT-HEADER-VARIANTS-001,
    // TOPAL-COMP-SYSTEMS-TEST-001.
    let mut image = busybox_shaped_static_elf();
    image[0x1000] = 0xa5;
    validate_x86_64_static_user_image(&image).unwrap();

    let mut external_header = image.clone();
    put_u64(&mut external_header, 40, 0x1e_dea0);
    put_u16(&mut external_header, 58, 64);
    put_u16(&mut external_header, 60, 23);
    put_u16(&mut external_header, 62, 22);
    validate_x86_64_static_user_image(&external_header).unwrap();

    let rejected = |mutate: fn(&mut Vec<u8>)| {
        let mut malformed = image.clone();
        mutate(&mut malformed);
        assert!(validate_x86_64_static_user_image(&malformed).is_err());
    };
    rejected(|candidate| {
        candidate.pop();
    });
    rejected(|candidate| put_u16(candidate, 56, 10));
    rejected(|candidate| put_u64(candidate, 24, 0x004a_6581));
    rejected(|candidate| put_u64(candidate, 64 + 2 * 56 + 32, 0x5_703f));
    rejected(|candidate| put_u64(candidate, 64 + 7 * 56 + 40, 0x81));
    rejected(|candidate| put_u64(candidate, 64 + 10 * 56 + 40, 0x799f));
    rejected(|candidate| put_u64(candidate, 40, 0x1e_dea0));
    rejected(|candidate| {
        put_u64(candidate, 40, 0x1e_dea0);
        put_u16(candidate, 58, 64);
        put_u16(candidate, 60, 24);
        put_u16(candidate, 62, 22);
    });
}

#[test]
fn realizes_the_pinned_static_elf_as_ordered_wx_safe_pages() {
    // TOPAL-SYSTEMS-STATIC-ELF-PAGE-PLAN-001, TOPAL-COMP-SYSTEMS-ARTIFACT-001.
    let load = derive_x86_64_static_elf_load_plan(&busybox_shaped_static_elf()).unwrap();
    let plan = derive_x86_64_static_elf_page_plan(&load).unwrap();
    assert_eq!(plan.frame_count, 503);
    assert_eq!(plan.alignment_frames, 1);
    assert_eq!(plan.pages.len(), 503);
    assert_eq!(plan.pages.first().unwrap().virtual_address, 0x0040_0000);
    assert_eq!(plan.pages.last().unwrap().virtual_address, 0x005f_6000);
    assert!(plan.pages.iter().enumerate().all(|(index, page)| {
        page.frame_ordinal == u64::try_from(index).unwrap()
            && page.initial_permission == StaticElfPagePermission::ReadWriteNoExecute
    }));
    assert_eq!(
        plan.pages
            .iter()
            .filter(|page| { page.entry_permission == StaticElfPagePermission::ReadOnlyExecute })
            .count(),
        396
    );
    assert_eq!(
        plan.pages
            .iter()
            .filter(|page| { page.entry_permission == StaticElfPagePermission::ReadOnlyNoExecute })
            .count(),
        89
    );
    assert_eq!(
        plan.pages
            .iter()
            .filter(|page| { page.entry_permission == StaticElfPagePermission::ReadWriteNoExecute })
            .count(),
        18
    );
    assert_eq!(
        plan.construction.first(),
        Some(&StaticElfConstructionOperation::Zero(StaticElfInterval {
            start: 0x0040_0000,
            end: 0x005f_7000,
        }))
    );
    assert_eq!(plan.construction.len(), 5);
    assert!(
        plan.construction[1..]
            .iter()
            .all(|operation| matches!(operation, StaticElfConstructionOperation::Copy(_)))
    );
    assert_eq!(
        plan.construction.last(),
        Some(&StaticElfConstructionOperation::Copy(
            StaticElfCopyOperation {
                file: StaticElfInterval {
                    start: 0x1e_4660,
                    end: 0x1e_dd88,
                },
                destination: StaticElfInterval {
                    start: 0x005e_5660,
                    end: 0x005e_ed88,
                },
            }
        ))
    );
}

#[test]
fn rejects_inconsistent_inherited_static_elf_page_facts() {
    // TOPAL-SYSTEMS-STATIC-ELF-PAGE-PLAN-001, TOPAL-COMP-SYSTEMS-TEST-001.
    let valid = derive_x86_64_static_elf_load_plan(&busybox_shaped_static_elf()).unwrap();
    let rejected = |mutate: fn(&mut StaticElfLoadPlan)| {
        let mut load = valid.clone();
        mutate(&mut load);
        assert!(derive_x86_64_static_elf_page_plan(&load).is_err());
    };

    rejected(|load| load.page_count = 502);
    rejected(|load| load.pages.end += STATIC_ELF_PAGE_SIZE);
    rejected(|load| load.segments[1].pages.start += STATIC_ELF_PAGE_SIZE);
    rejected(|load| load.segments[2].role = StaticElfLoadRole::WritableData);
    rejected(|load| load.segments[3].file.end = load.segments[3].memory.end + 1);
    rejected(|load| {
        load.relro_pages = Some(StaticElfInterval {
            start: load.segments[2].pages.start,
            end: load.segments[2].pages.end,
        });
    });
}

#[test]
fn rejects_malformed_bounded_static_elf_plans_before_publication() {
    // TOPAL-SYSTEMS-STATIC-ELF-PLAN-001, TOPAL-COMP-SYSTEMS-TEST-001.
    let valid = busybox_shaped_static_elf();
    let rejected = |mutate: fn(&mut [u8])| {
        let mut image = valid.clone();
        mutate(&mut image);
        assert!(derive_x86_64_static_elf_load_plan(&image).is_err());
    };

    rejected(|image| image[4] = 1);
    rejected(|image| put_u16(image, 56, 17));
    rejected(|image| put_u32(image, 64 + 4 * 56, 3));
    rejected(|image| put_u32(image, 64 + 4 * 56, 2));
    rejected(|image| put_u32(image, 64 + 56 + 4, 7));
    rejected(|image| put_u64(image, 64 + 3 * 56 + 32, 0x2_0000));
    rejected(|image| put_u64(image, 64 + 2 * 56 + 48, 0x3000));
    rejected(|image| put_u64(image, 64 + 2 * 56 + 8, 0x18_d001));
    rejected(|image| put_u64(image, 64 + 8, 0x1e_e000));
    rejected(|image| put_u64(image, 64 + 2 * 56 + 16, 0x0058_c000));
    rejected(|image| put_u64(image, 24, 0x0040_0000));
    rejected(|image| put_u32(image, 64 + 9 * 56 + 4, 7));
    rejected(|image| put_u32(image, 64 + 6 * 56, 7));
    rejected(|image| put_u64(image, 64 + 7 * 56 + 16, 0x0040_0000));
    rejected(|image| {
        put_u32(image, 64 + 6 * 56, 0x6474_e552);
        put_u64(image, 64 + 6 * 56 + 16, 0x005e_5000);
        put_u64(image, 64 + 6 * 56 + 40, 0x1000);
    });
    rejected(|image| put_u64(image, 64 + 10 * 56 + 16, 0x0040_0000));
    rejected(|image| put_u64(image, 64 + 3 * 56 + 40, 0x1_a9a1));
    rejected(|image| put_u32(image, 64 + 9 * 56, 4));
}

#[test]
#[allow(clippy::too_many_lines)] // Root dependencies, relocations, and access bytes form one structural audit.
fn generated_root_has_only_typed_provider_dependencies() {
    // TOPAL-COMP-SYSTEMS-X64-001, TOPAL-SYSTEMS-MACHINE-001.
    let root = generate_root_object(&program()).unwrap();
    let file = object::File::parse(root.as_slice()).unwrap();
    let undefined = file
        .symbols()
        .filter(object::ObjectSymbol::is_undefined)
        .map(|symbol| symbol.name().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_root_dependencies(&undefined);
    let text = file.section_by_name(X86_SYSTEMS_ROOT_TEXT_SECTION).unwrap();
    let root_bytes = text.data().unwrap();
    assert!(
        root_bytes
            .windows(5)
            .any(|bytes| bytes == [0xba, 0xf7, 0x01, 0x00, 0x00]),
        "checked root must request the exact 503-frame static ELF extent"
    );
    assert!(
        root_bytes
            .windows(5)
            .any(|bytes| bytes == [0xba, 0x04, 0x00, 0x00, 0x00]),
        "checked root must request the exact five-frame static ELF hierarchy extent"
    );
    assert!(
        root_bytes
            .windows(7)
            .any(|bytes| bytes == [0x4c, 0x8d, 0x83, 0x00, 0x70, 0x1f, 0x00]),
        "checked root must pass the complete 503-frame payload exclusion"
    );
    assert!(
        root_bytes
            .windows(3)
            .any(|bytes| bytes == [0x49, 0x89, 0xde])
            && root_bytes
                .windows(3)
                .any(|bytes| bytes == [0x4c, 0x89, 0xf3]),
        "checked root must retain and restore the exact payload owner"
    );
    assert!(
        root_bytes
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x8d, 0xbb, 0x00, 0x10, 0x00, 0x00])
            && root_bytes
                .windows(3)
                .any(|bytes| bytes == [0x49, 0x89, 0xdc])
            && root_bytes
                .windows(3)
                .any(|bytes| bytes == [0x4c, 0x89, 0xe3]),
        "checked root must place the hierarchy after the leaf table and preserve its owner"
    );
    assert!(
        root_bytes
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x8d, 0xbb, 0x00, 0x50, 0x00, 0x00])
            && root_bytes
                .windows(3)
                .any(|bytes| bytes == [0x48, 0x89, 0xdd])
            && root_bytes
                .windows(9)
                .any(|bytes| { bytes == [0x48, 0x31, 0xdb, 0x48, 0x89, 0xeb, 0x48, 0x31, 0xed] }),
        "checked root must allocate stack backing after the hierarchy, clear it, and restore the hierarchy owner"
    );
    assert!(
        root_bytes
            .windows(3)
            .filter(|bytes| *bytes == [0x49, 0x89, 0xd9])
            .count()
            >= 2,
        "checked root must pass the opaque backing identity to both stack-leaf transitions"
    );
    let relocation_targets = text
        .relocations()
        .map(|(_, relocation)| {
            let object::RelocationTarget::Symbol(symbol) = relocation.target() else {
                panic!("root relocations target only provider symbols");
            };
            file.symbol_by_index(symbol).unwrap().name().unwrap()
        })
        .collect::<Vec<_>>();
    // Process time adds one borrow of the root's private clock state;
    // ordinary source clock/deadline operation counts stay unchanged.
    assert_eq!(relocation_targets.len(), 9487);
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_BOOT_MEMORY_SYMBOL)
            .count(),
        1
    );
    for target in [
        X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL,
        X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL,
        X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL,
    ] {
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|actual| **actual == target)
                .count(),
            1
        );
    }
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL)
            .count(),
        2
    );
    for target in [
        X86_SYSTEMS_DEADLINE_AFTER_SYMBOL,
        X86_SYSTEMS_DEADLINE_ARM_SYMBOL,
        X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL,
    ] {
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|actual| **actual == target)
                .count(),
            1
        );
    }
    for (target, expected) in [
        (X86_SYSTEMS_CONTEXT_CREATE_SYMBOL, 3),
        (X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL, 6),
        (X86_SYSTEMS_CONTEXT_AWAIT_DEADLINE_PREEMPTION_SYMBOL, 1),
        (X86_SYSTEMS_CONTEXT_PREEMPT_CURRENT_SYMBOL, 1),
        (X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL, 3),
        (X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL, 3),
    ] {
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|actual| **actual == target)
                .count(),
            expected
        );
    }
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_KERNEL_THREAD_ENTRY)
            .count(),
        1
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_COMPLETION_THREAD_ENTRY)
            .count(),
        1
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_TERMINAL_THREAD_ENTRY)
            .count(),
        1
    );
    for target in [
        X86_SYSTEMS_ATOMIC_CREATE_SYMBOL,
        X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
        X86_SYSTEMS_ATOMIC_LOAD_SYMBOL,
    ] {
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|actual| **actual == target)
                .count(),
            1
        );
    }
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL)
            .count(),
        12
    );
    for target in [
        X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_ZERO_STACK_BACKING_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_PUBLISH_STACK_LEAF_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_INITIALIZE_STACK_ENTRY_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PAGESZ_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_ENTRY_FACT_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_ENTRY_FACT_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PAGESZ_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_CLEAR_STACK_ENTRY_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_LEAF_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_COPY_PAYLOAD_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_SEAL_PERMISSIONS_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_ATTACH_ADDRESS_SPACE_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_ACTIVATE_ADDRESS_SPACE_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_RESTORE_ADDRESS_SPACE_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTRY_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_SLOT_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_SLOT_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTRY_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_DETACH_ADDRESS_SPACE_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL,
        X86_SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY_SYMBOL,
    ] {
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|actual| **actual == target)
                .count(),
            1
        );
    }
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL)
            .count(),
        0
    );
    for target in [
        X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL,
        X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL,
        X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL,
    ] {
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|actual| **actual == target)
                .count(),
            2
        );
    }
    for target in [
        X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL,
        X86_SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE_SYMBOL,
        X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL,
        X86_SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE_SYMBOL,
        X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL,
        X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL,
    ] {
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|actual| **actual == target)
                .count(),
            1
        );
    }
    assert_translation_relocations(&relocation_targets);
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_CRITICAL_ENTER_SYMBOL)
            .count(),
        1
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL)
            .count(),
        1
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == "topal_x86_systems_uart16550_write")
            .count(),
        6858
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_PCI_CONFIGURATION_READ_DWORD_SYMBOL)
            .count(),
        48
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD_SYMBOL)
            .count(),
        25
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_CREATE_SYMBOL)
            .count(),
        24
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_BIND_REFERENCE_SYMBOL)
            .count(),
        11
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_STORE_INTEGER_SYMBOL)
            .count(),
        143
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_PREPARE_DEVICE_SYMBOL)
            .count(),
        20
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_COMPLETE_CPU_SYMBOL)
            .count(),
        20
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL)
            .count(),
        127
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_ENTROPY_SAMPLE_CAPTURE_SYMBOL)
            .count(),
        6
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_ENTROPY_POOL_MIX_SYMBOL)
            .count(),
        4
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE_SYMBOL)
            .count(),
        27
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_RAW_ENTROPY_SAMPLE_SYMBOL)
            .count(),
        15
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_ENTROPY_POOL_STATE_SYMBOL)
            .count(),
        10
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_ENTROPY_GENERATOR_KEY_SYMBOL)
            .count(),
        11
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_ENTROPY_USER_RANDOM_SERVICE_SYMBOL)
            .count(),
        1
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_ENTROPY_OUTPUT_SYMBOL)
            .count(),
        9
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_DMA_VIRTQUEUE_CONTROL_SYMBOL)
            .count(),
        328
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_DMA_ENTROPY_PAYLOAD_SYMBOL)
            .count(),
        37
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_PCI_REGION_READ_SYMBOL)
            .count(),
        231
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL)
            .count(),
        7
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == X86_SYSTEMS_PCI_REGION_WRITE_SYMBOL)
            .count(),
        207
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == "topal_bootstrap_storage")
            .count(),
        62
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == "topal_x86_systems_fatal")
            .count(),
        1096
    );
    assert_eq!(
        relocation_targets
            .iter()
            .filter(|target| **target == "topal_x86_systems_interrupt_return")
            .count(),
        4
    );
    let root = file.symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY).unwrap();
    let root_section = file
        .section_by_index(root.section_index().unwrap())
        .unwrap();
    let data = root_section.data().unwrap();
    let start = usize::try_from(root.address() - root_section.address()).unwrap();
    let end = start + usize::try_from(root.size()).unwrap();
    let root_bytes = &data[start..end];
    assert!(
        root_bytes
            .windows(7)
            .any(|bytes| bytes[..2] == [0xc6, 0x05] && bytes[6] == 90),
        "root must retain the real byte store"
    );
    assert!(
        root_bytes
            .windows(7)
            .any(|bytes| bytes == [0xc6, 0x83, 0, 0, 0, 0, 165]),
        "root must store through the selected physical frame mapping"
    );
    assert!(
        root_bytes.windows(11).any(|bytes| {
            bytes[..7] == [0x0f, 0xb6, 0x83, 0, 0, 0, 0] && bytes[7..11] == [0x3c, 165, 0x0f, 0x85]
        }),
        "root must load through the selected physical frame mapping"
    );
    assert!(
        root_bytes
            .windows(8)
            .any(|bytes| bytes == [0x41, 0xc6, 0x86, 0, 0, 0, 0, 60]),
        "root must store through the committed active translation mapping"
    );
    assert!(
        root_bytes.windows(12).any(|bytes| {
            bytes[..8] == [0x41, 0x0f, 0xb6, 0x86, 0, 0, 0, 0]
                && bytes[8..12] == [0x3c, 60, 0x0f, 0x85]
        }),
        "root must load through the committed active translation mapping"
    );
    assert!(
        root_bytes.windows(15).any(|bytes| {
            bytes[..3] == [0x0f, 0xb6, 0x05] && bytes[7..11] == [0x3c, 90, 0x0f, 0x85]
        }),
        "root must retain the real byte load and mismatch branch"
    );
    assert!(
        root_bytes
            .windows(9)
            .any(|bytes| bytes == [0x31, 0xc0, 0x48, 0x89, 0x07, 0x48, 0x89, 0x47, 8]),
        "root must overwrite the provider-private raw entropy sample"
    );
    assert!(
        root_bytes
            .windows(6)
            .any(|bytes| bytes == [0x48, 0x83, 0x3f, 0, 0x0f, 0x85]),
        "root must verify raw entropy-sample erasure and fail closed"
    );
    assert_eq!(
        root_bytes
            .windows(7)
            .filter(|bytes| bytes[..3] == [0x48, 0x8d, 0x3d])
            .count(),
        395,
        "root must derive opaque storage for DMA, atomic, notification, clock, deadline, context, and user operations"
    );
    assert!(
        root_bytes
            .windows(4)
            .any(|bytes| bytes == [0x48, 0x8b, 0x4f, 0x68]),
        "root must pass the prepared user backing only as a provider-private exclusion"
    );
    assert!(
        root_bytes
            .windows(4)
            .any(|bytes| bytes == [0x48, 0x83, 0xf8, 0x2a]),
        "root must retain the acquire-load confirmation"
    );
    assert!(file.symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY).is_some());
    assert!(file.symbol_by_name(X86_SYSTEMS_DEBUG_BREAK_ENTRY).is_some());
    assert!(
        file.symbol_by_name(X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY)
            .is_some()
    );
    assert!(file.symbol_by_name(X86_SYSTEMS_DEADLINE_ENTRY).is_some());
    assert!(
        file.symbol_by_name(X86_SYSTEMS_KERNEL_THREAD_ENTRY)
            .is_some()
    );
    assert!(
        file.symbol_by_name(X86_SYSTEMS_COMPLETION_THREAD_ENTRY)
            .is_some()
    );
    assert!(
        file.symbol_by_name(X86_SYSTEMS_TERMINAL_THREAD_ENTRY)
            .is_some()
    );
}

fn assert_translation_relocations(relocation_targets: &[&str]) {
    for target in [
        X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
        X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
        X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
    ] {
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|actual| **actual == target)
                .count(),
            1
        );
    }
    for (target, expected) in [
        (X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL, 2),
        (X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL, 2),
    ] {
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|actual| **actual == target)
                .count(),
            expected
        );
    }
}

fn assert_root_dependencies(undefined: &[String]) {
    assert_eq!(
        undefined,
        [
            "topal_x86_systems_describe_boot_memory",
            "topal_x86_systems_allocate_physical_frames",
            "topal_x86_systems_static_image_prepare_initial_leaves",
            "topal_x86_systems_static_image_zero_payload",
            "topal_x86_systems_static_image_zero_stack_backing",
            "topal_x86_systems_static_image_publish_stack_leaf",
            "topal_x86_systems_static_image_initialize_stack_entry",
            "topal_x86_systems_static_image_add_stack_pagesz",
            "topal_x86_systems_static_image_add_stack_entry_fact",
            "topal_x86_systems_static_image_add_stack_program_headers",
            "topal_x86_systems_static_image_add_stack_executable_name",
            "topal_x86_systems_static_image_add_stack_credentials",
            "topal_x86_systems_static_image_add_stack_secure_mode",
            "topal_x86_systems_static_image_add_stack_clock_tick_rate",
            "topal_x86_systems_static_image_add_stack_platform",
            "topal_x86_systems_static_image_add_stack_hardware_capabilities",
            "topal_x86_systems_static_image_add_stack_load_metadata",
            "topal_x86_systems_static_image_authorize_stack_random_slot",
            "topal_x86_systems_static_image_retain_prepared_process",
            "topal_x86_systems_static_image_qualify_retained_disjoint_work",
            "topal_x86_systems_static_image_authorize_entropy_handoff",
            "topal_x86_systems_static_image_authorize_entropy_acquisition",
            "topal_x86_systems_static_image_qualify_entropy_device",
            "topal_x86_systems_static_image_qualify_entropy_transport",
            "topal_x86_systems_static_image_qualify_entropy_negotiation",
            "topal_x86_systems_static_image_qualify_entropy_queue",
            "topal_x86_systems_static_image_qualify_entropy_bus_master",
            "topal_x86_systems_static_image_qualify_entropy_dma_buffers",
            "topal_x86_systems_static_image_qualify_entropy_split_ring",
            "topal_x86_systems_static_image_qualify_entropy_dma_preparation",
            "topal_x86_systems_static_image_qualify_entropy_queue_addresses",
            "topal_x86_systems_static_image_qualify_entropy_queue_activation",
            "topal_x86_systems_static_image_qualify_entropy_request_completion",
            "topal_x86_systems_static_image_qualify_entropy_sample_capture",
            "topal_x86_systems_static_image_qualify_entropy_sample_quality",
            "topal_x86_systems_static_image_qualify_entropy_pool_lifecycle",
            "topal_x86_systems_static_image_qualify_entropy_generator_lifecycle",
            "topal_x86_systems_static_image_deliver_entropy_initial_random",
            "topal_x86_systems_static_image_process_enter",
            "topal_x86_systems_static_image_process_restore_entry",
            "topal_x86_systems_static_image_revoke_entropy_initial_random",
            "topal_x86_systems_static_image_revoke_entropy_acquisition",
            "topal_x86_systems_static_image_revoke_entropy_handoff",
            "topal_x86_systems_static_image_restore_prepared_process",
            "topal_x86_systems_static_image_revoke_stack_random_slot",
            "topal_x86_systems_static_image_remove_stack_load_metadata",
            "topal_x86_systems_static_image_remove_stack_hardware_capabilities",
            "topal_x86_systems_static_image_remove_stack_platform",
            "topal_x86_systems_static_image_remove_stack_clock_tick_rate",
            "topal_x86_systems_static_image_remove_stack_secure_mode",
            "topal_x86_systems_static_image_remove_stack_credentials",
            "topal_x86_systems_static_image_remove_stack_executable_name",
            "topal_x86_systems_static_image_remove_stack_program_headers",
            "topal_x86_systems_static_image_remove_stack_entry_fact",
            "topal_x86_systems_static_image_remove_stack_pagesz",
            "topal_x86_systems_static_image_clear_stack_entry",
            "topal_x86_systems_static_image_revoke_stack_leaf",
            "topal_x86_systems_static_image_copy_payload",
            "topal_x86_systems_static_image_seal_permissions",
            "topal_x86_systems_static_image_attach_address_space",
            "topal_x86_systems_static_image_activate_address_space",
            "topal_x86_systems_static_image_restore_address_space",
            "topal_x86_systems_static_image_authorize_entry",
            "topal_x86_systems_static_image_authorize_stack_slot",
            "topal_x86_systems_static_image_revoke_stack_slot",
            "topal_x86_systems_static_image_revoke_entry",
            "topal_x86_systems_static_image_detach_address_space",
            "topal_x86_systems_static_image_revoke_initial_leaves",
            "topal_x86_systems_begin_bootstrap_translation",
            "topal_x86_systems_commit_bootstrap_translation",
            "topal_x86_systems_activate_bootstrap_translation",
            "topal_x86_systems_begin_active_translation_edit",
            "topal_x86_systems_stage_active_translation_map",
            "topal_x86_systems_stage_active_translation_unmap",
            "topal_x86_systems_commit_active_translation_edit",
            "topal_x86_systems_enter_local_interrupt_critical",
            "topal_x86_systems_restore_local_interrupt_critical",
            "topal_x86_systems_atomic_word_create",
            "topal_x86_systems_atomic_word_compare_exchange",
            "topal_x86_systems_atomic_word_load_acquire",
            "topal_x86_systems_local_notification_send",
            "topal_x86_systems_local_notification_wait",
            "topal_x86_systems_local_notification_complete",
            "topal_x86_systems_monotonic_clock_now",
            "topal_x86_systems_pci_configuration_read_dword_equals",
            "topal_x86_systems_pci_configuration_update_word_masked",
            "topal_x86_systems_dma_buffer_create",
            "topal_x86_systems_dma_buffer_bind_reference",
            "topal_x86_systems_dma_buffer_store_integer",
            "topal_x86_systems_dma_buffer_prepare_device",
            "topal_x86_systems_dma_buffer_complete_cpu",
            "topal_x86_systems_dma_buffer_load_integer_equals",
            "topal_x86_systems_entropy_capture_dma_sample",
            "topal_x86_systems_entropy_mix_blake2s_pool_sample",
            "topal_x86_systems_entropy_initialize_chacha20_generator",
            "topal_x86_systems_entropy_extract_chacha20_generator",
            "topal_x86_systems_entropy_deliver_initial_auxv_random",
            "topal_x86_systems_entropy_reserve_initial_user_random",
            "topal_x86_systems_entropy_bind_reusable_user_random",
            "topal_x86_systems_pci_region_configuration_gateway_write_dma_reference",
            "topal_x86_systems_dma_virtqueue_control",
            "topal_x86_systems_dma_entropy_payload",
            "topal_x86_systems_raw_entropy_sample",
            "topal_x86_systems_entropy_pool_state",
            "topal_x86_systems_entropy_generator_key",
            "topal_x86_systems_entropy_output",
            "topal_x86_systems_entropy_user_random_service",
            "topal_x86_static_process_brk_reserve",
            "topal_x86_static_process_exec_snapshot",
            "topal_x86_static_process_descriptor_table",
            "topal_x86_systems_pci_region_configuration_gateway_read_equals",
            "topal_x86_systems_pci_region_configuration_gateway_read_eventually_equals",
            "topal_x86_systems_pci_region_configuration_gateway_write",
            "topal_x86_systems_deadline_after",
            "topal_x86_systems_deadline_arm",
            "topal_x86_systems_deadline_wait",
            "topal_x86_systems_deadline_complete",
            "topal_x86_systems_context_create",
            "topal_x86_systems_context_transfer",
            "topal_x86_systems_context_await_deadline_preemption",
            "topal_x86_systems_context_preempt_current",
            "topal_x86_systems_context_retire",
            "topal_x86_systems_context_reclaim",
            "topal_x86_systems_user_prepare_initial_address_space",
            "topal_x86_systems_user_verify_initial_mapping",
            "topal_x86_systems_user_map_initial_frame",
            "topal_x86_systems_user_store_initial_mapping_byte",
            "topal_x86_systems_user_install_initial_entry_trampoline",
            "topal_x86_systems_user_protect_initial_mapping_read_only",
            "topal_x86_systems_user_load_initial_mapping_byte",
            "topal_x86_systems_user_verify_initial_entry_trampoline",
            "topal_x86_systems_user_permit_initial_mapping_execute",
            "topal_x86_systems_user_seal_initial_mapping_immutable_data",
            "topal_x86_systems_user_permit_initial_mapping_writable_data",
            "topal_x86_systems_user_reserve_initial_anonymous_mapping",
            "topal_x86_systems_user_release_initial_anonymous_mapping",
            "topal_x86_systems_user_unmap_initial_frame",
            "topal_x86_systems_user_prepare_initial_probe",
            "topal_x86_systems_user_complete_initial_backing",
            "topal_x86_systems_user_reclaim_initial_address_space",
            "topal_x86_systems_user_enter_initial_context",
            "topal_x86_systems_user_dispatch_initial_syscall",
            "topal_x86_systems_uart16550_write",
            "topal_x86_systems_debug_break",
            "topal_x86_systems_interrupt_return",
            "topal_x86_systems_fatal",
            "topal_bootstrap_storage",
        ]
    );
}

#[test]
fn supplied_initial_user_image_validation_is_bounded_and_payload_neutral() {
    // TOPAL-COMP-SYSTEMS-ARTIFACT-001.
    let canonical = initial_user_elf_file().unwrap();
    let mut supplied = canonical.clone();
    supplied[INITIAL_USER_DATA_FILE_OFFSET + 1] = 0xa5;
    validate_x86_64_initial_user_image(&supplied).unwrap();

    let mut wrong_size = supplied.clone();
    wrong_size.pop();
    assert!(validate_x86_64_initial_user_image(&wrong_size).is_err());

    let mut wrong_metadata = supplied.clone();
    wrong_metadata[0] = 0;
    assert!(validate_x86_64_initial_user_image(&wrong_metadata).is_err());

    let mut wrong_code_extent = supplied.clone();
    wrong_code_extent
        [INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET..INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET + 8]
        .copy_from_slice(&4096_u64.to_le_bytes());
    assert!(validate_x86_64_initial_user_image(&wrong_code_extent).is_err());

    let mut wrong_entry = supplied.clone();
    wrong_entry[INITIAL_USER_ENTRY_ELF_OFFSET..INITIAL_USER_ENTRY_ELF_OFFSET + 8]
        .copy_from_slice(&(INITIAL_USER_CODE_ADDRESS + 4096 + 256 - 1).to_le_bytes());
    assert!(validate_x86_64_initial_user_image(&wrong_entry).is_err());

    let mut wrong_data_extent = supplied;
    wrong_data_extent
        [INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET..INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET + 8]
        .copy_from_slice(&24_u64.to_le_bytes());
    wrong_data_extent
        [INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET..INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET + 8]
        .copy_from_slice(&16_u64.to_le_bytes());
    assert!(validate_x86_64_initial_user_image(&wrong_data_extent).is_err());
}
