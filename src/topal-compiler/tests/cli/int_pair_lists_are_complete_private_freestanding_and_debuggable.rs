#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn int_pair_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-TUPLE-EQUALITY-001, TOPAL-COMPILER-LIST-INT-PAIR-CORE-001
    let directory = temporary("gdb-list-int-pair-values");
    let source = directory.join("list-int-pair-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-int-pair-values.t"),
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
    assert_eq!(executed.stdout, b"((1, 2), (3, -4), true, true, true, 3, true, (7, 8), (1, 2), (11, 12), Entry ( (1, 2), Entry ( (3, -4), Entry ( (340282366920938463463374607431768211456, -170141183460469231731687303715884105728), Empty ) ) ))\n");
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
        "%topal.ListIntPairStorage = type { ptr, ptr, ptr }",
        "define internal i1 @topal.runtime.list.int.pair.equal(ptr",
        "define internal ptr @topal.runtime.list.int.pair.entry.count(ptr",
        "call i32 @topal.runtime.int.compare(ptr",
        "define internal fastcc { ptr, { ptr, ptr } } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.optional.string.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List (Int, Int) is Entry ((1, 2), Empty)\nvalues reverse\n").unwrap();
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
            "break list-int-pair-values.t:8",
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
        "type = List(Int, Int)",
        "candidate = Entry ( (1, 2), Entry ( (3, -4), Entry ( (340282366920938463463374607431768211456, -170141183460469231731687303715884105728), Empty ) ) )",
        "fallback = {_0 = 7, _1 = 8}",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn int_string_pair_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-TUPLE-EQUALITY-001,
    // TOPAL-COMPILER-LIST-INT-STRING-PAIR-CORE-001
    let directory = temporary("gdb-list-int-string-pair-values");
    let source = directory.join("list-int-string-pair-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-int-string-pair-values.t"),
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
    assert_eq!(executed.stdout, "((1, \"one\"), (-4, \"räv\"), true, true, true, true, 3, true, (7, \"seven\"), (1, \"one\"), (11, \"eleven\"), Entry ( (1, \"one\"), Entry ( (-4, \"räv\"), Entry ( (340282366920938463463374607431768211456, \"\"), Empty ) ) ))\n".as_bytes());
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
        "%topal.ListIntStringPairStorage = type { ptr, ptr, ptr }",
        "define internal i1 @topal.runtime.list.int-string.equal(ptr",
        "define internal ptr @topal.runtime.list.int-string.entry.count(ptr",
        "call i32 @topal.runtime.int.compare(ptr",
        "call i1 @topal.runtime.string.equal(ptr",
        "define internal fastcc { ptr, { ptr, ptr } } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.nested.int-string.equal",
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
            "break list-int-string-pair-values.t:8",
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
        "type = List(Int, String)",
        "candidate = Entry ( (1, \"one\"), Entry ( (-4, \"räv\"), Entry ( (340282366920938463463374607431768211456, \"\"), Empty ) ) )",
        "fallback = {_0 = 7, _1 = \"seven\"}",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn string_int_pair_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-TUPLE-EQUALITY-001,
    // TOPAL-COMPILER-LIST-STRING-INT-PAIR-CORE-001
    let directory = temporary("gdb-list-string-int-pair-values");
    let source = directory.join("list-string-int-pair-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-string-int-pair-values.t"),
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
    assert_eq!(executed.stdout, "((\"one\", 1), (\"räv\", -4), true, true, true, true, 3, true, (\"seven\", 7), (\"one\", 1), (\"eleven\", 11), Entry ( (\"one\", 1), Entry ( (\"räv\", -4), Entry ( (\"\", 340282366920938463463374607431768211456), Empty ) ) ))\n".as_bytes());
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
        "%topal.ListStringIntPairStorage = type { ptr, ptr, ptr }",
        "define internal i1 @topal.runtime.list.string-int.equal(ptr",
        "define internal ptr @topal.runtime.list.string-int.entry.count(ptr",
        "call i1 @topal.runtime.string.equal(ptr",
        "call i32 @topal.runtime.int.compare(ptr",
        "define internal fastcc { ptr, { ptr, ptr } } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.int-string.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List (String, Int) is Entry ((\"one\", 1), Empty)\nvalues reverse\n").unwrap();
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
            "break list-string-int-pair-values.t:8",
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
        "type = List(String, Int)",
        "candidate = Entry ( (\"one\", 1), Entry ( (\"räv\", -4), Entry ( (\"\", 340282366920938463463374607431768211456), Empty ) ) )",
        "fallback = {_0 = \"seven\", _1 = 7}",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn string_pair_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-TUPLE-EQUALITY-001,
    // TOPAL-COMPILER-LIST-STRING-PAIR-CORE-001
    let directory = temporary("gdb-list-string-pair-values");
    let source = directory.join("list-string-pair-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/list-string-pair-values.t"),
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
    assert_eq!(executed.stdout, "((\"one\", \"first\"), (\"räv\", \"andra\"), true, true, true, true, 3, true, (\"seven\", \"seventh\"), (\"one\", \"first\"), (\"eleven\", \"elfte\"), Entry ( (\"one\", \"first\"), Entry ( (\"räv\", \"andra\"), Entry ( (\"\", \"最後\"), Empty ) ) ))\n".as_bytes());
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
        "%topal.ListStringPairStorage = type { ptr, ptr, ptr }",
        "define internal i1 @topal.runtime.list.string-pair.equal(ptr",
        "define internal ptr @topal.runtime.list.string-pair.entry.count(ptr",
        "call i1 @topal.runtime.string.equal(ptr",
        "define internal fastcc { ptr, { ptr, ptr } } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.string-int.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List (String, String) is Entry ((\"one\", \"first\"), Empty)\nvalues reverse\n").unwrap();
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
            "break list-string-pair-values.t:8",
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
        "type = List(String, String)",
        "candidate = Entry ( (\"one\", \"first\"), Entry ( (\"räv\", \"andra\"), Entry ( (\"\", \"最後\"), Empty ) ) )",
        "fallback = {_0 = \"seven\", _1 = \"seventh\"}",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn int_list_containment_is_freestanding_and_gdb_renders_exact_entries() {
    // TOPAL-LIST-CONTAINS-ENTRY-001, TOPAL-LIST-CONTAINS-SEQUENCE-001,
    // TOPAL-LIST-CONTAINS-SUBSEQUENCE-001,
    // TOPAL-COMPILER-LIST-INT-CONTAINMENT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-int-list");
    let source = directory.join("int-list-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nretain is fn (values : List Int) -> List Int\n  values\nvalues : List Int is Entry (-170141183460469231731687303715884105728, Entry (340282366920938463463374607431768211456, Empty))\nempty-values : List Int is Empty\nkept is retain values\n(kept, empty-values contains-sequence empty-values, values contains-sequence empty-values, empty-values contains-subsequence empty-values, values contains-subsequence empty-values, empty-values contains-entry 0)\n",
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
        b"(Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) ), true, true, true, true, false)\n"
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
            "break int-list-boundary.t:3",
            "-ex",
            "run",
            "-ex",
            "whatis values",
            "-ex",
            "print values",
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
        text.contains(
            "$1 = Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) )"
        ),
        "{text}"
    );
    assert!(text.contains("topal.fn.retain"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn int_list_removal_is_immutable_freestanding_and_debuggable() {
    // TOPAL-LIST-REMOVE-FIRST-001, TOPAL-LIST-REMOVE-ALL-001,
    // TOPAL-COMPILER-LIST-INT-REMOVAL-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-int-list-removal");
    let source = directory.join("int-list-removal-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nerase-first is fn (values : List Int, target : Int) -> List Int\n  values remove-first target\nerase-all is fn (values : List Int, target : Int) -> List Int\n  values remove-all target\nvalues : List Int is Entry (-170141183460469231731687303715884105728, Entry (5, Entry (-170141183460469231731687303715884105728, Entry (340282366920938463463374607431768211456, Empty))))\nall-targets : List Int is Entry (5, Entry (5, Empty))\nempty-values : List Int is Empty\nwithout-first is erase-first (values, -170141183460469231731687303715884105728)\nwithout-all is erase-all (values, -170141183460469231731687303715884105728)\nmissing-all is erase-all (values, 9)\nmissing-first is erase-first (values, 9)\nnone-left is erase-all (all-targets, 5)\nempty-first is erase-first (empty-values, 5)\nempty-all is erase-all (empty-values, 5)\n(values, without-first, without-all, missing-all, missing-first, none-left, empty-first, empty-all)\n",
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
        b"(Entry ( -170141183460469231731687303715884105728, Entry ( 5, Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) ) ) ), Entry ( 5, Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) ) ), Entry ( 5, Entry ( 340282366920938463463374607431768211456, Empty ) ), Entry ( -170141183460469231731687303715884105728, Entry ( 5, Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) ) ) ), Entry ( -170141183460469231731687303715884105728, Entry ( 5, Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) ) ) ), Empty, Empty, Empty)\n"
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
            "break int-list-removal-boundary.t:3",
            "-ex",
            "run",
            "-ex",
            "whatis values",
            "-ex",
            "print values",
            "-ex",
            "print target",
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
        text.contains(
            "$1 = Entry ( -170141183460469231731687303715884105728, Entry ( 5, Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) ) ) )"
        ),
        "{text}"
    );
    assert!(
        text.contains("$2 = -170141183460469231731687303715884105728"),
        "{text}"
    );
    assert!(text.contains("topal.fn.erase_2dfirst.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn bound_anonymous_int_list_functions_are_specialized_and_debuggable() {
    // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
    // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-LIST-INT-BOUND-FUNCTIONS-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-bound-int-list-functions");
    let source = directory.join("bound-int-list-functions.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ninspect is fn (values : List Int) -> Int\n  entry-count values\nvalues : List Int is Entry (1, Entry (2, Entry (3, Empty)))\nfactor is 2\ntwice is { value } value * factor\npositive is { value } value > 0\nsum is { state, value } state + value\nmapped is values map twice\nselected is values select positive\nfolded is values fold 0 sum\nresult is inspect mapped\n(mapped, selected, folded)\n",
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
        b"(Entry ( 2, Entry ( 4, Entry ( 6, Empty ) ) ), Entry ( 1, Entry ( 2, Entry ( 3, Empty ) ) ), 6)\n"
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
            "break bound-int-list-functions.t:3",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis twice",
            "-ex",
            "print twice",
            "-ex",
            "whatis positive",
            "-ex",
            "print positive",
            "-ex",
            "whatis sum",
            "-ex",
            "print sum",
            "-ex",
            "print mapped",
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
    assert_eq!(text.matches("type = enum Function").count(), 3, "{text}");
    assert_eq!(text.matches("<anonymous fn/1>").count(), 2, "{text}");
    assert!(text.contains("<anonymous fn/2>"), "{text}");
    assert!(
        text.contains("$4 = Entry ( 2, Entry ( 4, Entry ( 6, Empty ) ) )"),
        "{text}"
    );
    assert!(text.contains("topal.fn.inspect.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn range_selection_is_freestanding_and_debuggable() {
    // TOPAL-RANGE-VALUE-SELECTION-001, TOPAL-RANGE-INDEX-SELECTION-001,
    // TOPAL-COMPILER-RANGE-SELECTION-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-range-selection");
    let source = directory.join("range-selection-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ninspect is fn (values : List Int, text : String) -> Int\n  _ is text = text\n  entry-count values\nselect-character is fn (text : String, index : Int) -> Optional Character\n  first (collect (characters (text select-index (index ..= index))))\nvalues : List Int is Entry (9, Entry (2, Entry (4, Entry (7, Entry (3, Entry (340282366920938463463374607431768211456, Empty))))))\nbounds : Range Int is 2 ..= 4\nindexes : Range Int is 1 .. 4\nchosen is values select bounds\npositions is values select-index indexes\nslice is \"Topal\" select-index indexes\ndynamic-character is select-character (\"á👩‍🔬🇸🇪\", 1)\nexact is values select (340282366920938463463374607431768211456 ..= 340282366920938463463374607431768211456)\nnone is values select (8 .. 2)\nno-positions is values select-index (4 <..= 4)\nresult is inspect (chosen, slice)\n(chosen, positions, slice, dynamic-character, result, exact, none, no-positions)\n",
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
        b"(Entry ( 2, Entry ( 4, Entry ( 3, Empty ) ) ), Entry ( 2, Entry ( 4, Entry ( 7, Empty ) ) ), \"opa\", Some \"\xf0\x9f\x91\xa9\xe2\x80\x8d\xf0\x9f\x94\xac\", 3, Entry ( 340282366920938463463374607431768211456, Empty ), Empty, Empty)\n"
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
            "break range-selection-boundary.t:3",
            "-ex",
            "run",
            "-ex",
            "whatis values",
            "-ex",
            "print values",
            "-ex",
            "whatis text",
            "-ex",
            "print text",
            "-ex",
            "finish",
            "-ex",
            "whatis bounds",
            "-ex",
            "print bounds",
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
    assert!(text.contains("type = String"), "{text}");
    assert!(text.contains("type = Range Int"), "{text}");
    assert!(
        text.contains("$1 = Entry ( 2, Entry ( 4, Entry ( 3, Empty ) ) )"),
        "{text}"
    );
    assert!(text.contains("$2 = \"opa\""), "{text}");
    assert!(text.contains("$4 = 2 ..= 4"), "{text}");
    assert!(text.contains("topal.fn.inspect."), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn traversal_control_fold_is_freestanding_short_circuiting_and_debuggable() {
    // TOPAL-EXEC-TRAVERSAL-CONTROL-001,
    // TOPAL-COMPILER-TRAVERSAL-CONTROL-001,
    // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-traversal-control");
    let source = directory.join("traversal-control-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ninspect is fn (value : Int) -> Int\n  value\nvalues : List Int is Entry (1, Entry (2, Entry (100, Empty)))\ncontrols is (Continue 1, Finish 2)\nadvance is { state, value } Continue (state + value)\nstop is { state, value } Finish (state + value)\ncontinued is values fold 0 advance\nstopped is values fold 0 stop\nresult is inspect stopped\n(continued, stopped, controls, result)\n",
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
    assert_eq!(executed.stdout, b"(103, 1, (Continue 1, Finish 2), 1)\n");

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
            "break traversal-control-boundary.t:3",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "up",
            "-ex",
            "whatis controls._0",
            "-ex",
            "print controls._0",
            "-ex",
            "whatis controls._1",
            "-ex",
            "print controls._1",
            "-ex",
            "whatis stopped",
            "-ex",
            "print stopped",
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
    assert_eq!(
        text.matches("type = TraversalControl Int").count(),
        2,
        "{text}"
    );
    assert!(text.contains("$2 = Continue 1"), "{text}");
    assert!(text.contains("$3 = Finish 2"), "{text}");
    assert!(text.contains("type = Int"), "{text}");
    assert!(text.contains("$4 = 1"), "{text}");
    assert!(text.contains("topal.fn.inspect.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn contextual_int_list_functions_are_freestanding_and_debuggable() {
    // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
    // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-LIST-INT-FUNCTIONS-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-int-list-functions");
    let source = directory.join("int-list-functions.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ninspect is fn (values : List Int) -> Int\n  entry-count values\nfactor is 3\nvalues : List Int is Entry (-170141183460469231731687303715884105728, Entry (0, Entry (340282366920938463463374607431768211456, Empty)))\nempty-values is empty List Int\nmapped is values map { value } value * factor\nselected is values select { value } value >= 0\nfolded is values fold 10 { sum, value } sum + value\nempty-mapped is empty-values map { value } value * factor\nempty-selected is empty-values select { value } value >= 0\nempty-folded is empty-values fold 10 { sum, value } sum + value\nresult is inspect mapped\n(mapped, selected, folded, empty-mapped, empty-selected, empty-folded, values)\n",
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
        b"(Entry ( -510423550381407695195061911147652317184, Entry ( 0, Entry ( 1020847100762815390390123822295304634368, Empty ) ) ), Entry ( 0, Entry ( 340282366920938463463374607431768211456, Empty ) ), 170141183460469231731687303715884105738, Empty, Empty, 10, Entry ( -170141183460469231731687303715884105728, Entry ( 0, Entry ( 340282366920938463463374607431768211456, Empty ) ) ))\n"
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
            "break int-list-functions.t:3",
            "-ex",
            "run",
            "-ex",
            "print values",
            "-ex",
            "up",
            "-ex",
            "whatis mapped",
            "-ex",
            "print mapped",
            "-ex",
            "whatis selected",
            "-ex",
            "print selected",
            "-ex",
            "whatis folded",
            "-ex",
            "print folded",
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
        text.contains(
            "$2 = Entry ( -510423550381407695195061911147652317184, Entry ( 0, Entry ( 1020847100762815390390123822295304634368, Empty ) ) )"
        ),
        "{text}"
    );
    assert!(
        text.contains("$3 = Entry ( 0, Entry ( 340282366920938463463374607431768211456, Empty ) )"),
        "{text}"
    );
    assert!(
        text.contains("$4 = 170141183460469231731687303715884105738"),
        "{text}"
    );
    assert!(text.contains("topal.fn.inspect.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn int_pair_list_product_map_is_freestanding_and_debuggable() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COLLECTION-MAP-001,
    // TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-LIST-INT-PAIR-MAP-001,
    // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-int-pair-list-map");
    let source = directory.join("int-pair-list-map.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ninspect is fn (values : List Int) -> Int\n  entry-count values\npairs : List (Int, Int) is Entry ((2, 3), Entry ((5, 7), Empty))\nmapped is pairs map { (left, right) } left + right\nresult is inspect mapped\n(pairs, mapped, result)\n",
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
        b"(Entry ( (2, 3), Entry ( (5, 7), Empty ) ), Entry ( 5, Entry ( 12, Empty ) ), 2)\n"
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
            "break int-pair-list-map.t:3",
            "-ex",
            "run",
            "-ex",
            "print values",
            "-ex",
            "up",
            "-ex",
            "whatis pairs",
            "-ex",
            "print pairs",
            "-ex",
            "whatis mapped",
            "-ex",
            "print mapped",
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
    assert!(text.contains("type = List(Int, Int)"), "{text}");
    assert!(
        text.contains("$2 = Entry ( (2, 3), Entry ( (5, 7), Empty ) )"),
        "{text}"
    );
    assert!(text.contains("type = List Int"), "{text}");
    assert!(
        text.contains("$3 = Entry ( 5, Entry ( 12, Empty ) )"),
        "{text}"
    );
    assert!(text.contains("topal.fn.inspect.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn recursive_int_string_lists_are_freestanding_and_debuggable() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-TYPE-LIST-RECURSIVE-001, TOPAL-LIST-FIRST-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-COMPILER-LIST-RECURSIVE-001,
    // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-recursive-int-string-list");
    let source = directory.join("nested-lists.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/nested-lists.t"),
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
        b"(Some Entry ( (7, \"seven\"), Empty ), 1, true)\n"
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
            "break nested-lists.t:8",
            "-ex",
            "run",
            "-ex",
            "print values",
            "-ex",
            "up",
            "-ex",
            "whatis pairs",
            "-ex",
            "print pairs",
            "-ex",
            "whatis nested",
            "-ex",
            "print nested",
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
        text.contains("$1 = Entry ( Entry ( (7, \"seven\"), Empty ), Empty )"),
        "{text}"
    );
    assert!(text.contains("type = List(Int, String)"), "{text}");
    assert!(
        text.contains("$2 = Entry ( (7, \"seven\"), Empty )"),
        "{text}"
    );
    assert!(text.contains("type = List List(Int, String)"), "{text}");
    assert!(
        text.contains("$3 = Entry ( Entry ( (7, \"seven\"), Empty ), Empty )"),
        "{text}"
    );
    assert!(text.contains("topal.fn.preserve.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn basic_int_list_operations_are_freestanding_and_debuggable() {
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-PREPEND-001, TOPAL-LIST-APPEND-001, TOPAL-LIST-CONCAT-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-LIST-EMPTY-001, TOPAL-LIST-ONE-001, TOPAL-LIST-UNCONS-001,
    // TOPAL-LIST-FIRST-001, TOPAL-LIST-REST-001, TOPAL-LIST-REVERSE-001,
    // TOPAL-COMPILER-LIST-INT-CORE-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-int-list-core");
    let source = directory.join("int-list-core-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ninspect is fn (values : List Int) -> Optional Int\n  values\n    Empty then None Int\n    Entry (value, rest) then Some value\nvalues : List Int is Entry (-170141183460469231731687303715884105728, Entry (340282366920938463463374607431768211456, Empty))\nempty-values is empty List Int\nsingleton is one 9\nprepended is values prepend 0\nappended is values append 9\ncombined is prepended concat singleton\nreversed is combined reverse\nfirst-value is first combined\nrest-value is rest combined\nparts is uncons combined\nempty-first is first empty-values\nempty-rest is rest empty-values\nempty-parts is uncons empty-values\nempty-plus-values is empty-values concat values\nvalues-plus-empty is values concat empty-values\nempty-reversed is empty-values reverse\nlonger is values append 9\ndecision is inspect values\n(decision, first-value, rest-value, parts, empty-first, empty-rest, empty-parts, entry-count combined, empty? combined, empty? empty-values, reversed reverse = combined, empty-plus-values, values-plus-empty, empty-reversed, empty-values = empty-reversed, values = values-plus-empty, values = longer, reversed)\n",
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
        b"(Some -170141183460469231731687303715884105728, Some 0, Some Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Entry ( 9, Empty ) ) ), Some (0, Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Entry ( 9, Empty ) ) )), None, None, None, 4, false, true, true, Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) ), Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) ), Empty, true, true, false, Entry ( 9, Entry ( 340282366920938463463374607431768211456, Entry ( -170141183460469231731687303715884105728, Entry ( 0, Empty ) ) ) ))\n"
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
            "break int-list-core-boundary.t:3",
            "-ex",
            "run",
            "-ex",
            "print values",
            "-ex",
            "up",
            "-ex",
            "whatis 'rest-value'",
            "-ex",
            "print 'rest-value'",
            "-ex",
            "whatis parts",
            "-ex",
            "print parts",
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
        text.contains(
            "$1 = Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Empty ) )"
        ),
        "{text}"
    );
    assert!(text.contains("type = Optional List Int"), "{text}");
    assert!(
        text.contains(
            "$2 = Some Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Entry ( 9, Empty ) ) )"
        ),
        "{text}"
    );
    assert!(text.contains("type = Optional (Int, List Int)"), "{text}");
    assert!(
        text.contains(
            "$3 = Some (0, Entry ( -170141183460469231731687303715884105728, Entry ( 340282366920938463463374607431768211456, Entry ( 9, Empty ) ) ))"
        ),
        "{text}"
    );
    assert!(text.contains("topal.fn.inspect.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn tuple_result_is_freestanding_and_gdb_exposes_its_source_shape() {
    // TOPAL-COMPILER-TUPLE-RESULT-001,
    // TOPAL-EXEC-COMPLETION-EFFECT-VALUE-001,
    // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-tuple-result");
    let source = directory.join("completion-effect-value.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/completion-effect-value.t"),
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
    assert_eq!(executed.stdout, b"(Completed, Effects ())\n");

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

    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break completion-effect-value.t:9",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "ptype 'topal.fn.finish.0'",
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
    assert!(text.contains("type = struct (Completed, Effect)"), "{text}");
    assert!(
        text.contains("enum Completed _0;") && text.contains("enum Effect _1;"),
        "{text}"
    );
    assert!(text.contains("$1 = {_0 = Completed, _1 = empty}"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn tuple_parameter_is_freestanding_and_gdb_exposes_its_source_shape() {
    // TOPAL-COMPILER-TUPLE-PARAMETER-001,
    // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-tuple-parameter");
    let source = directory.join("tuple-function-parameters.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/tuple-function-parameters.t"),
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
        b"(((42, true), \"nested\"), (7, \"kept\"), (0, \"fallback\"), \"tuple\", \"fields\", \"discarded\")\n"
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
            "break tuple-function-parameters.t:8",
            "-ex",
            "run",
            "-ex",
            "ptype 'topal.fn.retain.0'",
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
    assert!(
        text.contains("type = struct ((Int, Boolean), String)"),
        "{text}"
    );
    assert!(text.contains("struct (Int, Boolean) _0;"), "{text}");
    assert!(text.contains("String _1;"), "{text}");
    assert!(
        text.contains("$1 = {_0 = {_0 = 42, _1 = true}, _1 = \"nested\"}"),
        "{text}"
    );
    assert!(text.contains("topal.fn.retain.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn record_boundaries_are_freestanding_and_gdb_exposes_source_fields() {
    // TOPAL-COMPILER-RECORD-BOUNDARY-001,
    // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-record-boundary");
    let source = directory.join("record-function-boundaries.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/record-function-boundaries.t"),
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
        b"((name is \"Ada\", active is true), (active is false, name is \"Grace\"), (active is true, name is \"first\"), (name is \"second\", active is false), (score is 42, person is (name is \"Lin\", active is true)), \"Grace\", (value is -2, label is \"negative\"), (label is \"nonnegative\", value is 2), (label is \"less\", value is -1), (label is \"right\", value is 1), (value is 7, label is \"some\"), (label is \"none\", value is 0), (label is \"ok\", value is Rational ( 1, 2 )), (value is Rational ( 0, 1 ), label is \"error\"))\n"
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
            "break record-function-boundaries.t:11",
            "-ex",
            "run",
            "-ex",
            "ptype 'topal.fn.retain_2dperson.1'",
            "-ex",
            "whatis person",
            "-ex",
            "print person",
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
        text.contains("type = struct (active : Boolean, name : String)"),
        "{text}"
    );
    assert!(text.contains("Boolean active;"), "{text}");
    assert!(text.contains("String name;"), "{text}");
    assert!(
        text.contains("$1 = {active = false, name = \"Grace\"}"),
        "{text}"
    );
    assert!(text.contains("topal.fn.retain_2dperson.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}
