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
fn v6_namespace_personality_emission_closes_all_routes_without_publication() {
    // TOPAL-SYSTEMS-NAMESPACE-ABI-001/-ACTION-001. Code-generation closure,
    // not a linked/bootable kernel or execution of the five adapters.
    use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
    let work = temporary_work();
    let bytes = crate::systems_provider_object::namespace_personality_emission_object().unwrap();
    let file = object::File::parse(bytes.as_slice()).unwrap();
    let symbol = file
        .symbol_by_name(
            crate::systems_provider_object::X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL,
        )
        .unwrap();
    assert!(symbol.size() > 0);
    let section = file
        .section_by_index(symbol.section_index().unwrap())
        .unwrap();
    let calls = section
        .relocations()
        .filter_map(|(_, relocation)| {
            let object::RelocationTarget::Symbol(index) = relocation.target() else {
                return None;
            };
            Some(
                file.symbol_by_index(index)
                    .unwrap()
                    .name()
                    .unwrap()
                    .to_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        calls
            .iter()
            .filter(|name| name.as_str() == crate::systems_provider_object::namespace_region::APPLY)
            .count(),
        1
    );
    std::fs::write(work.join("namespace-personality-emission.o"), &bytes).unwrap();
    std::fs::write(
        work.join("external-calls.json"),
        serde_json::to_vec_pretty(&calls).unwrap(),
    )
    .unwrap();
}

#[test]
fn native_final_helpers_classify_without_traversing_and_bind_checked_entry_slots() {
    // TOPAL-SYSTEMS-NAMESPACE-ABI-001: independent guarded final-stage facts,
    // not complete syscall admission, live owner authority or kernel execution.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let object = work.join("namespace-final.o");
    std::fs::write(
        &object,
        crate::systems_provider_object::namespace_final_component_object().unwrap(),
    )
    .unwrap();
    let disassembly = observed(
        Command::new(tools.tool_path("llvm-objdump"))
            .args(["-d", "--no-show-raw-insn"])
            .arg(&object),
        &work,
        "final-disassembly",
    );
    crate::systems_library::inspect_instructions(&disassembly).unwrap();
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/namespace_final_client.c");
    let executable = work.join("namespace-final-client");
    observed(
        Command::new("cc")
            .args([
                "-Wall",
                "-Wextra",
                "-Werror",
                "-no-pie",
                "-Wl,-z,noexecstack",
            ])
            .arg(&client)
            .arg(&object)
            .arg("-o")
            .arg(&executable),
        &work,
        "final-compile",
    );
    let stdout = observed(&mut Command::new(&executable), &work, "final-native");
    assert_eq!(
        stdout.trim(),
        "NAMESPACE_FINAL_COMPONENT_PASS kinds=4099 lookups=12300"
    );
    eprintln!("{stdout}work={}", work.display());
}

#[test]
fn native_parent_walk_defers_final_disposition_and_preserves_copied_bytes() {
    // TOPAL-SYSTEMS-NAMESPACE-ABI-001: actual parent/final facts and frozen
    // lookup/open/unlink modes. CWD/absolute components, not FD/user-copy/VM.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/namespace_walk_client.c");
    for entries in [false, true] {
        let directory = work.join(if entries { "entries" } else { "legacy" });
        std::fs::create_dir(&directory).unwrap();
        let object = directory.join("namespace-walk.o");
        std::fs::write(
            &object,
            crate::systems_provider_object::namespace_walk_component_object(entries).unwrap(),
        )
        .unwrap();
        let disassembly = observed(
            Command::new(tools.tool_path("llvm-objdump"))
                .args(["-d", "--no-show-raw-insn"])
                .arg(&object),
            &directory,
            "walk-disassembly",
        );
        crate::systems_library::inspect_instructions(&disassembly).unwrap();
        let executable = directory.join("namespace-walk-client");
        observed(
            Command::new("cc")
                .args([
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-no-pie",
                    "-Wl,-z,noexecstack",
                ])
                .arg(format!("-DNS_WALK_ENTRIES={}", u8::from(entries)))
                .arg(&client)
                .arg(&object)
                .arg("-o")
                .arg(&executable),
            &directory,
            "walk-compile",
        );
        let stdout = observed(&mut Command::new(&executable), &directory, "walk-native");
        assert!(
            stdout.starts_with("NAMESPACE_PARENT_WALK_PASS "),
            "{stdout}"
        );
        eprintln!("{stdout}work={}", directory.display());
    }
}

#[test]
fn native_entry_components_use_separate_name_slots_and_checked_link_associations() {
    // TOPAL-SYSTEMS-NAMESPACE-OWNERSHIP-001/-ACTION-001. Same private emitter
    // under guarded C storage, not a kernel owner grant or Linux qualification.
    let tools = LlvmTools::discover(None).unwrap();
    let work = temporary_work();
    let path = work.join("namespace-entries.o");
    std::fs::write(
        &path,
        crate::systems_provider_object::namespace_entry_component_object().unwrap(),
    )
    .unwrap();
    let disassembly = observed(
        Command::new(tools.tool_path("llvm-objdump"))
            .args(["-d", "--no-show-raw-insn"])
            .arg(&path),
        &work,
        "entries-disassembly",
    );
    crate::systems_library::inspect_instructions(&disassembly).unwrap();
    let client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/namespace_entries_client.c");
    let executable = work.join("namespace-entries-client");
    observed(
        Command::new("cc")
            .args(["-Wall", "-Wextra", "-Werror", "-no-pie"])
            .arg(&client)
            .arg(&path)
            .arg("-o")
            .arg(&executable),
        &work,
        "entries-compile",
    );
    let stdout = observed(&mut Command::new(&executable), &work, "entries-native");
    assert!(
        stdout.starts_with("NAMESPACE_ENTRIES_COMPONENT_PASS "),
        "{stdout}"
    );
    let debug_executable = work.join("namespace-entries-debug-client");
    observed(
        Command::new("cc")
            .args([
                "-Wall",
                "-Wextra",
                "-Werror",
                "-g",
                "-no-pie",
                "-Wl,-z,noexecstack",
                "-DNS_ENTRIES_DEBUG=1",
            ])
            .arg(&client)
            .arg(&path)
            .arg("-o")
            .arg(&debug_executable),
        &work,
        "entries-debug-compile",
    );
    let debug = observed(
        Command::new("gdb")
            .args(["-q", "--batch", "-ex", "set debuginfod enabled off", "-x"])
            .arg(client.with_file_name("namespace_entries_debug.py"))
            .arg(&debug_executable),
        &work,
        "entries-debug-native",
    );
    assert!(
        debug.contains("NAMESPACE_ENTRIES_NATIVE_DEBUG_PASS checks=2 retired-spills=64"),
        "{debug}"
    );
    let leaf = observed(
        Command::new(tools.tool_path("llvm-objdump"))
            .args([
                "-d",
                "--no-show-raw-insn",
                "--disassemble-symbols=topal_namespace_metadata_check_v1",
            ])
            .arg(&path),
        &work,
        "entries-metadata-disassembly",
    );
    assert_eq!(leaf.matches("pushq").count(), 8, "{leaf}");
    assert_eq!(leaf.matches("popq").count(), 8, "{leaf}");
    for forbidden in [
        "call", "%rbx", "%rbp", "%r12", "%r13", "%r14", "%r15", "%xmm", "%ymm", "%zmm",
    ] {
        assert!(!leaf.contains(forbidden), "{forbidden}: {leaf}");
    }
    let old = crate::systems_provider_object::volatile_file_profile(true, true, false);
    let new = crate::systems_provider_object::volatile_file_profile(true, true, true);
    assert!(old.namespace_entries.is_none());
    assert_eq!(new.schema, "topal.volatile-file.x86_64/6");
    let entries = new.namespace_entries.as_ref().unwrap();
    assert_eq!(
        (
            entries.entry_offset,
            entries.entry_count,
            entries.entry_bytes
        ),
        (64, 4, 16)
    );
    assert_eq!(
        (
            entries.inode_link_count_offset,
            entries.inode_initial_link_intent_offset
        ),
        (56, 64)
    );
    assert_eq!(
        (
            entries.metadata_lookup_stack_bytes,
            entries.namespace_caller_stack_bytes
        ),
        (104, 256)
    );
    assert_eq!(entries.admission_retired_stack_bytes, 4096);
    let serialized = serde_json::to_value(&old).unwrap();
    assert!(serialized.get("namespace_entries").is_none());
    let mut null = serialized;
    null["namespace_entries"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<crate::SystemsVolatileFileProfile>(null).is_err());
    std::fs::write(
        work.join("entry-profile.json"),
        serde_json::to_vec_pretty(&new).unwrap(),
    )
    .unwrap();
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
