#![cfg(all(unix, target_arch = "x86_64"))]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use topal_c_abi::AccessLibraryModel;

static NEXT_TEST: AtomicU64 = AtomicU64::new(0);

fn temporary(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/topal-c-bindgen-tests");
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

fn llvm_tools() -> PathBuf {
    let output = Command::new("rustc")
        .args(["--print", "target-libdir"])
        .output()
        .unwrap();
    assert!(output.status.success());
    PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
        .parent()
        .unwrap()
        .join("bin")
}

#[test]
#[allow(clippy::too_many_lines)] // One fixture proves atomic static and shared generation from the same interface.
fn generates_a_canonical_access_library_atomically() {
    let directory = temporary("generate");
    let tools = llvm_tools();
    let header = directory.join("arithmetic.h");
    let ir = directory.join("arithmetic.ll");
    let bitcode = directory.join("arithmetic.bc");
    let object = directory.join("arithmetic.o");
    let archive = directory.join("libarithmetic.a");
    let shared_object = directory.join("libarithmetic.so");
    let clang = directory.join("clang-22");
    let output = directory.join("arithmetic");
    fs::write(
        &header,
        "int c_add(int left, int right) __attribute__((const));\n",
    )
    .unwrap();
    fs::write(
        &ir,
        "target triple = \"x86_64-unknown-linux-gnu\"\ndefine i32 @c_add(i32 %left, i32 %right) {\nentry:\n  %sum = add i32 %left, %right\n  ret i32 %sum\n}\n",
    )
    .unwrap();
    for status in [
        Command::new(tools.join("llvm-as"))
            .args([ir.to_str().unwrap(), "-o", bitcode.to_str().unwrap()])
            .status()
            .unwrap(),
        Command::new(tools.join("llc"))
            .args([
                "-filetype=obj",
                "-mtriple=x86_64-unknown-linux-gnu",
                bitcode.to_str().unwrap(),
                "-o",
                object.to_str().unwrap(),
            ])
            .status()
            .unwrap(),
        Command::new(tools.join("llvm-ar"))
            .args(["rcs", archive.to_str().unwrap(), object.to_str().unwrap()])
            .status()
            .unwrap(),
        Command::new(tools.join("rust-lld"))
            .args([
                "-flavor",
                "gnu",
                "-shared",
                "-soname",
                "libarithmetic.so",
                object.to_str().unwrap(),
                "-o",
                shared_object.to_str().unwrap(),
            ])
            .status()
            .unwrap(),
    ] {
        assert!(status.success());
    }
    fs::write(
        &clang,
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'clang version 22.0.0'\nelse\n  printf '%s' '{\"kind\":\"TranslationUnitDecl\",\"inner\":[{\"kind\":\"FunctionDecl\",\"name\":\"c_add\",\"type\":{\"qualType\":\"int (int, int)\"},\"inner\":[{\"kind\":\"ParmVarDecl\",\"name\":\"left\",\"type\":{\"qualType\":\"int\"}},{\"kind\":\"ParmVarDecl\",\"name\":\"right\",\"type\":{\"qualType\":\"int\"}},{\"kind\":\"ConstAttr\"}]}]}'\nfi\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&clang).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&clang, permissions).unwrap();

    let generated = Command::new(env!("CARGO_BIN_EXE_topal-c-bindgen"))
        .args([
            "--library",
            "arithmetic",
            "--header",
            header.to_str().unwrap(),
            "--archive",
            archive.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--clang",
            clang.to_str().unwrap(),
            "--llvm-tools",
            tools.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let source = fs::read_to_string(output.join("module.t")).unwrap();
    let model = AccessLibraryModel::decode_topal(&source).unwrap();
    let AccessLibraryModel::Static(model) = model else {
        panic!("archive input must generate the static schema")
    };
    assert_eq!(model.functions[0].symbol, "c_add");
    assert_eq!(model.header.file, "interface.h");
    assert_eq!(model.static_archive.file, "libarithmetic.a");
    assert!(output.join("libarithmetic.a").is_file());
    assert!(!output.join("module.topal-c-abi.json").exists());

    let shared_output = directory.join("arithmetic-shared");
    let generated = Command::new(env!("CARGO_BIN_EXE_topal-c-bindgen"))
        .args([
            "--library",
            "arithmetic",
            "--header",
            header.to_str().unwrap(),
            "--shared-object",
            shared_object.to_str().unwrap(),
            "--output",
            shared_output.to_str().unwrap(),
            "--clang",
            clang.to_str().unwrap(),
            "--llvm-tools",
            tools.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let source = fs::read_to_string(shared_output.join("module.t")).unwrap();
    let model = AccessLibraryModel::decode_topal(&source).unwrap();
    let AccessLibraryModel::Shared(model) = model else {
        panic!("shared input must generate the shared schema")
    };
    assert_eq!(model.shared_object.soname, "libarithmetic.so");
    assert_eq!(model.header.file, "interface.h");
    assert_eq!(model.shared_object.file, "libarithmetic.so");
    assert!(shared_output.join("libarithmetic.so").is_file());
    assert!(!shared_output.join("module.topal-c-abi.json").exists());
}
