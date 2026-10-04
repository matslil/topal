#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn returned_custom_generator_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-RESULT-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-RESULT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-function-result");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-function-result.t");
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
            "break custom-generator-function-result.t:16",
            "-ex",
            "run",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "finish",
            "-ex",
            "whatis $",
            "-ex",
            "print $",
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
        "type = Character",
        "$1 = \"T\"",
        "make",
        "Value returned is $2 = <Generator Character Unit Unit>",
        "type = enum Generator Character Unit Unit",
        "$3 = <Generator Character Unit Unit>",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn returned_custom_generator_character_result_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-character-result-result");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-character-return-result.t");
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
            "break custom-generator-character-return-result.t:16",
            "-ex",
            "run",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "finish",
            "-ex",
            "whatis $",
            "-ex",
            "print $",
            "-ex",
            "next",
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
        "type = Character",
        "$1 = \"Y\"",
        "make",
        "Value returned is $2 = <Generator Character Unit Character>",
        "type = enum Generator Character Unit Character",
        "$3 = <Generator Character Unit Character>",
        "$4 = \"Y\"",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn closed_string_character_foreach_is_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-character-foreach");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/string-character-foreach.t");
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
    for expected in ["type = Character", "$1 = \"🇸🇪\"", "topal.main"] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn named_string_character_generator_is_linear_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-CLASSIFIER-001,
    // TOPAL-STRING-CHARACTERS-LINEAR-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-named-string-character-generator");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/string-named-character-generator.t");
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
        "$2 = \"🇸🇪\"",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn single_yield_custom_generator_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-SINGLE-YIELD-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-single-yield-generator");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-single-yield-generator.t");
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
fn custom_generator_string_input_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-STRING-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-GENERATOR-STRING-INPUT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-string-input");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-string-input.t");
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
            "break topal.runtime.string.is.empty",
            "-ex",
            "break topal.platform.write_all",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
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
        "type = String",
        "$1 = \"Topal\"",
        "type = enum Generator Character Unit Unit",
        "$2 = <Generator Character Unit Unit>",
        "type = Character",
        "$3 = \"T\"",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_string_yields_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-STRING-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-GENERATOR-STRING-YIELD-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-string-yield");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-string-yield.t");
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
            "break topal.runtime.string.is.empty",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis text",
            "-ex",
            "print text",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "up",
            "-ex",
            "whatis text",
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
    for expected in [
        "type = enum Generator String Unit Unit",
        "$1 = <Generator String Unit Unit>",
        "type = String",
        "$2 = \"Topal\"",
        "$3 = \"\"",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_final_string_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-FINAL-STRING-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-string-return");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-string-return.t");
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
            "break topal.runtime.string.is.empty",
            "-ex",
            "break custom-generator-string-return.t:13",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis text",
            "-ex",
            "print text",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
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
        "type = enum Generator String Unit String",
        "$1 = <Generator String Unit String>",
        "type = String",
        "$2 = \"item\"",
        "$3 = <Generator String Unit String>",
        "custom-generator-string-return.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_resume_discard_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-BODY-STATEMENT-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-RESUME-DISCARD-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-discard-between-yields");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-discard-between-yields.t");
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
            "break topal.runtime.string.is.empty",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis text",
            "-ex",
            "print text",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "up",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "up",
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
    for expected in [
        "type = enum Generator String Unit Unit",
        "$1 = <Generator String Unit Unit>",
        "type = String",
        "$2 = \"Topal\"",
        "$3 = \"Topal\"",
        "$4 = \"\"",
        "custom-generator-discard-between-yields.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_explicit_return_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-EXPLICIT-RETURN-001, TOPAL-GENERATOR-FINAL-RETURN-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-EXPLICIT-RETURN-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-explicit-return");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-explicit-return.t");
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
            "break custom-generator-explicit-return.t:12",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
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
        "type = enum Generator String Unit String",
        "$1 = <Generator String Unit String>",
        "type = String",
        "$2 = \"unused\"",
        "custom-generator-explicit-return.t:12",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_return_after_yield_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-EXPLICIT-RETURN-001, TOPAL-GENERATOR-RESUMPTION-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-RETURN-AFTER-YIELD-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-return-after-yield");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-return-after-yield.t");
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
            "break topal.runtime.string.is.empty",
            "-ex",
            "break custom-generator-return-after-yield.t:13",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "whatis text",
            "-ex",
            "print text",
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
        "type = enum Generator String Unit String",
        "$1 = <Generator String Unit String>",
        "type = String",
        "$2 = \"item\"",
        "$3 = \"item\"",
        "custom-generator-return-after-yield.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_boolean_values_are_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-BOOLEAN-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-boolean-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-boolean-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"false\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break custom-generator-boolean-values.t:12",
            "-ex",
            "run",
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
            "nexti",
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
        "type = enum Generator Boolean Unit Boolean",
        "$1 = <Generator Boolean Unit Boolean>",
        "type = Boolean",
        "$2 = true",
        "$3 = true",
        "custom-generator-boolean-values.t:12",
        "custom-generator-boolean-values.t:16",
        "custom-generator-boolean-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_final_decision_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-GENERATOR-FINAL-DECISION-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-final-decision");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-final-decision.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"\"accepted\"\n");
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
            "break custom-generator-final-decision.t:12",
            "-ex",
            "run",
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
            "nexti",
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
        "type = enum Generator Boolean Unit String",
        "$1 = <Generator Boolean Unit String>",
        "type = Boolean",
        "$2 = true",
        "$3 = true",
        "custom-generator-final-decision.t:12",
        "custom-generator-final-decision.t:18",
        "custom-generator-final-decision.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_local_function_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-LOCAL-FUNCTION-001, TOPAL-GENERATOR-LOCAL-ENUM-001,
    // TOPAL-GENERATOR-SUSPEND-001, TOPAL-FUNCTION-ORDINARY-001,
    // TOPAL-COMPILER-GENERATOR-LOCAL-FUNCTION-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-local-function");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-local-function.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"\"accepted\"\n");
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
            "break custom-generator-local-function.t:17",
            "-ex",
            "break custom-generator-local-function.t:14",
            "-ex",
            "run",
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
            "nexti",
            "-ex",
            "whatis initial",
            "-ex",
            "print initial",
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
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "type = enum Generator Boolean Unit String",
        "$1 = <Generator Boolean Unit String>",
        "type = Boolean",
        "$2 = true",
        "$3 = true",
        "type = enum Choice",
        "$4 = Accepted",
        "custom-generator-local-function.t:17",
        "custom-generator-local-function.t:21",
        "custom-generator-local-function.t:18",
        "custom-generator-local-function.t:14",
        "topal.fn.label.0",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_local_close_handler_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-LOCAL-FUNCTION-001, TOPAL-GENERATOR-LOCAL-ENUM-001,
    // TOPAL-GENERATOR-CLOSE-001, TOPAL-GENERATOR-CLOSE-HANDLER-001,
    // TOPAL-COMPILER-GENERATOR-LOCAL-CLOSE-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-local-close-handler");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-local-close-handler.t");
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
            "break custom-generator-local-close-handler.t:17",
            "-ex",
            "break custom-generator-local-close-handler.t:14",
            "-ex",
            "run",
            "-ex",
            "whatis 'resume-result'",
            "-ex",
            "print 'resume-result'",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis choice",
            "-ex",
            "print choice",
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
        "type = Result (Unit, lang generator GeneratorErrorCode)",
        "Error ( domain is root, code is generator-closed )",
        "type = enum CloseChoice",
        "Closed",
        "custom-generator-local-close-handler.t:17",
        "custom-generator-local-close-handler.t:14",
        "topal.fn.cleanup.0",
        "topal.fn.abandon.1",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_int_values_are_freestanding_exact_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-INT-001,
    // TOPAL-COMPILER-INT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-int-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-int-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"1000000000000000000000000000000\n");
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
            "break custom-generator-int-values.t:12",
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
        "type = enum Generator Int Unit Int",
        "$1 = <Generator Int Unit Int>",
        "type = Int",
        "$2 = 999999999999999999999999999999",
        "$3 = 999999999999999999999999999999",
        "custom-generator-int-values.t:12",
        "custom-generator-int-values.t:17",
        "custom-generator-int-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_nat_values_are_freestanding_exact_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-NUM-NAT-CONSTRUCT-001, TOPAL-COMPILER-GENERATOR-NAT-001,
    // TOPAL-COMPILER-INT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-nat-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-nat-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"8\n");
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
            "break custom-generator-nat-values.t:12",
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
        "type = enum Generator Nat Unit Nat",
        "$1 = <Generator Nat Unit Nat>",
        "type = Nat",
        "$2 = 7",
        "$3 = 7",
        "custom-generator-nat-values.t:12",
        "custom-generator-nat-values.t:17",
        "16\tgenerated foreach { value }",
        "custom-generator-nat-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_enum_values_are_freestanding_nominal_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-ENUM-001, TOPAL-COMPILER-GENERATOR-ENUM-001,
    // TOPAL-COMPILER-ENUM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-enum-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-enum-values.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"Second\n");
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
            "break custom-generator-enum-values.t:13",
            "-ex",
            "break custom-generator-enum-values.t:18",
            "-ex",
            "break custom-generator-enum-values.t:14",
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
            "whatis choice",
            "-ex",
            "print choice",
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
        "type = enum Generator Choice Unit Choice",
        "$1 = <Generator Choice Unit Choice>",
        "type = enum Choice",
        "$2 = First",
        "$3 = First",
        "custom-generator-enum-values.t:13",
        "custom-generator-enum-values.t:18",
        "custom-generator-enum-values.t:14",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn custom_generator_product_values_are_freestanding_ordered_and_debuggable() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-PRODUCT-001, TOPAL-COMPILER-GENERATOR-PRODUCT-001,
    // TOPAL-COMPILER-TUPLE-RESULT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-product-values");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-product-values.t");
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
            "break custom-generator-product-values.t:12",
            "-ex",
            "break custom-generator-product-values.t:17",
            "-ex",
            "break custom-generator-product-values.t:13",
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
            "whatis value",
            "-ex",
            "print value",
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
        "type = enum Generator (Int, String) Unit (Int, String)",
        "$1 = <Generator (Int, String) Unit (Int, String)>",
        "type = struct (Int, String)",
        "$2 = {_0 = 7, _1 = \"item\"}",
        "$3 = {_0 = 7, _1 = \"item\"}",
        "custom-generator-product-values.t:12",
        "custom-generator-product-values.t:17",
        "custom-generator-product-values.t:13",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}
