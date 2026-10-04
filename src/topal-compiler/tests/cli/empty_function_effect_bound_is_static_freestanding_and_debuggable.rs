#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn empty_function_effect_bound_is_static_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-EFFECT-BOUND-001, TOPAL-EFFECT-CONTAIN-001,
    // TOPAL-INTRO-STATIC-001, TOPAL-INTRO-VIEW-001,
    // TOPAL-COMPILER-FUNCTION-EMPTY-EFFECT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-empty-effect-bound");
    let source = directory.join("function-effect-bound.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-effect-bound.t"),
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

    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool)
            .arg("--debug-info")
            .arg(&executable));
        assert!(dwarf.status.success());
        let dwarf = String::from_utf8_lossy(&dwarf.stdout);
        assert!(!dwarf.contains("FunctionView"), "{dwarf}");
        assert!(!dwarf.contains("signature"), "{dwarf}");
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
            "break function-effect-bound.t:9",
            "-ex",
            "run",
            "-ex",
            "whatis value",
            "-ex",
            "print value",
            "-ex",
            "up",
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
    assert!(text.contains("type = Int"), "{text}");
    assert!(text.contains("$1 = 42"), "{text}");
    assert!(!text.contains("signature ="), "{text}");
    assert!(text.contains("topal.fn.identity.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn assert_static_introspection_absent_from_dwarf(tools: &LlvmTools, executable: &Path) {
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if !dwarf_tool.is_file() {
        return;
    }
    let dwarf = run(Command::new(dwarf_tool).arg("--debug-info").arg(executable));
    assert!(dwarf.status.success());
    let dwarf = String::from_utf8_lossy(&dwarf.stdout);
    for static_name in [
        "integer-identity",
        "integer-view",
        "current-context",
        "lang Identity",
        "lang TypeView",
        "lang LanguageContext",
    ] {
        assert!(!dwarf.contains(static_name), "{dwarf}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn static_introspection_is_erased_and_version_is_freestanding_and_debuggable() {
    // TOPAL-INTRO-QUALIFIED-001, TOPAL-INTRO-STATIC-001,
    // TOPAL-INTRO-VIEW-001, TOPAL-INTRO-CONTEXT-001,
    // TOPAL-INTRO-RELATION-001, TOPAL-COMPILER-STATIC-INTROSPECTION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-static-introspection");
    let source = directory.join("static-introspection.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/static-introspection.t"),
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
    assert_eq!(executed.stdout, b"(true, false, v0.1)\n");

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

    assert_static_introspection_absent_from_dwarf(&tools, &executable);

    let debug_source = directory.join("version-debug.t");
    let debug_executable = directory.join("debug-application");
    fs::write(
        &debug_source,
        "use language (\n  version is v0.1\n)\ncurrent-version is lang version\ncurrent-version\n",
    )
    .unwrap();
    let compiled = run(topalc().args([
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
            "break version-debug.t:5",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "whatis 'current-version'",
            "-ex",
            "print 'current-version'",
            "-ex",
            "ptype Version",
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
    assert!(text.contains("type = Version"), "{text}");
    assert!(text.contains("$1 = v0.1"), "{text}");
    assert!(text.contains("Nat major"), "{text}");
    assert!(text.contains("Nat minor"), "{text}");
    assert!(text.contains("Nat patch"), "{text}");
    assert!(text.contains("Nat build"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn lint_language_variant_is_freestanding_and_debuggable() {
    // TOPAL-SYN-CONTEXT-001, TOPAL-LINT-VARIANT-001,
    // TOPAL-COMPILER-LINT-VARIANT-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-lint-language-variant");
    let source = directory.join("lint-language-variant.t");
    let executable = directory.join("application");
    let source_text = include_str!("../../../../examples/language/lint-language-variant.t");
    fs::write(&source, source_text).unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let expected = Session::new()
        .evaluate_source_file(source_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
    assert_eq!(executed.stdout, b"<namespace lang lint>\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let debug_source = directory.join("lint-scope-debug.t");
    let debug_executable = directory.join("debug-application");
    fs::write(
        &debug_source,
        "use language (\n  version is v0.1,\n  features is ( lint )\n)\nlint-scope : Scope is lang lint\nlint-scope\n",
    )
    .unwrap();
    let compiled = run(topalc().args([
        "-o",
        debug_executable.to_str().unwrap(),
        debug_source.to_str().unwrap(),
    ]));
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
            "break lint-scope-debug.t:6",
            "-ex",
            "run",
            "-ex",
            "whatis 'lint-scope'",
            "-ex",
            "print 'lint-scope'",
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
    assert!(text.contains("type = enum Scope"), "{text}");
    assert!(text.contains("$1 = <namespace lang lint>"), "{text}");
    assert!(text.contains("lint-scope-debug.t:6"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn native_serialization_is_canonical_freestanding_and_debuggable() {
    // TOPAL-SER-HEADER-001 through TOPAL-SER-DESER-001,
    // TOPAL-COMPILER-NATIVE-SERIALIZATION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-native-serialization");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/native-serialization.t");
    let source_text = fs::read_to_string(&source).unwrap();
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
        source.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
    assert_eq!(executed.stdout, b"(answer is 42, accepted is true)\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let display_source = directory.join("native-stream-display.t");
    let display_executable = directory.join("display-application");
    let display_text =
        "use language (version is v0.1)\nv0.1 (lang serialize) (answer is 42, accepted is true)\n";
    fs::write(&display_source, display_text).unwrap();
    let display_expected = Session::new()
        .evaluate_source_file(display_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let compiled = run(topalc().args([
        "-O0",
        "-g",
        "-o",
        display_executable.to_str().unwrap(),
        display_source.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let displayed = run(&mut Command::new(&display_executable));
    assert!(displayed.status.success());
    assert_eq!(displayed.stdout, display_expected.as_bytes());
    assert_eq!(displayed.stdout, b"SerializationStream ( 76 bytes )\n");

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
            "break topal.runtime.serialization.verify",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis stream",
            "-ex",
            "print stream",
            "-ex",
            "ptype SerializationStream",
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
        "type = SerializationStream",
        "$1 = SerializationStream ( 76 bytes )",
        "struct TopalSerializationStreamHeader",
        "u8 *data",
        "u64 byte_count",
        "native-serialization.t:9",
        "topal.main",
        "in _start",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }

    assert_serialization_corruption_exits(&executable, "set {long}($rdi+8)=75");
    assert_serialization_corruption_exits(&executable, "set {unsigned char}*(long*)$rdi=0");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn direct_task_transactions_are_freestanding_and_debuggable() {
    // TOPAL-TASK-DEFINITION-001, TOPAL-TASK-LIFECYCLE-001,
    // TOPAL-TASK-STATE-001, TOPAL-TASK-MESSAGE-001,
    // TOPAL-COMPILER-TASK-DIRECT-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-direct-task");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/task-declaration-order.t");
    let source_text = fs::read_to_string(&source).unwrap();
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
        source.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
    assert_eq!(executed.stdout, b"3\n");
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
            "break topal.runtime.task.state.load",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "up",
            "-ex",
            "whatis 'ordered-counter'",
            "-ex",
            "print 'ordered-counter'",
            "-ex",
            "ptype OrderedCounter",
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
        "type = OrderedCounter",
        "$1 = OrderedCounter ( identity is 1, active, count is 3 )",
        "struct TopalTask.OrderedCounter",
        "u64 identity",
        "u64 terminated",
        "Nat count",
        "task-declaration-order.t:23",
        "topal.main",
        "in _start",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn task_stream_transaction_is_freestanding_affine_and_debuggable() {
    // TOPAL-TASK-HANDLER-001, TOPAL-TASK-MESSAGE-001,
    // TOPAL-COMPILER-TASK-STREAM-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-task-stream");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/task-message-transactions.t");
    let source_text = fs::read_to_string(&source).unwrap();
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
        source.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
    assert_eq!(executed.stdout, b"42\n");
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
            "break topal.runtime.task.state.load",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "up",
            "-ex",
            "whatis counter",
            "-ex",
            "print counter",
            "-ex",
            "whatis stream",
            "-ex",
            "print stream",
            "-ex",
            "ptype Counter",
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
        "type = Counter",
        "$1 = Counter ( identity is 1, active, count is 42 )",
        "type = enum Generator Nat Unit Result (Unit, ())",
        "$2 = <Generator Nat Unit Result (Unit, ())>",
        "struct TopalTask.Counter",
        "task-message-transactions.t:22",
        "topal.main",
        "in _start",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn external_layout_location_is_freestanding_ordered_and_debuggable() {
    // TOPAL-LAYOUT-CONSTRUCT-001, TOPAL-ADDRESS-RANGE-001,
    // TOPAL-LOCATION-CONSTRUCT-001, TOPAL-LOCATION-READ-001,
    // TOPAL-LOCATION-WRITE-001, TOPAL-COMPILER-EXTERNAL-LOCATION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-external-location");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/external-layout-location.t");
    let source_text = fs::read_to_string(&source).unwrap();
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
        source.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
    assert_eq!(executed.stdout, b"42\n");
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
            "break topal.runtime.location.read",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis control",
            "-ex",
            "print control",
            "-ex",
            "whatis stored",
            "-ex",
            "print stored",
            "-ex",
            "ptype ControlLocation",
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
        "type = ControlLocation",
        "$1 = ControlLocation ( range-start is 1073741824, offset is 32, value is 42 )",
        "type = UInt32LE",
        "$2 = 42",
        "struct TopalLocation.ControlLocation",
        "external-layout-location.t:48",
        "topal.main",
        "in _start",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn capability_composition_is_static_freestanding_and_absent_from_dwarf() {
    // TOPAL-CAPABILITY-EVIDENCE-001, TOPAL-CAPABILITY-COHERENCE-001,
    // TOPAL-CAPABILITY-COMPOSE-001, TOPAL-COMPILER-CAPABILITY-COMPOSE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("capability-composition");
    let source = directory.join("capability-composition.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/capability-composition.t"),
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
        b"Equality and Ordering or Foldable and Membership\n"
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
        let dwarf = run(Command::new(dwarf_tool)
            .arg("--debug-info")
            .arg(&executable));
        assert!(dwarf.status.success());
        let dwarf = String::from_utf8_lossy(&dwarf.stdout);
        for static_name in [
            "Comparable",
            "Searchable",
            "ComparableOrSearchable",
            "Capability",
        ] {
            assert!(!dwarf.contains(static_name), "{dwarf}");
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn function_interface_is_direct_freestanding_debuggable_and_statically_erased() {
    // TOPAL-INTERFACE-SHAPE-001, TOPAL-INTERFACE-IMPLEMENTATION-001,
    // TOPAL-COMPILER-FUNCTION-INTERFACE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("function-interface");
    let source = directory.join("function-interface.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-interface.t"),
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
    assert_eq!(executed.stdout, b"true\n");

    let metadata_bytes = fs::read(metadata_path(&executable)).unwrap();
    let metadata = NativeArtifactMetadata::decode(&metadata_bytes).unwrap();
    assert!(metadata.exports.is_empty());
    assert_eq!(
        metadata
            .evidence
            .iter()
            .map(|entry| entry.identity.as_str())
            .collect::<Vec<_>>(),
        ["topal.architecture.generic-x86_64-linux/1"]
    );
    assert!(!String::from_utf8_lossy(&metadata_bytes).contains("root.Parser"));

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
        let dwarf = run(Command::new(dwarf_tool)
            .arg("--debug-info")
            .arg(&executable));
        assert!(dwarf.status.success());
        let dwarf = String::from_utf8_lossy(&dwarf.stdout);
        assert!(dwarf.contains("topal.fn.parse.0"), "{dwarf}");
        assert!(!dwarf.contains("root.Parser"), "{dwarf}");
        assert!(!dwarf.contains("Parser"), "{dwarf}");
        assert!(!dwarf.contains("TopalInterface"), "{dwarf}");
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
            "break function-interface.t:10",
            "-ex",
            "run",
            "-ex",
            "whatis source",
            "-ex",
            "print source",
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
    assert!(text.contains("type = String"), "{text}");
    assert!(text.contains("$1 = \"ok\""), "{text}");
    assert!(text.contains("topal.fn.parse.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn function_input_boundary_is_private_direct_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-PARAMETER-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-input-boundary");
    let source = directory.join("function-value-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-value-boundary.t"),
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
            "break function-value-boundary.t:7",
            "-ex",
            "run",
            "-ex",
            "whatis operation",
            "-ex",
            "print operation",
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
    assert!(text.contains("topal.fn.apply_2dpair.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn function_result_boundary_is_private_direct_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-RESULT-001, TOPAL-FUNCTION-VALUE-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-result-boundary");
    let source = directory.join("function-results.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-results.t"),
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
    assert_eq!(executed.stdout, b"(42, 42)\n");
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
            "break function-results.t:11",
            "-ex",
            "break function-results.t:14",
            "-ex",
            "run",
            "-ex",
            "print operation",
            "-ex",
            "continue",
            "-ex",
            "whatis selected",
            "-ex",
            "print selected",
            "-ex",
            "continue",
            "-ex",
            "print operation",
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
    assert!(text.contains("$1 = <fn increment>"), "{text}");
    assert!(text.contains("type = enum Function"), "{text}");
    assert!(text.contains("$2 = <fn increment>"), "{text}");
    assert!(text.contains("$3 = +"), "{text}");
    assert!(text.contains("topal.fn.select.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn direct_anonymous_functions_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-DIRECT-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-SYN-GRAMMAR-001, TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-direct-anonymous-functions");
    let source = directory.join("anonymous-function-application.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/anonymous-function-application.t"),
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
    assert_eq!(executed.stdout, b"(42, 42)\n");

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
            "break anonymous-function-application.t:7",
            "-ex",
            "break anonymous-function-application.t:8",
            "-ex",
            "break anonymous-function-application.t:9",
            "-ex",
            "run",
            "-ex",
            "whatis increment",
            "-ex",
            "print increment",
            "-ex",
            "print combine",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "continue",
            "-ex",
            "print left",
            "-ex",
            "print right",
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
    assert!(text.contains("$1 = <anonymous fn/1>"), "{text}");
    assert!(text.contains("$2 = <anonymous fn/2>"), "{text}");
    assert!(text.contains("$3 = 41"), "{text}");
    assert!(text.contains("$4 = 4"), "{text}");
    assert!(text.contains("$5 = 2"), "{text}");
    assert!(text.contains("topal.fn.anonymous.0"), "{text}");
    assert!(text.contains("topal.fn.anonymous.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn anonymous_captures_and_results_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ANONYMOUS-CAPTURE-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-anonymous-function-captures");
    let source = directory.join("anonymous-function-captures.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/anonymous-function-captures.t"),
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
    assert_eq!(executed.stdout, b"(42, 42)\n");
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
            "break anonymous-function-captures.t:8",
            "-ex",
            "break anonymous-function-captures.t:12",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "print input",
            "-ex",
            "print offset",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "frame 1",
            "-ex",
            "whatis twice",
            "-ex",
            "print twice",
            "-ex",
            "frame 0",
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
    assert!(text.contains("$2 = 1"), "{text}");
    assert!(text.contains("type = enum Function"), "{text}");
    assert!(text.contains("$3 = <anonymous fn/1>"), "{text}");
    assert!(text.contains("$4 = 21"), "{text}");
    assert!(text.contains("topal.fn.apply_2doffset"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One GDB session verifies each forwarding and material-capture frame.
fn captured_function_parameters_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-CAPTURE-PARAMETER-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-NESTED-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-capturing-function-parameters");
    let source = directory.join("capturing-function-parameters.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/capturing-function-parameters.t"),
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
    assert_eq!(executed.stdout, b"(42, 42, (7, \"seven\"), 42)\n");
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
            "break capturing-function-parameters.t:11",
            "-ex",
            "break capturing-function-parameters.t:17",
            "-ex",
            "break capturing-function-parameters.t:21",
            "-ex",
            "break capturing-function-parameters.t:26",
            "-ex",
            "run",
            "-ex",
            "print operation",
            "-ex",
            "print value",
            "-ex",
            "info args",
            "-ex",
            "disable 1",
            "-ex",
            "continue",
            "-ex",
            "print input",
            "-ex",
            "print left",
            "-ex",
            "print right",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print pair",
            "-ex",
            "continue",
            "-ex",
            "print input",
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
    assert!(text.contains("$1 = <anonymous fn/1>"), "{text}");
    assert!(text.contains("$2 = 39"), "{text}");
    assert!(text.contains("operation = <anonymous fn/1>"), "{text}");
    assert!(text.contains("value = 39"), "{text}");
    assert!(text.contains("$3 = 39"), "{text}");
    assert!(text.contains("$4 = 1"), "{text}");
    assert!(text.contains("$5 = 2"), "{text}");
    assert!(text.contains("$6 = {_0 = 7, _1 = \"seven\"}"), "{text}");
    assert!(text.contains("$7 = 40"), "{text}");
    assert!(text.contains("$8 = 2"), "{text}");
    assert!(!text.contains("operation capture"), "{text}");
    assert!(text.contains("topal.fn.apply_2dint"), "{text}");
    assert!(text.contains("topal.fn.forward_2dint"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.fn.add"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One native session covers factories, forwarding, returned captures, and frames.
fn captured_function_results_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-VALUE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-capturing-function-results");
    let source = directory.join("capturing-function-results.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/capturing-function-results.t"),
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
    assert_eq!(executed.stdout, b"(42, (7, \"seven\"), 42, 42)\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let rejected = directory.join("function-capture.t");
    fs::write(
        &rejected,
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nmake is fn (operation : Function) -> Function\n  { value } operation value\nresult is make increment\nresult 41\n",
    )
    .unwrap();
    let output = run(topalc().args([
        "-o",
        directory.join("function-capture").to_str().unwrap(),
        rejected.to_str().unwrap(),
    ]));
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E-COMPILER-UNSUPPORTED"), "{stderr}");
    assert!(stderr.contains("Function result"), "{stderr}");

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
            "break capturing-function-results.t:10",
            "-ex",
            "break capturing-function-results.t:13",
            "-ex",
            "break capturing-function-results.t:7",
            "-ex",
            "break capturing-function-results.t:11",
            "-ex",
            "break capturing-function-results.t:14",
            "-ex",
            "break capturing-function-results.t:17",
            "-ex",
            "disable 4 5 6",
            "-ex",
            "run",
            "-ex",
            "print left",
            "-ex",
            "print right",
            "-ex",
            "disable 1",
            "-ex",
            "continue",
            "-ex",
            "print pair",
            "-ex",
            "disable 2",
            "-ex",
            "continue",
            "-ex",
            "print operation",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 3",
            "-ex",
            "enable 4 5 6",
            "-ex",
            "continue",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "print left",
            "-ex",
            "print right",
            "-ex",
            "backtrace",
            "-ex",
            "disable 4",
            "-ex",
            "continue",
            "-ex",
            "print pair",
            "-ex",
            "disable 5",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "print left",
            "-ex",
            "print right",
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
    assert!(text.contains("$1 = 1"), "{text}");
    assert!(text.contains("$2 = 2"), "{text}");
    assert!(text.contains("$3 = {_0 = 7, _1 = \"seven\"}"), "{text}");
    assert!(text.contains("$4 = <anonymous fn/1>"), "{text}");
    assert!(text.contains("operation = <anonymous fn/1>"), "{text}");
    assert!(text.contains("$5 = 39"), "{text}");
    assert!(text.contains("$6 = 1"), "{text}");
    assert!(text.contains("$7 = 2"), "{text}");
    assert!(text.contains("$8 = {_0 = 7, _1 = \"seven\"}"), "{text}");
    assert!(text.contains("$9 = 39"), "{text}");
    assert!(text.contains("$10 = 1"), "{text}");
    assert!(text.contains("$11 = 2"), "{text}");
    assert!(!text.contains("topal.function.result"), "{text}");
    assert!(!text.contains("operation capture"), "{text}");
    assert!(text.contains("topal.fn.make_2dscalars"), "{text}");
    assert!(text.contains("topal.fn.make_2dpair"), "{text}");
    assert!(text.contains("topal.fn.return_2doperation"), "{text}");
    assert!(text.contains("topal.fn.make_2dforwarded"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One native session covers chain semantics, rejection, and source frames.
fn function_result_chains_are_once_only_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-RESULT-CHAIN-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-FUNCTION-VALUE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-result-chains");
    let source = directory.join("function-result-chains.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/function-result-chains.t"),
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
    assert_eq!(executed.stdout, b"(42, 42, (7, \"seven\"), 42, 42, 42)\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let rejected = directory.join("non-function-intermediate.t");
    fs::write(
        &rejected,
        "use language (version is v0.1)\nmake is fn (offset : Int) -> Function\n  { value } value + offset\nmake 1 41 0\n",
    )
    .unwrap();
    let output = run(topalc().args([
        "-o",
        directory.join("rejected").to_str().unwrap(),
        rejected.to_str().unwrap(),
    ]));
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E-NO-APPLICABLE-OVERLOAD"), "{stderr}");
    assert!(
        stderr.contains("application chain produced `Int`"),
        "{stderr}"
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
            "break function-result-chains.t:14",
            "-ex",
            "break function-result-chains.t:11",
            "-ex",
            "break function-result-chains.t:8",
            "-ex",
            "disable 2 3",
            "-ex",
            "run",
            "-ex",
            "print offset",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "print offset",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
            "-ex",
            "print operation",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 2",
            "-ex",
            "enable 3",
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
    assert!(text.contains("$1 = 1"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("$3 = 1"), "{text}");
    assert!(text.contains("$4 = <fn increment>"), "{text}");
    assert!(text.contains("operation = <fn increment>"), "{text}");
    assert!(text.contains("$5 = 41"), "{text}");
    assert!(!text.contains("topal.function.chain"), "{text}");
    assert!(text.contains("topal.fn.make_2doffset"), "{text}");
    assert!(text.contains("topal.fn.return_2doperation"), "{text}");
    assert!(text.contains("topal.fn.increment"), "{text}");
    assert!(text.contains("topal.fn.anonymous"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}
