static NEXT_TEST: AtomicU64 = AtomicU64::new(0);

fn topalc() -> Command {
    Command::new(env!("CARGO_BIN_EXE_topalc"))
}

fn temporary(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/topalc-tests");
    fs::create_dir_all(&root).unwrap();
    loop {
        let sequence = NEXT_TEST.fetch_add(1, Ordering::Relaxed);
        let path = root.join(format!("{name}-{}-{sequence}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => panic!("cannot create {}: {error}", path.display()),
        }
    }
}

fn run(command: &mut Command) -> Output {
    command.output().unwrap()
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn assert_direct_return_without_classifier(ir_text: &str, symbol: &str) {
    let function = ir_text
        .split_once(symbol)
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(
        function.matches("call ptr @topal.runtime.int.add").count(),
        1
    );
    assert_eq!(function.matches("ret ptr").count(), 1);
    for instruction in [
        "call i32 @topal.runtime.int.compare",
        "icmp",
        "br i1",
        "switch ",
        "select i1",
        " phi ",
        "@topal.platform.allocate",
    ] {
        assert!(!function.contains(instruction), "{function}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn assert_freestanding_elf_and_valid_dwarf(executable: &Path) {
    let tools = LlvmTools::discover(None).unwrap();
    let undefined = run(Command::new(tools.directory.join("llvm-nm"))
        .arg("--undefined-only")
        .arg(executable));
    assert!(undefined.status.success());
    assert!(
        undefined.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&undefined.stdout)
    );
    let inspected = run(Command::new(tools.directory.join("llvm-readobj"))
        .args(["--needed-libs", "--relocations"])
        .arg(executable));
    assert!(inspected.status.success());
    let inspected = String::from_utf8_lossy(&inspected.stdout);
    assert!(inspected.contains("NeededLibraries [\n]"), "{inspected}");
    assert!(inspected.contains("Relocations [\n]"), "{inspected}");
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(executable));
        assert!(
            dwarf.status.success(),
            "{}",
            String::from_utf8_lossy(&dwarf.stderr)
        );
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn assert_serialization_corruption_exits(executable: &Path, mutation: &str) {
    let corrupted = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break topal.runtime.serialization.verify",
            "-ex",
            "run",
            "-ex",
            mutation,
            "-ex",
            "continue",
        ])
        .arg(executable));
    assert!(
        corrupted.status.success(),
        "{}",
        String::from_utf8_lossy(&corrupted.stderr)
    );
    let corrupted = String::from_utf8_lossy(&corrupted.stdout);
    assert!(corrupted.contains("exited with code 0106"), "{corrupted}");
}

#[test]
fn compiles_and_executes_shared_regression_with_canonical_metadata() {
    // TOPAL-COMPILER-TEST-001, TOPAL-COMPILER-ARTIFACT-001
    let directory = temporary("shared-regression");
    let output = directory.join("ordinary-functions");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/ordinary-functions.t");
    let result = run(topalc().args([
        "-O0",
        "-g",
        "-o",
        output.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let executed = run(&mut Command::new(&output));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"42\n");

    let bytes = fs::read(metadata_path(&output)).unwrap();
    let metadata = NativeArtifactMetadata::decode(&bytes).unwrap();
    assert_eq!(metadata.target_triple, "x86_64-unknown-linux-gnu");
    assert!(metadata.llvm_version.starts_with("22."));
    assert_eq!(metadata.optimization, 0);
    assert_eq!(metadata.cpu, "x86-64");
    assert!(metadata.features.is_empty());
    assert_eq!(metadata.evidence.len(), 1);
    assert_eq!(
        metadata.evidence[0].identity,
        "topal.architecture.generic-x86_64-linux/1"
    );
    assert_eq!(metadata.evidence[0].sha256.len(), 64);
    assert_eq!(metadata.provenance, ["topal.optimization-plan/1"]);
    assert_eq!(metadata.native_slices[0].kind, "executable");
}

#[test]
fn rejects_unqualified_target_selections_before_toolchain_use() {
    // TOPAL-OPT-TARGET-001
    let directory = temporary("unsupported-optimization-target");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/ordinary-functions.t");
    let cases: &[&[&str]] = &[
        &["--cpu", "native"],
        &["--board", "unknown-board"],
        &["--target", "aarch64-unknown-linux-gnu"],
    ];
    for (index, options) in cases.iter().enumerate() {
        let output = directory.join(format!("rejected-{index}"));
        let result = run(topalc().args(*options).arg("-o").arg(&output).arg(&source));
        assert!(!result.status.success());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains("not implemented") || stderr.contains("unsupported"),
            "{stderr}"
        );
        assert!(!output.exists());
        assert!(!metadata_path(&output).exists());
    }
}

#[test]
fn lists_stable_optimizations_without_source_input() {
    // TOPAL-OPT-LIST-001
    let result = run(topalc().arg("--list-optimizations"));
    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("topal.runtime-global-dce/1"));
    assert!(stdout.contains("llvm.default-pipeline/22"));
    assert!(stdout.contains("Status: implemented"));
}

#[test]
fn lists_executable_and_model_only_targets_without_source_input() {
    // TOPAL-COMPILER-TARGET-REGISTRY-001
    let result = run(topalc().arg("--list-targets"));
    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("Registry: topal.target-qualification/1"));
    assert!(stdout.contains("generic-x86-64-linux"));
    assert!(stdout.contains("Status: executable-qualified"));
    assert!(stdout.contains("example-x86-64-avx2"));
    assert!(stdout.contains("example-riscv-dsp-board"));
    assert_eq!(stdout.matches("Status: model-only").count(), 2);
}

#[test]
fn model_only_target_selection_fails_with_qualification_details() {
    // TOPAL-COMPILER-TARGET-REGISTRY-001
    let directory = temporary("model-only-target-selection");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/ordinary-functions.t");
    let output = directory.join("unqualified");
    let result = run(topalc()
        .args(["--cpu", "x86-64-avx2"])
        .arg("-o")
        .arg(&output)
        .arg(&source));
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("model-only"), "{stderr}");
    assert!(stderr.contains("instruction legality"), "{stderr}");
    assert!(stderr.contains("topal.target-qualification/1"), "{stderr}");
    assert!(!output.exists());
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn controls_pipeline_and_writes_deterministic_optimization_explanation() {
    // TOPAL-OPT-CONTROL-001, TOPAL-OPT-EXPLAIN-001
    let directory = temporary("optimization-controls");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/ordinary-functions.t");
    let output = directory.join("controlled");
    let explanation = directory.join("optimization.json");
    let explanation_argument = format!("--explain-optimizations={}", explanation.display());
    let result = run(topalc().args([
        "-O3",
        "--disable-optimization",
        "llvm.default-pipeline/22",
        "--enable-optimization",
        "topal.runtime-global-dce/1",
        "--optimization-goal",
        "code-size",
        "--optimization-limit",
        "code-size=64KiB",
        &explanation_argument,
        "-o",
        output.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(run(&mut Command::new(&output)).stdout, b"42\n");
    let report = fs::read_to_string(explanation).unwrap();
    assert!(report.contains("\"profile\": \"O3\""));
    assert!(report.contains("\"code-size=64KiB\""));
    assert!(report.contains("topal.runtime-global-dce/1"));
    assert!(report.contains("optimization-workload"));
    assert!(!report.contains("llvm.default-pipeline/22"));
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn hard_code_size_limit_rejects_before_artifact_publication() {
    // TOPAL-OPT-FEASIBLE-001, TOPAL-OPT-CONTROL-001
    let directory = temporary("optimization-hard-limit");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/ordinary-functions.t");
    let output = directory.join("too-large");
    let result = run(topalc().args([
        "-O2",
        "--optimization-limit",
        "code-size=1B",
        "-o",
        output.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("exceeds optimization limit"));
    assert!(!output.exists());
    assert!(!metadata_path(&output).exists());
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn o2_default_llvm_pipeline_preserves_existing_program_result() {
    // TOPAL-COMPILER-LLVM-PIPELINE-001
    let directory = temporary("o2-default-pipeline");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/ordinary-functions.t");
    let output = directory.join("ordinary-o2");
    let result = run(topalc().args([
        "-O2",
        "-o",
        output.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(run(&mut Command::new(&output)).stdout, b"42\n");
    let metadata =
        NativeArtifactMetadata::decode(&fs::read(metadata_path(&output)).unwrap()).unwrap();
    assert_eq!(metadata.optimization, 2);
    assert!(
        metadata
            .provenance
            .contains(&"llvm.default-pipeline/22".into())
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn existing_example_covers_unoptimized_isolated_and_all_optimization_matrix() {
    // TOPAL-COMPILER-OPTIMIZATION-MATRIX-001
    let directory = temporary("optimization-fixture-matrix");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/ordinary-functions.t");
    let cases: &[(&str, &[&str], u8)] = &[
        ("none", &["-O0"], 0),
        (
            "isolated-runtime-dce",
            &["--only-optimization", "topal.runtime-global-dce/1"],
            1,
        ),
        ("all", &["-O2"], 2),
    ];
    let mut ir_sizes = Vec::new();
    let mut executable_sizes = Vec::new();
    for (name, controls, expected_profile) in cases {
        let ir = directory.join(format!("{name}.ll"));
        let ir_result = run(topalc()
            .args(*controls)
            .args(["--emit", "llvm-ir", "-o"])
            .arg(&ir)
            .arg(&source));
        assert!(
            ir_result.status.success(),
            "{}",
            String::from_utf8_lossy(&ir_result.stderr)
        );
        ir_sizes.push(fs::metadata(ir).unwrap().len());

        let executable = directory.join(name);
        let executable_result = run(topalc()
            .args(*controls)
            .arg("-o")
            .arg(&executable)
            .arg(&source));
        assert!(
            executable_result.status.success(),
            "{}",
            String::from_utf8_lossy(&executable_result.stderr)
        );
        assert_eq!(run(&mut Command::new(&executable)).stdout, b"42\n");
        executable_sizes.push(fs::metadata(&executable).unwrap().len());
        let metadata =
            NativeArtifactMetadata::decode(&fs::read(metadata_path(&executable)).unwrap()).unwrap();
        assert_eq!(metadata.optimization, *expected_profile);
    }
    assert!(ir_sizes[1] < ir_sizes[0]);
    assert!(ir_sizes[2] <= ir_sizes[1]);
    assert!(executable_sizes[1] < executable_sizes[0]);
    assert!(executable_sizes[2] <= executable_sizes[1]);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn o1_prunes_unreachable_runtime_and_preserves_existing_program_result() {
    // TOPAL-COMPILER-RUNTIME-GLOBAL-DCE-001, TOPAL-OPT-PROFILE-001
    let directory = temporary("o1-runtime-global-dce");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/ordinary-functions.t");
    let o0_ir = directory.join("ordinary-o0.ll");
    let o1_ir = directory.join("ordinary-o1.ll");
    for (level, output) in [("-O0", &o0_ir), ("-O1", &o1_ir)] {
        let result = run(topalc().args([
            level,
            "--emit",
            "llvm-ir",
            "-o",
            output.to_str().unwrap(),
            source.to_str().unwrap(),
        ]));
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let unoptimized = fs::read_to_string(&o0_ir).unwrap();
    let optimized = fs::read_to_string(&o1_ir).unwrap();
    assert!(unoptimized.contains("define internal ptr @topal.runtime.task.make"));
    assert!(!optimized.contains("@topal.runtime.task.make"));
    assert!(optimized.len() < unoptimized.len());

    let o0_executable = directory.join("ordinary-o0");
    let o1_executable = directory.join("ordinary-o1");
    for (level, expected_profile, executable) in
        [("-O0", 0, &o0_executable), ("-O1", 1, &o1_executable)]
    {
        let compiled = run(topalc().args([
            level,
            "-o",
            executable.to_str().unwrap(),
            source.to_str().unwrap(),
        ]));
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let executed = run(&mut Command::new(executable));
        assert!(executed.status.success());
        assert_eq!(executed.stdout, b"42\n");
        let metadata =
            NativeArtifactMetadata::decode(&fs::read(metadata_path(executable)).unwrap()).unwrap();
        assert_eq!(metadata.optimization, expected_profile);
        if expected_profile == 1 {
            assert!(
                metadata
                    .provenance
                    .contains(&"topal.runtime-global-dce/1".into())
            );
        }
    }
    assert!(
        fs::metadata(&o1_executable).unwrap().len() < fs::metadata(&o0_executable).unwrap().len()
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // The fixture, link, execution, and failure checks are one boundary proof.
fn links_a_checked_generated_c_access_library_from_a_static_archive() {
    // TOPAL-C-ABI-STATIC-001, TOPAL-C-BINDGEN-001,
    // TOPAL-COMPILER-C-ADAPTER-001
    let directory = temporary("c-static-library");
    let library_root = directory.join("library");
    let library = library_root.join("arithmetic");
    fs::create_dir_all(&library).unwrap();
    let header = b"int c_add(int left, int right);\n";
    fs::write(library.join("interface.h"), header).unwrap();
    let tools = LlvmTools::discover(None).unwrap();
    let ir = directory.join("arithmetic.ll");
    let bitcode = directory.join("arithmetic.bc");
    let object = directory.join("arithmetic.o");
    let archive = library.join("libarithmetic.a");
    fs::write(
        &ir,
        "target triple = \"x86_64-unknown-linux-gnu\"\ndefine i32 @c_add(i32 %left, i32 %right) {\nentry:\n  %sum = add i32 %left, %right\n  ret i32 %sum\n}\n",
    )
    .unwrap();
    for output in [
        run(Command::new(tools.directory.join("llvm-as")).args([
            ir.to_str().unwrap(),
            "-o",
            bitcode.to_str().unwrap(),
        ])),
        run(Command::new(tools.directory.join("llc")).args([
            "-filetype=obj",
            "-mtriple=x86_64-unknown-linux-gnu",
            bitcode.to_str().unwrap(),
            "-o",
            object.to_str().unwrap(),
        ])),
        run(Command::new(tools.directory.join("llvm-ar")).args([
            "rcs",
            archive.to_str().unwrap(),
            object.to_str().unwrap(),
        ])),
    ] {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let archive_bytes = fs::read(&archive).unwrap();
    let manifest = AccessLibrary {
        schema: SCHEMA.into(),
        identity: "arithmetic".into(),
        version: "v0.1".into(),
        target: TARGET.into(),
        platform_abi: PLATFORM_ABI.into(),
        clang_version: "clang version 22.0.0".into(),
        header: InputArtifact {
            file: "interface.h".into(),
            sha256: format!("{:x}", Sha256::digest(header)),
        },
        static_archive: InputArtifact {
            file: "libarithmetic.a".into(),
            sha256: format!("{:x}", Sha256::digest(&archive_bytes)),
        },
        functions: vec![Function {
            topal_name: "add".into(),
            symbol: "c_add".into(),
            parameters: vec![
                Parameter {
                    name: "left".into(),
                    value: CValue::SignedInt32,
                },
                Parameter {
                    name: "right".into(),
                    value: CValue::SignedInt32,
                },
            ],
            result: CValue::SignedInt32,
            effect: CEffect::NoObservableEffect,
        }],
    };
    fs::write(library.join("module.t"), manifest.topal_source()).unwrap();

    let source = directory.join("application.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nuse library arithmetic (version is v0.1)\nadd is arithmetic add\nadd (20, 22)\n",
    )
    .unwrap();
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
    assert_eq!(executed.stdout, b"42\n");
    assert_freestanding_elf_and_valid_dwarf(&executable);
    let metadata =
        NativeArtifactMetadata::decode(&fs::read(metadata_path(&executable)).unwrap()).unwrap();
    assert!(
        metadata
            .dependencies
            .iter()
            .any(|dependency| dependency.identity == "c-abi.arithmetic.static-archive")
    );

    let range_source = directory.join("range-error.t");
    let range_executable = directory.join("range-error");
    fs::write(
        &range_source,
        "use language (version is v0.1)\nuse library arithmetic (version is v0.1)\nadd is arithmetic add\nadd (2147483648, 0)\n",
    )
    .unwrap();
    let compiled = run(topalc().args([
        "--library-root",
        library_root.to_str().unwrap(),
        "-o",
        range_executable.to_str().unwrap(),
        range_source.to_str().unwrap(),
    ]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&range_executable));
    assert_eq!(executed.status.code(), Some(65));
    assert_eq!(
        executed.stderr,
        b"error[E-C-ABI-RANGE]: Topal Int is not representable as a C signed int\n"
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One end-to-end proof covers ELF input, deployment, linking, and execution.
fn links_and_deploys_a_checked_c_shared_object() {
    // TOPAL-C-ABI-SHARED-001, TOPAL-COMP-C-SHARED-001
    let directory = temporary("c-shared-library");
    let library_root = directory.join("library");
    let library = library_root.join("arithmetic");
    let output_directory = directory.join("output");
    fs::create_dir_all(&library).unwrap();
    fs::create_dir_all(&output_directory).unwrap();
    let header = b"int c_add(int left, int right) __attribute__((const));\n";
    fs::write(library.join("interface.h"), header).unwrap();
    let tools = LlvmTools::discover(None).unwrap();
    let ir = directory.join("arithmetic.ll");
    let bitcode = directory.join("arithmetic.bc");
    let object = directory.join("arithmetic.o");
    let shared_object = library.join("libarithmetic.so");
    fs::write(
        &ir,
        "target triple = \"x86_64-unknown-linux-gnu\"\ndefine i32 @c_add(i32 %left, i32 %right) {\nentry:\n  %sum = add i32 %left, %right\n  ret i32 %sum\n}\n",
    )
    .unwrap();
    for output in [
        run(Command::new(tools.directory.join("llvm-as")).args([
            ir.to_str().unwrap(),
            "-o",
            bitcode.to_str().unwrap(),
        ])),
        run(Command::new(tools.directory.join("llc")).args([
            "-filetype=obj",
            "-relocation-model=pic",
            "-mtriple=x86_64-unknown-linux-gnu",
            bitcode.to_str().unwrap(),
            "-o",
            object.to_str().unwrap(),
        ])),
        run(Command::new(tools.directory.join("rust-lld")).args([
            "-flavor",
            "gnu",
            "-shared",
            "-soname",
            "libarithmetic.so",
            object.to_str().unwrap(),
            "-o",
            shared_object.to_str().unwrap(),
        ])),
    ] {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let shared_bytes = fs::read(&shared_object).unwrap();
    let manifest = SharedAccessLibrary {
        schema: SHARED_SCHEMA.into(),
        identity: "arithmetic".into(),
        version: "v0.1".into(),
        target: TARGET.into(),
        platform_abi: PLATFORM_ABI.into(),
        clang_version: "clang version 22.0.0".into(),
        header: InputArtifact {
            file: "interface.h".into(),
            sha256: format!("{:x}", Sha256::digest(header)),
        },
        shared_object: SharedObjectArtifact {
            file: "libarithmetic.so".into(),
            sha256: format!("{:x}", Sha256::digest(&shared_bytes)),
            soname: "libarithmetic.so".into(),
        },
        functions: vec![Function {
            topal_name: "add".into(),
            symbol: "c_add".into(),
            parameters: vec![
                Parameter {
                    name: "left".into(),
                    value: CValue::SignedInt32,
                },
                Parameter {
                    name: "right".into(),
                    value: CValue::SignedInt32,
                },
            ],
            result: CValue::SignedInt32,
            effect: CEffect::NoObservableEffect,
        }],
    };
    fs::write(library.join("module.t"), manifest.topal_source()).unwrap();

    let source = directory.join("application.t");
    let executable = output_directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nuse library arithmetic (version is v0.1)\nadd is arithmetic add\nadd (20, 22)\n",
    )
    .unwrap();
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
    assert_eq!(
        fs::read(output_directory.join("libarithmetic.so")).unwrap(),
        shared_bytes
    );
    let inspected = run(Command::new(tools.directory.join("llvm-readobj"))
        .args([
            "--program-headers",
            "--dynamic-table",
            "--needed-libs",
            "--string-dump=.interp",
        ])
        .arg(&executable));
    assert!(inspected.status.success());
    let elf = String::from_utf8_lossy(&inspected.stdout);
    assert!(elf.contains("/lib64/ld-linux-x86-64.so.2"), "{elf}");
    assert!(elf.contains("libarithmetic.so"), "{elf}");
    assert!(elf.contains("$ORIGIN"), "{elf}");
    let executed = run(&mut Command::new(&executable));
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(executed.stdout, b"42\n");
    let metadata =
        NativeArtifactMetadata::decode(&fs::read(metadata_path(&executable)).unwrap()).unwrap();
    assert!(
        metadata
            .dependencies
            .iter()
            .any(|dependency| dependency.identity == "c-abi.arithmetic.shared-object")
    );
    assert!(
        metadata
            .platform_requirements
            .contains(&"shared-object:libarithmetic.so".into())
    );

    fs::write(
        output_directory.join("libarithmetic.so"),
        b"conflicting object",
    )
    .unwrap();
    let conflicting_output = output_directory.join("second-application");
    let rejected = run(topalc().args([
        "--library-root",
        library_root.to_str().unwrap(),
        "-o",
        conflicting_output.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("conflicts with required digest"));
    assert!(!conflicting_output.exists());
    assert!(!metadata_path(&conflicting_output).exists());
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)]
fn builds_links_and_deploys_one_sealed_standard_library_object() {
    // TOPAL-LIB-NATIVE-SLICE-001, TOPAL-REQ-NATIVE-LIBRARY-001,
    // TOPAL-COMP-STANDARD-LIBRARY-SHARED-001
    let directory = temporary("standard-library-shared-object");
    let slice_directory = directory.join("slice");
    let output_directory = directory.join("output");
    fs::create_dir_all(&slice_directory).unwrap();
    fs::create_dir_all(&output_directory).unwrap();
    let library_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
    let shared_object = slice_directory.join("libtopal-std.so.1");
    let built = run(topalc().args([
        "std-library",
        "--library-root",
        library_root.to_str().unwrap(),
        "-o",
        shared_object.to_str().unwrap(),
    ]));
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(
        shared_object
            .with_file_name("libtopal-std.so.1.topal-library.json")
            .is_file()
    );

    let tools = LlvmTools::discover(None).unwrap();
    let inspected = run(Command::new(tools.directory.join("llvm-readobj"))
        .args(["--file-header", "--dynamic-table", "--needed-libs"])
        .arg(&shared_object));
    let inspected = String::from_utf8_lossy(&inspected.stdout);
    assert!(inspected.contains("Type: SharedObject"), "{inspected}");
    assert!(
        inspected.contains("Library soname: [libtopal-std.so.1]"),
        "{inspected}"
    );
    assert!(inspected.contains("NeededLibraries [\n]"), "{inspected}");
    let symbols = run(Command::new(tools.directory.join("llvm-nm"))
        .args(["--defined-only", "--extern-only"])
        .arg(&shared_object));
    assert_eq!(
        String::from_utf8_lossy(&symbols.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
        1
    );
    assert!(String::from_utf8_lossy(&symbols.stdout).contains("topal_library_v1_entry"));
    let undefined = run(Command::new(tools.directory.join("llvm-nm"))
        .args(["--undefined-only", "--extern-only"])
        .arg(&shared_object));
    assert!(undefined.stdout.is_empty());

    let source = directory.join("application.t");
    fs::write(
        &source,
        "use language (version is v0.1)\nuse library std (version is v0.1)\nmin is std min\nmin (4, 2)\n",
    )
    .unwrap();
    let executable = output_directory.join("application");
    let compiled = run(topalc().args([
        "--library-root",
        library_root.to_str().unwrap(),
        "--std-library",
        shared_object.to_str().unwrap(),
        "-o",
        executable.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert_eq!(run(&mut Command::new(&executable)).stdout, b"2\n");
    assert!(output_directory.join("libtopal-std.so.1").is_file());
    let inspected = run(Command::new(tools.directory.join("llvm-readobj"))
        .args(["--needed-libs", "--dynamic-table"])
        .arg(&executable));
    let inspected = String::from_utf8_lossy(&inspected.stdout);
    assert_eq!(
        inspected
            .matches("Shared library: [libtopal-std.so.1]")
            .count(),
        1
    );
    assert!(
        inspected.contains("Library runpath: [$ORIGIN]"),
        "{inspected}"
    );
    let metadata =
        NativeArtifactMetadata::decode(&fs::read(metadata_path(&executable)).unwrap()).unwrap();
    assert_eq!(
        metadata
            .dependencies
            .iter()
            .filter(|dependency| dependency.identity.starts_with("topal-library."))
            .map(|dependency| dependency.identity.as_str())
            .collect::<Vec<_>>(),
        ["topal-library.std.shared-object"]
    );

    let mismatched_directory = directory.join("mismatched");
    fs::create_dir_all(&mismatched_directory).unwrap();
    let mismatched_object = mismatched_directory.join("libtopal-std.so.1");
    fs::copy(&shared_object, &mismatched_object).unwrap();
    let manifest_path = shared_object.with_file_name("libtopal-std.so.1.topal-library.json");
    let mut manifest = fs::read_to_string(manifest_path).unwrap();
    let marker = "\"source_interface_sha256\": \"";
    let digest_start = manifest.find(marker).unwrap() + marker.len();
    let replacement = if &manifest[digest_start..=digest_start] == "0" {
        "1"
    } else {
        "0"
    };
    manifest.replace_range(digest_start..=digest_start, replacement);
    fs::write(
        mismatched_directory.join("libtopal-std.so.1.topal-library.json"),
        manifest,
    )
    .unwrap();
    let rejected_output = output_directory.join("rejected");
    let rejected = run(topalc().args([
        "--library-root",
        library_root.to_str().unwrap(),
        "--std-library",
        mismatched_object.to_str().unwrap(),
        "-o",
        rejected_output.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("source interface"));
    assert!(!rejected_output.exists());
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn diagnostic_controls_are_validated_erased_and_freestanding() {
    // TOPAL-COMPILER-DIAGNOSTIC-CONTROL-001, TOPAL-SYN-DIAG-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("diagnostic-controls");
    let source = directory.join("diagnostic-controls.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../../examples/language/diagnostic-controls.t"),
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
            "break diagnostic-controls.t:16",
            "-ex",
            "run",
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
    assert!(text.contains("diagnostic-controls.t:16"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn arbitrary_int_runtime_is_exact_and_self_contained() {
    // TOPAL-COMP-INT-001, TOPAL-COMPILER-INT-001, TOPAL-COMPILER-PLATFORM-001
    let directory = temporary("arbitrary-int");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/arbitrary-integer-arithmetic.t");
    let source_text = fs::read_to_string(&source).unwrap();
    let expected = Session::new()
        .evaluate_source_file(&source_text, &mut std::io::sink())
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
    assert_eq!(metadata.native_abi, "topal-native/6");

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
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn arbitrary_int_runtime_matches_limb_boundary_semantics() {
    // TOPAL-COMP-INT-001; TOPAL-NUM-NEG/ABS/ADD/SUB/MUL/COMPARE-001
    let directory = temporary("int-limb-boundaries");
    let source = directory.join("boundaries.t");
    let executable = directory.join("application");
    let source_text = "use language (version is v0.1)\n(4294967295 + 1, 4294967296 - 1, 18446744073709551615 + 1, 18446744073709551616 - 1, 4294967295 * 4294967295, (negate 4294967296) + 1, 4294967296 + (negate 1), (negate 4294967296) + (negate 1), (negate 4294967296) - (negate 1), absolute (negate 18446744073709551616), 7 - 7, (negate 9) < (negate 8), 18446744073709551616 > 4294967296)\n";
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
fn finite_exact_runtime_matches_large_gcd_and_division_semantics() {
    // TOPAL-COMP-EXACT-001; TOPAL-NUM-RATIONAL/DIV/MOD/POW/COMPARE-001
    let directory = temporary("finite-exact-large");
    let source = directory.join("large-exact.t");
    let executable = directory.join("application");
    let source_text = "use language (version is v0.1)\ncommon is 1234567890123456789012345678901234567890\nleft is Rational (common * 37, common * 41)\nright is Rational (common * 43, common * 47)\n(left, right, left + right, left - right, left * right, left / right, left ^ 5, common % 1000000007, common / 97, left < right, left <=> right)\n";
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
fn structural_comparison_runtime_matches_interpreter() {
    // TOPAL-COMP-STRUCTURAL-COMPARISON-001,
    // TOPAL-TYPE-EQUALITY-001, TOPAL-TYPE-ORDERING-001
    let directory = temporary("structural-comparison");
    let source = directory.join("structural-comparison.t");
    let executable = directory.join("application");
    let source_text = "use language (version is v0.1)\nleft is (1, 2)\nequal is (1, 2)\ngreater is (1, 3)\nrecord-left is (name is \"Ada\", score is 1)\nrecord-equal is (score is 1, name is \"Ada\")\nrecord-different is (name is \"Ada\", score is 2)\n(left < greater, greater > left, left <= equal, greater >= left, left <=> greater, equal <=> left, greater <=> left, record-left = record-equal, record-left != record-different)\n";
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
fn record_reconstruction_runtime_matches_interpreter() {
    // TOPAL-COMP-RECONSTRUCT-001, TOPAL-TYPE-RECONSTRUCT-001
    let directory = temporary("record-reconstruction");
    let source = directory.join("record-reconstruction.t");
    let executable = directory.join("application");
    let source_text = "use language (version is v0.1)\nperson is (name is \"Ada\", age is 36)\nupdated is person with (age is person age + 1)\n(person, updated, person age, updated name, updated age)\n";
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
fn finite_range_runtime_matches_exact_interpreter_semantics() {
    // TOPAL-COMP-RANGE-001; TOPAL-RANGE-BOUNDS/MEMBERSHIP/INTERSECTION/EMPTY/BOUND-001
    let directory = temporary("finite-ranges");
    let source = directory.join("ranges.t");
    let executable = directory.join("application");
    let source_text = "use language (version is v0.1)\nlarge is 123456789012345678901234567890\nintegers is (negate large) <..= large\nrationals is (Rational (large, 7)) .. (Rational (large * 3, 7))\npreserve is fn (value : Range Rational) -> Range Rational\n  value\nintersection is (preserve rationals) and ((Rational (large * 2, 7)) ..= (Rational (large * 4, 7)))\n(integers, 0 in integers, integers contains large, empty? (large .. large), range-lower intersection, range-upper intersection, range-lower-inclusive? intersection, range-upper-inclusive? intersection, intersection)\n";
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
fn exact_infinity_runtime_matches_interpreter_and_is_debuggable() {
    // TOPAL-NUM-INFINITY-001, TOPAL-RANGE-BOUNDS-001,
    // TOPAL-RANGE-INTERSECTION-001,
    // TOPAL-COMPILER-INFINITY-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("exact-infinities");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/infinity-values-and-ranges.t");
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
            "break infinity-values-and-ranges.t:16",
            "-ex",
            "run",
            "-ex",
            "print negative",
            "-ex",
            "print positive",
            "-ex",
            "print natural",
            "-ex",
            "print whole",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let debugged = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "$1 = -Infinity",
        "$2 = +Infinity",
        "$3 = +Infinity",
        "$4 = -Infinity ..= +Infinity",
    ] {
        assert!(debugged.contains(expected), "{debugged}");
    }

    let malformed = run(Command::new("gdb")
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
            "break infinity-values-and-ranges.t:16",
            "-ex",
            "run",
            "-ex",
            "set {long}((char*)positive + 8) = 1",
            "-ex",
            "print positive",
        ])
        .arg(&executable));
    assert!(
        malformed.status.success(),
        "{}",
        String::from_utf8_lossy(&malformed.stderr)
    );
    let malformed = String::from_utf8_lossy(&malformed.stdout);
    assert!(
        malformed.contains("$1 = <invalid Infinity length 1>"),
        "{malformed}"
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn rational_infinity_runtime_matches_interpreter_and_is_debuggable() {
    // TOPAL-NUM-INFINITY-001, TOPAL-RANGE-RATIONAL-001,
    // TOPAL-COMPILER-INFINITY-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("rational-infinities");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/rational-infinity-values-and-ranges.t");
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
            "break rational-infinity-values-and-ranges.t:17",
            "-ex",
            "run",
            "-ex",
            "print negative",
            "-ex",
            "print positive",
            "-ex",
            "print whole",
            "-ex",
            "print mixed",
            "-ex",
            "set positive->denominator = finite->numerator",
            "-ex",
            "print positive",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let debugged = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "$1 = -Infinity",
        "$2 = +Infinity",
        "$3 = -Infinity ..= +Infinity",
        "$4 = Rational ( -1, 1 ) ..= +Infinity",
        "$5 = <invalid Rational Infinity denominator 3>",
    ] {
        assert!(debugged.contains(expected), "{debugged}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn infinity_arithmetic_runtime_matches_interpreter_and_is_debuggable() {
    // TOPAL-NUM-INFINITY-ARITHMETIC-001, TOPAL-COMPILER-INFINITY-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("infinity-arithmetic");
    let executable = directory.join("application");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/infinity-arithmetic.t");
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
            "break infinity-arithmetic.t:14",
            "-ex",
            "run",
            "-ex",
            "print integerNegative",
            "-ex",
            "print integerPositive",
            "-ex",
            "print rationalPositiveResult",
            "-ex",
            "next",
            "-ex",
            "print rationalNegativeResult",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let debugged = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "$1 = -Infinity",
        "$2 = +Infinity",
        "$3 = +Infinity",
        "$4 = -Infinity",
    ] {
        assert!(debugged.contains(expected), "{debugged}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn infinity_private_boundaries_match_the_interpreter_and_are_debuggable() {
    // TOPAL-NUM-INFINITY-001, TOPAL-NUM-INFINITY-ARITHMETIC-001,
    // TOPAL-COMPILER-INFINITY-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("infinity-private-boundaries");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/infinity-private-boundaries.t");
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
            "break infinity-private-boundaries.t:8",
            "-ex",
            "break infinity-private-boundaries.t:11",
            "-ex",
            "break infinity-private-boundaries.t:14",
            "-ex",
            "break infinity-private-boundaries.t:20",
            "-ex",
            "break infinity-private-boundaries.t:23",
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
            "print value",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "continue",
            "-ex",
            "print value",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let debugged = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "$1 = +Infinity",
        "$2 = +Infinity",
        "$3 = -Infinity",
        "$4 = {_0 = +Infinity, _1 = -Infinity}",
        "$5 = {integer = +Infinity, ratio = -Infinity}",
    ] {
        assert!(debugged.contains(expected), "{debugged}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn dynamic_infinity_results_match_the_interpreter_and_are_debuggable() {
    // TOPAL-NUM-INFINITY-ARITHMETIC-001, TOPAL-TYPE-RESULT-001,
    // TOPAL-COMPILER-INFINITY-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("dynamic-infinity-results");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/dynamic-infinity-results.t");
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
            "break dynamic-infinity-results.t:20",
            "-ex",
            "run",
            "-ex",
            "next",
            "-ex",
            "next",
            "-ex",
            "print 'int-success'",
            "-ex",
            "next",
            "-ex",
            "print 'int-failure'",
            "-ex",
            "next",
            "-ex",
            "print 'rational-success'",
            "-ex",
            "next",
            "-ex",
            "print 'rational-failure'",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let debugged = String::from_utf8_lossy(&debugged.stdout);
    for expected in [
        "$1 = -Infinity",
        "$2 = Error ( domain is root.*(Int,Int), code is indeterminate )",
        "$3 = +Infinity",
        "$4 = Error ( domain is root.*(Rational,Rational), code is indeterminate )",
    ] {
        assert!(debugged.contains(expected), "{debugged}");
    }
}

#[test]
fn unsupported_source_and_missing_llvm_do_not_publish_outputs() {
    // TOPAL-COMPILER-SUBSET-001, TOPAL-COMPILER-LLVM-001
    let directory = temporary("atomic-failure");
    let unsupported = directory.join("unsupported.t");
    fs::write(&unsupported, "use language (version is v0.1)\n1 / 0\n").unwrap();
    let first_output = directory.join("unsupported");
    let result = run(topalc().args([
        "-o",
        first_output.to_str().unwrap(),
        unsupported.to_str().unwrap(),
    ]));
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("E-DIVISION-BY-ZERO"));
    assert!(!first_output.exists());
    assert!(!metadata_path(&first_output).exists());

    let supported = directory.join("supported.t");
    fs::write(&supported, "use language (version is v0.1)\n42\n").unwrap();
    let missing_tools = directory.join("missing-tools");
    fs::create_dir_all(&missing_tools).unwrap();
    let second_output = directory.join("missing-llvm");
    let result = run(topalc().args([
        "--llvm-tools",
        missing_tools.to_str().unwrap(),
        "-o",
        second_output.to_str().unwrap(),
        supported.to_str().unwrap(),
    ]));
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("LLVM tool"));
    assert!(!second_output.exists());
    assert!(!metadata_path(&second_output).exists());
}

#[test]
fn emits_verifiable_llvm_ir_and_object() {
    // TOPAL-COMPILER-LLVM-001, TOPAL-COMPILER-TARGET-001
    let directory = temporary("emission-kinds");
    let source = directory.join("source.t");
    fs::write(&source, "use language (version is v0.1)\n6 * 7\n").unwrap();
    for (kind, extension) in [("llvm-ir", "ll"), ("object", "o")] {
        let output = directory.join(format!("output.{extension}"));
        let result = run(topalc().args([
            "--emit",
            kind,
            "-o",
            output.to_str().unwrap(),
            source.to_str().unwrap(),
        ]));
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(output.is_file());
        assert!(NativeArtifactMetadata::decode(&fs::read(metadata_path(&output)).unwrap()).is_ok());
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn executable_is_static_pie_without_foreign_runtime_or_loader() {
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-RESULT-001
    let directory = temporary("freestanding");
    let source = directory.join("source.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\n1.0 divide 0.0\n",
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
    let inspected = run(Command::new(tools.directory.join("llvm-readobj"))
        .args([
            "--file-headers",
            "--program-headers",
            "--dynamic-table",
            "--needed-libs",
            "--relocations",
        ])
        .arg(&executable));
    assert!(inspected.status.success());
    let text = String::from_utf8_lossy(&inspected.stdout);
    assert!(text.contains("Format: elf64-x86-64"));
    assert!(text.contains("Type: SharedObject"));
    assert!(!text.contains("PT_INTERP"));
    assert!(text.contains("NeededLibraries [\n]"));
    assert!(text.contains("Relocations [\n]"));
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_observes_source_breakpoint_stack_and_local_value() {
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb");
    let source = directory.join("debug-source.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nsubtract is fn (left : Int, right : Int) -> Int\n  difference is left - right\n  return difference\n123456789012345678901234567890 subtract 98765432109876543210987654321\n",
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
            "break debug-source.t:3",
            "-ex",
            "run",
            "-ex",
            "print left",
            "-ex",
            "print right",
            "-ex",
            "next",
            "-ex",
            "print difference",
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
    assert!(text.contains("Breakpoint 1, topal.fn.subtract.0"), "{text}");
    assert!(text.contains("debug-source.t"), "{text}");
    assert!(
        text.contains("$1 = 123456789012345678901234567890"),
        "{text}"
    );
    assert!(
        text.contains("$2 = 98765432109876543210987654321"),
        "{text}"
    );
    assert!(
        text.contains("$3 = 24691356902469135690246913569"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_observes_innermost_lexical_block_binding() {
    // TOPAL-EXEC-BLOCK-001, TOPAL-COMP-BLOCK-001, TOPAL-COMPILER-BLOCK-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-lexical-block");
    let source = directory.join("lexical-block-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nvalue is 39 + 1\nshadow is {\n  value is value + 1\n  value + 1\n}\n(shadow, value)\n",
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
            "break lexical-block-debug.t:5",
            "-ex",
            "run",
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
    assert!(text.contains("Breakpoint 1, topal.main"), "{text}");
    assert!(text.contains("$1 = 41"), "{text}");
}
