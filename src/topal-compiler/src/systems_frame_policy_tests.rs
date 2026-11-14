use super::{nat_tests::run, *};
use std::{fs, path::Path, process::Command};

#[test]
fn frame_metadata_policy_executes_owned_source_against_complete_state_oracle() {
    // TK-KERNEL-FRAME-POLICY-001: actual ordinary policy, no physical frame grant.
    let source = include_str!("../../../linux-kernel/kernel/memory/frame-metadata.t");
    let work = std::env::temp_dir().join(format!("frame-policy-{}", std::process::id()));
    fs::create_dir(&work).unwrap();
    eprintln!("frame policy fixture: {}", work.display());
    fs::write(work.join("source.t"), source).unwrap();
    let text = topal_source::SourceText::new(source).unwrap();
    fs::write(
        work.join("source.ast.txt"),
        format!(
            "{:#?}",
            topal_syntax::parse(&text, &topal_syntax::lex(&text))
        ),
    )
    .unwrap();
    let program = topal_language::compiler::analyze_kernel_region_tail_nat_source(
        source,
        &[],
        "frame-policy",
        3,
    )
    .unwrap();
    eprintln!(
        "frame policy checked functions: {}",
        program.functions.len()
    );
    let entry = &program
        .functions
        .iter()
        .find(|f| f.source_name == "frame-policy")
        .unwrap()
        .symbol;
    let owner = emit_systems_tail_nat_owner_component(&program, "frame-metadata.t", entry).unwrap();
    fs::write(work.join("source.ll"), &owner.runtime.source_step.llvm).unwrap();
    fs::write(work.join("runtime.ll"), &owner.runtime.llvm).unwrap();
    fs::write(work.join("owner.ll"), &owner.llvm).unwrap();
    fs::write(work.join("boundary.o"), &owner.runtime.boundary).unwrap();
    fs::write(
        work.join("component.json"),
        serde_json::to_vec_pretty(&owner).unwrap(),
    )
    .unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    for role in ["source", "runtime", "owner"] {
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
    let inner = crate::systems_tail_runtime::inspection::inspect(
        &fs::read(work.join("source.o")).unwrap(),
        &fs::read(work.join("runtime.o")).unwrap(),
    )
    .unwrap();
    let outer = inspection::inspect(&fs::read(work.join("owner.o")).unwrap(), &inner).unwrap();
    fs::write(
        work.join("inner-inspection.json"),
        serde_json::to_vec_pretty(&inner).unwrap(),
    )
    .unwrap();
    fs::write(
        work.join("owner-inspection.json"),
        serde_json::to_vec_pretty(&outer).unwrap(),
    )
    .unwrap();
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/frame_policy_client.c");
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
    .arg(format!(
        "-DFRAME_STEP_BOUND={}",
        owner.runtime.source_step.storage.source_frames_per_step
    ))
    .arg(client);
    for role in ["source", "runtime", "owner", "boundary"] {
        cc.arg(work.join(format!("{role}.o")));
    }
    cc.arg("-o").arg(work.join("client"));
    run(&work, "cc", &mut cc);
    run(&work, "native", &mut Command::new(work.join("client")));
    assert!(
        fs::read_to_string(work.join("native.stdout.log"))
            .unwrap()
            .starts_with("FRAME_POLICY_PASS")
    );
}
