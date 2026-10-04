#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn anonymous_product_patterns_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-PRODUCT-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-TYPE-PRODUCT-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-anonymous-product-functions");
    let source = directory.join("anonymous-product-functions.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/anonymous-product-functions.t"),
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

    let boundary_source = directory.join("anonymous-product-parameter.t");
    let boundary_executable = directory.join("parameter-application");
    fs::write(
        &boundary_source,
        "use language (version is v0.1)\napply is fn (operation : Function, values : (Int, Int)) -> Int\n  operation values\napply ({ (left, right) } left + right, (20, 22))\n",
    )
    .unwrap();
    let boundary_compiled = run(topalc().args([
        "-o",
        boundary_executable.to_str().unwrap(),
        boundary_source.to_str().unwrap(),
    ]));
    assert!(
        boundary_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&boundary_compiled.stderr)
    );
    let boundary_executed = run(&mut Command::new(&boundary_executable));
    assert!(boundary_executed.status.success());
    assert_eq!(boundary_executed.stdout, b"42\n");

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
            "break anonymous-product-functions.t:11",
            "-ex",
            "break anonymous-product-functions.t:19",
            "-ex",
            "run",
            "-ex",
            "print left",
            "-ex",
            "print right",
            "-ex",
            "print offset",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print left",
            "-ex",
            "print right",
            "-ex",
            "print extra",
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
        "$1 = 20", "$2 = 21", "$3 = 1", "$4 = 10", "$5 = 20", "$6 = 12",
    ] {
        assert!(text.contains(expected), "{text}");
    }
    assert!(text.contains("topal.fn.apply_2doffset"), "{text}");
    assert!(text.matches("topal.fn.anonymous").count() >= 2, "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One native session covers recursive projection, identity failure, and source frames.
fn nested_anonymous_patterns_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-NESTED-PATTERN-001,
    // TOPAL-COMPILER-ANONYMOUS-PRODUCT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-nested-anonymous-patterns");
    let source = directory.join("nested-anonymous-patterns.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/nested-anonymous-patterns.t"),
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

    let mismatch_source = directory.join("nested-pattern-mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        "use language (version is v0.1)\noperation : Function is { (value, (value, _)) } value\noperation (1, (2, 0))\n",
    )
    .unwrap();
    let mismatch_compiled = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        mismatch_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&mismatch_compiled.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("E-ANONYMOUS-PATTERN-IDENTITY"));

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
            "break nested-anonymous-patterns.t:9",
            "-ex",
            "break nested-anonymous-patterns.t:12",
            "-ex",
            "break nested-anonymous-patterns.t:18",
            "-ex",
            "disable 2 3",
            "-ex",
            "run",
            "-ex",
            "print left",
            "-ex",
            "print middle",
            "-ex",
            "print right",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "print left",
            "-ex",
            "print right",
            "-ex",
            "print tail",
            "-ex",
            "print offset",
            "-ex",
            "backtrace",
            "-ex",
            "disable 2",
            "-ex",
            "enable 3",
            "-ex",
            "continue",
            "-ex",
            "print left",
            "-ex",
            "print middle",
            "-ex",
            "print right",
            "-ex",
            "print extra",
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
        "$1 = 13", "$2 = 14", "$3 = 15", "$4 = 10", "$5 = 11", "$6 = 20", "$7 = 1", "$8 = 10",
        "$9 = 11", "$10 = 20", "$11 = 1",
    ] {
        assert!(text.contains(expected), "{text}");
    }
    assert!(text.contains("topal.fn.captured"), "{text}");
    assert!(text.matches("topal.fn.anonymous").count() >= 3, "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn function_aggregates_are_private_direct_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-001,
    // TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-aggregate-boundaries");
    let source = directory.join("function-aggregate-boundaries.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-aggregate-boundaries.t"),
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
            "break function-aggregate-boundaries.t:21",
            "-ex",
            "break function-aggregate-boundaries.t:28",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "whatis package",
            "-ex",
            "print package",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "whatis package",
            "-ex",
            "print package",
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
        "type = struct (operation : Function, value : Int)",
        "$1 = {operation = -, value = -42}",
        "type = struct (payload : (Function, Int))",
        "$2 = {payload = {_0 = <anonymous fn/1>, _1 = 21}}",
        "topal.fn.apply_2drecord",
        "topal.fn.apply_2dnested",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers aggregate results, parameters, nesting, rejection, and frames.
fn captured_function_aggregates_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-001,
    // TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-capturing-function-aggregate-boundaries");
    let source = directory.join("capturing-function-aggregate-boundaries.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/capturing-function-aggregate-boundaries.t"),
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
    assert_eq!(executed.stdout, b"((42, 40), (42, 40), 42, 42)\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let rejected_source_path = directory.join("function-capture.t");
    let rejected_executable = directory.join("function-capture");
    fs::write(
        &rejected_source_path,
        "use language (version is v0.1)\nmake is fn (operation : Function) -> Record (wrapped : Function)\n  wrapped : Function is { value } operation value\n  (wrapped is wrapped)\noffset is 1\ncaptured : Function is { value } value + offset\nmake captured\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source_path.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    let diagnostic = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        diagnostic.contains("E-COMPILER-UNSUPPORTED"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("private representation"),
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
            "break capturing-function-aggregate-boundaries.t:13",
            "-ex",
            "break capturing-function-aggregate-boundaries.t:24",
            "-ex",
            "break capturing-function-aggregate-boundaries.t:27",
            "-ex",
            "break capturing-function-aggregate-boundaries.t:31",
            "-ex",
            "disable 2 3 4",
            "-ex",
            "run",
            "-ex",
            "whatis package",
            "-ex",
            "print package",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "whatis package",
            "-ex",
            "print package",
            "-ex",
            "backtrace",
            "-ex",
            "disable 2",
            "-ex",
            "enable 3",
            "-ex",
            "continue",
            "-ex",
            "whatis package",
            "-ex",
            "print package",
            "-ex",
            "backtrace",
            "-ex",
            "disable 3",
            "-ex",
            "enable 4",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "print offset",
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
        "type = struct (operation : Function, scale : Function, value : Int)",
        "$1 = {operation = <anonymous fn/1>, scale = <anonymous fn/1>, value = 20}",
        "type = struct (Function, Int)",
        "$2 = {_0 = <anonymous fn/1>, _1 = 40}",
        "type = struct (operation : Function, value : Int)",
        "$3 = {operation = <fn increase>, value = 40}",
        "$4 = 40",
        "$5 = 2",
        "topal.fn.apply_2drecord",
        "topal.fn.apply_2dtuple",
        "topal.fn.apply_2done",
        "topal.fn.increase",
        "topal.fn.use_2dnested",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
    assert!(!text.contains("operation capture"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn repeated_anonymous_patterns_are_exact_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-repeated-anonymous-patterns");
    let source = directory.join("repeated-anonymous-patterns.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/repeated-anonymous-patterns.t"),
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
    assert_eq!(executed.stdout, b"(42, 20, 7, \"same\", 42)\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let mismatch_source = directory.join("repeated-pattern-mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        "use language (version is v0.1)\noperation : Function is { (value, value) } value\noperation (1, 2)\n",
    )
    .unwrap();
    let mismatch_compiled = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        mismatch_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&mismatch_compiled.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(mismatch.stdout.is_empty());
    assert_eq!(
        mismatch.stderr,
        b"error[E-ANONYMOUS-PATTERN-IDENTITY]: repeated pattern values differ\n"
    );

    let classifier_source = directory.join("repeated-pattern-classifier.t");
    fs::write(
        &classifier_source,
        "use language (version is v0.1)\noperation : Function is { (value, value) } value\noperation (1, 1.0)\n",
    )
    .unwrap();
    let classifier = run(topalc().args([
        "-o",
        directory.join("classifier").to_str().unwrap(),
        classifier_source.to_str().unwrap(),
    ]));
    assert!(!classifier.status.success());
    assert!(
        String::from_utf8_lossy(&classifier.stderr)
            .contains("E-ANONYMOUS-PATTERN-IDENTITY-CLASSIFIER")
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
            "break repeated-anonymous-patterns.t:10",
            "-ex",
            "run",
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
    assert!(text.contains("$1 = 42"), "{text}");
    assert!(text.contains("value = 42"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn repeated_anonymous_aggregate_values_are_exact_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-repeated-anonymous-aggregate-patterns");
    let source = directory.join("repeated-anonymous-aggregate-patterns.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/repeated-anonymous-aggregate-patterns.t"),
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
        b"((7, \"seven\"), (active is true, name is \"Ada\"), Some 9, Entry ( 1, Entry ( 2, Empty ) ))\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let mismatch_source = directory.join("repeated-aggregate-mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        "use language (version is v0.1)\noperation : Function is { value, value } value\noperation ((1, \"same\"), (2, \"same\"))\n",
    )
    .unwrap();
    let mismatch_compiled = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        mismatch_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&mismatch_compiled.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(mismatch.stdout.is_empty());
    assert_eq!(
        mismatch.stderr,
        b"error[E-ANONYMOUS-PATTERN-IDENTITY]: repeated pattern values differ\n"
    );

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    // The anonymous header and body share one source line. Continue into the
    // first structural guard and return so its aggregate debug shadow is live.
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
            "break repeated-anonymous-aggregate-patterns.t:10",
            "-ex",
            "break topal.runtime.int.compare",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "finish",
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
    assert!(text.contains("$1 = {_0 = 7, _1 = \"seven\"}"), "{text}");
    assert!(text.contains("value = {_0 = 7, _1 = \"seven\"}"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers both mismatch paths, rejection, IR, artifacts, and GDB.
fn repeated_sum_values_compare_only_the_active_payload_and_remain_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-SUM-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-repeated-sum-patterns");
    let source = directory.join("repeated-sum-patterns.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/repeated-sum-patterns.t"),
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
        b"(Stop, Number 7, Label \"seven\", Pair (7, \"seven\"), Wrapped Code 9)\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let variant_source = directory.join("repeated-variant-pattern.t");
    let variant_executable = directory.join("variant");
    fs::write(
        &variant_source,
        "use language (version is v0.1)\nChoice is Variant (Int, String)\nrepeat : Function is { value, value } value\nleft : Choice is Choice at 1 \"same\"\nright : Choice is Choice at 1 \"same\"\nrepeat (left, right)\n",
    )
    .unwrap();
    let variant_compiled = run(topalc().args([
        "-o",
        variant_executable.to_str().unwrap(),
        variant_source.to_str().unwrap(),
    ]));
    assert!(
        variant_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&variant_compiled.stderr)
    );
    let variant = run(&mut Command::new(&variant_executable));
    assert!(variant.status.success());
    assert_eq!(variant.stdout, b"at 1 \"same\"\n");

    let ir = directory.join("application.ll");
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
    let ir = fs::read_to_string(ir).unwrap();
    assert!(ir.contains("sum.equal.tag"), "{ir}");
    assert!(ir.contains("sum.equal.alternative"), "{ir}");
    assert!(ir.contains("sum.equal.unequal"), "{ir}");
    assert!(ir.contains("sum.equal.merge"), "{ir}");
    assert!(ir.contains("switch i32"), "{ir}");
    assert!(ir.contains("phi i1"), "{ir}");
    assert!(!ir.contains("topal.runtime.sum.equal"), "{ir}");

    for (name, right) in [("payload", "Number 8"), ("tag", "Label \"seven\"")] {
        let mismatch_source = directory.join(format!("{name}-mismatch.t"));
        let mismatch_executable = directory.join(format!("{name}-mismatch"));
        fs::write(
            &mismatch_source,
            format!(
                "use language (version is v0.1)\nToken is Union\n  Number : Int\n  Label : String\n\nrepeat : Function is {{ value, value }} value\nleft : Token is Number 7\nright : Token is {right}\nrepeat (left, right)\n"
            ),
        )
        .unwrap();
        let mismatch_compiled = run(topalc().args([
            "-o",
            mismatch_executable.to_str().unwrap(),
            mismatch_source.to_str().unwrap(),
        ]));
        assert!(
            mismatch_compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&mismatch_compiled.stderr)
        );
        let mismatch = run(&mut Command::new(&mismatch_executable));
        assert_eq!(mismatch.status.code(), Some(65));
        assert!(mismatch.stdout.is_empty());
        assert_eq!(
            mismatch.stderr,
            b"error[E-ANONYMOUS-PATTERN-IDENTITY]: repeated pattern values differ\n"
        );
    }

    let unsupported_source = directory.join("unsupported-sum-payload.t");
    let unsupported_executable = directory.join("unsupported");
    fs::write(
        &unsupported_source,
        "use language (version is v0.1)\nWindow is Union\n  Bounded : Range Int\n\nrepeat : Function is { value, value } value\nwindow : Window is Bounded (0 ..= 1)\nrepeat (window, window)\n",
    )
    .unwrap();
    let unsupported = run(topalc().args([
        "-o",
        unsupported_executable.to_str().unwrap(),
        unsupported_source.to_str().unwrap(),
    ]));
    assert!(!unsupported.status.success());
    assert!(!unsupported_executable.exists());
    assert!(!metadata_path(&unsupported_executable).exists());
    let diagnostic = String::from_utf8_lossy(&unsupported.stderr);
    assert!(
        diagnostic.contains("E-COMPILER-UNSUPPORTED"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("repeated anonymous pattern identity for `Window`"),
        "{diagnostic}"
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
            "break repeated-sum-patterns.t:18",
            "-ex",
            "run",
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
    assert!(text.contains("type = Token"), "{text}");
    assert!(text.contains("$1 = Stop"), "{text}");
    assert!(text.contains("value = Stop"), "{text}");
    assert!(!text.contains("value repeated"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers evidence rejection, IR, artifacts, and GDB.
fn nominal_sum_equality_is_tag_first_freestanding_and_debuggable() {
    // TOPAL-COMPILER-SUM-EQUALITY-001, TOPAL-TYPE-SUM-EQUALITY-001,
    // TOPAL-TYPE-EQUALITY-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-sum-equality");
    let source = directory.join("sum-equality.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/sum-equality.t"),
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
        b"(true, false, true, false, true, true, true, false, true, false, true)\n"
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
    assert!(ir.matches("sum.equal.tag").count() >= 3, "{ir}");
    assert!(ir.contains("sum.equal.alternative"), "{ir}");
    assert!(ir.contains("sum.equal.unequal"), "{ir}");
    assert!(ir.contains("sum.equal.merge"), "{ir}");
    assert!(ir.contains("switch i32"), "{ir}");
    assert!(ir.contains("phi i1"), "{ir}");
    assert!(!ir.contains("topal.runtime.sum.equal"), "{ir}");
    assert!(!ir.contains("llvm.memcmp"), "{ir}");

    for (name, source_text, code) in [
        (
            "unsupported",
            "use language (version is v0.1)\nHolder is Union\n  Blank\n  Window : Range Int\n\nleft : Holder is Blank\nright : Holder is Blank\nleft = right\n",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "nominal",
            "use language (version is v0.1)\nLeft is Union\n  LeftEmpty\n\nRight is Union\n  RightEmpty\n\nleft : Left is LeftEmpty\nright : Right is RightEmpty\nleft = right\n",
            "E-TYPE-MISMATCH",
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
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
        assert!(
            String::from_utf8_lossy(&rejected.stderr).contains(code),
            "{}",
            String::from_utf8_lossy(&rejected.stderr)
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
            "break sum-equality.t:22",
            "-ex",
            "run",
            "-ex",
            "whatis left",
            "-ex",
            "print left",
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
    assert!(text.contains("type = Token"), "{text}");
    assert!(text.contains("$1 = Stop"), "{text}");
    assert!(text.contains("$2 = Stop"), "{text}");
    assert!(text.contains("left = Stop"), "{text}");
    assert!(text.contains("right = Stop"), "{text}");
    assert!(text.contains("topal.fn.same_2dtoken"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers success, mismatch, artifacts, and GDB.
fn repeated_function_aggregate_values_are_exact_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-001, TOPAL-TYPE-MATCH-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-repeated-function-aggregate-patterns");
    let source = directory.join("repeated-function-aggregate-patterns.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/repeated-function-aggregate-patterns.t"),
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

    let mismatch_source = directory.join("repeated-function-aggregate-mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        "use language (version is v0.1)\nleft is (operation is +, value is 1)\nright is (operation is -, value is 1)\nrepeat : Function is { value, value } 42\nrepeat (left, right)\n",
    )
    .unwrap();
    let mismatch_compiled = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        mismatch_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&mismatch_compiled.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(mismatch.stdout.is_empty());
    assert_eq!(
        mismatch.stderr,
        b"error[E-ANONYMOUS-PATTERN-IDENTITY]: repeated pattern values differ\n"
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
            "break repeated-function-aggregate-patterns.t:16",
            "-ex",
            "break topal.runtime.int.compare",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "finish",
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
    assert!(text.contains("type = struct (Function, Int)"), "{text}");
    assert!(
        text.contains("value = {_0 = <fn increment>, _1 = 41}"),
        "{text}"
    );
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers success, mismatch, rejection, artifacts, and GDB.
fn repeated_captured_function_values_are_exact_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-TYPE-MATCH-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-repeated-captured-function-patterns");
    let source = directory.join("repeated-captured-function-patterns.t");
    let executable = directory.join("application");
    let source_text =
        include_str!("../../../../examples/language/repeated-captured-function-patterns.t");
    fs::write(&source, source_text).unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(42, 42)\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let mismatch_source = directory.join("repeated-captured-function-mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        source_text.replace(
            "(compare-captures (1, 1), compare-captures (21, 21))",
            "compare-captures (1, 2)",
        ),
    )
    .unwrap();
    let mismatch_compiled = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        mismatch_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&mismatch_compiled.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(mismatch.stdout.is_empty());
    assert_eq!(
        mismatch.stderr,
        b"error[E-ANONYMOUS-PATTERN-IDENTITY]: repeated pattern values differ\n"
    );

    let unsupported_source = directory.join("unsupported-captured-function-pattern.t");
    let unsupported_executable = directory.join("unsupported");
    fs::write(
        &unsupported_source,
        "use language (version is v0.1)\nWindow is Union\n  Bounded : Range Int\n\nmake is fn (window : Window) -> Function\n  operation : Function is { value } window\n  operation\n\nrepeat : Function is { operation, operation } 42\nrepeat (make (Bounded (0 ..= 1)), make (Bounded (0 ..= 1)))\n",
    )
    .unwrap();
    let unsupported = run(topalc().args([
        "-o",
        unsupported_executable.to_str().unwrap(),
        unsupported_source.to_str().unwrap(),
    ]));
    assert!(!unsupported.status.success());
    assert!(!unsupported_executable.exists());
    let diagnostic = String::from_utf8_lossy(&unsupported.stderr);
    assert!(
        diagnostic.contains("E-COMPILER-UNSUPPORTED"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("without exact capture equality"),
        "{diagnostic}"
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
            "break repeated-captured-function-patterns.t:12",
            "-ex",
            "break topal.runtime.int.compare",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "finish",
            "-ex",
            "whatis operation",
            "-ex",
            "print operation",
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
    assert!(text.contains("operation = <anonymous fn/1>"), "{text}");
    assert!(!text.contains("operation repeated"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.fn.compare_2dcaptures"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers success, mismatch, rejection, artifacts, and GDB.
fn repeated_captured_named_function_values_are_exact_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
    // TOPAL-FUNCTION-NESTED-001, TOPAL-TYPE-MATCH-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-repeated-captured-named-function-patterns");
    let source = directory.join("repeated-captured-named-function-patterns.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/repeated-captured-named-function-patterns.t"),
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
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let differing_source = directory.join("differing-captured-named-functions.t");
    let differing_executable = directory.join("differing");
    fs::write(
        &differing_source,
        "use language (version is v0.1)\nToken is Union\n  Value : Int\n\ncompare is fn (token : Token) -> Int\n  left is fn (value : Int) -> Token\n    token\n  right is fn (value : Int) -> Token\n    token\n  repeat : Function is { operation, operation } 42\n  repeat (left, right)\ncompare (Value 1)\n",
    )
    .unwrap();
    let differing_compiled = run(topalc().args([
        "-o",
        differing_executable.to_str().unwrap(),
        differing_source.to_str().unwrap(),
    ]));
    assert!(
        differing_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&differing_compiled.stderr)
    );
    let differing = run(&mut Command::new(&differing_executable));
    assert_eq!(differing.status.code(), Some(65));
    assert_eq!(
        differing.stderr,
        b"error[E-ANONYMOUS-PATTERN-IDENTITY]: repeated pattern values differ\n"
    );

    let unsupported_source = directory.join("unsupported-captured-named-function.t");
    let unsupported_executable = directory.join("unsupported");
    fs::write(
        &unsupported_source,
        "use language (version is v0.1)\nWindow is Union\n  Bounded : Range Int\n\ncompare is fn (window : Window) -> Int\n  operation is fn (value : Int) -> Window\n    window\n  repeat : Function is { function, function } 42\n  repeat (operation, operation)\ncompare (Bounded (0 ..= 1))\n",
    )
    .unwrap();
    let unsupported = run(topalc().args([
        "-o",
        unsupported_executable.to_str().unwrap(),
        unsupported_source.to_str().unwrap(),
    ]));
    assert!(!unsupported.status.success());
    assert!(!unsupported_executable.exists());
    assert!(!metadata_path(&unsupported_executable).exists());
    let diagnostic = String::from_utf8_lossy(&unsupported.stderr);
    assert!(
        diagnostic.contains("E-COMPILER-UNSUPPORTED"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("without exact capture equality"),
        "{diagnostic}"
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
            "break repeated-captured-named-function-patterns.t:10",
            "-ex",
            "break topal.runtime.int.compare",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "finish",
            "-ex",
            "whatis operation",
            "-ex",
            "print operation",
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
    assert!(text.contains("operation = <fn increase>"), "{text}");
    assert!(!text.contains("operation repeated"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.fn.compare_2dnested"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers success, mismatch, rejection, artifacts, and GDB.
fn repeated_captured_function_aggregate_values_are_exact_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-repeated-captured-function-aggregate-patterns");
    let source = directory.join("repeated-captured-function-aggregate-patterns.t");
    let executable = directory.join("application");
    let source_text = include_str!(
        "../../../../examples/language/repeated-captured-function-aggregate-patterns.t"
    );
    fs::write(&source, source_text).unwrap();
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

    let mismatch_source = directory.join("repeated-captured-function-aggregate-mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        source_text.replace(
            "(compare-records (1, 1), compare-nested (21, 21), compare-named 1)",
            "compare-records (1, 2)",
        ),
    )
    .unwrap();
    let mismatch_compiled = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        mismatch_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&mismatch_compiled.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(mismatch.stdout.is_empty());
    assert_eq!(
        mismatch.stderr,
        b"error[E-ANONYMOUS-PATTERN-IDENTITY]: repeated pattern values differ\n"
    );

    let differing_source = directory.join("differing-captured-function-aggregate.t");
    let differing_executable = directory.join("differing");
    fs::write(
        &differing_source,
        "use language (version is v0.1)\nToken is Union\n  Value : Int\n\nmake-left is fn (token : Token) -> Record (operation : Function)\n  operation : Function is { value } token\n  (operation is operation)\nmake-right is fn (token : Token) -> Record (operation : Function)\n  operation : Function is { value } token\n  (operation is operation)\nrepeat : Function is { package, package } 42\nrepeat (make-left (Value 1), make-right (Value 1))\n",
    )
    .unwrap();
    let differing_compiled = run(topalc().args([
        "-o",
        differing_executable.to_str().unwrap(),
        differing_source.to_str().unwrap(),
    ]));
    assert!(
        differing_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&differing_compiled.stderr)
    );
    let differing = run(&mut Command::new(&differing_executable));
    assert_eq!(differing.status.code(), Some(65));
    assert_eq!(
        differing.stderr,
        b"error[E-ANONYMOUS-PATTERN-IDENTITY]: repeated pattern values differ\n"
    );

    let unsupported_source = directory.join("unsupported-captured-function-aggregate.t");
    let unsupported_executable = directory.join("unsupported");
    fs::write(
        &unsupported_source,
        "use language (version is v0.1)\nWindow is Union\n  Bounded : Range Int\n\nmake is fn (window : Window) -> Record (operation : Function)\n  operation : Function is { value } window\n  (operation is operation)\n\nrepeat : Function is { package, package } 42\nrepeat (make (Bounded (0 ..= 1)), make (Bounded (0 ..= 1)))\n",
    )
    .unwrap();
    let unsupported = run(topalc().args([
        "-o",
        unsupported_executable.to_str().unwrap(),
        unsupported_source.to_str().unwrap(),
    ]));
    assert!(!unsupported.status.success());
    assert!(!unsupported_executable.exists());
    assert!(!metadata_path(&unsupported_executable).exists());
    let diagnostic = String::from_utf8_lossy(&unsupported.stderr);
    assert!(
        diagnostic.contains("E-COMPILER-UNSUPPORTED"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("without exact capture equality"),
        "{diagnostic}"
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
            "break repeated-captured-function-aggregate-patterns.t:16",
            "-ex",
            "break topal.runtime.int.compare",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "finish",
            "-ex",
            "continue",
            "-ex",
            "finish",
            "-ex",
            "whatis package",
            "-ex",
            "print package",
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
    assert!(
        text.contains("type = struct (operation : Function, value : Int)"),
        "{text}"
    );
    assert!(
        text.contains("package = {operation = <anonymous fn/1>, value = 41}"),
        "{text}"
    );
    assert!(!text.contains("package repeated"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.fn.compare_2drecords"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn nested_functions_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-NESTED-FUNCTION-001, TOPAL-FUNCTION-NESTED-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-nested-functions");
    let source = directory.join("nested-functions.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/nested-functions.t"),
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
            "break nested-functions.t:9",
            "-ex",
            "run",
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
    assert!(text.contains("value = 2"), "{text}");
    assert!(text.contains("input = 40"), "{text}");
    assert!(text.contains("topal.fn.add_2dinput.0"), "{text}");
    assert!(text.contains("topal.fn.answer.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}
