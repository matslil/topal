use std::fs::{self, File, FileTimes};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, UNIX_EPOCH};

use serde_json::json;
use topal_build::{Options, run};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    root: PathBuf,
    source: PathBuf,
    build: PathBuf,
    library: PathBuf,
}

impl Fixture {
    fn new(out_of_tree: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "topal-build-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        let source = root.join("source");
        let build = if out_of_tree {
            root.join("output/build")
        } else {
            source.join(".topal-build")
        };
        fs::create_dir_all(&source).unwrap();
        let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library");
        let helper = env!("CARGO_BIN_EXE_topal-build-test-action");
        let units = [
            ("core", "core.t", "core.out", Vec::<&str>::new(), "build"),
            ("app", "app.t", "app.out", vec!["core"], "build"),
            ("test-app", "test.t", "test.out", vec!["app"], "test"),
            ("other", "other.t", "other.out", Vec::<&str>::new(), "build"),
        ]
        .into_iter()
        .map(|(id, input, output, dependencies, kind)| {
            json!({
                "id": id,
                "kind": kind,
                "inputs": [input],
                "outputs": [output],
                "dependencies": dependencies,
                "command": [helper, id, output, input]
            })
        })
        .collect::<Vec<_>>();
        fs::write(
            source.join("topal-build.json"),
            serde_json::to_vec_pretty(&json!({"schema": 1, "units": units})).unwrap(),
        )
        .unwrap();
        for (index, name) in ["core.t", "app.t", "test.t", "other.t"]
            .into_iter()
            .enumerate()
        {
            write_at(&source.join(name), "initial", 10 + index as u64);
        }
        Self {
            root,
            source,
            build,
            library,
        }
    }

    fn options(&self) -> Options {
        Options {
            source_root: self.source.clone(),
            build_root: self.build.clone(),
            library_root: self.library.clone(),
            manifest: "topal-build.json".into(),
            dry_run: false,
        }
    }

    fn executed(&self) -> Vec<String> {
        fs::read_to_string(self.build.join("executed.log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    fn clear_log(&self) {
        fs::write(self.build.join("executed.log"), "").unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write_at(path: &Path, contents: &str, seconds: u64) {
    fs::write(path, contents).unwrap();
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(seconds)))
        .unwrap();
}

#[test]
fn in_tree_build_selects_only_reverse_transitive_dependents() {
    let fixture = Fixture::new(false);
    assert_eq!(
        run(&fixture.options()).unwrap().selected,
        ["core", "app", "test-app", "other"]
    );
    fixture.clear_log();
    assert!(run(&fixture.options()).unwrap().selected.is_empty());
    write_at(&fixture.source.join("core.t"), "reverted-content", 30);
    assert_eq!(
        run(&fixture.options()).unwrap().selected,
        ["core", "app", "test-app"]
    );
    assert_eq!(fixture.executed(), ["core", "app", "test-app"]);
}

#[test]
fn test_only_change_does_not_rebuild_production_units() {
    let fixture = Fixture::new(false);
    run(&fixture.options()).unwrap();
    fixture.clear_log();
    write_at(&fixture.source.join("test.t"), "changed test", 31);
    assert_eq!(run(&fixture.options()).unwrap().selected, ["test-app"]);
    assert_eq!(fixture.executed(), ["test-app"]);
}

#[test]
fn out_of_tree_build_keeps_outputs_and_state_outside_source() {
    let fixture = Fixture::new(true);
    assert_eq!(run(&fixture.options()).unwrap().selected.len(), 4);
    assert!(fixture.build.join("state.json").is_file());
    assert!(fixture.build.join("test.out").is_file());
    assert!(!fixture.source.join(".topal-build").exists());
    assert!(!fixture.source.join("core.out").exists());
}

#[test]
fn failed_action_does_not_advance_observation_state() {
    let fixture = Fixture::new(true);
    run(&fixture.options()).unwrap();
    let state = fs::read(fixture.build.join("state.json")).unwrap();
    write_at(&fixture.source.join("core.t"), "fail", 40);
    assert!(run(&fixture.options()).is_err());
    assert_eq!(fs::read(fixture.build.join("state.json")).unwrap(), state);
    write_at(&fixture.source.join("core.t"), "recovered", 41);
    assert_eq!(
        run(&fixture.options()).unwrap().selected,
        ["core", "app", "test-app"]
    );
}

#[test]
fn invalid_graph_and_escaping_paths_are_rejected() {
    let fixture = Fixture::new(true);
    let invalid = json!({
        "schema": 1,
        "units": [{
            "id": "bad", "kind": "build", "inputs": ["../escape"],
            "outputs": ["bad.out"], "dependencies": ["missing"],
            "command": ["not-run"]
        }]
    });
    fs::write(
        fixture.source.join("invalid.json"),
        serde_json::to_vec_pretty(&invalid).unwrap(),
    )
    .unwrap();
    let mut options = fixture.options();
    options.manifest = "invalid.json".into();
    assert!(run(&options).unwrap_err().contains("declared root"));
}
