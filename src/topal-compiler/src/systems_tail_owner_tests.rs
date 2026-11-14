use super::*;
use std::{fs, path::Path, process::Command};
fn run(path: &Path, label: &str, command: &mut Command) {
    let argv = format!("{command:?}");
    let result = command.output().unwrap();
    fs::write(path.join(format!("{label}.command.txt")), argv).unwrap();
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|v| v.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    fs::write(
        path.join(format!("{label}.command.json")),
        serde_json::to_vec_pretty(
            &serde_json::json!({"argv":argv,"exit_code":result.status.code()}),
        )
        .unwrap(),
    )
    .unwrap();
    fs::write(path.join(format!("{label}.stdout.log")), &result.stdout).unwrap();
    fs::write(path.join(format!("{label}.stderr.log")), &result.stderr).unwrap();
    assert!(
        result.status.success(),
        "{}\n{}\n{}\n{}",
        path.display(),
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
#[test]
fn native_owner_admits_actual_context_runs_all_roles_and_preserves_history() {
    // TOPAL-SYSTEMS-TAIL-ADMISSION-001. Privileged kernel caller remains separate.
    let source = include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t");
    let program =
        topal_language::compiler::analyze_kernel_region_tail_source(source, &[], "clear-prefix", 3)
            .unwrap();
    let entry = &program
        .functions
        .iter()
        .find(|f| f.source_name == "clear-prefix")
        .unwrap()
        .symbol;
    let original = emit_systems_tail_owner_component(&program, "clear-prefix.t", entry).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    for mode in 0..=2 {
        let mut owner = original.clone();
        if mode != 0 {
            owner.qualification = "explicit native controller/source fault fixture; no source correspondence or kernel qualification".into();
        }
        if mode == 1 {
            let llvm = &mut owner.runtime.source_step.llvm;
            let start = llvm.find("call void @topal_tail_stage_number_v1(").unwrap();
            let pointer = start + llvm[start..].find(", ptr ").unwrap() + 6;
            let end = pointer + llvm[pointer..].find(')').unwrap();
            llvm.replace_range(pointer..end, "inttoptr (i64 1 to ptr)");
            owner.runtime.source_step.llvm_sha256 =
                format!("{:x}", Sha256::digest(llvm.as_bytes()));
            owner.runtime.source_step.qualification = "explicit invalid pointer fixture; no source correspondence or kernel qualification".into();
        }
        if mode == 2 {
            let line = "%outcome.valid = icmp ule i64 %outcome, 1";
            assert_eq!(owner.llvm.matches(line).count(), 1);
            owner.llvm = owner
                .llvm
                .replacen(line, "%outcome.valid = icmp eq i64 %outcome, 7", 1);
            owner.llvm_sha256 = format!("{:x}", Sha256::digest(owner.llvm.as_bytes()));
        }
        let work = std::env::temp_dir().join(format!("tail-owner-{}-{mode}", std::process::id()));
        fs::create_dir_all(&work).unwrap();
        fs::write(work.join("source.ll"), &owner.runtime.source_step.llvm).unwrap();
        fs::write(work.join("runtime.ll"), &owner.runtime.llvm).unwrap();
        fs::write(work.join("owner.ll"), &owner.llvm).unwrap();
        fs::write(work.join("boundary.o"), &owner.runtime.boundary).unwrap();
        let mut metadata = serde_json::to_value(&owner).unwrap();
        metadata.as_object_mut().unwrap().remove("llvm");
        metadata["runtime"].as_object_mut().unwrap().remove("llvm");
        metadata["runtime"]
            .as_object_mut()
            .unwrap()
            .remove("boundary");
        metadata["runtime"]["source_step"]
            .as_object_mut()
            .unwrap()
            .remove("llvm");
        metadata["fixture"] = serde_json::json!(mode);
        fs::write(
            work.join("owner.json"),
            serde_json::to_vec_pretty(&metadata).unwrap(),
        )
        .unwrap();
        for name in ["source", "runtime", "owner"] {
            run(
                &work,
                &format!("{name}-verify"),
                Command::new(tools.directory.join("opt"))
                    .args(["-passes=verify", "-disable-output"])
                    .arg(work.join(format!("{name}.ll"))),
            );
            run(
                &work,
                &format!("{name}-llc"),
                Command::new(tools.directory.join("llc"))
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
                    .arg(work.join(format!("{name}.ll")))
                    .arg("-o")
                    .arg(work.join(format!("{name}.o"))),
            );
            run(
                &work,
                &format!("{name}-disassembly"),
                Command::new(tools.directory.join("llvm-objdump"))
                    .args(["-d", "--no-show-raw-insn"])
                    .arg(work.join(format!("{name}.o"))),
            );
            crate::systems_library::inspect_instructions(
                &fs::read_to_string(work.join(format!("{name}-disassembly.stdout.log"))).unwrap(),
            )
            .unwrap();
        }
        let inspection = crate::systems_tail_runtime::inspection::inspect(
            &fs::read(work.join("source.o")).unwrap(),
            &fs::read(work.join("runtime.o")).unwrap(),
        )
        .unwrap();
        fs::write(
            work.join("native-inspection.json"),
            serde_json::to_vec_pretty(&inspection).unwrap(),
        )
        .unwrap();
        let owner_inspection =
            super::inspection::inspect(&fs::read(work.join("owner.o")).unwrap(), &inspection)
                .unwrap();
        fs::write(
            work.join("owner-inspection.json"),
            serde_json::to_vec_pretty(&owner_inspection).unwrap(),
        )
        .unwrap();
        let client = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../linux-kernel/labs/qemu/x86_64/tail_owner_client.c");
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
            "-mgeneral-regs-only",
            "-mno-red-zone",
            "-fno-builtin",
        ]);
        if mode != 0 {
            cc.arg("-DOWNER_FATAL_FIXTURE=1");
        }
        cc.arg(client);
        for name in ["source", "runtime", "owner", "boundary"] {
            cc.arg(work.join(format!("{name}.o")));
        }
        cc.arg("-o").arg(work.join("client"));
        run(&work, "cc", &mut cc);
        run(&work, "native", &mut Command::new(work.join("client")));
        let output = fs::read_to_string(work.join("native.stdout.log")).unwrap();
        assert!(output.contains("TAIL_OWNER_PASS"));
        assert!(output.contains(if mode == 0 { "fatal=0" } else { "fatal=1" }));
        println!("{output}");
    }
}

#[test]
fn owner_interface_refuses_other_signatures_without_widening_component_profiles() {
    // TOPAL-SYSTEMS-TAIL-ADMISSION-001: explicit initial caller interface only.
    let source = "use language (version is v0.1)\nrepeat is fn (remaining : Nat) -> Unit : Decreases remaining\n  remaining\n    <= 0 then ()\n    otherwise repeat(remaining - 1)\nrepeat(4)\n";
    let program = topal_language::analyze_for_compiler(source).unwrap();
    let topal_language::compiler::CompilerExpressionKind::Call { symbol, .. } =
        &program.main.result.kind
    else {
        panic!()
    };
    assert!(
        emit_systems_tail_owner_component(&program, "pure.t", symbol)
            .unwrap_err()
            .to_string()
            .contains("initial owner interface")
    );
    assert!(crate::emit_systems_tail_runtime_component(&program, "pure.t", symbol).is_ok());
}
