//! Shared filesystem module loading for interpreters and compiler frontends.

use std::fs;
use std::path::Path;

use crate::{Session, TraceSink};
use topal_source::SourceText;
use topal_syntax::{Statement, lex, parse};

/// Test whether a source file explicitly declares one library identity.
#[must_use]
pub fn declares_library(source: &str, identity: &str) -> bool {
    let Ok(source) = SourceText::new(source) else {
        return false;
    };
    parse(&source, &lex(&source))
        .statements
        .iter()
        .any(|statement| {
            matches!(statement, Statement::LibrarySelection { name, version, .. }
            if source.slice(*name) == identity && source.slice(*version) == "v0.1")
        })
}

/// Test whether a source file declares the conventional string-input
/// application entry point.
#[must_use]
pub fn declares_string_solver(source: &str) -> bool {
    fn is_string_solver(statement: &Statement, source: &SourceText) -> bool {
        match statement {
            Statement::Function {
                name, parameters, ..
            } => {
                source.slice(*name) == "solve"
                    && parameters.len() == 1
                    && parameters[0].fields.is_empty()
                    && source.slice(parameters[0].classifier) == "String"
            }
            Statement::Published { declaration, .. } => is_string_solver(declaration, source),
            _ => false,
        }
    }

    let Ok(source) = SourceText::new(source) else {
        return false;
    };
    parse(&source, &lex(&source))
        .statements
        .iter()
        .any(|statement| is_string_solver(statement, &source))
}

/// Load every ordinary source module and constructed child module below one
/// package directory into an existing session.
///
/// Special facade and context files are deliberately excluded. The caller
/// selects and evaluates `package.t`, `library.t`, or `application.t` according
/// to its tool role after the shared module graph has been constructed.
///
/// # Errors
///
/// Returns a rendered source diagnostic or filesystem error without attaching
/// a partially loaded child module to its parent.
pub fn load_module_tree(
    session: &mut Session,
    directory: &Path,
    trace: &mut impl TraceSink,
) -> Result<(), String> {
    let mut paths = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    paths.sort();
    for path in &paths {
        if path.is_dir() {
            load_child_module(session, path, trace)?;
        } else if is_ordinary_source(path) {
            let name = path
                .file_stem()
                .ok_or_else(|| format!("source {} has no module name", path.display()))?
                .to_string_lossy();
            let source = read_source(path)?;
            let source_name = path.display().to_string();
            trace.push_source(&source_name, &source);
            let result = session
                .load_module(&name, &source, trace)
                .map_err(|error| error.render(&source_name));
            trace.pop_source();
            result?;
        }
    }
    Ok(())
}

fn load_child_module(
    parent: &mut Session,
    path: &Path,
    trace: &mut impl TraceSink,
) -> Result<(), String> {
    let mut child = Session::new();
    let descriptor = path.join("module.t");
    if descriptor.is_file() {
        let source = read_source(&descriptor)?;
        let source_name = descriptor.display().to_string();
        trace.push_source(&source_name, &source);
        let result = child
            .evaluate_source_file(&source, trace)
            .map_err(|error| error.render(&source_name));
        trace.pop_source();
        result?;
    }
    load_module_tree(&mut child, path, trace)?;
    let name = path
        .file_name()
        .ok_or_else(|| format!("module {} has no name", path.display()))?
        .to_string_lossy();
    parent
        .attach_module(&name, child, trace)
        .map(|_| ())
        .map_err(|error| error.render(&path.display().to_string()))
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn is_ordinary_source(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "t")
        && path.file_name().is_some_and(|name| {
            !matches!(
                name.to_str(),
                Some("application.t" | "package.t" | "library.t" | "module.t")
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use num_bigint::BigInt;

    #[test]
    fn string_solver_detection_requires_the_application_entry_shape() {
        assert!(declares_string_solver(
            "use language ( version is v0.1 )\nsolve is fn (input : String) -> Nat\n  entry-count input\n"
        ));
        assert!(declares_string_solver(
            "use language ( version is v0.1 )\npub solve is fn (input : String) -> Nat\n  entry-count input\n"
        ));
        assert!(!declares_string_solver(
            "use language ( version is v0.1 )\nsolve is fn (input : Nat) -> Nat\n  input\n"
        ));
        assert!(!declares_string_solver(
            "use language ( version is v0.1 )\nresult is \"solve is fn (input : String)\"\n"
        ));
    }

    #[test]
    fn shared_loader_executes_the_first_library_definition() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let undeclared = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nstd min (4, 2)",
                &mut trace,
            )
            .unwrap_err();
        assert_eq!(undeclared.code, "E-UNDECLARED-LIBRARY");
        let value = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nuse library std ( version is v0.1 )\nmin is std min\nmin (4, 2)",
                &mut trace,
            )
            .unwrap();
        assert_eq!(value, Value::Int(BigInt::from(2)));
        assert!(trace.iter().any(|event| event.contains("module.loaded")));
        let no_leaked_declaration = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nstd min (4, 2)",
                &mut trace,
            )
            .unwrap_err();
        assert_eq!(no_leaked_declaration.code, "E-UNDECLARED-LIBRARY");
        let duplicate = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.1 )\nuse library std ( version is v0.1 )\nuse library std ( version is v0.1 )\n()",
                &mut trace,
            )
            .unwrap_err();
        assert_eq!(duplicate.code, "E-DUPLICATE-LIBRARY");
        let unavailable = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.1 )\nuse library other ( version is v0.1 )\n()",
                &mut trace,
            )
            .unwrap_err();
        assert_eq!(unavailable.code, "E-UNSUPPORTED-LIBRARY");
    }

    #[test]
    fn shared_library_ordering_retains_one_exact_generic_type() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let mut boundary_session = session.clone();
        let value = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nuse library std ( version is v0.1 )\nmin is std min\nmin ((1, 3), (1, 2))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value,
            Value::Tuple(vec![
                Value::Int(BigInt::from(1)),
                Value::Int(BigInt::from(2))
            ])
        );

        let error = session
            .evaluate_source_file("use language ( version is v0.1 )\nmin (1, 1.0)", &mut trace)
            .unwrap_err();
        assert_eq!(error.code, "E-FUNCTION-ARGUMENT-TYPE");

        let boundaries = boundary_session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nuse library std ( version is v0.1 )\nmin is std min\nmax is std max\nmin-max is std min-max\n(min (2, 4), min (4, 2), min (2, 2), max (2.0, 4.0), max (4.0, 2.0), max (2.0, 2.0), min-max (4, 2), min-max (2, 4), min-max (2, 2))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            boundaries.to_string(),
            "(2, 2, 2, Rational ( 4, 1 ), Rational ( 4, 1 ), Rational ( 2, 1 ), (2, 4), (2, 4), (2, 2))"
        );
    }

    const INVALID_STANDARD_LIBRARY_CASES: &[(&str, &str)] = &[
        (
            "std statistics covariance ((one 1), (Empty Int))",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std statistics quantile ((one 1), (Rational 2))",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std sequence chunks ((one 1), 0)",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std sequence windows ((one 1), 0)",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std replace-all (\"text\", \"\", \"x\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern count (\"text\", \"\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern split (\"text\", \"\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern replace-all (\"text\", (\"\", \"x\"))",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std parse decimal-digits \"1x\"",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "weighted-shortest-path is std graph weighted-shortest-path\nnegative-edges : List (String, String, Rational) is one (\"a\", \"b\", Rational (-1, 1))\ngraph-nodes : List String is Entry (\"a\", Entry (\"b\", Empty))\nweighted-shortest-path (\"a\", (\"b\", negative-edges, graph-nodes))",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"[\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"(\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"a|\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"|a\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"*a\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"a**\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"\\q\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"[z-a]\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"a{2,1}\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"a{\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
        (
            "std pattern regex contains? (\"text\", \"(?=a)\")",
            "E-RESULT-PROJECTION-INFALLIBLE",
        ),
    ];

    fn assert_standard_library_namespace(session: &mut Session, trace: &mut Vec<String>) {
        let value = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nuse library std ( version is v0.1 )\n(std min (4, 2), std combinatorics revision, std graph revision, std ordered revision, std parse revision, std sequence revision, std statistics revision, std test revision, std text revision, std pattern regex revision, std transfer revision, std data revision, std store revision, std network revision, std device revision)",
                trace,
            )
            .unwrap();
        let mut expected = vec![Value::Int(BigInt::from(2))];
        expected.extend((0..14).map(|_| Value::Int(BigInt::from(1))));
        assert_eq!(value, Value::Tuple(expected));
    }

    fn assert_companion_library_namespace(session: &mut Session, trace: &mut Vec<String>) {
        let undeclared = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nadvent-of-code revision",
                trace,
            )
            .unwrap_err();
        assert_eq!(undeclared.code, "E-UNDECLARED-LIBRARY");

        let companion = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nuse library advent-of-code ( version is v0.1 )\n(advent-of-code revision, advent-of-code geometry revision, advent-of-code graph revision, advent-of-code machine revision, advent-of-code packing revision)",
                trace,
            )
            .unwrap();
        assert_eq!(
            companion,
            Value::Tuple(vec![Value::Int(BigInt::from(1)); 5])
        );
    }

    fn assert_invalid_standard_library_cases(session: &Session, trace: &mut Vec<String>) {
        for (expression, expected_code) in INVALID_STANDARD_LIBRARY_CASES {
            let mut invalid_session = session.clone();
            let source = format!(
                "use language ( version is v0.1 )\nuse library std ( version is v0.1 )\n{expression}"
            );
            let error = invalid_session
                .evaluate_source_file(&source, trace)
                .unwrap_err();
            assert_eq!(error.code, *expected_code, "{expression}: {error:?}");
        }
    }

    #[test]
    fn shared_library_retains_flat_fundamentals_and_nested_extensions() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        assert_standard_library_namespace(&mut session, &mut trace);
        assert_companion_library_namespace(&mut session, &mut trace);
        assert_invalid_standard_library_cases(&session, &mut trace);
    }

    #[test]
    fn capability_generic_result_substitutes_inside_a_product() {
        let mut session = Session::new();
        let mut trace = Vec::new();
        let value = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nmin-max is fn (left : (Value : TotalOrder), right : Value) -> (Value, Value)\n  left\n    < right then (left, right)\n    otherwise (right, left)\nmin-max (4.5, 2.5)",
                &mut trace,
            )
            .unwrap();
        assert!(matches!(
            value,
            Value::Tuple(values)
                if values.len() == 2 && values.iter().all(|value| matches!(value, Value::Rational(_)))
        ));
    }

    #[test]
    fn shared_optional_functions_preserve_related_generic_types() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let value = session
            .evaluate_source_file(
"use language ( version is v0.1 )
use library std ( version is v0.1 )
present? is std present?
absent? is std absent?
map is std map
chain is std chain
filter is std filter
value-or is std value-or
or-else is std or-else
zip is std zip
flatten is std flatten
no-optionals : List Optional Int is Empty
(present? (Some 1), present? (None Int), absent? (None String), absent? (Some \"value\"), map ((Some 4), { value } value + 1), map ((None Int), { value } value + 1), chain ((Some 4), { value } Some (value + 2)), chain ((None Int), { value } Some (value + 2)), filter ((Some 4), { value } value > 2), filter ((Some 1), { value } value > 2), filter ((None Int), { value } value > 2), value-or ((Some 4), 9), value-or ((None Int), 9), or-else ((Some \"value\"), (Some \"fallback\")), or-else ((None String), (Some \"fallback\")), zip ((Some 2), (Some \"items\")), zip ((Some 2), (None String)), zip ((None Int), (Some \"items\")), flatten (Some (Some 7)), flatten (first no-optionals))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "(true, false, true, false, Some 5, None, Some 6, None, Some 4, None, None, 4, 9, Some \"value\", Some \"fallback\", Some (2, \"items\"), None, None, Some 7, None)"
        );
    }

    #[test]
    fn shared_result_functions_preserve_success_and_complete_errors() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let value = session
            .evaluate_source_file(
"use language ( version is v0.1 )
use library std ( version is v0.1 )
result-map is std map
result-chain is std chain
map-error is std map-error
recover is std recover
result-value-or is std value-or
result-or-else is std or-else
ok? is std ok?
error? is std error?
result-zip is std zip
result-flatten is std flatten
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
successful is 4.0 divide 2.0
failed is 4.0 divide 0.0
double-result is fn (value : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  value * (Rational 2)
identity-error is fn (problem : Error) -> Error
  problem
recover-one is fn (problem : Error) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  _ is problem
  Rational 1
fail-result is fn (value : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  value / (Rational 0)
nested-result is fn (value : Rational) -> Result (Result (Rational, lang arithmetic ArithmeticErrorCode), lang arithmetic ArithmeticErrorCode)
  value
nested-error is fn (denominator : Rational) -> Result (Result (Rational, lang arithmetic ArithmeticErrorCode), lang arithmetic ArithmeticErrorCode)
  1.0 / denominator
(ok? successful, error? successful, ok? failed, error? failed,
 result-map (successful, { value } value + 1), error? (result-map (failed, { value } value + 1)),
 ok? (result-chain (successful, double-result)), error? (result-chain (failed, double-result)), error? (result-chain (successful, fail-result)),
 ok? (map-error (successful, identity-error)), error? (map-error (failed, identity-error)),
 ok? (recover (successful, recover-one)), ok? (recover (failed, recover-one)),
 result-value-or (successful, Rational 9), result-value-or (failed, Rational 9),
 ok? (result-or-else (successful, 9.0 divide 3.0)), ok? (result-or-else (failed, 9.0 divide 3.0)),
 result-zip (successful, 9.0 divide 3.0), error? (result-zip (failed, 9.0 divide 3.0)),
 error? (result-zip (successful, 9.0 divide 0.0)),
 result-flatten (nested-result (Rational 5)), error? (result-flatten (nested-error 0.0)))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "(true, false, false, true, Rational ( 3, 1 ), true, true, true, true, true, true, true, true, Rational ( 2, 1 ), Rational ( 9, 1 ), true, true, (Rational ( 2, 1 ), Rational ( 3, 1 )), true, true, Rational ( 5, 1 ), true)"
        );
    }

    #[test]
    fn shared_exact_number_functions_cover_euclidean_and_partial_operations() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let value = session
            .evaluate_source_file(
                "use language ( version is v0.1 )
use library std ( version is v0.1 )
sign is std sign
distance is std distance
gcd is std gcd
even? is std even?
odd? is std odd?
divides? is std divides?
reciprocal is std reciprocal
sum is std sum
product is std product
error? is std error?
ints : List Int is Entry (-2, Entry (3, Empty))
rationals : List Rational is Entry (Rational (1, 2), Entry (Rational (3, 2), Empty))
(sign -9, sign 0, sign 9, sign (Rational (-1, 2)), sign (Rational 0), sign (Rational (1, 2)),
 distance (-4, 5), distance (Rational (-1, 2), Rational 1), gcd (-54, 24), gcd (0, 0),
 even? -4, even? 3, odd? -3, odd? 4, divides? (0, 0), divides? (0, 4),
 divides? (6, 42), divides? (5, 42), reciprocal 4.0, error? (reciprocal 0.0),
 sum ints, sum rationals, sum (Empty Int), product ints, product rationals,
 product (Empty Rational))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "(-1, 0, 1, -1, 0, 1, 9, Rational ( 3, 2 ), 6, 0, true, false, true, false, false, false, true, false, Rational ( 1, 4 ), true, 1, Rational ( 2, 1 ), 0, -6, Rational ( 3, 4 ), Rational ( 1, 1 ))"
        );
        assert!(trace.iter().any(|event| {
            event.contains("function.recursion.descended")
                && event.contains("TOPAL-FUNCTION-RECURSION-EUCLIDEAN-001")
        }));
    }

    #[test]
    fn shared_range_functions_preserve_endpoint_domains_and_convexity() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let value = session
            .evaluate_source_file(
"use language ( version is v0.1 )
use library std ( version is v0.1 )
bounds is std bounds
intersection is std intersection
overlaps? is std overlaps?
hull is std hull
adjacent? is std adjacent?
(bounds (-2 .. 5), intersection (0 .. 8, 4 .. 12), overlaps? (0 .. 2, 3 .. 5), hull (0.5 .. 2.5, 2.0 .. 4.0), adjacent? (0 .. 2, 3 .. 5))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "((-2, 5), 4 .. 8, false, Rational ( 1, 2 ) .. Rational ( 4, 1 ), false)"
        );
        assert_eq!(
            trace
                .iter()
                .filter(|event| event.contains("TOPAL-RANGE-BOUND-001"))
                .count(),
            22
        );
    }

    #[test]
    fn shared_text_functions_apply_explicit_unicode_and_search_policy() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let value = session
            .evaluate_source_file(
"use language ( version is v0.1 )
use library std ( version is v0.1 )
nfd is std nfd
canonical-equal is std canonical-equal
starts-with? is std starts-with?
ends-with? is std ends-with?
contains? is std contains?
trim is std trim
replace-all is std replace-all
repeat is std repeat
(nfd \"é\", canonical-equal (\"é\", \"e\u{301}\"), starts-with? (\"Topal\", \"Top\"), ends-with? (\"Topal\", \"pal\"), contains? (\"Topal\", \"opa\"), trim \"  text\n\", replace-all (\"a-b-a\", \"a\", \"x\"), repeat (\"ab\", 3))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "(\"e\u{301}\", true, true, true, true, \"text\", \"x-b-x\", \"ababab\")"
        );
    }

    #[test]
    fn shared_finite_algorithms_preserve_generic_list_elements() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let value = session
            .evaluate_source_file(
"use language ( version is v0.1 )
use library std ( version is v0.1 )
any? is std any?
all? is std all?
none? is std none?
count-where is std count-where
find is std find
filter-map is std filter-map
flat-map is std flat-map
values : List Int is Entry (1, Entry (2, Entry (3, Empty)))
none : List Int is Empty
even-double is fn (value : Int) -> Optional Int
  value % 2 = 0
    true then Some (value * 2)
    false then None Int
with-negative is fn (value : Int) -> List Int
  negative is 0 - value
  Entry (value, Entry (negative, Empty))
(any? (values, { value } value > 2), any? (values, { value } value > 9), any? (none, { value } value > 0),
 all? (values, { value } value > 0), all? (values, { value } value < 3), all? (none, { value } value > 0),
 none? (values, { value } value < 0), none? (values, { value } value = 2), none? (none, { value } value > 0),
 count-where (values, { value } value >= 2), count-where (values, { value } value > 9), count-where (none, { value } value > 0),
 find (values, { value } value > 1), find (values, { value } value > 9), find (none, { value } value > 0),
 filter-map (values, even-double), filter-map (none, even-double),
 flat-map (values, with-negative), flat-map (none, with-negative))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "(true, false, false, true, false, true, true, false, true, 2, 0, 0, Some 2, None, None, Entry ( 4, Empty ), Empty, Entry ( 1, Entry ( -1, Entry ( 2, Entry ( -2, Entry ( 3, Entry ( -3, Empty ) ) ) ) ) ), Empty)"
        );
    }

    #[test]
    fn shared_lazy_generators_load_as_linear_continuations() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let value = session
            .evaluate_source_file(
                "use language ( version is v0.1 )
use library std ( version is v0.1 )
enumerate is std count-from
numbers is enumerate 3
collect (numbers take-while ({ value } value < 7))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "Entry ( 3, Entry ( 4, Entry ( 5, Entry ( 6, Empty ) ) ) )"
        );
        assert!(trace.iter().any(|event| event.contains("generator")));
    }
}
