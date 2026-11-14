use crate::systems_tail_owner::run_tail_native_fixture as run;
use object::{Object, ObjectSection, ObjectSymbol, RelocationTarget};
use std::{fs, process::Command};

#[path = "systems_frame_root_link_tests.rs"]
mod closed_link;
#[path = "systems_hierarchy_backing_tests.rs"]
mod hierarchy_backing;
#[path = "systems_process_mm_fault_tests.rs"]
mod policy_failure;

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
#[ignore = "requires pinned BusyBox, QEMU and GDB for the process-MM application corpus"]
fn process_mm_preserves_pinned_busybox_workloads() {
    // TOPAL-SYSTEMS-PROCESS-MM-001: unchanged third-party application and
    // existing command/output contracts on the experimental MM selection.
    execute_profiled(
        KernelProfile {
            memory: true,
            user_extents: true,
            owned_callers: true,
            process_memory: true,
            policy_failure: None,
        },
        Some(CallerFocus::Application),
    );
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
#[ignore = "requires the bounded QEMU/GDB independent process-MM cohort"]
fn process_mm_fork_handoff_and_exit_preserve_independent_heap_owners() {
    // TOPAL-SYSTEMS-PROCESS-MM-001: source-built ordinary fork/yield/brk/wait.
    execute_selected(true, true, true, Some(CallerFocus::SourceProcessOwners));
}

#[test]
#[ignore = "requires the bounded QEMU/GDB pre-import caller prefix"]
fn process_mm_prelude_missing_stat_executes_owned_read_and_retires_caller() {
    // TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001.
    execute_selected(true, true, true, Some(CallerFocus::SourcePreludeStat));
}

#[test]
#[ignore = "requires the bounded QEMU/GDB child-exec process-MM cohort"]
fn process_mm_child_exec_preserves_inactive_parent_and_fresh_child_heap() {
    // TOPAL-SYSTEMS-PROCESS-MM-001: ordinary exec/startup/yield/wait.
    execute_selected(true, true, true, Some(CallerFocus::SourceProcessExec));
}

#[test]
fn process_mm_closed_link_reconstructs_complete_private_image() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001 / KERNEL-IMAGE-PLAN-001.
    // Compile/link/reconstruct only; native execution is a separate test.
    execute_profiled_mode(
        KernelProfile {
            memory: true,
            user_extents: true,
            owned_callers: true,
            process_memory: true,
            policy_failure: None,
        },
        Some(CallerFocus::SourceProcessExec),
        false,
    );
}

#[test]
#[ignore = "requires the bounded QEMU/GDB physical allocation-refusal cohort"]
fn process_mm_allocation_refusal_preserves_owner_and_pid_history() {
    // TOPAL-SYSTEMS-PROCESS-MM-001: real physical shortage through ordinary
    // fork, then successful one-page and empty-heap creation on the same owner.
    execute_selected(true, true, true, Some(CallerFocus::SourceProcessAllocation));
}

#[test]
#[ignore = "requires the bounded QEMU/GDB maximum-copy geometry cohort"]
fn process_mm_maximum_fork_and_clone_copy_survive_empty_and_one_page_handoff() {
    // TOPAL-SYSTEMS-PROCESS-MM-001: full 519-page eager copy, independent
    // ordinary resize and handoff through zero/one-page child owners.
    for clone in [false, true] {
        execute_selected(
            true,
            true,
            true,
            Some(CallerFocus::SourceProcessGeometry(clone)),
        );
    }
}

#[test]
#[ignore = "requires the bounded QEMU/GDB process-MM ordinary-interface regression matrix"]
fn process_mm_preserves_source_built_application_interfaces() {
    // TOPAL-SYSTEMS-PROCESS-MM-001; TOPAL-SYSTEMS-USER-EXTENT-001:
    // The existing ordinary syscall programs and interface assertions run
    // unchanged on the independently owned process-MM kernel selection.
    let cohorts = [
        ("transfers", CallerFocus::SourceBoundary),
        ("exec", CallerFocus::SourceExec),
        ("signals", CallerFocus::SourceSignals),
        ("poll", CallerFocus::SourcePoll),
        ("pipes", CallerFocus::SourcePipes),
        ("storage", CallerFocus::SourceStorage),
        ("metadata", CallerFocus::SourceMetadata),
        ("serial", CallerFocus::SourceSerial),
        ("stat", CallerFocus::SourceStat),
    ];
    // A failed cohort can be rerun without discarding the other native
    // records. With no explicit selection the complete matrix still runs;
    // the archive independently requires all nine successful cohorts.
    let selection = std::env::var("TOPAL_TEST_PROCESS_MM_INTERFACES").ok();
    let selected: Vec<_> = selection
        .as_deref()
        .map(|value| value.split(',').collect())
        .unwrap_or_else(|| cohorts.iter().map(|(name, _)| *name).collect());
    assert!(!selected.is_empty());
    for (index, name) in selected.iter().enumerate() {
        assert!(cohorts.iter().any(|(candidate, _)| candidate == name));
        assert!(!selected[..index].contains(name));
    }
    for (name, focus) in cohorts {
        if !selected.contains(&name) {
            continue;
        }
        execute_profiled(
            KernelProfile {
                memory: true,
                user_extents: true,
                owned_callers: true,
                process_memory: true,
                policy_failure: None,
            },
            Some(focus),
        );
    }
}

#[test]
#[ignore = "requires the bounded QEMU/GDB live-child cancellation cohort"]
fn process_mm_parent_exit_retires_ready_and_zombie_relationships() {
    // TOPAL-SYSTEMS-PROCESS-MM-001: ordinary yield or child exit, followed
    // by outer-owner termination without reaping or resuming that child.
    for zombie in [false, true] {
        for early in [false, true] {
            for group in [false, true] {
                execute_selected(
                    true,
                    true,
                    true,
                    Some(CallerFocus::SourceProcessRelationship(early, group, zombie)),
                );
            }
        }
    }
}

#[test]
#[ignore = "requires the bounded QEMU/GDB live-child cancellation cohort"]
fn process_mm_parent_exit_cancels_blocked_child_and_all_resources() {
    // TOPAL-SYSTEMS-PROCESS-MM-001: actual pipe wait, both exit operations,
    // initial-shell and phase-zero replacement startup, zero/nonzero statuses.
    for early in [false, true] {
        for group in [false, true] {
            execute_selected(
                true,
                true,
                true,
                Some(CallerFocus::SourceProcessCancel(early, group, false)),
            );
        }
    }
}

#[test]
#[ignore = "requires the bounded QEMU/GDB blocked-writer cancellation cohort"]
fn process_mm_parent_exit_cancels_blocked_writer_and_dirty_fifo() {
    // TOPAL-SYSTEMS-PROCESS-MM-001; STATIC-PROCESS-PIPE-001:
    // ordinary full-pipe write, inactive last writer, parent reader aliases.
    for early in [false, true] {
        for group in [false, true] {
            execute_selected(
                true,
                true,
                true,
                Some(CallerFocus::SourceProcessCancel(early, group, true)),
            );
        }
    }
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
    SourceProcessOwners,
    SourcePreludeStat,
    SourceProcessExec,
    SourceProcessAllocation,
    SourceProcessGeometry(bool),
    SourceProcessRelationship(bool, bool, bool),
    SourceProcessCancel(bool, bool, bool),
    SourceSerial,
    SourceStat,
    Startup,
    Shell,
    Stat,
    StatAt,
    ShellIo,
}

fn process_memory_focus(focus: Option<CallerFocus>) -> bool {
    matches!(
        focus,
        Some(
            CallerFocus::SourceProcessOwners
                | CallerFocus::SourcePreludeStat
                | CallerFocus::SourceProcessExec
                | CallerFocus::SourceProcessAllocation
                | CallerFocus::SourceProcessGeometry(..)
                | CallerFocus::SourceProcessRelationship(..)
                | CallerFocus::SourceProcessCancel(..)
        )
    )
}

fn execute_selected(
    memory: bool,
    user_extents: bool,
    owned_callers: bool,
    focus: Option<CallerFocus>,
) -> std::path::PathBuf {
    execute_profiled(
        KernelProfile {
            memory,
            user_extents,
            owned_callers,
            process_memory: process_memory_focus(focus),
            policy_failure: None,
        },
        focus,
    )
}

#[derive(Clone, Copy)]
struct KernelProfile {
    memory: bool,
    user_extents: bool,
    owned_callers: bool,
    process_memory: bool,
    policy_failure: Option<policy_failure::Failure>,
}

fn execute_profiled(profile: KernelProfile, focus: Option<CallerFocus>) -> std::path::PathBuf {
    execute_profiled_mode(profile, focus, true)
}

fn execute_profiled_mode(
    profile: KernelProfile,
    focus: Option<CallerFocus>,
    native: bool,
) -> std::path::PathBuf {
    execute_profiled_input(profile, focus, native, None, None, None, None)
}

/// Compile a fresh ordinary source image with the existing private owning-MM
/// kernel. The caller separately executes its serial-only guest test; this
/// builder supplies neither production publication nor a protected context.
pub(crate) fn prepare_owned_source_fixture(
    image: &[u8],
    source_work: &std::path::Path,
) -> std::path::PathBuf {
    execute_profiled_input(
        KernelProfile {
            memory: true,
            user_extents: true,
            owned_callers: true,
            process_memory: true,
            policy_failure: None,
        },
        Some(CallerFocus::SourceBoundary),
        false,
        Some((image, source_work)),
        None,
        None,
        None,
    )
}

/// Same complete private producer with an explicit purpose-preserving source
/// request; compilation remains separate from ordinary serial-only execution.
pub(crate) fn prepare_provider_backing_source_fixture(
    image: &[u8],
    source_work: &std::path::Path,
) -> std::path::PathBuf {
    execute_profiled_input(
        KernelProfile {
            memory: true,
            user_extents: true,
            owned_callers: true,
            process_memory: true,
            policy_failure: None,
        },
        Some(CallerFocus::SourceBoundary),
        false,
        Some((image, source_work)),
        Some(&hierarchy_backing::selected_root()),
        None,
        None,
    )
}

/// Proposals remain compile-only until the caller binds the final fresh image.
pub(crate) fn prepare_image_backing_fixture(
    image: &[u8],
    source_work: &std::path::Path,
    plan: Option<&crate::systems_image_backing::Plan>,
) -> (std::path::PathBuf, crate::SystemsKernelImageLayout) {
    let selected = plan.map(|_| {
        crate::systems_image_backing::with_image_preparation(&hierarchy_backing::selected_root())
    });
    let mut layout = None;
    let work = execute_profiled_input(
        KernelProfile {
            memory: true,
            user_extents: true,
            owned_callers: true,
            process_memory: true,
            policy_failure: None,
        },
        Some(CallerFocus::SourceBoundary),
        false,
        Some((image, source_work)),
        selected.as_deref(),
        plan,
        Some(&mut layout),
    );
    (work, layout.unwrap())
}

fn execute_profiled_input(
    profile: KernelProfile,
    focus: Option<CallerFocus>,
    native: bool,
    user_fixture: Option<(&[u8], &std::path::Path)>,
    selected_root: Option<&str>,
    image_plan: Option<&crate::systems_image_backing::Plan>,
    layout_output: Option<&mut Option<crate::SystemsKernelImageLayout>>,
) -> std::path::PathBuf {
    let KernelProfile {
        memory,
        user_extents,
        owned_callers,
        process_memory,
        ..
    } = profile;
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: complete existing boot IR, not
    // a smaller constant probe. Canonical publication remains unqualified.
    let root = selected_root.map(str::to_owned).unwrap_or_else(root);
    let mut input = input();
    if let Some(failure) = profile.policy_failure {
        assert!(memory && user_extents && owned_callers && process_memory);
        assert!(focus == Some(CallerFocus::SourceProcessOwners));
        failure.prepare(&mut input);
    }
    let checked = crate::systems_frame_policy_binding::check(&root, &input).unwrap();
    let mut image_bindings = None;
    let (bytes, owners) = if let Some(plan) = image_plan {
        assert!(user_extents && process_memory && !native);
        let (bytes, owners, bindings) =
            crate::systems_artifact::generate_image_backing_root(&checked, plan).unwrap();
        image_bindings = Some(bindings);
        (bytes, Some(owners))
    } else if user_extents {
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
    let prefix = if process_memory && focus == Some(CallerFocus::Application) {
        "process-mm-applications"
    } else if process_memory && !process_memory_focus(focus) {
        "process-mm-interface-regression"
    } else if matches!(
        focus,
        Some(CallerFocus::SourceProcessRelationship(_, _, true))
    ) {
        "process-mm-zombie-termination"
    } else if matches!(focus, Some(CallerFocus::SourceProcessRelationship(..))) {
        "process-mm-ready-termination"
    } else if matches!(focus, Some(CallerFocus::SourceProcessGeometry(true))) {
        "process-mm-geometry-clone"
    } else if matches!(focus, Some(CallerFocus::SourceProcessGeometry(false))) {
        "process-mm-geometry-fork"
    } else if focus == Some(CallerFocus::SourceProcessAllocation) {
        "process-mm-allocation"
    } else if matches!(focus, Some(CallerFocus::SourceProcessCancel(_, _, true))) {
        "process-mm-writer-cancel"
    } else if matches!(focus, Some(CallerFocus::SourceProcessCancel(..))) {
        "process-mm-parent-cancel"
    } else if focus == Some(CallerFocus::SourceProcessExec) {
        "process-mm-child-exec"
    } else if focus == Some(CallerFocus::SourceProcessOwners) {
        "process-mm-owners"
    } else if focus == Some(CallerFocus::SourcePreludeStat) {
        "process-mm-prelude-stat"
    } else if focus == Some(CallerFocus::SourceProcessMemory) {
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
                | CallerFocus::SourceProcessOwners
                | CallerFocus::SourcePreludeStat
                | CallerFocus::SourceProcessExec
                | CallerFocus::SourceProcessAllocation
                | CallerFocus::SourceProcessGeometry(..)
                | CallerFocus::SourceProcessRelationship(..)
                | CallerFocus::SourceProcessCancel(..)
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
    if process_memory {
        fs::write(work.join("frame-policy.t"), &input.text).unwrap();
        fs::write(
            work.join("frame-policy.modules.json"),
            crate::systems_library::modules::snapshot(&input).unwrap(),
        )
        .unwrap();
    }
    if let Some(failure) = profile.policy_failure {
        failure.record(&work, &input);
    }
    if let Some(owners) = &owners {
        fs::write(
            work.join("root-backing-owners.json"),
            serde_json::to_vec_pretty(owners).unwrap(),
        )
        .unwrap();
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
    if let Some(bindings) = &image_bindings {
        fs::write(
            work.join("image-backing-bindings.json"),
            serde_json::to_vec_pretty(bindings).unwrap(),
        )
        .unwrap();
    }
    let tools = crate::LlvmTools::discover(None).unwrap();
    run(
        &work,
        "root-disassembly",
        Command::new(tools.tool_path("llvm-objdump"))
            .args(["-d", "--no-show-raw-insn"])
            .arg(work.join("root.o")),
    );
    let target = if let Some(owners) = &owners {
        if process_memory {
            super::process_mm::process_target_llvm(&checked.component, owners)
        } else if focus == Some(CallerFocus::SourceProcessMemory) {
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
    let mut layout = None;
    build_guest(
        &work,
        &checked,
        &tools,
        profile,
        focus,
        native,
        user_fixture,
        owners.as_ref(),
        image_plan.zip(image_bindings.as_ref()),
        &mut layout,
    );
    if let Some(output) = layout_output {
        *output = layout;
    }
    work
}

fn build_guest(
    work: &std::path::Path,
    checked: &crate::systems_frame_policy_binding::Checked,
    tools: &crate::LlvmTools,
    profile: KernelProfile,
    focus: Option<CallerFocus>,
    native: bool,
    user_fixture: Option<(&[u8], &std::path::Path)>,
    owner_bindings: Option<&crate::systems_user_extent::Owners>,
    image_backing: Option<(
        &crate::systems_image_backing::Plan,
        &crate::systems_image_backing::Bindings,
    )>,
    layout_output: &mut Option<crate::SystemsKernelImageLayout>,
) {
    let KernelProfile {
        memory,
        user_extents,
        owned_callers,
        process_memory,
        ..
    } = profile;
    use object::ObjectSegment;
    let fixture = work.join("static-elf-abi-fixture.elf");
    let fixture_bytes = if let Some((image, source_work)) = user_fixture {
        assert!(process_memory && owned_callers && !native && profile.policy_failure.is_none());
        assert!(focus == Some(CallerFocus::SourceBoundary));
        let source = fs::read(source_work.join("source-component.json")).unwrap();
        let record: serde_json::Value = serde_json::from_slice(&source).unwrap();
        assert_eq!(record["image_sha256"], crate::artifact::sha256(image));
        fs::write(work.join("source-component.json"), source).unwrap();
        image.to_vec()
    } else if let Some(CallerFocus::SourceProcessRelationship(early, group, zombie)) = focus {
        crate::systems_user_extent::source_fixture::build_process_memory_relationship(
            work, tools, early, group, zombie,
        )
    } else if let Some(CallerFocus::SourceProcessGeometry(clone)) = focus {
        crate::systems_user_extent::source_fixture::build_process_memory_geometry(
            work, tools, clone,
        )
    } else if focus == Some(CallerFocus::SourceProcessAllocation) {
        crate::systems_user_extent::source_fixture::build_process_memory_allocation(work, tools)
    } else if let Some(CallerFocus::SourceProcessCancel(early, group, writer)) = focus {
        crate::systems_user_extent::source_fixture::build_process_memory_cancel(
            work, tools, early, group, writer,
        )
    } else if focus == Some(CallerFocus::SourceProcessExec) {
        crate::systems_user_extent::source_fixture::build_process_memory_exec(work, tools)
    } else if focus == Some(CallerFocus::SourceProcessOwners) {
        crate::systems_user_extent::source_fixture::build_process_memory(work, tools)
    } else if focus == Some(CallerFocus::SourcePreludeStat) {
        crate::systems_user_extent::source_fixture::build_prelude_stat(work, tools)
    } else if matches!(
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
    if process_memory && focus == Some(CallerFocus::Application) {
        fs::write(
            work.join("process-mm-application.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "process_memory": true,
                "scope": "Unchanged pinned BusyBox corpus on the experimental process-MM kernel; no public activation",
            }))
            .unwrap(),
        )
        .unwrap();
    } else if process_memory && !process_memory_focus(focus) {
        assert!(matches!(
            focus,
            Some(
                CallerFocus::SourceBoundary
                    | CallerFocus::SourceExec
                    | CallerFocus::SourceSignals
                    | CallerFocus::SourcePoll
                    | CallerFocus::SourcePipes
                    | CallerFocus::SourceStorage
                    | CallerFocus::SourceMetadata
                    | CallerFocus::SourceSerial
                    | CallerFocus::SourceStat
            )
        ));
        fs::write(work.join("process-mm-regression.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "process_memory": true,
            "scope": "Existing ordinary source-built syscall fixtures on the experimental process-MM kernel; no public activation",
        })).unwrap()).unwrap();
    }
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
        if process_memory {
            super::process_mm::process_dispatch_llvm().unwrap()
        } else if focus == Some(CallerFocus::SourceProcessMemory) {
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
        if process_memory {
            closed_link::compile_input(work, tools, role);
            continue;
        }
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
        (if process_memory {
            crate::systems_provider_object::tail_boundary::frame_gateway::process_memory_component_object
        } else if owned_callers {
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
    let provider = (if process_memory {
        crate::systems_provider_object::generate_process_memory_provider(checked)
    } else if owned_callers {
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
    if process_memory {
        // Reconstruct the unchanged owned selection alongside the experiment.
        // 2160 is the archived preparation component's actual dispatch bound;
        // this artifact is compared at that same parameter, never selected.
        let retained =
            crate::systems_provider_object::generate_owned_user_extent_provider(checked).unwrap();
        fs::write(work.join("retained-owned-provider.o"), retained.bytes).unwrap();
        fs::write(
            work.join("retained-owned-gateway.o"),
            crate::systems_provider_object::tail_boundary::frame_gateway::owned_user_extent_component_object(2160).unwrap(),
        ).unwrap();
    }
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
    if process_memory {
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
    if let Some((plan, bindings)) = image_backing {
        let text = crate::systems_image_backing::llvm(plan, bindings).unwrap();
        crate::systems_frame_publication::compile("image-backing", &text, tools, work).unwrap();
    }
    let legacy = &baseline.directory;
    let prepared = process_memory
        .then(|| closed_link::prepare(work, tools, legacy, owned_callers, image_backing.is_some()));
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
            let file = format!("{name}.o");
            link.arg(
                prepared
                    .as_ref()
                    .map_or_else(|| work.join(&file), |p| p.path(&file)),
            );
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
                let file = format!("{name}.o");
                link.arg(
                    prepared
                        .as_ref()
                        .map_or_else(|| work.join(&file), |p| p.path(&file)),
                );
            }
        }
        for name in objects {
            let path = prepared
                .as_ref()
                .map_or_else(|| legacy.join(name), |p| p.path(name));
            assert!(path.is_file(), "missing {}", path.display());
            link.arg(path);
        }
        if image_backing.is_some() {
            link.arg(prepared.as_ref().unwrap().path("image-backing.o"));
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
    if let Some(prepared) = prepared {
        *layout_output = Some(prepared.inspect(work, &kernel, owner_bindings.unwrap()));
    }
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
    if !native {
        assert!(process_memory);
        fs::write(
            work.join("build-only.json"),
            b"{\"native_execution\":false}\n",
        )
        .unwrap();
        return;
    }
    run(
        work,
        "guest",
        Command::new("python3")
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                if profile.policy_failure.is_some() {
                    "../../linux-kernel/labs/qemu/x86_64/process_mm_policy_failure_inspect.py"
                } else if focus == Some(CallerFocus::SourcePreludeStat) {
                    "../../linux-kernel/labs/qemu/x86_64/process_mm_prelude_stat_inspect.py"
                } else if process_memory && focus == Some(CallerFocus::Application) {
                    "../../linux-kernel/labs/qemu/x86_64/process_mm_application_inspect.py"
                } else if process_memory_focus(focus) {
                    "../../linux-kernel/labs/qemu/x86_64/process_mm_process_inspect.py"
                } else if matches!(
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
