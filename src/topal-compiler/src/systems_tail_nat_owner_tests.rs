use super::*;
use std::{fs, path::Path, process::Command};

pub(crate) fn run(work: &Path, label: &str, command: &mut Command) {
    let result = command.output().unwrap();
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|s| s.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    fs::write(
        work.join(format!("{label}.command.json")),
        serde_json::to_vec_pretty(
            &serde_json::json!({"argv":argv,"exit_code":result.status.code()}),
        )
        .unwrap(),
    )
    .unwrap();
    fs::write(work.join(format!("{label}.stdout.log")), &result.stdout).unwrap();
    fs::write(work.join(format!("{label}.stderr.log")), &result.stderr).unwrap();
    assert!(
        result.status.success(),
        "{label}: {}\n{}",
        work.display(),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn natural_owner_returns_complete_values_and_actual_metadata_positions() {
    // TOPAL-SYSTEMS-TAIL-NAT-OWNER-001: native process component, no frame grant.
    let search = include_str!("../../../linux-kernel/kernel/algorithms/find-zero.t");
    let arithmetic = "use language (version is v0.1, features is (systems))\npub find-zero is fn ((region : KernelByteRegion, remaining : Nat, offset : Nat, state : Nat)) -> Nat : Decreases remaining\n  remaining\n    <= 0 then offset\n    otherwise find-zero(region, remaining - 1, offset + 1, state)\n";
    let constant = arithmetic.replace(
        "offset + 1",
        "115792089237316195423570985008687907853269984665640564039457584007913129639935",
    );
    let tools = crate::LlvmTools::discover(None).unwrap();
    for (mode, source) in [search, arithmetic, constant.as_str()]
        .into_iter()
        .enumerate()
    {
        let work =
            std::env::temp_dir().join(format!("tail-nat-owner-{}-{mode}", std::process::id()));
        fs::create_dir(&work).unwrap();
        eprintln!("natural owner fixture: {} mode={mode}", work.display());
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
            "find-zero",
            3,
        )
        .unwrap();
        assert!(
            topal_language::compiler::analyze_kernel_region_tail_source(
                source,
                &[],
                "find-zero",
                3
            )
            .is_err()
        );
        let entry = &program
            .functions
            .iter()
            .find(|f| f.source_name == "find-zero")
            .unwrap()
            .symbol;
        assert!(emit_systems_tail_owner_component(&program, "query.t", entry).is_err());
        let owner = emit_systems_tail_nat_owner_component(&program, "query.t", entry).unwrap();
        assert_eq!(owner.schema, "topal.tail-nat-owner-controller.x86_64/1");
        fs::write(work.join("source.ll"), &owner.runtime.source_step.llvm).unwrap();
        fs::write(work.join("runtime.ll"), &owner.runtime.llvm).unwrap();
        fs::write(work.join("owner.ll"), &owner.llvm).unwrap();
        fs::write(work.join("boundary.o"), &owner.runtime.boundary).unwrap();
        fs::write(
            work.join("component.json"),
            serde_json::to_vec_pretty(&owner).unwrap(),
        )
        .unwrap();
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
            .join("../../linux-kernel/labs/qemu/x86_64/tail_nat_owner_client.c");
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
        ])
        .arg(format!("-DNAT_QUERY_MODE={mode}"))
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
                .starts_with("TAIL_NAT_OWNER_PASS")
        );
    }
}

#[test]
fn natural_owner_constructor_retains_the_exact_source_and_result_profile() {
    // TOPAL-SYSTEMS-TAIL-NAT-OWNER-001: no Boolean interface widening.
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
    assert!(emit_systems_tail_nat_owner_component(&program, "clear.t", entry).is_err());
    assert!(emit_systems_tail_owner_component(&program, "clear.t", entry).is_ok());
    assert!(
        topal_language::compiler::analyze_kernel_region_tail_nat_source(
            source,
            &[],
            "clear-prefix",
            3
        )
        .is_err()
    );
    let other_measure = "use language (version is v0.1, features is (systems))\npub query is fn ((region : KernelByteRegion, remaining : Nat, offset : Nat, state : Nat)) -> Nat : Decreases state\n  state\n    <= 0 then offset\n    otherwise query(region, remaining, offset, state - 1)\n";
    let program = topal_language::compiler::analyze_kernel_region_tail_nat_source(
        other_measure,
        &[],
        "query",
        3,
    )
    .unwrap();
    let entry = &program
        .functions
        .iter()
        .find(|f| f.source_name == "query")
        .unwrap()
        .symbol;
    assert!(emit_systems_tail_nat_owner_component(&program, "measure.t", entry).is_err());
}
