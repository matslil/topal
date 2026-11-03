use std::path::{Path, PathBuf};
use std::process::Command;
use topal_compiler::*;

fn run(command: &mut Command) -> String {
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{command:?}: {}\n{}",
        String::from_utf8_lossy(&result.stderr),
        String::from_utf8_lossy(&result.stdout)
    );
    String::from_utf8(result.stdout).unwrap()
}

#[test]
fn same_signature_instances_execute_independently_and_refuse_foreign_code() {
    // TOPAL-SYSTEMS-REGION-INSTANCE-SOURCE-001, -BINDING-001.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "topal-region-instances-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../labs/qemu/x86_64/region_frame_client.c")
        .canonicalize()
        .unwrap();
    let tools = LlvmTools::discover(None).unwrap();
    let text = "use language (version is v0.1, features is (systems))\nremainder is fn (whole : Int, rest : Int) -> Int\n  rest\npub apply is fn ((region : KernelByteRegion, fact0 : Nat, fact1 : Nat, fact2 : Nat)) -> Boolean\n  weighted1 is fact1 * 2\n  weighted2 is fact2 * 3\n  total0 is fact0 + weighted1\n  total is total0 + weighted2\n  parts is total /% 251\n  value is absolute (remainder parts)\n  region byte store (offset-bytes is 0, value is value)\n";
    let mut objects = Vec::new();
    let mut clients: Vec<PathBuf> = Vec::new();
    for name in ["left", "right"] {
        let work = root.join(name);
        std::fs::create_dir(&work).unwrap();
        let source = work.join("instance.t");
        std::fs::write(&source, text).unwrap();
        let input = SystemsSourceLibraryInput {
            source: "policy/instance.t".into(),
            source_name: source.to_string_lossy().into(),
            text: text.into(),
            modules: Vec::new(),
        };
        let built = build_systems_region_instance_library(
            &input,
            "apply",
            3,
            &format!("{name}-owner"),
            &tools,
            &work,
        )
        .unwrap();
        let wrapper = work.join("client.c");
        std::fs::write(&wrapper, format!(
            "#define _GNU_SOURCE\n#include <unistd.h>\nextern void region_instance_probe_exit(void);\n__attribute__((noreturn)) static void {name}_exit(int);\n#define _exit {name}_exit\n#define main {name}_main\n#define arena {name}_arena\n#define cursor {name}_cursor\n#define topal_kernel_region_frame_apply_v1 {}\n#define topal_kernel_region_frame_allocate_v1 {}\n#define topal_kernel_region_frame_abort_v1 {}\n#include {}\n#undef _exit\n__attribute__((noreturn)) static void {name}_exit(int status) {{ region_instance_probe_exit(); _exit(status); }}\nuint64_t {name}_cursor_value(void) {{ return {name}_cursor; }}\nint {name}_arena_zero(void) {{ for (size_t i=0; i<sizeof({name}_arena); ++i) if ({name}_arena[i]) return 0; return 1; }}\n",
            built.record.native_export, built.record.imported_hooks[0], built.record.imported_hooks[1], serde_json::to_string(&fixture.to_string_lossy()).unwrap()
        )).unwrap();
        let client = work.join("client.o");
        run(Command::new("cc")
            .args([
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-O2",
                "-g",
                "-DFRAME_WORD_COUNT=3",
            ])
            .arg(format!(
                "-DFRAME_SOURCE_LIMIT_BYTES={}",
                built.record.allocation.numeric.maximum_bytes
            ))
            .arg("-c")
            .arg(wrapper)
            .arg("-o")
            .arg(&client));
        clients.push(client);
        objects.push((name, work, built.record));
    }
    let binary = root.join("instance-client");
    let mut command = Command::new("cc");
    command
        .args([
            "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2", "-g", "-no-pie",
        ])
        .arg(fixture.with_file_name("region_instance_client.c"))
        .args(clients);
    for (_, work, _) in &objects {
        command.arg(work.join("kernel-region.o"));
    }
    run(command.arg("-o").arg(&binary));
    let output = run(&mut Command::new(&binary));
    assert!(
        output.contains("INSTANCE_NATIVE_PASS contexts=2 calls=96 refusals=32 cross=2"),
        "{output}"
    );
    for (name, work, record) in &objects {
        let data = inspect_systems_region_read_only_data(&std::fs::read(&binary).unwrap(), record)
            .unwrap();
        let config = work.join("debug.json");
        std::fs::write(&config, serde_json::to_vec(&serde_json::json!({ "record": record, "data": data, "arena_symbol": format!("{name}_arena"), "cursor_symbol": format!("{name}_cursor") })).unwrap()).unwrap();
        let script = work.join("instance.gdb");
        std::fs::write(
            &script,
            format!(
                "set $frame_config = {}\nsource {}\n",
                serde_json::to_string(&config.to_string_lossy()).unwrap(),
                fixture.with_file_name("region_frame_debug.py").display()
            ),
        )
        .unwrap();
        let output = run(Command::new("gdb")
            .args(["-q", "-batch"])
            .arg(&binary)
            .arg("-x")
            .arg(script));
        assert!(
            output.contains("FRAME_DEBUG_PASS words=3 calls=48"),
            "{output}"
        );
        std::fs::write(work.join("debug.log"), output).unwrap();
    }
    eprintln!("instance native qualification: {}", root.display());
}
