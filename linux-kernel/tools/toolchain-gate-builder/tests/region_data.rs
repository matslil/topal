use std::path::{Path, PathBuf};
use std::process::Command;
use topal_compiler::*;

fn work() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("topal-region-data-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    path
}

fn input(source: &str) -> SystemsSourceLibraryInput {
    SystemsSourceLibraryInput {
        source: "policy/unit.t".into(),
        source_name: "unit.t".into(),
        text: source.into(),
        modules: Vec::new(),
    }
}

#[test]
fn region_unit_data_binding_preserves_the_entry_abi_and_valid_empty_graph() {
    // TOPAL-SYSTEMS-REGION-READONLY-DATA-001: source-object evidence only.
    let tools = LlvmTools::discover(None).unwrap();
    for body in [
        "  ()\n",
        "  copied is region byte store (offset-bytes is 7, value is 90)\n  ()\n",
    ] {
        let text = format!(
            "use language (version is v0.1, features is (systems))\npub apply is fn (region : KernelByteRegion) -> Unit\n{body}"
        );
        let input = input(&text);
        let object = build_systems_region_library(&input, "apply", &tools, &work()).unwrap();
        assert_eq!(
            object.record.schema,
            "topal.kernel-region-source-object.x86_64/3"
        );
        assert_eq!(object.record.native_export, "topal_kernel_region_apply_v1");
        assert_eq!(
            object.record.native_abi,
            "topal.kernel-region-control-view64.x86_64-sysv/1"
        );
        assert_eq!(object.record.declared_entry, "(KernelByteRegion) -> Unit");
        assert_eq!(
            object.record.imported_hooks,
            [
                "topal_kernel_region_allocate_v1",
                "topal_kernel_region_abort_v1"
            ]
        );
        assert_eq!(object.record.read_only_data.is_empty(), body == "  ()\n");
        assert!(
            object
                .record
                .read_only_data
                .iter()
                .all(|d| d.symbol.starts_with("topal.region.")
                    && !d.symbol.starts_with("topal.region.action.")
                    && d.bytes > 0
                    && d.alignment == 8)
        );
        verify_systems_region_library(&input, "apply", &tools, &work(), &object).unwrap();
        if !object.record.read_only_data.is_empty() {
            let mut changed = object.clone();
            changed.record.read_only_data[0].sha256 = "0".repeat(64);
            assert!(
                verify_systems_region_library(&input, "apply", &tools, &work(), &changed).is_err()
            );
        }
    }
}

fn offset(bytes: &[u8], address: u64, count: u64) -> usize {
    // Independent ELF64 address-to-file translation, without the product reader.
    let read64 = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
    let headers = read64(40) as usize;
    let stride = u16::from_le_bytes(bytes[58..60].try_into().unwrap()) as usize;
    let sections = u16::from_le_bytes(bytes[60..62].try_into().unwrap()) as usize;
    (0..sections)
        .find_map(|i| {
            let h = headers + i * stride;
            let start = read64(h + 16);
            let size = read64(h + 32);
            (start <= address
                && address
                    .checked_add(count)
                    .is_some_and(|end| end <= start + size))
            .then(|| (read64(h + 24) + address - start) as usize)
        })
        .unwrap()
}

#[test]
fn region_root_changed_literal_rejects_with_unchanged_code_and_corrected_outer_digest() {
    // TOPAL-SYSTEMS-REGION-ROOT-BINDING-001, -READONLY-DATA-001.
    let root = work();
    let library_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../kernel")
        .canonicalize()
        .unwrap();
    let user_path = root.join("user.elf");
    std::fs::write(&user_path, qualified_x86_64_static_user_image_fixture()).unwrap();
    let options = CompileOptions {
        source_name: root.join("root.t").to_string_lossy().into(),
        output: root.join("linked"),
        emit: Emit::Executable,
        llvm_tools: None,
        library_root,
        target: TargetSelection {
            target: Some(topal_language::compiler::INITIAL_SYSTEMS_TARGET.into()),
            board: Some(topal_language::compiler::INITIAL_SYSTEMS_BOARD.into()),
            ..Default::default()
        },
        optimization: Default::default(),
        standard_library: None,
        initial_user_image: None,
        static_user_image: Some(user_path),
    };
    // Keep this an explicit frozen Unit compatibility control while the
    // primary kernel selects the independently bound framed root instance.
    let source = include_str!("../../../labs/qemu/x86_64/pre-frame-kernel-root.t");
    let start = source.find("  region-library is").unwrap();
    let end = start + source[start..].find("  ),\n").unwrap() + 5;
    let legacy = source[start..end]
        .replace("bootstrap-byte-copy-frame.t", "bootstrap-byte-copy.t")
        .replace(
            "borrowed-byte-region-instance-action-v1",
            "borrowed-byte-region-v1",
        );
    let source = format!("{}{}{}", &source[..start], legacy, &source[end..]);
    let artifact = publish_systems_source(&source, &options).unwrap();
    let kernel = std::fs::read(&artifact.kernel).unwrap();
    let initial = std::fs::read(&artifact.initial_user_image).unwrap();
    let user = std::fs::read(artifact.static_user_image.as_ref().unwrap()).unwrap();
    let library = artifact.record.region_library.as_ref().unwrap();
    let binding = artifact.record.region_runtime.as_ref().unwrap();
    assert_eq!(binding.schema, "topal.root-region-invocation.x86_64/2");
    let encoded = serde_json::to_value(&artifact.record).unwrap();
    for field in ["instance", "scalar_count", "live_frame_bytes"] {
        assert!(encoded["region_runtime"].get(field).is_none());
        let mut changed = encoded.clone();
        changed["region_runtime"][field] = serde_json::Value::Null;
        assert!(serde_json::from_value::<SystemsArtifactProvenance>(changed).is_err());
    }
    for field in ["instance", "scalar_count"] {
        assert!(encoded["region_library"].get(field).is_none());
        let mut changed = encoded.clone();
        changed["region_library"][field] = serde_json::Value::Null;
        assert!(serde_json::from_value::<SystemsArtifactProvenance>(changed).is_err());
    }
    assert_eq!(
        binding.source_data,
        inspect_systems_region_read_only_data(&kernel, library).unwrap()
    );
    assert!(inspect_systems_region_action_data(&kernel, library).is_err());
    generate_x86_64_linux_boot_image(&kernel, &artifact.record, &initial, &user).unwrap();
    let destination = binding
        .source_data
        .iter()
        .find(|d| d.symbol == "topal.region.literal.int.1")
        .unwrap();
    let at = offset(&kernel, destination.address, destination.bytes);
    assert_eq!(
        u64::from_le_bytes(kernel[at..at + 8].try_into().unwrap()),
        0
    );
    assert_eq!(
        u64::from_le_bytes(kernel[at + 8..at + 16].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(kernel[at + 16..at + 20].try_into().unwrap()),
        1
    );
    // Read-only flags alone do not admit thread-local storage. Independently
    // add SHF_TLS to the actual constant section, preserving its bytes/name.
    let mut tls = kernel.clone();
    let read64 = |at: usize| u64::from_le_bytes(kernel[at..at + 8].try_into().unwrap());
    let headers = read64(40) as usize;
    let stride = u16::from_le_bytes(kernel[58..60].try_into().unwrap()) as usize;
    let sections = u16::from_le_bytes(kernel[60..62].try_into().unwrap()) as usize;
    let header = (0..sections)
        .map(|i| headers + i * stride)
        .find(|h| {
            let start = read64(h + 16);
            let size = read64(h + 32);
            start <= destination.address && destination.address < start + size
        })
        .unwrap();
    tls[header + 8..header + 16].copy_from_slice(&(read64(header + 8) | 0x400).to_le_bytes());
    let error = inspect_systems_region_read_only_data(&tls, library).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("region data is not immutable allocated storage"),
        "{error}"
    );
    let mut changed_kernel = kernel.clone();
    changed_kernel[at + 16..at + 20].copy_from_slice(&2_u32.to_le_bytes());
    let changed_path = root.join("changed-literal.elf");
    std::fs::write(&changed_path, &changed_kernel).unwrap();
    let sha = Command::new("sha256sum")
        .arg(&changed_path)
        .output()
        .unwrap();
    assert!(sha.status.success());
    let digest = String::from_utf8(sha.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    let mut changed = artifact.record.clone();
    changed
        .outputs
        .iter_mut()
        .find(|o| o.identity == SYSTEMS_KERNEL_FILE)
        .unwrap()
        .sha256 = digest;
    let error =
        generate_x86_64_linux_boot_image(&changed_kernel, &changed, &initial, &user).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("linked immutable region data bytes changed"),
        "{error}"
    );
    let refused = root.join("refused-boot");
    assert!(
        publish_x86_64_linux_boot_image(&changed_kernel, &changed, &initial, &user, &refused)
            .is_err()
    );
    assert!(!refused.exists());
    for mode in 0..14 {
        let mut changed = artifact.record.clone();
        let record = changed.region_library.as_mut().unwrap();
        match mode {
            0 => record.read_only_data.clear(),
            1 => changed.region_runtime.as_mut().unwrap().source_data.clear(),
            2 => changed.region_runtime.as_mut().unwrap().source_data[0].sha256 = "0".repeat(64),
            3 => record.read_only_data[0].bytes += 8,
            4 => record.read_only_data[0].alignment = 1,
            5 => {
                record.read_only_data[0].symbol = artifact
                    .record
                    .file_position_library
                    .as_ref()
                    .unwrap()
                    .read_only_data[0]
                    .symbol
                    .clone()
            }
            6 => record.read_only_data.swap(0, 1),
            7 => record.schema = "topal.kernel-region-source-object.x86_64/2".into(),
            8 => {
                changed.region_runtime.as_mut().unwrap().schema =
                    "topal.root-region-invocation.x86_64/1".into()
            }
            9 => record.read_only_data.push(record.read_only_data[0].clone()),
            10 => {
                record.read_only_data.remove(0);
            }
            11 => changed.region_runtime.as_mut().unwrap().source_data[0].address += 8,
            12 => record.scalar_count = Some(0),
            13 => record.allocation.adapter_family_sha256 = Some("0".repeat(64)),
            _ => unreachable!(),
        }
        assert!(
            generate_x86_64_linux_boot_image(&kernel, &changed, &initial, &user).is_err(),
            "case {mode}"
        );
    }
    eprintln!(
        "complete root immutable-input candidate: {}",
        root.display()
    );
}
