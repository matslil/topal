use super::*;
use crate::systems_provider_object::namespace_oracle as native;

fn observed(command: &mut Command, directory: &Path, name: &str) -> String {
    let output = command.output().unwrap();
    std::fs::write(directory.join(format!("{name}.stdout.log")), &output.stdout).unwrap();
    std::fs::write(directory.join(format!("{name}.stderr.log")), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "{command:?}: {:?}\n{}\n{}; logs={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        directory.display()
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn native_namespace_oracle_checks_every_byte_and_erases_without_publication() {
    // TOPAL-SYSTEMS-NAMESPACE-ACTION-001. Native helper components only: neither
    // an actual kernel grant nor its remaining private-stack/fatal-path proof.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let native_path = work.join("namespace-oracle.o");
    std::fs::write(&native_path, native::component_object().unwrap()).unwrap();
    let disassembly = run(Command::new(tools.tool_path("llvm-objdump"))
        .args(["-d", "--no-show-raw-insn"])
        .arg(&native_path))
    .unwrap();
    crate::systems_library::inspect_instructions(&disassembly).unwrap();
    // Establish the leaf bound from actual emitted instructions, including the
    // lack of callee-saved changes, stack slots, calls, red zone and SIMD use.
    for forbidden in [
        "%rsp", "%esp", "%rbp", "%ebp", "call", "push", "pop", "enter", "leave", "%rbx", "%ebx",
        "%r12", "%r13", "%r14", "%r15", "%bx", "%bl", "%bh", "%bp", "%bpl", "%sp", "%spl", "%xmm",
        "%ymm", "%zmm", "%mm", "%st(",
    ] {
        assert!(
            !disassembly.contains(forbidden),
            "{forbidden}: {disassembly}"
        );
    }
    assert_eq!(native::HELPER_STACK_BYTES, 8);
    assert_eq!(disassembly.matches("retq").count(), 4, "{disassembly}");
    std::fs::write(work.join("namespace-oracle.disassembly"), &disassembly).unwrap();
    for fault in 0..3 {
        let variant = work.join(format!("variant-{fault}"));
        std::fs::create_dir(&variant).unwrap();
        let mut input = namespace_action_input(&variant);
        match fault {
            0 => {}
            1 => input.text = ACTION.replace(
                "  support namespace-metadata apply (region, operation, seconds)",
                "  _ is support namespace-metadata apply (region, operation, seconds)\n  false",
            ),
            _ => input.text = "use language (version is v0.1, features is (systems))\nuse library support (version is v0.1)\ndamage is fn (region : KernelByteRegion) -> Boolean\n  region unsigned store (offset-bytes is 64, byte-count is 8, byte-order is little-endian, value is 999)\npub apply is fn ((region : KernelByteRegion, operation : Nat, seconds : Nat)) -> Boolean\n  changed is support namespace-metadata apply (region, operation, seconds)\n  changed\n    true then damage region\n    false then false\n".into(),
        }
        std::fs::write(&input.source_name, &input.text).unwrap();
        let source = build_systems_region_instance_library(
            &input,
            "apply",
            2,
            "namespace-metadata",
            &tools,
            &variant,
        )
        .unwrap();
        let symbols = source.record.native_symbols().unwrap();
        let client = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../linux-kernel/labs/qemu/x86_64/namespace_oracle_client.c");
        for debug in [false, true] {
            let name = if debug {
                "namespace-oracle-debug-client"
            } else {
                "namespace-oracle-client"
            };
            let executable = variant.join(name);
            let mut compile = Command::new("cc");
            compile
                .args([
                    "-std=c11",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-O2",
                    "-g",
                    "-no-pie",
                    "-Wl,-z,noexecstack",
                ])
                .arg(format!("-DNS_EXPORT={}", symbols.export))
                .arg(format!("-DNS_ALLOCATE={}", symbols.allocate))
                .arg(format!("-DNS_ABORT={}", symbols.abort))
                .arg(format!(
                    "-DNS_LIMIT={}",
                    source.record.allocation.numeric.maximum_bytes
                ))
                .arg(&client)
                .arg(&native_path)
                .arg(variant.join("kernel-region.o"))
                .arg("-o")
                .arg(&executable);
            if fault == 1 {
                compile.arg("-DNS_FALSE_RESULT=1");
            }
            if fault == 2 {
                compile.arg("-DNS_WRONG_ASSOCIATION=1");
            }
            if debug {
                compile.arg("-DNS_ORACLE_DEBUG=1");
            }
            observed(&mut compile, &variant, &format!("{name}.compile"));
            let output = if debug {
                observed(
                    Command::new("gdb")
                        .args(["-q", "--batch", "-ex", "set debuginfod enabled off", "-ex"])
                        .arg(format!("set $namespace_fault = {fault}"))
                        .arg("-x")
                        .arg(client.with_file_name("namespace_oracle_debug.py"))
                        .arg(&executable),
                    &variant,
                    name,
                )
            } else {
                observed(&mut Command::new(&executable), &variant, name)
            };
            let marker = if debug {
                "NAMESPACE_NATIVE_ORACLE_DEBUG_PASS prepare=8 check=4 erased=4096+512"
            } else {
                "NAMESPACE_NATIVE_ORACLE_PASS transactions=3650 refusals=839 corruptions=16384 overlaps=20054 source-calls=3650 result-refusals=20"
            };
            assert!(output.contains(marker), "{output}");
            std::fs::write(variant.join(format!("{name}.log")), &output).unwrap();
            eprintln!("{output}");
        }
        std::fs::write(
            variant.join("namespace-oracle-source.record.json"),
            serde_json::to_vec_pretty(&source.record).unwrap(),
        )
        .unwrap();
        eprintln!(
            "NAMESPACE_ORACLE_COMPONENT_PASS fault={fault} leaf_stack={} source_stack={} work={}",
            native::HELPER_STACK_BYTES,
            source.record.maximum_stack_bytes,
            variant.display()
        );
    }
}
