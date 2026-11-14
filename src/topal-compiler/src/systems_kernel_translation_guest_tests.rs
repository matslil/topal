//! Ordinary source-built guest execution of the inactive byte initializer.
//! TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001: this is not root activation, a frame
//! ownership proof, effective hardware protection or a protected-context test.
use super::*;
use crate::systems_tail_owner::run_tail_native_fixture as run;
use object::{Object, ObjectSection, ObjectSymbol};
use std::{fs, path::Path, process::Command};

fn build_user(work: &Path, tools: &crate::LlvmTools) -> Vec<u8> {
    let selected = layout(vec![
        part(0x200000, 0x201000, Immutable),
        part(0x800000, 0x2ae4000, Mutable),
        part(0x2ae4000, 0x2aec000, Immutable),
        part(0x2aec000, 0x2c22000, Code),
        part(0x2c22000, 0x2c23000, Immutable),
    ]);
    let plan = TranslationTables::new(&selected).unwrap();
    assert_eq!(plan.pages(), 8);
    crate::systems_frame_publication::compile("image-table-prepare", &plan.llvm(), tools, work)
        .unwrap();
    let source =
        include_str!("../../../linux-kernel/labs/qemu/x86_64/user_image_table_data_cases.c");
    fs::write(work.join("image-table-user.c"), source).unwrap();
    run(
        work,
        "source-cc-version",
        Command::new("cc").arg("--version"),
    );
    run(
        work,
        "source-compile",
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
            .arg(work.join("image-table-user.c"))
            .arg("-o")
            .arg(work.join("image-table-user.o")),
    );
    fs::write(
        work.join("source-syscall.o"),
        crate::systems_user_extent::source_fixture::syscall_adapter(
            b"topal_image_table_preparation_cases",
            false,
        ),
    )
    .unwrap();
    let script = "SECTIONS { . = 0x570000; .text : { *(.text .text.*) *(.rodata .rodata.*) } \
                  /DISCARD/ : { *(.eh_frame) *(.comment) *(.note*) *(.stack_sizes) } }\n";
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
                "topal_image_table_preparation_cases",
                "-z",
                "noexecstack",
                "-T",
            ])
            .arg(work.join("source-user.ld"))
            .arg(work.join("image-table-user.o"))
            .arg(work.join("image-table-prepare.o"))
            .arg(work.join("source-syscall.o"))
            .arg("-o")
            .arg(work.join("source-user.elf")),
    );
    let linked = fs::read(work.join("source-user.elf")).unwrap();
    let elf = object::File::parse(linked.as_slice()).unwrap();
    let text = elf.section_by_name(".text").unwrap();
    assert_eq!(text.address(), 0x570000);
    for section in elf.sections() {
        assert_eq!(section.relocations().count(), 0);
        if matches!(
            section.kind(),
            object::SectionKind::Data | object::SectionKind::UninitializedData
        ) {
            assert_eq!(
                section.size(),
                0,
                "ordinary fixture has no writable globals"
            );
        }
    }
    assert!(!elf.symbols().any(|symbol| symbol.is_undefined()));
    // Sum every C frame rather than assuming the largest frame is the complete
    // call path. Add the separately measured emitted initializer and adapters.
    let mut source_stack = 0_u64;
    for row in fs::read_to_string(work.join("image-table-user.su"))
        .unwrap()
        .lines()
    {
        let fields = row.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[2], "static");
        source_stack += fields[1].parse::<u64>().unwrap() + 8;
    }
    let raw = fs::read(work.join("image-table-prepare.o")).unwrap();
    let object = object::File::parse(raw.as_slice()).unwrap();
    let native_stack = crate::systems_library::stack_sizes(&object).unwrap()["topal_x86_image_table_prepare_data_v1"];
    let conservative_stack = 1024 + 128 + source_stack + native_stack + 8;
    assert!(
        conservative_stack <= 2048,
        "source/initializer call path exceeds fixture stack"
    );
    let image = crate::systems_artifact::source_built_static_user_image_fixture(
        u32::try_from(elf.entry()).unwrap(),
        None,
        None,
        text.data().unwrap(),
    );
    fs::write(work.join("source-component.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "scope": "Ordinary user-mode execution of inactive byte preparation; numeric physical arguments create no owners or active translations",
        "source_sha256": crate::artifact::sha256(source.as_bytes()),
        "llvm_sha256": crate::artifact::sha256(plan.llvm().as_bytes()),
        "object_sha256": crate::artifact::sha256(&raw),
        "linked_sha256": crate::artifact::sha256(&linked),
        "image_sha256": crate::artifact::sha256(&image),
        "text_bytes": text.size(), "conservative_stack_bytes": conservative_stack,
        "valid_requests_per_call": 4, "refused_requests_per_call": 23,
        "protected_context": false,
    })).unwrap()).unwrap();
    image
}

#[test]
fn ordinary_guest_program_links_without_execution() {
    let work = crate::systems_tail_publication::Scratch::new().unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let image = build_user(&work.0, &tools);
    assert!(!image.is_empty());
}

#[test]
#[ignore = "requires a C compiler and the bounded authorized QEMU VM"]
fn ordinary_guest_checks_inactive_buffer_preparation() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = repository.join(format!(
        "linux-kernel/.cache/x86_64/image-table-user-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(work.parent().unwrap()).unwrap();
    fs::create_dir(&work).unwrap();
    eprintln!("image table user fixture: {}", work.display());
    let tools = crate::LlvmTools::discover(None).unwrap();
    let image = build_user(&work, &tools);
    fs::write(work.join("static-elf-abi-fixture.elf"), &image).unwrap();
    // Use the existing owning-MM kernel: its normal brk implementation admits
    // this independent heap loan. The legacy public fixture profile accepts
    // only its original fixed brk/caller sequence and cannot run this program.
    let guest =
        crate::systems_frame_target::root_tests::prepare_owned_source_fixture(&image, &work);
    fs::write(work.join("guest-build.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "build": guest, "production_qualified": false,
        "scope": "Fresh complete private owning-MM kernel; serial-only ordinary application execution is separate from compile-only preparation",
    })).unwrap()).unwrap();
    let observer = repository.join("linux-kernel/labs/qemu/x86_64/user_image_table_data_check.py");
    run(
        &guest,
        "image-table-data",
        Command::new("python3").arg(observer).arg(&guest),
    );
    let result: serde_json::Value =
        serde_json::from_slice(&fs::read(guest.join("observations.json")).unwrap()).unwrap();
    assert_eq!(result["passed"], true);
    assert_eq!(result["native_execution"], true);
    assert_eq!(result["protected_context"], false);
}
