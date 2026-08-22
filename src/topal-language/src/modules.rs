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
            session
                .load_module(&name, &source, trace)
                .map_err(|error| error.render(&path.display().to_string()))?;
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
        child
            .evaluate_source_file(&source, trace)
            .map_err(|error| error.render(&descriptor.display().to_string()))?;
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
    }

    #[test]
    fn shared_library_retains_flat_fundamentals_and_nested_extensions() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();

        let value = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nuse library std ( version is v0.1 )\n(std min (4, 2), std transfer revision, std data revision, std store revision, std network revision, std device revision)",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value,
            Value::Tuple(vec![
                Value::Int(BigInt::from(2)),
                Value::Int(BigInt::from(1)),
                Value::Int(BigInt::from(1)),
                Value::Int(BigInt::from(1)),
                Value::Int(BigInt::from(1)),
                Value::Int(BigInt::from(1)),
            ])
        );
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
(present? (Some 1), absent? (None String), map ((Some 4), { value } value + 1), chain ((Some 4), { value } Some (value + 2)), filter ((Some 4), { value } value > 2), filter ((Some 1), { value } value > 2), value-or ((None Int), 9), or-else ((None String), (Some \"fallback\")), zip ((Some 2), (Some \"items\")), flatten (Some (Some 7)))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "(true, true, Some 5, Some 6, Some 4, None, 9, Some \"fallback\", Some (2, \"items\"), Some 7)"
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
ok? is std ok?
error? is std error?
result-zip is std zip
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
successful is 4.0 divide 2.0
failed is 4.0 divide 0.0
(ok? successful, error? failed, result-map (successful, { value } value + 1), result-map (failed, { value } value + 1), result-zip (successful, 9.0 divide 3.0))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "(true, true, Rational ( 3, 1 ), Error ( domain is root./(Rational,Rational), code is division-by-zero ), (Rational ( 2, 1 ), Rational ( 3, 1 )))"
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
gcd is std gcd
even? is std even?
odd? is std odd?
divides? is std divides?
reciprocal is std reciprocal
(gcd (-54, 24), even? -4, odd? -3, divides? (0, 0), divides? (6, 42), reciprocal 4.0)",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "(6, true, true, true, true, Rational ( 1, 4 ))"
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
            "((-2, 5), 4 .. 8, false, Rational ( 1, 2 ) .. Rational ( 4, 1 ), true)"
        );
        assert_eq!(
            trace
                .iter()
                .filter(|event| event.contains("TOPAL-RANGE-BOUND-001"))
                .count(),
            10
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
values : List Int is Entry (1, Entry (2, Entry (3, Empty)))
(any? (values, { value } value > 2), all? (values, { value } value > 0), none? (values, { value } value < 0), count-where (values, { value } value >= 2), find (values, { value } value > 1))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(value.to_string(), "(true, true, true, 2, Some 2)");
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
