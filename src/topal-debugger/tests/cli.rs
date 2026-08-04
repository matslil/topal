use std::process::Command;

#[test]
fn navigates_recorded_execution_in_both_directions() {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/debugger/basic-history.t"
    );
    let commands = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/debugger/basic-history.debug"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args(["--script", commands, source])
        .output()
        .expect("debugger should run script");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("#0 context.selected [TOPAL-SYN-UNICODE-001]"));
    assert!(stdout.contains("#1 source.accepted [TOPAL-SYN-SOURCE-001]"));
    assert!(stdout.contains("> #0 context.selected"));
}

#[test]
fn script_mode_reports_command_file_errors() {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/debugger/basic-history.t"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args(["--script", "missing-debug-commands", source])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("cannot read command script missing-debug-commands")
    );
}

#[test]
fn reports_debuggee_diagnostics_with_its_source_name() {
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .arg("missing-topal-debugger-example.t")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("cannot read missing-topal-debugger-example.t")
    );
}
