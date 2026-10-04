#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn nat_lists_are_private_freestanding_and_debuggable() {
    // TOPAL-NUM-NAT-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-NAT-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-nat-values");
    let source = directory.join("list-nat-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-nat-values.t"),
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
        b"(0, 123456789012345678901234567890, true, true, 3, true, 5, 0, 6, Entry ( 0, Entry ( 123456789012345678901234567890, Entry ( +Infinity, Empty ) ) ))\n"
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
    for expected in [
        "%topal.ListStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.int.equal(ptr",
        "define internal ptr @topal.runtime.list.int.entry.count(ptr",
        "call i32 @topal.runtime.int.compare(ptr",
        "@topal.runtime.int.positive.infinity",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, ptr } @topal.fn.return_2dpair.",
        "store ptr",
        "load ptr",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.string.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Nat is Entry (1, Empty)\nvalues reverse\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-COMPILER-UNSUPPORTED"));
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
            "break list-nat-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Nat",
        "candidate = Entry ( 0, Entry ( 123456789012345678901234567890, Entry ( +Infinity, Empty ) ) )",
        "fallback = 9",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn rational_lists_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-LIST-RATIONAL-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-rational-values");
    let source = directory.join("list-rational-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-rational-values.t"),
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
    assert!(String::from_utf8(executed.stdout).unwrap().contains("Entry ( Rational ( 1, 2 ), Entry ( Rational ( 17636684144620811271604938270, 1 ), Entry ( +Infinity, Empty ) ) )"));
    assert_freestanding_elf_and_valid_dwarf(&executable);
    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListRationalStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.rational.equal(ptr",
        "define internal ptr @topal.runtime.list.rational.entry.count(ptr",
        "call i32 @topal.runtime.rational.compare(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [" byval", " sret", " inalloca", "preallocated", "call ptr %"] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
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
            "break list-rational-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Rational",
        "candidate = Entry ( Rational ( 1, 2 )",
        "fallback = Rational ( 9, 1 )",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn effect_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-EFFECT-EMPTY-001, TOPAL-EFFECT-IDENTITY-001,
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-EFFECT-CORE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-effect-values");
    let source = directory.join("list-effect-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-effect-values.t"),
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
        b"(Effects (), Effects (), true, true, 2, true, Effects (), Effects (), Effects (), Entry ( Effects (), Entry ( Effects (), Empty ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListEffectStorage = type { i8, ptr }",
        "define internal i1 @topal.runtime.list.effect.equal(ptr",
        "define internal ptr @topal.runtime.list.effect.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i8 } @topal.fn.return_2dpair.",
        "store i8 0",
        "load i8",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.boolean.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Effect is Entry (Effects (), Empty)\nvalues reverse\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-COMPILER-UNSUPPORTED"));
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
            "break list-effect-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Effect",
        "candidate = Entry ( Effects (), Entry ( Effects (), Empty ) )",
        "fallback = empty",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn comparison_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-COMPARISON-CORE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-comparison-values");
    let source = directory.join("list-comparison-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-comparison-values.t"),
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
        b"(Less, Equal, true, true, 3, true, Equal, Less, Greater, Entry ( Less, Entry ( Equal, Entry ( Greater, Empty ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListComparisonStorage = type { i32, ptr }",
        "define internal i1 @topal.runtime.list.comparison.equal(ptr",
        "define internal ptr @topal.runtime.list.comparison.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i32 } @topal.fn.return_2dpair.",
        "store i32",
        "load i32",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.boolean.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Comparison is Entry (1 <=> 2, Empty)\nvalues reverse\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-COMPILER-UNSUPPORTED"));
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
            "break list-comparison-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Comparison",
        "candidate = Entry ( Less, Entry ( Equal, Entry ( Greater, Empty ) ) )",
        "fallback = Greater",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn error_code_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-NUM-ARITHMETIC-ERROR-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-ERROR-CODE-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-error-code-values");
    let source = directory.join("list-error-code-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-error-code-values.t"),
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
        b"(out-of-range, not-representable, true, true, 4, true, division-by-zero, out-of-range, indeterminate, Entry ( out-of-range, Entry ( not-representable, Entry ( division-by-zero, Entry ( indeterminate, Empty ) ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListErrorCodeStorage = type { i32, ptr }",
        "define internal i1 @topal.runtime.list.error.code.equal(ptr",
        "define internal ptr @topal.runtime.list.error.code.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i32 } @topal.fn.return_2dpair.",
        "store i32",
        "load i32",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.comparison.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List ErrorCode is Entry (lang arithmetic out-of-range, Empty)\nvalues reverse\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-COMPILER-UNSUPPORTED"));
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
            "break list-error-code-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List lang arithmetic ArithmeticErrorCode",
        "candidate = Entry ( out-of-range, Entry ( not-representable, Entry ( division-by-zero, Entry ( indeterminate, Empty ) ) ) )",
        "fallback = indeterminate",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn unit_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-PRODUCT-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-UNIT-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-unit-values");
    let source = directory.join("list-unit-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-unit-values.t"),
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
        b"((), (), true, true, 3, true, (), (), (), Entry ( (), Entry ( (), Entry ( (), Empty ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListUnitStorage = type { i8, ptr }",
        "define internal i1 @topal.runtime.list.unit.equal(ptr",
        "define internal ptr @topal.runtime.list.unit.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i8 } @topal.fn.return_2dpair.",
        "store i8 0",
        "load i8",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.effect.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Unit is Entry ((), Empty)\nvalues reverse\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-COMPILER-UNSUPPORTED"));
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
            "break list-unit-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Unit",
        "candidate = Entry ( (), Entry ( (), Entry ( (), Empty ) ) )",
        "fallback = ()",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn completed_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-COMPLETED-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-COMPLETED-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-completed-values");
    let source = directory.join("list-completed-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-completed-values.t"),
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
        b"(Completed, Completed, true, true, 3, true, Completed, Completed, Completed, Entry ( Completed, Entry ( Completed, Entry ( Completed, Empty ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListCompletedStorage = type { i8, ptr }",
        "define internal i1 @topal.runtime.list.completed.equal(ptr",
        "define internal ptr @topal.runtime.list.completed.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i8 } @topal.fn.return_2dpair.",
        "store i8 0",
        "load i8",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.unit.equal",
        "topal.runtime.list.effect.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Completed is Entry (Completed, Empty)\nvalues reverse\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-COMPILER-UNSUPPORTED"));
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
            "break list-completed-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Completed",
        "candidate = Entry ( Completed, Entry ( Completed, Entry ( Completed, Empty ) ) )",
        "fallback = Completed",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn type_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-ABSTRACTION-TYPE-VALUE-001, TOPAL-ABSTRACTION-TYPE-IDENTITY-001,
    // TOPAL-ABSTRACTION-TYPE-BOUNDARY-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-TYPE-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-type-values");
    let source = directory.join("list-type-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-type-values.t"),
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
        b"(Boolean, Int, true, true, true, 7, true, String, Boolean, Unit, Entry ( Boolean, Entry ( Int, Entry ( Nat, Entry ( Rational, Entry ( String, Entry ( Unit, Entry ( Scope, Empty ) ) ) ) ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListTypeStorage = type { i32, ptr }",
        "define internal i1 @topal.runtime.list.type.equal(ptr",
        "define internal ptr @topal.runtime.list.type.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i32 } @topal.fn.return_2dpair.",
        "store i32 0",
        "load i32",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.comparison.equal",
        "topal.runtime.list.error.code.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Type is Entry (Int, Empty)\nvalues reverse\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-COMPILER-UNSUPPORTED"));
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
            "break list-type-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Type",
        "candidate = Entry ( Boolean, Entry ( Int, Entry ( Nat, Entry ( Rational, Entry ( String, Entry ( Unit, Entry ( Scope, Empty ) ) ) ) ) ) )",
        "fallback = String",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn nominal_enum_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-ENUM-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-ENUM-001, TOPAL-COMPILER-LIST-ENUM-CORE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-enum-values");
    let source = directory.join("list-enum-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-enum-values.t"),
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
        b"(Red, Green, true, true, true, 3, true, Blue, Red, Green, Entry ( Red, Entry ( Green, Entry ( Blue, Empty ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListEnumStorage = type { i32, ptr }",
        "define internal i1 @topal.runtime.list.enum.equal(ptr",
        "define internal ptr @topal.runtime.list.enum.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i32 } @topal.fn.return_2dpair.",
        "store i32 0",
        "store i32 1",
        "store i32 2",
        "load i32",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.type.equal",
        "topal.runtime.list.comparison.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nColor is Enum (Red, Green)\nvalues : List Color is Entry (Red, Empty)\nvalues reverse\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-COMPILER-UNSUPPORTED"));
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
            "break list-enum-values.t:10",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Color",
        "candidate = Entry ( Red, Entry ( Green, Entry ( Blue, Empty ) ) )",
        "fallback = Blue",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn nominal_modular_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001,
    // TOPAL-COMPILER-MODULAR-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-MODULAR-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-modular-values");
    let source = directory.join("list-modular-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-modular-values.t"),
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
        b"(ByteCounter 0, ByteCounter 255, true, true, true, 3, true, ByteCounter 7, ByteCounter 0, ByteCounter 8, Entry ( ByteCounter 0, Entry ( ByteCounter 255, Entry ( ByteCounter 42, Empty ) ) ))\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let record_source = directory.join("record-values.t");
    let record_executable = directory.join("record-values");
    fs::write(
        &record_source,
        "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nretain-record is fn (package : Record (values : List ByteCounter, fallback : ByteCounter)) -> Record (values : List ByteCounter, fallback : ByteCounter)\n  package\nvalues : List ByteCounter is Entry (ByteCounter 1, Empty)\nretained is retain-record (values is values, fallback is ByteCounter 7)\nretained values\n",
    )
    .unwrap();
    let record_compiled = run(topalc().args([
        "-o",
        record_executable.to_str().unwrap(),
        record_source.to_str().unwrap(),
    ]));
    assert!(
        record_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&record_compiled.stderr)
    );
    let record_executed = run(&mut Command::new(&record_executable));
    assert!(record_executed.status.success());
    assert_eq!(record_executed.stdout, b"Entry ( ByteCounter 1, Empty )\n");
    assert_freestanding_elf_and_valid_dwarf(&record_executable);

    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListModularStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.modular.equal(ptr",
        "define internal ptr @topal.runtime.list.modular.entry.count(ptr",
        "call i32 @topal.runtime.int.compare(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, ptr } @topal.fn.return_2dpair.",
        "store ptr",
        "load ptr",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.rational.equal",
        "topal.runtime.list.string.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nvalues : List ByteCounter is Entry (ByteCounter 1, Empty)\nvalues reverse\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-COMPILER-UNSUPPORTED"));
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
            "break list-modular-values.t:11",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List ByteCounter",
        "candidate = Entry ( ByteCounter 0, Entry ( ByteCounter 255, Entry ( ByteCounter 42, Empty ) ) )",
        "fallback = ByteCounter 7",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn optional_int_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001, TOPAL-COMPILER-LIST-OPTIONAL-INT-CORE-001
    let directory = temporary("gdb-list-optional-int-values");
    let source = directory.join("list-optional-int-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-optional-int-values.t"),
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
    assert_eq!(executed.stdout, b"(Some 1, None, true, true, true, 3, true, Some 7, Some 1, Some 8, Entry ( Some 1, Entry ( None, Entry ( Some -2, Empty ) ) ))\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);
    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListOptionalIntStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.optional.int.equal(ptr",
        "define internal ptr @topal.runtime.list.optional.int.entry.count(ptr",
        "call i1 @topal.runtime.optional.int.equal(ptr",
        "define internal fastcc { ptr, ptr } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.modular.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List Optional Int is Entry (Some 1, Empty)\nvalues reverse\n").unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
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
            "break list-optional-int-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Optional Int",
        "candidate = Entry ( Some 1, Entry ( None, Entry ( Some -2, Empty ) ) )",
        "fallback = Some 7",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn optional_rational_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001,
    // TOPAL-COMPILER-LIST-OPTIONAL-RATIONAL-CORE-001
    let directory = temporary("gdb-list-optional-rational-values");
    let source = directory.join("list-optional-rational-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-optional-rational-values.t"),
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
    assert_eq!(executed.stdout, b"(Some Rational ( 1, 2 ), None, true, true, true, 3, true, Some Rational ( 7, 3 ), Some Rational ( 1, 2 ), Some Rational ( 8, 5 ), Entry ( Some Rational ( 1, 2 ), Entry ( None, Entry ( Some Rational ( -3, 4 ), Empty ) ) ))\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);
    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListOptionalRationalStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.optional.rational.equal(ptr",
        "define internal ptr @topal.runtime.list.optional.rational.entry.count(ptr",
        "call i1 @topal.runtime.optional.rational.equal(ptr",
        "define internal fastcc { ptr, ptr } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.optional.int.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List Optional Rational is Entry (Some 1.5, Empty)\nvalues reverse\n").unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
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
            "break list-optional-rational-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Optional Rational",
        "candidate = Entry ( Some Rational ( 1, 2 ), Entry ( None, Entry ( Some Rational ( -3, 4 ), Empty ) ) )",
        "fallback = Some Rational ( 7, 3 )",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn optional_string_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001,
    // TOPAL-COMPILER-LIST-OPTIONAL-STRING-CORE-001
    let directory = temporary("gdb-list-optional-string-values");
    let source = directory.join("list-optional-string-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-optional-string-values.t"),
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
    assert_eq!(executed.stdout, "(Some \"first\", None, true, true, true, 3, true, Some \"fallback\", Some \"first\", Some \"record\", Entry ( Some \"first\", Entry ( None, Entry ( Some \"世界\", Empty ) ) ))\n".as_bytes());
    assert_freestanding_elf_and_valid_dwarf(&executable);
    let ir_path = directory.join("application.ll");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir_path.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListOptionalStringStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.optional.string.equal(ptr",
        "define internal ptr @topal.runtime.list.optional.string.entry.count(ptr",
        "call i1 @topal.runtime.optional.string.equal(ptr",
        "define internal fastcc { ptr, ptr } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.optional.rational.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List Optional String is Entry (Some \"value\", Empty)\nvalues reverse\n").unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
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
            "break list-optional-string-values.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        "type = List Optional String",
        "candidate = Entry ( Some \"first\", Entry ( None, Entry ( Some \"世界\", Empty ) ) )",
        "fallback = Some \"fallback\"",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}
