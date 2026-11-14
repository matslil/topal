use std::path::{Path, PathBuf};
use std::process::Command;
use topal_compiler::*;

fn work() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("topal-region-frame-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    path
}
fn run(command: &mut Command) -> String {
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap()
}
fn source(count: u8) -> String {
    let mut text = "use language (version is v0.1, features is (systems))\n".to_owned();
    if count == 0 {
        text.push_str("pub apply is fn (region : KernelByteRegion) -> Boolean\n  true\n");
        return text;
    }
    text.push_str("remainder is fn (whole : Int, rest : Int) -> Int\n  rest\n\npub apply is fn ((region : KernelByteRegion");
    for i in 0..count {
        text.push_str(&format!(", fact{i} : Nat"));
    }
    text.push_str(")) -> Boolean\n  total0 is 0\n");
    for i in 0..count {
        text.push_str(&format!(
            "  weighted{i} is fact{i} * {}\n  total{} is total{i} + weighted{i}\n",
            i + 1,
            i + 1
        ));
    }
    text.push_str(&format!("  parts is total{count} /% 251\n  value is absolute (remainder parts)\n  region byte store (offset-bytes is 0, value is value)\n"));
    text
}

#[test]
fn framed_native_inputs_preserve_u64_values_order_readonly_extent_and_refusal() {
    // TOPAL-SYSTEMS-REGION-FRAME-SOURCE-001, -FRAME-ABI-001.
    let tools = LlvmTools::discover(None).unwrap();
    for count in [0, 1, 2, 3, 16] {
        let root = work();
        let text = source(count);
        let path = root.join("frame.t");
        std::fs::write(&path, &text).unwrap();
        let input = SystemsSourceLibraryInput {
            source: "policy/frame.t".into(),
            source_name: path.to_string_lossy().into(),
            text,
            modules: Vec::new(),
        };
        let object =
            build_systems_region_framed_action_library(&input, "apply", count, &tools, &root)
                .unwrap();
        assert_eq!(object.record.scalar_count, Some(count));
        assert_eq!(
            object.record.allocation.numeric.bridge_bytes,
            64 + 24 * u64::from(count)
        );
        let client = root.join("frame-client");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../labs/qemu/x86_64/region_frame_client.c");
        run(Command::new("cc")
            .args([
                "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2", "-g", "-no-pie",
            ])
            .arg(format!("-DFRAME_WORD_COUNT={count}"))
            .arg(format!(
                "-DFRAME_SOURCE_LIMIT_BYTES={}",
                object.record.allocation.numeric.maximum_bytes
            ))
            .arg(&fixture)
            .arg(root.join("kernel-region.o"))
            .arg("-o")
            .arg(&client));
        let output = run(&mut Command::new(&client));
        assert!(
            output.contains(&format!(
                "FRAME_NATIVE_PASS words={count} calls=48 refusals=16"
            )),
            "{output}"
        );
        let data =
            inspect_systems_region_read_only_data(&std::fs::read(&client).unwrap(), &object.record)
                .unwrap();
        let config = root.join("debug.json");
        std::fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({
                "record": object.record, "data": data,
            }))
            .unwrap(),
        )
        .unwrap();
        let debug = fixture.with_file_name("region_frame_debug.py");
        let script = root.join("frame.gdb");
        std::fs::write(
            &script,
            format!(
                "set $frame_config = {}\nsource {}\n",
                serde_json::to_string(&config.to_string_lossy()).unwrap(),
                debug.display(),
            ),
        )
        .unwrap();
        let output = run(Command::new("gdb")
            .args(["-q", "-batch"])
            .arg(&client)
            .arg("-x")
            .arg(&script));
        assert!(
            output.contains(&format!("FRAME_DEBUG_PASS words={count} calls=48")),
            "{output}"
        );
        std::fs::write(root.join("debug.log"), &output).unwrap();
        eprintln!(
            "framed native qualification count={count}: {}",
            root.display()
        );
    }
}
