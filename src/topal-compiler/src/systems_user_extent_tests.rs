use super::*;
use crate::systems_tail_owner::run_tail_native_fixture as run;
use object::{Object, ObjectSymbol};
use std::{fs, process::Command};

fn normal_guest() -> &'static std::path::PathBuf {
    static WORK: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    WORK.get_or_init(crate::systems_frame_target::root_tests::execute_user_extents)
}

#[test]
fn user_extent_normal_guest_boot_and_memory_operations() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: normal boot and real MM operations only.
    // Keep this independently selectable from deliberate guest-state controls.
    let observations: serde_json::Value = serde_json::from_slice(
        &fs::read(normal_guest().join("user-extents/observations.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(observations["passed"], true);
    assert_eq!(observations["user_extents"].as_array().unwrap().len(), 324);
    assert_eq!(observations["memory"].as_array().unwrap().len(), 80);
}

#[test]
fn user_extent_shared_callers_use_real_large_heap_buffers() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: ordinary syscalls through the migrated
    // helper. Direct-caller migration and publication remain separate gates.
    let work = crate::systems_frame_target::root_tests::execute_user_extent_callers();
    let observations: serde_json::Value = serde_json::from_slice(
        &fs::read(work.join("user-extent-callers/observations.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(observations["passed"], true);
    assert_eq!(
        observations["shared_caller_registers_and_retirement_checked"],
        true
    );
    assert_eq!(observations["startup_outputs_checked"], true);
    assert_eq!(observations["shell_outputs_checked"], true);
    assert_eq!(observations["stat_outputs_checked"], true);
    assert_eq!(observations["statat_paths_checked"], true);
    assert_eq!(
        observations["write_fixture_cases_checked"],
        serde_json::json!((0..20).collect::<Vec<_>>())
    );
    assert_eq!(
        observations["file_fixture_cases_checked"],
        serde_json::json!((0..11).collect::<Vec<_>>())
    );
    assert_eq!(
        observations["shell_io_fixture_cases_checked"],
        serde_json::json!((0..15).collect::<Vec<_>>())
    );
    assert_eq!(
        observations["command_fixture_cases_checked"],
        serde_json::json!([0, 1, 2, 3, 4])
    );
    assert_eq!(
        observations["statat_fixture_cases_checked"],
        serde_json::json!([0, 1, 2, 3, 4, 5])
    );
    assert_eq!(
        observations["stat_fixture_cases_checked"],
        serde_json::json!([0, 1, 2, 3])
    );
    assert_eq!(observations["process_cleanup_checked"], true);
    assert!(observations["memory"].as_array().unwrap().len() >= 83);
    assert!(observations["helper_calls"].as_array().unwrap().len() >= 4);
}

#[test]
fn user_extent_direct_startup_outputs_use_grown_split_page_heap() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: real user-issued startup requests. This
    // focused endpoint does not claim whole-process cleanup or publication.
    let work = crate::systems_frame_target::root_tests::execute_user_extent_startup();
    let observed: serde_json::Value = serde_json::from_slice(
        &fs::read(work.join("user-extent-callers/observations.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(observed["passed"], true);
    assert_eq!(observed["startup_outputs_checked"], true);
    assert_eq!(observed["process_cleanup_checked"], false);
}

#[test]
fn user_extent_direct_shell_outputs_use_grown_split_page_heap() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: complete uname and initial root getcwd
    // through ordinary user requests; later getcwd paths need the full run.
    let work = crate::systems_frame_target::root_tests::execute_user_extent_shell();
    let observed: serde_json::Value = serde_json::from_slice(
        &fs::read(work.join("user-extent-callers/observations.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(observed["passed"], true);
    assert_eq!(observed["startup_outputs_checked"], true);
    assert_eq!(observed["shell_outputs_checked"], true);
    assert_eq!(observed["process_cleanup_checked"], false);
}

#[test]
fn user_extent_shell_io_uses_actual_transfer_lengths() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: first-image calls through serial input.
    let work = crate::systems_frame_target::root_tests::execute_user_extent_shell_io();
    let observed: serde_json::Value = serde_json::from_slice(
        &fs::read(work.join("user-extent-callers/observations.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(observed["passed"], true);
    assert_eq!(observed["process_cleanup_checked"], false);
    assert_eq!(
        observed["shell_io_fixture_cases_checked"],
        serde_json::json!((0..15).collect::<Vec<_>>())
    );
    assert_eq!(
        observed["command_fixture_cases_checked"],
        serde_json::json!([0, 1, 2, 3, 4])
    );
}

#[test]
fn user_extent_direct_fstat_outputs_use_grown_split_page_heap() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: ordinary console fstat with full native
    // record/padding and staging checks. Other descriptors need the full run.
    let work = crate::systems_frame_target::root_tests::execute_user_extent_stat();
    let observed: serde_json::Value = serde_json::from_slice(
        &fs::read(work.join("user-extent-callers/observations.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(observed["passed"], true);
    assert_eq!(observed["stat_outputs_checked"], true);
    assert_eq!(
        observed["stat_fixture_cases_checked"],
        serde_json::json!([0])
    );
    assert_eq!(observed["process_cleanup_checked"], false);
}

#[test]
fn user_extent_direct_newfstatat_paths_preserve_lookup_error_order() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: ordinary pathname read/refusal and an
    // absent lookup with unused unmapped output. Positive paths need full run.
    let work = crate::systems_frame_target::root_tests::execute_user_extent_statat();
    let observed: serde_json::Value = serde_json::from_slice(
        &fs::read(work.join("user-extent-callers/observations.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(observed["passed"], true);
    assert_eq!(observed["statat_paths_checked"], true);
    assert_eq!(
        observed["statat_fixture_cases_checked"],
        serde_json::json!([0])
    );
    assert_eq!(observed["process_cleanup_checked"], false);
}

#[test]
fn user_extent_owns_real_kernel_tables_and_dynamic_heap_in_qemu() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: unpublished actual-root component. Full
    // production syscall-buffer migration/publication remain separate.
    let work = normal_guest();
    let inspector = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/user_extent_fault_inspect.py");
    for mode in [
        "user-4",
        "user-3",
        "user-2",
        "user-1",
        "write-2",
        "write-1",
        "absent-2",
        "absent-1",
        "huge",
        "leaf-format",
        "foreign-table",
        "foreign-backing",
        "foreign-issuer",
        "unpinned-table",
        "stale-cell",
        "return-site",
        "busy",
        "epoch-zero",
        "address-space",
        "context-self",
        "late-epoch",
    ] {
        run(
            &work,
            &format!("extent-fault-{mode}"),
            Command::new("python3").arg(&inspector).arg(&work).arg(mode),
        );
    }
}

#[test]
fn user_extent_owners_come_from_the_checked_root_without_changing_its_bytes() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: compiler identities alone grant no runtime lease.
    let root = crate::systems_frame_target::root_tests::root();
    let input = crate::systems_frame_target::root_tests::input();
    let checked = crate::systems_frame_policy_binding::check(&root, &input).unwrap();
    let (bytes, owners) =
        crate::systems_artifact::generate_frame_user_extent_root(&checked).unwrap();
    assert_eq!(
        bytes,
        crate::systems_artifact::generate_frame_root_checked(&checked).unwrap()
    );
    let file = object::File::parse(bytes.as_slice()).unwrap();
    assert_eq!(
        file.symbol_by_name("topal_x86_frame_root_tokens_v1")
            .unwrap()
            .size(),
        owners.token_bytes
    );
    owners.validate().unwrap();
    assert_eq!(
        owners.roles.map(|o| (o.count, o.pins)),
        [(5, 1), (1, 1), (503, 1), (2, 2)]
    );
    for role in 0..4 {
        let mut invalid = owners.clone();
        invalid.roles[role].pins += 1;
        assert!(invalid.validate().is_err());
        let mut invalid = owners.clone();
        invalid.roles[role].count += 1;
        assert!(invalid.validate().is_err());
        let mut invalid = owners.clone();
        invalid.roles[role].slot = u64::MAX;
        assert!(invalid.validate().is_err());
    }
    let mut invalid = owners.clone();
    invalid.roles[1].slot = invalid.roles[0].slot;
    assert!(invalid.validate().is_err());
    for bytes in [0, 1, owners.token_bytes - 1] {
        let mut invalid = owners.clone();
        invalid.token_bytes = bytes;
        assert!(invalid.validate().is_err());
    }
}

#[test]
fn native_user_extent_walk_preserves_guarded_tables_and_checks_every_level() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: a guarded native component, not a kernel lease.
    let work = std::env::temp_dir().join(format!(
        "user-extent-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&work).unwrap();
    eprintln!(
        "unpublished user extent native component: {}",
        work.display()
    );
    fs::write(work.join("extent.ll"), llvm()).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    run(
        &work,
        "verify",
        Command::new(tools.tool_path("opt"))
            .args(["-passes=verify", "-disable-output"])
            .arg(work.join("extent.ll")),
    );
    run(
        &work,
        "compile",
        Command::new(tools.tool_path("llc"))
            .args([
                "-filetype=obj",
                "-O2",
                "-mcpu=x86-64",
                "-mattr=-mmx,-sse,-sse2,-avx,-avx2",
                "--frame-pointer=all",
                "-stack-size-section",
                "-function-sections",
                "--disable-tail-calls",
            ])
            .arg(work.join("extent.ll"))
            .arg("-o")
            .arg(work.join("extent.o")),
    );
    let bytes = fs::read(work.join("extent.o")).unwrap();
    let file = object::File::parse(bytes.as_slice()).unwrap();
    let imports = file
        .symbols()
        .filter(|s| s.is_undefined())
        .map(|s| s.name().unwrap().to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        imports,
        ["begin", "root", "table", "backing", "end", "fatal"]
            .map(|r| format!("topal_x86_user_extent_{r}_v1"))
            .into_iter()
            .collect()
    );
    let frames = crate::systems_library::stack_sizes(&file).unwrap();
    assert_eq!(frames.len(), 1);
    assert!(frames["topal_x86_owned_user_extent_v1"] > 0);
    assert!(frames["topal_x86_owned_user_extent_v1"] < 512);
    fs::write(
        work.join("component-cost.json"),
        serde_json::to_vec_pretty(&frames).unwrap(),
    )
    .unwrap();
    run(
        &work,
        "disassemble",
        Command::new(tools.tool_path("llvm-objdump"))
            .args(["-d", "--no-show-raw-insn"])
            .arg(work.join("extent.o")),
    );
    crate::systems_library::inspect_instructions(
        &fs::read_to_string(work.join("disassemble.stdout.log")).unwrap(),
    )
    .unwrap();
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/user_extent_component.c");
    fs::copy(&fixture, work.join("fixture.c")).unwrap();
    run(
        &work,
        "client-compile",
        Command::new("cc")
            .args(["-std=c11", "-O2", "-Wall", "-Wextra", "-Werror"])
            .arg(work.join("fixture.c"))
            .arg(work.join("extent.o"))
            .arg("-o")
            .arg(work.join("probe")),
    );
    run(&work, "client", &mut Command::new(work.join("probe")));
    let observed: serde_json::Value =
        serde_json::from_slice(&fs::read(work.join("client.stdout.log")).unwrap()).unwrap();
    assert_eq!(observed["passed"], true);
    assert_eq!(observed["permission_cases"], 8192);
    assert!(observed["calls"].as_u64().unwrap() > 8230);
}

#[test]
fn user_extent_target_reconstructs_actual_owner_offsets_and_verifies_natively() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: verification/object evidence only. The
    // owning op-8 gateway and actual privileged execution are still required.
    let root = crate::systems_frame_target::root_tests::root();
    let input = crate::systems_frame_target::root_tests::input();
    let checked = crate::systems_frame_policy_binding::check(&root, &input).unwrap();
    let (_, owners) = crate::systems_artifact::generate_frame_user_extent_root(&checked).unwrap();
    let text = target_llvm(&checked.component, &owners).unwrap();
    assert!(!text.contains("{{"));
    for role in owners.roles {
        assert!(text.contains(&format!(
            "ptr @topal_x86_frame_root_tokens_v1,i64 {}",
            role.slot * 24
        )));
    }
    let work = std::env::temp_dir().join(format!(
        "user-extent-target-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&work).unwrap();
    eprintln!("unpublished user extent target: {}", work.display());
    fs::write(work.join("target.ll"), text).unwrap();
    fs::write(work.join("owners.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "token_bytes": owners.token_bytes,
        "roles":owners.roles.map(|o| serde_json::json!({"slot":o.slot,"count":o.count,"pins":o.pins})),
        "scope":"compiler-derived owners, not serialized runtime authority"})).unwrap()).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    run(
        &work,
        "verify",
        Command::new(tools.tool_path("opt"))
            .args(["-passes=verify", "-disable-output"])
            .arg(work.join("target.ll")),
    );
    run(
        &work,
        "compile",
        Command::new(tools.tool_path("llc"))
            .args([
                "-filetype=obj",
                "-O2",
                "-mcpu=x86-64",
                "-mattr=-mmx,-sse,-sse2,-avx,-avx2",
                "--frame-pointer=all",
                "-stack-size-section",
                "-function-sections",
                "--disable-tail-calls",
            ])
            .arg(work.join("target.ll"))
            .arg("-o")
            .arg(work.join("target.o")),
    );
    let bytes = fs::read(work.join("target.o")).unwrap();
    let file = object::File::parse(bytes.as_slice()).unwrap();
    let frames = crate::systems_library::stack_sizes(&file).unwrap();
    assert!(frames["u.owner"] > 0 && frames["u.owner"] < 512);
    assert!(frames["u.syscall_context"] > 0 && frames["u.syscall_context"] < 512);
    fs::write(
        work.join("target-frames.json"),
        serde_json::to_vec_pretty(&frames).unwrap(),
    )
    .unwrap();
    run(
        &work,
        "disassemble",
        Command::new(tools.tool_path("llvm-objdump"))
            .args(["-d", "--no-show-raw-insn"])
            .arg(work.join("target.o")),
    );
    let disassembly = fs::read_to_string(work.join("disassemble.stdout.log")).unwrap();
    inspect_target_instructions(&disassembly).unwrap();
    assert!(inspect_target_instructions(&disassembly.replacen("%cr3,", "%cr2,", 1)).is_err());
    assert!(inspect_target_instructions(&disassembly.replacen("rdmsr", "wrmsr", 1)).is_err());
}
