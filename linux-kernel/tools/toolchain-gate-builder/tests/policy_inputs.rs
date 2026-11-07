use std::path::{Path, PathBuf};
use std::process::Command;
use topal_compiler::*;

fn work() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "topal-policy-inputs-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&path).unwrap();
    path
}
fn offset(bytes: &[u8], address: u64, count: u64) -> usize {
    // Independent ELF64 address-to-file translation, without the product reader.
    let u64at = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
    let headers = u64at(40) as usize;
    let stride = u16::from_le_bytes(bytes[58..60].try_into().unwrap()) as usize;
    let count_sections = u16::from_le_bytes(bytes[60..62].try_into().unwrap()) as usize;
    (0..count_sections)
        .find_map(|i| {
            let h = headers + i * stride;
            let start = u64at(h + 16);
            let size = u64at(h + 32);
            (u64at(h + 8) & 2 != 0
                && start <= address
                && address
                    .checked_add(count)
                    .is_some_and(|end| end <= start + size))
            .then(|| (u64at(h + 24) + address - start) as usize)
        })
        .unwrap()
}
fn corrected(
    record: &SystemsArtifactProvenance,
    bytes: &[u8],
    root: &Path,
) -> SystemsArtifactProvenance {
    let path = root.join("changed.elf");
    std::fs::write(&path, bytes).unwrap();
    let sha = Command::new("sha256sum").arg(&path).output().unwrap();
    assert!(sha.status.success());
    let digest = String::from_utf8(sha.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    let mut changed = record.clone();
    changed
        .outputs
        .iter_mut()
        .find(|o| o.identity == SYSTEMS_KERNEL_FILE)
        .unwrap()
        .sha256 = digest;
    changed
}

fn symbols(bytes: &[u8]) -> Vec<(usize, String)> {
    let u64at = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()) as usize;
    let u32at = |at| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
    let headers = u64at(40);
    let stride = u16::from_le_bytes(bytes[58..60].try_into().unwrap()) as usize;
    let count = u16::from_le_bytes(bytes[60..62].try_into().unwrap()) as usize;
    let mut result = Vec::new();
    for i in 0..count {
        let h = headers + i * stride;
        if u32at(h + 4) != 2 {
            continue;
        }
        let strings = headers + u32at(h + 40) * stride;
        let names = &bytes[u64at(strings + 24)..u64at(strings + 24) + u64at(strings + 32)];
        for at in (u64at(h + 24)..u64at(h + 24) + u64at(h + 32)).step_by(u64at(h + 56)) {
            let start = u32at(at);
            let end = names[start..].iter().position(|b| *b == 0).unwrap() + start;
            result.push((at, std::str::from_utf8(&names[start..end]).unwrap().into()));
        }
    }
    result
}

#[test]
fn policy_linked_sections_refuse_actual_code_literals_tables_and_incomplete_bindings() {
    // TOPAL-SYSTEMS-POLICY-LINKED-INPUTS-001: public publisher and boot boundary.
    let root = work();
    let user_path = root.join("user.elf");
    std::fs::write(&user_path, qualified_x86_64_static_user_image_fixture()).unwrap();
    let options = CompileOptions {
        source_name: root.join("root.t").to_string_lossy().into(),
        output: root.join("linked"),
        emit: Emit::Executable,
        llvm_tools: None,
        library_root: Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../kernel")
            .canonicalize()
            .unwrap(),
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
    let artifact = publish_systems_source(
        include_str!("../../../kernel/arch/x86_64/toolchain-gate.t"),
        &options,
    )
    .unwrap();
    let kernel = std::fs::read(&artifact.kernel).unwrap();
    let initial = std::fs::read(&artifact.initial_user_image).unwrap();
    let user = std::fs::read(artifact.static_user_image.as_ref().unwrap()).unwrap();
    generate_x86_64_linux_boot_image(&kernel, &artifact.record, &initial, &user).unwrap();
    let value = serde_json::to_value(&artifact.record).unwrap();
    let slots = [
        "kernel_library",
        "time_library",
        "image_library",
        "dispatch_library",
        "process_identity_library",
    ];
    for slot in slots {
        let library = &value[slot];
        let sections = library["sections"].as_array().unwrap();
        let linked = library["linked_inputs"]["sections"].as_array().unwrap();
        assert_eq!(sections.len(), linked.len());
        assert_eq!(
            library["object_sha256"],
            library["linked_inputs"]["object_sha256"]
        );
        assert_eq!(
            library["linked_inputs"]["schema"],
            "topal.policy-linked-inputs.x86_64/1"
        );
        assert_eq!(
            library["schema"],
            if slot == "kernel_library" {
                "topal.kernel-source-library/3"
            } else {
                "topal.kernel-numeric-source-library/9"
            }
        );
        for executable in [true, false] {
            let entry = linked
                .iter()
                .find(|s| s["executable"] == executable)
                .unwrap();
            let at = offset(
                &kernel,
                entry["address"].as_u64().unwrap(),
                entry["bytes"].as_u64().unwrap(),
            );
            let mut bytes = kernel.clone();
            bytes[at] ^= 1;
            let changed = corrected(&artifact.record, &bytes, &root);
            let error =
                generate_x86_64_linux_boot_image(&bytes, &changed, &initial, &user).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("linked policy section bytes or complete binding changed"),
                "{slot}/{executable}: {error}"
            );
            let refused = root.join(format!("refused-{slot}-{executable}"));
            assert!(
                publish_x86_64_linux_boot_image(&bytes, &changed, &initial, &user, &refused)
                    .is_err()
            );
            assert!(!refused.exists());
        }
        for mode in 0..18 {
            let mut changed = value.clone();
            let record = &mut changed[slot];
            match mode {
                0 => record["linked_inputs"] = serde_json::Value::Null,
                1 => record["sections"] = serde_json::json!([]),
                2 => record["linked_inputs"]["sections"] = serde_json::json!([]),
                3 => record["sections"][0]["marker"] = "topal.policy-inputs.foreign.0".into(),
                4 => record["sections"][0]["bytes"] = 1.into(),
                5 => record["sections"][0]["alignment"] = 3.into(),
                6 => record["sections"][0]["sha256"] = "0".repeat(64).into(),
                7 => record["sections"][0]["source_section"] = "foreign-section".into(),
                8 => record["sections"].as_array_mut().unwrap().swap(0, 1),
                9 => record["linked_inputs"]["object_sha256"] = "0".repeat(64).into(),
                10 => record["linked_inputs"]["sections"][0]["address"] = 0.into(),
                11 => record["linked_inputs"]["sections"][0]["sha256"] = "0".repeat(64).into(),
                12 => record["linked_inputs"]["schema"] = "old-incomplete/0".into(),
                13 => {
                    record["schema"] = if slot == "kernel_library" {
                        "topal.kernel-source-library/2"
                    } else {
                        "topal.kernel-numeric-source-library/8"
                    }
                    .into()
                }
                14 => {
                    let first = record["sections"][0].clone();
                    record["sections"].as_array_mut().unwrap().push(first);
                }
                15 => {
                    record["sections"].as_array_mut().unwrap().pop();
                }
                16 => record["linked_inputs"]["manifest_sha256"] = "0".repeat(64).into(),
                17 => {
                    record["object_sha256"] = "0".repeat(64).into();
                    record["linked_inputs"]["object_sha256"] = "0".repeat(64).into();
                }
                _ => unreachable!(),
            }
            let changed: SystemsArtifactProvenance = serde_json::from_value(changed).unwrap();
            assert!(
                generate_x86_64_linux_boot_image(&kernel, &changed, &initial, &user).is_err(),
                "{slot} metadata case {mode}"
            );
        }
    }
    // Alter actual ELF marker identities/endpoints and code-section rights,
    // independently of the product's object reader and metadata generators.
    let entries = symbols(&kernel);
    let marker = value["time_library"]["sections"][0]["marker"]
        .as_str()
        .unwrap()
        .to_owned()
        + ".begin";
    let at = entries.iter().find(|(_, name)| name == &marker).unwrap().0;
    let owner = u16::from_le_bytes(kernel[at + 6..at + 8].try_into().unwrap()) as usize;
    let headers = u64::from_le_bytes(kernel[40..48].try_into().unwrap()) as usize;
    let stride = u16::from_le_bytes(kernel[58..60].try_into().unwrap()) as usize;
    let header = headers + owner * stride;
    for mode in 0..6 {
        let mut bytes = kernel.clone();
        match mode {
            0 => bytes[at..at + 4].fill(0),
            1 => bytes[at + 4] = 0x10,
            2 => {
                let address = u64::from_le_bytes(bytes[at + 8..at + 16].try_into().unwrap());
                bytes[at + 8..at + 16].copy_from_slice(&(address + 1).to_le_bytes());
            }
            3 => {
                let other = entries
                    .iter()
                    .find(|(_, name)| {
                        name.starts_with("topal.policy-inputs.time-library.")
                            && name.ends_with(".end")
                    })
                    .unwrap()
                    .0;
                let entry = kernel[at..at + 24].to_vec();
                bytes[other..other + 24].copy_from_slice(&entry);
            }
            4 | 5 => {
                let flags = u64::from_le_bytes(bytes[header + 8..header + 16].try_into().unwrap());
                bytes[header + 8..header + 16]
                    .copy_from_slice(&(flags | if mode == 4 { 0x400 } else { 1 }).to_le_bytes());
            }
            _ => unreachable!(),
        }
        let changed = corrected(&artifact.record, &bytes, &root);
        let error =
            generate_x86_64_linux_boot_image(&bytes, &changed, &initial, &user).unwrap_err();
        assert!(
            error.to_string().contains("E-SYSTEMS-POLICY-INPUTS"),
            "ELF mutation {mode}: {error}"
        );
    }
    // A finite jump table is relocated: the object bytes differ from the actual
    // resolved input. The previous per-literal mechanism could not cover this.
    let finite = &value["kernel_library"];
    let index = finite["sections"]
        .as_array()
        .unwrap()
        .iter()
        .position(|s| s["source_section"] == ".rodata")
        .unwrap();
    assert_ne!(
        finite["sections"][index]["sha256"],
        finite["linked_inputs"]["sections"][index]["sha256"]
    );
    // Independently locate the actual Nat 10 used by the deadline source,
    // rather than assuming the earlier linker's fixed address.
    let time = &value["time_library"]["linked_inputs"]["sections"];
    let mut literals = Vec::new();
    for section in time
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["executable"] == false)
    {
        let at = offset(
            &kernel,
            section["address"].as_u64().unwrap(),
            section["bytes"].as_u64().unwrap(),
        );
        let size = section["bytes"].as_u64().unwrap() as usize;
        if size < 24 {
            continue;
        }
        for relative in (0..=size - 24).step_by(8) {
            let at = at + relative;
            if kernel[at..at + 8] == 0_u64.to_le_bytes()
                && kernel[at + 8..at + 16] == 1_u64.to_le_bytes()
                && kernel[at + 16..at + 20] == 10_u32.to_le_bytes()
            {
                literals.push(at);
            }
        }
    }
    assert!(!literals.is_empty());
    let mut changed = kernel.clone();
    let at = literals[0];
    changed[at + 16..at + 20].copy_from_slice(&11_u32.to_le_bytes());
    let record = corrected(&artifact.record, &changed, &root);
    let error = generate_x86_64_linux_boot_image(&changed, &record, &initial, &user).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("linked policy section bytes or complete binding changed"),
        "{error}"
    );
    eprintln!("complete policy-input candidate: {}", root.display());
}
