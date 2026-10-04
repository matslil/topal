#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn modular_values_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-MODULAR-001, TOPAL-NUM-MODULAR-TYPE-001,
    // TOPAL-NUM-MODULAR-REDUCE-001, TOPAL-NUM-MODULAR-ARITHMETIC-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-modular-values");
    let source = directory.join("modular-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nSignedByte is ModInt ((-128) ..= 127)\nretain is fn (value : ByteCounter) -> ByteCounter\n  value\nHuge is ModNat (0 ..= 340282366920938463463374607431768211455)\nstart is retain (ByteCounter 255)\nwrapped is 128 modulo SignedByte\nhuge is (Huge 340282366920938463463374607431768211455) + (Huge 1)\n(start, wrapped, start + (ByteCounter 1), (ByteCounter 0) - (ByteCounter 1), (ByteCounter 16) * (ByteCounter 16), -(SignedByte (-128)), (ByteCounter 2) <=> (ByteCounter 1), huge)\n",
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
        b"(ByteCounter 255, SignedByte -128, ByteCounter 0, ByteCounter 255, ByteCounter 0, SignedByte -128, Greater, Huge 0)\n"
    );

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
            "break modular-values.t:5",
            "-ex",
            "run",
            "-ex",
            "whatis value",
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
    assert!(text.contains("type = ByteCounter"), "{text}");
    assert!(text.contains("$1 = ByteCounter 255"), "{text}");
    assert!(text.contains("topal.fn.retain.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn dynamic_modular_results_are_freestanding_and_debuggable() {
    // TOPAL-COMPILER-MODULAR-CONSTRUCTION-001,
    // TOPAL-NUM-MODULAR-CONSTRUCT-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-modular-results");
    let source = directory.join("modular-checked-construction.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nByteRange is 0 ..= 255\nByteCounter is ModNat ByteRange\nconstruct is fn (value : Int) -> Result (ByteCounter, lang arithmetic ArithmeticErrorCode)\n  ByteCounter value\ninspect is fn (accepted : Result (ByteCounter, lang arithmetic ArithmeticErrorCode), rejected : Result (ByteCounter, lang arithmetic ArithmeticErrorCode)) -> Result (ByteCounter, lang arithmetic ArithmeticErrorCode)\n  rejected\n    Ok value then accepted\n    Error problem then accepted\ninspect (construct 255, construct 256)\n",
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
    assert_eq!(executed.stdout, b"ByteCounter 255\n");

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
            "break modular-checked-construction.t:7",
            "-ex",
            "run",
            "-ex",
            "whatis accepted",
            "-ex",
            "print accepted",
            "-ex",
            "print rejected",
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
    assert!(
        text.contains("type = Result (ByteCounter, lang arithmetic ArithmeticErrorCode)"),
        "{text}"
    );
    assert!(text.contains("$1 = ByteCounter 255"), "{text}");
    assert!(
        text.contains("$2 = Error ( domain is root.ByteCounter(Int), code is out-of-range"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One GDB session verifies both overload selections and both typed final bindings.
fn custom_generator_overloads_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-OVERLOAD-001, TOPAL-GENERATOR-FOREACH-RESULT-001,
    // TOPAL-COMPILER-GENERATOR-OVERLOAD-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-overloads");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-overloads.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(\"unary\", \"binary\")\n");
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
            "break custom-generator-overloads.t:20",
            "-ex",
            "break custom-generator-overloads.t:29",
            "-ex",
            "break topal.runtime.string.print",
            "-ex",
            "run",
            "-ex",
            "whatis 'binary-generated'",
            "-ex",
            "print 'binary-generated'",
            "-ex",
            "whatis 'unary-generated'",
            "-ex",
            "print 'unary-generated'",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
            "-ex",
            "whatis suffix",
            "-ex",
            "print suffix",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "up",
            "-ex",
            "whatis 'unary-result'",
            "-ex",
            "print 'unary-result'",
            "-ex",
            "whatis 'binary-result'",
            "-ex",
            "print 'binary-result'",
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
        "type = enum Generator String Unit String",
        "<Generator String Unit String>",
        "type = enum Generator Int Unit String",
        "<Generator Int Unit String>",
        "type = Int",
        "$3 = 7",
        "type = String",
        "$4 = \"item\"",
        "$5 = \"item\"",
        "$6 = \"unary\"",
        "$7 = \"binary\"",
        "custom-generator-overloads.t:20",
        "custom-generator-overloads.t:29",
        "custom-generator-overloads.t:30",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session verifies factory, transfer, traversal, result, and frames.
fn generic_custom_generator_function_boundaries_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-COMPILER-GENERATOR-FUNCTION-BOUNDARY-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-generic-function-boundaries");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-generic-function-boundaries.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"\"done\"\n");
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
            "break custom-generator-generic-function-boundaries.t:16",
            "-ex",
            "break custom-generator-generic-function-boundaries.t:20",
            "-ex",
            "break custom-generator-generic-function-boundaries.t:21",
            "-ex",
            "run",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis result",
            "-ex",
            "print result",
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
        "topal.fn.make.0 (initial=7)",
        "type = Int",
        "$1 = 7",
        "type = enum Generator Int Unit String",
        "$2 = <Generator Int Unit String>",
        "$3 = 7",
        "type = String",
        "$4 = \"done\"",
        "topal.fn.consume.1 (generated=<Generator Int Unit String>)",
        "custom-generator-generic-function-boundaries.t:16",
        "custom-generator-generic-function-boundaries.t:20",
        "custom-generator-generic-function-boundaries.t:21",
        "custom-generator-generic-function-boundaries.t:23",
        "custom-generator-generic-function-boundaries.t:24",
        "in topal.main",
        "in _start",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session verifies aggregate factory, transfer, traversal, result, and frames.
fn compound_custom_generator_function_boundaries_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-COMPOUND-FUNCTION-BOUNDARY-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-compound-function-boundaries");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-compound-function-boundaries.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(8, \"done\")\n");
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
            "break custom-generator-compound-function-boundaries.t:16",
            "-ex",
            "break custom-generator-compound-function-boundaries.t:20",
            "-ex",
            "break custom-generator-compound-function-boundaries.t:21",
            "-ex",
            "run",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis result",
            "-ex",
            "print result",
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
        "type = struct (Int, String)",
        "$1 = {_0 = 7, _1 = \"item\"}",
        "type = enum Generator (Int, String) Unit (Int, String)",
        "$2 = <Generator (Int, String) Unit (Int, String)>",
        "$3 = {_0 = 7, _1 = \"item\"}",
        "$4 = {_0 = 8, _1 = \"done\"}",
        "topal.fn.consume.1 (generated=<Generator (Int, String) Unit (Int, String)>)",
        "custom-generator-compound-function-boundaries.t:16",
        "custom-generator-compound-function-boundaries.t:20",
        "custom-generator-compound-function-boundaries.t:21",
        "custom-generator-compound-function-boundaries.t:23",
        "custom-generator-compound-function-boundaries.t:24",
        "in topal.main",
        "in _start",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session verifies recursive factory, transfer, traversal, result, and frames.
fn nested_custom_generator_function_boundaries_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-TYPE-RESULT-001,
    // TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-FUNCTION-BOUNDARY-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-nested-function-boundaries");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-nested-function-boundaries.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(8, \"done\")\n");
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
            "break custom-generator-nested-function-boundaries.t:17",
            "-ex",
            "break custom-generator-nested-function-boundaries.t:21",
            "-ex",
            "break custom-generator-nested-function-boundaries.t:22",
            "-ex",
            "run",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis result",
            "-ex",
            "print result",
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
        "topal.fn.make.0 (initial=Some (7, \"item\"))",
        "type = Optional(Int, String)",
        "$1 = Some (7, \"item\")",
        "type = enum Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)",
        "$2 = <Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)>",
        "$3 = Some (7, \"item\")",
        "type = Result ((Int, String), lang arithmetic ArithmeticErrorCode)",
        "$4 = (8, \"done\")",
        "custom-generator-nested-function-boundaries.t:17",
        "custom-generator-nested-function-boundaries.t:21",
        "custom-generator-nested-function-boundaries.t:22",
        "custom-generator-nested-function-boundaries.t:24",
        "custom-generator-nested-function-boundaries.t:25",
        "in topal.main",
        "in _start",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session verifies List transfer, traversal, append, result, and frames.
fn list_custom_generator_function_boundaries_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001,
    // TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-GENERATOR-FOREACH-RESULT-001,
    // TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-LIST-APPEND-001,
    // TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-COMPILER-GENERATOR-LIST-FUNCTION-BOUNDARY-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-list-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-list-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"Entry ( 7, Entry ( 9, Empty ) )\n");
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
            "break custom-generator-list-values.t:16",
            "-ex",
            "break custom-generator-list-values.t:20",
            "-ex",
            "break custom-generator-list-values.t:13",
            "-ex",
            "break custom-generator-list-values.t:21",
            "-ex",
            "run",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis values",
            "-ex",
            "print values",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis result",
            "-ex",
            "print result",
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
        "topal.fn.make.0 (initial=Entry ( 7, Empty ))",
        "type = List Int",
        "$1 = Entry ( 7, Empty )",
        "type = enum Generator List Int Unit List Int",
        "$2 = <Generator List Int Unit List Int>",
        "$3 = Entry ( 7, Empty )",
        "$4 = Entry ( 7, Empty )",
        "$5 = Entry ( 7, Entry ( 9, Empty ) )",
        "custom-generator-list-values.t:13",
        "custom-generator-list-values.t:16",
        "custom-generator-list-values.t:20",
        "custom-generator-list-values.t:21",
        "custom-generator-list-values.t:23",
        "custom-generator-list-values.t:24",
        "in topal.main",
        "in _start",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One boundary covers every ordered sequence result shape and its debugger view.
fn complete_list_sequence_operations_are_freestanding_and_debuggable() {
    // TOPAL-LIST-BOUNDARY-CHECK-001 through TOPAL-LIST-UNZIP-001,
    // TOPAL-COLLECTION-FOREACH-001, TOPAL-COLLECTION-ENTRIES-001,
    // TOPAL-COLLECTION-COLLECT-LIST-001, TOPAL-COLLECTION-COLLECT-STRING-001,
    // TOPAL-COMPILER-LIST-SEQUENCE-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-sequence-operations");
    let executable = directory.join("application");
    let shared = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/list-sequence-operations.t");
    let source_text = fs::read_to_string(&shared).unwrap();
    let expected = Session::new()
        .evaluate_source_file(&source_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let compiled = run(topalc().args([
        "-O0",
        "-g",
        "-o",
        executable.to_str().unwrap(),
        shared.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let debug_source = directory.join("list-sequence-debug.t");
    let debug_executable = directory.join("debug-application");
    fs::write(
        &debug_source,
        "use language (version is v0.1)\nvalues : List Int is Entry (1, Entry (2, Entry (3, Empty)))\nother : List Int is Entry (7, Entry (8, Empty))\nfragments : List String is Entry (\"Top\", Entry (\"al\", Empty))\npairs is values zip-shortest other\nindexed is values entries\ncombined is fragments collect String\n(values, pairs, indexed, fragments, combined)\n",
    )
    .unwrap();
    let compiled = run(topalc().args([
        "-O0",
        "-g",
        "-o",
        debug_executable.to_str().unwrap(),
        debug_source.to_str().unwrap(),
    ]));
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
            "break list-sequence-debug.t:8",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "whatis fragments",
            "-ex",
            "print fragments",
            "-ex",
            "whatis pairs",
            "-ex",
            "print pairs",
            "-ex",
            "whatis indexed",
            "-ex",
            "print indexed",
            "-ex",
            "whatis combined",
            "-ex",
            "print combined",
            "-ex",
            "backtrace",
        ])
        .arg(&debug_executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = List String",
        "$1 = Entry ( \"Top\", Entry ( \"al\", Empty ) )",
        "type = List(Int, Int)",
        "$2 = Entry ( (1, 7), Entry ( (2, 8), Empty ) )",
        "type = List (index : Int, value : Int)",
        "$3 = Entry ( (index is 0, value is 1), Entry ( (index is 1, value is 2), Entry ( (index is 2, value is 3), Empty ) ) )",
        "type = String",
        "$4 = \"Topal\"",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One boundary checks all four fundamental containers and debugger views.
fn fundamental_containers_are_freestanding_shared_and_debuggable() {
    // TOPAL-ARRAY-COLLECT-001, TOPAL-SET-COLLECT-001,
    // TOPAL-BAG-COLLECT-001, TOPAL-MAP-COLLECT-001,
    // TOPAL-COLLECTION-ENTRY-COUNT-001,
    // TOPAL-COLLECTION-EMPTY-PREDICATE-001,
    // TOPAL-ARRAY-GET-CHECKED-001, TOPAL-MAP-LOOKUP-001,
    // TOPAL-SET-CONTAINS-001, TOPAL-BAG-MULTIPLICITY-001,
    // TOPAL-COMPILER-FUNDAMENTAL-CONTAINERS-001,
    // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-fundamental-containers");
    let executable = directory.join("application");
    let shared = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/fundamental-containers.t");
    let source_text = fs::read_to_string(&shared).unwrap();
    let expected = Session::new()
        .evaluate_source_file(&source_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let compiled = run(topalc().args([
        "-O0",
        "-g",
        "-o",
        executable.to_str().unwrap(),
        shared.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let policies_source = directory.join("map-collision-policies.t");
    let policies_executable = directory.join("map-collision-policies");
    let policies_text = "use language (version is v0.1)\npairs : List (String, Int) is Entry ((\"Ada\", 1), Entry ((\"Ada\", 2), Empty))\nunique : List (String, Int) is Entry ((\"Ada\", 1), Entry ((\"Lin\", 2), Empty))\n(collect-map pairs resolving keep-first, collect-map pairs resolving keep-last, collect-map unique resolving reject)\n";
    fs::write(&policies_source, policies_text).unwrap();
    let policies_expected = Session::new()
        .evaluate_source_file(policies_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let compiled = run(topalc().args([
        "-O0",
        "-g",
        "-o",
        policies_executable.to_str().unwrap(),
        policies_source.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&policies_executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, policies_expected.as_bytes());

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
            "break fundamental-containers.t:8",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "whatis pairs",
            "-ex",
            "print pairs",
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
        "type = List(String, Int)",
        "$1 = Entry ( (\"Ada\", 10), Entry ( (\"Lin\", 8), Entry ( (\"Ada\", 11), Empty ) ) )",
        "type = Array 3 Int",
        "$2 = Array (2, 1, 2)",
        "type = Set Int",
        "$3 = Set (2, 1)",
        "type = Bag Int",
        "$4 = Bag ((2, 2), (1, 1))",
        "type = Map(String, Int)",
        "$5 = Map ((\"Ada\", 11), (\"Lin\", 8))",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}
