#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_values_projected_from_reconstructed_records() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-RECONSTRUCT-001
    let directory = temporary("gdb-record-reconstruction");
    let source = directory.join("record-reconstruction-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nperson is (name is \"Ada\", age is 36)\nupdated is person with (age is person age + 1)\noriginal-age is person age + 0\nupdated-age is updated age\n(original-age, updated-age, original-age + updated-age)\n",
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
            "break topal.main",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "print 'original-age'",
            "-ex",
            "print 'updated-age'",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 36"), "{text}");
    assert!(text.contains("$2 = 37"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_structural_comparison_results() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-STRUCTURAL-COMPARISON-001
    let directory = temporary("gdb-structural-comparison");
    let source = directory.join("equality-and-ordering.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/equality-and-ordering.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break equality-and-ordering.t:9",
            "-ex",
            "run",
            "-ex",
            "print 'same-exact-value'",
            "-ex",
            "print 'different-text'",
            "-ex",
            "next",
            "-ex",
            "print 'same-record'",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = true"), "{text}");
    assert!(text.contains("$2 = true"), "{text}");
    assert!(text.contains("$3 = true"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_checked_nat_result_parameters_and_locals() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-RESULT-001
    let directory = temporary("gdb-nat-result");
    let source = directory.join("nat-result-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nas-nat is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)\n  Nat value\nretain is fn (value : Result (Nat, lang arithmetic ArithmeticErrorCode)) -> Result (Nat, lang arithmetic ArithmeticErrorCode)\n  result is value\n  result\n(as-nat -1) retain\n",
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
            "break nat-result-debug.t:6",
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
    let expected = "Error ( domain is root.Nat(Int), code is out-of-range )";
    assert!(text.contains(&format!("$1 = {expected}")), "{text}");
    assert!(text.contains(&format!("$2 = {expected}")), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_nat_comparison_parameters_with_nat_identity() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-NAT-COMPARISON-001
    let directory = temporary("gdb-nat-comparison");
    let source = directory.join("nat-equality-and-ordering.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/nat-equality-and-ordering.t"),
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
            "break nat-equality-and-ordering.t:7",
            "-ex",
            "run",
            "-ex",
            "print left",
            "-ex",
            "print right",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(
        text.contains("left=1, right=123456789012345678901234567890"),
        "{text}"
    );
    assert!(text.contains("$1 = 1"), "{text}");
    assert!(
        text.contains("$2 = 123456789012345678901234567890"),
        "{text}"
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_string_and_error_fields() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-RESULT-001
    let directory = temporary("gdb-result-decision");
    let source = directory.join("result-decision-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ninspect-error is fn (value : Error) -> ErrorDomain\n  result is value domain\n  result\nretain-string is fn (value : String) -> String\n  result is value\n  result\nretain-code is fn (value : ErrorCode) -> ErrorCode\n  result is value\n  result\nretain-domain is fn (value : ErrorDomain) -> ErrorDomain\n  result is value\n  result\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational, fallback : Result (Rational, lang arithmetic ArithmeticErrorCode)) -> ErrorDomain\n  1.0 divide denominator\n    Ok value then fallback domain\n    Error problem then problem inspect-error\nfailed is 1.0 divide 0.0\ncode is failed code\ndomain is failed domain\ndescription is \"error\"\nobserved is 0.0 describe failed\n(description retain-string, code retain-code, domain retain-domain, observed, failed)\n",
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
            "break result-decision-debug.t:3",
            "-ex",
            "break result-decision-debug.t:7",
            "-ex",
            "break result-decision-debug.t:10",
            "-ex",
            "break result-decision-debug.t:13",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "print result",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "print result",
            "-ex",
            "continue",
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
    assert!(text.contains("$2 = \"error\""), "{text}");
    assert!(text.contains("$3 = \"error\""), "{text}");
    assert!(text.contains("$4 = division-by-zero"), "{text}");
    assert!(text.contains("$5 = division-by-zero"), "{text}");
    assert!(text.contains("$6 = root./(Rational,Rational)"), "{text}");
    assert!(text.contains("$7 = root./(Rational,Rational)"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn optional_error_fields_are_freestanding_and_debuggable() {
    // TOPAL-COMP-ERROR-OPTIONAL-FIELDS-001,
    // TOPAL-COMPILER-ERROR-OPTIONAL-FIELDS-001, TOPAL-ERROR-FIELD-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-error-optional-fields");
    let source = directory.join("error-optional-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nretain-location is fn (candidate : Optional SourceLocation) -> Optional SourceLocation\n  candidate\n    Some value then Some value\n    None then None SourceLocation\nwrap-error is fn (candidate : Result (Rational, lang arithmetic ArithmeticErrorCode)) -> Optional Error\n  candidate\n    Ok value then None Error\n    Error problem then Some problem\ninspect is fn (failed : Result (Rational, lang arithmetic ArithmeticErrorCode)) -> (Optional String, Optional Error, Optional SourceLocation, Optional Error)\n  detail : Optional String is failed detail\n  cause : Optional Error is failed cause\n  location : Optional SourceLocation is retain-location (failed source)\n  retained : Optional Error is wrap-error failed\n  (detail, cause, location, retained)\ninspect (1.0 divide 0.0)\n",
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
        b"(None, None, Some (line is 3, column is 10), Some Error ( domain is root./(Rational,Rational), code is division-by-zero ))\n"
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
            "break error-optional-fields.t:17",
            "-ex",
            "run",
            "-ex",
            "print detail",
            "-ex",
            "print cause",
            "-ex",
            "print location",
            "-ex",
            "print retained",
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
    assert!(text.contains("$1 = None"), "{text}");
    assert!(text.contains("$2 = None"), "{text}");
    assert!(
        text.contains("$3 = Some (line is 3, column is 10)"),
        "{text}"
    );
    assert!(
        text.contains(
            "$4 = Some Error ( domain is root./(Rational,Rational), code is division-by-zero )"
        ),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_dynamically_concatenated_strings() {
    // TOPAL-COMP-STRING-CONSTRUCTION-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-concat");
    let source = directory.join("string-concat-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ncombine is fn (left : String, right : String) -> String\n  combined is left concat right\n  combined\ncombine (text__\"value \"text and \"text_ marker\"text__, \"!\")\n",
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
            "break string-concat-debug.t:4",
            "-ex",
            "run",
            "-ex",
            "print combined",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(
        text.contains("$1 = text__\"value \"text and \"text_ marker!\"text__"),
        "{text}"
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_distinguishes_overloads_and_static_functions() {
    // TOPAL-COMP-DEBUG-001, TOPAL-FUNCTION-OVERLOAD-001,
    // TOPAL-FUNCTION-STATIC-NULLARY-001, TOPAL-FUNCTION-STATIC-BINARY-001
    let directory = temporary("gdb-function-overloads");
    let source = directory.join("function-overloads-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ndescribe is fn (value : Int) -> Int\n  value\ndescribe is fn (value : String) -> String\n  value\nanswer is fn static () -> Int\n  42\nadd is fn static (left : Int, right : Int) -> Int\n  left + right\n(describe 42, describe \"Topal\", answer (), 20 add 22)\n",
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
            "break function-overloads-debug.t:3",
            "-ex",
            "break function-overloads-debug.t:5",
            "-ex",
            "break function-overloads-debug.t:7",
            "-ex",
            "break function-overloads-debug.t:9",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "continue",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print left",
            "-ex",
            "print right",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 42"), "{text}");
    assert!(text.contains("$2 = \"Topal\""), "{text}");
    assert!(text.contains("topal.fn.answer."), "{text}");
    assert!(text.contains("$3 = 20"), "{text}");
    assert!(text.contains("$4 = 22"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_forward_callee_and_caller_frames() {
    // TOPAL-COMP-DEBUG-001, TOPAL-FUNCTION-FORWARD-DECLARATION-001
    let directory = temporary("gdb-forward-function");
    let source = directory.join("forward-function-declarations.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/forward-function-declarations.t"),
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
            "break forward-function-declarations.t:10",
            "-ex",
            "run",
            "-ex",
            "print text",
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
    assert!(text.contains("$1 = \"Topal\""), "{text}");
    assert!(text.contains("topal.fn.decorate.0"), "{text}");
    assert!(text.contains("topal.fn.render.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_recursive_int_frames_and_parameters() {
    // TOPAL-COMP-DEBUG-001, TOPAL-FUNCTION-RECURSION-INT-001
    let directory = temporary("gdb-decreasing-int-recursion");
    let source = directory.join("decreasing-int-recursion.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/decreasing-int-recursion.t"),
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
            "break decreasing-int-recursion.t:10",
            "-ex",
            "run",
            "-ex",
            "print value",
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
    assert!(text.contains("$1 = 5"), "{text}");
    assert!(text.contains("$2 = 4"), "{text}");
    assert!(text.matches("topal.fn.sum_2ddown.0").count() >= 2, "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_increasing_recursive_int_frames_and_parameters() {
    // TOPAL-COMP-DEBUG-001, TOPAL-FUNCTION-RECURSION-INT-INCREASING-001
    let directory = temporary("gdb-increasing-int-recursion");
    let source = directory.join("increasing-int-recursion.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/increasing-int-recursion.t"),
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
            "break increasing-int-recursion.t:10",
            "-ex",
            "run",
            "-ex",
            "print value",
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
    assert!(text.contains("$1 = -5"), "{text}");
    assert!(text.contains("$2 = -4"), "{text}");
    assert!(
        text.matches("topal.fn.distance_2dup.0").count() >= 2,
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_recursive_nat_frames_and_constraint_identity() {
    // TOPAL-COMP-DEBUG-001, TOPAL-FUNCTION-RECURSION-NAT-001
    let directory = temporary("gdb-nat-recursion");
    let source = directory.join("nat-recursion.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/nat-recursion.t"),
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
            "break nat-recursion.t:10",
            "-ex",
            "run",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
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
    assert!(text.contains("type = Nat"), "{text}");
    assert!(text.contains("$1 = 8"), "{text}");
    assert!(text.contains("$2 = 5"), "{text}");
    assert!(
        text.matches("topal.fn.count_2ddown.0").count() >= 2,
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_explicit_measure_state_across_recursive_frames() {
    // TOPAL-COMP-DEBUG-001, TOPAL-FUNCTION-DECREASES-001
    let directory = temporary("gdb-explicit-multi-parameter-decreases");
    let source = directory.join("explicit-multi-parameter-decreases.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/explicit-multi-parameter-decreases.t"),
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
            "break explicit-multi-parameter-decreases.t:11",
            "-ex",
            "run",
            "-ex",
            "whatis count",
            "-ex",
            "print count",
            "-ex",
            "print total",
            "-ex",
            "continue",
            "-ex",
            "print count",
            "-ex",
            "print total",
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
    assert!(text.contains("type = Nat"), "{text}");
    assert!(text.contains("$1 = 4"), "{text}");
    assert!(text.contains("$2 = 0"), "{text}");
    assert!(text.contains("$3 = 3"), "{text}");
    assert!(text.contains("$4 = 3"), "{text}");
    assert!(
        text.matches("topal.fn.repeat_2dadd.0").count() >= 2,
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_distinguishes_same_named_cross_overload_frames() {
    // TOPAL-COMP-DEBUG-001, TOPAL-FUNCTION-RECURSION-OVERLOAD-IDENTITY-001
    let directory = temporary("gdb-overload-recursion-identity");
    let source = directory.join("overload-recursion-identity.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/overload-recursion-identity.t"),
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
            "break overload-recursion-identity.t:10",
            "-ex",
            "break overload-recursion-identity.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
            "-ex",
            "continue",
            "-ex",
            "backtrace",
            "-ex",
            "frame 1",
            "-ex",
            "whatis value",
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
    assert!(text.contains("type = String"), "{text}");
    assert!(text.contains("$1 = \"Topal\""), "{text}");
    assert!(text.contains("$2 = \"Topal\""), "{text}");
    assert!(text.contains("topal.fn.describe.0"), "{text}");
    assert!(text.contains("topal.fn.describe.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_distinct_mutual_recursion_frames() {
    // TOPAL-COMP-DEBUG-001, TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001
    let directory = temporary("gdb-mutual-int-recursion");
    let source = directory.join("mutual-int-recursion.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/mutual-int-recursion.t"),
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
            "break mutual-int-recursion.t:10",
            "-ex",
            "break mutual-int-recursion.t:14",
            "-ex",
            "run",
            "-ex",
            "print value",
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
    assert!(text.contains("$1 = 6"), "{text}");
    assert!(text.contains("$2 = 5"), "{text}");
    assert!(text.contains("topal.fn.even.0"), "{text}");
    assert!(text.contains("topal.fn.odd.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_mutual_nat_evidence_and_frames() {
    // TOPAL-COMP-DEBUG-001, TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001
    let directory = temporary("gdb-mutual-nat-recursion");
    let source = directory.join("nat-mutual-recursion.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/nat-mutual-recursion.t"),
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
            "break nat-mutual-recursion.t:9",
            "-ex",
            "break nat-mutual-recursion.t:13",
            "-ex",
            "run",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
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
    assert!(text.contains("type = Nat"), "{text}");
    assert!(text.contains("$1 = 8"), "{text}");
    assert!(text.contains("$2 = 5"), "{text}");
    assert!(text.contains("topal.fn.even.0"), "{text}");
    assert!(text.contains("topal.fn.odd.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_effect_identity_across_a_function_boundary() {
    // TOPAL-COMP-DEBUG-001, TOPAL-EFFECT-BOUNDARY-001
    let directory = temporary("gdb-effect-boundary");
    let source = directory.join("effect-function-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/effect-function-boundary.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break effect-function-boundary.t:7",
            "-ex",
            "run",
            "-ex",
            "whatis effects",
            "-ex",
            "print effects",
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
    assert!(text.contains("type = enum Effect"), "{text}");
    assert!(text.contains("$1 = empty"), "{text}");
    assert!(text.contains("topal.fn.identity_2deffect.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn effect_list_is_freestanding_and_gdb_renders_its_source_shape() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COMPILER-LIST-EFFECT-001,
    // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-effect-list");
    let source = directory.join("list-effect-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nretain is fn (rows : List Effect) -> List Effect\n  rows\nrows : List Effect is Entry (Effects (), Entry (Effects (), Empty))\nretain rows\n",
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
        b"Entry ( Effects (), Entry ( Effects (), Empty ) )\n"
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
            "break list-effect-boundary.t:3",
            "-ex",
            "run",
            "-ex",
            "whatis rows",
            "-ex",
            "print rows",
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
    assert!(text.contains("type = List Effect"), "{text}");
    assert!(
        text.contains("$1 = Entry ( Effects (), Entry ( Effects (), Empty ) )"),
        "{text}"
    );
    assert!(text.contains("topal.fn.retain"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, and source-level GDB values.
fn boolean_lists_are_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-BOOLEAN-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-boolean-values");
    let source = directory.join("list-boolean-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-boolean-values.t"),
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
        b"(true, false, true, true, 2, true, true, true, false, Entry ( true, Entry ( false, Empty ) ))\n"
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
        "%topal.ListBooleanStorage = type { i1, ptr }",
        "define internal i1 @topal.runtime.list.boolean.equal(ptr",
        "define internal ptr @topal.runtime.list.boolean.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i1 } @topal.fn.return_2dpair.",
        "store i1 false, ptr",
        "load i1, ptr",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.int.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Boolean is Entry (true, Empty)\nvalues reverse\n",
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
            "break list-boolean-values.t:8",
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
        "type = List Boolean",
        "candidate = Entry ( true, Entry ( false, Empty ) )",
        "fallback = false",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn string_lists_are_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-STRING-CORE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-string-values");
    let source = directory.join("list-string-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-string-values.t"),
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
        b"(\"Top\", \"al\", true, true, 2, true, \"package\", \"Top\", \"record\", Entry ( \"Top\", Entry ( \"al\", Empty ) ))\n"
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
        "%topal.ListStringStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.string.equal(ptr",
        "define internal ptr @topal.runtime.list.string.entry.count(ptr",
        "call i1 @topal.runtime.string.equal(ptr",
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
        "topal.runtime.list.int.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List String is Entry (\"Top\", Empty)\nvalues reverse\n",
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
            "break list-string-values.t:8",
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
        "type = List String",
        "candidate = Entry ( \"Top\", Entry ( \"al\", Empty ) )",
        "fallback = \"missing\"",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn character_lists_are_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-CHARACTER-CORE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-character-values");
    let source = directory.join("list-character-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-character-values.t"),
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
        "(\"A\u{30a}\", \"👩‍💻\", true, true, 2, true, \"K\", \"A\u{30a}\", \"R\", Entry ( \"A\u{30a}\", Entry ( \"👩‍💻\", Empty ) ))\n".as_bytes()
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
        "%topal.ListStringStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.string.equal(ptr",
        "define internal ptr @topal.runtime.list.string.entry.count(ptr",
        "call i1 @topal.runtime.string.equal(ptr",
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
        "topal.runtime.list.int.equal",
        "call ptr %",
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
            "break list-character-values.t:8",
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
        "type = List Character",
        "candidate = Entry ( \"A\u{30a}\", Entry ( \"👩‍💻\", Empty ) )",
        "fallback = \"?\"",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}
