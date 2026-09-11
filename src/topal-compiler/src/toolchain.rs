use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use topal_language::CompilerProgram;

use crate::{
    CompileError, CompileOptions, Emit, LLVM_MAJOR, NativeArtifactMetadata, metadata_path,
};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct LlvmTools {
    pub directory: PathBuf,
    pub version: String,
}

impl LlvmTools {
    /// Locate the pinned LLVM command-line tools.
    ///
    /// # Errors
    ///
    /// Returns an actionable error when LLVM 22 cannot be located or verified.
    pub fn discover(explicit: Option<&Path>) -> Result<Self, CompileError> {
        let directory = if let Some(directory) = explicit {
            directory.to_owned()
        } else if let Some(directory) = env::var_os("TOPAL_LLVM_TOOLS") {
            PathBuf::from(directory)
        } else if let Some(prefix) = env::var_os("LLVM_SYS_220_PREFIX") {
            PathBuf::from(prefix).join("bin")
        } else {
            rust_toolchain_bin()?
        };
        let version = verify(&directory)?;
        Ok(Self { directory, version })
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.join(name)
    }
}

fn verify(directory: &Path) -> Result<String, CompileError> {
    for name in ["llvm-as", "opt", "llc", "rust-lld"] {
        let path = directory.join(name);
        if !path.is_file() {
            return Err(CompileError::Tool(format!(
                "LLVM tool `{}` is missing; install rustup component llvm-tools for LLVM {LLVM_MAJOR}, or pass --llvm-tools DIR",
                path.display()
            )));
        }
    }
    let mut exact_version = None;
    for (name, arguments, marker) in [
        ("llvm-as", &["--version"][..], "LLVM version"),
        ("opt", &["--version"][..], "LLVM version"),
        ("llc", &["--version"][..], "LLVM version"),
        ("rust-lld", &["-flavor", "gnu", "--version"][..], "LLD"),
    ] {
        let path = directory.join(name);
        let output = run_output(Command::new(&path).args(arguments))?;
        let version = String::from_utf8_lossy(&output.stdout);
        let prefix = format!("{marker} ");
        let reported = version
            .lines()
            .map(str::trim)
            .find_map(|line| line.strip_prefix(&prefix))
            .and_then(|value| value.split_whitespace().next())
            .ok_or_else(|| {
                CompileError::Tool(format!(
                    "`{}` did not report a recognizable LLVM version",
                    path.display()
                ))
            })?;
        let normalized = reported.split_once('-').map_or(reported, |(base, _)| base);
        if !normalized.starts_with(&format!("{LLVM_MAJOR}.")) {
            return Err(CompileError::Tool(format!(
                "topalc requires LLVM {LLVM_MAJOR}; `{}` reported {reported}",
                path.display()
            )));
        }
        if let Some(expected) = &exact_version {
            if expected != normalized {
                return Err(CompileError::Tool(format!(
                    "LLVM tool suite mixes versions {expected} and {normalized} at `{}`",
                    path.display()
                )));
            }
        } else {
            exact_version = Some(normalized.to_owned());
        }
    }
    exact_version.ok_or_else(|| CompileError::Tool("llvm-as version is unavailable".into()))
}

pub(crate) fn materialize(
    program: &CompilerProgram,
    llvm: &[u8],
    options: &CompileOptions,
) -> Result<NativeArtifactMetadata, CompileError> {
    let tools = LlvmTools::discover(options.llvm_tools.as_deref())?;
    let output_parent = options.output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(output_parent).map_err(|error| {
        CompileError::Io(format!(
            "cannot create output directory {}: {error}",
            output_parent.display()
        ))
    })?;
    let temporary = temporary_directory(output_parent)?;
    let result = materialize_in(program, llvm, options, &tools, &temporary);
    let _ = fs::remove_dir_all(&temporary);
    result
}

fn materialize_in(
    program: &CompilerProgram,
    llvm: &[u8],
    options: &CompileOptions,
    tools: &LlvmTools,
    temporary: &Path,
) -> Result<NativeArtifactMetadata, CompileError> {
    let input = temporary.join("module.ll");
    fs::write(&input, llvm).map_err(io_error("write LLVM IR", &input))?;
    let generated = match options.emit {
        Emit::LlvmIr => input,
        Emit::Object | Emit::Executable => {
            let bitcode = temporary.join("module.bc");
            run(Command::new(tools.path("llvm-as"))
                .arg(&input)
                .arg("-o")
                .arg(&bitcode))?;
            let verified = temporary.join("verified.bc");
            run(Command::new(tools.path("opt"))
                .arg("-passes=verify")
                .arg(&bitcode)
                .arg("-o")
                .arg(&verified))?;
            let object = temporary.join("module.o");
            run(Command::new(tools.path("llc"))
                .arg("-O0")
                .arg("-filetype=obj")
                .arg("-mtriple=x86_64-unknown-linux-gnu")
                .arg("-mcpu=x86-64")
                .arg("-relocation-model=pic")
                .arg("--frame-pointer=all")
                .arg(&verified)
                .arg("-o")
                .arg(&object))?;
            if options.emit == Emit::Object {
                object
            } else {
                let executable = temporary.join("application");
                run(Command::new(tools.path("rust-lld"))
                    .arg("-flavor")
                    .arg("gnu")
                    .arg("-static")
                    .arg("-pie")
                    .arg("--no-dynamic-linker")
                    .arg("-e")
                    .arg("_start")
                    .arg(&object)
                    .arg("-o")
                    .arg(&executable))?;
                executable
            }
        }
    };
    let output_bytes =
        fs::read(&generated).map_err(io_error("read generated output", &generated))?;
    let metadata = NativeArtifactMetadata::for_program(
        program,
        &options.source_name,
        options.emit.name(),
        &output_bytes,
        &tools.version,
    );
    let encoded = metadata.encode().map_err(CompileError::Tool)?;
    let staged_output = temporary.join("published-output");
    let staged_metadata = temporary.join("published-metadata");
    fs::write(&staged_output, &output_bytes).map_err(io_error("stage output", &staged_output))?;
    let permissions = fs::metadata(&generated)
        .map_err(io_error("read output permissions", &generated))?
        .permissions();
    fs::set_permissions(&staged_output, permissions)
        .map_err(io_error("preserve output permissions", &staged_output))?;
    fs::write(&staged_metadata, encoded).map_err(io_error("stage metadata", &staged_metadata))?;

    let sidecar = metadata_path(&options.output);
    fs::rename(&staged_metadata, &sidecar).map_err(io_error("publish metadata", &sidecar))?;
    if let Err(error) = fs::rename(&staged_output, &options.output) {
        let _ = fs::remove_file(&sidecar);
        return Err(CompileError::Io(format!(
            "cannot publish output {}: {error}",
            options.output.display()
        )));
    }
    Ok(metadata)
}

fn temporary_directory(parent: &Path) -> Result<PathBuf, CompileError> {
    for _ in 0..100 {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".topalc-{}-{sequence}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(CompileError::Io(format!(
                    "cannot create temporary directory {}: {error}",
                    path.display()
                )));
            }
        }
    }
    Err(CompileError::Io(
        "cannot allocate a unique compiler temporary directory".into(),
    ))
}

fn rust_toolchain_bin() -> Result<PathBuf, CompileError> {
    let output = run_output(Command::new("rustc").arg("--print").arg("target-libdir"))?;
    let target_libdir = String::from_utf8(output.stdout).map_err(|error| {
        CompileError::Tool(format!(
            "rustc returned a non-UTF-8 target library directory: {error}"
        ))
    })?;
    PathBuf::from(target_libdir.trim())
        .parent()
        .map(|target| target.join("bin"))
        .ok_or_else(|| CompileError::Tool("rustc returned an invalid target library path".into()))
}

fn run(command: &mut Command) -> Result<(), CompileError> {
    let display = format!("{command:?}");
    let output = run_output(command)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(CompileError::Tool(format!(
            "LLVM command failed ({display}):\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn run_output(command: &mut Command) -> Result<Output, CompileError> {
    command
        .output()
        .map_err(|error| CompileError::Tool(format!("cannot execute {command:?}: {error}")))
}

fn io_error<'a>(
    action: &'static str,
    path: &'a Path,
) -> impl FnOnce(std::io::Error) -> CompileError + 'a {
    move |error| CompileError::Io(format!("cannot {action} {}: {error}", path.display()))
}
