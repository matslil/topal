fn language_example(name: &str) -> String {
    format!(
        "{}/../../examples/language/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn run_debugger_commands(source: &str, commands: &str) -> String {
    let library_root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../library");
    let mut child = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args(["--library-root", library_root, "--script", "-", source])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            format!("use language ( version is v0.1, features is ( debug ) )\n{commands}\nquit\n")
                .as_bytes(),
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

fn run_basic_commands(commands: &str) -> String {
    run_debugger_commands(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/debugger/basic-history.t"
        ),
        commands,
    )
}

#[test]
fn startup_continue_until_and_run_have_live_source_semantics() {
    let stdout = run_basic_commands("continue\nprint\nrun\nuntil 8\nprint\nrun\nuntil true");
    assert!(stdout.contains("basic-history.t:2:1"));
    assert!(stdout.contains("application finished"));
    assert!(stdout.matches("basic-history.t:2:1").count() >= 3);
    assert!(stdout.contains("basic-history.t:8:1"));
    assert!(stdout.contains("\n42\n"));
    assert!(stdout.contains("\n40\n"));
}

#[test]
fn language_selection_is_the_initial_position_and_built_in_step_boundary() {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/data-transfer/packet-filter.t"
    );
    for command in ["next", "step"] {
        let stdout = run_debugger_commands(source, command);
        let language = stdout.find("packet-filter.t:2:1").unwrap();
        let library = stdout.find("packet-filter.t:5:1").unwrap();
        assert!(language < library, "{command} output:\n{stdout}");
        assert!(!stdout.contains("packet-filter.t:1:1"));
    }
}

#[test]
fn every_language_example_executes_through_the_debugger() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language");
    let mut examples = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "t"))
        .collect::<Vec<_>>();
    examples.sort();
    assert_eq!(examples.len(), 309);
    let commands = "use language ( version is v0.1, features is ( debug ) )\ncontinue\nquit\n";
    for example in examples {
        let mut child = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
            .args(["--script", "-"])
            .arg(&example)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(commands.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{} failed in debugger:\n{}",
            example.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn executes_the_standard_library_example_from_its_shared_module_tree() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../library");
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../library/minimum.debug");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args(["--script", script, root])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("decision #"));
    assert!(stdout.contains("evaluation.result"));
    assert!(stdout.contains("((Int, Rational, (Int, Int), (Int, Int), Int, Int, Int, Rational, Optional Int, Boolean, Optional (Int, String), Rational, Int, Boolean, Rational, Int, Int, (Int, Int), Range Int, String, String, Boolean, Optional Int, List Int, List Int, List Int))"));
}

#[test]
fn declared_standard_library_file_executes_directly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../library");
    let rest = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/data-transfer/rest-controller.t"
    );
    let advent_of_code = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/advent-of-code/2025/day10-part1.t"
    );
    for source in [rest, advent_of_code] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
            .args(["--library-root", root, "--script", "-", source])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"use language ( version is v0.1, features is ( debug ) )\ncontinue\nquit\n")
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn step_enters_declared_library_while_next_stays_in_the_current_file() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../library");
    let rest = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/data-transfer/rest-controller.t"
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args(["--library-root", root, "--script", "-", rest])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b"use language ( version is v0.1, features is ( debug ) )\nnext\nstep\nbacktrace\nbreak 10\nbreakpoints\ncontinue\nnext\nreverse-next\nfinish\nquit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("examples/data-transfer/rest-controller.t:5:1"));
    assert!(stdout.contains("library/advent-of-code/module.t:10:17"));
    assert!(stdout.contains("#0 <dependency> at"));
    assert!(stdout.contains("library/advent-of-code/module.t:10:17"));
    assert!(stdout.contains("#1 <script> at"));
    assert!(stdout.contains("breakpoint set at line 10 in"));
    assert!(stdout.contains("library/advent-of-code/module.t:10"));
    assert!(stdout.contains("pub revision is 1"));
    assert!(
        stdout
            .matches("examples/data-transfer/rest-controller.t:5:1")
            .count()
            >= 2
    );
    assert!(!stdout.contains("rest-controller.t:12:50"));
    assert!(!stdout.contains('^'));
    assert!(!stdout.contains("error["));
}

fn language_diagnostic(name: &str) -> String {
    format!(
        "{}/../../examples/language-diagnostics/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn navigates_recorded_execution_in_both_directions() {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/debugger/basic-history.t"
    );
    let commands = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/debugger/basic-history.debug"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args(["--script", commands, source])
        .output()
        .expect("debugger should run script");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("#0 context.selected [TOPAL-SYN-UNICODE-001]"));
    assert!(stdout.contains(
        "decision #0: context.selected because TOPAL-SYN-UNICODE-001 (design-0;Unicode=17.0.0)"
    ));
    assert!(stdout.contains("#1 source.accepted [TOPAL-SYN-SOURCE-001]"));
    assert!(stdout.contains("> #0 context.selected"));
    assert!(stdout.contains("no value at current execution state"));
    assert!(stdout.contains("basic-history.t:8:1"));
    assert!(stdout.contains("answer is 40"));
    assert!(stdout.contains("breakpoint set at line 7"));
    assert!(stdout.contains("breakpoint set at line 8"));
    assert!(stdout.contains("breakpoint removed from line 7"));
    assert!(stdout.contains("watchpoint set for answer"));
    assert!(stdout.contains("watchpoint removed for answer"));
    assert!(stdout.contains("checkpoint result saved"));
    assert!(stdout.contains("checkpoint result restored"));
    assert!(stdout.contains("checkpoint result deleted"));
    assert!(stdout.contains("#0 <script> before first statement"));
    assert!(stdout.contains("#0 <script> at"));
}

#[test]
fn executes_the_debuggee_only_when_commands_advance_it() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}live-execution.debug"),
            &format!("{root}basic-history.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let before = stdout.find("no value at current execution state").unwrap();
    let binding = stdout.find("answer = 40").unwrap();
    let result = stdout.rfind("\n42\n").unwrap();
    assert!(before < binding && binding < result);
}

#[test]
fn retains_inspectable_history_when_live_execution_fails() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}failing-history.debug"),
            &format!("{root}failing-history.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("error[E-UNBOUND-NAME]: name is not bound"));
    assert!(stdout.contains("answer = 40"));
    assert!(stdout.contains("binding.bind [TOPAL-SYN-BIND-001] answer"));
    assert!(stdout.contains("no value at current execution state"));
}

#[test]
fn exposes_intermediate_expression_values_as_reversible_states() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}expression-stepping.debug"),
            &format!("{root}expression-stepping.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for value in ["\n40\n", "\n41\n", "\n42\n"] {
        assert!(
            stdout.contains(value),
            "missing intermediate value {value:?}"
        );
    }
}

#[test]
fn records_reversible_static_function_call_decisions() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("static-nullary-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let declaration = stdout.find("function.declared").unwrap();
    let entered = stdout.find("function.entry").unwrap();
    let body = stdout.find("root.+(Int,Int)").unwrap();
    let returned = stdout.find("function.exit").unwrap();
    assert!(declaration < entered && entered < body && body < returned);
    assert!(stdout.contains("(42, 42)"));
}

#[test]
fn records_reversible_static_function_argument_binding() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("static-nullary-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let bound = stdout.find("function.argument.bound").unwrap();
    let entered = bound + stdout[bound..].find("function.entry").unwrap();
    let body = bound + stdout[bound..].find("root.+(Int,Int)").unwrap();
    let returned = bound + stdout[bound..].find("function.exit").unwrap();
    assert!(bound < entered && entered < body && body < returned);
    assert!(stdout.contains("(42, 42)"));
}

#[test]
fn records_reversible_static_product_argument_bindings() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("static-product-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let left = stdout
        .find("function.argument.bound [TOPAL-FUNCTION-STATIC-BINARY-001] left")
        .unwrap();
    let right = stdout
        .find("function.argument.bound [TOPAL-FUNCTION-STATIC-BINARY-001] right")
        .unwrap();
    let entered = stdout.find("function.entry").unwrap();
    let created = stdout
        .find("binding.bind [TOPAL-SYN-BIND-001] sum")
        .unwrap();
    let resolved = stdout
        .find("binding.resolved [TOPAL-SYN-BIND-001] sum")
        .unwrap();
    assert!(left < right && right < entered && entered < created && created < resolved);
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_explicit_function_return() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("function-return.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let explicit = stdout.find("function.return.explicit").unwrap();
    let returned = stdout.find("function.exit").unwrap();
    assert!(explicit < returned);
    assert!(!stdout.contains("binding.resolved [TOPAL-SYN-BIND-001] missing"));
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_ordinary_function_execution() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("ordinary-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("function.entry [TOPAL-FUNCTION-ORDINARY-001] subtract"));
    assert!(stdout.contains("function.exit [TOPAL-FUNCTION-ORDINARY-001] subtract"));
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_nat_function_execution() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("nat-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("identity (Nat)"));
    assert!(stdout.contains("function.entry [TOPAL-FUNCTION-ORDINARY-001] identity"));
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_nat_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("nat-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-NAT-001"));
    assert!(stdout.contains("function.recursion.descended"));
    assert!(stdout.contains("\n2\n"));
}

#[test]
fn records_reversible_increasing_nat_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("nat-increasing-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"));
    assert!(stdout.contains("function.recursion.descended"));
    assert!(stdout.contains("\n6\n"));
}

#[test]
fn records_reversible_mutual_nat_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("nat-mutual-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"));
    assert!(stdout.contains("function.recursion.cycle.proven"));
    assert!(stdout.contains("\n(true, false)\n"));
}

#[test]
fn records_reversible_mutual_increasing_nat_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("nat-mutual-increasing-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"));
    assert!(stdout.contains("function.recursion.cycle.proven"));
    assert!(stdout.contains("\n(true, false)\n"));
}

#[test]
fn records_reversible_enum_values() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("enum-values.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("enum.declared [TOPAL-TYPE-ENUM-001] Color"));
    assert!(stdout.contains("\n(Red, Green, true, false)\n"));
}

#[test]
fn records_reversible_enum_function_classification() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("enum-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("identity (Color)"));
    assert!(stdout.contains("\n(Red, Green)\n"));
}

#[test]
fn records_reversible_exhaustive_enum_decision() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("enum-decisions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-DECISION-ENUM-001"));
    assert!(stdout.contains("decision.rule.considered"));
    assert!(stdout.contains("\n(\"red\", \"green\")\n"));
}

#[test]
fn records_reversible_arithmetic_error_code_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("arithmetic-error-codes.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(
            "namespace.member.selected [TOPAL-NUM-ARITHMETIC-ERROR-001] division-by-zero"
        )
    );
    assert!(stdout.contains("\n(division-by-zero, indeterminate, true)\n"));
}

#[test]
fn records_reversible_successful_result_contract() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("result-success.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("function.result.contract [TOPAL-TYPE-RESULT-001]"));
    assert!(stdout.contains("\nRational ( 3, 2 )\n"));
}

#[test]
fn records_reversible_dynamic_division_error() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("result-division-error.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result.error.constructed [TOPAL-TYPE-RESULT-001]"));
    assert!(
        stdout.contains("Error ( domain is root./(Rational,Rational), code is division-by-zero )")
    );
}

#[test]
fn records_reversible_negative_rational_exponentiation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("rational-negative-exponent.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RAT-NEG-POW-001"));
    assert!(stdout.contains("\nRational ( 4, 9 )\n"));
}

#[test]
fn records_reversible_dynamic_negative_power_error() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("result-negative-power-error.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.^(Rational,Int);division-by-zero"));
    assert!(stdout.contains("Error ( domain is root.^(Rational,Int), code is division-by-zero )"));
}

#[test]
fn records_reversible_result_error_propagation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("result-error-propagation.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result.error.constructed"));
    assert!(stdout.contains("result.error.propagated"));
    assert!(stdout.contains("domain=root./(Rational,Rational);code=division-by-zero"));
}

#[test]
fn records_reversible_result_decisions() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("result-decisions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-DECISION-RESULT-001"));
    assert!(stdout.contains("result.payload.bound"));
    assert!(stdout.contains("\n(\"ok\", \"error\")\n"));
}

#[test]
fn records_reversible_error_field_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("error-field-selection.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-ERROR-FIELD-001"));
    assert!(stdout.contains("error.field.selected"));
    assert!(stdout.contains("(division-by-zero, root./(Rational,Rational))"));
}

#[test]
fn records_reversible_error_code_decisions() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("error-code-decisions.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-DECISION-ERROR-CODE-001"));
    assert!(stdout.contains("error.code.matched"));
    assert!(stdout.contains("(\"ok\", \"division by zero\")"));
}

#[test]
fn records_reversible_result_success_projection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("result-success-projection.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-RESULT-PROJECT-001"));
    assert!(stdout.contains("result.success.projected"));
    assert!(stdout.contains("result.error.projected"));
}

#[test]
fn records_reversible_exhaustive_error_code_decision() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("exhaustive-error-code-decisions.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-DECISION-ERROR-CODE-001"));
    assert!(stdout.contains("error.code.matched"));
    assert!(stdout.contains("(\"ok\", \"zero\")"));
}

#[test]
fn records_reversible_character_classification() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("character-classification.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("function.argument.bound"));
    assert!(stdout.contains("string.from-character"));
    assert!(stdout.contains("(\"🙂\", \"a\u{301}\", true, true, true)"));
}

#[test]
fn records_reversible_int_euclidean_modulo() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("int-euclidean-modulo.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-INT-MODULO-001"));
    assert!(stdout.contains("TOPAL-NUM-INT-QUOTIENT-MODULO-001"));
    assert!(stdout.contains("root.%(Int,Int);division-by-zero"));
    assert!(stdout.contains("Error ( domain is root.%(Int,Int), code is division-by-zero )"));
    assert!(stdout.contains("Error ( domain is root./%(Int,Int), code is division-by-zero )"));
}

#[test]
fn records_reversible_exact_numeric_absolute() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("exact-numeric-absolute.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.absolute(Int)"));
    assert!(stdout.contains("root.absolute(Rational)"));
    assert!(stdout.contains("(42, 42, Rational ( 5, 4 ), Rational ( 5, 4 ))"));
}

#[test]
fn records_reversible_named_exact_numeric_negation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("exact-numeric-negate.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.negate(Int)"));
    assert!(stdout.contains("root.negate(Rational)"));
    assert!(stdout.contains("(-42, 42, Rational ( -5, 4 ), Rational ( 5, 4 ))"));
}

#[test]
fn records_reversible_exact_numeric_zero() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("exact-numeric-zero.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.zero(Int)"));
    assert!(stdout.contains("root.zero(Rational)"));
    assert!(stdout.contains("root.one(Int)"));
    assert!(stdout.contains("root.zero(Nat)"));
    assert!(stdout.contains("root.one(Nat)"));
    assert!(stdout.contains("root.one(Rational)"));
    assert!(stdout.contains("(0, 0, Rational ( 0, 1 ), 1, 1, Rational ( 1, 1 ))"));
}

#[test]
fn records_reversible_exact_three_way_comparison() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("exact-three-way-comparison.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-THREE-WAY-COMPARE-001"));
    assert!(stdout.contains("Int->Rational:left"));
    assert!(stdout.contains("TOPAL-DECISION-ENUM-001"));
    assert!(stdout.contains("(Less, Equal, Greater, Less, \"less\", \"equal\", \"greater\")"));
}

#[test]
fn records_reversible_exact_rational_int_narrowing() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("exact-rational-int-narrowing.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RATIONAL-INT-EXACT-001"));
    assert!(stdout.contains("Rational->Int:exact"));
    assert!(stdout.contains("(50, -3)"));
}

#[test]
fn records_reversible_dynamic_rational_int_validation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("dynamic-rational-int-validation.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RATIONAL-INT-VALIDATE-001"));
    assert!(stdout.contains("Rational->Int:validated"));
    assert!(stdout.contains("root.Int(Rational);not-representable"));
    assert!(
        stdout.contains("(50, Error ( domain is root.Int(Rational), code is not-representable ))")
    );
}

#[test]
fn records_reversible_checked_int_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("int-checked-construction.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-INT-CONSTRUCT-001"));
    assert!(stdout.contains("Int->Int:identity"));
    assert!(stdout.contains("Rational->Int:exact"));
    assert!(stdout.contains("root.Int(Rational);not-representable"));
    assert!(
        stdout
            .contains("(7, 6, Error ( domain is root.Int(Rational), code is not-representable ))")
    );
}

#[test]
fn records_reversible_checked_nat_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("nat-checked-construction.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-NAT-CONSTRUCT-001"));
    assert!(stdout.contains("Int->Nat:nonnegative"));
    assert!(stdout.contains("root.Nat(Int);out-of-range"));
    assert!(stdout.contains("(7, 6, Error ( domain is root.Nat(Int), code is out-of-range ))"));
}

#[test]
fn records_reversible_canonical_rational_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("rational-exact-construction.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-001"));
    assert!(stdout.contains("numeric.rational.constructed"));
    assert!(stdout.contains("Int->Rational:explicit"));
    assert!(
        stdout.contains(
            "(Rational ( 7, 1 ), Rational ( 1, 2 ), Rational ( -1, 2 ), Rational ( 0, 1 ))"
        )
    );
}

#[test]
fn records_reversible_dynamic_rational_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("dynamic-rational-construction.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001"));
    assert!(stdout.contains("root.Rational(Int,Int);division-by-zero"));
    assert!(stdout.contains("root.Rational(Int,Int);indeterminate"));
    assert!(stdout.contains("Rational ( 1, 2 )"));
}

#[test]
fn records_reversible_int_range_endpoint_forms() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("inclusive-int-ranges.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-RANGE-BOUNDS-001"));
    assert!(stdout.contains("TOPAL-RANGE-MEMBERSHIP-001"));
    assert!(stdout.contains("range.constructed"));
    assert!(stdout.contains("range.membership.tested"));
    assert!(stdout.contains("TOPAL-RANGE-INTERSECTION-001"));
    assert!(stdout.contains("(0 ..= 10, 0 .. 10, 0 <.. 10, 0 <..= 10, true, false, false, false, false, true, 5 ..= 10, 20 ..= 10, true, false, false)"));
}

#[test]
fn records_reversible_rational_ranges() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("rational-ranges.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Int->Rational:left"));
    assert!(stdout.contains("Int->Rational:membership"));
    assert!(stdout.contains("Rational ( 0, 1 ) ..= Rational ( 5, 2 )"));
    assert!(stdout.contains("TOPAL-RANGE-INTERSECTION-001"));
}

#[test]
fn records_reversible_boolean_not() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("boolean-logic.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-BOOLEAN-LOGIC-001"));
    assert!(stdout.contains("root.not(Boolean)"));
    assert!(stdout.contains("root.and(Boolean,Boolean)"));
    assert!(stdout.contains("and:eager"));
    assert!(stdout.contains("root.or(Boolean,Boolean)"));
    assert!(stdout.contains("or:eager"));
    assert!(stdout.contains("root.xor(Boolean,Boolean)"));
    assert!(stdout.contains("xor:eager"));
    assert!(stdout.contains("(false, true, true, false, false, false, true, true, true, false, false, true, true, false)"));
}

#[test]
fn records_reversible_explicit_optional_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("optional-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-OPTIONAL-CONSTRUCT-001"));
    assert!(stdout.contains("optional.some.constructed"));
    assert!(stdout.contains("optional.none.constructed"));
    assert!(stdout.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001"));
    assert!(stdout.contains("preserve"));
    assert!(stdout.contains("absent"));
    assert!(stdout.contains("TOPAL-DECISION-OPTIONAL-001"));
    assert!(stdout.contains("optional.payload.bound"));
    assert!(stdout.contains("TOPAL-TYPE-OPTIONAL-EQUALITY-001"));
    assert!(stdout.contains(
        "(Some 42, Some \"present\", None, None, None, Some 7, None, None, \"present\", \"absent\", true, true, false, true)"
    ));
}

#[test]
fn records_reversible_string_character_indexing() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-character-at.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTER-AT-001"));
    assert!(stdout.contains("string.character-at"));
    assert!(stdout.contains("Some \"👩‍🔬\""));
    assert!(stdout.contains("None, None"));
    assert!(stdout.contains("TOPAL-DECISION-OPTIONAL-001"));
    assert!(stdout.contains("TOPAL-STRING-FROM-CHARACTER-001"));
    assert!(stdout.contains("\"👩‍🔬\", \"missing\""));
}

#[test]
fn records_reversible_universal_unicode_uppercase() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-uppercase.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-UPPER-001"));
    assert!(stdout.contains("string.uppercased"));
    assert!(stdout.contains("\"STRASSE ΣΣ\""));
}

#[test]
fn records_reversible_universal_unicode_lowercase() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-lowercase.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-LOWER-001"));
    assert!(stdout.contains("string.lowercased"));
    assert!(stdout.contains("\"i\u{307}ς\""));
}

#[test]
fn records_reversible_full_unicode_case_folding() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-case-fold.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CASE-FOLD-001"));
    assert!(stdout.contains("string.case-folded"));
    assert!(stdout.contains("\"strasse σσ\""));
}

#[test]
fn records_reversible_canonical_string_equality() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-canonical-equality.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CANONICAL-EQUALITY-001"));
    assert!(stdout.contains("string.canonical-equality.compared"));
    assert!(stdout.contains("(false, true, false)"));
}

#[test]
fn records_reversible_character_traversal_collection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-character-traversal.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-COLLECT-001"));
    assert!(stdout.contains("generator.yielded"));
    assert!(stdout.contains("\"a\u{301}👩‍🔬🇸🇪\""));
}

#[test]
fn records_reversible_character_generator_foreach() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-character-foreach.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-FOREACH-001"));
    assert!(stdout.contains("generator.yielded"));
    assert!(stdout.contains("generator.resumed"));
    assert!(stdout.contains("generator.returned"));
}

#[test]
fn records_reversible_named_character_generator_consumption() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-named-character-generator.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-GENERATOR-001"));
    assert!(stdout.contains("generator.started"));
    assert!(stdout.contains("generator.consumed"));
}

#[test]
fn records_reversible_returned_character_generator() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-character-generator-result.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-RESULT-001"));
    assert!(stdout.contains("function.exit"));
    assert!(stdout.contains("generator.consumed"));
}

#[test]
fn records_reversible_generator_parameter_transfer() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-character-generator-parameter.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-PARAMETER-001"));
    assert!(stdout.contains("generator.parameter.transferred"));
    assert!(stdout.contains("generator.consumed"));
}

#[test]
fn records_reversible_abandoned_generator_close() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("string-character-generator-close.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-CLOSE-001"));
    assert!(stdout.contains("generator.closed"));
    assert!(stdout.contains("domain=root;code=generator-closed;generator=root.characters"));
}

#[test]
fn records_reversible_generator_error_code_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("generator-error-codes.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-ERROR-CODE-001"));
    assert!(stdout.contains("generator-closed"));
}

#[test]
fn records_custom_multiple_yield_generator_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-multiple-yield-generator.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-DECLARATION-001"));
    assert!(stdout.contains("generator.declared"));
    assert!(stdout.contains("generator.started"));
    assert!(stdout.contains("generator.yielded"));
    assert_eq!(stdout.matches("generator.yielded").count(), 2);
    assert_eq!(stdout.matches("generator.resumed").count(), 2);
}

#[test]
fn records_custom_generator_local_binding_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-local-binding.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("binding.bind"));
    assert!(stdout.contains("generator.yielded"));
    assert!(stdout.contains("TOPAL-GENERATOR-FOREACH-001"));
}

#[test]
fn records_generator_return_before_yield_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-early-return.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("generator.yielded"));
    assert!(stdout.contains("generator.returned"));
    assert!(stdout.contains("TOPAL-GENERATOR-EARLY-RETURN-001"));
}

#[test]
fn records_distinct_generator_final_character_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-final-character.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
    assert!(stdout.contains("generator.yielded"));
    assert!(stdout.contains("\"R\""));
}

#[test]
fn records_custom_generator_suspension_order_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-suspension.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("generator.suspended").count(), 2);
    assert!(stdout.contains("TOPAL-GENERATOR-SUSPEND-001"));
}

#[test]
fn records_unit_resume_binding_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-resume-binding.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-RESUME-BINDING-001"));
    assert!(stdout.contains("generator.resume.bound"));
}

#[test]
fn records_abandoned_custom_generator_close_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-close.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-CLOSE-001"));
    assert!(stdout.contains("domain=root;code=generator-closed;generator=root.pause-once"));
}

#[test]
fn records_custom_generator_close_handler_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-close-handler.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-CLOSE-HANDLER-001"));
    assert!(stdout.contains("generator.close.bound"));
    assert!(stdout.contains("decision.rule.selected"));
}

#[test]
fn records_qualified_generator_close_code_match_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-close-code-pattern.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001"));
    assert!(stdout.contains("generator.error.code.matched"));
}

#[test]
fn records_custom_generator_function_result_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-function-result.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"));
    assert!(stdout.contains("generator.result.transferred"));
    assert!(stdout.contains("generator.yielded"));
}

#[test]
fn records_custom_generator_function_parameter_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-function-parameter.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"));
    assert!(stdout.contains("generator.parameter.transferred"));
    assert!(stdout.contains("generator.yielded"));
}

#[test]
fn records_unconsumed_custom_generator_parameter_close_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("custom-generator-parameter-close.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"));
    assert!(stdout.contains("TOPAL-GENERATOR-CLOSE-001"));
    assert!(stdout.contains("generator.closed"));
}
