#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn packaged_function_operand_is_flat_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-PACKAGED-OPERAND-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-packaged-function-operand");
    let source = directory.join("packaged-function-operand.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/packaged-function-operand.t"),
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
            "break packaged-function-operand.t:13",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "print fallback",
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
    assert!(text.contains("$1 = 40"), "{text}");
    assert!(text.contains("$2 = 2"), "{text}");
    assert!(text.contains("topal.fn.sum.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers ordering, rejection, artifacts, and GDB.
fn packaged_field_association_preserves_source_order_and_remains_debuggable() {
    // TOPAL-COMPILER-PACKAGED-ASSOCIATION-ORDER-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-packaged-function-association-order");
    let source = directory.join("packaged-function-association-order.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/packaged-function-association-order.t"),
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
    assert_eq!(executed.stdout, b"(42, 42, 42)\n");
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
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let right = main
        .find("call fastcc ptr @topal.fn.right_2dvalue")
        .unwrap();
    let left = main.find("call fastcc ptr @topal.fn.left_2dvalue").unwrap();
    let combine = main.find("call fastcc ptr @topal.fn.combine").unwrap();
    assert!(right < left && left < combine, "{main}");
    assert!(
        main.contains("@topal.fn.combine.2(ptr %v1, ptr @.topal.int.2, ptr %v0)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, call) in [
        ("unknown", "combine (left is 39, unknown is 1, right is 2)"),
        ("duplicate", "combine (left is 39, right is 1, right is 2)"),
    ] {
        let rejected_source = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(
            &rejected_source,
            format!(
                "use language (version is v0.1)\ncombine is fn ((left : Int, offset : Int default 1, right : Int)) -> Int\n  left + offset + right\n{call}\n"
            ),
        )
        .unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_source.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
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
            "break packaged-function-association-order.t:14",
            "-ex",
            "run",
            "-ex",
            "print left",
            "-ex",
            "print offset",
            "-ex",
            "print right",
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
    assert!(text.contains("$1 = 39"), "{text}");
    assert!(text.contains("$2 = 1"), "{text}");
    assert!(text.contains("$3 = 2"), "{text}");
    assert!(text.contains("topal.fn.combine.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers operand order, rejection, artifacts, and GDB.
fn compound_packaged_operands_are_flat_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-COMPOUND-PACKAGED-OPERAND-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-compound-packaged-function-operands");
    let source = directory.join("compound-packaged-function-operands.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/compound-packaged-function-operands.t"),
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
    assert_eq!(executed.stdout, b"(42, 42, 42, 42, 42)\n");
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
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.combine.2(ptr %arg0, ptr %arg1, ptr %arg2, ptr %arg3)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc ptr @topal.fn.scale.7(ptr %arg0, ptr %arg1, ptr %arg2)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc ptr @topal.fn.shift.8(ptr %arg0, ptr %arg1, ptr %arg2)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let left = main.find("call fastcc ptr @topal.fn.left_2dvalue").unwrap();
    let right = main
        .find("call fastcc ptr @topal.fn.right_2dvalue")
        .unwrap();
    let combine = main.find("call fastcc ptr @topal.fn.combine").unwrap();
    let scale_value = main
        .find("call fastcc ptr @topal.fn.scale_2dvalue")
        .unwrap();
    let scale_factor = main
        .find("call fastcc ptr @topal.fn.scale_2dfactor")
        .unwrap();
    let scale = main.find("call fastcc ptr @topal.fn.scale.7").unwrap();
    assert!(left < right && right < combine, "{main}");
    assert!(scale_value < scale_factor && scale_factor < scale, "{main}");
    assert!(
        main.contains(
            "@topal.fn.combine.2(ptr %v0, ptr @.topal.int.4, ptr %v1, ptr @.topal.int.5)"
        ),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, text) in [
        (
            "unknown",
            "use language (version is v0.1)\ncombine is fn ((left : Int), (right : Int)) -> Int\n  left + right\n(left is 20) combine (unknown is 22)\n",
        ),
        (
            "duplicate-parameter",
            "use language (version is v0.1)\ncombine is fn ((value : Int), (value : Int)) -> Int\n  value\n(value is 20) combine (value is 22)\n",
        ),
    ] {
        let rejected_source = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_source, text).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_source.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
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
            "break compound-packaged-function-operands.t:21",
            "-ex",
            "run",
            "-ex",
            "print left",
            "-ex",
            "print 'left-offset'",
            "-ex",
            "print right",
            "-ex",
            "print 'right-offset'",
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
    assert!(text.contains("$1 = 20"), "{text}");
    assert!(text.contains("$2 = 1"), "{text}");
    assert!(text.contains("$3 = 21"), "{text}");
    assert!(text.contains("$4 = 0"), "{text}");
    assert!(text.contains("topal.fn.combine.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact aggregate fields, rejection, artifacts, and GDB.
fn structured_packaged_fields_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-STRUCTURED-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-structured-packaged-function-fields");
    let source = directory.join("structured-packaged-function-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/structured-packaged-function-fields.t"),
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
        b"(((20, 22), \"Ada\"), ((20, 22), \"Ada\", true), ((21, 21), \"default\", false), ((20, 22), \"Grace\", true))\n"
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
    assert!(
        ir.contains(
            "define internal fastcc { { ptr, ptr }, ptr } @topal.fn.retain_2done.2({ ptr, ptr } %arg0, { i1, ptr, i32, i32 } %arg1)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc { { ptr, ptr }, ptr, i1 } @topal.fn.retain_2dmixed.5({ ptr, ptr } %arg0, { i1, ptr, i32, i32 } %arg1, i1 %arg2)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let person = main
        .find("call fastcc { i1, ptr, i32, i32 } @topal.fn.make_2dperson.0")
        .unwrap();
    let pair = main
        .find("call fastcc { ptr, ptr } @topal.fn.make_2dpair.1")
        .unwrap();
    let retained = main
        .find("call fastcc { { ptr, ptr }, ptr } @topal.fn.retain_2done.2")
        .unwrap();
    assert!(person < pair && pair < retained, "{main}");
    assert!(
        main.contains("@topal.fn.retain_2done.2({ ptr, ptr } %v9, { i1, ptr, i32, i32 } %v13)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("opaque-package.t");
    let rejected_executable = directory.join("opaque-package");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nretain is fn ((pair : (Int, Int), person : Record (name : String, active : Boolean))) -> Int\n  0\nbundle is (pair is (20, 22), person is (name is \"Ada\", active is true))\nretain bundle\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(!rejected_executable.exists());
    assert!(!metadata_path(&rejected_executable).exists());

    let rejected_root_path = directory.join("root-callable-aggregate.t");
    let rejected_root_executable = directory.join("root-callable-aggregate");
    fs::write(
        &rejected_root_path,
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nread is fn () -> (Function, Int)\n  root bundle\nbundle is (increment, 40)\nread ()\n",
    )
    .unwrap();
    let rejected_root = run(topalc().args([
        "-o",
        rejected_root_executable.to_str().unwrap(),
        rejected_root_path.to_str().unwrap(),
    ]));
    assert!(!rejected_root.status.success());
    assert!(
        String::from_utf8_lossy(&rejected_root.stderr)
            .contains("unsupported function-body root data capture representation"),
        "{}",
        String::from_utf8_lossy(&rejected_root.stderr)
    );
    assert!(!rejected_root_executable.exists());
    assert!(!metadata_path(&rejected_root_executable).exists());

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
            "break structured-packaged-function-fields.t:14",
            "-ex",
            "run",
            "-ex",
            "print pair",
            "-ex",
            "print person",
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
    assert!(text.contains("$1 = {_0 = 20, _1 = 22}"), "{text}");
    assert!(
        text.contains("$2 = {active = true, name = \"Ada\"}"),
        "{text}"
    );
    assert!(text.contains("topal.fn.retain_2done.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers nominal Sum fields, rejection, artifacts, and GDB.
fn sum_packaged_fields_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-SUM-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-sum-packaged-function-fields");
    let source = directory.join("sum-packaged-function-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/sum-packaged-function-fields.t"),
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
    assert_eq!(executed.stdout, b"(42, 42, 42, 42)\n");
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
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.score.1({ i32, ptr } %arg0, ptr %arg1)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc ptr @topal.fn.shift_2dscore.7({ i32, ptr } %arg0, ptr %arg1)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let message = main
        .find("call fastcc { i32, ptr } @topal.fn.make_2dmessage.5")
        .unwrap();
    let offset = main
        .find("call fastcc ptr @topal.fn.offset_2dvalue.6")
        .unwrap();
    let shifted = main
        .find("call fastcc ptr @topal.fn.shift_2dscore.7")
        .unwrap();
    assert!(message < offset && offset < shifted, "{main}");
    assert!(
        main.contains("@topal.fn.score.1({ i32, ptr } %v4, ptr @.topal.int.4)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, text) in [
        (
            "opaque-package",
            "use language (version is v0.1)\nMessage is Union\n  Stop\n  Move : Int\nscore is fn ((message : Message, fallback : Int)) -> Int\n  fallback\nbundle is (message is Move 42, fallback is 0)\nscore bundle\n",
        ),
        (
            "function-sum-field",
            "use language (version is v0.1)\nCarrier is Union\n  Carry : Function\nadd is +\napply is fn ((value : Carrier)) -> Int\n  0\napply (value is Carry add)\n",
        ),
    ] {
        let rejected_source = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_source, text).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_source.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
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
            "break sum-packaged-function-fields.t:20",
            "-ex",
            "run",
            "-ex",
            "whatis message",
            "-ex",
            "print message",
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
    assert!(text.contains("type = Message"), "{text}");
    assert!(text.contains("$1 = Move 42"), "{text}");
    assert!(text.contains("topal.fn.score.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Function fields, captures, rejection, artifacts, and GDB.
fn function_packaged_fields_retain_exact_callable_facts_and_debugging() {
    // TOPAL-COMPILER-FUNCTION-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-packaged-fields");
    let source = directory.join("function-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-packaged-fields.t"),
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
    assert_eq!(executed.stdout, b"(42, 42, 42, 42)\n");
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
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.apply.2(i32 %arg0, ptr %arg1)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc ptr @topal.fn.apply_2dcaptured.6(i32 %arg0, ptr %arg1, ptr %arg2)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let value = main
        .find("call fastcc ptr @topal.fn.make_2dvalue.0")
        .unwrap();
    let operation = main
        .find("call fastcc i32 @topal.fn.make_2doperation.1")
        .unwrap();
    let applied = main.find("call fastcc ptr @topal.fn.apply.2").unwrap();
    assert!(value < operation && operation < applied, "{main}");
    assert!(
        main.contains("@topal.fn.apply.2(i32 %v1, ptr %v0)"),
        "{main}"
    );
    assert!(
        main.contains("@topal.fn.apply_2dcaptured.6(i32 22, ptr @.topal.int.7, ptr @.topal.int.4)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
        "call fastcc ptr %",
        "call fastcc i32 %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("opaque-package.t");
    let rejected_executable = directory.join("opaque-package");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\napply is fn ((operation : Function, value : Int)) -> Int\n  operation (value, 2)\nbundle is (operation is +, value is 40)\napply bundle\n",
    )
    .unwrap();
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
            "break function-packaged-fields.t:13",
            "-ex",
            "run",
            "-ex",
            "whatis operation",
            "-ex",
            "print operation",
            "-ex",
            "print value",
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
    assert!(text.contains("type = enum Function"), "{text}");
    assert!(text.contains("$1 = +"), "{text}");
    assert!(text.contains("$2 = 40"), "{text}");
    assert!(text.contains("topal.fn.apply.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers aggregate Function fields, captures, rejection, artifacts, and GDB.
fn function_aggregate_packaged_fields_retain_recursive_callable_facts() {
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-aggregate-packaged-fields");
    let source = directory.join("function-aggregate-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-aggregate-packaged-fields.t"),
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
    assert_eq!(executed.stdout, b"(42, 42, 42)\n");
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
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.apply_2drecord.2({ i32, ptr, i32, i32 } %arg0, ptr %arg1)"),
        "{ir}"
    );
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.apply_2dtuple.6({ i32, ptr } %arg0, ptr %arg1, ptr %arg2)"),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let addend = main
        .find("call fastcc ptr @topal.fn.make_2daddend.0")
        .unwrap();
    let bundle = main
        .find("call fastcc { i32, ptr, i32, i32 } @topal.fn.make_2drecord.1")
        .unwrap();
    let applied = main
        .find("call fastcc ptr @topal.fn.apply_2drecord.2")
        .unwrap();
    assert!(addend < bundle && bundle < applied, "{main}");
    assert!(
        main.contains("@topal.fn.apply_2drecord.2({ i32, ptr, i32, i32 } %v9, ptr %v0)"),
        "{main}"
    );
    assert!(
        main.contains("@topal.fn.apply_2dtuple.6({ i32, ptr } %v17, ptr @.topal.int.3, ptr %v15)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
        "call fastcc ptr %",
        "call fastcc i32 %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("opaque-function-aggregate.t");
    let rejected_executable = directory.join("opaque-function-aggregate");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\napply is fn ((bundle : Record (operation : Function, value : Int))) -> Int\n  (bundle operation) (bundle value)\nouter is (bundle is (operation is { value } value + 2, value is 40))\napply outer\n",
    )
    .unwrap();
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
            "break function-aggregate-packaged-fields.t:22",
            "-ex",
            "run",
            "-ex",
            "whatis bundle",
            "-ex",
            "print bundle",
            "-ex",
            "print marker",
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
    assert!(text.contains("type = struct (Function, Int)"), "{text}");
    assert!(text.contains("_0 = <anonymous fn/1>, _1 = 40"), "{text}");
    assert!(text.contains("$2 = 0"), "{text}");
    assert!(text.contains("topal.fn.apply_2dtuple.6"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers represented containers, rejection, artifacts, and GDB.
fn container_packaged_fields_retain_exact_private_representations() {
    // TOPAL-COMPILER-CONTAINER-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-container-packaged-fields");
    let source = directory.join("container-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/container-packaged-fields.t"),
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
        b"((Entry ( 20, Entry ( 22, Empty ) ), Some 42, 42, 40 ..= 42), (Entry ( 20, Entry ( 22, Empty ) ), None, 42, 40 ..= 42))\n"
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
    assert!(
        ir.contains(
            "define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.4(ptr %arg0, ptr %arg1, ptr %arg2, ptr %arg3)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let span = main
        .find("call fastcc ptr @topal.fn.make_2dspan.0")
        .unwrap();
    let outcome = main
        .find("call fastcc ptr @topal.fn.make_2doutcome.1")
        .unwrap();
    let maybe = main
        .find("call fastcc ptr @topal.fn.make_2dmaybe.2")
        .unwrap();
    let values = main
        .find("call fastcc ptr @topal.fn.make_2dvalues.3")
        .unwrap();
    let retained = main
        .find("call fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.4")
        .unwrap();
    assert!(
        span < outcome && outcome < maybe && maybe < values && values < retained,
        "{main}"
    );
    assert!(
        main.contains("@topal.fn.retain.4(ptr %v3, ptr %v2, ptr %v1, ptr %v0)"),
        "{main}"
    );
    let second_span = main.find("call ptr @topal.runtime.range.make").unwrap();
    let second_outcome = main
        .find("call fastcc ptr @topal.fn.make_2doutcome.5")
        .unwrap();
    let second_values = main
        .find("call fastcc ptr @topal.fn.make_2dvalues.6")
        .unwrap();
    let defaulted_maybe = main.find("call ptr @topal.runtime.optional.none").unwrap();
    let second_retained = main
        .find("call fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.7")
        .unwrap();
    assert!(
        second_span < second_outcome
            && second_outcome < second_values
            && second_values < defaulted_maybe
            && defaulted_maybe < second_retained,
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
    ] {
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
            "break container-packaged-fields.t:20",
            "-ex",
            "run",
            "-ex",
            "whatis values",
            "-ex",
            "print values",
            "-ex",
            "whatis maybe",
            "-ex",
            "print maybe",
            "-ex",
            "whatis outcome",
            "-ex",
            "print outcome",
            "-ex",
            "whatis span",
            "-ex",
            "print span",
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
    assert!(text.contains("type = List Int"), "{text}");
    assert!(
        text.contains("$1 = Entry ( 20, Entry ( 22, Empty ) )"),
        "{text}"
    );
    assert!(text.contains("type = Optional Int"), "{text}");
    assert!(text.contains("$2 = Some 42"), "{text}");
    assert!(
        text.contains("type = Result (Int, lang arithmetic ArithmeticErrorCode)"),
        "{text}"
    );
    assert!(text.contains("$3 = 42"), "{text}");
    assert!(text.contains("type = Range Int"), "{text}");
    assert!(text.contains("$4 = 40 ..= 42"), "{text}");
    assert!(text.contains("topal.fn.retain.4"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact collections, rejection, artifacts, and GDB.
fn collection_packaged_fields_retain_exact_private_representations() {
    // TOPAL-COMPILER-COLLECTION-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-collection-packaged-fields");
    let source = directory.join("collection-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/collection-packaged-fields.t"),
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
        b"((Array (2, 1, 2), Set (2, 1), Bag ((2, 2), (1, 1)), Map ((\"Ada\", 11), (\"Lin\", 8))), (Array (2, 1, 2), Set (2, 1), Bag ((2, 2), (1, 1)), Map ((\"Ada\", 11), (\"Lin\", 8))))\n"
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
    assert!(
        ir.contains(
            "define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.4(ptr %arg0, ptr %arg1, ptr %arg2, ptr %arg3)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let scores = main.find("call fastcc ptr @topal.fn.make_2dmap.0").unwrap();
    let occurrences = main.find("call fastcc ptr @topal.fn.make_2dbag.1").unwrap();
    let members = main.find("call fastcc ptr @topal.fn.make_2dset.2").unwrap();
    let array = main
        .find("call fastcc ptr @topal.fn.make_2darray.3")
        .unwrap();
    let retained = main
        .find("call fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.4")
        .unwrap();
    assert!(
        scores < occurrences && occurrences < members && members < array && array < retained,
        "{main}"
    );
    assert!(
        main.contains("@topal.fn.retain.4(ptr %v3, ptr %v2, ptr %v1, ptr %v0)"),
        "{main}"
    );
    for call in [
        "call fastcc ptr @topal.fn.make_2darray.5",
        "call fastcc ptr @topal.fn.make_2dset.6",
        "call fastcc ptr @topal.fn.make_2dbag.7",
        "call fastcc ptr @topal.fn.make_2dmap.8",
        "call fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.9(ptr %v9, ptr %v10, ptr %v11, ptr %v12)",
    ] {
        assert!(main.contains(call), "{call}: {main}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("mismatched-array-field.t");
    let rejected_executable = directory.join("mismatched-array-field");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nmake is fn () -> Array (2, Int)\n  values : List Int is Entry (1, Entry (2, Entry (3, Empty)))\n  values collect Array\nmake ()\n",
    )
    .unwrap();
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
            "break collection-packaged-fields.t:24",
            "-ex",
            "run",
            "-ex",
            "whatis array",
            "-ex",
            "print array",
            "-ex",
            "whatis members",
            "-ex",
            "print members",
            "-ex",
            "whatis occurrences",
            "-ex",
            "print occurrences",
            "-ex",
            "whatis scores",
            "-ex",
            "print scores",
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
    assert!(text.contains("type = Array 3 Int"), "{text}");
    assert!(text.contains("$1 = Array (2, 1, 2)"), "{text}");
    assert!(text.contains("type = Set Int"), "{text}");
    assert!(text.contains("$2 = Set (2, 1)"), "{text}");
    assert!(text.contains("type = Bag Int"), "{text}");
    assert!(text.contains("$3 = Bag ((2, 2), (1, 1))"), "{text}");
    assert!(text.contains("type = Map(String, Int)"), "{text}");
    assert!(
        text.contains("$4 = Map ((\"Ada\", 11), (\"Lin\", 8))"),
        "{text}"
    );
    assert!(text.contains("topal.fn.retain.4"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Scope packages, rejection, artifacts, and GDB.
fn scope_packaged_fields_retain_exact_private_environments() {
    // TOPAL-COMPILER-SCOPE-PACKAGED-FIELD-001,
    // TOPAL-COMPILER-NAMESPACE-BOUNDARY-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-scope-packaged-fields");
    let source = directory.join("scope-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/scope-packaged-fields.t"),
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
    assert_eq!(executed.stdout, b"((42, 42), (42, 42), (42, 42))\n");
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
    for definition in [
        "define internal fastcc { ptr, ptr } @topal.fn.observe.1(i32 %arg0, ptr %arg1, ptr %arg2)",
        "define internal fastcc { ptr, ptr } @topal.fn.observe.2(i32 %arg0, ptr %arg1, ptr %arg2)",
        "define internal fastcc { ptr, ptr } @topal.fn.observe_2ddefault.4(i32 %arg0, ptr %arg1, ptr %arg2)",
    ] {
        assert!(ir.contains(definition), "{definition}: {ir}");
    }
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let value = main
        .find("call fastcc ptr @topal.fn.make_2dvalue.0")
        .unwrap();
    let observed = main
        .find("call fastcc { ptr, ptr } @topal.fn.observe.1")
        .unwrap();
    assert!(value < observed, "{main}");
    for call in [
        "@topal.fn.observe.1(i32 0, ptr %v1, ptr %v0)",
        "@topal.fn.observe.2(i32 0, ptr @.topal.int.7, ptr %v0)",
        "@topal.fn.observe_2ddefault.4(i32 0, ptr %v8, ptr %v0)",
    ] {
        assert!(main.contains(call), "{call}: {main}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "scope.runtime",
        "topal.package.argument",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("function-live-root-package.t");
    let rejected_executable = directory.join("function-live-root-package");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nanswer is 42\naccept is fn ((api : Scope, value : Int)) -> Int\n  api answer + value\nwrapper is fn () -> Int\n  accept (api is root, value is 0)\nwrapper ()\n",
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
            .contains("function-body live root Scope argument"),
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
            "break scope-packaged-fields.t:18",
            "-ex",
            "run",
            "-ex",
            "whatis scope",
            "-ex",
            "print scope",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
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
    assert!(text.contains("type = enum Scope"), "{text}");
    assert!(text.contains("$1 = <namespace root>"), "{text}");
    assert!(text.contains("type = Int"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("scope answer = 42"), "{text}");
    assert!(text.contains("topal.fn.observe.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers lowering, rejection, artifacts, and GDB.
fn function_root_data_is_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-001, TOPAL-NAMESPACE-ROOT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-root-data");
    let source = directory.join("function-root-data.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-root-data.t"),
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
    assert!(
        ir.contains(
            "define internal fastcc { ptr, ptr, ptr } @topal.fn.read.1(ptr %arg0, ptr %arg1, ptr %arg2)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let initialized = main
        .find("call fastcc ptr @topal.fn.make_2danswer.0()")
        .unwrap();
    let invoked = main
        .find("call fastcc { ptr, ptr, ptr } @topal.fn.read.1(ptr @.topal.int.4, ptr %v0, ptr %v1)")
        .unwrap();
    assert!(initialized < invoked, "{main}");
    for forbidden in [
        "topal.root",
        "root.runtime",
        "namespace.runtime",
        "context.runtime",
        "lookup.root",
    ] {
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
            "break function-root-data.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis answer",
            "-ex",
            "print answer",
            "-ex",
            "print 'root label'",
            "-ex",
            "print 'root answer'",
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
    assert!(text.contains("type = Int"), "{text}");
    assert!(text.contains("$1 = 0"), "{text}");
    assert!(text.contains("$2 = \"ready\""), "{text}");
    assert!(text.contains("$3 = 42"), "{text}");
    assert!(
        text.contains("answer=0, root label=\"ready\", root answer=42"),
        "{text}"
    );
    assert!(text.contains("topal.fn.read.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}
