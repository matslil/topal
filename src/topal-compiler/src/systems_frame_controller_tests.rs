use super::*;
use crate::systems_tail_owner::run_tail_native_fixture as run;
use std::{fs, path::Path, process::Command};

#[test]
fn frame_controller_uses_source_extents_and_preserves_opaque_handle_lifetimes() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: native controller, no CPL0 grant.
    let source = include_str!("../../../linux-kernel/kernel/memory/frame-metadata.t");
    let nofit = source.replace("length >= (word (region, 8))", "length < 0");
    let wrong = source.replace(
        "write (region, cell cursor, identity)",
        "write (region, cell cursor, identity + 1)",
    );
    assert_ne!(source, nofit);
    assert_ne!(source, wrong);
    for (mode, text) in [source, nofit.as_str(), wrong.as_str()]
        .into_iter()
        .enumerate()
    {
        execute(text, mode);
    }
}

fn execute(source: &str, mode: usize) {
    let work = std::env::temp_dir().join(format!("frame-controller-{}-{mode}", std::process::id()));
    fs::create_dir(&work).unwrap();
    eprintln!("native frame controller fixture: {}", work.display());
    fs::write(work.join("source.t"), source).unwrap();
    let program = topal_language::compiler::analyze_kernel_region_tail_nat_source(
        source,
        &[],
        "frame-policy",
        3,
    )
    .unwrap();
    let entry = &program
        .functions
        .iter()
        .find(|f| f.source_name == "frame-policy")
        .unwrap()
        .symbol;
    let owner =
        crate::emit_systems_tail_nat_owner_component(&program, "memory/frame-metadata.t", entry)
            .unwrap();
    fs::write(work.join("source.ll"), &owner.runtime.source_step.llvm).unwrap();
    fs::write(work.join("runtime.ll"), &owner.runtime.llvm).unwrap();
    fs::write(work.join("owner.ll"), &owner.llvm).unwrap();
    fs::write(work.join("controller.ll"), llvm().unwrap()).unwrap();
    fs::write(work.join("boundary.o"), &owner.runtime.boundary).unwrap();
    fs::write(
        work.join("component.json"),
        serde_json::to_vec_pretty(&owner).unwrap(),
    )
    .unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    for role in ["source", "runtime", "owner", "controller"] {
        run(
            &work,
            &format!("{role}-verify"),
            Command::new(tools.tool_path("opt"))
                .args(["-passes=verify", "-disable-output"])
                .arg(work.join(format!("{role}.ll"))),
        );
        run(
            &work,
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
        run(
            &work,
            &format!("{role}-disassembly"),
            Command::new(tools.tool_path("llvm-objdump"))
                .args(["-d", "--no-show-raw-insn"])
                .arg(work.join(format!("{role}.o"))),
        );
        crate::systems_library::inspect_instructions(
            &fs::read_to_string(work.join(format!("{role}-disassembly.stdout.log"))).unwrap(),
        )
        .unwrap();
    }
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/frame_controller_client.c");
    let mut cc = Command::new("cc");
    cc.args([
        "-std=gnu11",
        "-O2",
        "-g",
        "-fno-omit-frame-pointer",
        "-fno-strict-aliasing",
        "-fno-pie",
        "-no-pie",
        "-Wall",
        "-Wextra",
        "-Werror",
        "-mgeneral-regs-only",
        "-mno-red-zone",
        "-fno-builtin",
    ])
    .arg(format!("-DCONTROLLER_MODE={mode}"))
    .arg(format!(
        "-DFRAME_STEP_BOUND={}",
        owner.runtime.source_step.storage.source_frames_per_step
    ))
    .arg(format!(
        "-DLOAN_STEP_BOUND={}",
        owner
            .runtime
            .source_step
            .storage
            .source_region_loans_per_step
    ))
    .arg(client);
    for role in ["source", "runtime", "owner", "controller", "boundary"] {
        cc.arg(work.join(format!("{role}.o")));
    }
    cc.arg("-o").arg(work.join("client"));
    run(&work, "cc", &mut cc);
    run(&work, "native", &mut Command::new(work.join("client")));
    assert!(
        fs::read_to_string(work.join("native.stdout.log"))
            .unwrap()
            .starts_with("FRAME_CONTROLLER_PASS")
    );
}
