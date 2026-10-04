#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_fundamental_type_identity_across_a_function_boundary() {
    // TOPAL-COMP-DEBUG-001, TOPAL-ABSTRACTION-TYPE-BOUNDARY-001
    let directory = temporary("gdb-type-boundary");
    let source = directory.join("type-function-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/type-function-boundary.t"),
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
            "break type-function-boundary.t:7",
            "-ex",
            "run",
            "-ex",
            "whatis kind",
            "-ex",
            "print kind",
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
    assert!(text.contains("type = enum Type"), "{text}");
    assert!(text.contains("$1 = Int"), "{text}");
    assert!(text.contains("topal.fn.identity_2dtype.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn named_constraint_value_is_freestanding_and_debuggable() {
    // TOPAL-COMPILER-CONSTRAINT-VALUE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-constraint-value");
    let source = directory.join("constraint-classifier.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/constraint-classifier.t"),
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
    assert_eq!(executed.stdout, b"<Constraint rule>\n");

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
            "break constraint-classifier.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis Positive",
            "-ex",
            "print Positive",
            "-ex",
            "whatis rule",
            "-ex",
            "print rule",
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
        text.matches("type = enum Constraint").count() >= 2,
        "{text}"
    );
    assert!(text.contains("= <Constraint Positive>"), "{text}");
    assert!(text.contains("= <Constraint rule>"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn constraint_validation_is_freestanding_and_debuggable() {
    // TOPAL-COMPILER-CONSTRAINT-VALIDATE-001,
    // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-constraint-validation");
    let source = directory.join("constraints-and-derived-capabilities.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/constraints-and-derived-capabilities.t"),
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
        b"(3, true, true, 5, Error ( domain is root.Positive(Int), code is out-of-range ))\n"
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
            "break topal.fn.validate.0",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis first",
            "-ex",
            "print first",
            "-ex",
            "whatis second",
            "-ex",
            "print second",
            "-ex",
            "down",
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
    assert!(text.matches("type = Positive").count() >= 2, "{text}");
    assert!(text.contains("$1 = 3"), "{text}");
    assert!(text.contains("$2 = 5"), "{text}");
    assert!(text.contains("$3 = 0"), "{text}");
    assert!(text.contains("topal.fn.validate.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn constraint_validation_executes_dynamic_success_and_failure() {
    // TOPAL-COMPILER-CONSTRAINT-VALIDATE-001,
    // TOPAL-TYPE-CONSTRAINT-VALIDATE-001
    let directory = temporary("constraint-validation-paths");
    let source = directory.join("constraint-validation-paths.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nPositive is Int constraint { value } value > 0\nvalidate is fn (value : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  Positive value\n(validate 3, validate 0)\n",
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(executable));
    assert!(executed.status.success());
    assert_eq!(
        executed.stdout,
        b"(3, Error ( domain is root.Positive(Int), code is out-of-range ))\n"
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn fundamental_constraint_bases_are_freestanding_and_match_the_interpreter() {
    // TOPAL-COMPILER-CONSTRAINT-FUNDAMENTAL-BASES-001,
    // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("constraint-fundamental-bases");
    let source = directory.join("constraint-fundamental-bases.t");
    let executable = directory.join("application");
    let source_text = include_str!("../../../../examples/language/constraint-fundamental-bases.t");
    fs::write(&source, source_text).unwrap();
    let expected = Session::new()
        .evaluate_source_file(source_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
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
            "break topal.fn.validate_2dpass.0",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis accepted",
            "-ex",
            "print accepted",
            "-ex",
            "whatis name",
            "-ex",
            "whatis ratio",
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
    assert!(text.contains("type = Pass"), "{text}");
    assert!(text.contains("type = Nonempty"), "{text}");
    assert!(text.contains("type = PositiveRational"), "{text}");
    assert!(text.contains("$1 = true"), "{text}");
    assert!(text.contains("topal.fn.validate_2dpass.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn dependency_only_standard_library_program_matches_the_interpreter() {
    // TOPAL-SYN-LIBRARY-001, TOPAL-LIB-DEPENDENCY-001,
    // TOPAL-COMPILER-LIBRARY-DEPENDENCY-001,
    // TOPAL-COMPILER-NAT-ARITHMETIC-001
    let directory = temporary("dependency-only-standard-library");
    let source = directory.join("packet-filter.t");
    let executable = directory.join("application");
    let source_text = include_str!("../../../../examples/data-transfer/packet-filter.t");
    fs::write(&source, source_text).unwrap();
    let expected = Session::new()
        .evaluate_source_file(source_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let metadata =
        NativeArtifactMetadata::decode(&fs::read(metadata_path(&executable)).unwrap()).unwrap();
    assert!(metadata.dependencies.is_empty());

    let failing_source = directory.join("top-level-projection-failure.t");
    let failing_executable = directory.join("failing-application");
    let failing_text = "use language (version is v0.1)\nuse library std (version is v0.1)\nPass is Boolean constraint { value } value\nidentity is fn (value : Boolean) -> Boolean\n  value\nrejected : Pass is Pass (identity false)\nrejected\n";
    fs::write(&failing_source, failing_text).unwrap();
    let interpreted = Session::new()
        .evaluate_source_file(failing_text, &mut std::io::sink())
        .unwrap_err();
    assert_eq!(interpreted.code, "E-RESULT-PROJECTION-OUTSIDE-FUNCTION");
    let compiled = run(topalc().args([
        "-o",
        failing_executable.to_str().unwrap(),
        failing_source.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&failing_executable));
    assert_eq!(executed.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&executed.stdout)
            .contains("Error ( domain is root.Pass(Boolean), code is out-of-range )")
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn assert_source_library_program(
    directory: &Path,
    library_root: &Path,
    name: &str,
    source_text: &str,
    dependency_identities: &[&str],
) {
    let source = directory.join(format!("{name}.t"));
    let executable = directory.join(name);
    fs::write(&source, source_text).unwrap();
    let mut session = Session::new();
    let interpreted_root = directory.join("interpreted-library");
    let libraries = declared_libraries(source_text);
    let names = libraries.iter().map(String::as_str).collect::<Vec<_>>();
    for module in select_source_modules(source_text, library_root, &names).unwrap() {
        let source_path = Path::new(&module.source_name);
        let relative = source_path.strip_prefix(library_root).unwrap();
        let destination = interpreted_root.join(relative);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::write(destination, module.source).unwrap();
    }
    load_module_tree(&mut session, &interpreted_root, &mut std::io::sink()).unwrap();
    let expected = session
        .evaluate_source_file(source_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let compiled = run(topalc().args([
        "--library-root",
        library_root.to_str().unwrap(),
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
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let metadata =
        NativeArtifactMetadata::decode(&fs::read(metadata_path(&executable)).unwrap()).unwrap();
    assert_eq!(
        metadata
            .dependencies
            .iter()
            .map(|dependency| dependency.identity.as_str())
            .collect::<Vec<_>>(),
        dependency_identities
    );
    assert!(
        metadata
            .dependencies
            .iter()
            .all(|dependency| dependency.sha256.len() == 64)
    );
    if name == "firewall" {
        let changed_root = directory.join("changed-library");
        fs::create_dir_all(changed_root.join("std/data")).unwrap();
        fs::create_dir_all(changed_root.join("std/network")).unwrap();
        fs::write(
            changed_root.join("std/data/spans.t"),
            format!(
                "{}\n# dependency digest change\n",
                include_str!("../../../../library/std/data/spans.t")
            ),
        )
        .unwrap();
        fs::write(
            changed_root.join("std/network/addresses.t"),
            include_str!("../../../../library/std/network/addresses.t"),
        )
        .unwrap();
        let changed_executable = directory.join("firewall-changed-dependency");
        let changed = run(topalc().args([
            "--library-root",
            changed_root.to_str().unwrap(),
            "-o",
            changed_executable.to_str().unwrap(),
            source.to_str().unwrap(),
        ]));
        assert!(
            changed.status.success(),
            "{}",
            String::from_utf8_lossy(&changed.stderr)
        );
        let changed_metadata =
            NativeArtifactMetadata::decode(&fs::read(metadata_path(&changed_executable)).unwrap())
                .unwrap();
        assert_eq!(changed_metadata.source_sha256, metadata.source_sha256);
        assert_ne!(changed_metadata.dependencies, metadata.dependencies);
        assert_ne!(changed_metadata.build_identity, metadata.build_identity);
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn standard_library_application_matches_the_interpreter() {
    // TOPAL-LIB-SOURCE-001, TOPAL-COMPILER-LIBRARY-SOURCE-001,
    // TOPAL-COMPILER-STANDARD-LIBRARY-PARITY-001
    let directory = temporary("standard-library-application");
    let library_root = directory.join("library");
    fs::create_dir_all(library_root.join("std")).unwrap();
    fs::write(
        library_root.join("std/module.t"),
        include_str!("../../../../library/std/module.t"),
    )
    .unwrap();
    assert_source_library_program(
        &directory,
        &library_root,
        "application",
        include_str!("../../../../library/application.t"),
        &["std"],
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // The source-module matrix keeps each dependency identity beside its corpus.
fn qualified_standard_library_functions_compile_from_source_modules() {
    // TOPAL-LIB-SOURCE-001, TOPAL-NAMESPACE-USE-001,
    // TOPAL-COMPILER-LIBRARY-SOURCE-001,
    // TOPAL-COMPILER-LIBRARY-MODULE-SELECTION-001,
    // TOPAL-COMPILER-PACKING-LISTS-001
    let directory = temporary("qualified-standard-library-source");
    let library_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
    for (name, source_text, dependency_identities) in [
        (
            "build-graph",
            include_str!("../../../../tests/standard-library/build-graph.t"),
            &["std.build.graph"][..],
        ),
        (
            "combinatorics-algorithms",
            include_str!("../../../../tests/standard-library/combinatorics-algorithms.t"),
            &["std.combinatorics"][..],
        ),
        (
            "parse-algorithms",
            include_str!("../../../../tests/standard-library/parse-algorithms.t"),
            &["std.parse"][..],
        ),
        (
            "ordered-algorithms",
            include_str!("../../../../tests/standard-library/ordered-algorithms.t"),
            &["std.ordered"][..],
        ),
        (
            "sequence-algorithms",
            include_str!("../../../../tests/standard-library/sequence-algorithms.t"),
            &["std.sequence"][..],
        ),
        (
            "fundamental-boundaries",
            include_str!("../../../../tests/standard-library/fundamental-boundaries.t"),
            &["std"][..],
        ),
        (
            "geometry-algorithms",
            include_str!("../../../../tests/standard-library/geometry-algorithms.t"),
            &["advent-of-code.geometry"][..],
        ),
        (
            "machine-algorithms",
            include_str!("../../../../tests/standard-library/machine-algorithms.t"),
            &["advent-of-code.machine"][..],
        ),
        (
            "packing-algorithms",
            include_str!("../../../../tests/standard-library/packing-algorithms.t"),
            &["advent-of-code.packing"][..],
        ),
        (
            "specialized-algorithms",
            include_str!("../../../../tests/standard-library/specialized-algorithms.t"),
            &[
                "std.combinatorics",
                "std.graph",
                "std.statistics",
                "std.text",
            ][..],
        ),
        (
            "statistics-algorithms",
            include_str!("../../../../tests/standard-library/statistics-algorithms.t"),
            &["std.statistics"][..],
        ),
        (
            "firewall",
            include_str!("../../../../examples/data-transfer/firewall.t"),
            &["std.data.spans", "std.network.addresses"][..],
        ),
        (
            "rest-controller",
            include_str!("../../../../examples/data-transfer/rest-controller.t"),
            &["std.web.http"][..],
        ),
        (
            "device-i2c",
            include_str!("../../../../tests/standard-library/device-i2c.t"),
            &["std.device.i2c"][..],
        ),
        (
            "data-spans",
            include_str!("../../../../tests/standard-library/data-spans.t"),
            &["std.data.spans"][..],
        ),
        (
            "network-addresses",
            include_str!("../../../../tests/standard-library/network-addresses.t"),
            &["std.network.addresses"][..],
        ),
        (
            "store-memory",
            include_str!("../../../../tests/standard-library/store-memory.t"),
            &["std", "std.store.memory"][..],
        ),
        (
            "transfer-queues",
            include_str!("../../../../tests/standard-library/transfer-queues.t"),
            &["std", "std.transfer.queues"][..],
        ),
        (
            "web-http",
            include_str!("../../../../tests/standard-library/web-http.t"),
            &["std.web.http"][..],
        ),
    ] {
        assert_source_library_program(
            &directory,
            &library_root,
            name,
            source_text,
            dependency_identities,
        );
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn capability_generic_standard_library_facade_is_specialized_from_source() {
    // TOPAL-COMPILER-LIBRARY-GENERIC-001, TOPAL-LIB-SOURCE-001,
    // TOPAL-TYPE-CALL-001, TOPAL-CAPABILITY-COMPOSE-001
    let directory = temporary("generic-standard-library-facade");
    let library_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
    let source_text = "use language (\n  version is v0.1\n)\nuse library std (\n  version is v0.1\n)\nminimum is std min\nmaximum is std max\nordered is std min-max\npresent? is std present?\nabsent? is std absent?\nvalue-or is std value-or\nlower-bound is std lower-bound\nbounds is std bounds\n((minimum (4, 2)) = 2,\n (maximum (2.0, 4.0)) = (Rational 4),\n (ordered (4, 2)) = (2, 4),\n present? (Some 7), absent? (None Int),\n (value-or ((None Int), 9)) = 9,\n (lower-bound (-2 <..= 5)) = -2,\n (bounds (-2 .. 5)) = (-2, 5))\n";
    assert_source_library_program(
        &directory,
        &library_root,
        "generic-facade",
        source_text,
        &["std"],
    );

    let packaged_source = "use language (\n  version is v0.1\n)\nuse library std (\n  version is v0.1\n)\nenqueue? is std transfer queues enqueue?\nabsent? is std absent?\nvalue-or is std value-or\nempty : List Int is Empty\none : List Int is Entry (7, Empty)\n(absent? (enqueue? (empty, 7, 0)),\n (value-or ((enqueue? (empty, 7, 1)), empty)) = one)\n";
    assert_source_library_program(
        &directory,
        &library_root,
        "generic-packaged-module",
        packaged_source,
        &["std", "std.transfer.queues"],
    );

    let invalid_source = directory.join("invalid-generic-capability.t");
    let invalid_executable = directory.join("invalid-generic-capability");
    fs::write(
        &invalid_source,
        "use language (version is v0.1)\nuse library std (version is v0.1)\nminimum is std min\nminimum (false, true)\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "--library-root",
        library_root.to_str().unwrap(),
        "-o",
        invalid_executable.to_str().unwrap(),
        invalid_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E-NO-APPLICABLE-OVERLOAD"));
    assert!(!invalid_executable.exists());
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn nominal_sums_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-SUM-001, TOPAL-TYPE-UNION-001,
    // TOPAL-TYPE-VARIANT-001, TOPAL-DECISION-UNION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-nominal-sums");
    let source = directory.join("unions-and-recursive-products.t");
    let executable = directory.join("application");
    let source_text = include_str!("../../../../examples/language/unions-and-recursive-products.t");
    fs::write(&source, source_text).unwrap();
    let expected = Session::new()
        .evaluate_source_file(source_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());

    let metadata =
        NativeArtifactMetadata::decode(&fs::read(metadata_path(&executable)).unwrap()).unwrap();
    assert_eq!(metadata.native_abi, NATIVE_ABI);
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
            "break unions-and-recursive-products.t:15",
            "-ex",
            "break unions-and-recursive-products.t:20",
            "-ex",
            "run",
            "-ex",
            "whatis message",
            "-ex",
            "print message",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "whatis scalar",
            "-ex",
            "print scalar",
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
    assert!(text.contains("= Move (10, (20, 30))"), "{text}");
    assert!(text.contains("type = Scalar"), "{text}");
    assert!(text.contains("= at 0 \"text\""), "{text}");
    assert!(text.contains("topal.fn.describe"), "{text}");
    assert!(text.contains("topal.fn.show_2dscalar"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn nominal_sum_display_matches_the_interpreter() {
    // TOPAL-COMPILER-SUM-001, TOPAL-TYPE-UNION-001,
    // TOPAL-TYPE-VARIANT-001
    let directory = temporary("nominal-sum-display");
    let source = directory.join("sum-display.t");
    let executable = directory.join("application");
    let source_text = "use language (version is v0.1)\nMessage is Union\n  Stop\n  Move : (Int, (Int, Int))\n\nScalar is Variant (String, Int)\n\n(Stop, Move (10, (20, 30)), Scalar at 0 \"text\", Scalar at 1 42)\n";
    fs::write(&source, source_text).unwrap();
    let expected = Session::new()
        .evaluate_source_file(source_text, &mut std::io::sink())
        .unwrap()
        .to_string()
        + "\n";
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, expected.as_bytes());
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn root_namespace_is_freestanding_and_qualified_calls_keep_debug_frames() {
    // TOPAL-COMPILER-ROOT-NAMESPACE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-root-namespace");
    let source = directory.join("root-namespace.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/root-namespace.t"),
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
    assert_eq!(executed.stdout, b"(<namespace root>, 42)\n");

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
            "break root-namespace.t:8",
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
    assert!(text.contains("type = Int"), "{text}");
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.increment.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn use_root_namespace_is_static_freestanding_and_debuggable() {
    // TOPAL-COMPILER-NAMESPACE-USE-001, TOPAL-NAMESPACE-USE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-use-root-namespace");
    let source = directory.join("use-namespace.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/use-namespace.t"),
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
            "break use-namespace.t:10",
            "-ex",
            "run",
            "-ex",
            "whatis current",
            "-ex",
            "print current",
            "-ex",
            "break use-namespace.t:7",
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
    assert!(text.contains("type = enum Scope"), "{text}");
    assert!(text.contains("$1 = <namespace root>"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("topal.fn.increment.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn namespace_function_alias_is_static_freestanding_and_debuggable() {
    // TOPAL-COMPILER-NAMESPACE-FUNCTION-ALIAS-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-namespace-function-alias");
    let source = directory.join("namespace-alias.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/namespace-alias.t"),
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
            "break namespace-alias.t:11",
            "-ex",
            "run",
            "-ex",
            "whatis current",
            "-ex",
            "print current",
            "-ex",
            "break namespace-alias.t:8",
            "-ex",
            "continue",
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
    assert!(text.contains("type = enum Scope"), "{text}");
    assert!(text.contains("$1 = <namespace root>"), "{text}");
    assert!(text.contains("type = Int"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("topal.fn.increment.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn namespace_generator_is_static_freestanding_and_debuggable() {
    // TOPAL-COMPILER-NAMESPACE-GENERATOR-001,
    // TOPAL-NAMESPACE-GENERATOR-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-namespace-generator");
    let executable = directory.join("application");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/namespace-generator.t");
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
            "whatis api",
            "-ex",
            "print api",
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
        "type = enum Scope",
        "$1 = <namespace root>",
        "type = enum Generator Character Unit Unit",
        "$2 = <Generator Character Unit Unit>",
        "type = Character",
        "$3 = \"T\"",
        "namespace-generator.t:14",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn namespace_data_snapshot_is_single_evaluation_freestanding_and_debuggable() {
    // TOPAL-COMPILER-NAMESPACE-DATA-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-namespace-data-snapshot");
    let source = directory.join("namespace-snapshot.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/namespace-snapshot.t"),
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
    assert_eq!(executed.stdout, b"(41, 42)\n");

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
            "break namespace-snapshot.t:9",
            "-ex",
            "run",
            "-ex",
            "whatis earlier",
            "-ex",
            "print earlier",
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
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn scope_parameter_is_specialized_freestanding_and_debuggable() {
    // TOPAL-COMPILER-NAMESPACE-BOUNDARY-001,
    // TOPAL-NAMESPACE-FUNCTION-BOUNDARY-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-scope-parameter");
    let source = directory.join("namespace-function-parameter.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/namespace-function-parameter.t"),
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
            "break namespace-function-parameter.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis api",
            "-ex",
            "print api",
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
    assert!(text.contains("api answer = 42"), "{text}");
    assert!(text.contains("topal.fn.read_2danswer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn named_function_value_is_direct_freestanding_and_debuggable() {
    // TOPAL-COMPILER-NAMED-FUNCTION-VALUE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-named-function-value");
    let source = directory.join("named-function-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/named-function-values.t"),
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
            "break named-function-values.t:11",
            "-ex",
            "run",
            "-ex",
            "whatis operation",
            "-ex",
            "print operation",
            "-ex",
            "break named-function-values.t:8",
            "-ex",
            "continue",
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
    assert!(text.contains("type = enum Function"), "{text}");
    assert!(text.contains("$1 = <fn increment>"), "{text}");
    assert!(text.contains("type = Int"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("topal.fn.increment.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn symbolic_callable_values_are_direct_freestanding_and_debuggable() {
    // TOPAL-COMPILER-SYMBOLIC-CALLABLE-VALUE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-symbolic-callable-values");
    let source = directory.join("callable-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/callable-values.t"),
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
    assert_eq!(executed.stdout, b"(42, -5, Less)\n");

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
            "break callable-values.t:10",
            "-ex",
            "run",
            "-ex",
            "whatis add",
            "-ex",
            "print add",
            "-ex",
            "print negate",
            "-ex",
            "print 'compare-values'",
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
    assert!(text.contains("$2 = -"), "{text}");
    assert!(text.contains("$3 = <=>"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn expanded_symbolic_callable_values_are_direct_freestanding_and_debuggable() {
    // TOPAL-COMPILER-SYMBOLIC-CALLABLE-EXPANDED-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-expanded-symbolic-callable-values");
    let source = directory.join("expanded-callable-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/expanded-callable-values.t"),
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
        b"(true, true, true, true, true, true, 42, Rational ( 3, 4 ), (3, 2), 3, 1024, 0 .. 3, 0 <.. 3, 0 ..= 3, 0 <..= 3, 42)\n"
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
            "break expanded-callable-values.t:24",
            "-ex",
            "break topal.runtime.int.compare",
            "-ex",
            "run",
            "-ex",
            "print operation",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "up",
            "-ex",
            "print equal",
            "-ex",
            "print 'not-equal'",
            "-ex",
            "print multiply",
            "-ex",
            "print divide",
            "-ex",
            "print 'half-open'",
            "-ex",
            "print 'lower-open'",
            "-ex",
            "print selected",
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
        "$1 = *",
        "$2 = =",
        "$3 = /=",
        "$4 = *",
        "$5 = /",
        "$6 = ..",
        "$7 = <..=",
        "$8 = *",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
    assert!(text.contains("topal.fn.select"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}
