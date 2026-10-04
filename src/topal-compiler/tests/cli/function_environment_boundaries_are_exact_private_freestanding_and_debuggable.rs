#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers scalar/aggregate/results, rejection, artifacts, and GDB frames.
fn function_environment_boundaries_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-ENVIRONMENT-BOUNDARY-001,
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-environment-boundaries");
    let source = directory.join("function-environment-boundaries.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-environment-boundaries.t"),
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
        b"(42, \"context\", 9, \"root\", (2, \"context-pair\"), (7, \"root-pair\"), (42, \"context\", 9, \"root\", (2, \"context-pair\"), (7, \"root-pair\")), 48, 49, (49, 50))\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir = fs::read_to_string(ir_path).unwrap();
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains(
                    "define internal fastcc { i32, ptr, ptr } @topal.fn.return_2doperation.",
                ) && line.contains("(i32 %arg0, ptr %arg1, ptr %arg2)")
            })
            .count(),
        3,
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains(
                "define internal fastcc { i32, { ptr, ptr }, { ptr, ptr } } @topal.fn.return_2doperation.",
            ) && line.contains("(i32 %arg0, { ptr, ptr } %arg1, { ptr, ptr } %arg2)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc { i32, ptr, ptr } @topal.fn.make_2danonymous.")
                && line.contains("(ptr %arg0, ptr %arg1)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc { ptr, ptr, ptr, ptr, { ptr, ptr }, { ptr, ptr } } @topal.fn.apply_2drecord.")
                && line.contains("%arg0, ptr %arg1, ptr %arg2, { ptr, ptr } %arg3, { ptr, ptr } %arg4, ptr %arg5, ptr %arg6")
        }),
        "{ir}"
    );
    for name in ["read_2dcontext", "read_2droot"] {
        assert_eq!(
            ir.lines()
                .filter(|line| {
                    line.contains(&format!("define internal fastcc ptr @topal.fn.{name}."))
                        && line.contains("(ptr %arg0, ptr %arg1)")
                        && !line.contains("ptr %arg2")
                })
                .count(),
            4,
            "{name}: {ir}"
        );
    }
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains("define internal fastcc { ptr, ptr } @topal.fn.read_2dpair.")
                    && line.contains("(ptr %arg0, { ptr, ptr } %arg1)")
            })
            .count(),
        4,
        "{ir}"
    );
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains("define internal fastcc ptr @topal.fn.anonymous.")
                    && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
            })
            .count(),
        2,
        "{ir}"
    );
    for name in [
        "return_2doperation",
        "make_2danonymous",
        "make_2danonymous_2drecord",
        "apply_2drecord",
        "forward_2drecord",
        "apply_2dint",
        "forward_2dint",
        "anonymous",
        "increase",
    ] {
        assert!(
            ir.lines().any(|line| line.contains("call fastcc ")
                && line.contains(&format!("@topal.fn.{name}."))),
            "{name}: {ir}"
        );
    }
    for variable in [
        "operation",
        "package",
        "@ context-number",
        "@ context-label",
        "@ context-pair",
        "root live-number",
        "root live-label",
        "root live-pair",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{variable}\"")),
            "{variable}: {ir}"
        );
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.context",
        "topal.root",
        "context.runtime",
        "namespace.runtime",
        "lookup.context",
        "lookup.root",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("fact-dependent-boundary.t");
    let rejected_executable = directory.join("fact-dependent-boundary");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\noffset is 40\nselect is fn (value : Nat) -> Int\n  @ offset\nselect is fn (value : Int) -> Int\n  0\napply is fn (operation : Function, value : Int) -> Int\n  operation value\napply (select, 0)\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("value-fact-dependent named Function environment selection"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(!rejected_executable.exists());
    assert!(!metadata_path(&rejected_executable).exists());

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let named_debugged = run(Command::new("gdb")
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
            "break function-environment-boundaries.t:36",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        named_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&named_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&named_debugged.stdout);
    assert!(text.contains("operation = <fn read-context>"), "{text}");
    assert!(text.contains("topal.fn.return_2doperation."), "{text}");
    assert!(text.contains("topal.main"), "{text}");

    let captured_debugged = run(Command::new("gdb")
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
            "break function-environment-boundaries.t:55",
            "-ex",
            "break function-environment-boundaries.t:64",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "frame 2",
            "-ex",
            "info args",
            "-ex",
            "frame 3",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        captured_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&captured_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&captured_debugged.stdout);
    assert_eq!(text.matches("@ context-number = 40").count(), 3, "{text}");
    assert_eq!(text.matches("root live-number = 7").count(), 3, "{text}");
    assert!(text.contains("operation = <fn increase>"), "{text}");
    assert!(text.matches("value = 2").count() >= 3, "{text}");
    for frame in [
        "topal.fn.anonymous.",
        "topal.fn.increase.",
        "topal.fn.apply_2dint.",
        "topal.fn.forward_2dint.",
        "topal.fn.use_2dnested.",
        "topal.main",
    ] {
        assert!(text.contains(frame), "{frame}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers scalar/aggregate escape, rejection, IR, and GDB frames.
fn escaping_nested_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-escaping-nested-function-environments");
    let source = directory.join("escaping-nested-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/escaping-nested-function-environments.t"),
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
    assert_eq!(executed.stdout, b"(43, 44, 45, 46, 47, (7, \"seven\"))\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir = fs::read_to_string(ir_path).unwrap();
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains(
                    "define internal fastcc { i32, ptr, ptr, ptr } @topal.fn.make_2doperation.",
                ) && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
            })
            .count(),
        4,
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains(
                "define internal fastcc { i32, { ptr, ptr } } @topal.fn.make_2dpair_2doperation.",
            ) && line.contains("({ ptr, ptr } %arg0)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc { { i32, ptr, i32, i32 }, ptr, ptr, ptr, ptr } @topal.fn.make_2drecord.")
                && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2, ptr %arg3)")
        }),
        "{ir}"
    );
    for name in [
        "make_2doperation",
        "return_2doperation",
        "make_2dpair_2doperation",
        "make_2drecord",
        "forward_2drecord",
        "apply_2drecord",
        "increase",
        "read_2dpair",
    ] {
        assert!(
            ir.lines().any(|line| line.contains("call fastcc ")
                && line.contains(&format!("@topal.fn.{name}."))),
            "{name}: {ir}"
        );
    }
    for variable in [
        "offset",
        "pair",
        "operation",
        "package",
        "@ context-offset",
        "root live-offset",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{variable}\"")),
            "{variable}: {ir}"
        );
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("function-capture.t");
    let rejected_executable = directory.join("function-capture");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nmake is fn (operation : Function) -> Function\n  wrapped is fn (value : Int) -> Int\n    operation value\n  wrapped\nresult is make increment\nresult 41\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    let diagnostic = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        diagnostic.contains("E-COMPILER-UNSUPPORTED"),
        "{diagnostic}"
    );
    assert!(!rejected_executable.exists());
    assert!(!metadata_path(&rejected_executable).exists());

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
            "break escaping-nested-function-environments.t:10",
            "-ex",
            "break escaping-nested-function-environments.t:20",
            "-ex",
            "break escaping-nested-function-environments.t:25",
            "-ex",
            "disable 2 3",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 2",
            "-ex",
            "enable 3",
            "-ex",
            "continue",
            "-ex",
            "info args",
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
        "operation = <fn increase>",
        "value = 1",
        "operand = 1",
        "offset = 1",
        "offset = 4",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.return_2doperation.",
        "topal.fn.increase.",
        "topal.fn.apply_2drecord.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Optional paths, rejection, mismatch, IR, and GDB frames.
fn optional_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-OPTIONAL-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-optional-function-environments");
    let source = directory.join("optional-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/optional-function-environments.t"),
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
        b"(43, 44, 45, 5, 42, Some +, 46, 47, 48, 0, 1, Some <fn increase>, None)\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir = fs::read_to_string(ir_path).unwrap();
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2doptional.")
            .count(),
        5,
        "{ir}"
    );
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains("call ptr @topal.runtime.optional.some(ptr"));
    assert!(ir.contains("call ptr @topal.runtime.optional.none()"));
    assert_eq!(
        ir.matches("call void @topal.runtime.pattern.identity.fail()")
            .count(),
        4,
        "{ir}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let mismatch_source = directory.join("mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        "use language (version is v0.1)\nmake is fn (offset : Int) -> Optional Function\n  increase is fn (value : Int) -> Int\n    value + offset\n  Some increase\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 2)\n",
    )
    .unwrap();
    let compiled_mismatch = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        compiled_mismatch.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled_mismatch.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(mismatch.stdout.is_empty());
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("E-ANONYMOUS-PATTERN-IDENTITY"));

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nchoose is fn (flag : Boolean) -> Optional Function\n  flag\n    true then Some increment\n    false then Some decrement\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Optional Function\n  nested is fn (value : Int) -> Int\n    operation value\n  Some nested\nwrap increment\n",
        ),
        (
            "optional-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (candidate : Optional Function) -> Optional Function\n  nested is fn (value : Int) -> Int\n    candidate\n      Some operation then operation value\n      None then 0\n  Some nested\nwrap (Some increment)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
    }

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
            "break optional-function-environments.t:20",
            "-ex",
            "break optional-function-environments.t:42",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "info args",
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
        "candidate=Some <fn increase>",
        "candidate = Some <fn increase>",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2doptional.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Sum paths, rejection, mismatch, IR, and GDB frames.
fn sum_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-SUM-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-sum-function-environments");
    let source = directory.join("sum-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/sum-function-environments.t"),
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
        b"(43, 44, 45, 5, 42, Apply +, 46, 47, 48, 0, 1, Apply <fn increase>, Unavailable)\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir = fs::read_to_string(ir_path).unwrap();
    assert_eq!(
        ir.matches(
            "define internal fastcc { { i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2doperation."
        )
        .count(),
        5,
        "{ir}"
    );
    assert!(ir.contains(
        "define internal fastcc { { i32, i32, ptr }, ptr, ptr, ptr } @topal.fn.make_2dchoice."
    ));
    assert_eq!(
        ir.matches("call void @topal.runtime.pattern.identity.fail()")
            .count(),
        4,
        "{ir}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let record_source = directory.join("record.t");
    let record_executable = directory.join("record");
    fs::write(
        &record_source,
        "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\ncontext-offset is 40\nmake is fn (offset : Int) -> Record (candidate : Operation, value : Int)\n  increase is fn (operand : Int) -> Int\n    operand + offset + @ context-offset + (root live-offset)\n  (candidate is Apply increase, value is 1)\napply is fn (package : Record (candidate : Operation, value : Int)) -> Int\n  package candidate\n    Apply operation then operation (package value)\n    Missing then 0\nlive-offset is 1\napply (make 1)\n",
    )
    .unwrap();
    let compiled_record = run(topalc().args([
        "-o",
        record_executable.to_str().unwrap(),
        record_source.to_str().unwrap(),
    ]));
    assert!(
        compiled_record.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled_record.stderr)
    );
    assert_eq!(run(&mut Command::new(&record_executable)).stdout, b"43\n");

    let mismatch_source = directory.join("mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nmake is fn (offset : Int) -> Operation\n  increase is fn (value : Int) -> Int\n    value + offset\n  Apply increase\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 2)\n",
    )
    .unwrap();
    let compiled_mismatch = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        compiled_mismatch.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled_mismatch.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(mismatch.stdout.is_empty());
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("E-ANONYMOUS-PATTERN-IDENTITY"));

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nchoose is fn (flag : Boolean) -> Operation\n  flag\n    true then Apply increment\n    false then Apply decrement\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Operation\n  nested is fn (value : Int) -> Int\n    operation value\n  Apply nested\nwrap increment\n",
        ),
        (
            "sum-function-capture",
            "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (candidate : Operation) -> Operation\n  nested is fn (value : Int) -> Int\n    candidate\n      Apply operation then operation value\n      Missing then 0\n  Apply nested\nwrap (Apply increment)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
    }

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
            "break sum-function-environments.t:21",
            "-ex",
            "break sum-function-environments.t:42",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "info args",
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
        "candidate=Apply <fn increase>",
        "candidate = Apply <fn increase>",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2doperation.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Result paths, rejection, Error propagation, IR, and GDB frames.
fn result_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-RESULT-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-result-function-environments");
    let source = directory.join("result-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/result-function-environments.t"),
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
        b"(43, 44, 45, 5, 42, +, 46, 47, 48, 49, 49, 0, 0, 0, <fn increase>, Error ( domain is root./(Rational,Rational), code is division-by-zero ))\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir = fs::read_to_string(ir_path).unwrap();
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2dresult.")
            .count(),
        7,
        "{ir}"
    );
    assert_eq!(
        ir.matches(
            "define internal fastcc { ptr, ptr, ptr, ptr, ptr, ptr } @topal.fn.make_2dfallible."
        )
        .count(),
        2,
        "{ir}"
    );
    assert!(
        ir.contains("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.project_2dresult.")
    );
    assert!(ir.contains(
        "define internal fastcc { ptr, ptr, ptr, ptr, ptr, ptr } @topal.fn.project_2dresult."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains("call ptr @topal.runtime.result.success(ptr"));
    assert!(ir.contains("insertvalue { ptr, ptr, ptr, ptr, ptr, ptr } poison, ptr"));
    assert!(ir.contains(", ptr null, 5"));
    assert!(ir.contains("ret { ptr, ptr, ptr, ptr, ptr, ptr }"));
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  increment\nright is fn () -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  decrement\nchoose is fn (flag : Boolean) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  nested is fn (value : Int) -> Int\n    operation value\n  nested\nwrap increment\n",
        ),
        (
            "result-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  increment\nwrap is fn (candidate : Result (Function, lang arithmetic ArithmeticErrorCode)) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  nested is fn (value : Int) -> Int\n    candidate\n      Ok operation then operation value\n      Error problem then 0\n  nested\nwrap (source ())\n",
        ),
        (
            "repeated-identity",
            "use language (version is v0.1)\nmake is fn (offset : Int) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  increase is fn (value : Int) -> Int\n    value + offset\n  increase\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
    }

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
            "break result-function-environments.t:24",
            "-ex",
            "break result-function-environments.t:46",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "info args",
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
        "candidate=<fn increase>",
        "candidate = <fn increase>",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2dresult.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact List paths, rejection, IR, and GDB frames.
fn list_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-LIST-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-function-environments");
    let source = directory.join("list-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-function-environments.t"),
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
        b"(43, 44, 45, 5, 42, Entry ( +, Empty ), 46, 47, 48, 1, 0, Entry ( <fn increment>, Entry ( <fn increase>, Empty ) ), Empty)\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir = fs::read_to_string(ir_path).unwrap();
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2dlist.")
            .count(),
        6,
        "{ir}"
    );
    assert!(ir.contains("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.return_2dlist."));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(ir.contains("store i32 "));
    assert!(ir.contains("getelementptr i8, ptr"));
    assert!(ir.contains(", i64 8"));
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> List Function\n  Entry (increment, Empty)\nright is fn () -> List Function\n  Entry (decrement, Empty)\nchoose is fn (flag : Boolean) -> List Function\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> List Function\n  nested is fn (value : Int) -> Int\n    operation value\n  Entry (nested, Empty)\nwrap increment\n",
        ),
        (
            "list-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> List Function\n  Entry (increment, Empty)\nwrap is fn (candidate : List Function) -> List Function\n  nested is fn (value : Int) -> Int\n    candidate\n      Entry (operation, remaining) then operation value\n      Empty then 0\n  Entry (nested, Empty)\nwrap (source ())\n",
        ),
        (
            "repeated-identity",
            "use language (version is v0.1)\nmake is fn (offset : Int) -> List Function\n  increase is fn (value : Int) -> Int\n    value + offset\n  Entry (increase, Empty)\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
    }

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
            "break list-function-environments.t:24",
            "-ex",
            "break list-function-environments.t:43",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "info args",
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
        "candidate=Entry ( <fn increment>, Entry ( <fn increase>, Empty ) )",
        "candidate = Entry ( <fn increment>, Entry ( <fn increase>, Empty ) )",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2dsecond.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact Array paths, rejection, IR, and GDB frames.
fn array_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ARRAY-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-array-function-environments");
    let source = directory.join("array-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/array-function-environments.t"),
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
        b"(2, 43, 44, 45, 5, 42, Array (+), 46, 47, 48, 2, 0, true, Array (<fn increment>, <fn increase>), Array ())\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir = fs::read_to_string(ir_path).unwrap();
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2darray.")
            .count(),
        6,
        "{ir}"
    );
    assert!(ir.contains("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.return_2darray."));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains("call ptr @topal.runtime.container.array.function.collect(ptr"));
    assert!(ir.contains("call ptr @topal.runtime.container.array.function.at(ptr"));
    assert!(ir.contains("%topal.ContainerSequenceHeader = type { i64, ptr }"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 4)"));
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> Array (1, Function)\n  values : List Function is Entry (increment, Empty)\n  values collect Array\nright is fn () -> Array (1, Function)\n  values : List Function is Entry (decrement, Empty)\n  values collect Array\nchoose is fn (flag : Boolean) -> Array (1, Function)\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Array (1, Function)\n  nested is fn (value : Int) -> Int\n    operation value\n  values : List Function is Entry (nested, Empty)\n  values collect Array\nwrap increment\n",
        ),
        (
            "array-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Array (1, Function)\n  values : List Function is Entry (increment, Empty)\n  values collect Array\nwrap is fn (candidate : Array (1, Function)) -> Array (1, Function)\n  nested is fn (value : Int) -> Int\n    array-at? (candidate, 0)\n      Some operation then operation value\n      None then 0\n  values : List Function is Entry (nested, Empty)\n  values collect Array\nwrap (source ())\n",
        ),
        (
            "repeated-identity",
            "use language (version is v0.1)\nmake is fn (offset : Int) -> Array (1, Function)\n  increase is fn (value : Int) -> Int\n    value + offset\n  values : List Function is Entry (increase, Empty)\n  values collect Array\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
    }

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
            "break array-function-environments.t:23",
            "-ex",
            "break array-function-environments.t:47",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "info args",
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
        "candidate=Array (<fn increment>, <fn increase>)",
        "candidate = Array (<fn increment>, <fn increase>)",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2dsecond.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact Map paths, rejection, IR, and GDB frames.
fn map_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-MAP-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-map-function-environments");
    let source = directory.join("map-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/map-function-environments.t"),
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
        b"(2, 43, 44, 45, 5, 42, 3, 46, 47, 48, 2, 8, 2, 0, false, Map ((\"increment\", <fn increment>), (\"increase\", <fn increase>)))\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir = fs::read_to_string(ir_path).unwrap();
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2dmap.")
            .count(),
        6,
        "{ir}"
    );
    assert!(ir.contains("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.return_2dmap."));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains("call ptr @topal.runtime.container.map.string-function.collect(ptr"));
    assert!(ir.contains("call ptr @topal.runtime.container.map.string-function.lookup(ptr"));
    assert!(ir.contains("%topal.ContainerSequenceHeader = type { i64, ptr }"));
    assert!(ir.contains("%topal.ContainerMapNode = type { ptr, ptr, ptr }"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 24)"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 4)"));
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, rejected_source) in [
        (
            "dynamic-map",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", increment), Empty)\n  collect-map pairs resolving reject\nright is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", decrement), Empty)\n  collect-map pairs resolving reject\nchoose is fn (flag : Boolean) -> Map (String, Function)\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Map (String, Function)\n  nested is fn (value : Int) -> Int\n    operation value\n  pairs : List (String, Function) is Entry ((\"operation\", nested), Empty)\n  collect-map pairs resolving reject\nwrap increment\n",
        ),
        (
            "map-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", increment), Empty)\n  collect-map pairs resolving reject\nwrap is fn (candidate : Map (String, Function)) -> Map (String, Function)\n  nested is fn (value : Int) -> Int\n    map-lookup (candidate, \"operation\")\n      Some operation then operation value\n      None then 0\n  pairs : List (String, Function) is Entry ((\"nested\", nested), Empty)\n  collect-map pairs resolving reject\nwrap (source ())\n",
        ),
        (
            "repeated-identity",
            "use language (version is v0.1)\nmake is fn (offset : Int) -> Map (String, Function)\n  increase is fn (value : Int) -> Int\n    value + offset\n  pairs : List (String, Function) is Entry ((\"operation\", increase), Empty)\n  collect-map pairs resolving reject\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
        ),
        (
            "dynamic-key",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", increment), Empty)\n  collect-map pairs resolving reject\nselect is fn (flag : Boolean) -> String\n  flag\n    true then \"operation\"\n    false then \"missing\"\nlookup is fn (candidate : Map (String, Function), key : String) -> Int\n  map-lookup (candidate, key)\n    Some operation then operation 1\n    None then 0\nlookup (source (), select true)\n",
        ),
        (
            "empty",
            "use language (version is v0.1)\nempty is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Empty\n  collect-map pairs resolving reject\nempty ()\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
    }

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
            "break map-function-environments.t:24",
            "-ex",
            "break map-function-environments.t:58",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "info args",
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
        "candidate=Map ((\"increment\", <fn increment>), (\"increase\", <fn increase>))",
        "candidate = Map ((\"increment\", <fn increment>), (\"increase\", <fn increase>))",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2dincrease.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}
