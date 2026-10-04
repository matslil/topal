#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers forwarding, rejection, artifacts, and every GDB frame.
fn function_root_data_forwarding_is_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001, TOPAL-NAMESPACE-ROOT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-root-data-forwarding");
    let source = directory.join("function-root-data-forwarding.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-root-data-forwarding.t"),
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
    assert_eq!(executed.stdout, b"(42, 0, \"ready\")\n");
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
    for name in ["read", "relay", "forward"] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!("@topal.fn.{name}."))
                    && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
            }),
            "{name}: {ir}"
        );
    }
    assert!(ir.lines().any(|line| {
        line.contains("call fastcc { ptr, ptr, ptr } @topal.fn.read.")
            && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
    }));
    assert!(ir.lines().any(|line| {
        line.contains("call fastcc { ptr, ptr, ptr } @topal.fn.relay.")
            && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
    }));
    for forbidden in [
        "topal.root",
        "root.runtime",
        "namespace.runtime",
        "context.runtime",
        "lookup.root",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_path = directory.join("overloaded.t");
    let rejected_executable = directory.join("overloaded");
    fs::write(
        &rejected_path,
        "use language (version is v0.1)\nread is fn (value : Nat) -> Int\n  root answer\nread is fn (value : Int) -> Int\n  0\nwrapper is fn (value : Int) -> Int\n  read value\nanswer is 42\nwrapper 0\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_path.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("overload-dependent root-data capture forwarding"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
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
            "break function-root-data-forwarding.t:8",
            "-ex",
            "run",
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
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert_eq!(text.matches("root label = \"ready\"").count(), 3, "{text}");
    assert_eq!(text.matches("root answer = 42").count(), 3, "{text}");
    assert_eq!(text.matches("answer = 0").count(), 3, "{text}");
    assert!(text.contains("topal.fn.read.0"), "{text}");
    assert!(text.contains("topal.fn.relay.1"), "{text}");
    assert!(text.contains("topal.fn.forward.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn defining_context_capture_is_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-CONTEXT-CAPTURE-001, TOPAL-CONTEXT-SELECT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-defining-context-capture");
    let source = directory.join("constructed-context.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/constructed-context.t"),
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
    assert_eq!(executed.stdout, b"42\n");

    let tools = LlvmTools::discover(None).unwrap();
    let undefined = run(Command::new(tools.directory.join("llvm-nm"))
        .arg("--undefined-only")
        .arg(&executable));
    assert!(undefined.status.success());
    assert!(
        undefined.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&undefined.stdout)
    );
    let inspected = run(Command::new(tools.directory.join("llvm-readobj"))
        .args(["--needed-libs", "--relocations"])
        .arg(&executable));
    assert!(inspected.status.success());
    let inspected = String::from_utf8_lossy(&inspected.stdout);
    assert!(inspected.contains("NeededLibraries [\n]"), "{inspected}");
    assert!(inspected.contains("Relocations [\n]"), "{inspected}");

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
            "break constructed-context.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "print '@ offset'",
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
    assert!(text.contains("$1 = 2"), "{text}");
    assert!(text.contains("$2 = 40"), "{text}");
    assert!(text.contains("@ offset=40"), "{text}");
    assert!(text.contains("topal.fn.add_2doffset.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers forwarding, rejection, artifacts, and every GDB frame.
fn defining_context_forwarding_is_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001, TOPAL-CONTEXT-SELECT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-defining-context-forwarding");
    let source = directory.join("defining-context-forwarding.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/defining-context-forwarding.t"),
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
    assert_eq!(executed.stdout, b"(40, 2, \"ready\")\n");
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
    for name in ["read", "relay", "forward"] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!("@topal.fn.{name}."))
                    && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
            }),
            "{name}: {ir}"
        );
    }
    assert!(ir.lines().any(|line| {
        line.contains("call fastcc { ptr, ptr, ptr } @topal.fn.read.")
            && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
    }));
    assert!(ir.lines().any(|line| {
        line.contains("call fastcc { ptr, ptr, ptr } @topal.fn.relay.")
            && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
    }));
    for forbidden in [
        "topal.context",
        "context.runtime",
        "context.environment",
        "lookup.context",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_path = directory.join("overloaded.t");
    let rejected_executable = directory.join("overloaded");
    fs::write(
        &rejected_path,
        "use language (version is v0.1)\noffset is 40\nread is fn (value : Nat) -> Int\n  @ offset\nread is fn (value : Int) -> Int\n  0\nwrapper is fn (value : Int) -> Int\n  read value\nwrapper 0\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_path.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("overload-dependent defining-context capture forwarding"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
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
            "break defining-context-forwarding.t:11",
            "-ex",
            "run",
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
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert_eq!(text.matches("@ offset = 40").count(), 3, "{text}");
    assert_eq!(text.matches("@ label = \"ready\"").count(), 3, "{text}");
    assert_eq!(text.matches("offset = 2").count(), 3, "{text}");
    assert!(text.contains("topal.fn.read.0"), "{text}");
    assert!(text.contains("topal.fn.relay.1"), "{text}");
    assert!(text.contains("topal.fn.forward.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers proof gating, exact cyclic IR, artifacts, and every GDB frame.
fn recursive_scalar_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-recursive-scalar-environments");
    let source = directory.join("recursive-scalar-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/recursive-scalar-environments.t"),
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
    assert_eq!(executed.stdout, b"((false, 0, 2), (true, 40, 0))\n");
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
    let definitions = ir
        .lines()
        .filter(|line| {
            line.contains("define internal fastcc { i1, ptr, ptr } @topal.fn.cycle_2d")
                && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
        })
        .collect::<Vec<_>>();
    assert_eq!(definitions.len(), 4, "{ir}");
    assert!(definitions.iter().all(|line| line.contains("noinline")));
    assert!(definitions.iter().all(|line| !line.contains("norecurse")));
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains("call fastcc { i1, ptr, ptr } @topal.fn.cycle_2d")
                    && line.contains("ptr %arg1, ptr %arg2)")
            })
            .count(),
        4,
        "{ir}"
    );
    assert_eq!(
        ir.matches("!DILocalVariable(name: \"@ captured\", arg: 2")
            .count(),
        4
    );
    assert_eq!(
        ir.matches("!DILocalVariable(name: \"root live\", arg: 3")
            .count(),
        4
    );
    for forbidden in [
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

    let rejected_path = directory.join("unproven.t");
    let rejected_executable = directory.join("unproven");
    fs::write(
        &rejected_path,
        "use language (version is v0.1)\ncaptured is 40\nloop is fn (value : Int) -> Int\n  value\n    <= 0 then @ captured\n    otherwise loop value\nloop 1\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_path.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("recursive function call"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
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
            "break recursive-scalar-environments.t:16",
            "-ex",
            "run",
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
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert_eq!(text.matches("@ captured = 40").count(), 4, "{text}");
    assert_eq!(text.matches("root live = 2").count(), 4, "{text}");
    for value in 0..=3 {
        assert!(text.contains(&format!("value = {value}")), "{text}");
    }
    assert!(text.contains("topal.fn.cycle_2deven.0"), "{text}");
    assert!(text.contains("topal.fn.cycle_2dodd.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers aggregate classes, recursion, rejection, artifacts, and GDB frames.
fn aggregate_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-aggregate-environments");
    let source = directory.join("aggregate-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/aggregate-environments.t"),
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
        b"((40, \"context\"), (amount is 2, enabled is true), (7, \"root\"), (amount is 9, enabled is false), Label \"context-sum\", Number 11)\n"
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
    for name in [
        "select_2dcontext_2dpair",
        "forward_2dcontext_2dpair",
        "select_2droot_2dpair",
        "forward_2droot_2dpair",
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!(
                    "define internal fastcc {{ ptr, ptr }} @topal.fn.{name}."
                )) && line.contains("(ptr %arg0, { ptr, ptr } %arg1)")
            }),
            "{name}: {ir}"
        );
    }
    for name in [
        "select_2dcontext_2drecord",
        "forward_2dcontext_2drecord",
        "select_2droot_2drecord",
        "forward_2droot_2drecord",
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!(
                    "define internal fastcc {{ ptr, i1, i32, i32 }} @topal.fn.{name}."
                )) && line.contains("(ptr %arg0, { ptr, i1, i32, i32 } %arg1)")
            }),
            "{name}: {ir}"
        );
    }
    for name in [
        "select_2dcontext_2dtoken",
        "forward_2dcontext_2dtoken",
        "select_2droot_2dtoken",
        "forward_2droot_2dtoken",
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!(
                    "define internal fastcc {{ i32, ptr, ptr }} @topal.fn.{name}."
                )) && line.contains("({ i32, ptr, ptr } %arg0)")
            }),
            "{name}: {ir}"
        );
    }
    assert!(
        ir.lines().any(|line| {
            line.contains("call fastcc { ptr, ptr } @topal.fn.select_2dcontext_2dpair.")
                && line.contains("{ ptr, ptr }")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("call fastcc { ptr, i1, i32, i32 } @topal.fn.select_2droot_2drecord.")
                && line.contains("{ ptr, i1, i32, i32 }")
        }),
        "{ir}"
    );
    for capture in [
        "@ context-pair",
        "@ context-record",
        "@ context-token",
        "root live-pair",
        "root live-record",
        "root live-token",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{capture}\"")),
            "{capture}: {ir}"
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

    let rejected_path = directory.join("callable-aggregate.t");
    let rejected_executable = directory.join("callable-aggregate");
    fs::write(
        &rejected_path,
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nbundle is (increment, 40)\nread is fn () -> (Function, Int)\n  @ bundle\nread ()\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_path.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("unsupported defining-context capture representation"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(!rejected_executable.exists());
    assert!(!metadata_path(&rejected_executable).exists());

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let tuple_debugged = run(Command::new("gdb")
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
            "break aggregate-environments.t:18",
            "-ex",
            "run",
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
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        tuple_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&tuple_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&tuple_debugged.stdout);
    assert_eq!(
        text.matches("@ context-pair = {_0 = 40, _1 = \"context\"}")
            .count(),
        3,
        "{text}"
    );
    assert!(
        text.contains("topal.fn.select_2dcontext_2dpair.0"),
        "{text}"
    );
    assert!(
        text.contains("topal.fn.forward_2dcontext_2dpair.1"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");

    let record_debugged = run(Command::new("gdb")
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
            "break aggregate-environments.t:42",
            "-ex",
            "run",
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
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        record_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&record_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&record_debugged.stdout);
    assert_eq!(
        text.matches("root live-record = {amount = 9, enabled = false}")
            .count(),
        3,
        "{text}"
    );
    assert!(text.contains("topal.fn.select_2droot_2drecord.6"), "{text}");
    assert!(
        text.contains("topal.fn.forward_2droot_2drecord.7"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");

    let sum_debugged = run(Command::new("gdb")
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
            "break aggregate-environments.t:51",
            "-ex",
            "break aggregate-environments.t:57",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        sum_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&sum_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&sum_debugged.stdout);
    assert!(
        text.contains("@ context-token = Label \"context-sum\""),
        "{text}"
    );
    assert!(text.contains("root live-token = Number 11"), "{text}");
    assert!(
        text.contains("topal.fn.forward_2droot_2dtoken.11"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers overload selection, recursion, rejection, artifacts, and GDB frames.
fn overload_environments_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-overload-environments");
    let source = directory.join("overload-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/overload-environments.t"),
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
        b"(40, \"context\", (2, \"context-pair\"), (7, \"root-pair\"), 7, \"root\", 47, 40, \"root\")\n"
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
    for (name, signature) in [
        ("choose_2dcontext", "(ptr %arg0, ptr %arg1)"),
        ("choose_2droot", "(ptr %arg0, ptr %arg1)"),
        ("forward_2dcontext_2dnumber", "(ptr %arg0)"),
        ("forward_2dcontext_2dlabel", "(ptr %arg0)"),
        ("forward_2droot_2dnumber", "(ptr %arg0)"),
        ("forward_2droot_2dlabel", "(ptr %arg0)"),
        ("cross", "(ptr %arg0, ptr %arg1, ptr %arg2)"),
        ("forward_2dcross", "(ptr %arg0, ptr %arg1)"),
        ("forward_2dproduct_2dtuple", "(ptr %arg0)"),
        ("forward_2dproduct_2drecord", "(ptr %arg0)"),
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!("define internal fastcc ptr @topal.fn.{name}."))
                    && line.contains(signature)
            }),
            "{name}: {ir}"
        );
    }
    for name in [
        "choose_2dpair",
        "forward_2dcontext_2dpair",
        "forward_2droot_2dpair",
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!(
                    "define internal fastcc {{ ptr, ptr }} @topal.fn.{name}."
                )) && line.contains("{ ptr, ptr }")
            }),
            "{name}: {ir}"
        );
    }
    assert_eq!(
        ir.lines()
            .filter(|line| line.contains("define internal fastcc ptr @topal.fn.choose_2dcontext."))
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.lines()
            .filter(|line| line.contains("define internal fastcc ptr @topal.fn.cross."))
            .count(),
        2,
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc ptr @topal.fn.choose_2dproduct.")
                && line.contains("({ ptr, ptr } %arg0, ptr %arg1)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc ptr @topal.fn.choose_2dproduct.")
                && line.contains("({ ptr, ptr, i32, i32 } %arg0, ptr %arg1)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("call fastcc ptr @topal.fn.cross.")
                && line.contains("(ptr @.topal.int.")
                && line.matches("ptr %arg").count() == 2
        }),
        "{ir}"
    );
    for capture in [
        "@ context-number",
        "@ context-label",
        "@ context-pair",
        "root live-pair",
        "root live-number",
        "root live-label",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{capture}\"")),
            "{capture}: {ir}"
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

    for (name, source_text, diagnostic) in [
        (
            "ambiguous-context",
            "use language (version is v0.1)\noffset is 40\nselect is fn (value : Nat) -> Int\n  @ offset\nselect is fn (value : Int) -> Int\n  0\nforward is fn (value : Int) -> Int\n  select value\nforward 0\n",
            "overload-dependent defining-context capture forwarding",
        ),
        (
            "ambiguous-root",
            "use language (version is v0.1)\nselect is fn (value : Nat) -> Int\n  root answer\nselect is fn (value : Int) -> Int\n  0\nforward is fn (value : Int) -> Int\n  select value\nanswer is 40\nforward 0\n",
            "overload-dependent root-data capture forwarding",
        ),
    ] {
        let rejected_source = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_source, source_text).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_source.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        assert!(
            String::from_utf8_lossy(&rejected.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&rejected.stderr)
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
    }

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let aggregate_debugged = run(Command::new("gdb")
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
            "break overload-environments.t:26",
            "-ex",
            "break overload-environments.t:29",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
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
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        aggregate_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&aggregate_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&aggregate_debugged.stdout);
    assert_eq!(
        text.matches("@ context-pair = {_0 = 2, _1 = \"context-pair\"}")
            .count(),
        2,
        "{text}"
    );
    assert_eq!(
        text.matches("root live-pair = {_0 = 7, _1 = \"root-pair\"}")
            .count(),
        2,
        "{text}"
    );
    assert!(text.contains("topal.fn.choose_2dpair.4"), "{text}");
    assert!(
        text.contains("topal.fn.forward_2dcontext_2dpair.5"),
        "{text}"
    );
    assert!(text.contains("topal.fn.choose_2dpair.6"), "{text}");
    assert!(text.contains("topal.fn.forward_2droot_2dpair.7"), "{text}");

    let recursive_debugged = run(Command::new("gdb")
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
            "break overload-environments.t:13",
            "-ex",
            "run",
            "-ex",
            "continue",
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
        recursive_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&recursive_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&recursive_debugged.stdout);
    assert_eq!(text.matches("@ context-number = 40").count(), 4, "{text}");
    for value in 0..=2 {
        assert!(text.contains(&format!("value = {value}")), "{text}");
    }
    assert!(text.contains("topal.fn.choose_2dcontext.0"), "{text}");
    assert!(
        text.contains("topal.fn.forward_2dcontext_2dnumber.1"),
        "{text}"
    );

    let cross_debugged = run(Command::new("gdb")
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
            "break overload-environments.t:52",
            "-ex",
            "run",
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
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        cross_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&cross_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&cross_debugged.stdout);
    assert_eq!(text.matches("@ context-number = 40").count(), 3, "{text}");
    assert_eq!(text.matches("root live-number = 7").count(), 3, "{text}");
    assert!(text.contains("topal.fn.cross.12"), "{text}");
    assert!(text.contains("topal.fn.cross.13"), "{text}");
    assert!(text.contains("topal.fn.forward_2dcross.14"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers aliases, nested calls, rejection, IR, artifacts, and GDB frames.
fn local_function_environments_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001,
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-local-function-environments");
    let source = directory.join("local-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/local-function-environments.t"),
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
        b"((42, \"context\", 9, \"root\", (2, \"context-pair\"), (7, \"root-pair\")), (42, 10))\n"
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
    for name in ["read_2dcontext", "read_2droot"] {
        assert_eq!(
            ir.lines()
                .filter(|line| {
                    line.contains(&format!("define internal fastcc ptr @topal.fn.{name}."))
                        && line.contains("(ptr %arg0, ptr %arg1)")
                })
                .count(),
            2,
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
        2,
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains(
                "define internal fastcc { ptr, ptr, ptr, ptr, { ptr, ptr }, { ptr, ptr } } @topal.fn.alias_2dvalues.",
            ) && line.contains(
                "(ptr %arg0, ptr %arg1, { ptr, ptr } %arg2, ptr %arg3, ptr %arg4, { ptr, ptr } %arg5)",
            )
        }),
        "{ir}"
    );
    for name in ["nested_2dcontext", "nested_2droot"] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!("define internal fastcc ptr @topal.fn.{name}."))
                    && line.contains("(ptr %arg0, ptr %arg1)")
                    && !line.contains("ptr %arg2")
            }),
            "{name}: {ir}"
        );
    }
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc { ptr, ptr } @topal.fn.nested_2dvalues.")
                && line.contains("(ptr %arg0, ptr %arg1)")
        }),
        "{ir}"
    );
    for name in [
        "read_2dcontext",
        "read_2droot",
        "read_2dpair",
        "nested_2dcontext",
        "nested_2droot",
    ] {
        assert!(
            ir.lines().any(|line| line.contains("call fastcc ")
                && line.contains(&format!("@topal.fn.{name}."))),
            "{name}: {ir}"
        );
    }
    for alias in [
        "context_2doperation",
        "context_2dchain",
        "root_2doperation",
        "root_2dchain",
        "pair_2doperation",
    ] {
        assert!(
            !ir.contains(&format!("@topal.fn.{alias}.")),
            "{alias}: {ir}"
        );
    }
    for local in [
        "context-operation",
        "context-chain",
        "root-operation",
        "root-chain",
        "pair-operation",
        "@ context-number",
        "@ context-pair",
        "root live-number",
        "root live-pair",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{local}\"")),
            "{local}: {ir}"
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

    let rejected_source = directory.join("ambiguous-alias.t");
    let rejected_executable = directory.join("ambiguous-alias");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\noffset is 40\nselect is fn (value : Nat) -> Int\n  @ offset\nselect is fn (value : Int) -> Int\n  0\nforward is fn (value : Int) -> Int\n  operation is select\n  operation value\nforward 0\n",
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
            .contains("overload-dependent defining-context capture forwarding"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(!rejected_executable.exists());
    assert!(!metadata_path(&rejected_executable).exists());

    let shadow_source = directory.join("shadow.t");
    let shadow_executable = directory.join("shadow");
    fs::write(
        &shadow_source,
        "use language (version is v0.1)\noffset is 40\nread is fn (value : Int) -> Int\n  value + @ offset\nwrapper is fn () -> Int\n  read is +\n  read (20, 22)\nwrapper ()\n",
    )
    .unwrap();
    let shadowed = run(topalc().args([
        "-o",
        shadow_executable.to_str().unwrap(),
        shadow_source.to_str().unwrap(),
    ]));
    assert!(
        shadowed.status.success(),
        "{}",
        String::from_utf8_lossy(&shadowed.stderr)
    );
    let shadow_output = run(&mut Command::new(&shadow_executable));
    assert_eq!(shadow_output.stdout, b"42\n");

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let alias_debugged = run(Command::new("gdb")
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
            "break local-function-environments.t:12",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "info locals",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        alias_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&alias_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&alias_debugged.stdout);
    for value in [
        "value = 2",
        "@ context-number = 40",
        "@ context-label = \"context\"",
        "@ context-pair = {_0 = 2, _1 = \"context-pair\"}",
        "root live-number = 7",
        "root live-label = \"root\"",
        "root live-pair = {_0 = 7, _1 = \"root-pair\"}",
        "context-operation = <fn read-context>",
        "context-chain = <fn read-context>",
        "root-operation = <fn read-root>",
        "root-chain = <fn read-root>",
        "pair-operation = <fn read-pair>",
        "topal.fn.read_2dcontext.",
        "topal.fn.alias_2dvalues.",
        "topal.main",
    ] {
        assert!(text.contains(value), "{value}: {text}");
    }

    let nested_debugged = run(Command::new("gdb")
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
            "break local-function-environments.t:46",
            "-ex",
            "break local-function-environments.t:48",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "info locals",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "info locals",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        nested_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&nested_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&nested_debugged.stdout);
    for value in [
        "value = 2",
        "value = 3",
        "@ context-number = 40",
        "root live-number = 7",
        "context-operation = <fn nested-context>",
        "root-operation = <fn nested-root>",
        "topal.fn.nested_2dcontext.",
        "topal.fn.nested_2droot.",
        "topal.fn.nested_2dvalues.",
        "topal.main",
    ] {
        assert!(text.contains(value), "{value}: {text}");
    }
}
