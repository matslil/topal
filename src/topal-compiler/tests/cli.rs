use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use topal_compiler::{LlvmTools, NATIVE_ABI, NativeArtifactMetadata, metadata_path};
use topal_language::Session;

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
        "--target",
        "x86_64-unknown-linux-gnu",
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
    assert_eq!(metadata.native_slices[0].kind, "executable");
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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_canonical_rational_parameters_and_locals() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-EXACT-001
    let directory = temporary("gdb-rational");
    let source = directory.join("rational-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nadjust is fn (value : Rational) -> Rational\n  result is value + 0.5\n  return result\n1.25 adjust\n",
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
            "break rational-debug.t:3",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "next",
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
    assert!(text.contains("$1 = Rational ( 5, 4 )"), "{text}");
    assert!(text.contains("$2 = Rational ( 7, 4 )"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_nominal_enum_parameters_and_locals() {
    // TOPAL-COMP-ENUM-001, TOPAL-COMP-DEBUG-001, TOPAL-COMPILER-ENUM-001
    let directory = temporary("gdb-enum");
    let source = directory.join("enum-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nColor is Enum (Red, Green, Blue)\nretain is fn (value : Color) -> Color\n  result is value\n  return result\nretain Green\n",
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
    let debugged = run(Command::new("gdb")
        .args([
            "-q",
            "--batch",
            "-ex",
            "set debuginfod enabled off",
            "-ex",
            "set disable-randomization off",
            "-ex",
            "break enum-debug.t:4",
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
    assert!(text.contains("$1 = Green"), "{text}");
    assert!(text.contains("$2 = Green"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_exact_range_parameters_and_locals() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-RANGE-001
    let directory = temporary("gdb-range");
    let source = directory.join("range-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nnarrow is fn (integers : Range Int, rationals : Range Rational) -> Range Rational\n  _ is integers contains 5\n  result is rationals and (1.0 <..= 2.0)\n  return result\n(0 <..= 10) narrow (0.5 ..= 2.5)\n",
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
            "break range-debug.t:3",
            "-ex",
            "run",
            "-ex",
            "print integers",
            "-ex",
            "print rationals",
            "-ex",
            "next",
            "-ex",
            "next",
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
    assert!(text.contains("$1 = 0 <..= 10"), "{text}");
    assert!(
        text.contains("$2 = Rational ( 1, 2 ) ..= Rational ( 5, 2 )"),
        "{text}"
    );
    assert!(
        text.contains("$3 = Rational ( 1, 1 ) <..= Rational ( 2, 1 )"),
        "{text}"
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_structured_arithmetic_result_parameters_and_locals() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-RESULT-001
    let directory = temporary("gdb-result");
    let source = directory.join("result-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nretain is fn (value : Result (Rational, lang arithmetic ArithmeticErrorCode)) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  result is value\n  result\n(1.0 divide 0.0) retain\n",
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
            "break result-debug.t:6",
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
    let expected = "Error ( domain is root./(Rational,Rational), code is division-by-zero )";
    assert!(text.contains(&format!("$1 = {expected}")), "{text}");
    assert!(text.contains(&format!("$2 = {expected}")), "{text}");
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
