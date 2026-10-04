#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn lexical_block_return_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-lexical-block-return");
    let source = directory.join("function-return-from-block.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-from-block.t"),
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
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir_text = fs::read_to_string(&ir).unwrap();
    assert!(ir_text.contains("define internal fastcc ptr @topal.fn.answer"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-from-block.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
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
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn return_operand_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-OPERAND-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERAND-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-operand-block");
    let source = directory.join("function-return-block-operand.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-block-operand.t"),
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
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir_text = fs::read_to_string(&ir).unwrap();
    assert!(ir_text.contains("define internal fastcc ptr @topal.fn.answer"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-block-operand.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
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
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers native output, exact IR, DWARF, and both GDB stops.
fn operator_operand_block_exits_are_ordered_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-OPERATOR-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERATOR-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-operator-operand");
    let source = directory.join("function-return-operator-operand.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-operator-operand.t"),
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
    assert_eq!(executed.stdout, b"(42, 43)\n");

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir_text = fs::read_to_string(&ir).unwrap();
    let right = ir_text
        .split_once("define internal fastcc ptr @topal.fn.right_2dexit")
        .unwrap()
        .1
        .split_once("define internal fastcc ptr @topal.fn.left_2dexit")
        .unwrap()
        .0;
    assert_eq!(
        right.matches("call fastcc ptr @topal.fn.preceding").count(),
        1
    );
    assert_eq!(right.matches("call ptr @topal.runtime.int.add").count(), 1);
    let left = ir_text
        .split_once("define internal fastcc ptr @topal.fn.left_2dexit")
        .unwrap()
        .1
        .split_once("define internal void @topal.main")
        .unwrap()
        .0;
    assert_eq!(left.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 2);
    assert!(!ir_text.contains("missing"));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-operator-operand.t:9",
            "-ex",
            "break function-return-operator-operand.t:12",
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
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("topal.fn.right_2dexit"), "{text}");
    assert!(text.contains("topal.fn.left_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers native output, exact IR, DWARF, and both product forms.
fn product_field_block_exits_are_ordered_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-PRODUCT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-PRODUCT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-product-field");
    let source = directory.join("function-return-product-field.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-product-field.t"),
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
    assert_eq!(executed.stdout, b"(42, 43)\n");

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir_text = fs::read_to_string(&ir).unwrap();
    let tuple = ir_text
        .split_once("define internal fastcc ptr @topal.fn.tuple_2dexit")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    let record = ir_text
        .split_once("define internal fastcc ptr @topal.fn.record_2dexit")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    for function in [tuple, record] {
        assert_eq!(
            function
                .matches("call fastcc ptr @topal.fn.preceding")
                .count(),
            2
        );
        assert_eq!(
            function.matches("call ptr @topal.runtime.int.add").count(),
            1
        );
        assert!(!function.contains("insertvalue"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 2);
    assert!(!ir_text.contains("missing"));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-product-field.t:9",
            "-ex",
            "break function-return-product-field.t:12",
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
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("topal.fn.tuple_2dexit"), "{text}");
    assert!(text.contains("topal.fn.record_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers native output, exact IR, DWARF, and three GDB stops.
fn named_call_argument_block_exits_are_ordered_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-CALL-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CALL-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-call-argument");
    let source = directory.join("function-return-call-argument.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-call-argument.t"),
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
    assert_eq!(executed.stdout, b"(42, 43, 44)\n");

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
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
    let right = body("define internal fastcc ptr @topal.fn.right_2dexit");
    let left = body("define internal fastcc ptr @topal.fn.left_2dexit");
    let unary = body("define internal fastcc ptr @topal.fn.unary_2dexit");
    assert_eq!(
        right.matches("call fastcc ptr @topal.fn.preceding").count(),
        1
    );
    for function in [right, left, unary] {
        assert_eq!(
            function.matches("call ptr @topal.runtime.int.add").count(),
            1
        );
        assert!(!function.contains("call fastcc ptr @topal.fn.combine"));
        assert!(!function.contains("call fastcc ptr @topal.fn.identity"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 3);
    assert!(!ir_text.contains("missing"));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-call-argument.t:13",
            "-ex",
            "break function-return-call-argument.t:16",
            "-ex",
            "break function-return-call-argument.t:19",
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
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("$3 = 41"), "{text}");
    assert!(text.contains("topal.fn.right_2dexit"), "{text}");
    assert!(text.contains("topal.fn.left_2dexit"), "{text}");
    assert!(text.contains("topal.fn.unary_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn optional_constructor_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-COMP-LEXICAL-RETURN-OPTIONAL-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-optional-constructor");
    let source = directory.join("function-return-optional-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-optional-constructor.t"),
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
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let ir_text = fs::read_to_string(&ir).unwrap();
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert!(!answer.contains("call ptr @topal.runtime.optional.some"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-optional-constructor.t:7",
            "-ex",
            "run",
            "-ex",
            "print value",
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
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and five constructor-source stops.
fn strict_unary_constructor_argument_block_exits_are_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-UNION-001,
    // TOPAL-COMP-LEXICAL-RETURN-CONSTRUCTOR-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-unary-constructor");
    let source = directory.join("function-return-unary-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-unary-constructor.t"),
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
    assert_eq!(executed.stdout, b"(42, 43, 44, 45, 46)\n");

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
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
        "define internal fastcc ptr @topal.fn.string_2dexit",
        "define internal fastcc ptr @topal.fn.int_2dexit",
        "define internal fastcc ptr @topal.fn.nat_2dexit",
        "define internal fastcc ptr @topal.fn.rational_2dexit",
        "define internal fastcc ptr @topal.fn.union_2dexit",
    ] {
        let function = body(symbol);
        assert_eq!(
            function.matches("call ptr @topal.runtime.int.add").count(),
            1
        );
        assert!(!function.contains("@topal.runtime.optional.some"));
        assert!(!function.contains("@topal.runtime.rational.construct"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 5);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-unary-constructor.t:10",
            "-ex",
            "break function-return-unary-constructor.t:13",
            "-ex",
            "break function-return-unary-constructor.t:16",
            "-ex",
            "break function-return-unary-constructor.t:19",
            "-ex",
            "break function-return-unary-constructor.t:22",
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
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for index in 1..=5 {
        assert!(text.contains(&format!("${index} = 41")), "{text}");
    }
    for symbol in [
        "topal.fn.string_2dexit",
        "topal.fn.int_2dexit",
        "topal.fn.nat_2dexit",
        "topal.fn.rational_2dexit",
        "topal.fn.union_2dexit",
        "topal.main",
    ] {
        assert!(text.contains(symbol), "{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn positional_variant_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-VARIANT-001,
    // TOPAL-COMP-LEXICAL-RETURN-VARIANT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-VARIANT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-variant-constructor");
    let source = directory.join("function-return-variant-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-variant-constructor.t"),
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
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
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
            "break function-return-variant-constructor.t:9",
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
fn character_constructor_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-STRING-CHARACTER-CLASSIFIER-001,
    // TOPAL-COMP-LEXICAL-RETURN-CHARACTER-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CHARACTER-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-character-constructor");
    let source = directory.join("function-return-character-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-character-constructor.t"),
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
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
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
            "break function-return-character-constructor.t:7",
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
fn named_constraint_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMP-LEXICAL-RETURN-CONSTRAINT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRAINT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-constraint-constructor");
    let source = directory.join("function-return-constraint-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-constraint-constructor.t"),
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
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("icmp"));
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
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
            "break function-return-constraint-constructor.t:9",
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
fn named_modular_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-NUM-MODULAR-CONSTRUCT-001,
    // TOPAL-COMP-LEXICAL-RETURN-MODULAR-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-modular-constructor");
    let source = directory.join("function-return-modular-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-modular-constructor.t"),
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
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("icmp"));
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
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
            "break function-return-modular-constructor.t:9",
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
fn modular_reduction_operand_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-NUM-MODULAR-REDUCE-001,
    // TOPAL-COMP-LEXICAL-RETURN-MODULAR-REDUCE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-modular-reduction");
    let source = directory.join("function-return-modular-reduction.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-modular-reduction.t"),
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
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.runtime.int.subtract"));
    assert!(!answer.contains("@topal.runtime.int.modulo"));
    assert!(!answer.contains("@topal.runtime.result."));
    assert!(!answer.contains("icmp"));
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
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
            "break function-return-modular-reduction.t:9",
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
fn list_collect_source_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COLLECTION-COLLECT-LIST-001,
    // TOPAL-COMP-LEXICAL-RETURN-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COLLECT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-list-collect");
    let source = directory.join("function-return-list-collect.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-list-collect.t"),
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
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.runtime.list."));
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
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
            "break function-return-list-collect.t:8",
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
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and both unordered collectors.
fn unordered_collect_source_block_exits_are_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-SET-COLLECT-001,
    // TOPAL-BAG-COLLECT-001, TOPAL-COMP-LEXICAL-RETURN-UNORDERED-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-unordered-collect");
    let source = directory.join("function-return-unordered-collect.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-unordered-collect.t"),
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
        "define internal fastcc ptr @topal.fn.set_2dexit",
        "define internal fastcc ptr @topal.fn.bag_2dexit",
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
            "break function-return-unordered-collect.t:8",
            "-ex",
            "break function-return-unordered-collect.t:11",
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
    assert!(text.contains("topal.fn.set_2dexit"), "{text}");
    assert!(text.contains("topal.fn.bag_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn map_collect_source_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-MAP-COLLECT-001,
    // TOPAL-COMP-LEXICAL-RETURN-MAP-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-map-collect");
    let source = directory.join("function-return-map-collect.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-return-map-collect.t"),
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
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
    assert!(!ir_text.contains("@topal.runtime.container."));
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
            "break function-return-map-collect.t:8",
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
