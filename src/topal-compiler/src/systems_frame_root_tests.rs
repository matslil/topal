use crate::systems_tail_owner::run_tail_native_fixture as run;
use object::{Object, ObjectSection, ObjectSymbol, RelocationTarget};
use std::{fs, process::Command};

const ROOT: &str = include_str!("../../../linux-kernel/labs/qemu/x86_64/pre-frame-kernel-root.t");
const FRAME: &str = "  frame-allocator-library is lang systems source-library (\n    source is \"memory/frame-metadata.t\",\n    export is frame-policy,\n    interface is physical-frame-policy-tail-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";
pub(crate) fn root() -> String {
    let selected = ROOT.replacen(
        "lang systems artifact (\n",
        &format!("lang systems artifact (\n{FRAME}"),
        1,
    );
    assert_eq!(
        selected,
        include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t")
    );
    selected
}
pub(crate) fn input() -> crate::SystemsSourceLibraryInput {
    crate::SystemsSourceLibraryInput {
        source: "memory/frame-metadata.t".into(),
        source_name: "memory/frame-metadata.t".into(),
        text: include_str!("../../../linux-kernel/kernel/memory/frame-metadata.t").into(),
        modules: vec![],
    }
}

#[test]
fn frame_root_replaces_real_allocation_release_and_mapping_lifetimes() {
    execute(false);
}

#[test]
#[ignore = "requires TOPAL_PINNED_STATIC_USER_IMAGE, QEMU and GDB"]
fn owned_first_image_exit_retires_transferred_signal_state() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: ordinary unmodified application execution.
    execute_selected(true, true, true, Some(CallerFocus::Application));
}

#[test]
#[ignore = "requires a C compiler, QEMU and GDB"]
fn owned_source_fixture_checks_native_user_buffer_boundaries() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: ordinary source-compiled user requests.
    execute_selected(true, true, true, Some(CallerFocus::SourceBoundary));
}

#[test]
#[ignore = "requires a C compiler, QEMU and GDB"]
fn owned_source_fixture_checks_exec_vector_boundaries() {
    // TOPAL-SYSTEMS-USER-EXTENT-001 and STATIC-PROCESS-EXEC-ARGUMENTS-001.
    execute_selected(true, true, true, Some(CallerFocus::SourceExec));
}

#[test]
#[ignore = "requires a C compiler, QEMU and GDB"]
fn owned_source_fixture_checks_signal_transfer_boundaries() {
    // TOPAL-SYSTEMS-USER-EXTENT-001; STATIC-PROCESS-SIGNAL-{ACTION,MASK}-001.
    execute_selected(true, true, true, Some(CallerFocus::SourceSignals));
}

#[test]
#[ignore = "requires a C compiler, QEMU and GDB"]
fn owned_source_fixture_checks_poll_transfer_boundaries() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: source-issued descriptor readiness.
    execute_selected(true, true, true, Some(CallerFocus::SourcePoll));
}

#[test]
#[ignore = "requires a C compiler, QEMU and GDB"]
fn owned_source_fixture_checks_pipe_transfer_boundaries() {
    // TOPAL-SYSTEMS-USER-EXTENT-001; STATIC-PROCESS-PIPE-{001,TRANSFER-001}.
    execute_selected(true, true, true, Some(CallerFocus::SourcePipes));
}

#[test]
#[ignore = "requires a C compiler, QEMU and GDB"]
fn owned_source_fixture_checks_storage_transfer_boundaries() {
    // TOPAL-SYSTEMS-USER-EXTENT-001; VOLATILE/POSITIONAL-FILE-TRANSFER-001.
    execute_selected(true, true, true, Some(CallerFocus::SourceStorage));
}

#[test]
#[ignore = "requires the bounded QEMU/GDB metadata cohort"]
fn owned_source_fixture_checks_metadata_lifetimes() {
    execute_selected(true, true, true, Some(CallerFocus::SourceMetadata));
}

#[test]
#[ignore = "requires the bounded QEMU/GDB process-MM preparation cohort"]
fn process_mm_preparation_copies_and_retires_real_frame_backing() {
    // TOPAL-SYSTEMS-PROCESS-MM-001: native preparation only, not fork activation.
    execute_selected(true, true, true, Some(CallerFocus::SourceProcessMemory));
}

#[test]
#[ignore = "requires a C compiler, QEMU and GDB"]
fn owned_source_fixture_checks_serial_transfer_boundaries() {
    // TOPAL-SYSTEMS-USER-EXTENT-001; STATIC-PROCESS-FD-TABLE-001.
    execute_selected(true, true, true, Some(CallerFocus::SourceSerial));
}

#[test]
#[ignore = "requires a C compiler, QEMU and GDB"]
fn owned_source_fixture_checks_stat_transfer_boundaries() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: all selected metadata preparation variants.
    execute_selected(true, true, true, Some(CallerFocus::SourceStat));
}

pub(super) fn execute(memory: bool) -> std::path::PathBuf {
    execute_selected(memory, false, false, None)
}

pub(crate) fn execute_user_extents() -> std::path::PathBuf {
    execute_selected(true, true, false, None)
}

pub(crate) fn execute_user_extent_callers() -> std::path::PathBuf {
    execute_selected(true, true, true, None)
}

pub(crate) fn execute_user_extent_startup() -> std::path::PathBuf {
    execute_selected(true, true, true, Some(CallerFocus::Startup))
}

pub(crate) fn execute_user_extent_shell() -> std::path::PathBuf {
    execute_selected(true, true, true, Some(CallerFocus::Shell))
}

pub(crate) fn execute_user_extent_stat() -> std::path::PathBuf {
    execute_selected(true, true, true, Some(CallerFocus::Stat))
}

pub(crate) fn execute_user_extent_statat() -> std::path::PathBuf {
    execute_selected(true, true, true, Some(CallerFocus::StatAt))
}

pub(crate) fn execute_user_extent_shell_io() -> std::path::PathBuf {
    execute_selected(true, true, true, Some(CallerFocus::ShellIo))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CallerFocus {
    Application,
    SourceBoundary,
    SourceExec,
    SourceSignals,
    SourcePoll,
    SourcePipes,
    SourceStorage,
    SourceMetadata,
    SourceProcessMemory,
    SourceSerial,
    SourceStat,
    Startup,
    Shell,
    Stat,
    StatAt,
    ShellIo,
}

fn execute_selected(
    memory: bool,
    user_extents: bool,
    owned_callers: bool,
    focus: Option<CallerFocus>,
) -> std::path::PathBuf {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: complete existing boot IR, not
    // a smaller constant probe. Canonical publication remains unqualified.
    let root = root();
    let input = input();
    let checked = crate::systems_frame_policy_binding::check(&root, &input).unwrap();
    let (bytes, owners) = if user_extents {
        let (bytes, owners) =
            crate::systems_artifact::generate_frame_user_extent_root(&checked).unwrap();
        (bytes, Some(owners))
    } else {
        (
            crate::systems_artifact::generate_frame_policy_root_object(&root, &input).unwrap(),
            None,
        )
    };
    assert!(crate::systems_artifact::generate_root_object(&checked.program).is_err());
    let file = object::File::parse(bytes.as_slice()).unwrap();
    let tokens = file
        .symbol_by_name("topal_x86_frame_root_tokens_v1")
        .unwrap();
    assert!(tokens.size() > 24 && tokens.size() % 24 == 0);
    let mut calls = std::collections::BTreeMap::new();
    for section in file.sections() {
        let data = section.data().unwrap();
        for (offset, r) in section.relocations() {
            if offset == 0 || data.get(offset as usize - 1) != Some(&0xe8) {
                continue;
            }
            if let RelocationTarget::Symbol(index) = r.target() {
                let name = file.symbol_by_index(index).unwrap().name().unwrap();
                *calls.entry(name.to_owned()).or_insert(0) += 1;
            }
        }
    }
    assert!(!calls.contains_key("topal_x86_systems_allocate_physical_frames"));
    assert!(
        !calls.contains_key(crate::systems_provider_object::X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL)
    );
    assert!(calls["topal_x86_frame_gateway_v1"] > 30);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let prefix = if focus == Some(CallerFocus::SourceProcessMemory) {
        "process-mm-preparation"
    } else if focus == Some(CallerFocus::SourceMetadata) {
        "user-extent-source-metadata"
    } else if focus == Some(CallerFocus::SourceStat) {
        "user-extent-source-stat"
    } else if focus == Some(CallerFocus::SourceSerial) {
        "user-extent-source-serial"
    } else if focus == Some(CallerFocus::SourceStorage) {
        "user-extent-source-storage"
    } else if focus == Some(CallerFocus::SourcePipes) {
        "user-extent-source-pipes"
    } else if focus == Some(CallerFocus::SourcePoll) {
        "user-extent-source-poll"
    } else if focus == Some(CallerFocus::SourceSignals) {
        "user-extent-source-signals"
    } else if focus == Some(CallerFocus::SourceExec) {
        "user-extent-source-exec"
    } else if focus == Some(CallerFocus::SourceBoundary) {
        "user-extent-source"
    } else if focus == Some(CallerFocus::Application) {
        "user-extent-application"
    } else if focus == Some(CallerFocus::ShellIo) {
        "user-extent-shell-io"
    } else if focus == Some(CallerFocus::StatAt) {
        "user-extent-statat"
    } else if focus == Some(CallerFocus::Stat) {
        "user-extent-stat"
    } else if focus == Some(CallerFocus::Shell) {
        "user-extent-shell"
    } else if focus == Some(CallerFocus::Startup) {
        "user-extent-startup"
    } else if owned_callers {
        "user-extent-callers"
    } else if user_extents {
        "user-extent-root"
    } else if memory {
        "frame-memory"
    } else {
        "frame-root"
    };
    let parent = if matches!(
        focus,
        Some(
            CallerFocus::Application
                | CallerFocus::SourceBoundary
                | CallerFocus::SourceExec
                | CallerFocus::SourceSignals
                | CallerFocus::SourcePoll
                | CallerFocus::SourcePipes
                | CallerFocus::SourceStorage
                | CallerFocus::SourceMetadata
                | CallerFocus::SourceProcessMemory
                | CallerFocus::SourceSerial
                | CallerFocus::SourceStat
        )
    ) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../linux-kernel/.cache/x86_64");
        fs::create_dir_all(&path).unwrap();
        path
    } else {
        std::env::temp_dir()
    };
    let work = parent.join(format!("{prefix}-{}-{nonce}", std::process::id()));
    fs::create_dir(&work).unwrap();
    eprintln!("actual frame source root: {}", work.display());
    fs::write(work.join("root.t"), root).unwrap();
    fs::write(work.join("root.o"), &bytes).unwrap();
    if let Some(owners) = &owners {
        fs::write(
            work.join("user-owners.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
            "token_bytes":owners.token_bytes,"roles":owners.roles.map(|o| serde_json::json!({
                "slot":o.slot,"count":o.count,"pins":o.pins}))}))
            .unwrap(),
        )
        .unwrap();
    }
    fs::write(
        work.join("root-tokens.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"bytes":tokens.size()})).unwrap(),
    )
    .unwrap();
    fs::write(
        work.join("root-calls.json"),
        serde_json::to_vec_pretty(&calls).unwrap(),
    )
    .unwrap();
    fs::write(
        work.join("source-obligation.json"),
        serde_json::to_vec_pretty(&checked.source).unwrap(),
    )
    .unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    run(
        &work,
        "root-disassembly",
        Command::new(tools.tool_path("llvm-objdump"))
            .args(["-d", "--no-show-raw-insn"])
            .arg(work.join("root.o")),
    );
    let target = if let Some(owners) = &owners {
        if focus == Some(CallerFocus::SourceProcessMemory) {
            super::process_mm::target_llvm(&checked.component, owners)
        } else if owned_callers {
            crate::systems_user_extent::caller_target_llvm(&checked.component, owners)
        } else {
            crate::systems_user_extent::target_llvm(&checked.component, owners)
        }
    } else if memory {
        super::memory_llvm(&checked.component, tokens.size())
    } else {
        super::root_llvm(&checked.component, tokens.size())
    }
    .unwrap();
    fs::write(work.join("target.ll"), &target).unwrap();
    run(
        &work,
        "root-target-verify",
        Command::new(tools.tool_path("opt"))
            .args(["-passes=verify", "-disable-output"])
            .arg(work.join("target.ll")),
    );
    build_guest(
        &work,
        &checked,
        &tools,
        memory,
        user_extents,
        owned_callers,
        focus,
    );
    work
}

fn build_guest(
    work: &std::path::Path,
    checked: &crate::systems_frame_policy_binding::Checked,
    tools: &crate::LlvmTools,
    memory: bool,
    user_extents: bool,
    owned_callers: bool,
    focus: Option<CallerFocus>,
) {
    use object::ObjectSegment;
    let fixture = work.join("static-elf-abi-fixture.elf");
    let fixture_bytes = if matches!(
        focus,
        Some(CallerFocus::SourceMetadata | CallerFocus::SourceProcessMemory)
    ) {
        crate::systems_user_extent::source_fixture::build_metadata(work, tools)
    } else if focus == Some(CallerFocus::SourceStat) {
        crate::systems_user_extent::source_fixture::build_stat(work, tools)
    } else if focus == Some(CallerFocus::SourceSerial) {
        crate::systems_user_extent::source_fixture::build_serial(work, tools)
    } else if focus == Some(CallerFocus::SourceStorage) {
        crate::systems_user_extent::source_fixture::build_storage(work, tools)
    } else if focus == Some(CallerFocus::SourcePipes) {
        crate::systems_user_extent::source_fixture::build_pipes(work, tools)
    } else if focus == Some(CallerFocus::SourcePoll) {
        crate::systems_user_extent::source_fixture::build_poll(work, tools)
    } else if focus == Some(CallerFocus::SourceSignals) {
        crate::systems_user_extent::source_fixture::build_signals(work, tools)
    } else if focus == Some(CallerFocus::SourceExec) {
        crate::systems_user_extent::source_fixture::build_exec(work, tools)
    } else if focus == Some(CallerFocus::SourceBoundary) {
        crate::systems_user_extent::source_fixture::build(work, tools)
    } else if focus == Some(CallerFocus::Application) {
        use sha2::{Digest, Sha256};
        let path = std::env::var_os("TOPAL_PINNED_STATIC_USER_IMAGE")
            .expect("provide the pinned BusyBox ELF through TOPAL_PINNED_STATIC_USER_IMAGE");
        let bytes = fs::read(path).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd"
        );
        bytes
    } else if owned_callers {
        let fixture = super::memory_tests::caller_fixture();
        fs::write(
            work.join("caller-fixture-sites.json"),
            serde_json::to_vec_pretty(&fixture.assertions).unwrap(),
        )
        .unwrap();
        fs::write(
            work.join("caller-observation.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                    "startup_only":focus == Some(CallerFocus::Startup),
                    "shell_only":focus == Some(CallerFocus::Shell),
                    "stat_only":focus == Some(CallerFocus::Stat),
                    "statat_only":focus == Some(CallerFocus::StatAt),
                    "shell_io_only":focus == Some(CallerFocus::ShellIo),
                    "startup_completion":fixture.startup_completed,
                    "shell_completion":fixture.shell_completed,
                    "stat_completion":fixture.stat_completed,"stat_pages":fixture.stat_pages,
                    "statat_completion":fixture.statat_completed,"statat_pages":fixture.statat_pages,
                    "write_pages":fixture.write_pages,"write_lengths":super::memory_tests::WRITE_LENGTHS,
                    "file_cases":fixture.file_cases,"shell_io_cases":fixture.shell_io_cases,
                    "command_cases":fixture.command_cases}))
            .unwrap(),
        )
        .unwrap();
        fixture.image
    } else if memory {
        super::memory_tests::fixture()
    } else {
        crate::qualified_x86_64_static_user_image_fixture()
    };
    fs::write(&fixture, fixture_bytes).unwrap();
    let baseline = crate::publish_systems_source(
        ROOT,
        &crate::CompileOptions {
            source_name: "arch/x86_64/toolchain-gate.t".into(),
            output: work.join("baseline"),
            emit: crate::Emit::Executable,
            llvm_tools: None,
            library_root: std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../linux-kernel/kernel"),
            target: crate::TargetSelection {
                target: Some("x86_64-unknown-none".into()),
                board: Some("topal-qemu-pc-q35-10.2".into()),
                ..Default::default()
            },
            optimization: Default::default(),
            standard_library: None,
            initial_user_image: None,
            static_user_image: Some(fixture),
        },
    )
    .unwrap();
    let owner = &checked.component;
    for (role, text) in [
        ("source", owner.runtime.source_step.llvm.as_str()),
        ("runtime", &owner.runtime.llvm),
        ("owner", &owner.llvm),
        ("controller", &checked.controller),
    ] {
        fs::write(work.join(format!("{role}.ll")), text).unwrap();
    }
    fs::write(
        work.join("dispatch.ll"),
        if focus == Some(CallerFocus::SourceProcessMemory) {
            super::process_mm::dispatch_llvm().unwrap()
        } else if user_extents {
            crate::systems_user_extent::dispatch_llvm().unwrap()
        } else if memory {
            super::memory_dispatch()
        } else {
            format!(
                "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n{}",
                crate::X86_SYSTEMS_DATA_LAYOUT,
                include_str!("runtime/kernel_frame_dispatch.ll")
            )
        },
    )
    .unwrap();
    fs::write(work.join("boundary.o"), &owner.runtime.boundary).unwrap();
    let mut roles = vec![
        "source",
        "runtime",
        "owner",
        "controller",
        "target",
        "dispatch",
    ];
    if user_extents {
        fs::write(work.join("extent.ll"), crate::systems_user_extent::llvm()).unwrap();
        if !owned_callers {
            fs::write(
                work.join("extent-probe.ll"),
                format!(
                    "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n{}",
                    crate::X86_SYSTEMS_DATA_LAYOUT,
                    include_str!("../../../linux-kernel/labs/qemu/x86_64/user_extent_probe.ll")
                ),
            )
            .unwrap();
        }
        fs::write(
            work.join("extent-bridge.o"),
            crate::systems_provider_object::user_extent_bridge::object().unwrap(),
        )
        .unwrap();
        roles.push("extent");
        if owned_callers {
            fs::write(
                work.join("extent-callers.o"),
                crate::systems_provider_object::user_extent_bridge::caller_object().unwrap(),
            )
            .unwrap();
        } else {
            roles.push("extent-probe");
        }
    }
    for role in roles {
        run(
            work,
            &format!("{role}-llc"),
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
                .arg(work.join(format!("{role}.ll")))
                .arg("-o")
                .arg(work.join(format!("{role}.o"))),
        );
    }
    let inner = crate::systems_tail_runtime::inspection::inspect(
        &fs::read(work.join("source.o")).unwrap(),
        &fs::read(work.join("runtime.o")).unwrap(),
    )
    .unwrap();
    let outer = crate::systems_tail_owner::inspection::inspect(
        &fs::read(work.join("owner.o")).unwrap(),
        &inner,
    )
    .unwrap();
    let controller = fs::read(work.join("controller.o")).unwrap();
    let target = fs::read(work.join("target.o")).unwrap();
    let dispatch = fs::read(work.join("dispatch.o")).unwrap();
    let mut frame = if user_extents {
        super::inspection::inspect_user_extents(
            &controller,
            &target,
            &dispatch,
            &fs::read(work.join("extent.o")).unwrap(),
            &outer,
        )
    } else {
        super::inspection::inspect_gateway(&controller, &target, &dispatch, &outer)
    }
    .unwrap();
    if memory {
        let body = frame["roots"]["topal_frame_kernel_dispatch_v1"]
            .as_u64()
            .unwrap();
        let total = body.checked_add(80).unwrap();
        assert!(total <= 4096);
        frame["syscall_adapter"] = serde_json::json!({
            "zero_alignment_bytes": 8,
            "combined_stack_bound_bytes": total,
            "physical_retirement_bytes": 4032 + 48,
            "return_and_adapter_bytes": 16,
            "caller_reserve_bytes": 4096,
        });
        if owned_callers {
            use crate::systems_provider_object::user_extent_bridge::{
                CALLER_OUTER_BYTES, CALLER_RETIRED_BYTES,
            };
            let complete = body + 64 + CALLER_OUTER_BYTES;
            assert!(complete <= 4096 && CALLER_RETIRED_BYTES + 48 + CALLER_OUTER_BYTES <= 4096);
            frame["syscall_adapter"]["physical_retirement_bytes"] =
                serde_json::json!(CALLER_RETIRED_BYTES + 48);
            frame["user_extent_callers"] = serde_json::json!({"complete_stack_bytes":complete,
                "outer_bytes":CALLER_OUTER_BYTES,"inner_retirement_bytes":CALLER_RETIRED_BYTES,
                "complete_physical_bytes":CALLER_RETIRED_BYTES+48+CALLER_OUTER_BYTES,"reserve_bytes":4096});
        } else if user_extents {
            let bytes = fs::read(work.join("extent-probe.o")).unwrap();
            let file = object::File::parse(bytes.as_slice()).unwrap();
            let frames = crate::systems_library::stack_sizes(&file).unwrap();
            let probe = frames["topal_x86_user_extent_probe_v1"] + 8 + frames["probe.check"] + 8;
            let full = total + probe + 16;
            assert!(full <= 4096);
            frame["user_extent_probe"] = serde_json::json!({"frames":frames,
                "probe_call_path_bytes":probe,"hook_saved_result_and_bridge_return_bytes":16,
                "complete_stack_bytes":full,"reserve_bytes":4096,
                "probe_completed_caller_retirement_bytes":4096});
        }
    }
    fs::write(
        work.join("frame-inspection.json"),
        serde_json::to_vec_pretty(&frame).unwrap(),
    )
    .unwrap();
    fs::write(
        work.join("gateway.o"),
        (if owned_callers {
            crate::systems_provider_object::tail_boundary::frame_gateway::owned_user_extent_component_object
        } else if user_extents {
            crate::systems_provider_object::tail_boundary::frame_gateway::user_extent_component_object
        } else if memory {
            crate::systems_provider_object::tail_boundary::frame_gateway::memory_component_object
        } else {
            crate::systems_provider_object::tail_boundary::frame_gateway::component_object
        })(
            frame["roots"]["topal_frame_kernel_dispatch_v1"]
                .as_u64()
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut dependencies = checked.program.clone();
    dependencies.frame_allocator_library = None;
    let provider = (if owned_callers {
        crate::systems_provider_object::generate_owned_user_extent_provider(checked)
    } else if user_extents {
        crate::systems_provider_object::generate_user_extent_probe_provider(checked)
    } else if memory {
        crate::systems_provider_object::generate_frame_memory_provider(checked)
    } else {
        crate::generate_x86_64_systems_provider_object(&dependencies)
    })
    .unwrap();
    fs::write(work.join("provider.o"), &provider.bytes).unwrap();
    let access_audit_selected = matches!(
        focus,
        Some(
            CallerFocus::Application
                | CallerFocus::SourceBoundary
                | CallerFocus::SourcePoll
                | CallerFocus::SourcePipes
                | CallerFocus::SourceStorage
                | CallerFocus::SourceMetadata
                | CallerFocus::SourceProcessMemory
                | CallerFocus::SourceSerial
                | CallerFocus::SourceStat
        )
    );
    if access_audit_selected {
        fs::write(
            work.join("process-byte-origins.json"),
            serde_json::to_vec_pretty(provider.byte_origins.as_ref().unwrap()).unwrap(),
        )
        .unwrap();
        fs::write(
            work.join("extent-call-inventory.json"),
            serde_json::to_vec_pretty(provider.extent_inventory.as_ref().unwrap()).unwrap(),
        )
        .unwrap();
        run(
            work,
            "extent-call-disassembly",
            Command::new(tools.tool_path("llvm-objdump"))
                .args(["-d", "--no-show-raw-insn"])
                .arg(work.join("provider.o")),
        );
        run(
            work,
            "extent-call-inspection",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/inspect_call_inventory.py"),
                )
                .arg(work),
        );
        run(
            work,
            "transfer-guard-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_transfer_guards.py"),
                )
                .arg(work),
        );
    }
    if owned_callers {
        // Retain actual predecessor factories for separate byte comparison;
        // the new component selection cannot qualify the old layout by itself.
        fs::write(
            work.join("predecessor-provider.o"),
            crate::systems_provider_object::generate_user_extent_probe_provider(checked)
                .unwrap()
                .bytes,
        )
        .unwrap();
        fs::write(work.join("predecessor-gateway.o"),
            crate::systems_provider_object::tail_boundary::frame_gateway::user_extent_component_object(
                frame["roots"]["topal_frame_kernel_dispatch_v1"].as_u64().unwrap())
                .unwrap()).unwrap();
    }
    let legacy = &baseline.directory;
    let objects = [
        "namespace-library.o",
        "inode-mode-library.o",
        "ram-page-pool-library.o",
        "file-position-library.o",
        "region-library.o",
        "time-library.o",
        "image-library.o",
        "process-identity-library.o",
        "dispatch-library.o",
        "kernel-library.o",
    ];
    let mut reserved = 0x4000000_u64;
    for label in ["layout", "link"] {
        let mut link = Command::new(tools.tool_path("rust-lld"));
        link.args([
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
        link.arg(format!(
            "--defsym=topal_x86_kernel_reservation_end_v1={reserved}"
        ));
        for name in [
            "root",
            "source",
            "runtime",
            "owner",
            "controller",
            "target",
            "dispatch",
            "gateway",
            "boundary",
            "provider",
        ] {
            link.arg(work.join(format!("{name}.o")));
        }
        if user_extents {
            for name in [
                "extent",
                if owned_callers {
                    "extent-callers"
                } else {
                    "extent-probe"
                },
                "extent-bridge",
            ] {
                link.arg(work.join(format!("{name}.o")));
            }
        }
        for name in objects {
            let path = legacy.join(name);
            assert!(path.is_file(), "missing {}", path.display());
            link.arg(path);
        }
        link.arg("-o").arg(work.join("kernel.elf"));
        run(work, label, &mut link);
        let bytes = fs::read(work.join("kernel.elf")).unwrap();
        let elf = object::File::parse(bytes.as_slice()).unwrap();
        let end = elf
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
    let kernel = fs::read(work.join("kernel.elf")).unwrap();
    if access_audit_selected {
        run(
            work,
            "process-access-inventory",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/inventory_process_accesses.py"),
                )
                .arg(work)
                .arg(tools.tool_path("llvm-objdump")),
        );
        run(
            work,
            "callback-bindings",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/inspect_callback_bindings.py"),
                )
                .arg(work)
                .arg(tools.tool_path("llvm-objdump")),
        );
        run(
            work,
            "process-access-origins",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/trace_process_origins.py"),
                )
                .arg(work)
                .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")),
        );
        run(
            work,
            "fixed-transfer-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_fixed_transfers.py"),
                )
                .arg(work),
        );
        run(
            work,
            "copy-transfer-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_copy_transfers.py"),
                )
                .arg(work),
        );
        run(
            work,
            "staged-transfer-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_staged_transfers.py"),
                )
                .arg(work),
        );
        run(
            work,
            "additional-scalar-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_additional_scalars.py"),
                )
                .arg(work),
        );
        run(
            work,
            "staging-copy-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_staging_copies.py"),
                )
                .arg(work),
        );
        run(
            work,
            "pipe-fixed-output-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_pipe_fixed_outputs.py"),
                )
                .arg(work),
        );
        run(
            work,
            "stat-output-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_stat_outputs.py"),
                )
                .arg(work),
        );
        run(
            work,
            "serial-prefix-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_serial_prefix.py"),
                )
                .arg(work),
        );
        run(
            work,
            "pipe-loop-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_pipe_loops.py"),
                )
                .arg(work),
        );
        run(
            work,
            "file-input-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_file_inputs.py"),
                )
                .arg(work),
        );
        run(
            work,
            "warning-input-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_warning_inputs.py"),
                )
                .arg(work),
        );
        run(
            work,
            "random-output-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_random_outputs.py"),
                )
                .arg(work),
        );
        run(
            work,
            "stack-retirement-audit",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/audit_stack_retirement.py"),
                )
                .arg(work),
        );
    }
    fs::write(
        work.join("bzImage"),
        crate::systems_boot_image::unpublished_component_fixture(&kernel).unwrap(),
    )
    .unwrap();
    let resource = crate::systems_boot_resource::generate_initial_boot_resource(
        &fs::read(&baseline.initial_user_image).unwrap(),
        &fs::read(baseline.static_user_image.unwrap()).unwrap(),
    )
    .unwrap();
    fs::write(work.join("initramfs.cpio"), resource).unwrap();
    run(
        work,
        "guest",
        Command::new("python3")
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                if matches!(
                    focus,
                    Some(
                        CallerFocus::SourceBoundary
                            | CallerFocus::SourceExec
                            | CallerFocus::SourceSignals
                            | CallerFocus::SourcePoll
                            | CallerFocus::SourcePipes
                            | CallerFocus::SourceStorage
                            | CallerFocus::SourceMetadata
                            | CallerFocus::SourceProcessMemory
                            | CallerFocus::SourceSerial
                            | CallerFocus::SourceStat
                    )
                ) {
                    "../../linux-kernel/labs/qemu/x86_64/user_extent_source_inspect.py"
                } else if focus == Some(CallerFocus::Application) {
                    "../../linux-kernel/labs/qemu/x86_64/user_extent_application_inspect.py"
                } else if owned_callers {
                    "../../linux-kernel/labs/qemu/x86_64/user_extent_callers_inspect.py"
                } else if user_extents {
                    "../../linux-kernel/labs/qemu/x86_64/user_extent_inspect.py"
                } else if memory {
                    "../../linux-kernel/labs/qemu/x86_64/frame_memory_inspect.py"
                } else {
                    "../../linux-kernel/labs/qemu/x86_64/frame_root_inspect.py"
                },
            ))
            .arg(work)
            .envs(
                (focus == Some(CallerFocus::SourceProcessMemory))
                    .then_some(("TOPAL_TEST_PROCESS_MM_PREPARATION", "metadata")),
            ),
    );
    if access_audit_selected {
        run(
            work,
            "process-access-coverage",
            Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/tools/user-memory/inventory_process_accesses.py"),
                )
                .arg("--verify-coverage")
                .arg(work),
        );
    }
}
