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
fn diagnostic_controls_are_validated_erased_and_freestanding() {
    // TOPAL-COMPILER-DIAGNOSTIC-CONTROL-001, TOPAL-SYN-DIAG-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("diagnostic-controls");
    let source = directory.join("diagnostic-controls.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/diagnostic-controls.t"),
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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn lexical_block_return_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-lexical-block-return");
    let source = directory.join("function-return-from-block.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-from-block.t"),
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
    let ir_text = fs::read_to_string(&ir).unwrap();
    assert!(ir_text.contains("define internal fastcc ptr @topal.fn.answer"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-from-block.t:8",
            "-ex",
            "run",
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
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn return_operand_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-OPERAND-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERAND-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-operand-block");
    let source = directory.join("function-return-block-operand.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-block-operand.t"),
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
    let ir_text = fs::read_to_string(&ir).unwrap();
    assert!(ir_text.contains("define internal fastcc ptr @topal.fn.answer"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-block-operand.t:8",
            "-ex",
            "run",
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
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers native output, exact IR, DWARF, and both GDB stops.
fn operator_operand_block_exits_are_ordered_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-OPERATOR-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERATOR-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-operator-operand");
    let source = directory.join("function-return-operator-operand.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-operator-operand.t"),
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
    assert_eq!(executed.stdout, b"(42, 43)\n");

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
    let ir_text = fs::read_to_string(&ir).unwrap();
    let right = ir_text
        .split_once("define internal fastcc ptr @topal.fn.right_2dexit")
        .unwrap()
        .1
        .split_once("define internal fastcc ptr @topal.fn.left_2dexit")
        .unwrap()
        .0;
    assert_eq!(
        right.matches("call fastcc ptr @topal.fn.preceding").count(),
        1
    );
    assert_eq!(right.matches("call ptr @topal.runtime.int.add").count(), 1);
    let left = ir_text
        .split_once("define internal fastcc ptr @topal.fn.left_2dexit")
        .unwrap()
        .1
        .split_once("define internal void @topal.main")
        .unwrap()
        .0;
    assert_eq!(left.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 2);
    assert!(!ir_text.contains("missing"));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-operator-operand.t:9",
            "-ex",
            "break function-return-operator-operand.t:12",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("topal.fn.right_2dexit"), "{text}");
    assert!(text.contains("topal.fn.left_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers native output, exact IR, DWARF, and both product forms.
fn product_field_block_exits_are_ordered_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-PRODUCT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-PRODUCT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-product-field");
    let source = directory.join("function-return-product-field.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-product-field.t"),
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
    assert_eq!(executed.stdout, b"(42, 43)\n");

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
    let ir_text = fs::read_to_string(&ir).unwrap();
    let tuple = ir_text
        .split_once("define internal fastcc ptr @topal.fn.tuple_2dexit")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    let record = ir_text
        .split_once("define internal fastcc ptr @topal.fn.record_2dexit")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    for function in [tuple, record] {
        assert_eq!(
            function
                .matches("call fastcc ptr @topal.fn.preceding")
                .count(),
            2
        );
        assert_eq!(
            function.matches("call ptr @topal.runtime.int.add").count(),
            1
        );
        assert!(!function.contains("insertvalue"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 2);
    assert!(!ir_text.contains("missing"));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-product-field.t:9",
            "-ex",
            "break function-return-product-field.t:12",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("topal.fn.tuple_2dexit"), "{text}");
    assert!(text.contains("topal.fn.record_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers native output, exact IR, DWARF, and three GDB stops.
fn named_call_argument_block_exits_are_ordered_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMP-LEXICAL-RETURN-CALL-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CALL-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-call-argument");
    let source = directory.join("function-return-call-argument.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-call-argument.t"),
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
    assert_eq!(executed.stdout, b"(42, 43, 44)\n");

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
    let ir_text = fs::read_to_string(&ir).unwrap();
    let body = |symbol: &str| {
        ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0
    };
    let right = body("define internal fastcc ptr @topal.fn.right_2dexit");
    let left = body("define internal fastcc ptr @topal.fn.left_2dexit");
    let unary = body("define internal fastcc ptr @topal.fn.unary_2dexit");
    assert_eq!(
        right.matches("call fastcc ptr @topal.fn.preceding").count(),
        1
    );
    for function in [right, left, unary] {
        assert_eq!(
            function.matches("call ptr @topal.runtime.int.add").count(),
            1
        );
        assert!(!function.contains("call fastcc ptr @topal.fn.combine"));
        assert!(!function.contains("call fastcc ptr @topal.fn.identity"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 3);
    assert!(!ir_text.contains("missing"));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-call-argument.t:13",
            "-ex",
            "break function-return-call-argument.t:16",
            "-ex",
            "break function-return-call-argument.t:19",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("$3 = 41"), "{text}");
    assert!(text.contains("topal.fn.right_2dexit"), "{text}");
    assert!(text.contains("topal.fn.left_2dexit"), "{text}");
    assert!(text.contains("topal.fn.unary_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn optional_constructor_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-COMP-LEXICAL-RETURN-OPTIONAL-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-optional-constructor");
    let source = directory.join("function-return-optional-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-optional-constructor.t"),
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
    let ir_text = fs::read_to_string(&ir).unwrap();
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert!(!answer.contains("call ptr @topal.runtime.optional.some"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-optional-constructor.t:7",
            "-ex",
            "run",
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
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and five constructor-source stops.
fn strict_unary_constructor_argument_block_exits_are_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-UNION-001,
    // TOPAL-COMP-LEXICAL-RETURN-CONSTRUCTOR-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-unary-constructor");
    let source = directory.join("function-return-unary-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-unary-constructor.t"),
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
    assert_eq!(executed.stdout, b"(42, 43, 44, 45, 46)\n");

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
    let ir_text = fs::read_to_string(&ir).unwrap();
    let body = |symbol: &str| {
        ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0
    };
    for symbol in [
        "define internal fastcc ptr @topal.fn.string_2dexit",
        "define internal fastcc ptr @topal.fn.int_2dexit",
        "define internal fastcc ptr @topal.fn.nat_2dexit",
        "define internal fastcc ptr @topal.fn.rational_2dexit",
        "define internal fastcc ptr @topal.fn.union_2dexit",
    ] {
        let function = body(symbol);
        assert_eq!(
            function.matches("call ptr @topal.runtime.int.add").count(),
            1
        );
        assert!(!function.contains("@topal.runtime.optional.some"));
        assert!(!function.contains("@topal.runtime.rational.construct"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 5);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

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
            "break function-return-unary-constructor.t:10",
            "-ex",
            "break function-return-unary-constructor.t:13",
            "-ex",
            "break function-return-unary-constructor.t:16",
            "-ex",
            "break function-return-unary-constructor.t:19",
            "-ex",
            "break function-return-unary-constructor.t:22",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
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
    for index in 1..=5 {
        assert!(text.contains(&format!("${index} = 41")), "{text}");
    }
    for symbol in [
        "topal.fn.string_2dexit",
        "topal.fn.int_2dexit",
        "topal.fn.nat_2dexit",
        "topal.fn.rational_2dexit",
        "topal.fn.union_2dexit",
        "topal.main",
    ] {
        assert!(text.contains(symbol), "{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn positional_variant_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-VARIANT-001,
    // TOPAL-COMP-LEXICAL-RETURN-VARIANT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-VARIANT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-variant-constructor");
    let source = directory.join("function-return-variant-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-variant-constructor.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-variant-constructor.t:9",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn character_constructor_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-STRING-CHARACTER-CLASSIFIER-001,
    // TOPAL-COMP-LEXICAL-RETURN-CHARACTER-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CHARACTER-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-character-constructor");
    let source = directory.join("function-return-character-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-character-constructor.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-character-constructor.t:7",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn named_constraint_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMP-LEXICAL-RETURN-CONSTRAINT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRAINT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-constraint-constructor");
    let source = directory.join("function-return-constraint-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-constraint-constructor.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("icmp"));
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-constraint-constructor.t:9",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn named_modular_argument_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-NUM-MODULAR-CONSTRUCT-001,
    // TOPAL-COMP-LEXICAL-RETURN-MODULAR-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-modular-constructor");
    let source = directory.join("function-return-modular-constructor.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-modular-constructor.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("icmp"));
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-modular-constructor.t:9",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn modular_reduction_operand_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-NUM-MODULAR-REDUCE-001,
    // TOPAL-COMP-LEXICAL-RETURN-MODULAR-REDUCE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-modular-reduction");
    let source = directory.join("function-return-modular-reduction.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-modular-reduction.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.runtime.int.subtract"));
    assert!(!answer.contains("@topal.runtime.int.modulo"));
    assert!(!answer.contains("@topal.runtime.result."));
    assert!(!answer.contains("icmp"));
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-modular-reduction.t:9",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn list_collect_source_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COLLECTION-COLLECT-LIST-001,
    // TOPAL-COMP-LEXICAL-RETURN-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COLLECT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-list-collect");
    let source = directory.join("function-return-list-collect.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-list-collect.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.runtime.list."));
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-list-collect.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and both unordered collectors.
fn unordered_collect_source_block_exits_are_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-SET-COLLECT-001,
    // TOPAL-BAG-COLLECT-001, TOPAL-COMP-LEXICAL-RETURN-UNORDERED-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-unordered-collect");
    let source = directory.join("function-return-unordered-collect.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-unordered-collect.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let body = |symbol: &str| {
        ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0
    };
    for symbol in [
        "define internal fastcc ptr @topal.fn.set_2dexit",
        "define internal fastcc ptr @topal.fn.bag_2dexit",
    ] {
        let function = body(symbol);
        assert_eq!(
            function.matches("call ptr @topal.runtime.int.add").count(),
            1
        );
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert!(!function.contains("@topal.platform.allocate"));
        assert!(!function.contains("insertvalue"));
        assert!(!function.contains("extractvalue"));
    }
    assert!(!ir_text.contains("@topal.runtime.container."));
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 2);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-unordered-collect.t:8",
            "-ex",
            "break function-return-unordered-collect.t:11",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 40"), "{text}");
    assert!(text.contains("$2 = -1"), "{text}");
    assert!(text.contains("topal.fn.set_2dexit"), "{text}");
    assert!(text.contains("topal.fn.bag_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn map_collect_source_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-MAP-COLLECT-001,
    // TOPAL-COMP-LEXICAL-RETURN-MAP-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-map-collect");
    let source = directory.join("function-return-map-collect.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-map-collect.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let answer = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    assert_eq!(answer.matches("call ptr @topal.runtime.int.add").count(), 1);
    assert_eq!(answer.matches("ret ptr").count(), 1);
    assert!(!answer.contains("@topal.platform.allocate"));
    assert!(!answer.contains("insertvalue"));
    assert!(!answer.contains("extractvalue"));
    assert!(!ir_text.contains("@topal.runtime.container."));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-map-collect.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact IR, DWARF, and both infix targets.
fn infix_collect_source_block_exits_are_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-ARRAY-COLLECT-001,
    // TOPAL-COLLECTION-COLLECT-STRING-001,
    // TOPAL-COMP-LEXICAL-RETURN-INFIX-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-INFIX-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-infix-collect");
    let source = directory.join("function-return-infix-collect.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-infix-collect.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let body = |symbol: &str| {
        ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0
    };
    for symbol in [
        "define internal fastcc ptr @topal.fn.array_2dexit",
        "define internal fastcc ptr @topal.fn.string_2dexit",
    ] {
        let function = body(symbol);
        assert_eq!(
            function.matches("call ptr @topal.runtime.int.add").count(),
            1
        );
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert!(!function.contains("@topal.platform.allocate"));
        assert!(!function.contains("insertvalue"));
        assert!(!function.contains("extractvalue"));
    }
    assert!(!ir_text.contains("@topal.runtime.container."));
    assert!(!ir_text.contains("@topal.runtime.string.collect"));
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 2);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-infix-collect.t:8",
            "-ex",
            "break function-return-infix-collect.t:11",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 40"), "{text}");
    assert!(text.contains("$2 = -1"), "{text}");
    assert!(text.contains("topal.fn.array_2dexit"), "{text}");
    assert!(text.contains("topal.fn.string_2dexit"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn boolean_decision_subject_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMP-LEXICAL-RETURN-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-decision-subject");
    let source = directory.join("function-return-decision-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-decision-subject.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let function = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
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
    assert!(!function.contains("br i1"));
    assert!(!function.contains("select i1"));
    assert!(!function.contains(" phi "));
    assert!(!function.contains("@topal.platform.allocate"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-decision-subject.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn exhaustive_boolean_decision_subject_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMP-LEXICAL-RETURN-EXHAUSTIVE-BOOLEAN-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-EXHAUSTIVE-BOOLEAN-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-exhaustive-boolean-subject");
    let source = directory.join("function-return-exhaustive-boolean-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-exhaustive-boolean-subject.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let function = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
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
    assert!(!function.contains("br i1"));
    assert!(!function.contains("select i1"));
    assert!(!function.contains(" phi "));
    assert!(!function.contains("@topal.platform.allocate"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-exhaustive-boolean-subject.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn comparison_decision_subject_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-COMPARISON-001,
    // TOPAL-COMP-LEXICAL-RETURN-COMPARISON-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-comparison-decision-subject");
    let source = directory.join("function-return-comparison-decision-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-comparison-decision-subject.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let function = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
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
    assert!(!function.contains("call i32 @topal.runtime.int.compare"));
    assert!(!function.contains("icmp"));
    assert!(!function.contains("br i1"));
    assert!(!function.contains("select i1"));
    assert!(!function.contains(" phi "));
    assert!(!function.contains("@topal.platform.allocate"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-comparison-decision-subject.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn fallback_decision_subject_block_exit_is_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001,
    // TOPAL-COMP-LEXICAL-RETURN-FALLBACK-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-FALLBACK-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-fallback-decision-subject");
    let source = directory.join("function-return-fallback-decision-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-fallback-decision-subject.t"),
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

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    let function = ir_text
        .split_once("define internal fastcc ptr @topal.fn.answer")
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
    assert!(!function.contains("call i32 @topal.runtime.int.compare"));
    assert!(!function.contains("icmp"));
    assert!(!function.contains("br i1"));
    assert!(!function.contains("switch "));
    assert!(!function.contains("select i1"));
    assert!(!function.contains(" phi "));
    assert!(!function.contains("@topal.platform.allocate"));
    assert!(ir_text.contains("!DILexicalBlock("));
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-fallback-decision-subject.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 41"), "{text}");
    assert!(text.contains("topal.fn.answer"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn complete_decision_subject_block_exits_are_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-OPTIONAL-001,
    // TOPAL-DECISION-RESULT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-COMP-LEXICAL-RETURN-COMPLETE-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPLETE-DECISION-SUBJECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-complete-decision-subject");
    let source = directory.join("function-return-complete-decision-subject.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-complete-decision-subject.t"),
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
    assert_eq!(executed.stdout, b"(40, 41, 42)\n");

    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    for symbol in [
        "define internal fastcc ptr @topal.fn.optional_2dexit",
        "define internal fastcc ptr @topal.fn.result_2dexit",
        "define internal fastcc ptr @topal.fn.list_2dexit",
    ] {
        assert_direct_return_without_classifier(&ir_text, symbol);
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 3);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));

    let tools = LlvmTools::discover(None).unwrap();
    let dwarf_tool = tools.directory.join("llvm-dwarfdump");
    if dwarf_tool.is_file() {
        let dwarf = run(Command::new(dwarf_tool).arg("--verify").arg(&executable));
        assert!(dwarf.status.success());
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
            "break function-return-complete-decision-subject.t:8",
            "-ex",
            "break function-return-complete-decision-subject.t:14",
            "-ex",
            "break function-return-complete-decision-subject.t:20",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = 39"), "{text}");
    assert!(text.contains("$2 = 40"), "{text}");
    assert!(text.contains("$3 = 41"), "{text}");
    for symbol in [
        "topal.fn.optional_2dexit",
        "topal.fn.result_2dexit",
        "topal.fn.list_2dexit",
        "topal.main",
    ] {
        assert!(text.contains(symbol), "{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn boolean_decision_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMP-LEXICAL-RETURN-BOOLEAN-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-BOOLEAN-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-boolean-decision-actions");
    let source = directory.join("function-return-boolean-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-return-boolean-decision-actions.t"),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(40, 41)\n");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    for symbol in ["@topal.fn.choose.0", "@topal.fn.choose.1"] {
        let function = ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(function.matches("br i1").count(), 1);
        assert_eq!(function.matches("br label").count(), 2);
        assert_eq!(function.matches("phi ptr").count(), 1);
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert!(!function.contains("select i1"));
        assert!(!function.contains("switch "));
        assert!(!function.contains("@topal.platform.allocate"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 4);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));
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
            "break function-return-boolean-decision-actions.t:9",
            "-ex",
            "break function-return-boolean-decision-actions.t:10",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("$1 = false"), "{text}");
    assert!(text.contains("$2 = true"), "{text}");
    assert!(text.contains("topal.fn.choose.0"), "{text}");
    assert!(text.contains("topal.fn.choose.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn comparison_value_action_exits_are_joined_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-ENUM-001,
    // TOPAL-COMP-LEXICAL-RETURN-COMPARISON-VALUE-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-VALUE-DECISION-ACTIONS-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-return-comparison-value-decision-actions");
    let source = directory.join("function-return-comparison-value-decision-actions.t");
    let executable = directory.join("application");
    let ir = directory.join("application.ll");
    fs::write(
        &source,
        include_str!(
            "../../../examples/language/function-return-comparison-value-decision-actions.t"
        ),
    )
    .unwrap();
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(compiled.status.success());
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(40, 41, 42)\n");
    let emitted = run(topalc().args([
        "--emit",
        "llvm-ir",
        "-o",
        ir.to_str().unwrap(),
        source.to_str().unwrap(),
    ]));
    assert!(emitted.status.success());
    let ir_text = fs::read_to_string(&ir).unwrap();
    for symbol in [
        "@topal.fn.choose.0",
        "@topal.fn.choose.1",
        "@topal.fn.choose.2",
    ] {
        let function = ir_text
            .split_once(symbol)
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(function.matches("switch i32").count(), 1);
        assert_eq!(function.matches("br label").count(), 3);
        assert_eq!(function.matches("phi ptr").count(), 1);
        assert_eq!(function.matches("ret ptr").count(), 1);
        assert!(!function.contains("select i1"));
        assert!(!function.contains("@topal.platform.allocate"));
    }
    assert!(ir_text.matches("!DILexicalBlock(").count() >= 9);
    assert!(!ir_text.contains("@printf"));
    assert!(!ir_text.contains("@malloc"));
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
            "break function-return-comparison-value-decision-actions.t:9",
            "-ex",
            "break function-return-comparison-value-decision-actions.t:10",
            "-ex",
            "break function-return-comparison-value-decision-actions.t:11",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
            "-ex",
            "continue",
            "-ex",
            "print value",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(debugged.status.success());
    let text = String::from_utf8_lossy(&debugged.stdout);
    for expected in ["$1 = Less", "$2 = Equal", "$3 = Greater"] {
        assert!(text.contains(expected), "{text}");
    }
    for symbol in [
        "topal.fn.choose.0",
        "topal.fn.choose.1",
        "topal.fn.choose.2",
        "topal.main",
    ] {
        assert!(text.contains(symbol), "{text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_completed_evidence_by_source_name() {
    // TOPAL-EXEC-COMPLETED-001, TOPAL-COMP-COMPLETED-001,
    // TOPAL-COMPILER-COMPLETED-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-completed");
    let source = directory.join("completed-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nfinish is fn () -> Completed\n  result is Completed\n  return result\nfinish ()\n",
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
            "break completed-debug.t:4",
            "-ex",
            "run",
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
    assert!(text.contains("Breakpoint 1, topal.fn.finish.0"), "{text}");
    assert!(text.contains("$1 = Completed"), "{text}");
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
fn layout_policy_values_are_freestanding_and_debuggable() {
    // TOPAL-LAYOUT-ENDIAN-001, TOPAL-LAYOUT-ACCESS-001,
    // TOPAL-LAYOUT-BIT-ORDER-001, TOPAL-LAYOUT-PACKING-001,
    // TOPAL-LAYOUT-FIELD-ORDER-001, TOPAL-LAYOUT-PAYLOAD-PLACEMENT-001,
    // TOPAL-LAYOUT-ABSENCE-POLICY-001,
    // TOPAL-COMPILER-LAYOUT-POLICY-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-layout-policy-values");
    let source = directory.join("layout-policy-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nendian is Big\naccess is Reserved\nbits is LeastSignificantFirst\npacking is Packed\nfields is Declared\npayload is Overlay\nabsence is NoTerminator\n(endian, access, bits, packing, fields, payload, absence)\n",
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
        b"(Big, Reserved, LeastSignificantFirst, Packed, Declared, Overlay, NoTerminator)\n"
    );

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
            "break layout-policy-values.t:9",
            "-ex",
            "run",
            "-ex",
            "whatis endian",
            "-ex",
            "print endian",
            "-ex",
            "whatis access",
            "-ex",
            "print access",
            "-ex",
            "whatis bits",
            "-ex",
            "print bits",
            "-ex",
            "whatis packing",
            "-ex",
            "print packing",
            "-ex",
            "whatis fields",
            "-ex",
            "print fields",
            "-ex",
            "whatis payload",
            "-ex",
            "print payload",
            "-ex",
            "whatis absence",
            "-ex",
            "print absence",
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
        "type = enum Endian",
        "$1 = Big",
        "type = enum Access",
        "$2 = Reserved",
        "type = enum BitOrder",
        "$3 = LeastSignificantFirst",
        "type = enum Packing",
        "$4 = Packed",
        "type = enum FieldOrder",
        "$5 = Declared",
        "type = enum PayloadPlacement",
        "$6 = Overlay",
        "type = enum LayoutPolicy",
        "$7 = NoTerminator",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn generator_error_code_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-ERROR-CODE-001,
    // TOPAL-COMPILER-GENERATOR-ERROR-CODE-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-generator-error-code");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/generator-error-codes.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"(generator-closed, true)\n");
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
            "break generator-error-codes.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis closed",
            "-ex",
            "print closed",
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
        "type = enum lang generator GeneratorErrorCode",
        "$1 = generator-closed",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn lazy_iterate_values_are_freestanding_linear_and_debuggable() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-CONSTRUCT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-lazy-iterate-values");
    let cases = [
        ("iterate-generator.t", "numbers"),
        ("iterate-take-while.t", "digits"),
    ];
    for (filename, binding) in cases {
        let executable = directory.join(filename.trim_end_matches(".t"));
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/language")
            .join(filename);
        let compiled =
            run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let executed = run(&mut Command::new(&executable));
        assert!(executed.status.success());
        assert_eq!(executed.stdout, b"<Generator Int Unit Unit>\n");
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
                &format!("break {filename}:8"),
                "-ex",
                "run",
                "-ex",
                &format!("whatis {binding}"),
                "-ex",
                &format!("print {binding}"),
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
            "type = enum Generator Int Unit Unit",
            "$1 = <Generator Int Unit Unit>",
            "topal.main",
        ] {
            assert!(text.contains(expected), "missing {expected:?}: {text}");
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn bounded_iterate_collection_is_freestanding_ordered_and_debuggable() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-GENERATOR-COLLECT-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-bounded-iterate-collection");
    let executable = directory.join("application");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/generated-collect.t");
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
        b"Entry ( 0, Entry ( 1, Entry ( 2, Entry ( 3, Entry ( 4, Empty ) ) ) ) )\n"
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
            "break topal.platform.write_all",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis digits",
            "-ex",
            "print digits",
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
            "$1 = Entry ( 0, Entry ( 1, Entry ( 2, Entry ( 3, Entry ( 4, Empty ) ) ) ) )"
        ),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn bounded_iterate_foreach_is_freestanding_ordered_and_debuggable() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-GENERATOR-ITERATE-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-FOREACH-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-bounded-iterate-foreach");
    let executable = directory.join("application");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/generated-foreach.t");
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
            "break topal.runtime.int.add",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis digit",
            "-ex",
            "print digit",
            "-ex",
            "disable 1",
            "-ex",
            "break topal.platform.write_all",
            "-ex",
            "continue",
            "-ex",
            "up",
            "-ex",
            "whatis completed",
            "-ex",
            "print completed",
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
        "type = Int",
        "$1 = 0",
        "type = Unit",
        "$2 = ()",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn lazy_unfold_value_is_freestanding_and_debuggable_without_step_execution() {
    // TOPAL-GENERATOR-UNFOLD-001,
    // TOPAL-COMPILER-GENERATOR-UNFOLD-CONSTRUCT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-lazy-unfold-value");
    let executable = directory.join("application");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language/unfold-generator.t");
    let compiled =
        run(topalc().args(["-o", executable.to_str().unwrap(), source.to_str().unwrap()]));
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = run(&mut Command::new(&executable));
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"<Generator Int Unit Unit>\n");
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
        "type = enum Generator Int Unit Unit",
        "$1 = <Generator Int Unit Unit>",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn finite_unfold_collection_is_freestanding_ordered_and_debuggable() {
    // TOPAL-GENERATOR-UNFOLD-001, TOPAL-GENERATOR-UNFOLD-COLLECT-001,
    // TOPAL-COMPILER-GENERATOR-UNFOLD-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-finite-unfold-collection");
    let source = directory.join("unfold-collect-debug.t");
    let executable = directory.join("application");
    let shared = include_str!("../../../examples/language/unfold-collect.t");
    fs::write(
        &source,
        shared.replace(
            "collect generated\n",
            "collected is collect generated\n(values, collected)\n",
        ),
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
        b"(Entry ( 4, Entry ( 5, Entry ( 6, Empty ) ) ), Entry ( 4, Entry ( 5, Entry ( 6, Empty ) ) ))\n"
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
            "break topal.platform.write_all",
            "-ex",
            "run",
            "-ex",
            "up",
            "-ex",
            "whatis collected",
            "-ex",
            "print collected",
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
        text.contains("$1 = Entry ( 4, Entry ( 5, Entry ( 6, Empty ) ) )"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
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
fn gdb_renders_optional_parameters_and_locals() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-OPTIONAL-001
    let directory = temporary("gdb-optional");
    let source = directory.join("optional-debug.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nobserve is fn (candidate : Optional String) -> Int\n  candidate\n    Some text then 1\n    None then 0\ninspect is fn (present : Optional Int, absent : Optional String) -> Optional Int\n  state is observe absent\n  result : Optional Int is present\n  return result\ninspect (Some 42, None String)\n",
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
            "break optional-debug.t:7",
            "-ex",
            "break optional-debug.t:9",
            "-ex",
            "run",
            "-ex",
            "print present",
            "-ex",
            "print absent",
            "-ex",
            "continue",
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
    assert!(text.contains("$1 = Some 42"), "{text}");
    assert!(text.contains("$2 = None"), "{text}");
    assert!(text.contains("$3 = Some 42"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_optional_rational_parameters() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-OPTIONAL-RATIONAL-001
    let directory = temporary("gdb-optional-rational");
    let source = directory.join("optional-rational-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/optional-rational-values.t"),
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
            "break optional-rational-values.t:8",
            "-ex",
            "run",
            "-ex",
            "print candidate",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("candidate=Some Rational ( 7, 2 )"), "{text}");
    assert!(text.contains("$1 = Some Rational ( 7, 2 )"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_character_parameters() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-CHARACTER-001
    let directory = temporary("gdb-character");
    let source = directory.join("character-classification.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/character-classification.t"),
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
            "break character-classification.t:8",
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
    assert!(text.contains("value=\"🙂\""), "{text}");
    assert!(text.contains("$1 = \"🙂\""), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_renders_optional_character_parameters() {
    // TOPAL-COMP-DEBUG-001, TOPAL-COMP-CHARACTER-OBSERVATION-001
    let directory = temporary("gdb-optional-character");
    let source = directory.join("string-character-at.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/string-character-at.t"),
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
            "break string-character-at.t:9",
            "-ex",
            "run",
            "-ex",
            "print candidate",
        ])
        .arg(&executable));
    assert!(
        debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&debugged.stderr)
    );
    let text = String::from_utf8_lossy(&debugged.stdout);
    assert!(text.contains("candidate=Some \"👩‍🔬\""), "{text}");
    assert!(text.contains("$1 = Some \"👩‍🔬\""), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn closed_string_character_collection_is_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-character-collection");
    let source = directory.join("string-character-collection.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        "use language (version is v0.1)\nreconstructed : String is characters \"a\u{301}👩‍🔬🇸🇪\" collect String\n_ is reconstructed = reconstructed\nreconstructed\n",
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
    assert_eq!(executed.stdout, "\"a\u{301}👩‍🔬🇸🇪\"\n".as_bytes());
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
            "break string-character-collection.t:3",
            "-ex",
            "run",
            "-ex",
            "whatis reconstructed",
            "-ex",
            "print reconstructed",
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
    for expected in ["type = String", "$1 = \"a\u{301}👩‍🔬🇸🇪\"", "topal.main"] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn transferred_string_character_generator_close_is_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-STRING-CHARACTERS-CLOSE-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-CLOSE-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-character-generator-close");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/string-character-generator-close.t");
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
            "break string-character-generator-close.t:9",
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
        "ignore",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn transferred_string_character_generator_traversal_is_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-character-generator-parameter");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/string-character-generator-parameter.t");
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
            "break string-character-generator-parameter.t:8",
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
        "consume",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn returned_string_character_generator_is_freestanding_and_debuggable() {
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-RESULT-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-string-character-generator-result");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/string-character-generator-result.t");
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
            "break string-character-generator-result.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis text",
            "-ex",
            "print text",
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
        "type = String",
        "$1 = \"a\u{301}👩‍🔬🇸🇪\"",
        "generate",
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
fn transferred_custom_generator_close_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-CLOSE-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-CLOSE-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-parameter-close");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-parameter-close.t");
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
            "break custom-generator-parameter-close.t:16",
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
        "ignore",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn transferred_custom_generator_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-function-parameter");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-function-parameter.t");
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
            "break custom-generator-function-parameter.t:16",
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
        "consume",
        "topal.main",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn transferred_custom_generator_character_result_is_freestanding_and_debuggable() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-PARAMETER-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-custom-generator-character-result-parameter");
    let executable = directory.join("application");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/language/custom-generator-character-return-parameter.t");
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
            "break custom-generator-character-return-parameter.t:16",
            "-ex",
            "run",
            "-ex",
            "whatis generated",
            "-ex",
            "print generated",
            "-ex",
            "next",
            "-ex",
            "whatis character",
            "-ex",
            "print character",
            "-ex",
            "backtrace",
            "-ex",
            "finish",
            "-ex",
            "print (Character)$3",
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
        "consume",
        "topal.main",
        "$4 = \"R\"",
    ] {
        assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
}

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
        include_str!("../../../examples/language/string-normalization.t"),
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
        include_str!("../../../examples/language/strings-and-products.t"),
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
        include_str!("../../../examples/language/equality-and-ordering.t"),
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
        include_str!("../../../examples/language/nat-equality-and-ordering.t"),
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
        include_str!("../../../examples/language/forward-function-declarations.t"),
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
        include_str!("../../../examples/language/decreasing-int-recursion.t"),
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
        include_str!("../../../examples/language/increasing-int-recursion.t"),
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
        include_str!("../../../examples/language/nat-recursion.t"),
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
        include_str!("../../../examples/language/explicit-multi-parameter-decreases.t"),
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
        include_str!("../../../examples/language/overload-recursion-identity.t"),
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
        include_str!("../../../examples/language/mutual-int-recursion.t"),
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
        include_str!("../../../examples/language/nat-mutual-recursion.t"),
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
        include_str!("../../../examples/language/effect-function-boundary.t"),
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
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
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
        include_str!("../../../examples/language/list-boolean-values.t"),
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
        include_str!("../../../examples/language/list-string-values.t"),
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
        include_str!("../../../examples/language/list-character-values.t"),
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

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Character is Entry (\"A\", Empty)\nvalues reverse\n",
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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn nat_lists_are_private_freestanding_and_debuggable() {
    // TOPAL-NUM-NAT-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-NAT-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-nat-values");
    let source = directory.join("list-nat-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-nat-values.t"),
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
        b"(0, 123456789012345678901234567890, true, true, 3, true, 5, 0, 6, Entry ( 0, Entry ( 123456789012345678901234567890, Entry ( +Infinity, Empty ) ) ))\n"
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
        "%topal.ListStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.int.equal(ptr",
        "define internal ptr @topal.runtime.list.int.entry.count(ptr",
        "call i32 @topal.runtime.int.compare(ptr",
        "@topal.runtime.int.positive.infinity",
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
        "topal.runtime.list.string.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Nat is Entry (1, Empty)\nvalues reverse\n",
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
            "break list-nat-values.t:8",
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
        "type = List Nat",
        "candidate = Entry ( 0, Entry ( 123456789012345678901234567890, Entry ( +Infinity, Empty ) ) )",
        "fallback = 9",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn rational_lists_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-LIST-RATIONAL-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-rational-values");
    let source = directory.join("list-rational-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-rational-values.t"),
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
    assert!(String::from_utf8(executed.stdout).unwrap().contains("Entry ( Rational ( 1, 2 ), Entry ( Rational ( 17636684144620811271604938270, 1 ), Entry ( +Infinity, Empty ) ) )"));
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
        "%topal.ListRationalStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.rational.equal(ptr",
        "define internal ptr @topal.runtime.list.rational.entry.count(ptr",
        "call i32 @topal.runtime.rational.compare(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [" byval", " sret", " inalloca", "preallocated", "call ptr %"] {
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
            "break list-rational-values.t:8",
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
        "type = List Rational",
        "candidate = Entry ( Rational ( 1, 2 )",
        "fallback = Rational ( 9, 1 )",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn effect_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-EFFECT-EMPTY-001, TOPAL-EFFECT-IDENTITY-001,
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-EFFECT-CORE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-effect-values");
    let source = directory.join("list-effect-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-effect-values.t"),
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
        b"(Effects (), Effects (), true, true, 2, true, Effects (), Effects (), Effects (), Entry ( Effects (), Entry ( Effects (), Empty ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListEffectStorage = type { i8, ptr }",
        "define internal i1 @topal.runtime.list.effect.equal(ptr",
        "define internal ptr @topal.runtime.list.effect.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i8 } @topal.fn.return_2dpair.",
        "store i8 0",
        "load i8",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.boolean.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Effect is Entry (Effects (), Empty)\nvalues reverse\n",
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
            "break list-effect-values.t:8",
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
        "type = List Effect",
        "candidate = Entry ( Effects (), Entry ( Effects (), Empty ) )",
        "fallback = empty",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn comparison_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-COMPARISON-CORE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-comparison-values");
    let source = directory.join("list-comparison-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-comparison-values.t"),
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
        b"(Less, Equal, true, true, 3, true, Equal, Less, Greater, Entry ( Less, Entry ( Equal, Entry ( Greater, Empty ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListComparisonStorage = type { i32, ptr }",
        "define internal i1 @topal.runtime.list.comparison.equal(ptr",
        "define internal ptr @topal.runtime.list.comparison.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i32 } @topal.fn.return_2dpair.",
        "store i32",
        "load i32",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.boolean.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Comparison is Entry (1 <=> 2, Empty)\nvalues reverse\n",
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
            "break list-comparison-values.t:8",
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
        "type = List Comparison",
        "candidate = Entry ( Less, Entry ( Equal, Entry ( Greater, Empty ) ) )",
        "fallback = Greater",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn error_code_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-NUM-ARITHMETIC-ERROR-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-ERROR-CODE-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-error-code-values");
    let source = directory.join("list-error-code-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-error-code-values.t"),
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
        b"(out-of-range, not-representable, true, true, 4, true, division-by-zero, out-of-range, indeterminate, Entry ( out-of-range, Entry ( not-representable, Entry ( division-by-zero, Entry ( indeterminate, Empty ) ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListErrorCodeStorage = type { i32, ptr }",
        "define internal i1 @topal.runtime.list.error.code.equal(ptr",
        "define internal ptr @topal.runtime.list.error.code.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i32 } @topal.fn.return_2dpair.",
        "store i32",
        "load i32",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.comparison.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List ErrorCode is Entry (lang arithmetic out-of-range, Empty)\nvalues reverse\n",
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
            "break list-error-code-values.t:8",
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
        "type = List lang arithmetic ArithmeticErrorCode",
        "candidate = Entry ( out-of-range, Entry ( not-representable, Entry ( division-by-zero, Entry ( indeterminate, Empty ) ) ) )",
        "fallback = indeterminate",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn unit_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-PRODUCT-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-UNIT-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-unit-values");
    let source = directory.join("list-unit-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-unit-values.t"),
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
        b"((), (), true, true, 3, true, (), (), (), Entry ( (), Entry ( (), Entry ( (), Empty ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListUnitStorage = type { i8, ptr }",
        "define internal i1 @topal.runtime.list.unit.equal(ptr",
        "define internal ptr @topal.runtime.list.unit.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i8 } @topal.fn.return_2dpair.",
        "store i8 0",
        "load i8",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.effect.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Unit is Entry ((), Empty)\nvalues reverse\n",
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
            "break list-unit-values.t:8",
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
        "type = List Unit",
        "candidate = Entry ( (), Entry ( (), Entry ( (), Empty ) ) )",
        "fallback = ()",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn completed_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-FUNCTION-COMPLETED-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-COMPLETED-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-completed-values");
    let source = directory.join("list-completed-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-completed-values.t"),
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
        b"(Completed, Completed, true, true, 3, true, Completed, Completed, Completed, Entry ( Completed, Entry ( Completed, Entry ( Completed, Empty ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListCompletedStorage = type { i8, ptr }",
        "define internal i1 @topal.runtime.list.completed.equal(ptr",
        "define internal ptr @topal.runtime.list.completed.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i8 } @topal.fn.return_2dpair.",
        "store i8 0",
        "load i8",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.unit.equal",
        "topal.runtime.list.effect.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Completed is Entry (Completed, Empty)\nvalues reverse\n",
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
            "break list-completed-values.t:8",
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
        "type = List Completed",
        "candidate = Entry ( Completed, Entry ( Completed, Entry ( Completed, Empty ) ) )",
        "fallback = Completed",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn type_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-ABSTRACTION-TYPE-VALUE-001, TOPAL-ABSTRACTION-TYPE-IDENTITY-001,
    // TOPAL-ABSTRACTION-TYPE-BOUNDARY-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-TYPE-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-type-values");
    let source = directory.join("list-type-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-type-values.t"),
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
        b"(Boolean, Int, true, true, true, 7, true, String, Boolean, Unit, Entry ( Boolean, Entry ( Int, Entry ( Nat, Entry ( Rational, Entry ( String, Entry ( Unit, Entry ( Scope, Empty ) ) ) ) ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListTypeStorage = type { i32, ptr }",
        "define internal i1 @topal.runtime.list.type.equal(ptr",
        "define internal ptr @topal.runtime.list.type.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i32 } @topal.fn.return_2dpair.",
        "store i32 0",
        "load i32",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.comparison.equal",
        "topal.runtime.list.error.code.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nvalues : List Type is Entry (Int, Empty)\nvalues reverse\n",
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
            "break list-type-values.t:8",
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
        "type = List Type",
        "candidate = Entry ( Boolean, Entry ( Int, Entry ( Nat, Entry ( Rational, Entry ( String, Entry ( Unit, Entry ( Scope, Empty ) ) ) ) ) ) )",
        "fallback = String",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn nominal_enum_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-ENUM-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-ENUM-001, TOPAL-COMPILER-LIST-ENUM-CORE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-enum-values");
    let source = directory.join("list-enum-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-enum-values.t"),
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
        b"(Red, Green, true, true, true, 3, true, Blue, Red, Green, Entry ( Red, Entry ( Green, Entry ( Blue, Empty ) ) ))\n"
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
    assert!(emitted.status.success());
    let ir = fs::read_to_string(ir_path).unwrap();
    for expected in [
        "%topal.ListEnumStorage = type { i32, ptr }",
        "define internal i1 @topal.runtime.list.enum.equal(ptr",
        "define internal ptr @topal.runtime.list.enum.entry.count(ptr",
        "define internal fastcc ptr @topal.fn.return_2dlist.",
        "define internal fastcc { ptr, i32 } @topal.fn.return_2dpair.",
        "store i32 0",
        "store i32 1",
        "store i32 2",
        "load i32",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.runtime.list.type.equal",
        "topal.runtime.list.comparison.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nColor is Enum (Red, Green)\nvalues : List Color is Entry (Red, Empty)\nvalues reverse\n",
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
            "break list-enum-values.t:10",
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
        "type = List Color",
        "candidate = Entry ( Red, Entry ( Green, Entry ( Blue, Empty ) ) )",
        "fallback = Blue",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and source-level GDB values.
fn nominal_modular_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001,
    // TOPAL-COMPILER-MODULAR-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-MODULAR-CORE-001, TOPAL-COMPILER-PLATFORM-001,
    // TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-modular-values");
    let source = directory.join("list-modular-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-modular-values.t"),
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
        b"(ByteCounter 0, ByteCounter 255, true, true, true, 3, true, ByteCounter 7, ByteCounter 0, ByteCounter 8, Entry ( ByteCounter 0, Entry ( ByteCounter 255, Entry ( ByteCounter 42, Empty ) ) ))\n"
    );
    assert_freestanding_elf_and_valid_dwarf(&executable);

    let record_source = directory.join("record-values.t");
    let record_executable = directory.join("record-values");
    fs::write(
        &record_source,
        "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nretain-record is fn (package : Record (values : List ByteCounter, fallback : ByteCounter)) -> Record (values : List ByteCounter, fallback : ByteCounter)\n  package\nvalues : List ByteCounter is Entry (ByteCounter 1, Empty)\nretained is retain-record (values is values, fallback is ByteCounter 7)\nretained values\n",
    )
    .unwrap();
    let record_compiled = run(topalc().args([
        "-o",
        record_executable.to_str().unwrap(),
        record_source.to_str().unwrap(),
    ]));
    assert!(
        record_compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&record_compiled.stderr)
    );
    let record_executed = run(&mut Command::new(&record_executable));
    assert!(record_executed.status.success());
    assert_eq!(record_executed.stdout, b"Entry ( ByteCounter 1, Empty )\n");
    assert_freestanding_elf_and_valid_dwarf(&record_executable);

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
        "%topal.ListModularStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.modular.equal(ptr",
        "define internal ptr @topal.runtime.list.modular.entry.count(ptr",
        "call i32 @topal.runtime.int.compare(ptr",
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
        "topal.runtime.list.rational.equal",
        "topal.runtime.list.string.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("unsupported-transform.t");
    let rejected_executable = directory.join("unsupported-transform");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nvalues : List ByteCounter is Entry (ByteCounter 1, Empty)\nvalues reverse\n",
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
            "break list-modular-values.t:11",
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
        "type = List ByteCounter",
        "candidate = Entry ( ByteCounter 0, Entry ( ByteCounter 255, Entry ( ByteCounter 42, Empty ) ) )",
        "fallback = ByteCounter 7",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn optional_int_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001, TOPAL-COMPILER-LIST-OPTIONAL-INT-CORE-001
    let directory = temporary("gdb-list-optional-int-values");
    let source = directory.join("list-optional-int-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-optional-int-values.t"),
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
    assert_eq!(executed.stdout, b"(Some 1, None, true, true, true, 3, true, Some 7, Some 1, Some 8, Entry ( Some 1, Entry ( None, Entry ( Some -2, Empty ) ) ))\n");
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
        "%topal.ListOptionalIntStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.optional.int.equal(ptr",
        "define internal ptr @topal.runtime.list.optional.int.entry.count(ptr",
        "call i1 @topal.runtime.optional.int.equal(ptr",
        "define internal fastcc { ptr, ptr } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.modular.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List Optional Int is Entry (Some 1, Empty)\nvalues reverse\n").unwrap();
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
            "break list-optional-int-values.t:8",
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
        "type = List Optional Int",
        "candidate = Entry ( Some 1, Entry ( None, Entry ( Some -2, Empty ) ) )",
        "fallback = Some 7",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn optional_rational_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001,
    // TOPAL-COMPILER-LIST-OPTIONAL-RATIONAL-CORE-001
    let directory = temporary("gdb-list-optional-rational-values");
    let source = directory.join("list-optional-rational-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-optional-rational-values.t"),
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
    assert_eq!(executed.stdout, b"(Some Rational ( 1, 2 ), None, true, true, true, 3, true, Some Rational ( 7, 3 ), Some Rational ( 1, 2 ), Some Rational ( 8, 5 ), Entry ( Some Rational ( 1, 2 ), Entry ( None, Entry ( Some Rational ( -3, 4 ), Empty ) ) ))\n");
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
        "%topal.ListOptionalRationalStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.optional.rational.equal(ptr",
        "define internal ptr @topal.runtime.list.optional.rational.entry.count(ptr",
        "call i1 @topal.runtime.optional.rational.equal(ptr",
        "define internal fastcc { ptr, ptr } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.optional.int.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List Optional Rational is Entry (Some 1.5, Empty)\nvalues reverse\n").unwrap();
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
            "break list-optional-rational-values.t:8",
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
        "type = List Optional Rational",
        "candidate = Entry ( Some Rational ( 1, 2 ), Entry ( None, Entry ( Some Rational ( -3, 4 ), Empty ) ) )",
        "fallback = Some Rational ( 7, 3 )",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers artifacts, IR, rejection, and GDB values.
fn optional_string_lists_are_complete_private_freestanding_and_debuggable() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001,
    // TOPAL-COMPILER-LIST-OPTIONAL-STRING-CORE-001
    let directory = temporary("gdb-list-optional-string-values");
    let source = directory.join("list-optional-string-values.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-optional-string-values.t"),
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
    assert_eq!(executed.stdout, "(Some \"first\", None, true, true, true, 3, true, Some \"fallback\", Some \"first\", Some \"record\", Entry ( Some \"first\", Entry ( None, Entry ( Some \"世界\", Empty ) ) ))\n".as_bytes());
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
        "%topal.ListOptionalStringStorage = type { ptr, ptr }",
        "define internal i1 @topal.runtime.list.optional.string.equal(ptr",
        "define internal ptr @topal.runtime.list.optional.string.entry.count(ptr",
        "call i1 @topal.runtime.optional.string.equal(ptr",
        "define internal fastcc { ptr, ptr } @topal.fn.return_2dpair.",
    ] {
        assert!(ir.contains(expected), "{expected}: {ir}");
    }
    for forbidden in [
        " byval",
        " sret",
        "topal.runtime.list.optional.rational.equal",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List Optional String is Entry (Some \"value\", Empty)\nvalues reverse\n").unwrap();
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
            "break list-optional-string-values.t:8",
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
        "type = List Optional String",
        "candidate = Entry ( Some \"first\", Entry ( None, Entry ( Some \"世界\", Empty ) ) )",
        "fallback = Some \"fallback\"",
        "topal.fn.head_2dor.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

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
        include_str!("../../../examples/language/list-int-pair-values.t"),
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
        include_str!("../../../examples/language/list-int-string-pair-values.t"),
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
    let rejected_source = directory.join("unsupported.t");
    let rejected_executable = directory.join("unsupported");
    fs::write(&rejected_source, "use language (version is v0.1)\nvalues : List (Int, String) is Entry ((1, \"one\"), Empty)\nvalues reverse\n").unwrap();
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
        include_str!("../../../examples/language/list-string-int-pair-values.t"),
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
        include_str!("../../../examples/language/list-string-pair-values.t"),
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
        "use language (version is v0.1)\ninspect is fn (values : List Int, text : String) -> Int\n  _ is text = text\n  entry-count values\nvalues : List Int is Entry (9, Entry (2, Entry (4, Entry (7, Entry (3, Entry (340282366920938463463374607431768211456, Empty))))))\nbounds : Range Int is 2 ..= 4\nindexes : Range Int is 1 .. 4\nchosen is values select bounds\npositions is values select-index indexes\nslice is \"Topal\" select-index indexes\nexact is values select (340282366920938463463374607431768211456 ..= 340282366920938463463374607431768211456)\nnone is values select (8 .. 2)\nno-positions is values select-index (4 <..= 4)\nresult is inspect (chosen, slice)\n(chosen, positions, slice, result, exact, none, no-positions)\n",
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
        b"(Entry ( 2, Entry ( 4, Entry ( 3, Empty ) ) ), Entry ( 2, Entry ( 4, Entry ( 7, Empty ) ) ), \"opa\", 3, Entry ( 340282366920938463463374607431768211456, Empty ), Empty, Empty)\n"
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
    assert!(text.contains("topal.fn.inspect.0"), "{text}");
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
        include_str!("../../../examples/language/nested-lists.t"),
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
        include_str!("../../../examples/language/completion-effect-value.t"),
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
        include_str!("../../../examples/language/tuple-function-parameters.t"),
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
        include_str!("../../../examples/language/record-function-boundaries.t"),
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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn gdb_retains_fundamental_type_identity_across_a_function_boundary() {
    // TOPAL-COMP-DEBUG-001, TOPAL-ABSTRACTION-TYPE-BOUNDARY-001
    let directory = temporary("gdb-type-boundary");
    let source = directory.join("type-function-boundary.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/type-function-boundary.t"),
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
        include_str!("../../../examples/language/constraint-classifier.t"),
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
        include_str!("../../../examples/language/constraints-and-derived-capabilities.t"),
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
fn nominal_sums_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-SUM-001, TOPAL-TYPE-UNION-001,
    // TOPAL-TYPE-VARIANT-001, TOPAL-DECISION-UNION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-nominal-sums");
    let source = directory.join("unions-and-recursive-products.t");
    let executable = directory.join("application");
    let source_text = include_str!("../../../examples/language/unions-and-recursive-products.t");
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
        include_str!("../../../examples/language/root-namespace.t"),
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
        include_str!("../../../examples/language/use-namespace.t"),
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
        include_str!("../../../examples/language/namespace-alias.t"),
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
        include_str!("../../../examples/language/namespace-snapshot.t"),
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
        include_str!("../../../examples/language/namespace-function-parameter.t"),
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
        include_str!("../../../examples/language/named-function-values.t"),
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
        include_str!("../../../examples/language/callable-values.t"),
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
        include_str!("../../../examples/language/expanded-callable-values.t"),
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
        include_str!("../../../examples/language/function-effect-bound.t"),
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
        include_str!("../../../examples/language/static-introspection.t"),
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
    let source_text = include_str!("../../../examples/language/lint-language-variant.t");
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
        include_str!("../../../examples/language/capability-composition.t"),
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
        include_str!("../../../examples/language/function-interface.t"),
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
    assert!(metadata.evidence.is_empty());
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
        include_str!("../../../examples/language/function-value-boundary.t"),
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
        include_str!("../../../examples/language/function-results.t"),
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
        include_str!("../../../examples/language/anonymous-function-application.t"),
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
        include_str!("../../../examples/language/anonymous-function-captures.t"),
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
        include_str!("../../../examples/language/capturing-function-parameters.t"),
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
        include_str!("../../../examples/language/capturing-function-results.t"),
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
        include_str!("../../../examples/language/function-result-chains.t"),
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
        include_str!("../../../examples/language/anonymous-product-functions.t"),
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
        include_str!("../../../examples/language/nested-anonymous-patterns.t"),
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
        include_str!("../../../examples/language/function-aggregate-boundaries.t"),
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
        include_str!("../../../examples/language/capturing-function-aggregate-boundaries.t"),
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
        include_str!("../../../examples/language/repeated-anonymous-patterns.t"),
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
        include_str!("../../../examples/language/repeated-anonymous-aggregate-patterns.t"),
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
        include_str!("../../../examples/language/repeated-sum-patterns.t"),
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
        include_str!("../../../examples/language/sum-equality.t"),
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
        include_str!("../../../examples/language/repeated-function-aggregate-patterns.t"),
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
        include_str!("../../../examples/language/repeated-captured-function-patterns.t");
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
        include_str!("../../../examples/language/repeated-captured-named-function-patterns.t"),
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
    let source_text =
        include_str!("../../../examples/language/repeated-captured-function-aggregate-patterns.t");
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
        include_str!("../../../examples/language/nested-functions.t"),
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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn packaged_function_operand_is_flat_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-PACKAGED-OPERAND-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-packaged-function-operand");
    let source = directory.join("packaged-function-operand.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/packaged-function-operand.t"),
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
            "break packaged-function-operand.t:13",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "print fallback",
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
    assert!(text.contains("$1 = 40"), "{text}");
    assert!(text.contains("$2 = 2"), "{text}");
    assert!(text.contains("topal.fn.sum.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers ordering, rejection, artifacts, and GDB.
fn packaged_field_association_preserves_source_order_and_remains_debuggable() {
    // TOPAL-COMPILER-PACKAGED-ASSOCIATION-ORDER-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-packaged-function-association-order");
    let source = directory.join("packaged-function-association-order.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/packaged-function-association-order.t"),
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
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let right = main
        .find("call fastcc ptr @topal.fn.right_2dvalue")
        .unwrap();
    let left = main.find("call fastcc ptr @topal.fn.left_2dvalue").unwrap();
    let combine = main.find("call fastcc ptr @topal.fn.combine").unwrap();
    assert!(right < left && left < combine, "{main}");
    assert!(
        main.contains("@topal.fn.combine.2(ptr %v1, ptr @.topal.int.2, ptr %v0)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, call) in [
        ("unknown", "combine (left is 39, unknown is 1, right is 2)"),
        ("duplicate", "combine (left is 39, right is 1, right is 2)"),
    ] {
        let rejected_source = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(
            &rejected_source,
            format!(
                "use language (version is v0.1)\ncombine is fn ((left : Int, offset : Int default 1, right : Int)) -> Int\n  left + offset + right\n{call}\n"
            ),
        )
        .unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_source.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
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
            "break packaged-function-association-order.t:14",
            "-ex",
            "run",
            "-ex",
            "print left",
            "-ex",
            "print offset",
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
    assert!(text.contains("$1 = 39"), "{text}");
    assert!(text.contains("$2 = 1"), "{text}");
    assert!(text.contains("$3 = 2"), "{text}");
    assert!(text.contains("topal.fn.combine.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers operand order, rejection, artifacts, and GDB.
fn compound_packaged_operands_are_flat_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-COMPOUND-PACKAGED-OPERAND-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-compound-packaged-function-operands");
    let source = directory.join("compound-packaged-function-operands.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/compound-packaged-function-operands.t"),
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
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.combine.2(ptr %arg0, ptr %arg1, ptr %arg2, ptr %arg3)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc ptr @topal.fn.scale.7(ptr %arg0, ptr %arg1, ptr %arg2)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc ptr @topal.fn.shift.8(ptr %arg0, ptr %arg1, ptr %arg2)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let left = main.find("call fastcc ptr @topal.fn.left_2dvalue").unwrap();
    let right = main
        .find("call fastcc ptr @topal.fn.right_2dvalue")
        .unwrap();
    let combine = main.find("call fastcc ptr @topal.fn.combine").unwrap();
    let scale_value = main
        .find("call fastcc ptr @topal.fn.scale_2dvalue")
        .unwrap();
    let scale_factor = main
        .find("call fastcc ptr @topal.fn.scale_2dfactor")
        .unwrap();
    let scale = main.find("call fastcc ptr @topal.fn.scale.7").unwrap();
    assert!(left < right && right < combine, "{main}");
    assert!(scale_value < scale_factor && scale_factor < scale, "{main}");
    assert!(
        main.contains(
            "@topal.fn.combine.2(ptr %v0, ptr @.topal.int.4, ptr %v1, ptr @.topal.int.5)"
        ),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, text) in [
        (
            "unknown",
            "use language (version is v0.1)\ncombine is fn ((left : Int), (right : Int)) -> Int\n  left + right\n(left is 20) combine (unknown is 22)\n",
        ),
        (
            "duplicate-parameter",
            "use language (version is v0.1)\ncombine is fn ((value : Int), (value : Int)) -> Int\n  value\n(value is 20) combine (value is 22)\n",
        ),
    ] {
        let rejected_source = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_source, text).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_source.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
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
            "break compound-packaged-function-operands.t:21",
            "-ex",
            "run",
            "-ex",
            "print left",
            "-ex",
            "print 'left-offset'",
            "-ex",
            "print right",
            "-ex",
            "print 'right-offset'",
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
    assert!(text.contains("$1 = 20"), "{text}");
    assert!(text.contains("$2 = 1"), "{text}");
    assert!(text.contains("$3 = 21"), "{text}");
    assert!(text.contains("$4 = 0"), "{text}");
    assert!(text.contains("topal.fn.combine.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact aggregate fields, rejection, artifacts, and GDB.
fn structured_packaged_fields_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-STRUCTURED-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-structured-packaged-function-fields");
    let source = directory.join("structured-packaged-function-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/structured-packaged-function-fields.t"),
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
        b"(((20, 22), \"Ada\"), ((20, 22), \"Ada\", true), ((21, 21), \"default\", false), ((20, 22), \"Grace\", true))\n"
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
    assert!(
        ir.contains(
            "define internal fastcc { { ptr, ptr }, ptr } @topal.fn.retain_2done.2({ ptr, ptr } %arg0, { i1, ptr, i32, i32 } %arg1)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc { { ptr, ptr }, ptr, i1 } @topal.fn.retain_2dmixed.5({ ptr, ptr } %arg0, { i1, ptr, i32, i32 } %arg1, i1 %arg2)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let person = main
        .find("call fastcc { i1, ptr, i32, i32 } @topal.fn.make_2dperson.0")
        .unwrap();
    let pair = main
        .find("call fastcc { ptr, ptr } @topal.fn.make_2dpair.1")
        .unwrap();
    let retained = main
        .find("call fastcc { { ptr, ptr }, ptr } @topal.fn.retain_2done.2")
        .unwrap();
    assert!(person < pair && pair < retained, "{main}");
    assert!(
        main.contains("@topal.fn.retain_2done.2({ ptr, ptr } %v9, { i1, ptr, i32, i32 } %v13)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("opaque-package.t");
    let rejected_executable = directory.join("opaque-package");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nretain is fn ((pair : (Int, Int), person : Record (name : String, active : Boolean))) -> Int\n  0\nbundle is (pair is (20, 22), person is (name is \"Ada\", active is true))\nretain bundle\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(!rejected_executable.exists());
    assert!(!metadata_path(&rejected_executable).exists());

    let rejected_root_path = directory.join("root-callable-aggregate.t");
    let rejected_root_executable = directory.join("root-callable-aggregate");
    fs::write(
        &rejected_root_path,
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nread is fn () -> (Function, Int)\n  root bundle\nbundle is (increment, 40)\nread ()\n",
    )
    .unwrap();
    let rejected_root = run(topalc().args([
        "-o",
        rejected_root_executable.to_str().unwrap(),
        rejected_root_path.to_str().unwrap(),
    ]));
    assert!(!rejected_root.status.success());
    assert!(
        String::from_utf8_lossy(&rejected_root.stderr)
            .contains("unsupported function-body root data capture representation"),
        "{}",
        String::from_utf8_lossy(&rejected_root.stderr)
    );
    assert!(!rejected_root_executable.exists());
    assert!(!metadata_path(&rejected_root_executable).exists());

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
            "break structured-packaged-function-fields.t:14",
            "-ex",
            "run",
            "-ex",
            "print pair",
            "-ex",
            "print person",
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
    assert!(text.contains("$1 = {_0 = 20, _1 = 22}"), "{text}");
    assert!(
        text.contains("$2 = {active = true, name = \"Ada\"}"),
        "{text}"
    );
    assert!(text.contains("topal.fn.retain_2done.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers nominal Sum fields, rejection, artifacts, and GDB.
fn sum_packaged_fields_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-SUM-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-sum-packaged-function-fields");
    let source = directory.join("sum-packaged-function-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/sum-packaged-function-fields.t"),
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
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.score.1({ i32, ptr } %arg0, ptr %arg1)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc ptr @topal.fn.shift_2dscore.7({ i32, ptr } %arg0, ptr %arg1)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let message = main
        .find("call fastcc { i32, ptr } @topal.fn.make_2dmessage.5")
        .unwrap();
    let offset = main
        .find("call fastcc ptr @topal.fn.offset_2dvalue.6")
        .unwrap();
    let shifted = main
        .find("call fastcc ptr @topal.fn.shift_2dscore.7")
        .unwrap();
    assert!(message < offset && offset < shifted, "{main}");
    assert!(
        main.contains("@topal.fn.score.1({ i32, ptr } %v4, ptr @.topal.int.4)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, text) in [
        (
            "opaque-package",
            "use language (version is v0.1)\nMessage is Union\n  Stop\n  Move : Int\nscore is fn ((message : Message, fallback : Int)) -> Int\n  fallback\nbundle is (message is Move 42, fallback is 0)\nscore bundle\n",
        ),
        (
            "function-sum-field",
            "use language (version is v0.1)\nCarrier is Union\n  Carry : Function\nadd is +\napply is fn ((value : Carrier)) -> Int\n  0\napply (value is Carry add)\n",
        ),
    ] {
        let rejected_source = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_source, text).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_source.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
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
            "break sum-packaged-function-fields.t:20",
            "-ex",
            "run",
            "-ex",
            "whatis message",
            "-ex",
            "print message",
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
    assert!(text.contains("type = Message"), "{text}");
    assert!(text.contains("$1 = Move 42"), "{text}");
    assert!(text.contains("topal.fn.score.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Function fields, captures, rejection, artifacts, and GDB.
fn function_packaged_fields_retain_exact_callable_facts_and_debugging() {
    // TOPAL-COMPILER-FUNCTION-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-packaged-fields");
    let source = directory.join("function-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-packaged-fields.t"),
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
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.apply.2(i32 %arg0, ptr %arg1)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "define internal fastcc ptr @topal.fn.apply_2dcaptured.6(i32 %arg0, ptr %arg1, ptr %arg2)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let value = main
        .find("call fastcc ptr @topal.fn.make_2dvalue.0")
        .unwrap();
    let operation = main
        .find("call fastcc i32 @topal.fn.make_2doperation.1")
        .unwrap();
    let applied = main.find("call fastcc ptr @topal.fn.apply.2").unwrap();
    assert!(value < operation && operation < applied, "{main}");
    assert!(
        main.contains("@topal.fn.apply.2(i32 %v1, ptr %v0)"),
        "{main}"
    );
    assert!(
        main.contains("@topal.fn.apply_2dcaptured.6(i32 22, ptr @.topal.int.7, ptr @.topal.int.4)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
        "call fastcc ptr %",
        "call fastcc i32 %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("opaque-package.t");
    let rejected_executable = directory.join("opaque-package");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\napply is fn ((operation : Function, value : Int)) -> Int\n  operation (value, 2)\nbundle is (operation is +, value is 40)\napply bundle\n",
    )
    .unwrap();
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
            "break function-packaged-fields.t:13",
            "-ex",
            "run",
            "-ex",
            "whatis operation",
            "-ex",
            "print operation",
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
    assert!(text.contains("type = enum Function"), "{text}");
    assert!(text.contains("$1 = +"), "{text}");
    assert!(text.contains("$2 = 40"), "{text}");
    assert!(text.contains("topal.fn.apply.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers aggregate Function fields, captures, rejection, artifacts, and GDB.
fn function_aggregate_packaged_fields_retain_recursive_callable_facts() {
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-aggregate-packaged-fields");
    let source = directory.join("function-aggregate-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-aggregate-packaged-fields.t"),
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
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.apply_2drecord.2({ i32, ptr, i32, i32 } %arg0, ptr %arg1)"),
        "{ir}"
    );
    assert!(
        ir.contains("define internal fastcc ptr @topal.fn.apply_2dtuple.6({ i32, ptr } %arg0, ptr %arg1, ptr %arg2)"),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let addend = main
        .find("call fastcc ptr @topal.fn.make_2daddend.0")
        .unwrap();
    let bundle = main
        .find("call fastcc { i32, ptr, i32, i32 } @topal.fn.make_2drecord.1")
        .unwrap();
    let applied = main
        .find("call fastcc ptr @topal.fn.apply_2drecord.2")
        .unwrap();
    assert!(addend < bundle && bundle < applied, "{main}");
    assert!(
        main.contains("@topal.fn.apply_2drecord.2({ i32, ptr, i32, i32 } %v9, ptr %v0)"),
        "{main}"
    );
    assert!(
        main.contains("@topal.fn.apply_2dtuple.6({ i32, ptr } %v17, ptr @.topal.int.3, ptr %v15)"),
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
        "call fastcc ptr %",
        "call fastcc i32 %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("opaque-function-aggregate.t");
    let rejected_executable = directory.join("opaque-function-aggregate");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\napply is fn ((bundle : Record (operation : Function, value : Int))) -> Int\n  (bundle operation) (bundle value)\nouter is (bundle is (operation is { value } value + 2, value is 40))\napply outer\n",
    )
    .unwrap();
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
            "break function-aggregate-packaged-fields.t:22",
            "-ex",
            "run",
            "-ex",
            "whatis bundle",
            "-ex",
            "print bundle",
            "-ex",
            "print marker",
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
    assert!(text.contains("_0 = <anonymous fn/1>, _1 = 40"), "{text}");
    assert!(text.contains("$2 = 0"), "{text}");
    assert!(text.contains("topal.fn.apply_2dtuple.6"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers represented containers, rejection, artifacts, and GDB.
fn container_packaged_fields_retain_exact_private_representations() {
    // TOPAL-COMPILER-CONTAINER-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-container-packaged-fields");
    let source = directory.join("container-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/container-packaged-fields.t"),
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
        b"((Entry ( 20, Entry ( 22, Empty ) ), Some 42, 42, 40 ..= 42), (Entry ( 20, Entry ( 22, Empty ) ), None, 42, 40 ..= 42))\n"
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
    assert!(
        ir.contains(
            "define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.4(ptr %arg0, ptr %arg1, ptr %arg2, ptr %arg3)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let span = main
        .find("call fastcc ptr @topal.fn.make_2dspan.0")
        .unwrap();
    let outcome = main
        .find("call fastcc ptr @topal.fn.make_2doutcome.1")
        .unwrap();
    let maybe = main
        .find("call fastcc ptr @topal.fn.make_2dmaybe.2")
        .unwrap();
    let values = main
        .find("call fastcc ptr @topal.fn.make_2dvalues.3")
        .unwrap();
    let retained = main
        .find("call fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.4")
        .unwrap();
    assert!(
        span < outcome && outcome < maybe && maybe < values && values < retained,
        "{main}"
    );
    assert!(
        main.contains("@topal.fn.retain.4(ptr %v3, ptr %v2, ptr %v1, ptr %v0)"),
        "{main}"
    );
    let second_span = main.find("call ptr @topal.runtime.range.make").unwrap();
    let second_outcome = main
        .find("call fastcc ptr @topal.fn.make_2doutcome.5")
        .unwrap();
    let second_values = main
        .find("call fastcc ptr @topal.fn.make_2dvalues.6")
        .unwrap();
    let defaulted_maybe = main.find("call ptr @topal.runtime.optional.none").unwrap();
    let second_retained = main
        .find("call fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.7")
        .unwrap();
    assert!(
        second_span < second_outcome
            && second_outcome < second_values
            && second_values < defaulted_maybe
            && defaulted_maybe < second_retained,
        "{main}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
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
            "break container-packaged-fields.t:20",
            "-ex",
            "run",
            "-ex",
            "whatis values",
            "-ex",
            "print values",
            "-ex",
            "whatis maybe",
            "-ex",
            "print maybe",
            "-ex",
            "whatis outcome",
            "-ex",
            "print outcome",
            "-ex",
            "whatis span",
            "-ex",
            "print span",
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
    assert!(text.contains("type = List Int"), "{text}");
    assert!(
        text.contains("$1 = Entry ( 20, Entry ( 22, Empty ) )"),
        "{text}"
    );
    assert!(text.contains("type = Optional Int"), "{text}");
    assert!(text.contains("$2 = Some 42"), "{text}");
    assert!(
        text.contains("type = Result (Int, lang arithmetic ArithmeticErrorCode)"),
        "{text}"
    );
    assert!(text.contains("$3 = 42"), "{text}");
    assert!(text.contains("type = Range Int"), "{text}");
    assert!(text.contains("$4 = 40 ..= 42"), "{text}");
    assert!(text.contains("topal.fn.retain.4"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact collections, rejection, artifacts, and GDB.
fn collection_packaged_fields_retain_exact_private_representations() {
    // TOPAL-COMPILER-COLLECTION-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-collection-packaged-fields");
    let source = directory.join("collection-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/collection-packaged-fields.t"),
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
        b"((Array (2, 1, 2), Set (2, 1), Bag ((2, 2), (1, 1)), Map ((\"Ada\", 11), (\"Lin\", 8))), (Array (2, 1, 2), Set (2, 1), Bag ((2, 2), (1, 1)), Map ((\"Ada\", 11), (\"Lin\", 8))))\n"
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
    assert!(
        ir.contains(
            "define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.4(ptr %arg0, ptr %arg1, ptr %arg2, ptr %arg3)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let scores = main.find("call fastcc ptr @topal.fn.make_2dmap.0").unwrap();
    let occurrences = main.find("call fastcc ptr @topal.fn.make_2dbag.1").unwrap();
    let members = main.find("call fastcc ptr @topal.fn.make_2dset.2").unwrap();
    let array = main
        .find("call fastcc ptr @topal.fn.make_2darray.3")
        .unwrap();
    let retained = main
        .find("call fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.4")
        .unwrap();
    assert!(
        scores < occurrences && occurrences < members && members < array && array < retained,
        "{main}"
    );
    assert!(
        main.contains("@topal.fn.retain.4(ptr %v3, ptr %v2, ptr %v1, ptr %v0)"),
        "{main}"
    );
    for call in [
        "call fastcc ptr @topal.fn.make_2darray.5",
        "call fastcc ptr @topal.fn.make_2dset.6",
        "call fastcc ptr @topal.fn.make_2dbag.7",
        "call fastcc ptr @topal.fn.make_2dmap.8",
        "call fastcc { ptr, ptr, ptr, ptr } @topal.fn.retain.9(ptr %v9, ptr %v10, ptr %v11, ptr %v12)",
    ] {
        assert!(main.contains(call), "{call}: {main}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "topal.package.argument",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("mismatched-array-field.t");
    let rejected_executable = directory.join("mismatched-array-field");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nmake is fn () -> Array (2, Int)\n  values : List Int is Entry (1, Entry (2, Entry (3, Empty)))\n  values collect Array\nmake ()\n",
    )
    .unwrap();
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
            "break collection-packaged-fields.t:24",
            "-ex",
            "run",
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
    assert!(text.contains("type = Array 3 Int"), "{text}");
    assert!(text.contains("$1 = Array (2, 1, 2)"), "{text}");
    assert!(text.contains("type = Set Int"), "{text}");
    assert!(text.contains("$2 = Set (2, 1)"), "{text}");
    assert!(text.contains("type = Bag Int"), "{text}");
    assert!(text.contains("$3 = Bag ((2, 2), (1, 1))"), "{text}");
    assert!(text.contains("type = Map(String, Int)"), "{text}");
    assert!(
        text.contains("$4 = Map ((\"Ada\", 11), (\"Lin\", 8))"),
        "{text}"
    );
    assert!(text.contains("topal.fn.retain.4"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Scope packages, rejection, artifacts, and GDB.
fn scope_packaged_fields_retain_exact_private_environments() {
    // TOPAL-COMPILER-SCOPE-PACKAGED-FIELD-001,
    // TOPAL-COMPILER-NAMESPACE-BOUNDARY-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-scope-packaged-fields");
    let source = directory.join("scope-packaged-fields.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/scope-packaged-fields.t"),
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
    assert_eq!(executed.stdout, b"((42, 42), (42, 42), (42, 42))\n");
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
    for definition in [
        "define internal fastcc { ptr, ptr } @topal.fn.observe.1(i32 %arg0, ptr %arg1, ptr %arg2)",
        "define internal fastcc { ptr, ptr } @topal.fn.observe.2(i32 %arg0, ptr %arg1, ptr %arg2)",
        "define internal fastcc { ptr, ptr } @topal.fn.observe_2ddefault.4(i32 %arg0, ptr %arg1, ptr %arg2)",
    ] {
        assert!(ir.contains(definition), "{definition}: {ir}");
    }
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let value = main
        .find("call fastcc ptr @topal.fn.make_2dvalue.0")
        .unwrap();
    let observed = main
        .find("call fastcc { ptr, ptr } @topal.fn.observe.1")
        .unwrap();
    assert!(value < observed, "{main}");
    for call in [
        "@topal.fn.observe.1(i32 0, ptr %v1, ptr %v0)",
        "@topal.fn.observe.2(i32 0, ptr @.topal.int.7, ptr %v0)",
        "@topal.fn.observe_2ddefault.4(i32 0, ptr %v8, ptr %v0)",
    ] {
        assert!(main.contains(call), "{call}: {main}");
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        " preallocated",
        "package.runtime",
        "scope.runtime",
        "topal.package.argument",
        "topal.package.operand",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("function-live-root-package.t");
    let rejected_executable = directory.join("function-live-root-package");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nanswer is 42\naccept is fn ((api : Scope, value : Int)) -> Int\n  api answer + value\nwrapper is fn () -> Int\n  accept (api is root, value is 0)\nwrapper ()\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("function-body live root Scope argument"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
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
            "break scope-packaged-fields.t:18",
            "-ex",
            "run",
            "-ex",
            "whatis scope",
            "-ex",
            "print scope",
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
    assert!(text.contains("type = enum Scope"), "{text}");
    assert!(text.contains("$1 = <namespace root>"), "{text}");
    assert!(text.contains("type = Int"), "{text}");
    assert!(text.contains("$2 = 41"), "{text}");
    assert!(text.contains("scope answer = 42"), "{text}");
    assert!(text.contains("topal.fn.observe.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers lowering, rejection, artifacts, and GDB.
fn function_root_data_is_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-001, TOPAL-NAMESPACE-ROOT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-root-data");
    let source = directory.join("function-root-data.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-root-data.t"),
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
    assert_eq!(executed.stdout, b"(42, 0, \"ready\")\n");
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
    assert!(
        ir.contains(
            "define internal fastcc { ptr, ptr, ptr } @topal.fn.read.1(ptr %arg0, ptr %arg1, ptr %arg2)"
        ),
        "{ir}"
    );
    let main = ir.split("define internal void @topal.main").nth(1).unwrap();
    let initialized = main
        .find("call fastcc ptr @topal.fn.make_2danswer.0()")
        .unwrap();
    let invoked = main
        .find("call fastcc { ptr, ptr, ptr } @topal.fn.read.1(ptr @.topal.int.4, ptr %v0, ptr %v1)")
        .unwrap();
    assert!(initialized < invoked, "{main}");
    for forbidden in [
        "topal.root",
        "root.runtime",
        "namespace.runtime",
        "context.runtime",
        "lookup.root",
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
            "break function-root-data.t:8",
            "-ex",
            "run",
            "-ex",
            "whatis answer",
            "-ex",
            "print answer",
            "-ex",
            "print 'root label'",
            "-ex",
            "print 'root answer'",
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
    assert!(text.contains("type = Int"), "{text}");
    assert!(text.contains("$1 = 0"), "{text}");
    assert!(text.contains("$2 = \"ready\""), "{text}");
    assert!(text.contains("$3 = 42"), "{text}");
    assert!(
        text.contains("answer=0, root label=\"ready\", root answer=42"),
        "{text}"
    );
    assert!(text.contains("topal.fn.read.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers forwarding, rejection, artifacts, and every GDB frame.
fn function_root_data_forwarding_is_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001, TOPAL-NAMESPACE-ROOT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-root-data-forwarding");
    let source = directory.join("function-root-data-forwarding.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-root-data-forwarding.t"),
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
    assert_eq!(executed.stdout, b"(42, 0, \"ready\")\n");
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
    for name in ["read", "relay", "forward"] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!("@topal.fn.{name}."))
                    && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
            }),
            "{name}: {ir}"
        );
    }
    assert!(ir.lines().any(|line| {
        line.contains("call fastcc { ptr, ptr, ptr } @topal.fn.read.")
            && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
    }));
    assert!(ir.lines().any(|line| {
        line.contains("call fastcc { ptr, ptr, ptr } @topal.fn.relay.")
            && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
    }));
    for forbidden in [
        "topal.root",
        "root.runtime",
        "namespace.runtime",
        "context.runtime",
        "lookup.root",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_path = directory.join("overloaded.t");
    let rejected_executable = directory.join("overloaded");
    fs::write(
        &rejected_path,
        "use language (version is v0.1)\nread is fn (value : Nat) -> Int\n  root answer\nread is fn (value : Int) -> Int\n  0\nwrapper is fn (value : Int) -> Int\n  read value\nanswer is 42\nwrapper 0\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_path.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("overload-dependent root-data capture forwarding"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
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
            "break function-root-data-forwarding.t:8",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "frame 2",
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
    assert_eq!(text.matches("root label = \"ready\"").count(), 3, "{text}");
    assert_eq!(text.matches("root answer = 42").count(), 3, "{text}");
    assert_eq!(text.matches("answer = 0").count(), 3, "{text}");
    assert!(text.contains("topal.fn.read.0"), "{text}");
    assert!(text.contains("topal.fn.relay.1"), "{text}");
    assert!(text.contains("topal.fn.forward.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn defining_context_capture_is_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-CONTEXT-CAPTURE-001, TOPAL-CONTEXT-SELECT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-defining-context-capture");
    let source = directory.join("constructed-context.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/constructed-context.t"),
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
            "break constructed-context.t:8",
            "-ex",
            "run",
            "-ex",
            "print value",
            "-ex",
            "print '@ offset'",
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
    assert!(text.contains("$1 = 2"), "{text}");
    assert!(text.contains("$2 = 40"), "{text}");
    assert!(text.contains("@ offset=40"), "{text}");
    assert!(text.contains("topal.fn.add_2doffset.0"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers forwarding, rejection, artifacts, and every GDB frame.
fn defining_context_forwarding_is_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001, TOPAL-CONTEXT-SELECT-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-defining-context-forwarding");
    let source = directory.join("defining-context-forwarding.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/defining-context-forwarding.t"),
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
    assert_eq!(executed.stdout, b"(40, 2, \"ready\")\n");
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
    for name in ["read", "relay", "forward"] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!("@topal.fn.{name}."))
                    && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
            }),
            "{name}: {ir}"
        );
    }
    assert!(ir.lines().any(|line| {
        line.contains("call fastcc { ptr, ptr, ptr } @topal.fn.read.")
            && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
    }));
    assert!(ir.lines().any(|line| {
        line.contains("call fastcc { ptr, ptr, ptr } @topal.fn.relay.")
            && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
    }));
    for forbidden in [
        "topal.context",
        "context.runtime",
        "context.environment",
        "lookup.context",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_path = directory.join("overloaded.t");
    let rejected_executable = directory.join("overloaded");
    fs::write(
        &rejected_path,
        "use language (version is v0.1)\noffset is 40\nread is fn (value : Nat) -> Int\n  @ offset\nread is fn (value : Int) -> Int\n  0\nwrapper is fn (value : Int) -> Int\n  read value\nwrapper 0\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_path.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("overload-dependent defining-context capture forwarding"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
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
            "break defining-context-forwarding.t:11",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "frame 2",
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
    assert_eq!(text.matches("@ offset = 40").count(), 3, "{text}");
    assert_eq!(text.matches("@ label = \"ready\"").count(), 3, "{text}");
    assert_eq!(text.matches("offset = 2").count(), 3, "{text}");
    assert!(text.contains("topal.fn.read.0"), "{text}");
    assert!(text.contains("topal.fn.relay.1"), "{text}");
    assert!(text.contains("topal.fn.forward.2"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers proof gating, exact cyclic IR, artifacts, and every GDB frame.
fn recursive_scalar_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-recursive-scalar-environments");
    let source = directory.join("recursive-scalar-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/recursive-scalar-environments.t"),
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
    assert_eq!(executed.stdout, b"((false, 0, 2), (true, 40, 0))\n");
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
    let definitions = ir
        .lines()
        .filter(|line| {
            line.contains("define internal fastcc { i1, ptr, ptr } @topal.fn.cycle_2d")
                && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
        })
        .collect::<Vec<_>>();
    assert_eq!(definitions.len(), 4, "{ir}");
    assert!(definitions.iter().all(|line| line.contains("noinline")));
    assert!(definitions.iter().all(|line| !line.contains("norecurse")));
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains("call fastcc { i1, ptr, ptr } @topal.fn.cycle_2d")
                    && line.contains("ptr %arg1, ptr %arg2)")
            })
            .count(),
        4,
        "{ir}"
    );
    assert_eq!(
        ir.matches("!DILocalVariable(name: \"@ captured\", arg: 2")
            .count(),
        4
    );
    assert_eq!(
        ir.matches("!DILocalVariable(name: \"root live\", arg: 3")
            .count(),
        4
    );
    for forbidden in [
        "topal.context",
        "topal.root",
        "context.runtime",
        "namespace.runtime",
        "lookup.context",
        "lookup.root",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_path = directory.join("unproven.t");
    let rejected_executable = directory.join("unproven");
    fs::write(
        &rejected_path,
        "use language (version is v0.1)\ncaptured is 40\nloop is fn (value : Int) -> Int\n  value\n    <= 0 then @ captured\n    otherwise loop value\nloop 1\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_path.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("recursive function call"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
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
            "break recursive-scalar-environments.t:16",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "frame 2",
            "-ex",
            "info args",
            "-ex",
            "frame 3",
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
    assert_eq!(text.matches("@ captured = 40").count(), 4, "{text}");
    assert_eq!(text.matches("root live = 2").count(), 4, "{text}");
    for value in 0..=3 {
        assert!(text.contains(&format!("value = {value}")), "{text}");
    }
    assert!(text.contains("topal.fn.cycle_2deven.0"), "{text}");
    assert!(text.contains("topal.fn.cycle_2dodd.1"), "{text}");
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers aggregate classes, recursion, rejection, artifacts, and GDB frames.
fn aggregate_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-aggregate-environments");
    let source = directory.join("aggregate-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/aggregate-environments.t"),
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
        b"((40, \"context\"), (amount is 2, enabled is true), (7, \"root\"), (amount is 9, enabled is false), Label \"context-sum\", Number 11)\n"
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
    for name in [
        "select_2dcontext_2dpair",
        "forward_2dcontext_2dpair",
        "select_2droot_2dpair",
        "forward_2droot_2dpair",
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!(
                    "define internal fastcc {{ ptr, ptr }} @topal.fn.{name}."
                )) && line.contains("(ptr %arg0, { ptr, ptr } %arg1)")
            }),
            "{name}: {ir}"
        );
    }
    for name in [
        "select_2dcontext_2drecord",
        "forward_2dcontext_2drecord",
        "select_2droot_2drecord",
        "forward_2droot_2drecord",
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!(
                    "define internal fastcc {{ ptr, i1, i32, i32 }} @topal.fn.{name}."
                )) && line.contains("(ptr %arg0, { ptr, i1, i32, i32 } %arg1)")
            }),
            "{name}: {ir}"
        );
    }
    for name in [
        "select_2dcontext_2dtoken",
        "forward_2dcontext_2dtoken",
        "select_2droot_2dtoken",
        "forward_2droot_2dtoken",
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!(
                    "define internal fastcc {{ i32, ptr, ptr }} @topal.fn.{name}."
                )) && line.contains("({ i32, ptr, ptr } %arg0)")
            }),
            "{name}: {ir}"
        );
    }
    assert!(
        ir.lines().any(|line| {
            line.contains("call fastcc { ptr, ptr } @topal.fn.select_2dcontext_2dpair.")
                && line.contains("{ ptr, ptr }")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("call fastcc { ptr, i1, i32, i32 } @topal.fn.select_2droot_2drecord.")
                && line.contains("{ ptr, i1, i32, i32 }")
        }),
        "{ir}"
    );
    for capture in [
        "@ context-pair",
        "@ context-record",
        "@ context-token",
        "root live-pair",
        "root live-record",
        "root live-token",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{capture}\"")),
            "{capture}: {ir}"
        );
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.context",
        "topal.root",
        "context.runtime",
        "namespace.runtime",
        "lookup.context",
        "lookup.root",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_path = directory.join("callable-aggregate.t");
    let rejected_executable = directory.join("callable-aggregate");
    fs::write(
        &rejected_path,
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nbundle is (increment, 40)\nread is fn () -> (Function, Int)\n  @ bundle\nread ()\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_path.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("unsupported defining-context capture representation"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(!rejected_executable.exists());
    assert!(!metadata_path(&rejected_executable).exists());

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let tuple_debugged = run(Command::new("gdb")
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
            "break aggregate-environments.t:18",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "frame 2",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        tuple_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&tuple_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&tuple_debugged.stdout);
    assert_eq!(
        text.matches("@ context-pair = {_0 = 40, _1 = \"context\"}")
            .count(),
        3,
        "{text}"
    );
    assert!(
        text.contains("topal.fn.select_2dcontext_2dpair.0"),
        "{text}"
    );
    assert!(
        text.contains("topal.fn.forward_2dcontext_2dpair.1"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");

    let record_debugged = run(Command::new("gdb")
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
            "break aggregate-environments.t:42",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "frame 2",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        record_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&record_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&record_debugged.stdout);
    assert_eq!(
        text.matches("root live-record = {amount = 9, enabled = false}")
            .count(),
        3,
        "{text}"
    );
    assert!(text.contains("topal.fn.select_2droot_2drecord.6"), "{text}");
    assert!(
        text.contains("topal.fn.forward_2droot_2drecord.7"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");

    let sum_debugged = run(Command::new("gdb")
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
            "break aggregate-environments.t:51",
            "-ex",
            "break aggregate-environments.t:57",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        sum_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&sum_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&sum_debugged.stdout);
    assert!(
        text.contains("@ context-token = Label \"context-sum\""),
        "{text}"
    );
    assert!(text.contains("root live-token = Number 11"), "{text}");
    assert!(
        text.contains("topal.fn.forward_2droot_2dtoken.11"),
        "{text}"
    );
    assert!(text.contains("topal.main"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers overload selection, recursion, rejection, artifacts, and GDB frames.
fn overload_environments_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-overload-environments");
    let source = directory.join("overload-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/overload-environments.t"),
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
        b"(40, \"context\", (2, \"context-pair\"), (7, \"root-pair\"), 7, \"root\", 47, 40, \"root\")\n"
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
    for (name, signature) in [
        ("choose_2dcontext", "(ptr %arg0, ptr %arg1)"),
        ("choose_2droot", "(ptr %arg0, ptr %arg1)"),
        ("forward_2dcontext_2dnumber", "(ptr %arg0)"),
        ("forward_2dcontext_2dlabel", "(ptr %arg0)"),
        ("forward_2droot_2dnumber", "(ptr %arg0)"),
        ("forward_2droot_2dlabel", "(ptr %arg0)"),
        ("cross", "(ptr %arg0, ptr %arg1, ptr %arg2)"),
        ("forward_2dcross", "(ptr %arg0, ptr %arg1)"),
        ("forward_2dproduct_2dtuple", "(ptr %arg0)"),
        ("forward_2dproduct_2drecord", "(ptr %arg0)"),
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!("define internal fastcc ptr @topal.fn.{name}."))
                    && line.contains(signature)
            }),
            "{name}: {ir}"
        );
    }
    for name in [
        "choose_2dpair",
        "forward_2dcontext_2dpair",
        "forward_2droot_2dpair",
    ] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!(
                    "define internal fastcc {{ ptr, ptr }} @topal.fn.{name}."
                )) && line.contains("{ ptr, ptr }")
            }),
            "{name}: {ir}"
        );
    }
    assert_eq!(
        ir.lines()
            .filter(|line| line.contains("define internal fastcc ptr @topal.fn.choose_2dcontext."))
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.lines()
            .filter(|line| line.contains("define internal fastcc ptr @topal.fn.cross."))
            .count(),
        2,
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc ptr @topal.fn.choose_2dproduct.")
                && line.contains("({ ptr, ptr } %arg0, ptr %arg1)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc ptr @topal.fn.choose_2dproduct.")
                && line.contains("({ ptr, ptr, i32, i32 } %arg0, ptr %arg1)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("call fastcc ptr @topal.fn.cross.")
                && line.contains("(ptr @.topal.int.")
                && line.matches("ptr %arg").count() == 2
        }),
        "{ir}"
    );
    for capture in [
        "@ context-number",
        "@ context-label",
        "@ context-pair",
        "root live-pair",
        "root live-number",
        "root live-label",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{capture}\"")),
            "{capture}: {ir}"
        );
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.context",
        "topal.root",
        "context.runtime",
        "namespace.runtime",
        "lookup.context",
        "lookup.root",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, source_text, diagnostic) in [
        (
            "ambiguous-context",
            "use language (version is v0.1)\noffset is 40\nselect is fn (value : Nat) -> Int\n  @ offset\nselect is fn (value : Int) -> Int\n  0\nforward is fn (value : Int) -> Int\n  select value\nforward 0\n",
            "overload-dependent defining-context capture forwarding",
        ),
        (
            "ambiguous-root",
            "use language (version is v0.1)\nselect is fn (value : Nat) -> Int\n  root answer\nselect is fn (value : Int) -> Int\n  0\nforward is fn (value : Int) -> Int\n  select value\nanswer is 40\nforward 0\n",
            "overload-dependent root-data capture forwarding",
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
        assert!(
            String::from_utf8_lossy(&rejected.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&rejected.stderr)
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
    }

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let aggregate_debugged = run(Command::new("gdb")
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
            "break overload-environments.t:26",
            "-ex",
            "break overload-environments.t:29",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        aggregate_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&aggregate_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&aggregate_debugged.stdout);
    assert_eq!(
        text.matches("@ context-pair = {_0 = 2, _1 = \"context-pair\"}")
            .count(),
        2,
        "{text}"
    );
    assert_eq!(
        text.matches("root live-pair = {_0 = 7, _1 = \"root-pair\"}")
            .count(),
        2,
        "{text}"
    );
    assert!(text.contains("topal.fn.choose_2dpair.4"), "{text}");
    assert!(
        text.contains("topal.fn.forward_2dcontext_2dpair.5"),
        "{text}"
    );
    assert!(text.contains("topal.fn.choose_2dpair.6"), "{text}");
    assert!(text.contains("topal.fn.forward_2droot_2dpair.7"), "{text}");

    let recursive_debugged = run(Command::new("gdb")
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
            "break overload-environments.t:13",
            "-ex",
            "run",
            "-ex",
            "continue",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "frame 2",
            "-ex",
            "info args",
            "-ex",
            "frame 3",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        recursive_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&recursive_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&recursive_debugged.stdout);
    assert_eq!(text.matches("@ context-number = 40").count(), 4, "{text}");
    for value in 0..=2 {
        assert!(text.contains(&format!("value = {value}")), "{text}");
    }
    assert!(text.contains("topal.fn.choose_2dcontext.0"), "{text}");
    assert!(
        text.contains("topal.fn.forward_2dcontext_2dnumber.1"),
        "{text}"
    );

    let cross_debugged = run(Command::new("gdb")
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
            "break overload-environments.t:52",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "frame 2",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        cross_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&cross_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&cross_debugged.stdout);
    assert_eq!(text.matches("@ context-number = 40").count(), 3, "{text}");
    assert_eq!(text.matches("root live-number = 7").count(), 3, "{text}");
    assert!(text.contains("topal.fn.cross.12"), "{text}");
    assert!(text.contains("topal.fn.cross.13"), "{text}");
    assert!(text.contains("topal.fn.forward_2dcross.14"), "{text}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers aliases, nested calls, rejection, IR, artifacts, and GDB frames.
fn local_function_environments_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001,
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-local-function-environments");
    let source = directory.join("local-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/local-function-environments.t"),
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
        b"((42, \"context\", 9, \"root\", (2, \"context-pair\"), (7, \"root-pair\")), (42, 10))\n"
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
    for name in ["read_2dcontext", "read_2droot"] {
        assert_eq!(
            ir.lines()
                .filter(|line| {
                    line.contains(&format!("define internal fastcc ptr @topal.fn.{name}."))
                        && line.contains("(ptr %arg0, ptr %arg1)")
                })
                .count(),
            2,
            "{name}: {ir}"
        );
    }
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains("define internal fastcc { ptr, ptr } @topal.fn.read_2dpair.")
                    && line.contains("(ptr %arg0, { ptr, ptr } %arg1)")
            })
            .count(),
        2,
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains(
                "define internal fastcc { ptr, ptr, ptr, ptr, { ptr, ptr }, { ptr, ptr } } @topal.fn.alias_2dvalues.",
            ) && line.contains(
                "(ptr %arg0, ptr %arg1, { ptr, ptr } %arg2, ptr %arg3, ptr %arg4, { ptr, ptr } %arg5)",
            )
        }),
        "{ir}"
    );
    for name in ["nested_2dcontext", "nested_2droot"] {
        assert!(
            ir.lines().any(|line| {
                line.contains(&format!("define internal fastcc ptr @topal.fn.{name}."))
                    && line.contains("(ptr %arg0, ptr %arg1)")
                    && !line.contains("ptr %arg2")
            }),
            "{name}: {ir}"
        );
    }
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc { ptr, ptr } @topal.fn.nested_2dvalues.")
                && line.contains("(ptr %arg0, ptr %arg1)")
        }),
        "{ir}"
    );
    for name in [
        "read_2dcontext",
        "read_2droot",
        "read_2dpair",
        "nested_2dcontext",
        "nested_2droot",
    ] {
        assert!(
            ir.lines().any(|line| line.contains("call fastcc ")
                && line.contains(&format!("@topal.fn.{name}."))),
            "{name}: {ir}"
        );
    }
    for alias in [
        "context_2doperation",
        "context_2dchain",
        "root_2doperation",
        "root_2dchain",
        "pair_2doperation",
    ] {
        assert!(
            !ir.contains(&format!("@topal.fn.{alias}.")),
            "{alias}: {ir}"
        );
    }
    for local in [
        "context-operation",
        "context-chain",
        "root-operation",
        "root-chain",
        "pair-operation",
        "@ context-number",
        "@ context-pair",
        "root live-number",
        "root live-pair",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{local}\"")),
            "{local}: {ir}"
        );
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.context",
        "topal.root",
        "context.runtime",
        "namespace.runtime",
        "lookup.context",
        "lookup.root",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("ambiguous-alias.t");
    let rejected_executable = directory.join("ambiguous-alias");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\noffset is 40\nselect is fn (value : Nat) -> Int\n  @ offset\nselect is fn (value : Int) -> Int\n  0\nforward is fn (value : Int) -> Int\n  operation is select\n  operation value\nforward 0\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("overload-dependent defining-context capture forwarding"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(!rejected_executable.exists());
    assert!(!metadata_path(&rejected_executable).exists());

    let shadow_source = directory.join("shadow.t");
    let shadow_executable = directory.join("shadow");
    fs::write(
        &shadow_source,
        "use language (version is v0.1)\noffset is 40\nread is fn (value : Int) -> Int\n  value + @ offset\nwrapper is fn () -> Int\n  read is +\n  read (20, 22)\nwrapper ()\n",
    )
    .unwrap();
    let shadowed = run(topalc().args([
        "-o",
        shadow_executable.to_str().unwrap(),
        shadow_source.to_str().unwrap(),
    ]));
    assert!(
        shadowed.status.success(),
        "{}",
        String::from_utf8_lossy(&shadowed.stderr)
    );
    let shadow_output = run(&mut Command::new(&shadow_executable));
    assert_eq!(shadow_output.stdout, b"42\n");

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let alias_debugged = run(Command::new("gdb")
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
            "break local-function-environments.t:12",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "info locals",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        alias_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&alias_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&alias_debugged.stdout);
    for value in [
        "value = 2",
        "@ context-number = 40",
        "@ context-label = \"context\"",
        "@ context-pair = {_0 = 2, _1 = \"context-pair\"}",
        "root live-number = 7",
        "root live-label = \"root\"",
        "root live-pair = {_0 = 7, _1 = \"root-pair\"}",
        "context-operation = <fn read-context>",
        "context-chain = <fn read-context>",
        "root-operation = <fn read-root>",
        "root-chain = <fn read-root>",
        "pair-operation = <fn read-pair>",
        "topal.fn.read_2dcontext.",
        "topal.fn.alias_2dvalues.",
        "topal.main",
    ] {
        assert!(text.contains(value), "{value}: {text}");
    }

    let nested_debugged = run(Command::new("gdb")
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
            "break local-function-environments.t:46",
            "-ex",
            "break local-function-environments.t:48",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "info locals",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "info locals",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        nested_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&nested_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&nested_debugged.stdout);
    for value in [
        "value = 2",
        "value = 3",
        "@ context-number = 40",
        "root live-number = 7",
        "context-operation = <fn nested-context>",
        "root-operation = <fn nested-root>",
        "topal.fn.nested_2dcontext.",
        "topal.fn.nested_2droot.",
        "topal.fn.nested_2dvalues.",
        "topal.main",
    ] {
        assert!(text.contains(value), "{value}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers scalar/aggregate/results, rejection, artifacts, and GDB frames.
fn function_environment_boundaries_are_exact_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-FUNCTION-ENVIRONMENT-BOUNDARY-001,
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-function-environment-boundaries");
    let source = directory.join("function-environment-boundaries.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/function-environment-boundaries.t"),
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
        b"(42, \"context\", 9, \"root\", (2, \"context-pair\"), (7, \"root-pair\"), (42, \"context\", 9, \"root\", (2, \"context-pair\"), (7, \"root-pair\")), 48, 49, (49, 50))\n"
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
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains(
                    "define internal fastcc { i32, ptr, ptr } @topal.fn.return_2doperation.",
                ) && line.contains("(i32 %arg0, ptr %arg1, ptr %arg2)")
            })
            .count(),
        3,
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains(
                "define internal fastcc { i32, { ptr, ptr }, { ptr, ptr } } @topal.fn.return_2doperation.",
            ) && line.contains("(i32 %arg0, { ptr, ptr } %arg1, { ptr, ptr } %arg2)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc { i32, ptr, ptr } @topal.fn.make_2danonymous.")
                && line.contains("(ptr %arg0, ptr %arg1)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc { ptr, ptr, ptr, ptr, { ptr, ptr }, { ptr, ptr } } @topal.fn.apply_2drecord.")
                && line.contains("%arg0, ptr %arg1, ptr %arg2, { ptr, ptr } %arg3, { ptr, ptr } %arg4, ptr %arg5, ptr %arg6")
        }),
        "{ir}"
    );
    for name in ["read_2dcontext", "read_2droot"] {
        assert_eq!(
            ir.lines()
                .filter(|line| {
                    line.contains(&format!("define internal fastcc ptr @topal.fn.{name}."))
                        && line.contains("(ptr %arg0, ptr %arg1)")
                        && !line.contains("ptr %arg2")
                })
                .count(),
            4,
            "{name}: {ir}"
        );
    }
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains("define internal fastcc { ptr, ptr } @topal.fn.read_2dpair.")
                    && line.contains("(ptr %arg0, { ptr, ptr } %arg1)")
            })
            .count(),
        4,
        "{ir}"
    );
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains("define internal fastcc ptr @topal.fn.anonymous.")
                    && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
            })
            .count(),
        2,
        "{ir}"
    );
    for name in [
        "return_2doperation",
        "make_2danonymous",
        "make_2danonymous_2drecord",
        "apply_2drecord",
        "forward_2drecord",
        "apply_2dint",
        "forward_2dint",
        "anonymous",
        "increase",
    ] {
        assert!(
            ir.lines().any(|line| line.contains("call fastcc ")
                && line.contains(&format!("@topal.fn.{name}."))),
            "{name}: {ir}"
        );
    }
    for variable in [
        "operation",
        "package",
        "@ context-number",
        "@ context-label",
        "@ context-pair",
        "root live-number",
        "root live-label",
        "root live-pair",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{variable}\"")),
            "{variable}: {ir}"
        );
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "topal.context",
        "topal.root",
        "context.runtime",
        "namespace.runtime",
        "lookup.context",
        "lookup.root",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("fact-dependent-boundary.t");
    let rejected_executable = directory.join("fact-dependent-boundary");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\noffset is 40\nselect is fn (value : Nat) -> Int\n  @ offset\nselect is fn (value : Int) -> Int\n  0\napply is fn (operation : Function, value : Int) -> Int\n  operation value\napply (select, 0)\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("value-fact-dependent named Function environment selection"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(!rejected_executable.exists());
    assert!(!metadata_path(&rejected_executable).exists());

    let pretty_printers = Path::new(env!("CARGO_MANIFEST_DIR")).join("gdb/topal.py");
    let named_debugged = run(Command::new("gdb")
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
            "break function-environment-boundaries.t:36",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        named_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&named_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&named_debugged.stdout);
    assert!(text.contains("operation = <fn read-context>"), "{text}");
    assert!(text.contains("topal.fn.return_2doperation."), "{text}");
    assert!(text.contains("topal.main"), "{text}");

    let captured_debugged = run(Command::new("gdb")
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
            "break function-environment-boundaries.t:55",
            "-ex",
            "break function-environment-boundaries.t:64",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "continue",
            "-ex",
            "info args",
            "-ex",
            "frame 1",
            "-ex",
            "info args",
            "-ex",
            "frame 2",
            "-ex",
            "info args",
            "-ex",
            "frame 3",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
        ])
        .arg(&executable));
    assert!(
        captured_debugged.status.success(),
        "{}",
        String::from_utf8_lossy(&captured_debugged.stderr)
    );
    let text = String::from_utf8_lossy(&captured_debugged.stdout);
    assert_eq!(text.matches("@ context-number = 40").count(), 3, "{text}");
    assert_eq!(text.matches("root live-number = 7").count(), 3, "{text}");
    assert!(text.contains("operation = <fn increase>"), "{text}");
    assert!(text.matches("value = 2").count() >= 3, "{text}");
    for frame in [
        "topal.fn.anonymous.",
        "topal.fn.increase.",
        "topal.fn.apply_2dint.",
        "topal.fn.forward_2dint.",
        "topal.fn.use_2dnested.",
        "topal.main",
    ] {
        assert!(text.contains(frame), "{frame}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers scalar/aggregate escape, rejection, IR, and GDB frames.
fn escaping_nested_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-escaping-nested-function-environments");
    let source = directory.join("escaping-nested-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/escaping-nested-function-environments.t"),
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
    assert_eq!(executed.stdout, b"(43, 44, 45, 46, 47, (7, \"seven\"))\n");
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
    assert_eq!(
        ir.lines()
            .filter(|line| {
                line.contains(
                    "define internal fastcc { i32, ptr, ptr, ptr } @topal.fn.make_2doperation.",
                ) && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2)")
            })
            .count(),
        4,
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains(
                "define internal fastcc { i32, { ptr, ptr } } @topal.fn.make_2dpair_2doperation.",
            ) && line.contains("({ ptr, ptr } %arg0)")
        }),
        "{ir}"
    );
    assert!(
        ir.lines().any(|line| {
            line.contains("define internal fastcc { { i32, ptr, i32, i32 }, ptr, ptr, ptr, ptr } @topal.fn.make_2drecord.")
                && line.contains("(ptr %arg0, ptr %arg1, ptr %arg2, ptr %arg3)")
        }),
        "{ir}"
    );
    for name in [
        "make_2doperation",
        "return_2doperation",
        "make_2dpair_2doperation",
        "make_2drecord",
        "forward_2drecord",
        "apply_2drecord",
        "increase",
        "read_2dpair",
    ] {
        assert!(
            ir.lines().any(|line| line.contains("call fastcc ")
                && line.contains(&format!("@topal.fn.{name}."))),
            "{name}: {ir}"
        );
    }
    for variable in [
        "offset",
        "pair",
        "operation",
        "package",
        "@ context-offset",
        "root live-offset",
    ] {
        assert!(
            ir.contains(&format!("!DILocalVariable(name: \"{variable}\"")),
            "{variable}: {ir}"
        );
    }
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let rejected_source = directory.join("function-capture.t");
    let rejected_executable = directory.join("function-capture");
    fs::write(
        &rejected_source,
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nmake is fn (operation : Function) -> Function\n  wrapped is fn (value : Int) -> Int\n    operation value\n  wrapped\nresult is make increment\nresult 41\n",
    )
    .unwrap();
    let rejected = run(topalc().args([
        "-o",
        rejected_executable.to_str().unwrap(),
        rejected_source.to_str().unwrap(),
    ]));
    assert!(!rejected.status.success());
    let diagnostic = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        diagnostic.contains("E-COMPILER-UNSUPPORTED"),
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
            "break escaping-nested-function-environments.t:10",
            "-ex",
            "break escaping-nested-function-environments.t:20",
            "-ex",
            "break escaping-nested-function-environments.t:25",
            "-ex",
            "disable 2 3",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
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
        "operation = <fn increase>",
        "value = 1",
        "operand = 1",
        "offset = 1",
        "offset = 4",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.return_2doperation.",
        "topal.fn.increase.",
        "topal.fn.apply_2drecord.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Optional paths, rejection, mismatch, IR, and GDB frames.
fn optional_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-OPTIONAL-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-optional-function-environments");
    let source = directory.join("optional-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/optional-function-environments.t"),
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
        b"(43, 44, 45, 5, 42, Some +, 46, 47, 48, 0, 1, Some <fn increase>, None)\n"
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
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2doptional.")
            .count(),
        5,
        "{ir}"
    );
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains("call ptr @topal.runtime.optional.some(ptr"));
    assert!(ir.contains("call ptr @topal.runtime.optional.none()"));
    assert_eq!(
        ir.matches("call void @topal.runtime.pattern.identity.fail()")
            .count(),
        4,
        "{ir}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let mismatch_source = directory.join("mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        "use language (version is v0.1)\nmake is fn (offset : Int) -> Optional Function\n  increase is fn (value : Int) -> Int\n    value + offset\n  Some increase\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 2)\n",
    )
    .unwrap();
    let compiled_mismatch = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        compiled_mismatch.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled_mismatch.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(mismatch.stdout.is_empty());
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("E-ANONYMOUS-PATTERN-IDENTITY"));

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nchoose is fn (flag : Boolean) -> Optional Function\n  flag\n    true then Some increment\n    false then Some decrement\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Optional Function\n  nested is fn (value : Int) -> Int\n    operation value\n  Some nested\nwrap increment\n",
        ),
        (
            "optional-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (candidate : Optional Function) -> Optional Function\n  nested is fn (value : Int) -> Int\n    candidate\n      Some operation then operation value\n      None then 0\n  Some nested\nwrap (Some increment)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
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
            "break optional-function-environments.t:20",
            "-ex",
            "break optional-function-environments.t:42",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
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
        "candidate=Some <fn increase>",
        "candidate = Some <fn increase>",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2doptional.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Sum paths, rejection, mismatch, IR, and GDB frames.
fn sum_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-SUM-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-sum-function-environments");
    let source = directory.join("sum-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/sum-function-environments.t"),
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
        b"(43, 44, 45, 5, 42, Apply +, 46, 47, 48, 0, 1, Apply <fn increase>, Unavailable)\n"
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
    assert_eq!(
        ir.matches(
            "define internal fastcc { { i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2doperation."
        )
        .count(),
        5,
        "{ir}"
    );
    assert!(ir.contains(
        "define internal fastcc { { i32, i32, ptr }, ptr, ptr, ptr } @topal.fn.make_2dchoice."
    ));
    assert_eq!(
        ir.matches("call void @topal.runtime.pattern.identity.fail()")
            .count(),
        4,
        "{ir}"
    );
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    let record_source = directory.join("record.t");
    let record_executable = directory.join("record");
    fs::write(
        &record_source,
        "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\ncontext-offset is 40\nmake is fn (offset : Int) -> Record (candidate : Operation, value : Int)\n  increase is fn (operand : Int) -> Int\n    operand + offset + @ context-offset + (root live-offset)\n  (candidate is Apply increase, value is 1)\napply is fn (package : Record (candidate : Operation, value : Int)) -> Int\n  package candidate\n    Apply operation then operation (package value)\n    Missing then 0\nlive-offset is 1\napply (make 1)\n",
    )
    .unwrap();
    let compiled_record = run(topalc().args([
        "-o",
        record_executable.to_str().unwrap(),
        record_source.to_str().unwrap(),
    ]));
    assert!(
        compiled_record.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled_record.stderr)
    );
    assert_eq!(run(&mut Command::new(&record_executable)).stdout, b"43\n");

    let mismatch_source = directory.join("mismatch.t");
    let mismatch_executable = directory.join("mismatch");
    fs::write(
        &mismatch_source,
        "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nmake is fn (offset : Int) -> Operation\n  increase is fn (value : Int) -> Int\n    value + offset\n  Apply increase\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 2)\n",
    )
    .unwrap();
    let compiled_mismatch = run(topalc().args([
        "-o",
        mismatch_executable.to_str().unwrap(),
        mismatch_source.to_str().unwrap(),
    ]));
    assert!(
        compiled_mismatch.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled_mismatch.stderr)
    );
    let mismatch = run(&mut Command::new(&mismatch_executable));
    assert_eq!(mismatch.status.code(), Some(65));
    assert!(mismatch.stdout.is_empty());
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("E-ANONYMOUS-PATTERN-IDENTITY"));

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nchoose is fn (flag : Boolean) -> Operation\n  flag\n    true then Apply increment\n    false then Apply decrement\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Operation\n  nested is fn (value : Int) -> Int\n    operation value\n  Apply nested\nwrap increment\n",
        ),
        (
            "sum-function-capture",
            "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (candidate : Operation) -> Operation\n  nested is fn (value : Int) -> Int\n    candidate\n      Apply operation then operation value\n      Missing then 0\n  Apply nested\nwrap (Apply increment)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
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
            "break sum-function-environments.t:21",
            "-ex",
            "break sum-function-environments.t:42",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
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
        "candidate=Apply <fn increase>",
        "candidate = Apply <fn increase>",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2doperation.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers Result paths, rejection, Error propagation, IR, and GDB frames.
fn result_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-RESULT-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-result-function-environments");
    let source = directory.join("result-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/result-function-environments.t"),
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
        b"(43, 44, 45, 5, 42, +, 46, 47, 48, 49, 49, 0, 0, 0, <fn increase>, Error ( domain is root./(Rational,Rational), code is division-by-zero ))\n"
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
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2dresult.")
            .count(),
        7,
        "{ir}"
    );
    assert_eq!(
        ir.matches(
            "define internal fastcc { ptr, ptr, ptr, ptr, ptr, ptr } @topal.fn.make_2dfallible."
        )
        .count(),
        2,
        "{ir}"
    );
    assert!(
        ir.contains("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.project_2dresult.")
    );
    assert!(ir.contains(
        "define internal fastcc { ptr, ptr, ptr, ptr, ptr, ptr } @topal.fn.project_2dresult."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains("call ptr @topal.runtime.result.success(ptr"));
    assert!(ir.contains("insertvalue { ptr, ptr, ptr, ptr, ptr, ptr } poison, ptr"));
    assert!(ir.contains(", ptr null, 5"));
    assert!(ir.contains("ret { ptr, ptr, ptr, ptr, ptr, ptr }"));
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  increment\nright is fn () -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  decrement\nchoose is fn (flag : Boolean) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  nested is fn (value : Int) -> Int\n    operation value\n  nested\nwrap increment\n",
        ),
        (
            "result-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  increment\nwrap is fn (candidate : Result (Function, lang arithmetic ArithmeticErrorCode)) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  nested is fn (value : Int) -> Int\n    candidate\n      Ok operation then operation value\n      Error problem then 0\n  nested\nwrap (source ())\n",
        ),
        (
            "repeated-identity",
            "use language (version is v0.1)\nmake is fn (offset : Int) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  increase is fn (value : Int) -> Int\n    value + offset\n  increase\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
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
            "break result-function-environments.t:24",
            "-ex",
            "break result-function-environments.t:46",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
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
        "candidate=<fn increase>",
        "candidate = <fn increase>",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2dresult.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact List paths, rejection, IR, and GDB frames.
fn list_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-LIST-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-list-function-environments");
    let source = directory.join("list-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/list-function-environments.t"),
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
        b"(43, 44, 45, 5, 42, Entry ( +, Empty ), 46, 47, 48, 1, 0, Entry ( <fn increment>, Entry ( <fn increase>, Empty ) ), Empty)\n"
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
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2dlist.")
            .count(),
        6,
        "{ir}"
    );
    assert!(ir.contains("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.return_2dlist."));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(ir.contains("store i32 "));
    assert!(ir.contains("getelementptr i8, ptr"));
    assert!(ir.contains(", i64 8"));
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> List Function\n  Entry (increment, Empty)\nright is fn () -> List Function\n  Entry (decrement, Empty)\nchoose is fn (flag : Boolean) -> List Function\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> List Function\n  nested is fn (value : Int) -> Int\n    operation value\n  Entry (nested, Empty)\nwrap increment\n",
        ),
        (
            "list-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> List Function\n  Entry (increment, Empty)\nwrap is fn (candidate : List Function) -> List Function\n  nested is fn (value : Int) -> Int\n    candidate\n      Entry (operation, remaining) then operation value\n      Empty then 0\n  Entry (nested, Empty)\nwrap (source ())\n",
        ),
        (
            "repeated-identity",
            "use language (version is v0.1)\nmake is fn (offset : Int) -> List Function\n  increase is fn (value : Int) -> Int\n    value + offset\n  Entry (increase, Empty)\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
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
            "break list-function-environments.t:24",
            "-ex",
            "break list-function-environments.t:43",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
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
        "candidate=Entry ( <fn increment>, Entry ( <fn increase>, Empty ) )",
        "candidate = Entry ( <fn increment>, Entry ( <fn increase>, Empty ) )",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2dsecond.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact Array paths, rejection, IR, and GDB frames.
fn array_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-ARRAY-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-array-function-environments");
    let source = directory.join("array-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/array-function-environments.t"),
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
        b"(2, 43, 44, 45, 5, 42, Array (+), 46, 47, 48, 2, 0, true, Array (<fn increment>, <fn increase>), Array ())\n"
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
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2darray.")
            .count(),
        6,
        "{ir}"
    );
    assert!(ir.contains("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.return_2darray."));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains("call ptr @topal.runtime.container.array.function.collect(ptr"));
    assert!(ir.contains("call ptr @topal.runtime.container.array.function.at(ptr"));
    assert!(ir.contains("%topal.ContainerSequenceHeader = type { i64, ptr }"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 4)"));
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, rejected_source) in [
        (
            "dynamic",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> Array (1, Function)\n  values : List Function is Entry (increment, Empty)\n  values collect Array\nright is fn () -> Array (1, Function)\n  values : List Function is Entry (decrement, Empty)\n  values collect Array\nchoose is fn (flag : Boolean) -> Array (1, Function)\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Array (1, Function)\n  nested is fn (value : Int) -> Int\n    operation value\n  values : List Function is Entry (nested, Empty)\n  values collect Array\nwrap increment\n",
        ),
        (
            "array-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Array (1, Function)\n  values : List Function is Entry (increment, Empty)\n  values collect Array\nwrap is fn (candidate : Array (1, Function)) -> Array (1, Function)\n  nested is fn (value : Int) -> Int\n    array-at? (candidate, 0)\n      Some operation then operation value\n      None then 0\n  values : List Function is Entry (nested, Empty)\n  values collect Array\nwrap (source ())\n",
        ),
        (
            "repeated-identity",
            "use language (version is v0.1)\nmake is fn (offset : Int) -> Array (1, Function)\n  increase is fn (value : Int) -> Int\n    value + offset\n  values : List Function is Entry (increase, Empty)\n  values collect Array\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
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
            "break array-function-environments.t:23",
            "-ex",
            "break array-function-environments.t:47",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
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
        "candidate=Array (<fn increment>, <fn increase>)",
        "candidate = Array (<fn increment>, <fn increase>)",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2dsecond.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
#[allow(clippy::too_many_lines)] // One session covers exact Map paths, rejection, IR, and GDB frames.
fn map_function_environments_are_private_freestanding_and_debuggable() {
    // TOPAL-COMPILER-MAP-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let directory = temporary("gdb-map-function-environments");
    let source = directory.join("map-function-environments.t");
    let executable = directory.join("application");
    fs::write(
        &source,
        include_str!("../../../examples/language/map-function-environments.t"),
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
        b"(2, 43, 44, 45, 5, 42, 3, 46, 47, 48, 2, 8, 2, 0, false, Map ((\"increment\", <fn increment>), (\"increase\", <fn increase>)))\n"
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
    assert_eq!(
        ir.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2dmap.")
            .count(),
        6,
        "{ir}"
    );
    assert!(ir.contains("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.return_2dmap."));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(ir.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(ir.contains("call ptr @topal.runtime.container.map.string-function.collect(ptr"));
    assert!(ir.contains("call ptr @topal.runtime.container.map.string-function.lookup(ptr"));
    assert!(ir.contains("%topal.ContainerSequenceHeader = type { i64, ptr }"));
    assert!(ir.contains("%topal.ContainerMapNode = type { ptr, ptr, ptr }"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 24)"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 16)"));
    assert!(ir.contains("call ptr @topal.platform.allocate(i64 4)"));
    for forbidden in [
        " byval",
        " sret",
        " inalloca",
        "preallocated",
        "closure.runtime",
        "environment.runtime",
        "call ptr %",
    ] {
        assert!(!ir.contains(forbidden), "{forbidden}: {ir}");
    }

    for (name, rejected_source) in [
        (
            "dynamic-map",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", increment), Empty)\n  collect-map pairs resolving reject\nright is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", decrement), Empty)\n  collect-map pairs resolving reject\nchoose is fn (flag : Boolean) -> Map (String, Function)\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        ),
        (
            "function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Map (String, Function)\n  nested is fn (value : Int) -> Int\n    operation value\n  pairs : List (String, Function) is Entry ((\"operation\", nested), Empty)\n  collect-map pairs resolving reject\nwrap increment\n",
        ),
        (
            "map-function-capture",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", increment), Empty)\n  collect-map pairs resolving reject\nwrap is fn (candidate : Map (String, Function)) -> Map (String, Function)\n  nested is fn (value : Int) -> Int\n    map-lookup (candidate, \"operation\")\n      Some operation then operation value\n      None then 0\n  pairs : List (String, Function) is Entry ((\"nested\", nested), Empty)\n  collect-map pairs resolving reject\nwrap (source ())\n",
        ),
        (
            "repeated-identity",
            "use language (version is v0.1)\nmake is fn (offset : Int) -> Map (String, Function)\n  increase is fn (value : Int) -> Int\n    value + offset\n  pairs : List (String, Function) is Entry ((\"operation\", increase), Empty)\n  collect-map pairs resolving reject\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
        ),
        (
            "dynamic-key",
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", increment), Empty)\n  collect-map pairs resolving reject\nselect is fn (flag : Boolean) -> String\n  flag\n    true then \"operation\"\n    false then \"missing\"\nlookup is fn (candidate : Map (String, Function), key : String) -> Int\n  map-lookup (candidate, key)\n    Some operation then operation 1\n    None then 0\nlookup (source (), select true)\n",
        ),
        (
            "empty",
            "use language (version is v0.1)\nempty is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Empty\n  collect-map pairs resolving reject\nempty ()\n",
        ),
    ] {
        let rejected_path = directory.join(format!("{name}.t"));
        let rejected_executable = directory.join(name);
        fs::write(&rejected_path, rejected_source).unwrap();
        let rejected = run(topalc().args([
            "-o",
            rejected_executable.to_str().unwrap(),
            rejected_path.to_str().unwrap(),
        ]));
        assert!(!rejected.status.success());
        let diagnostic = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            diagnostic.contains("E-COMPILER-UNSUPPORTED"),
            "{diagnostic}"
        );
        assert!(!rejected_executable.exists());
        assert!(!metadata_path(&rejected_executable).exists());
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
            "break map-function-environments.t:24",
            "-ex",
            "break map-function-environments.t:58",
            "-ex",
            "disable 2",
            "-ex",
            "run",
            "-ex",
            "info args",
            "-ex",
            "backtrace",
            "-ex",
            "disable 1",
            "-ex",
            "enable 2",
            "-ex",
            "continue",
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
        "candidate=Map ((\"increment\", <fn increment>), (\"increase\", <fn increase>))",
        "candidate = Map ((\"increment\", <fn increment>), (\"increase\", <fn increase>))",
        "value = 1",
        "offset = 1",
        "@ context-offset = 40",
        "root live-offset = 1",
        "topal.fn.apply_2dincrease.",
        "topal.fn.increase.",
        "topal.main",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

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
