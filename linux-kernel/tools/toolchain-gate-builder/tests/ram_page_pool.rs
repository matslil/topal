use std::path::{Path, PathBuf};
use std::process::Command;
use topal_compiler::*;

fn run(command: &mut Command) -> String {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn directory(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::create_dir(&path).unwrap();
    path
}

fn digest(path: &Path) -> String {
    run(Command::new("sha256sum").arg(path))
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

fn reverse_order(text: &str) -> String {
    let mut changed = 0;
    let lines = text
        .lines()
        .map(|line| {
            for slot in 0..8 {
                let expected = format!("try-reserve (pool, {slot})");
                if line.trim() == expected {
                    changed += 1;
                    return line.replacen(
                        &expected,
                        &format!("try-reserve (pool, {})", 7 - slot),
                        1,
                    );
                }
            }
            line.to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(changed, 8);
    lines.join("\n") + "\n"
}

#[test]
fn ordinary_page_pool_executes_every_occupancy_and_release_with_checked_frames() {
    // TOPAL-SYSTEMS-REGION-PAGE-OWNER-001, -ALLOCATE-001, -RELEASE-001.
    // Source-component evidence only: live kernel ownership/retired-stack
    // reclamation and Linux service integration require their own VM stage.
    let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let lab = project.join("labs/qemu/x86_64").canonicalize().unwrap();
    let kernel = project.join("kernel").canonicalize().unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "topal-ram-page-pool-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let primary_input = SystemsSourceLibraryInput {
        source: "memory/ram-page-pool.t".into(),
        source_name: "memory/ram-page-pool.t".into(),
        text: std::fs::read_to_string(kernel.join("memory/ram-page-pool.t")).unwrap(),
        modules: vec![SystemsSourceModuleInput {
            source: "support/page-slots.t".into(),
            identity: vec!["support".into(), "page-slots".into()],
            text: std::fs::read_to_string(kernel.join("support/page-slots.t")).unwrap(),
        }],
    };
    let tools = LlvmTools::discover(None).unwrap();
    let mut previous = None;
    let mut observations = Vec::new();
    for (name, reverse) in [("first", false), ("last", true)] {
        let variant = directory(&root, name);
        let mut input = primary_input.clone();
        if reverse {
            input.modules[0].text = reverse_order(&input.modules[0].text);
        }
        let work = directory(&variant, "source");
        let object = build_systems_region_instance_library(
            &input,
            "apply",
            2,
            "ram-page-pool",
            &tools,
            &work,
        )
        .unwrap();
        let record = &object.record;
        assert_eq!(record.instance.as_deref(), Some("ram-page-pool"));
        assert_eq!(record.scalar_count, Some(2));
        assert!(
            record.maximum_stack_bytes <= 4096 && record.allocation.numeric.maximum_bytes <= 65536
        );
        let encoded = serde_json::to_value(record).unwrap();
        assert_eq!(
            serde_json::from_value::<SystemsRegionLibraryArtifact>(encoded.clone()).unwrap(),
            *record
        );
        for field in ["instance", "scalar_count"] {
            let mut malformed = encoded.clone();
            malformed[field] = serde_json::Value::Null;
            assert!(serde_json::from_value::<SystemsRegionLibraryArtifact>(malformed).is_err());
        }
        verify_systems_region_instance_library(
            &input,
            "apply",
            2,
            "ram-page-pool",
            &tools,
            &directory(&variant, "verify"),
            &object,
        )
        .unwrap();
        assert!(
            verify_systems_region_instance_library(
                &input,
                "apply",
                2,
                "foreign-pool",
                &tools,
                &directory(&variant, "wrong-instance"),
                &object
            )
            .is_err()
        );
        assert!(
            verify_systems_region_instance_library(
                &input,
                "apply",
                3,
                "ram-page-pool",
                &tools,
                &directory(&variant, "wrong-count"),
                &object
            )
            .is_err()
        );
        let mut undercharged = object.clone();
        undercharged.record.allocation.numeric.maximum_bytes -= 8;
        assert!(
            verify_systems_region_instance_library(
                &input,
                "apply",
                2,
                "ram-page-pool",
                &tools,
                &directory(&variant, "undercharged"),
                &undercharged
            )
            .is_err()
        );
        if let Some(previous) = &previous {
            assert!(
                verify_systems_region_instance_library(
                    &input,
                    "apply",
                    2,
                    "ram-page-pool",
                    &tools,
                    &directory(&variant, "changed-helper"),
                    previous
                )
                .is_err()
            );
            assert_ne!(object.record.object_sha256, previous.record.object_sha256);
            assert_ne!(
                object.record.module_snapshot_sha256,
                previous.record.module_snapshot_sha256
            );
        }
        let binary = variant.join("ram-page-client");
        run(Command::new("cc")
            .args([
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-O2",
                "-g",
                "-no-pie",
                "-DFRAME_WORD_COUNT=2",
            ])
            .arg(format!("-DRAM_REVERSE_ORDER={}", u8::from(reverse)))
            .arg(format!(
                "-DFRAME_SOURCE_LIMIT_BYTES={}",
                record.allocation.numeric.maximum_bytes
            ))
            .arg(format!(
                "-Dtopal_kernel_region_frame_apply_v1={}",
                record.native_export
            ))
            .arg(format!(
                "-Dtopal_kernel_region_frame_allocate_v1={}",
                record.imported_hooks[0]
            ))
            .arg(format!(
                "-Dtopal_kernel_region_frame_abort_v1={}",
                record.imported_hooks[1]
            ))
            .arg(lab.join("ram_page_pool_client.c"))
            .arg(work.join("kernel-region.o"))
            .arg("-o")
            .arg(&binary));
        let output = run(&mut Command::new(&binary));
        assert!(
            output.contains("RAM_PAGE_POOL_NATIVE_PASS calls=2323 samples=55 refusals=16"),
            "{output}"
        );
        let native_output = output;
        let data = inspect_systems_region_read_only_data(&std::fs::read(&binary).unwrap(), record)
            .unwrap();
        let config = variant.join("debug.json");
        std::fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({"record": record, "data": data})).unwrap(),
        )
        .unwrap();
        let script = variant.join("observe.gdb");
        std::fs::write(
            &script,
            format!(
                "set $ram_config = {}\nsource {}\n",
                serde_json::to_string(&config.to_string_lossy()).unwrap(),
                lab.join("ram_page_pool_debug.py").display()
            ),
        )
        .unwrap();
        let output = run(Command::new("gdb")
            .args(["-q", "-batch"])
            .arg(&binary)
            .arg("-x")
            .arg(script));
        assert!(
            output.contains("RAM_PAGE_POOL_DEBUG_PASS samples=55 helper-kinds=2"),
            "{output}"
        );
        std::fs::write(variant.join("debug.log"), &output).unwrap();
        observations.push(serde_json::json!({
            "placement": name,
            "source_record": record,
            "linked_read_only_data": data,
            "native_calls": 2323,
            "native_pre_entry_refusals": 16,
            "gdb_samples": 55,
            "native_output": native_output,
            "gdb_result": output.lines().find(|line| line.starts_with("RAM_PAGE_POOL_DEBUG_PASS")).unwrap(),
            "native_client_sha256": digest(&binary),
        }));
        previous = Some(object);
    }
    if std::env::var_os("TOPAL_WRITE_RAM_PAGE_EVIDENCE").is_some() {
        let evidence = serde_json::json!({
            "schema": "topal.source-ram-page-pool-component/1",
            "qualification": "ordinary-source-object-only",
            "live_kernel_caller_qualified": false,
            "kernel_retired_stack_erasure_qualified": false,
            "native_calls": 4646,
            "native_pre_entry_refusals": 32,
            "gdb_samples": 110,
            "controlled_metadata_refusals": 11,
            "source_sha256": digest(&kernel.join("memory/ram-page-pool.t")),
            "helper_sha256": digest(&kernel.join("support/page-slots.t")),
            "client_sha256": digest(&lab.join("ram_page_pool_client.c")),
            "frame_client_sha256": digest(&lab.join("region_frame_client.c")),
            "debugger_sha256": digest(&lab.join("ram_page_pool_debug.py")),
            "test_sha256": digest(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ram_page_pool.rs")),
            "observations": observations,
        });
        std::fs::write(
            lab.join("results/topal-source-ram-page-pool-component.json"),
            serde_json::to_string_pretty(&evidence).unwrap() + "\n",
        )
        .unwrap();
    }
    eprintln!("page-pool source qualification: {}", root.display());
}
