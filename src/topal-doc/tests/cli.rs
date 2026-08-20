use std::fs;
use std::process::Command;

fn scratch(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("topal-doc-{}-{name}", std::process::id()))
}

#[test]
fn generates_explicit_source_and_optional_lang_pages() {
    let output = scratch("explicit");
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../../library/std.t");
    let result = Command::new(env!("CARGO_BIN_EXE_topal-doc"))
        .args([
            "--output",
            output.to_str().unwrap(),
            "--include-lang",
            source,
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let standard = fs::read_to_string(output.join("std.rst")).unwrap();
    let lang = fs::read_to_string(output.join("lang.rst")).unwrap();
    assert!(standard.contains("Return the smaller value"));
    assert!(standard.contains(".. code-block:: topal"));
    assert!(lang.contains("lang Int"));
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn directory_traversal_is_shallow_unless_recursive() {
    let input = scratch("input");
    let nested = input.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let header = "use language (\n  version is v0.1\n)\n### A value.\npub value is 1\n";
    fs::write(input.join("direct.t"), header).unwrap();
    fs::write(nested.join("nested.t"), header).unwrap();
    let shallow = scratch("shallow");
    let recursive = scratch("recursive");
    assert!(
        Command::new(env!("CARGO_BIN_EXE_topal-doc"))
            .args([
                "--output",
                shallow.to_str().unwrap(),
                input.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    assert!(shallow.join("direct.rst").is_file());
    assert!(!shallow.join("nested.rst").exists());
    assert!(
        Command::new(env!("CARGO_BIN_EXE_topal-doc"))
            .args([
                "--output",
                recursive.to_str().unwrap(),
                "--recurse",
                input.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    assert!(recursive.join("nested.rst").is_file());
    fs::remove_dir_all(input).unwrap();
    fs::remove_dir_all(shallow).unwrap();
    fs::remove_dir_all(recursive).unwrap();
}
