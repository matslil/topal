#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and both infix targets.
fn infix_collect_source_block_exits_are_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-ARRAY-COLLECT-001,
    // TOPAL-COLLECTION-COLLECT-STRING-001,
    // TOPAL-COMP-LEXICAL-RETURN-INFIX-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-INFIX-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-infix-collect");
    let source = directory.join("function-return-infix-collect.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-infix-collect.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let body = |symbol: &str| {
        ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0
    };
    for symbol in [
        "define internal fastcc ptr @topal.fn.array_2dexit",
        "define internal fastcc ptr @topal.fn.string_2dexit",
    ] {
        let function = body(symbol);
        assert_eq!(
            function.matches("call ptr @topal.runtime.int.add").count(),
            1
        );
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert!(!function.contains("@topal.platform.allocate"));
        assert!(!function.contains("insertvalue"));
        assert!(!function.contains("extractvalue"));
    }
    assert!(!ir_text.contains("@topal.runtime.container."));
    assert!(!ir_text.contains("@topal.runtime.string.collect"));
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 2);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-infix-collect.t:8",
            "-ex",
            "break function-return-infix-collect.t:11",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    assert!(text.contains("$1 = 40"), "{text}");
    assert!(text.contains("$2 = -1"), "{text}");
    assert!(text.contains("topal.fn.array_2dexit"), "{text}");
    assert!(text.contains("topal.fn.string_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn boolean_decision_subject_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMP-LEXICAL-RETURN-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-decision-subject");
    let source = directory.join("function-return-decision-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-decision-subject.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let function = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(
        function.matches("call ptr @topal.runtime.int.add").count(),
        1
    );
    assert_eq!(function.matches("ret ptr").count(), 1);
    assert!(!function.contains("br i1"));
    assert!(!function.contains("select i1"));
    assert!(!function.contains(" phi "));
    assert!(!function.contains("@topal.platform.allocate"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-decision-subject.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn exhaustive_boolean_decision_subject_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMP-LEXICAL-RETURN-EXHAUSTIVE-BOOLEAN-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-EXHAUSTIVE-BOOLEAN-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-exhaustive-boolean-subject");
    let source = directory.join("function-return-exhaustive-boolean-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-exhaustive-boolean-subject.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let function = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(
        function.matches("call ptr @topal.runtime.int.add").count(),
        1
    );
    assert_eq!(function.matches("ret ptr").count(), 1);
    assert!(!function.contains("br i1"));
    assert!(!function.contains("select i1"));
    assert!(!function.contains(" phi "));
    assert!(!function.contains("@topal.platform.allocate"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-exhaustive-boolean-subject.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn comparison_decision_subject_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-COMPARISON-001,
    // TOPAL-COMP-LEXICAL-RETURN-COMPARISON-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-comparison-decision-subject");
    let source = directory.join("function-return-comparison-decision-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-comparison-decision-subject.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let function = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(
        function.matches("call ptr @topal.runtime.int.add").count(),
        1
    );
    assert_eq!(function.matches("ret ptr").count(), 1);
    assert!(!function.contains("call i32 @topal.runtime.int.compare"));
    assert!(!function.contains("icmp"));
    assert!(!function.contains("br i1"));
    assert!(!function.contains("select i1"));
    assert!(!function.contains(" phi "));
    assert!(!function.contains("@topal.platform.allocate"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-comparison-decision-subject.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn fallback_decision_subject_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001,
    // TOPAL-COMP-LEXICAL-RETURN-FALLBACK-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-FALLBACK-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-fallback-decision-subject");
    let source = directory.join("function-return-fallback-decision-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-fallback-decision-subject.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let function = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(
        function.matches("call ptr @topal.runtime.int.add").count(),
        1
    );
    assert_eq!(function.matches("ret ptr").count(), 1);
    assert!(!function.contains("call i32 @topal.runtime.int.compare"));
    assert!(!function.contains("icmp"));
    assert!(!function.contains("br i1"));
    assert!(!function.contains("switch "));
    assert!(!function.contains("select i1"));
    assert!(!function.contains(" phi "));
    assert!(!function.contains("@topal.platform.allocate"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-fallback-decision-subject.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn complete_decision_subject_block_exits_are_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-OPTIONAL-001,
    // TOPAL-DECISION-RESULT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-COMP-LEXICAL-RETURN-COMPLETE-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPLETE-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-complete-decision-subject");
    let source = directory.join("function-return-complete-decision-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-complete-decision-subject.t"),
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
        "define internal fastcc ptr @topal.fn.optional_2dexit",
        "define internal fastcc ptr @topal.fn.result_2dexit",
        "define internal fastcc ptr @topal.fn.list_2dexit",
    ] {
        assert_direct_return_without_classifier(&ir_text, symbol);
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 3);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-complete-decision-subject.t:8",
            "-ex",
            "break function-return-complete-decision-subject.t:14",
            "-ex",
            "break function-return-complete-decision-subject.t:20",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    assert!(text.contains("$1 = 39"), "{text}");
    assert!(text.contains("$2 = 40"), "{text}");
    assert!(text.contains("$3 = 41"), "{text}");
    for symbol in [
        "topal.fn.optional_2dexit",
        "topal.fn.result_2dexit",
        "topal.fn.list_2dexit",
        "topal.main",
    ] {
        assert!(text.contains(symbol), "{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn boolean_decision_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMP-LEXICAL-RETURN-BOOLEAN-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-BOOLEAN-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-boolean-decision-actions");
    let source = directory.join("function-return-boolean-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-boolean-decision-actions.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(40, 41)\n");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    for symbol in ["@topal.fn.choose.0", "@topal.fn.choose.1"] {
        let function = ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(function.matches("br i1").count(), 1);
        assert_eq!(function.matches("br label").count(), 2);
        assert_eq!(function.matches("phi ptr").count(), 1);
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert!(!function.contains("select i1"));
        assert!(!function.contains("switch "));
        assert!(!function.contains("@topal.platform.allocate"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 4);
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
            "break function-return-boolean-decision-actions.t:9",
            "-ex",
            "break function-return-boolean-decision-actions.t:10",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    assert!(text.contains("$1 = false"), "{text}");
    assert!(text.contains("$2 = true"), "{text}");
    assert!(text.contains("topal.fn.choose.0"), "{text}");
    assert!(text.contains("topal.fn.choose.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn comparison_value_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-ENUM-001,
    // TOPAL-COMP-LEXICAL-RETURN-COMPARISON-VALUE-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-VALUE-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-comparison-value-decision-actions");
    let source = directory.join("function-return-comparison-value-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!(
            "../../../../examples/language/function-return-comparison-value-decision-actions.t"
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
        assert_eq!(function.matches("switch i32").count(), 1);
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
            "break function-return-comparison-value-decision-actions.t:9",
            "-ex",
            "break function-return-comparison-value-decision-actions.t:10",
            "-ex",
            "break function-return-comparison-value-decision-actions.t:11",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    for expected in ["$1 = Less", "$2 = Equal", "$3 = Greater"] {
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
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and three GDB paths.
fn enum_fallback_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-ENUM-001,
    // TOPAL-COMP-LEXICAL-RETURN-ENUM-FALLBACK-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ENUM-FALLBACK-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-enum-fallback-decision-actions");
    let source = directory.join("function-return-enum-fallback-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!(
            "../../../../examples/language/function-return-enum-fallback-decision-actions.t"
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
        assert_eq!(function.matches("switch i32").count(), 1);
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
            "break function-return-enum-fallback-decision-actions.t:11",
            "-ex",
            "break function-return-enum-fallback-decision-actions.t:12",
            "-ex",
            "break function-return-enum-fallback-decision-actions.t:13",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    for expected in ["$1 = Red", "$2 = Green", "$3 = Blue"] {
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
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and three GDB paths.
fn exhaustive_enum_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-ENUM-001,
    // TOPAL-COMP-LEXICAL-RETURN-ENUM-EXHAUSTIVE-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ENUM-EXHAUSTIVE-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-exhaustive-enum-decision-actions");
    let source = directory.join("function-return-exhaustive-enum-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!(
            "../../../../examples/language/function-return-exhaustive-enum-decision-actions.t"
        ),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(43, 44, 45)\n");
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
        assert_eq!(function.matches("switch i32").count(), 1);
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
            "break function-return-exhaustive-enum-decision-actions.t:11",
            "-ex",
            "break function-return-exhaustive-enum-decision-actions.t:12",
            "-ex",
            "break function-return-exhaustive-enum-decision-actions.t:13",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    for expected in ["$1 = Red", "$2 = Green", "$3 = Blue"] {
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
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and four GDB paths.
fn optional_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-OPTIONAL-001,
    // TOPAL-COMP-LEXICAL-RETURN-OPTIONAL-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-optional-decision-actions");
    let source = directory.join("function-return-optional-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-optional-decision-actions.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(43, 40, 44, 45)\n");
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
        "@topal.fn.explicit.0",
        "@topal.fn.explicit.1",
        "@topal.fn.some_2dfallback.2",
        "@topal.fn.none_2dfallback.3",
    ] {
        let function = ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(
            function.matches("@topal.runtime.optional.is.some").count(),
            1
        );
        assert_eq!(function.matches("br i1").count(), 1);
        assert_eq!(function.matches("br label").count(), 2);
        assert_eq!(function.matches("phi ptr").count(), 1);
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert!(!function.contains("select i1"));
        assert!(!function.contains("@topal.platform.allocate"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 8);
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
            "break function-return-optional-decision-actions.t:9",
            "-ex",
            "break function-return-optional-decision-actions.t:10",
            "-ex",
            "break function-return-optional-decision-actions.t:16",
            "-ex",
            "break function-return-optional-decision-actions.t:21",
            "-ex",
            "run",
            "-ex",
            "print candidate",
            "-ex",
            "nexti",
            "-ex",
            "nexti",
            "-ex",
            "print payload",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print candidate",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print candidate",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in ["$1 = Some 42", "$2 = 42", "$3 = None", "$4 = None"] {
        assert!(text.contains(expected), "{text}");
    }
    for symbol in [
        "topal.fn.explicit.0",
        "topal.fn.explicit.1",
        "topal.fn.some_2dfallback.2",
        "topal.fn.none_2dfallback.3",
        "topal.main",
    ] {
        assert!(text.contains(symbol), "{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and both Result paths.
fn result_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-RESULT-001,
    // TOPAL-COMP-LEXICAL-RETURN-RESULT-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-RESULT-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-result-decision-actions");
    let source = directory.join("function-return-result-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-result-decision-actions.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(true, true)\n");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    for symbol in ["@topal.fn.choose.1", "@topal.fn.choose.3"] {
        let function = ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(
            function.matches("@topal.runtime.result.is.error").count(),
            1
        );
        assert_eq!(function.matches("@topal.runtime.result.payload").count(), 2);
        assert_eq!(function.matches("br i1").count(), 1);
        assert_eq!(function.matches("br label").count(), 2);
        assert_eq!(function.matches("phi i1").count(), 1);
        assert_eq!(function.matches("ret i1").count(), 1);
        assert!(!function.contains("select i1"));
        assert!(!function.contains("@topal.platform.allocate"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 4);
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
            "break function-return-result-decision-actions.t:12",
            "-ex",
            "break function-return-result-decision-actions.t:13",
            "-ex",
            "run",
            "-ex",
            "nexti",
            "-ex",
            "nexti",
            "-ex",
            "print quotient",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print problem",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "$1 = Rational ( 42, 1 )",
        "$2 = Error ( domain is root./(Rational,Rational), code is division-by-zero )",
        "topal.fn.choose.1",
        "topal.fn.choose.3",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers fallback and exhaustive Error-code tables.
fn error_code_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-ERROR-CODE-001,
    // TOPAL-COMP-LEXICAL-RETURN-ERROR-CODE-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ERROR-CODE-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-error-code-decision-actions");
    let source = directory.join("function-return-error-code-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-error-code-decision-actions.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(true, false, true, 0, 3, 4)\n");
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
        "@topal.fn.recover.1",
        "@topal.fn.recover.3",
        "@topal.fn.recover.5",
    ] {
        let function = ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(
            function.matches("@topal.runtime.result.is.error").count(),
            1
        );
        assert_eq!(function.matches("@topal.runtime.result.payload").count(), 2);
        assert_eq!(function.matches("@topal.runtime.error.code").count(), 3);
        assert_eq!(function.matches("switch i32").count(), 1);
        assert_eq!(function.matches("i32 2, label").count(), 1);
        assert_eq!(function.matches("br i1").count(), 1);
        assert_eq!(function.matches("br label").count(), 3);
        assert_eq!(function.matches("phi i1").count(), 1);
        assert_eq!(function.matches("ret i1").count(), 1);
        assert!(!function.contains("select i1"));
        assert!(!function.contains("@topal.platform.allocate"));
    }
    for symbol in [
        "@topal.fn.classify.7",
        "@topal.fn.classify.9",
        "@topal.fn.classify.11",
    ] {
        let function = ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(
            function.matches("@topal.runtime.result.is.error").count(),
            1
        );
        assert_eq!(function.matches("@topal.runtime.result.payload").count(), 2);
        assert_eq!(function.matches("@topal.runtime.error.code").count(), 1);
        assert_eq!(function.matches("switch i32").count(), 1);
        assert_eq!(function.matches("label %result.decision.code").count(), 4);
        assert_eq!(function.matches("br i1").count(), 1);
        assert_eq!(function.matches("br label").count(), 5);
        assert_eq!(function.matches("phi ptr").count(), 1);
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert_eq!(function.matches("@topal.platform.exit(i64 70)").count(), 1);
        assert_eq!(function.matches("unreachable").count(), 1);
        assert!(!function.contains("select i1"));
        assert!(!function.contains("@topal.platform.allocate"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 24);
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
            "break function-return-error-code-decision-actions.t:12",
            "-ex",
            "break function-return-error-code-decision-actions.t:13",
            "-ex",
            "break function-return-error-code-decision-actions.t:14",
            "-ex",
            "break function-return-error-code-decision-actions.t:22",
            "-ex",
            "break function-return-error-code-decision-actions.t:23",
            "-ex",
            "run",
            "-ex",
            "nexti",
            "-ex",
            "nexti",
            "-ex",
            "print quotient",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print problem",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "$1 = Rational ( 1, 2 )",
        "$2 = Error ( domain is root.Rational(Int,Int), code is indeterminate )",
        "topal.fn.recover.1",
        "topal.fn.recover.3",
        "topal.fn.recover.5",
        "topal.fn.classify.9",
        "topal.fn.classify.11",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn list_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-LIST-001,
    // TOPAL-COMP-LEXICAL-RETURN-LIST-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-LIST-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-list-decision-actions");
    let source = directory.join("function-return-list-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-list-decision-actions.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(43, 40)\n");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    for symbol in ["@topal.fn.choose.0", "@topal.fn.choose.1"] {
        let function = ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(function.matches("icmp eq ptr").count(), 1);
        assert_eq!(function.matches("load ptr").count(), 2);
        assert_eq!(function.matches("br i1").count(), 1);
        assert_eq!(function.matches("br label").count(), 2);
        assert_eq!(function.matches("phi ptr").count(), 1);
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert!(!function.contains("@topal.platform.allocate"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 4);
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
            "break function-return-list-decision-actions.t:9",
            "-ex",
            "break function-return-list-decision-actions.t:10",
            "-ex",
            "run",
            "-ex",
            "nexti",
            "-ex",
            "nexti",
            "-ex",
            "nexti",
            "-ex",
            "print first",
            "-ex",
            "print rest",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "$1 = 42",
        "$2 = Entry ( 9, Empty )",
        "topal.fn.choose.0",
        "topal.fn.choose.1",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}
