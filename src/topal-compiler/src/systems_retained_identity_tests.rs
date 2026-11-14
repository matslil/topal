//! Actual persistent-value adapter/native protocol; kernel integration is separate.
use super::*;

fn source() -> (
    CompilerSystemsSourceLibraryRequirement,
    SystemsSourceLibraryInput,
) {
    let requirement = CompilerSystemsSourceLibraryRequirement {
        source: "policy/process-identity.t".into(),
        export: "process-identity".into(),
        interface: CompilerSystemsSourceLibraryInterface::RetainedProcessIdentityV1,
        stack_limit_bytes: 4096,
        runtime_limit_bytes: Some(65536),
        state_limit_bytes: Some(4096),
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
    let input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
    (requirement, input)
}
fn compile_client(directory: &std::path::Path) -> std::path::PathBuf {
    let driver = directory.join("client.c");
    std::fs::write(
        &driver,
        include_str!("../../../linux-kernel/labs/qemu/x86_64/retained_identity_client.c"),
    )
    .unwrap();
    let executable = directory.join("retained-policy");
    let output = std::process::Command::new("cc")
        .arg("-no-pie")
        .arg(&driver)
        .arg(directory.join("process-identity-library.o"))
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}
#[test]
fn native_retained_identity_preserves_published_graph_until_preparation_is_consumed() {
    // TOPAL-SYSTEMS-RETAINED-NUMERIC-STATE-001; TOPAL-SYSTEMS-RETAINED-PROCESS-IDENTITY-001
    let (requirement, input) = source();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let directory =
        std::env::temp_dir().join(format!("topal-retained-identity-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let mut reversed = input.clone();
    reversed.text = reversed
        .text
        .replace(
            "Record (allocated : Nat, next : Nat)",
            "Record (next : Nat, allocated : Nat)",
        )
        .replace(
            "(allocated is allocated, next is following)",
            "(next is following, allocated is allocated)",
        );
    for (index, input) in [input, reversed].iter().enumerate() {
        let library = build_numeric_library(&requirement, input, &tools, &directory).unwrap();
        assert_eq!(
            library.record.retained_state,
            Some(SystemsRetainedStateBound::qualified())
        );
        assert!(library.record.result_order_suffix);
        assert_eq!(library.record.allocation.maximum_bytes, 312);
        let executable = compile_client(&directory);
        let output = std::process::Command::new(executable).output().unwrap();
        assert!(
            output.status.success(),
            "status={:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
        let highwater = String::from_utf8(output.stdout)
            .unwrap()
            .trim()
            .parse::<u64>()
            .unwrap();
        assert!(highwater <= library.record.allocation.maximum_bytes);
        eprintln!(
            "retained order={index} stack={} bound={} highwater={highwater}",
            library.record.maximum_stack_bytes, library.record.allocation.maximum_bytes
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
fn malformed_source_result_cannot_modify_published_or_prepared_storage() {
    // TOPAL-SYSTEMS-RETAINED-NUMERIC-STATE-001
    let (requirement, input) = source();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let directory =
        std::env::temp_dir().join(format!("topal-retained-invalid-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    for expression in ["0", "2147483648", "18446744073709551618"] {
        let mut changed = input.clone();
        changed.text = input
            .text
            .replace("next is following", &format!("next is {expression}"));
        build_numeric_library(&requirement, &changed, &tools, &directory).unwrap();
        let executable = compile_client(&directory);
        let output = std::process::Command::new(executable)
            .arg("invalid-result")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "status={:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}
