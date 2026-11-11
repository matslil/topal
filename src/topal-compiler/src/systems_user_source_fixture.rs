//! Source-built ordinary userspace requests, TOPAL-SYSTEMS-USER-EXTENT-001.
//! The source object and its backend ABI adapter are linked before the
//! first-party ABI witness is constructed. No completed application is patched.

use crate::systems_tail_owner::run_tail_native_fixture as run;
use object::{Object as _, ObjectSection, ObjectSymbol};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

const BASE: u64 = 0x570000;
const PREDECESSOR_SHA256: &str = "41a3dcd7b7e42a2f77ad995e6259e43ef84f7abf95e2dfd36cff7bea187c9fda";

fn syscall_adapter() -> Vec<u8> {
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
        name: b"topal_user_extent_startup_cases".to_vec(),
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
    object.write().unwrap()
}

pub(crate) fn build(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    let lab = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/labs/qemu/x86_64");
    let source = lab.join("user_extent_source_cases.c");
    run(
        work,
        "source-cc-version",
        Command::new("cc").arg("--version"),
    );
    for (input, stem) in [
        ("user_extent_source_cases.c", "source-cases"),
        ("user_extent_startup_cases.c", "startup-cases"),
        ("user_extent_file_cases.c", "file-cases"),
        ("user_extent_shell_cases.c", "shell-cases"),
        ("user_extent_command_cases.c", "command-cases"),
    ] {
        fs::copy(lab.join(input), work.join(format!("{stem}.c"))).unwrap();
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
                .arg(work.join(format!("{stem}.c")))
                .arg("-o")
                .arg(work.join(format!("{stem}.o"))),
        );
    }
    fs::write(work.join("source-syscall.o"), syscall_adapter()).unwrap();
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
                "topal_user_extent_source_cases",
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
    for stem in [
        "source-cases",
        "startup-cases",
        "file-cases",
        "shell-cases",
        "command-cases",
    ] {
        let mut cost = 0_u64;
        let stack = fs::read_to_string(work.join(format!("{stem}.su"))).unwrap();
        for line in stack.lines() {
            let columns = line.split('\t').collect::<Vec<_>>();
            assert_eq!(columns.len(), 3, "unexpected stack-usage row: {line}");
            assert_eq!(columns[2], "static");
            cost += columns[1].parse::<u64>().unwrap() + 8;
        }
        frames.insert(stem, cost);
    }
    // The runtime entry cannot call the startup entry; only the latter can
    // delegate to file cases. Sum every frame on either possible call path.
    let delegated = frames["file-cases"]
        .max(frames["shell-cases"])
        .max(frames["command-cases"]);
    let user_stack = 1024 + 128 + frames["source-cases"].max(frames["startup-cases"] + delegated);
    assert!(
        user_stack <= 2048,
        "source call must fit below the witness's live user stack"
    );
    let startup = elf.symbol_by_name("topal_test_startup_call").unwrap();
    let source_startup = elf
        .symbol_by_name("topal_user_extent_startup_cases")
        .unwrap();
    let syscall4 = elf.symbol_by_name("topal_test_syscall4").unwrap();
    let image = crate::systems_artifact::source_built_static_user_image_fixture(
        u32::try_from(elf.entry()).unwrap(),
        u32::try_from(startup.address()).unwrap(),
        bytes,
    );
    let report = serde_json::json!({
        "source_sha256": format!("{:x}", Sha256::digest(fs::read(&source).unwrap())),
        "linked_sha256": format!("{:x}", Sha256::digest(&linked)),
        "image_sha256": format!("{:x}", Sha256::digest(&image)),
        "base": BASE, "entry": elf.entry(), "syscall": syscall.address(),
        "startup": source_startup.address(), "startup_call": startup.address(),
        "syscall4": syscall4.address(),
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
