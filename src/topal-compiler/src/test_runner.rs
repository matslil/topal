use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use topal_compiler::{CompileError, CompileOptions, Emit, compile_source};
use topal_language::Session;

const SHARED_REGRESSIONS: &[&str] = &[
    "examples/language/aggregate-environments.t",
    "examples/language/anonymous-function-application.t",
    "examples/language/anonymous-function-captures.t",
    "examples/language/anonymous-list-functions.t",
    "examples/language/anonymous-product-functions.t",
    "examples/language/anonymous-product-pattern.t",
    "examples/language/arithmetic-error-codes.t",
    "examples/language/arbitrary-integer-arithmetic.t",
    "examples/language/array-function-environments.t",
    "examples/language/bindings-and-discard.t",
    "examples/language/boolean-decisions.t",
    "examples/language/boolean-logic.t",
    "examples/language/bound-anonymous-functions.t",
    "examples/language/callable-values.t",
    "examples/language/capability-composition.t",
    "examples/language/capturing-function-aggregate-boundaries.t",
    "examples/language/capturing-function-parameters.t",
    "examples/language/capturing-function-results.t",
    "examples/language/character-classification.t",
    "examples/language/collection-packaged-fields.t",
    "examples/language/comparison-decision-forms.t",
    "examples/language/comparison-decisions.t",
    "examples/language/completed-evidence.t",
    "examples/language/completion-effect-value.t",
    "examples/language/compound-packaged-function-operands.t",
    "examples/language/constraint-classifier.t",
    "examples/language/constraints-and-derived-capabilities.t",
    "examples/language/constructed-context.t",
    "examples/language/defining-context-forwarding.t",
    "examples/language/container-packaged-fields.t",
    "examples/language/custom-generator-boolean-values.t",
    "examples/language/custom-generator-comparison-values.t",
    "examples/language/custom-generator-character-return-parameter.t",
    "examples/language/custom-generator-character-return-result.t",
    "examples/language/custom-generator-close-code-pattern.t",
    "examples/language/custom-generator-close-handler.t",
    "examples/language/custom-generator-close.t",
    "examples/language/custom-generator-discard-between-yields.t",
    "examples/language/custom-generator-early-return.t",
    "examples/language/custom-generator-enum-values.t",
    "examples/language/custom-generator-final-decision.t",
    "examples/language/custom-generator-product-values.t",
    "examples/language/custom-generator-result-values.t",
    "examples/language/custom-generator-explicit-return.t",
    "examples/language/custom-generator-final-character.t",
    "examples/language/custom-generator-function-parameter.t",
    "examples/language/custom-generator-function-result.t",
    "examples/language/custom-generator-compound-function-boundaries.t",
    "examples/language/custom-generator-generic-function-boundaries.t",
    "examples/language/custom-generator-int-values.t",
    "examples/language/custom-generator-list-values.t",
    "examples/language/custom-generator-local-binding.t",
    "examples/language/custom-generator-local-close-handler.t",
    "examples/language/custom-generator-local-function.t",
    "examples/language/custom-generator-nat-values.t",
    "examples/language/custom-generator-nested-function-boundaries.t",
    "examples/language/custom-generator-nested-none-values.t",
    "examples/language/custom-generator-nested-optional-values.t",
    "examples/language/custom-generator-nested-result-values.t",
    "examples/language/custom-generator-optional-values.t",
    "examples/language/custom-generator-overloads.t",
    "examples/language/custom-generator-parameter-close.t",
    "examples/language/custom-generator-range-values.t",
    "examples/language/custom-generator-rational-values.t",
    "examples/language/custom-generator-recursive-nominal-values.t",
    "examples/language/custom-generator-resume-binding.t",
    "examples/language/custom-generator-return-after-yield.t",
    "examples/language/custom-generator-string-input.t",
    "examples/language/custom-generator-string-return.t",
    "examples/language/custom-generator-string-yield.t",
    "examples/language/custom-generator-suspension.t",
    "examples/language/custom-generator-unit-values.t",
    "examples/language/custom-multiple-yield-generator.t",
    "examples/language/custom-single-yield-generator.t",
    "examples/language/decreasing-int-recursion.t",
    "examples/language/decision-operand-expressions.t",
    "examples/language/diagnostic-controls.t",
    "examples/language/discard-function-pattern.t",
    "examples/language/dynamic-infinity-results.t",
    "examples/language/dynamic-rational-int-validation.t",
    "examples/language/dynamic-rational-construction.t",
    "examples/language/effect-classifier.t",
    "examples/language/effect-function-boundary.t",
    "examples/language/effect-identity.t",
    "examples/language/effect-list.t",
    "examples/language/effect-products.t",
    "examples/language/empty-block.t",
    "examples/language/empty-effects.t",
    "examples/language/equality-and-ordering.t",
    "examples/language/enum-decisions.t",
    "examples/language/enum-functions.t",
    "examples/language/enum-values.t",
    "examples/language/error-code-decisions.t",
    "examples/language/error-field-selection.t",
    "examples/language/escaping-nested-function-environments.t",
    "examples/language/exact-arithmetic.t",
    "examples/language/exact-numeric-absolute.t",
    "examples/language/exact-numeric-negate.t",
    "examples/language/exact-numeric-zero.t",
    "examples/language/exact-rational-int-narrowing.t",
    "examples/language/exact-three-way-comparison.t",
    "examples/language/expanded-callable-values.t",
    "examples/language/explicit-multi-parameter-decreases.t",
    "examples/language/exhaustive-boolean-decisions.t",
    "examples/language/exhaustive-error-code-decisions.t",
    "examples/language/external-layout-location.t",
    "examples/language/finite-exact-division-and-comparison.t",
    "examples/language/finite-range-observation.t",
    "examples/language/forward-function-declarations.t",
    "examples/language/fundamental-containers.t",
    "examples/language/function-aggregate-boundaries.t",
    "examples/language/function-aggregate-packaged-fields.t",
    "examples/language/function-classifier.t",
    "examples/language/function-call-chains.t",
    "examples/language/function-effect-bound.t",
    "examples/language/function-environment-boundaries.t",
    "examples/language/function-interface.t",
    "examples/language/function-local-shadowing.t",
    "examples/language/function-overloads.t",
    "examples/language/function-packaged-fields.t",
    "examples/language/function-result-chains.t",
    "examples/language/function-results.t",
    "examples/language/function-return.t",
    "examples/language/function-root-data.t",
    "examples/language/function-root-data-forwarding.t",
    "examples/language/function-value-boundary.t",
    "examples/language/generated-collect.t",
    "examples/language/generated-foreach.t",
    "examples/language/generator-error-codes.t",
    "examples/language/inclusive-int-ranges.t",
    "examples/language/increasing-int-recursion.t",
    "examples/language/infinity-arithmetic.t",
    "examples/language/infinity-values-and-ranges.t",
    "examples/language/int-checked-construction.t",
    "examples/language/int-euclidean-modulo.t",
    "examples/language/iterate-generator.t",
    "examples/language/iterate-take-while.t",
    "examples/language/layout-absence-policies.t",
    "examples/language/layout-access.t",
    "examples/language/layout-bit-order.t",
    "examples/language/layout-endian.t",
    "examples/language/layout-field-order.t",
    "examples/language/layout-packing.t",
    "examples/language/layout-payload-placement.t",
    "examples/language/lint-language-variant.t",
    "examples/language/list-boolean-values.t",
    "examples/language/list-containment.t",
    "examples/language/list-function-environments.t",
    "examples/language/map-function-environments.t",
    "examples/language/list-removal.t",
    "examples/language/list-sequence-operations.t",
    "examples/language/list-character-values.t",
    "examples/language/list-string-values.t",
    "examples/language/lists.t",
    "examples/language/local-function-environments.t",
    "examples/language/modular-checked-construction.t",
    "examples/language/modular-numbers.t",
    "examples/language/multiple-recursive-calls.t",
    "examples/language/mutual-increasing-int-recursion.t",
    "examples/language/mutual-int-recursion.t",
    "examples/language/mutual-multiple-recursive-calls.t",
    "examples/language/named-function-values.t",
    "examples/language/nat-equality-and-ordering.t",
    "examples/language/nat-functions.t",
    "examples/language/nat-checked-construction.t",
    "examples/language/nat-increasing-recursion.t",
    "examples/language/nat-mutual-increasing-recursion.t",
    "examples/language/nat-mutual-recursion.t",
    "examples/language/nat-recursion.t",
    "examples/language/namespace-alias-chain.t",
    "examples/language/namespace-alias.t",
    "examples/language/namespace-function-parameter.t",
    "examples/language/namespace-generator.t",
    "examples/language/namespace-overloads.t",
    "examples/language/namespace-snapshot.t",
    "examples/language/native-serialization.t",
    "examples/language/nested-anonymous-patterns.t",
    "examples/language/nested-functions.t",
    "examples/language/nested-lists.t",
    "examples/language/ordinary-functions.t",
    "examples/language/optional-function-environments.t",
    "examples/language/optional-rational-values.t",
    "examples/language/optional-result-composition.t",
    "examples/language/optional-values.t",
    "examples/language/overload-environments.t",
    "examples/language/overload-recursion-identity.t",
    "examples/language/packaged-function-association-order.t",
    "examples/language/packaged-function-operand.t",
    "examples/language/positive-recursion-steps.t",
    "examples/language/published-root-member.t",
    "examples/language/rational-exact-construction.t",
    "examples/language/rational-exponentiation.t",
    "examples/language/rational-infinity-values-and-ranges.t",
    "examples/language/rational-negative-exponent.t",
    "examples/language/rational-ranges.t",
    "examples/language/range-selection.t",
    "examples/language/record-function-boundaries.t",
    "examples/language/record-reconstruction.t",
    "examples/language/recursive-scalar-environments.t",
    "examples/language/repeated-anonymous-aggregate-patterns.t",
    "examples/language/repeated-anonymous-patterns.t",
    "examples/language/repeated-captured-function-aggregate-patterns.t",
    "examples/language/repeated-captured-function-patterns.t",
    "examples/language/repeated-captured-named-function-patterns.t",
    "examples/language/repeated-function-aggregate-patterns.t",
    "examples/language/repeated-sum-patterns.t",
    "examples/language/result-division-error.t",
    "examples/language/result-error-propagation.t",
    "examples/language/result-function-environments.t",
    "examples/language/result-negative-power-error.t",
    "examples/language/result-decisions.t",
    "examples/language/result-success.t",
    "examples/language/result-success-projection.t",
    "examples/language/root-namespace.t",
    "examples/language/scope-classifier.t",
    "examples/language/scope-packaged-fields.t",
    "examples/language/static-introspection.t",
    "examples/language/static-nullary-functions.t",
    "examples/language/static-product-functions.t",
    "examples/language/string-canonical-equality.t",
    "examples/language/string-case-fold.t",
    "examples/language/string-character-at.t",
    "examples/language/string-character-foreach.t",
    "examples/language/string-character-generator-close.t",
    "examples/language/string-character-generator-parameter.t",
    "examples/language/string-character-generator-result.t",
    "examples/language/string-character-traversal.t",
    "examples/language/string-construction.t",
    "examples/language/string-display-delimiters.t",
    "examples/language/string-exact-equality.t",
    "examples/language/string-lowercase.t",
    "examples/language/string-named-character-generator.t",
    "examples/language/string-normalization-nfd.t",
    "examples/language/string-normalization.t",
    "examples/language/string-uppercase.t",
    "examples/language/string-utf8-byte-count.t",
    "examples/language/strings-and-products.t",
    "examples/language/structured-packaged-function-fields.t",
    "examples/language/sum-equality.t",
    "examples/language/sum-function-environments.t",
    "examples/language/sum-packaged-function-fields.t",
    "examples/language/task-declaration-order.t",
    "examples/language/task-message-transactions.t",
    "examples/language/traversal-control.t",
    "examples/language/tuple-decision-results.t",
    "examples/language/tuple-equality.t",
    "examples/language/tuple-function-parameters.t",
    "examples/language/type-classifier.t",
    "examples/language/type-function-boundary.t",
    "examples/language/type-identity.t",
    "examples/language/type-values.t",
    "examples/language/unfold-collect.t",
    "examples/language/unfold-generator.t",
    "examples/language/unicode-identifiers.t",
    "examples/language/unions-and-recursive-products.t",
    "examples/language/unit-effect-value.t",
    "examples/language/use-namespace.t",
];

pub(crate) fn run(arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let arguments = Arguments::parse(arguments)?;
    let root = env::current_dir().map_err(|error| error.to_string())?;
    let tests = SHARED_REGRESSIONS
        .iter()
        .filter(|identity| {
            arguments
                .exact
                .as_deref()
                .is_none_or(|exact| exact == **identity)
        })
        .copied()
        .collect::<Vec<_>>();
    if tests.is_empty() {
        return Err("no compiler regressions matched".into());
    }
    if arguments.list {
        for identity in tests {
            println!("{identity}");
        }
        return Ok(());
    }

    let temporary = root.join("target/topalc-regressions");
    fs::create_dir_all(&temporary)
        .map_err(|error| format!("cannot create {}: {error}", temporary.display()))?;
    let mut failures = Vec::new();
    for identity in &tests {
        if let Err(error) = execute(identity, &root, &temporary, arguments.llvm_tools.as_deref()) {
            failures.push((identity, error));
        } else {
            println!("PASS {identity}");
        }
    }
    for (identity, error) in &failures {
        println!("FAIL {identity}");
        eprintln!("{error}");
    }
    println!(
        "{} compiler regressions: {} passed; {} failed",
        tests.len(),
        tests.len() - failures.len(),
        failures.len()
    );
    if failures.is_empty() {
        Ok(())
    } else {
        Err("compiler regression run failed".into())
    }
}

struct Arguments {
    exact: Option<String>,
    list: bool,
    llvm_tools: Option<PathBuf>,
}

impl Arguments {
    fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut exact = None;
        let mut list = false;
        let mut llvm_tools = None;
        let mut arguments = arguments.peekable();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--exact" => exact = Some(arguments.next().ok_or("--exact requires an identity")?),
                "--list" => list = true,
                "--llvm-tools" => {
                    llvm_tools = Some(PathBuf::from(
                        arguments
                            .next()
                            .ok_or("--llvm-tools requires a directory")?,
                    ));
                }
                option => return Err(format!("unknown compiler-test option: {option}")),
            }
        }
        Ok(Self {
            exact,
            list,
            llvm_tools,
        })
    }
}

fn execute(
    identity: &str,
    root: &Path,
    temporary: &Path,
    llvm_tools: Option<&Path>,
) -> Result<(), String> {
    let path = root.join(identity);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let expected = Session::new()
        .evaluate_source_file(&source, &mut std::io::sink())
        .map_err(|diagnostic| diagnostic.render(identity))?
        .to_string()
        + "\n";
    let executable = temporary.join(identity.replace(['/', '\\'], "-") + ".bin");
    let options = CompileOptions {
        source_name: identity.into(),
        output: executable.clone(),
        emit: Emit::Executable,
        llvm_tools: llvm_tools.map(Path::to_owned),
    };
    compile_source(&source, &options).map_err(|error| render_error(error, identity))?;
    let output = Command::new(&executable)
        .output()
        .map_err(|error| format!("cannot execute {}: {error}", executable.display()))?;
    if !output.status.success() {
        return Err(format!(
            "compiled executable exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let actual = String::from_utf8(output.stdout)
        .map_err(|error| format!("compiled executable emitted non-UTF-8 output: {error}"))?;
    if actual != expected {
        return Err(format!(
            "interpreter: {expected:?}\ncompiler:    {actual:?}"
        ));
    }
    Ok(())
}

fn render_error(error: CompileError, identity: &str) -> String {
    match error {
        CompileError::Diagnostic(diagnostic) => diagnostic.render(identity),
        error => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use super::SHARED_REGRESSIONS;

    #[test]
    fn shared_regression_manifest_covers_the_entire_canonical_corpus() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let language_examples = root.join("examples/language");
        let canonical = fs::read_dir(&language_examples)
            .expect("canonical language regression directory must be readable")
            .map(|entry| {
                entry
                    .expect("language regression entry must be readable")
                    .path()
            })
            .filter(|path| path.extension().is_some_and(|extension| extension == "t"))
            .map(|path| {
                path.strip_prefix(&root)
                    .expect("language regression must be below the repository root")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        let listed = SHARED_REGRESSIONS
            .iter()
            .map(|identity| (*identity).to_owned())
            .collect::<Vec<_>>();
        let unique = listed.iter().collect::<BTreeSet<_>>();
        let canonical = canonical.iter().collect::<BTreeSet<_>>();

        assert_eq!(unique.len(), listed.len(), "manifest contains duplicates");
        assert_eq!(unique, canonical);
    }
}
