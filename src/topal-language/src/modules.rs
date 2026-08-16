//! Shared filesystem module loading for interpreters and compiler frontends.

use std::fs;
use std::path::Path;

use crate::{Session, TraceSink};

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
        let value = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nmin is fundamental ordering min\nmin (4, 2)",
                &mut trace,
            )
            .unwrap();
        assert_eq!(value, Value::Int(BigInt::from(2)));
        assert!(trace.iter().any(|event| event.contains("module.loaded")));
    }

    #[test]
    fn shared_library_ordering_retains_one_exact_generic_type() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let mut session = Session::new();
        let mut trace = Vec::new();
        load_module_tree(&mut session, &root, &mut trace).unwrap();
        let value = session
            .evaluate_source_file(
                "use language ( version is v0.1 )\nmin is fundamental ordering min\nmin ((1, 3), (1, 2))",
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
present? is fundamental optional present?
absent? is fundamental optional absent?
map is fundamental optional map
chain is fundamental optional chain
filter is fundamental optional filter
value-or is fundamental optional value-or
or-else is fundamental optional or-else
zip is fundamental optional zip
flatten is fundamental optional flatten
(present? (Some 1), absent? (None String), map ((Some 4), { value } value + 1), chain ((Some 4), { value } Some (value + 2)), filter ((Some 4), { value } value > 2), filter ((Some 1), { value } value > 2), value-or ((None Int), 9), or-else ((None String), (Some \"fallback\")), zip ((Some 2), (Some \"items\")), flatten (Some (Some 7)))",
                &mut trace,
            )
            .unwrap();
        assert_eq!(
            value.to_string(),
            "(true, true, Some 5, Some 6, Some 4, None, 9, Some \"fallback\", Some (2, \"items\"), Some 7)"
        );
    }
}
