//! Native capabilities for the source-level Topal incremental-build policy.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use topal_language::{Session, Value, load_module_tree};

const STATE_SCHEMA: u32 = 1;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub units: Vec<Unit>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub id: String,
    pub kind: UnitKind,
    #[serde(default)]
    pub inputs: Vec<PathBuf>,
    #[serde(default)]
    pub outputs: Vec<PathBuf>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub command: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum UnitKind {
    Build,
    Test,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Stamp {
    seconds: u64,
    nanoseconds: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct State {
    schema: u32,
    manifest: Option<Stamp>,
    inputs: BTreeMap<String, Stamp>,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub source_root: PathBuf,
    pub build_root: PathBuf,
    pub library_root: PathBuf,
    pub manifest: PathBuf,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Outcome {
    pub selected: Vec<String>,
}

/// Loads the manifest, selects affected units through the Topal policy, and
/// executes their declared commands.
///
/// # Errors
///
/// Returns an error when roots or manifest entries are invalid, filesystem
/// observation fails, Topal policy evaluation fails, an action fails, or an
/// action does not produce every declared output.
pub fn run(options: &Options) -> Result<Outcome, String> {
    let manifest_path = resolve_source_path(&options.source_root, &options.manifest)?;
    let manifest_text = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_str(&manifest_text)
        .map_err(|error| format!("invalid {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;

    fs::create_dir_all(&options.build_root)
        .map_err(|error| format!("cannot create {}: {error}", options.build_root.display()))?;
    let state_path = options.build_root.join("state.json");
    let old_state = read_state(&state_path)?;
    let current_manifest = stamp(&manifest_path)?;
    let mut current_inputs = BTreeMap::new();
    let mut changed = BTreeSet::new();

    if old_state.manifest.as_ref() != Some(&current_manifest) {
        changed.extend(manifest.units.iter().map(|unit| unit.id.clone()));
    }
    for unit in &manifest.units {
        for input in &unit.inputs {
            let input_path = resolve_source_path(&options.source_root, input)?;
            let observation = stamp(&input_path)?;
            let identity = normalized_relative(input)?;
            if old_state.inputs.get(&identity) != Some(&observation) {
                changed.insert(unit.id.clone());
            }
            current_inputs.insert(identity, observation);
        }
        if unit.outputs.iter().any(|output| {
            resolve_build_path(&options.build_root, output).is_ok_and(|path| !path.exists())
        }) {
            changed.insert(unit.id.clone());
        }
    }

    let selected_set = select_with_topal(
        &options.library_root,
        &manifest,
        &changed.into_iter().collect::<Vec<_>>(),
    )?
    .into_iter()
    .collect::<BTreeSet<_>>();
    let selected = manifest
        .units
        .iter()
        .filter(|unit| selected_set.contains(&unit.id))
        .map(|unit| unit.id.clone())
        .collect::<Vec<_>>();

    if !options.dry_run {
        for unit in manifest
            .units
            .iter()
            .filter(|unit| selected_set.contains(&unit.id))
        {
            execute(unit, options)?;
        }
        write_state(
            &state_path,
            &State {
                schema: STATE_SCHEMA,
                manifest: Some(current_manifest),
                inputs: current_inputs,
            },
        )?;
    }
    Ok(Outcome { selected })
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != 1 {
        return Err(format!("unsupported manifest schema {}", manifest.schema));
    }
    let mut known = BTreeSet::new();
    let mut outputs = BTreeSet::new();
    for unit in &manifest.units {
        if unit.id.is_empty() || !known.insert(unit.id.clone()) {
            return Err(format!("duplicate or empty unit identity `{}`", unit.id));
        }
        if unit.command.is_empty() {
            return Err(format!("unit `{}` has no command", unit.id));
        }
        for path in &unit.inputs {
            normalized_relative(path)?;
        }
        for path in &unit.outputs {
            let identity = normalized_relative(path)?;
            if !outputs.insert(identity.clone()) {
                return Err(format!("output `{identity}` has more than one producer"));
            }
        }
        for dependency in &unit.dependencies {
            if dependency == &unit.id || !known.contains(dependency) {
                return Err(format!(
                    "unit `{}` depends on unknown or later unit `{dependency}`",
                    unit.id
                ));
            }
        }
    }
    Ok(())
}

fn select_with_topal(
    library_root: &Path,
    manifest: &Manifest,
    changed: &[String],
) -> Result<Vec<String>, String> {
    let mut session = Session::new();
    let mut trace = Vec::new();
    load_module_tree(&mut session, library_root, &mut trace)?;
    let edges = manifest.units.iter().flat_map(|unit| {
        unit.dependencies
            .iter()
            .map(move |dependency| (dependency.as_str(), unit.id.as_str()))
    });
    let source = format!(
        "use language ( version is v0.1 )\nuse library std ( version is v0.1 )\nselect-build-units is std build graph selected\nchanged-units : List String is {}\ndependency-edges : List (String, String) is {}\nall-units : List String is {}\nselect-build-units (changed-units, (dependency-edges, all-units))",
        string_list(changed.iter().map(String::as_str)),
        pair_list(edges),
        string_list(manifest.units.iter().map(|unit| unit.id.as_str()))
    );
    let value = session
        .evaluate_source_file(&source, &mut trace)
        .map_err(|error| {
            format!(
                "{}\nGenerated policy input:\n{source}",
                error.render("<topal-build-policy>")
            )
        })?;
    let Value::List { entries, .. } = value else {
        return Err("Topal build policy returned a non-List value".into());
    };
    entries
        .into_iter()
        .map(|entry| match entry {
            Value::String(identity) => Ok(identity),
            _ => Err("Topal build policy returned a non-String identity".into()),
        })
        .collect()
}

fn execute(unit: &Unit, options: &Options) -> Result<(), String> {
    let (program, arguments) = unit.command.split_first().expect("validated command");
    let status = Command::new(program)
        .args(arguments)
        .current_dir(&options.source_root)
        .env("TOPAL_SOURCE_ROOT", &options.source_root)
        .env("TOPAL_BUILD_ROOT", &options.build_root)
        .status()
        .map_err(|error| format!("cannot execute unit `{}`: {error}", unit.id))?;
    if !status.success() {
        return Err(format!("unit `{}` failed with {status}", unit.id));
    }
    for output in &unit.outputs {
        let path = resolve_build_path(&options.build_root, output)?;
        if !path.exists() {
            return Err(format!(
                "unit `{}` did not produce declared output {}",
                unit.id,
                path.display()
            ));
        }
    }
    Ok(())
}

fn stamp(path: &Path) -> Result<Stamp, String> {
    let modified = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|error| format!("cannot observe {}: {error}", path.display()))?;
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("invalid timestamp for {}: {error}", path.display()))?;
    Ok(Stamp {
        seconds: duration.as_secs(),
        nanoseconds: duration.subsec_nanos(),
    })
}

fn read_state(path: &Path) -> Result<State, String> {
    if !path.exists() {
        return Ok(State {
            schema: STATE_SCHEMA,
            ..State::default()
        });
    }
    let state: State = serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?,
    )
    .map_err(|error| format!("invalid {}: {error}", path.display()))?;
    if state.schema != STATE_SCHEMA {
        return Err(format!("unsupported state schema {}", state.schema));
    }
    Ok(state)
}

fn write_state(path: &Path, state: &State) -> Result<(), String> {
    let temporary = path.with_extension("json.tmp");
    let contents = serde_json::to_vec_pretty(state).map_err(|error| error.to_string())?;
    fs::write(&temporary, contents)
        .map_err(|error| format!("cannot write {}: {error}", temporary.display()))?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("cannot publish {}: {error}", path.display()))
}

fn resolve_source_path(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    Ok(root.join(normalized_relative(relative)?))
}

fn resolve_build_path(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    Ok(root.join(normalized_relative(relative)?))
}

fn normalized_relative(path: &Path) -> Result<String, String> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "path must remain relative to its declared root: {}",
            path.display()
        ));
    }
    Ok(path.to_string_lossy().replace('\\', "/"))
}

fn string_literal(value: &str) -> String {
    serde_json::to_string(value).expect("String is JSON representable")
}

fn string_list<'a>(values: impl DoubleEndedIterator<Item = &'a str>) -> String {
    values.rev().fold("Empty".to_owned(), |tail, value| {
        format!("Entry ({}, {tail})", string_literal(value))
    })
}

fn pair_list<'a>(values: impl DoubleEndedIterator<Item = (&'a str, &'a str)>) -> String {
    values
        .rev()
        .fold("Empty".to_owned(), |tail, (left, right)| {
            format!(
                "Entry (({}, {}), {tail})",
                string_literal(left),
                string_literal(right)
            )
        })
}
