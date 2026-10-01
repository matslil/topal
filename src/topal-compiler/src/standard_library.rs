use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::artifact::sha256;
use crate::{CompileError, DATA_LAYOUT, LlvmTools, TARGET_TRIPLE};

pub const STANDARD_LIBRARY_ABI: &str = "topal-library/1";
pub const STANDARD_LIBRARY_SCHEMA: &str = "topal.standard-library-slice/1";
pub const STANDARD_LIBRARY_SONAME: &str = "libtopal-std.so.1";
pub const STANDARD_LIBRARY_ENTRY: &str = "topal_library_v1_entry";
static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StandardLibrarySlice {
    pub schema: String,
    pub language_revision: String,
    pub library_identity: String,
    pub library_major: u32,
    pub abi_revision: String,
    pub target_triple: String,
    pub object_format: String,
    pub data_layout: String,
    pub soname: String,
    pub source_interface_sha256: String,
    pub export_map: Vec<String>,
    pub shared_object_sha256: String,
}

impl StandardLibrarySlice {
    fn validate(&self) -> Result<(), String> {
        for (actual, expected, field) in [
            (self.schema.as_str(), STANDARD_LIBRARY_SCHEMA, "schema"),
            (self.language_revision.as_str(), "v0.1", "language revision"),
            (self.library_identity.as_str(), "std", "library identity"),
            (
                self.abi_revision.as_str(),
                STANDARD_LIBRARY_ABI,
                "ABI revision",
            ),
            (self.target_triple.as_str(), TARGET_TRIPLE, "target triple"),
            (self.object_format.as_str(), "elf64-x86-64", "object format"),
            (self.data_layout.as_str(), DATA_LAYOUT, "data layout"),
            (self.soname.as_str(), STANDARD_LIBRARY_SONAME, "SONAME"),
        ] {
            if actual != expected {
                return Err(format!(
                    "standard-library slice has unsupported {field}: expected `{expected}`, found `{actual}`"
                ));
            }
        }
        if self.library_major != 1
            || self.export_map != [STANDARD_LIBRARY_ENTRY]
            || !valid_digest(&self.source_interface_sha256)
            || !valid_digest(&self.shared_object_sha256)
        {
            return Err(
                "standard-library slice has an invalid major, export map, or digest".into(),
            );
        }
        Ok(())
    }

    fn encode(&self) -> Result<Vec<u8>, CompileError> {
        self.validate().map_err(CompileError::Tool)?;
        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| CompileError::Tool(error.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, CompileError> {
        let manifest: Self =
            serde_json::from_slice(bytes).map_err(|error| CompileError::Tool(error.to_string()))?;
        manifest.validate().map_err(CompileError::Tool)?;
        if manifest.encode()? != bytes {
            return Err(CompileError::Tool(
                "standard-library slice manifest is not canonical".into(),
            ));
        }
        Ok(manifest)
    }
}

#[must_use]
pub fn standard_library_manifest_path(shared_object: &Path) -> PathBuf {
    let mut name = shared_object.as_os_str().to_owned();
    name.push(".topal-library.json");
    PathBuf::from(name)
}

/// Build and atomically describe the qualified standard-library ELF slice.
///
/// # Errors
///
/// Returns an I/O, manifest, target, or LLVM toolchain error before a complete
/// object-and-manifest pair is published.
pub fn build_standard_library(
    library_root: &Path,
    output: &Path,
    llvm_tools: Option<&Path>,
) -> Result<StandardLibrarySlice, CompileError> {
    if output.file_name().and_then(|name| name.to_str()) != Some(STANDARD_LIBRARY_SONAME) {
        return Err(CompileError::Tool(format!(
            "the qualified ELF standard library must be named `{STANDARD_LIBRARY_SONAME}`"
        )));
    }
    let source_interface_sha256 = source_interface_digest(library_root)?;
    let tools = LlvmTools::discover(llvm_tools)?;
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        CompileError::Io(format!(
            "cannot create output directory {}: {error}",
            parent.display()
        ))
    })?;
    let temporary = parent.join(format!(
        ".topalc-std-{}-{}",
        std::process::id(),
        NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&temporary).map_err(|error| {
        CompileError::Io(format!(
            "cannot create temporary directory {}: {error}",
            temporary.display()
        ))
    })?;
    let result = build_standard_library_in(&tools, &temporary, output, &source_interface_sha256);
    let _ = fs::remove_dir_all(&temporary);
    result
}

fn build_standard_library_in(
    tools: &LlvmTools,
    temporary: &Path,
    output: &Path,
    source_interface_sha256: &str,
) -> Result<StandardLibrarySlice, CompileError> {
    let ir = temporary.join("std.ll");
    let bitcode = temporary.join("std.bc");
    let verified = temporary.join("std.verified.bc");
    let object = temporary.join("std.o");
    let shared = temporary.join(STANDARD_LIBRARY_SONAME);
    let llvm = format!(
        "target datalayout = \"{DATA_LAYOUT}\"\ntarget triple = \"{TARGET_TRIPLE}\"\n@topal_library_v1_source_interface = hidden constant [64 x i8] c\"{source_interface_sha256}\"\n@{STANDARD_LIBRARY_ENTRY} = protected constant {{ i32, i32, ptr }} {{ i32 1, i32 1, ptr @topal_library_v1_source_interface }}\n"
    );
    fs::write(&ir, llvm)
        .map_err(|error| CompileError::Io(format!("cannot write {}: {error}", ir.display())))?;
    run(Command::new(tools.tool_path("llvm-as"))
        .arg(&ir)
        .arg("-o")
        .arg(&bitcode))?;
    run(Command::new(tools.tool_path("opt"))
        .arg("-passes=verify")
        .arg(&bitcode)
        .arg("-o")
        .arg(&verified))?;
    run(Command::new(tools.tool_path("llc"))
        .args([
            "-O0",
            "-filetype=obj",
            "-mtriple=x86_64-unknown-linux-gnu",
            "-mcpu=x86-64",
            "-relocation-model=pic",
        ])
        .arg(&verified)
        .arg("-o")
        .arg(&object))?;
    run(Command::new(tools.tool_path("rust-lld"))
        .args([
            "-flavor",
            "gnu",
            "-shared",
            "--no-undefined",
            "--soname",
            STANDARD_LIBRARY_SONAME,
        ])
        .arg(&object)
        .arg("-o")
        .arg(&shared))?;
    let payload = fs::read(&shared)
        .map_err(|error| CompileError::Io(format!("cannot read {}: {error}", shared.display())))?;
    let manifest = StandardLibrarySlice {
        schema: STANDARD_LIBRARY_SCHEMA.into(),
        language_revision: "v0.1".into(),
        library_identity: "std".into(),
        library_major: 1,
        abi_revision: STANDARD_LIBRARY_ABI.into(),
        target_triple: TARGET_TRIPLE.into(),
        object_format: "elf64-x86-64".into(),
        data_layout: DATA_LAYOUT.into(),
        soname: STANDARD_LIBRARY_SONAME.into(),
        source_interface_sha256: source_interface_sha256.into(),
        export_map: vec![STANDARD_LIBRARY_ENTRY.into()],
        shared_object_sha256: sha256(&payload),
    };
    let manifest_bytes = manifest.encode()?;
    fs::write(output, payload).map_err(|error| {
        CompileError::Io(format!("cannot publish {}: {error}", output.display()))
    })?;
    let manifest_path = standard_library_manifest_path(output);
    fs::write(&manifest_path, manifest_bytes).map_err(|error| {
        let _ = fs::remove_file(output);
        CompileError::Io(format!(
            "cannot publish {}: {error}",
            manifest_path.display()
        ))
    })?;
    Ok(manifest)
}

pub(crate) fn source_interface_digest(library_root: &Path) -> Result<String, CompileError> {
    let root = library_root.join("std");
    let mut paths = Vec::new();
    collect_topal_sources(&root, &mut paths)?;
    paths.sort();
    if paths.is_empty() {
        return Err(CompileError::Io(format!(
            "standard-library source tree {} is empty",
            root.display()
        )));
    }
    let mut interface = Vec::new();
    for path in paths {
        let relative = path
            .strip_prefix(&root)
            .expect("collected path is below root");
        interface.extend_from_slice(relative.to_string_lossy().as_bytes());
        interface.push(0);
        interface.extend(fs::read(&path).map_err(|error| {
            CompileError::Io(format!("cannot read {}: {error}", path.display()))
        })?);
        interface.push(0);
    }
    Ok(sha256(&interface))
}

fn collect_topal_sources(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), CompileError> {
    let entries = fs::read_dir(directory).map_err(|error| {
        CompileError::Io(format!(
            "cannot read standard-library directory {}: {error}",
            directory.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| CompileError::Io(error.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_topal_sources(&path, paths)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("t") {
            paths.push(path);
        }
    }
    Ok(())
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn run(command: &mut Command) -> Result<(), CompileError> {
    let display = format!("{command:?}");
    let output = command
        .output()
        .map_err(|error| CompileError::Tool(format!("cannot execute {display}: {error}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(CompileError::Tool(format!(
            "LLVM command failed ({display}):\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}
