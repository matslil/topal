#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_result_values_are_freestanding_structured_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-RESULT-001, TOPAL-COMPILER-GENERATOR-RESULT-001,
    // TOPAL-COMPILER-ERROR-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-result-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-result-values.t");
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
        b"Error ( domain is root./(Rational,Rational), code is division-by-zero )\n"
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
            "break custom-generator-result-values.t:12",
            "-ex",
            "break custom-generator-result-values.t:17",
            "-ex",
            "break custom-generator-result-values.t:13",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Result (Rational, lang arithmetic ArithmeticErrorCode) Unit Result (Rational, lang arithmetic ArithmeticErrorCode)",
        "$1 = <Generator Result (Rational, lang arithmetic ArithmeticErrorCode) Unit Result (Rational, lang arithmetic ArithmeticErrorCode)>",
        "type = Result (Rational, lang arithmetic ArithmeticErrorCode)",
        "$2 = Rational ( 1, 1 )",
        "$3 = Rational ( 1, 1 )",
        "custom-generator-result-values.t:12",
        "custom-generator-result-values.t:17",
        "custom-generator-result-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_comparison_values_are_freestanding_nominal_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-DECISION-COMPARISON-001, TOPAL-COMPILER-GENERATOR-COMPARISON-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-comparison-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-comparison-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"Greater\n");
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
            "break custom-generator-comparison-values.t:12",
            "-ex",
            "break custom-generator-comparison-values.t:17",
            "-ex",
            "break custom-generator-comparison-values.t:13",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis comparison",
            "-ex",
            "print comparison",
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
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Comparison Unit Comparison",
        "$1 = <Generator Comparison Unit Comparison>",
        "type = enum Comparison",
        "$2 = Less",
        "$3 = Less",
        "custom-generator-comparison-values.t:12",
        "custom-generator-comparison-values.t:17",
        "custom-generator-comparison-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_rational_values_are_freestanding_exact_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-RATIONAL-001,
    // TOPAL-COMPILER-EXACT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-rational-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-rational-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"Rational ( 2, 3 )\n");
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
            "break custom-generator-rational-values.t:12",
            "-ex",
            "run",
            "-ex",
            "nexti",
            "-ex",
            "nexti",
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
            "next",
            "-ex",
            "next",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Rational Unit Rational",
        "$1 = <Generator Rational Unit Rational>",
        "type = Rational",
        "$2 = Rational ( 1, 3 )",
        "$3 = Rational ( 1, 3 )",
        "custom-generator-rational-values.t:12",
        "custom-generator-rational-values.t:17",
        "custom-generator-rational-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_unit_values_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-UNIT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-unit-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-unit-values.t");
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
            "break custom-generator-unit-values.t:12",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis signal",
            "-ex",
            "print signal",
            "-ex",
            "backtrace",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Unit Unit Unit",
        "$1 = <Generator Unit Unit Unit>",
        "type = Unit",
        "$2 = ()",
        "$3 = ()",
        "custom-generator-unit-values.t:12",
        "custom-generator-unit-values.t:17",
        "16\tgenerated foreach { signal }",
        "custom-generator-unit-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_optional_values_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-OPTIONAL-BOUNDARY-001, TOPAL-COMPILER-GENERATOR-OPTIONAL-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-optional-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-optional-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"None\n");
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
            "break custom-generator-optional-values.t:12",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
            "-ex",
            "backtrace",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Optional Int Unit Optional Int",
        "$1 = <Generator Optional Int Unit Optional Int>",
        "type = Optional Int",
        "$2 = Some 7",
        "$3 = Some 7",
        "custom-generator-optional-values.t:12",
        "custom-generator-optional-values.t:17",
        "16\tgenerated foreach { candidate }",
        "custom-generator-optional-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_nested_none_values_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-NONE-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-nested-none-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-nested-none-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"None\n");
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
            "break custom-generator-nested-none-values.t:12",
            "-ex",
            "break custom-generator-nested-none-values.t:17",
            "-ex",
            "break custom-generator-nested-none-values.t:13",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Optional (Int, String) Unit Optional (Int, String)",
        "$1 = <Generator Optional (Int, String) Unit Optional (Int, String)>",
        "type = Optional(Int, String)",
        "$2 = None",
        "$3 = None",
        "custom-generator-nested-none-values.t:12",
        "custom-generator-nested-none-values.t:17",
        "custom-generator-nested-none-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_nested_optional_values_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-OPTIONAL-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-nested-optional-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-nested-optional-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"Some (8, \"done\")\n");
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
            "break custom-generator-nested-optional-values.t:12",
            "-ex",
            "break custom-generator-nested-optional-values.t:17",
            "-ex",
            "break custom-generator-nested-optional-values.t:13",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Optional (Int, String) Unit Optional (Int, String)",
        "$1 = <Generator Optional (Int, String) Unit Optional (Int, String)>",
        "type = Optional(Int, String)",
        "$2 = Some (7, \"item\")",
        "$3 = Some (7, \"item\")",
        "custom-generator-nested-optional-values.t:12",
        "custom-generator-nested-optional-values.t:17",
        "custom-generator-nested-optional-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_nested_result_values_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-RESULT-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-RESULT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-nested-result-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-nested-result-values.t");
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
            "break custom-generator-nested-result-values.t:12",
            "-ex",
            "break custom-generator-nested-result-values.t:17",
            "-ex",
            "break custom-generator-nested-result-values.t:13",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Result ((Int, String), lang arithmetic ArithmeticErrorCode) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)",
        "$1 = <Generator Result ((Int, String), lang arithmetic ArithmeticErrorCode) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)>",
        "type = Result ((Int, String), lang arithmetic ArithmeticErrorCode)",
        "$2 = (7, \"item\")",
        "$3 = (7, \"item\")",
        "custom-generator-nested-result-values.t:12",
        "custom-generator-nested-result-values.t:17",
        "custom-generator-nested-result-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_recursive_nominal_values_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-ENUM-001, TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-TYPE-RESULT-001, TOPAL-COMPILER-GENERATOR-RECURSIVE-NOMINAL-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-recursive-nominal-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-recursive-nominal-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(Some Second, Second)\n");
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
            "break custom-generator-recursive-nominal-values.t:13",
            "-ex",
            "break custom-generator-recursive-nominal-values.t:18",
            "-ex",
            "break custom-generator-recursive-nominal-values.t:14",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis candidate",
            "-ex",
            "print candidate",
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
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)) Unit (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))",
        "$1 = <Generator (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)) Unit (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))>",
        "type = struct (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))",
        "$2 = {_0 = Some First, _1 = First}",
        "$3 = {_0 = Some First, _1 = First}",
        "custom-generator-recursive-nominal-values.t:13",
        "custom-generator-recursive-nominal-values.t:18",
        "custom-generator-recursive-nominal-values.t:14",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_range_values_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-RANGE-BOUNDS-001, TOPAL-RANGE-CLASSIFIER-001,
    // TOPAL-COMPILER-GENERATOR-RANGE-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-range-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-range-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"5 ..= 10\n");
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
            "break custom-generator-range-values.t:12",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis interval",
            "-ex",
            "print interval",
            "-ex",
            "backtrace",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Range Int Unit Range Int",
        "$1 = <Generator Range Int Unit Range Int>",
        "type = Range Int",
        "$2 = 0 ..= 10",
        "$3 = 0 ..= 10",
        "custom-generator-range-values.t:12",
        "custom-generator-range-values.t:17",
        "16\tgenerated foreach { interval }",
        "custom-generator-range-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn multiple_yield_custom_generator_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-MULTIPLE-YIELD-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-multiple-yield-generator");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-multiple-yield-generator.t");
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
            "whatis character",
            "-ex",
            "print character",
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
        "type = Character",
        "$2 = \"T\"",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_early_return_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-EARLY-RETURN-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-EARLY-RETURN-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-early-return");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-early-return.t");
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
        "type = enum Generator Character Unit Unit",
        "$1 = <Generator Character Unit Unit>",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_final_character_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FINAL-RETURN-001,
    // TOPAL-GENERATOR-SUSPEND-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-FINAL-CHARACTER-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-final-character");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-final-character.t");
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
            "break topal.platform.write_all",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "up",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis character",
            "-ex",
            "print character",
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
        "type = enum Generator Character Unit Character",
        "$1 = <Generator Character Unit Character>",
        "type = Character",
        "$2 = \"Y\"",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_local_binding_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-LOCAL-BINDING-001,
    // TOPAL-GENERATOR-SUSPEND-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-LOCAL-BINDING-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-local-binding");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-local-binding.t");
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
            "break custom-generator-local-binding.t:12",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "whatis copy",
            "-ex",
            "print copy",
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
    for expected in ["type = Character", "$1 = \"T\"", "topal.main"] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_suspension_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-BODY-STATEMENT-001,
    // TOPAL-GENERATOR-LOCAL-BINDING-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-SUSPENSION-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-suspension");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-suspension.t");
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
            "break custom-generator-suspension.t:13",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis copy",
            "-ex",
            "print copy",
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
        "type = Character",
        "$2 = \"T\"",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_resume_binding_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-RESUME-BINDING-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-RESUME-BINDING-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-resume-binding");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-resume-binding.t");
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
            "break custom-generator-resume-binding.t:12",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis resumed",
            "-ex",
            "print resumed",
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
        "type = Unit",
        "$2 = ()",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_close_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-CLOSE-001,
    // TOPAL-GENERATOR-ERROR-CODE-001, TOPAL-COMPILER-GENERATOR-CLOSE-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-close");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-close.t");
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
            "break custom-generator-close.t:17",
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
        "topal.fn.abandon",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_close_handler_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-CLOSE-001, TOPAL-GENERATOR-CLOSE-HANDLER-001,
    // TOPAL-GENERATOR-ERROR-CODE-001,
    // TOPAL-COMPILER-GENERATOR-CLOSE-HANDLER-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-close-handler");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-close-handler.t");
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
            "break custom-generator-close-handler.t:14",
            "-ex",
            "run",
            "-ex",
            "whatis 'resume-result'",
            "-ex",
            "print 'resume-result'",
            "-ex",
            "whatis problem",
            "-ex",
            "print problem",
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
        "type = Result (Unit, lang generator GeneratorErrorCode)",
        "type = Error (lang generator GeneratorErrorCode)",
        "Error ( domain is root, code is generator-closed )",
        "topal.fn.abandon",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_close_code_pattern_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001,
    // TOPAL-GENERATOR-CLOSE-HANDLER-001,
    // TOPAL-COMPILER-GENERATOR-CLOSE-CODE-PATTERN-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-close-code-pattern");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-close-code-pattern.t");
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
            "break custom-generator-close-code-pattern.t:14",
            "-ex",
            "run",
            "-ex",
            "whatis 'resume-result'",
            "-ex",
            "print 'resume-result'",
            "-ex",
            "info locals",
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
        "type = Result (Unit, lang generator GeneratorErrorCode)",
        "Error ( domain is root, code is generator-closed )",
        "topal.fn.abandon",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
    assert!(
        !text.contains("problem ="),
        "fallback binding became live: {text}"
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_closed_normalized_strings() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-UNICODE-FOLD-001
    let directory = temporary("gdb-normalized-string");
    let source = directory.join("string-normalization.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/string-normalization.t"),
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
            "break string-normalization.t:9",
            "-ex",
            "run",
            "-ex",
            "print preserved",
            "-ex",
            "print normalized",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = \"e\u{301}\""), "{text}");
    assert!(text.contains("$2 = \"é\""), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_values_projected_from_anonymous_records() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-RECORD-001
    let directory = temporary("gdb-record-projection");
    let source = directory.join("strings-and-products.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/strings-and-products.t"),
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
            "break strings-and-products.t:17",
            "-ex",
            "run",
            "-ex",
            "print 'person-name'",
            "-ex",
            "next",
            "-ex",
            "print greeting",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = \"Ada\""), "{text}");
    assert!(text.contains("$2 = \"Hello, Ada\""), "{text}");
}
