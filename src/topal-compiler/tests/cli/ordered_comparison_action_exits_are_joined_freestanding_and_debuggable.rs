#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and three GDB paths.
fn ordered_comparison_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-COMPARISON-001,
    // TOPAL-COMP-LEXICAL-RETURN-ORDERED-COMPARISON-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ORDERED-COMPARISON-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-ordered-comparison-decision-actions");
    let source = directory.join("function-return-ordered-comparison-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!(
            "../../../../examples/language/function-return-ordered-comparison-decision-actions.t"
        ),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(40, 41, 42)\n");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    for symbol in [
        "@topal.fn.choose.0",
        "@topal.fn.choose.1",
        "@topal.fn.choose.2",
    ] {
        let function = ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(
            function
                .matches("call i32 @topal.runtime.int.compare")
                .count(),
            2
        );
        assert_eq!(function.matches("br i1").count(), 2);
        assert_eq!(function.matches("br label").count(), 3);
        assert_eq!(function.matches("phi ptr").count(), 1);
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert!(!function.contains("select i1"));
        assert!(!function.contains("@topal.platform.allocate"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 9);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break function-return-ordered-comparison-decision-actions.t:9",
            "-ex",
            "break function-return-ordered-comparison-decision-actions.t:10",
            "-ex",
            "break function-return-ordered-comparison-decision-actions.t:11",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "disable 2",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in ["$1 = -1", "$2 = 0", "$3 = 1"] {
        assert!(text.contains(expected), "{text}");
    }
    for symbol in [
        "topal.fn.choose.0",
        "topal.fn.choose.1",
        "topal.fn.choose.2",
        "topal.main",
    ] {
        assert!(text.contains(symbol), "{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_completed_evidence_by_source_name() {
    // TOPAL-EXEC-COMPLETED-001, TOPAL-COMP-COMPLETED-001,
    // TOPAL-COMPILER-COMPLETED-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-completed");
    let source = directory.join("completed-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nfinish is fn () -> Completed\n  result is Completed\n  return result\nfinish ()\n",
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(
            dwarf.status.success(),
            "{}",
            String::from_utf8_lossy(&dwarf.stderr)
        );
    }
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break completed-debug.t:4",
            "-ex",
            "run",
            "-ex",
            "print result",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("Breakpoint 1, topal.fn.finish.0"), "{text}");
    assert!(text.contains("$1 = Completed"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_canonical_rational_parameters_and_locals() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-EXACT-001
    let directory = temporary("gdb-rational");
    let source = directory.join("rational-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nadjust is fn (value : Rational) -> Rational\n  result is value + 0.5\n  return result\n1.25 adjust\n",
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break rational-debug.t:3",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "next",
            "-ex",
            "print result",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = Rational ( 5, 4 )"), "{text}");
    assert!(text.contains("$2 = Rational ( 7, 4 )"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_nominal_enum_parameters_and_locals() {
    // TOPAL-COMP-ENUM-001, TOPAL-COMP-DEBUG-001, TOPAL-COMPILER-ENUM-001
    let directory = temporary("gdb-enum");
    let source = directory.join("enum-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nColor is Enum (Red, Green, Blue)\nretain is fn (value : Color) -> Color\n  result is value\n  return result\nretain Green\n",
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(
            dwarf.status.success(),
            "{}",
            String::from_utf8_lossy(&dwarf.stderr)
        );
    }
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break enum-debug.t:4",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "print result",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = Green"), "{text}");
    assert!(text.contains("$2 = Green"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn layout_policy_values_are_freestanding_and_debuggable() {
    // TOPAL-LAYOUT-ENDIAN-001, TOPAL-LAYOUT-ACCESS-001,
    // TOPAL-LAYOUT-BIT-ORDER-001, TOPAL-LAYOUT-PACKING-001,
    // TOPAL-LAYOUT-FIELD-ORDER-001, TOPAL-LAYOUT-PAYLOAD-PLACEMENT-001,
    // TOPAL-LAYOUT-ABSENCE-POLICY-001,
    // TOPAL-COMPILER-LAYOUT-POLICY-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-layout-policy-values");
    let source = directory.join("layout-policy-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nendian is Big\naccess is Reserved\nbits is LeastSignificantFirst\npacking is Packed\nfields is Declared\npayload is Overlay\nabsence is NoTerminator\n(endian, access, bits, packing, fields, payload, absence)\n",
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(
        executed.stdout,
        b"(Big, Reserved, LeastSignificantFirst, Packed, Declared, Overlay, NoTerminator)\n"
    );

    assert_freestanding_elf_and_valid_dwarf(&executable);

    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break layout-policy-values.t:9",
            "-ex",
            "run",
            "-ex",
            "whatis endian",
            "-ex",
            "print endian",
            "-ex",
            "whatis access",
            "-ex",
            "print access",
            "-ex",
            "whatis bits",
            "-ex",
            "print bits",
            "-ex",
            "whatis packing",
            "-ex",
            "print packing",
            "-ex",
            "whatis fields",
            "-ex",
            "print fields",
            "-ex",
            "whatis payload",
            "-ex",
            "print payload",
            "-ex",
            "whatis absence",
            "-ex",
            "print absence",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Endian",
        "$1 = Big",
        "type = enum Access",
        "$2 = Reserved",
        "type = enum BitOrder",
        "$3 = LeastSignificantFirst",
        "type = enum Packing",
        "$4 = Packed",
        "type = enum FieldOrder",
        "$5 = Declared",
        "type = enum PayloadPlacement",
        "$6 = Overlay",
        "type = enum LayoutPolicy",
        "$7 = NoTerminator",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn generator_error_code_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-ERROR-CODE-001,
    // TOPAL-COMPILER-GENERATOR-ERROR-CODE-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-generator-error-code");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/generator-error-codes.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(generator-closed, true)\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break generator-error-codes.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis closed",
            "-ex",
            "print closed",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum lang generator GeneratorErrorCode",
        "$1 = generator-closed",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn lazy_iterate_values_are_freestanding_linear_and_debuggable() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-CONSTRUCT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-lazy-iterate-values");
    let cases = [
        ("iterate-generator.t", "numbers"),
        ("iterate-take-while.t", "digits"),
    ];
    for (filename, binding) in cases {
        let executable = directory.join(filename.trim_end_matches(".t"));
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/language")
            .join(filename);
        let compiled =
            run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let executed = run(&mut Command::new(&executable));
        assert!(executed.status.success());
        assert_eq!(executed.stdout, b"<Generator Int Unit Unit>\n");
        assert_freestanding_elf_and_valid_dwarf(&executable);

        let debugged = run(Command::new("gdb")
            .args([
                "-q",
                "--batch",
                "-ex",
                "set debuginfod enabled off",
                "-ex",
                "set disable-randomization off",
                "-ex",
                &format!("break {filename}:8"),
                "-ex",
                "run",
                "-ex",
                &format!("whatis {binding}"),
                "-ex",
                &format!("print {binding}"),
                "-ex",
                "backtrace",
            ])
            .arg(&executable));
        assert!(
            debugged.status.success(),
            "{}",
            String::from_utf8_lossy(&debugged.stderr)
        );
        let text = String::from_utf8_lossy(&debugged.stdout);
        for expected in [
            "type = enum Generator Int Unit Unit",
            "$1 = <Generator Int Unit Unit>",
            "topal.main",
        ] {
            assert!(text.contains(expected), "missing {expected:?}: {text}");
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn bounded_iterate_collection_is_freestanding_ordered_and_debuggable() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-GENERATOR-COLLECT-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-bounded-iterate-collection");
    let executable = directory.join("application");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/generated-collect.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(
        executed.stdout,
        b"Entry ( 0, Entry ( 1, Entry ( 2, Entry ( 3, Entry ( 4, Empty ) ) ) ) )\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break topal.platform.write_all",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis digits",
            "-ex",
            "print digits",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("type = List Int"), "{text}");
    assert!(
        text.contains(
            "$1 = Entry ( 0, Entry ( 1, Entry ( 2, Entry ( 3, Entry ( 4, Empty ) ) ) ) )"
        ),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn bounded_iterate_foreach_is_freestanding_ordered_and_debuggable() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-GENERATOR-ITERATE-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-FOREACH-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-bounded-iterate-foreach");
    let executable = directory.join("application");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/generated-foreach.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"()\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break topal.runtime.int.add",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis digit",
            "-ex",
            "print digit",
            "-ex",
            "disable 1",
            "-ex",
            "break topal.platform.write_all",
            "-ex",
            "continue",
            "-ex",
            "up",
            "-ex",
            "whatis completed",
            "-ex",
            "print completed",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = Int",
        "$1 = 0",
        "type = Unit",
        "$2 = ()",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn lazy_unfold_value_is_freestanding_and_debuggable_without_step_execution() {
    // TOPAL-GENERATOR-UNFOLD-001,
    // TOPAL-COMPILER-GENERATOR-UNFOLD-CONSTRUCT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-lazy-unfold-value");
    let executable = directory.join("application");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/unfold-generator.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"<Generator Int Unit Unit>\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break topal.platform.write_all",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Int Unit Unit",
        "$1 = <Generator Int Unit Unit>",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn finite_unfold_collection_is_freestanding_ordered_and_debuggable() {
    // TOPAL-GENERATOR-UNFOLD-001, TOPAL-GENERATOR-UNFOLD-COLLECT-001,
    // TOPAL-COMPILER-GENERATOR-UNFOLD-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-finite-unfold-collection");
    let source = directory.join("unfold-collect-debug.t");
    let executable = directory.join("application");
    let shared = include_str!("../../../../examples/language/unfold-collect.t");
    fs::write(
        &source,
        shared.replace(
            "collect generated\n",
            "collected is collect generated\n(values, collected)\n",
        ),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(
        executed.stdout,
        b"(Entry ( 4, Entry ( 5, Entry ( 6, Empty ) ) ), Entry ( 4, Entry ( 5, Entry ( 6, Empty ) ) ))\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break topal.platform.write_all",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis collected",
            "-ex",
            "print collected",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("type = List Int"), "{text}");
    assert!(
        text.contains("$1 = Entry ( 4, Entry ( 5, Entry ( 6, Empty ) ) )"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_exact_range_parameters_and_locals() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-RANGE-001
    let directory = temporary("gdb-range");
    let source = directory.join("range-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nnarrow is fn (integers : Range Int, rationals : Range Rational) -> Range Rational\n  _ is integers contains 5\n  result is rationals and (1.0 <..= 2.0)\n  return result\n(0 <..= 10) narrow (0.5 ..= 2.5)\n",
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break range-debug.t:3",
            "-ex",
            "run",
            "-ex",
            "print integers",
            "-ex",
            "print rationals",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "print result",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 0 <..= 10"), "{text}");
    assert!(
        text.contains("$2 = Rational ( 1, 2 ) ..= Rational ( 5, 2 )"),
        "{text}"
    );
    assert!(
        text.contains("$3 = Rational ( 1, 1 ) <..= Rational ( 2, 1 )"),
        "{text}"
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_structured_arithmetic_result_parameters_and_locals() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-RESULT-001
    let directory = temporary("gdb-result");
    let source = directory.join("result-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nretain is fn (value : Result (Rational, lang arithmetic ArithmeticErrorCode)) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  result is value\n  result\n(1.0 divide 0.0) retain\n",
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break result-debug.t:6",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "print result",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    let expected = "Error ( domain is root./(Rational,Rational), code is division-by-zero )";
    assert!(text.contains(&format!("$1 = {expected}")), "{text}");
    assert!(text.contains(&format!("$2 = {expected}")), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_optional_parameters_and_locals() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-OPTIONAL-001
    let directory = temporary("gdb-optional");
    let source = directory.join("optional-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nobserve is fn (candidate : Optional String) -> Int\n  candidate\n    Some text then 1\n    None then 0\ninspect is fn (present : Optional Int, absent : Optional String) -> Optional Int\n  state is observe absent\n  result : Optional Int is present\n  return result\ninspect (Some 42, None String)\n",
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break optional-debug.t:7",
            "-ex",
            "break optional-debug.t:9",
            "-ex",
            "run",
            "-ex",
            "print present",
            "-ex",
            "print absent",
            "-ex",
            "continue",
            "-ex",
            "print result",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = Some 42"), "{text}");
    assert!(text.contains("$2 = None"), "{text}");
    assert!(text.contains("$3 = Some 42"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_optional_rational_parameters() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-OPTIONAL-RATIONAL-001
    let directory = temporary("gdb-optional-rational");
    let source = directory.join("optional-rational-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/optional-rational-values.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break optional-rational-values.t:8",
            "-ex",
            "run",
            "-ex",
            "print candidate",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("candidate=Some Rational ( 7, 2 )"), "{text}");
    assert!(text.contains("$1 = Some Rational ( 7, 2 )"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_character_parameters() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-CHARACTER-001
    let directory = temporary("gdb-character");
    let source = directory.join("character-classification.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/character-classification.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break character-classification.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("value=\"🙂\""), "{text}");
    assert!(text.contains("$1 = \"🙂\""), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_optional_character_parameters() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-CHARACTER-OBSERVATION-001
    let directory = temporary("gdb-optional-character");
    let source = directory.join("string-character-at.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/string-character-at.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break string-character-at.t:9",
            "-ex",
            "run",
            "-ex",
            "print candidate",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("candidate=Some \"👩‍🔬\""), "{text}");
    assert!(text.contains("$1 = Some \"👩‍🔬\""), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn closed_string_character_collection_is_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-character-collection");
    let source = directory.join("string-character-collection.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nreconstructed : String is characters \"a\u{301}👩‍🔬🇸🇪\" collect String\n_ is reconstructed = reconstructed\nreconstructed\n",
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, "\"a\u{301}👩‍🔬🇸🇪\"\n".as_bytes());
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break string-character-collection.t:3",
            "-ex",
            "run",
            "-ex",
            "whatis reconstructed",
            "-ex",
            "print reconstructed",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in ["type = String", "$1 = \"a\u{301}👩‍🔬🇸🇪\"", "topal.main"] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn transferred_string_character_generator_close_is_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-STRING-CHARACTERS-CLOSE-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-CLOSE-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-character-generator-close");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/string-character-generator-close.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"()\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break string-character-generator-close.t:9",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Character Unit Unit",
        "$1 = <Generator Character Unit Unit>",
        "ignore",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn transferred_string_character_generator_traversal_is_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-character-generator-parameter");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/string-character-generator-parameter.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"()\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break string-character-generator-parameter.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Character Unit Unit",
        "$1 = <Generator Character Unit Unit>",
        "consume",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn returned_string_character_generator_is_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-RESULT-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-character-generator-result");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/string-character-generator-result.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"()\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break string-character-generator-result.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis text",
            "-ex",
            "print text",
            "-ex",
            "backtrace",
            "-ex",
            "finish",
            "-ex",
            "whatis $",
            "-ex",
            "print $",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = String",
        "$1 = \"a\u{301}👩‍🔬🇸🇪\"",
        "generate",
        "Value returned is $2 = <Generator Character Unit Unit>",
        "type = enum Generator Character Unit Unit",
        "$3 = <Generator Character Unit Unit>",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn transferred_custom_generator_close_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-CLOSE-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-CLOSE-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-parameter-close");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-parameter-close.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"()\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break custom-generator-parameter-close.t:16",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Character Unit Unit",
        "$1 = <Generator Character Unit Unit>",
        "ignore",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn transferred_custom_generator_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-function-parameter");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-function-parameter.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"()\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break custom-generator-function-parameter.t:16",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Character Unit Unit",
        "$1 = <Generator Character Unit Unit>",
        "consume",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn transferred_custom_generator_character_result_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-PARAMETER-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-character-result-parameter");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-character-return-parameter.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"\"R\"\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            &format!("source {}", pretty_printers.display()),
            "-ex",
            "break custom-generator-character-return-parameter.t:16",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "next",
            "-ex",
            "whatis character",
            "-ex",
            "print character",
            "-ex",
            "backtrace",
            "-ex",
            "finish",
            "-ex",
            "print (Character)$3",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Character Unit Character",
        "$1 = <Generator Character Unit Character>",
        "type = Character",
        "$2 = \"Y\"",
        "consume",
        "topal.main",
        "$4 = \"R\"",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}
