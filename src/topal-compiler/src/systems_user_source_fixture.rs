//! Source-built ordinary userspace requests, TOPAL-SYSTEMS-USER-EXTENT-001.
//! The source object and its backend ABI adapter are linked before the
//! first-party ABI witness is constructed. No completed application is patched.

use crate::systems_tail_owner::run_tail_native_fixture as run;
use object::{Object as _, ObjectSection, ObjectSymbol};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

const BASE: u64 = 0x570000;
const PREDECESSOR_SHA256: &str = "41a3dcd7b7e42a2f77ad995e6259e43ef84f7abf95e2dfd36cff7bea187c9fda";

fn syscall_adapter(target_name: &[u8], image_entry: bool) -> Vec<u8> {
    use object::write::{Object, Symbol, SymbolSection};
    use object::{
        Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
    };
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let section = object.add_section(
        vec![],
        b".text.topal_test_syscall3".to_vec(),
        SectionKind::Text,
    );
    // System V (number, a, b, c) -> Linux (RAX, RDI, RSI, RDX).
    // SYSCALL's RCX/R11 destruction agrees with the source calling convention.
    let code = [
        0x48, 0x89, 0xf8, 0x48, 0x89, 0xf7, 0x48, 0x89, 0xd6, 0x48, 0x89, 0xca, 0x0f, 0x05, 0xc3,
    ];
    object.append_section_data(section, &code, 16);
    object.add_symbol(Symbol {
        name: b"topal_test_syscall3".to_vec(),
        value: 0,
        size: code.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
    let fourth = object.add_section(
        vec![],
        b".text.topal_test_syscall4".to_vec(),
        SectionKind::Text,
    );
    let mut four_code = vec![0x4d, 0x89, 0xc2]; // fourth argument -> Linux R10
    four_code.extend_from_slice(&code);
    object.append_section_data(fourth, &four_code, 16);
    object.add_symbol(Symbol {
        name: b"topal_test_syscall4".to_vec(),
        value: 0,
        size: four_code.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(fourth),
        flags: SymbolFlags::None,
    });
    let startup = object.add_section(
        vec![],
        b".text.topal_test_startup_call".to_vec(),
        SectionKind::Text,
    );
    let mut bridge = vec![
        0x57, 0x56, 0x52, 0x41, 0x52, 0x41, 0x50, 0x41, 0x51, 0x41, 0x54, 0x49, 0x89, 0xe4, 0x48,
        0x83, 0xe4, 0xf0, // save/alignment
        0x48, 0x89, 0xc7, // original syscall number -> System V first arg
        0x49, 0x8b, 0x74, 0x24, 48, 0x49, 0x8b, 0x54, 0x24, 40, 0x49, 0x8b, 0x4c, 0x24, 32, 0x4d,
        0x8b, 0x44, 0x24, 24, 0xe8, 0, 0, 0,
        0, // ordinary link relocation to the source function
    ];
    let call_offset = bridge.len() as u64 - 4;
    bridge.extend_from_slice(&[
        0x4c, 0x89, 0xe4, 0x41, 0x5c, 0x41, 0x59, 0x41, 0x58, 0x41, 0x5a, 0x5a, 0x5e, 0x5f, 0xc3,
    ]);
    object.append_section_data(startup, &bridge, 16);
    object.add_symbol(Symbol {
        name: b"topal_test_startup_call".to_vec(),
        value: 0,
        size: bridge.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(startup),
        flags: SymbolFlags::None,
    });
    let target = object.add_symbol(Symbol {
        name: target_name.to_vec(),
        value: 0,
        size: 0,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Undefined,
        flags: SymbolFlags::None,
    });
    object
        .add_relocation(
            startup,
            object::write::Relocation {
                offset: call_offset,
                symbol: target,
                addend: -4,
                flags: object::RelocationFlags::Generic {
                    kind: object::RelocationKind::Relative,
                    encoding: object::RelocationEncoding::X86Branch,
                    size: 32,
                },
            },
        )
        .unwrap();
    if image_entry {
        append_image_entry(&mut object);
    }
    object.write().unwrap()
}

fn append_image_entry(object: &mut object::write::Object<'_>) {
    use object::write::{Symbol, SymbolSection};
    use object::{
        RelocationEncoding, RelocationFlags, RelocationKind, SectionKind, SymbolFlags, SymbolKind,
        SymbolScope,
    };
    let section = object.add_section(
        vec![],
        b".text.topal_test_image_entry".to_vec(),
        SectionKind::Text,
    );
    let mut bytes = vec![
        0x48, 0x81, 0xec, 0x00, 0x04, 0, 0, // scratch below retained initial payloads
        0x50, 0x53, 0x51, 0x52, 0x56, 0x57, 0x55, 0x41, 0x50, 0x41, 0x51, 0x41, 0x52, 0x41, 0x53,
        0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57, 0x49, 0x89,
        0xe4, // saved caller stack in callee-preserved R12
        0x48, 0x8d, 0xbc, 0x24, 0x78, 0x04, 0, 0, // original SP: scratch + saved registers
        0x48, 0x83, 0xe4, 0xf0, 0xe8, 0, 0, 0, 0,
    ];
    let call = bytes.len() as u64 - 4;
    bytes.extend_from_slice(&[
        0x4c, 0x89, 0xe4, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x41, 0x5b, 0x41, 0x5a,
        0x41, 0x59, 0x41, 0x58, 0x5d, 0x5f, 0x5e, 0x5a, 0x59, 0x5b, 0x58, 0x48, 0x81, 0xc4, 0x00,
        0x04, 0, 0, 0xe9, 0, 0, 0, 0,
    ]);
    let jump = bytes.len() as u64 - 4;
    object.append_section_data(section, &bytes, 16);
    object.add_symbol(Symbol {
        name: b"topal_test_image_entry".to_vec(),
        value: 0,
        size: bytes.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
    for (offset, name, value, symbol_section) in [
        (
            call,
            b"topal_process_exec_entry".as_slice(),
            0,
            SymbolSection::Undefined,
        ),
        (
            jump,
            b"topal_test_fixture_start".as_slice(),
            0x4a6100,
            SymbolSection::Absolute,
        ),
    ] {
        let symbol = object.add_symbol(Symbol {
            name: name.to_vec(),
            value,
            size: 0,
            kind: SymbolKind::Text,
            scope: SymbolScope::Linkage,
            weak: false,
            section: symbol_section,
            flags: SymbolFlags::None,
        });
        object
            .add_relocation(
                section,
                object::write::Relocation {
                    offset,
                    symbol,
                    addend: -4,
                    flags: RelocationFlags::Generic {
                        kind: RelocationKind::Relative,
                        encoding: RelocationEncoding::X86Branch,
                        size: 32,
                    },
                },
            )
            .unwrap();
    }
}

pub(crate) fn build(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "transfers")
}

pub(crate) fn build_exec(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "exec")
}

pub(crate) fn build_signals(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "signals")
}

pub(crate) fn build_poll(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "poll")
}

pub(crate) fn build_pipes(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "pipes")
}

pub(crate) fn build_storage(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "storage")
}

pub(crate) fn build_metadata(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "metadata")
}

pub(crate) fn build_process_memory(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "process-mm")
}

pub(crate) fn build_process_memory_exec(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "process-mm-exec")
}

pub(crate) fn build_process_memory_allocation(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "process-mm-allocation")
}

pub(crate) fn build_process_memory_cancel(
    work: &Path,
    tools: &crate::LlvmTools,
    early: bool,
    group: bool,
    writer: bool,
) -> Vec<u8> {
    build_configured(
        work,
        tools,
        "process-mm-cancel",
        Some((early, group, writer)),
    )
}

pub(crate) fn build_serial(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "serial")
}

pub(crate) fn build_stat(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    build_selected(work, tools, "stat")
}

fn build_selected(work: &Path, tools: &crate::LlvmTools, mode: &str) -> Vec<u8> {
    build_configured(work, tools, mode, None)
}

fn build_configured(
    work: &Path,
    tools: &crate::LlvmTools,
    mode: &str,
    cancellation: Option<(bool, bool, bool)>,
) -> Vec<u8> {
    let image_entry_selected = matches!(
        mode,
        "process-mm-exec" | "process-mm-cancel" | "process-mm-allocation"
    );
    let cancellation_config = cancellation.map(|(early, group, writer)| {
        let status = match (early, group) {
            (false, false) => 0x13a,
            (false, true) => 0x200,
            (true, false) => 0x2c5,
            (true, true) => 0x301,
        };
        (early, if group { 231 } else { 60 }, status, writer)
    });
    let (entry_name, extra) = match mode {
        "process-mm-allocation" => (
            "topal_process_memory_allocation_cases",
            Some(("process_mm_allocation", "process-mm-allocation-cases")),
        ),
        "process-mm-cancel" => (
            "topal_process_memory_cancel_cases",
            Some(("process_mm_cancel", "process-mm-cancel-cases")),
        ),
        "process-mm-exec" => (
            "topal_process_memory_exec_cases",
            Some(("process_mm_exec", "process-mm-exec-cases")),
        ),
        "process-mm" => (
            "topal_process_memory_cases",
            Some(("process_mm", "process-mm-cases")),
        ),
        "metadata" => (
            "topal_user_extent_metadata_cases",
            Some(("metadata", "metadata-cases")),
        ),
        "stat" => ("topal_user_extent_stat_cases", Some(("stat", "stat-cases"))),
        "serial" => (
            "topal_user_extent_serial_cases",
            Some(("serial", "serial-cases")),
        ),
        "storage" => (
            "topal_user_extent_storage_cases",
            Some(("storage", "storage-cases")),
        ),
        "pipes" => ("topal_user_extent_pipe_cases", Some(("pipe", "pipe-cases"))),
        "transfers" => ("topal_user_extent_source_cases", None),
        "exec" => ("topal_user_extent_exec_cases", Some(("exec", "exec-cases"))),
        "signals" => (
            "topal_user_extent_signal_cases",
            Some(("signal", "signal-cases")),
        ),
        "poll" => (
            "topal_user_extent_source_cases",
            Some(("poll", "poll-cases")),
        ),
        _ => panic!("unknown source fixture cohort"),
    };
    let lab = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/labs/qemu/x86_64");
    let source = lab.join("user_extent_source_cases.c");
    run(
        work,
        "source-cc-version",
        Command::new("cc").arg("--version"),
    );
    let mut inputs = vec![
        ("user_extent_source_cases.c", "source-cases"),
        ("user_extent_startup_cases.c", "startup-cases"),
        ("user_extent_file_cases.c", "file-cases"),
        ("user_extent_shell_cases.c", "shell-cases"),
        ("user_extent_command_cases.c", "command-cases"),
    ];
    let extra_name = extra.map(|(name, _)| format!("user_extent_{name}_cases.c"));
    if let Some((_, stem)) = extra {
        inputs.push((extra_name.as_deref().unwrap(), stem));
    }
    for (input, stem) in &inputs {
        fs::copy(lab.join(input), work.join(format!("{stem}.c"))).unwrap();
        let defines = if *stem == "process-mm-cancel-cases" {
            let (early, syscall, status, writer) = cancellation_config.unwrap();
            vec![
                format!("-DTOPAL_CANCEL_EARLY={}", u8::from(early)),
                format!("-DTOPAL_CANCEL_EXIT_SYSCALL={syscall}"),
                format!("-DTOPAL_CANCEL_STATUS={status}"),
                format!("-DTOPAL_CANCEL_WRITER={}", u8::from(writer)),
            ]
        } else {
            vec![]
        };
        run(
            work,
            &format!("{stem}-compile"),
            Command::new("cc")
                .args([
                    "-std=c11",
                    "-O2",
                    "-g",
                    "-ffreestanding",
                    "-fno-builtin",
                    "-fno-pie",
                    "-fno-stack-protector",
                    "-fno-asynchronous-unwind-tables",
                    "-fno-unwind-tables",
                    "-mgeneral-regs-only",
                    "-mno-red-zone",
                    "-fstack-usage",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-c",
                ])
                .args(&defines)
                .arg(if mode == "transfers" && *stem == "source-cases" {
                    "-DTOPAL_PROBE_CONSOLE=1"
                } else {
                    "-DTOPAL_PROBE_CONSOLE=0"
                })
                .arg(work.join(format!("{stem}.c")))
                .arg("-o")
                .arg(work.join(format!("{stem}.o"))),
        );
    }
    let startup_name = if mode == "poll" {
        "topal_user_extent_poll_cases"
    } else {
        "topal_user_extent_startup_cases"
    };
    fs::write(
        work.join("source-syscall.o"),
        syscall_adapter(startup_name.as_bytes(), image_entry_selected),
    )
    .unwrap();
    let script = "SECTIONS { . = 0x570000; .text : { *(.text .text.*) *(.rodata .rodata.*) } \
                  /DISCARD/ : { *(.eh_frame) *(.comment) *(.note*) } }\n";
    fs::write(work.join("source-user.ld"), script).unwrap();
    run(
        work,
        "source-link",
        Command::new(tools.tool_path("rust-lld"))
            .args([
                "-flavor",
                "gnu",
                "-m",
                "elf_x86_64",
                "-static",
                "--no-dynamic-linker",
                "--build-id=none",
                "--entry",
                entry_name,
                "-z",
                "noexecstack",
                "-T",
            ])
            .arg(work.join("source-user.ld"))
            .arg(work.join("source-cases.o"))
            .arg(work.join("startup-cases.o"))
            .arg(work.join("file-cases.o"))
            .arg(work.join("shell-cases.o"))
            .arg(work.join("command-cases.o"))
            .args(extra.map(|(_, stem)| work.join(format!("{stem}.o"))))
            .arg(work.join("source-syscall.o"))
            .arg("-o")
            .arg(work.join("source-user.elf")),
    );
    let linked = fs::read(work.join("source-user.elf")).unwrap();
    let elf = object::File::parse(linked.as_slice()).unwrap();
    let text = elf.section_by_name(".text").unwrap();
    assert_eq!(text.address(), BASE);
    let bytes = text.data().unwrap();
    assert!(!bytes.is_empty() && bytes.len() <= 65536);
    assert!((BASE..BASE + bytes.len() as u64).contains(&elf.entry()));
    for section in elf.sections() {
        assert_eq!(section.relocations().count(), 0);
        if matches!(
            section.kind(),
            object::SectionKind::Data | object::SectionKind::UninitializedData
        ) {
            assert_eq!(
                section.size(),
                0,
                "fixture must not introduce writable source globals"
            );
        }
    }
    let syscall = elf.symbol_by_name("topal_test_syscall3").unwrap();
    assert_eq!(syscall.size(), 15);
    // Disjoint startup scratch plus both generated adapters/alignment/returns.
    let mut frames = std::collections::BTreeMap::new();
    for (_, stem) in &inputs {
        let mut cost = 0_u64;
        let stack = fs::read_to_string(work.join(format!("{stem}.su"))).unwrap();
        for line in stack.lines() {
            let columns = line.split('\t').collect::<Vec<_>>();
            assert_eq!(columns.len(), 3, "unexpected stack-usage row: {line}");
            assert_eq!(columns[2], "static");
            cost += columns[1].parse::<u64>().unwrap() + 8;
        }
        frames.insert(*stem, cost);
    }
    // The runtime entry cannot call the startup entry; only the latter can
    // delegate to file cases. Sum every frame on either possible call path.
    let delegated = frames["file-cases"]
        .max(frames["shell-cases"])
        .max(frames["command-cases"]);
    let source_stack = if mode == "poll" {
        frames["source-cases"].max(frames["poll-cases"])
    } else if let Some((_, stem)) = extra {
        frames[stem]
    } else {
        frames["source-cases"].max(frames["startup-cases"] + delegated)
    };
    // The alternate image-entry adapter saves all fifteen non-SP registers,
    // alignment and its return word, before calling the C entry selector.
    let adapter_stack = if image_entry_selected { 144 } else { 128 };
    let user_stack = 1024 + adapter_stack + source_stack;
    assert!(
        user_stack <= 2048,
        "source call must fit below the witness's live user stack"
    );
    let startup = elf.symbol_by_name("topal_test_startup_call").unwrap();
    let source_startup = elf.symbol_by_name(startup_name).unwrap();
    let syscall4 = elf.symbol_by_name("topal_test_syscall4").unwrap();
    let image_entry = elf
        .symbol_by_name("topal_test_image_entry")
        .map(|s| s.address());
    let image = if image_entry_selected {
        crate::systems_artifact::source_built_static_user_image_with_entry_fixture(
            u32::try_from(elf.entry()).unwrap(),
            u32::try_from(image_entry.unwrap()).unwrap(),
            bytes,
        )
    } else {
        crate::systems_artifact::source_built_static_user_image_fixture(
            u32::try_from(elf.entry()).unwrap(),
            if extra.is_some() {
                None
            } else {
                Some(u32::try_from(startup.address()).unwrap())
            },
            if mode == "poll" {
                Some(u32::try_from(startup.address()).unwrap())
            } else {
                None
            },
            bytes,
        )
    };
    let report = serde_json::json!({
        "source_sha256": format!("{:x}", Sha256::digest(fs::read(&source).unwrap())),
        "linked_sha256": format!("{:x}", Sha256::digest(&linked)),
        "image_sha256": format!("{:x}", Sha256::digest(&image)),
        "base": BASE, "entry": elf.entry(), "syscall": syscall.address(),
        "startup": source_startup.address(), "startup_call": startup.address(),
        "syscall4": syscall4.address(),
        "mode": mode,
        "image_entry": image_entry,
        "allocation_profile": if mode == "process-mm-allocation" { Some(serde_json::json!({
            "guest_ram_mib": 52,
            "process_storage_bytes": crate::systems_provider_object::PROCESS_MM_TABLE_BYTES,
            "identity_storage_bytes": crate::systems_provider_object::PROCESS_MM_IDENTITY_STORE_BYTES,
        })) } else { None },
        "process_mm_allocation_source_sha256": if mode == "process-mm-allocation" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_process_mm_allocation_cases.c")).unwrap()))) } else { None },
        "cancellation": cancellation_config.map(|(early, syscall, status, writer)| serde_json::json!({
            "early": early, "syscall": syscall, "status": status,
            "writer": writer,
            "process_storage_bytes": crate::systems_provider_object::PROCESS_MM_TABLE_BYTES,
        })),
        "process_mm_cancel_source_sha256": if mode == "process-mm-cancel" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_process_mm_cancel_cases.c")).unwrap()))) } else { None },
        "process_mm_exec_source_sha256": if mode == "process-mm-exec" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_process_mm_exec_cases.c")).unwrap()))) } else { None },
        "exec_source_sha256": if mode == "exec" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_exec_cases.c")).unwrap()))) } else { None },
        "signal_source_sha256": if mode == "signals" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_signal_cases.c")).unwrap()))) } else { None },
        "poll_source_sha256": if mode == "poll" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_poll_cases.c")).unwrap()))) } else { None },
        "pipe_source_sha256": if mode == "pipes" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_pipe_cases.c")).unwrap()))) } else { None },
        "storage_source_sha256": if mode == "storage" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_storage_cases.c")).unwrap()))) } else { None },
        "metadata_source_sha256": if mode == "metadata" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_metadata_cases.c")).unwrap()))) } else { None },
        "process_mm_source_sha256": if mode == "process-mm" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_process_mm_cases.c")).unwrap()))) } else { None },
        "serial_source_sha256": if mode == "serial" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_serial_cases.c")).unwrap()))) } else { None },
        "stat_source_sha256": if mode == "stat" { Some(format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_stat_cases.c")).unwrap()))) } else { None },
        "startup_source_sha256": format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_startup_cases.c")).unwrap())),
        "file_source_sha256": format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_file_cases.c")).unwrap())),
        "shell_source_sha256": format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_shell_cases.c")).unwrap())),
        "command_source_sha256": format!("{:x}", Sha256::digest(fs::read(lab.join("user_extent_command_cases.c")).unwrap())),
        "stack_frame_costs": frames,
        "text_bytes": bytes.len(), "conservative_user_stack_bytes": user_stack,
        "scope": "source-built ordinary syscall component in a freshly generated first-party ABI witness; not an arbitrary-program loader or production qualification",
    });
    fs::write(
        work.join("source-component.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    image
}

#[test]
fn default_abi_fixture_preserves_committed_generator_bytes() {
    // Rebuilt from the exact generator sources and namespace data in 81ae694.
    // This protects the default constructor while adding the source variant.
    let image = crate::qualified_x86_64_static_user_image_fixture();
    assert_eq!(format!("{:x}", Sha256::digest(image)), PREDECESSOR_SHA256);
}

#[test]
fn source_user_component_links_before_abi_image_construction() {
    let work = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/.cache/x86_64")
        .join(format!(
            "source-component-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&work).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let image = build(&work, &tools);
    let baseline = crate::qualified_x86_64_static_user_image_fixture();
    assert_eq!(&image[..0x1000], &baseline[..0x1000]);
    assert_eq!(image.len(), baseline.len());
    assert_ne!(image, baseline);
    fs::write(work.join("source-abi.elf"), image).unwrap();
    eprintln!("source-built component: {}", work.display());
}
