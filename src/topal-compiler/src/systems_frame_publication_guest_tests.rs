use super::*;
use object::ObjectSegment;

#[test]
fn frame_reconstruction_binds_actual_loads_and_executes_syscall_memory() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: fresh native objects, relocated
    // load ownership and real user requests; no public publication bypass.
    let root = crate::systems_frame_target::root_tests::root();
    let input = crate::systems_frame_target::root_tests::input();
    let tools = LlvmTools::discover(None).unwrap();
    let work = super::tests::work("guest");
    let mut fresh = rebuild(&root, &(&input).into(), &tools, &work.join("native")).unwrap();
    let sources = sources(&fresh.program);
    fresh
        .complete_libraries(&sources, &tools, &work.join("native"))
        .unwrap();
    assert_eq!(fresh.record.objects.len(), 20);
    fs::write(
        work.join("other-sources.json"),
        serde_json::to_vec_pretty(&sources).unwrap(),
    )
    .unwrap();
    let kernel = link(&work, &tools, &fresh);
    fresh.bind(&kernel).unwrap();
    assert!(!fresh.record.linked.is_empty());
    fs::write(
        work.join("binding.json"),
        serde_json::to_vec_pretty(&fresh.record).unwrap(),
    )
    .unwrap();
    let again = validate_binding(&kernel, &fresh.record, &tools, &work.join("replayed")).unwrap();
    assert_eq!(again.record, fresh.record);
    let mut bad = fresh.record.clone();
    bad.bounds.combined_with_syscall_caller -= 8;
    assert!(validate_binding(&kernel, &bad, &tools, &work.join("wrong-bounds")).is_err());
    bad = fresh.record.clone();
    bad.objects.pop();
    assert!(validate_binding(&kernel, &bad, &tools, &work.join("missing-object")).is_err());
    bad = fresh.record.clone();
    bad.other_sources.remove("library-time");
    assert!(validate_binding(&kernel, &bad, &tools, &work.join("missing-library")).is_err());
    bad = fresh.record.clone();
    bad.other_original_objects
        .insert("library-namespace".into(), "0".repeat(64));
    assert!(validate_binding(&kernel, &bad, &tools, &work.join("wrong-library-object")).is_err());
    bad = fresh.record.clone();
    bad.library.text = bad.library.text.replace("-> Nat\n", "-> Boolean\n");
    assert!(validate_binding(&kernel, &bad, &tools, &work.join("wrong-source")).is_err());
    bad = fresh.record.clone();
    bad.llvm_version.push_str("-different");
    assert!(validate_binding(&kernel, &bad, &tools, &work.join("wrong-toolchain")).is_err());
    assert!(!work.join("wrong-toolchain").exists());
    // Alter one real initialized load contribution per role without changing
    // its hash or trusted fresh object. Every contribution must be compared.
    let parsed = object::File::parse(kernel.as_slice()).unwrap();
    for role in &fresh.record.objects {
        let section = role.sections.iter().find(|s| !s.zero).unwrap();
        let start = fresh
            .record
            .linked
            .iter()
            .find(|p| p.marker == section.marker)
            .unwrap()
            .address;
        let load = parsed
            .segments()
            .find(|s| s.address() <= start && start < s.address() + s.size())
            .unwrap();
        let offset = load.file_range().0 + start - load.address();
        let mut changed = kernel.clone();
        changed[offset as usize] ^= 1;
        assert!(
            again_check(&fresh, &changed),
            "unchecked native role {}",
            role.role
        );
    }
    let mut changed = kernel.clone();
    let phoff = u64::from_le_bytes(kernel[32..40].try_into().unwrap()) as usize;
    let entry = u16::from_le_bytes(kernel[54..56].try_into().unwrap()) as usize;
    let count = u16::from_le_bytes(kernel[56..58].try_into().unwrap()) as usize;
    let mut altered = false;
    for i in 0..count {
        let at = phoff + i * entry;
        let kind = u32::from_le_bytes(kernel[at..at + 4].try_into().unwrap());
        let flags = u32::from_le_bytes(kernel[at + 4..at + 8].try_into().unwrap());
        if kind == 1 && flags == 6 {
            changed[at + 4..at + 8].copy_from_slice(&7_u32.to_le_bytes());
            altered = true;
            break;
        }
    }
    assert!(altered && again_check(&fresh, &changed));
    let mut changed = kernel.clone();
    changed[24] ^= 1;
    assert!(again_check(&fresh, &changed));
    let padding_names = kernel
        .windows(b".relro_padding\0".len())
        .enumerate()
        .filter_map(|(i, b)| (b == b".relro_padding\0").then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(padding_names.len(), 1);
    let mut changed = kernel.clone();
    changed[padding_names[0] + 1] = b'z';
    assert!(again_check(&fresh, &changed));
    // Keep exact fresh objects and replay evidence for the actual VM checks.
    fs::write(
        work.join("bzImage"),
        crate::systems_boot_image::unpublished_component_fixture(&kernel).unwrap(),
    )
    .unwrap();
    let fixture = crate::systems_frame_target::memory_tests::fixture();
    let resource = crate::systems_boot_resource::generate_initial_boot_resource(
        &crate::systems_provider_object::initial_user_elf_file().unwrap(),
        &fixture,
    )
    .unwrap();
    fs::write(work.join("initramfs.cpio"), resource).unwrap();
    let tokens = parsed
        .symbol_by_name("topal_x86_frame_root_tokens_v1")
        .unwrap()
        .size();
    fs::write(
        work.join("root-tokens.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"bytes":tokens})).unwrap(),
    )
    .unwrap();
    let observer = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/frame_memory_inspect.py");
    crate::systems_tail_owner::run_tail_native_fixture(
        &work,
        "guest",
        Command::new("python3").arg(observer).arg(&work),
    );
    crate::systems_frame_target::memory_tests::inspect_faults(&work);
}

fn again_check(fresh: &Rebuilt, kernel: &[u8]) -> bool {
    let objects = fresh
        .objects
        .iter()
        .zip(&fresh.record.objects)
        .map(|(bytes, record)| (bytes.as_slice(), record.sections.as_slice()))
        .collect::<Vec<_>>();
    match linked::inspect(kernel, &objects) {
        Err(_) => true,
        Ok(placements) => {
            validate_reservation(kernel).is_err()
                || coverage::validate(kernel, &placements).is_err()
        }
    }
}

fn link(work: &Path, tools: &LlvmTools, fresh: &Rebuilt) -> Vec<u8> {
    let fixture = work.join("static-elf-abi-fixture.elf");
    fs::write(
        &fixture,
        crate::systems_frame_target::memory_tests::fixture(),
    )
    .unwrap();
    fs::write(
        work.join("initial-user.elf"),
        crate::systems_provider_object::initial_user_elf_file().unwrap(),
    )
    .unwrap();
    let mut reserved = 0x1000000_u64;
    for label in ["layout", "link"] {
        let mut command = Command::new(tools.tool_path("rust-lld"));
        command.args([
            "-flavor",
            "gnu",
            "-m",
            "elf_x86_64",
            "-static",
            "--no-dynamic-linker",
            "--build-id=none",
            "--entry",
            crate::X86_SYSTEMS_KERNEL_ENTRY,
            "--section-start=.bss=0x800000",
            "-z",
            "max-page-size=0x1000",
        ]);
        command.arg(format!(
            "--defsym=topal_x86_kernel_reservation_end_v1={reserved}"
        ));
        for record in &fresh.record.objects {
            command.arg(work.join(format!("native/{}.o", record.role)));
        }
        command.arg("-o").arg(work.join("kernel.elf"));
        run(work, label, &mut command).unwrap();
        let raw = fs::read(work.join("kernel.elf")).unwrap();
        let file = object::File::parse(raw.as_slice()).unwrap();
        let end = file
            .segments()
            .map(|s| s.address() + s.size())
            .max()
            .unwrap()
            .max(0x1000000);
        if label == "link" {
            assert_eq!(reserved, end);
        } else {
            reserved = end;
        }
    }
    fs::read(work.join("kernel.elf")).unwrap()
}

fn sources(program: &topal_language::compiler::CompilerSystemsProgram) -> BTreeMap<String, Source> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
    [
        ("library-kernel", &program.kernel_library),
        ("library-time", &program.time_library),
        ("library-image", &program.image_library),
        ("library-dispatch", &program.dispatch_library),
        (
            "library-process-identity",
            &program.process_identity_library,
        ),
        ("library-region", &program.region_library),
        ("library-file-position", &program.file_position_library),
        ("library-ram-page-pool", &program.ram_page_pool_library),
        ("library-inode-mode", &program.inode_mode_library),
        ("library-namespace", &program.namespace_library),
    ]
    .into_iter()
    .filter_map(|(role, requirement)| requirement.as_ref().map(|r| (role, r)))
    .map(|(role, r)| {
        (
            role.into(),
            (&crate::systems_library::load_source_library(r, &directory).unwrap()).into(),
        )
    })
    .collect()
}
