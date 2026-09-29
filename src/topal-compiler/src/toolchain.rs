use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use topal_language::compiler::CompilerProgram;

use crate::{
    CompileError, CompileOptions, DigestEntry, Emit, LLVM_MAJOR, NativeArtifactMetadata,
    artifact::sha256,
    backend::LlvmModule,
    frontend::{SharedObjectLink, StaticArchiveLink},
    metadata_path,
};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);
const DYNAMIC_LOADER: &str = "/lib64/ld-linux-x86-64.so.2";

struct ForeignLinkInputs<'a> {
    static_archives: &'a [StaticArchiveLink],
    shared_objects: &'a [SharedObjectLink],
    dependencies: &'a [DigestEntry],
}

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
    static_archives: &[StaticArchiveLink],
    shared_objects: &[SharedObjectLink],
    foreign_dependencies: &[DigestEntry],
    llvm: &LlvmModule,
    options: &CompileOptions,
) -> Result<NativeArtifactMetadata, CompileError> {
    let tools = LlvmTools::discover(options.llvm_tools.as_deref())?;
    let mut foreign_symbols = verify_static_archives(static_archives, &tools)?;
    for (identity, symbols) in verify_shared_objects(shared_objects, &tools)? {
        foreign_symbols.entry(identity).or_default().extend(symbols);
    }
    verify_foreign_symbols(program, &foreign_symbols)?;
    let output_parent = options.output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(output_parent).map_err(|error| {
        CompileError::Io(format!(
            "cannot create output directory {}: {error}",
            output_parent.display()
        ))
    })?;
    let temporary = temporary_directory(output_parent)?;
    let result = materialize_in(
        program,
        &ForeignLinkInputs {
            static_archives,
            shared_objects,
            dependencies: foreign_dependencies,
        },
        llvm.as_bytes(),
        options,
        &tools,
        &temporary,
    );
    let _ = fs::remove_dir_all(&temporary);
    result
}

fn verify_shared_objects(
    shared_objects: &[SharedObjectLink],
    tools: &LlvmTools,
) -> Result<BTreeMap<String, BTreeSet<String>>, CompileError> {
    let mut libraries = BTreeMap::new();
    for shared in shared_objects {
        let inspected = run_output(
            Command::new(tools.path("llvm-readobj"))
                .args([
                    "--file-header",
                    "--program-headers",
                    "--dynamic-table",
                    "--needed-libs",
                ])
                .arg(&shared.path),
        )?;
        let description = String::from_utf8_lossy(&inspected.stdout);
        if !description.contains("Format: elf64-x86-64")
            || !description.contains("Type: SharedObject")
            || !description.contains(&format!("Library soname: [{}]", shared.soname))
        {
            return Err(CompileError::Tool(format!(
                "C shared object {} does not match qualified SONAME `{}`",
                shared.path.display(),
                shared.soname
            )));
        }
        let needed = description
            .split_once("NeededLibraries [")
            .and_then(|(_, suffix)| suffix.split_once(']'))
            .map_or("", |(entries, _)| entries)
            .trim();
        if !needed.is_empty() {
            return Err(CompileError::Tool(format!(
                "C shared object {} has undeclared dynamic dependencies",
                shared.path.display()
            )));
        }
        if [" INIT ", " INIT_ARRAY ", " FINI ", " FINI_ARRAY "]
            .iter()
            .any(|tag| description.contains(tag))
        {
            return Err(CompileError::Tool(format!(
                "C shared object {} contains a load or unload initializer",
                shared.path.display()
            )));
        }
        let undefined = run_output(
            Command::new(tools.path("llvm-nm"))
                .args(["--undefined-only", "--extern-only"])
                .arg(&shared.path),
        )?;
        if !String::from_utf8_lossy(&undefined.stdout).trim().is_empty() {
            return Err(CompileError::Tool(format!(
                "C shared object {} contains unresolved external symbols",
                shared.path.display()
            )));
        }
        let names = run_output(
            Command::new(tools.path("llvm-nm"))
                .args(["--defined-only", "--extern-only"])
                .arg(&shared.path),
        )?;
        let symbols = libraries
            .entry(shared.library_identity.clone())
            .or_insert_with(BTreeSet::new);
        symbols.extend(
            String::from_utf8_lossy(&names.stdout)
                .split_whitespace()
                .map(str::to_owned),
        );
    }
    Ok(libraries)
}

fn verify_static_archives(
    static_archives: &[StaticArchiveLink],
    tools: &LlvmTools,
) -> Result<BTreeMap<String, BTreeSet<String>>, CompileError> {
    let mut libraries = BTreeMap::new();
    for archive in static_archives {
        let headers = run_output(
            Command::new(tools.path("llvm-readobj"))
                .arg("--file-headers")
                .arg(&archive.path),
        )?;
        let headers = String::from_utf8_lossy(&headers.stdout);
        if !headers.contains("elf64-x86-64") && !headers.contains("x86_64") {
            return Err(CompileError::Tool(format!(
                "C static archive {} does not contain qualified x86-64 ELF objects",
                archive.path.display()
            )));
        }
        let names = run_output(
            Command::new(tools.path("llvm-nm"))
                .args(["--defined-only", "--extern-only"])
                .arg(&archive.path),
        )?;
        let symbols = libraries
            .entry(archive.library_identity.clone())
            .or_insert_with(BTreeSet::new);
        symbols.extend(
            String::from_utf8_lossy(&names.stdout)
                .split_whitespace()
                .map(str::to_owned),
        );
    }
    Ok(libraries)
}

fn verify_foreign_symbols(
    program: &CompilerProgram,
    symbols: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), CompileError> {
    for function in program
        .functions
        .iter()
        .filter_map(|function| function.foreign.as_ref())
    {
        if !symbols
            .get(&function.library_identity)
            .is_some_and(|symbols| symbols.contains(&function.external_symbol))
        {
            return Err(CompileError::Tool(format!(
                "selected C libraries do not define `{}`",
                function.external_symbol
            )));
        }
    }
    Ok(())
}

fn materialize_in(
    program: &CompilerProgram,
    foreign: &ForeignLinkInputs<'_>,
    llvm: &[u8],
    options: &CompileOptions,
    tools: &LlvmTools,
    temporary: &Path,
) -> Result<NativeArtifactMetadata, CompileError> {
    let generated = generate_native_output(foreign, llvm, options, tools, temporary)?;
    let output_bytes =
        fs::read(&generated).map_err(io_error("read generated output", &generated))?;
    let deploy_shared_objects =
        options.emit == Emit::Executable && !foreign.shared_objects.is_empty();
    let additional_platform_requirements =
        shared_platform_requirements(foreign.shared_objects, deploy_shared_objects);
    let metadata = NativeArtifactMetadata::for_program(
        program,
        foreign.dependencies,
        &additional_platform_requirements,
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

    let staged_shared_objects = stage_shared_objects(
        foreign.shared_objects,
        deploy_shared_objects,
        &options.output,
        temporary,
    )?;
    let published_shared_objects = publish_shared_objects(staged_shared_objects)?;
    let sidecar = metadata_path(&options.output);
    if let Err(error) = fs::rename(&staged_metadata, &sidecar) {
        remove_files(&published_shared_objects);
        return Err(CompileError::Io(format!(
            "cannot publish metadata {}: {error}",
            sidecar.display()
        )));
    }
    if let Err(error) = fs::rename(&staged_output, &options.output) {
        let _ = fs::remove_file(&sidecar);
        remove_files(&published_shared_objects);
        return Err(CompileError::Io(format!(
            "cannot publish output {}: {error}",
            options.output.display()
        )));
    }
    Ok(metadata)
}

fn generate_native_output(
    foreign: &ForeignLinkInputs<'_>,
    llvm: &[u8],
    options: &CompileOptions,
    tools: &LlvmTools,
    temporary: &Path,
) -> Result<PathBuf, CompileError> {
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
                let mut linker = Command::new(tools.path("rust-lld"));
                linker.arg("-flavor").arg("gnu");
                if foreign.shared_objects.is_empty() {
                    linker.arg("-static").arg("--no-dynamic-linker");
                } else {
                    linker
                        .arg("--dynamic-linker")
                        .arg(DYNAMIC_LOADER)
                        .arg("--enable-new-dtags")
                        .arg("-rpath")
                        .arg("$ORIGIN");
                }
                linker
                    .arg("-pie")
                    .arg("-e")
                    .arg("_start")
                    .arg(&object)
                    .args(foreign.static_archives.iter().map(|archive| &archive.path));
                if !foreign.shared_objects.is_empty() {
                    linker.arg("--no-as-needed");
                    for shared in foreign.shared_objects {
                        linker.arg(&shared.path);
                    }
                }
                linker.arg("-o").arg(&executable);
                run(&mut linker)?;
                executable
            }
        }
    };
    Ok(generated)
}

type StagedSharedObject = (PathBuf, PathBuf, String);

fn shared_platform_requirements(shared_objects: &[SharedObjectLink], deploy: bool) -> Vec<String> {
    if deploy {
        let mut requirements = vec![format!("elf-interpreter:{DYNAMIC_LOADER}")];
        requirements.extend(
            shared_objects
                .iter()
                .map(|shared| format!("shared-object:{}", shared.soname)),
        );
        requirements.sort();
        requirements.dedup();
        requirements
    } else {
        Vec::new()
    }
}

fn stage_shared_objects(
    shared_objects: &[SharedObjectLink],
    deploy: bool,
    output: &Path,
    temporary: &Path,
) -> Result<Vec<StagedSharedObject>, CompileError> {
    if !deploy {
        return Ok(Vec::new());
    }
    let output_parent = output.parent().unwrap_or_else(|| Path::new("."));
    let mut staged_objects = Vec::new();
    for (index, shared) in shared_objects.iter().enumerate() {
        let destination = output_parent.join(&shared.soname);
        if destination == output {
            return Err(CompileError::Tool(format!(
                "output path collides with C shared-object SONAME `{}`",
                shared.soname
            )));
        }
        let source_bytes =
            fs::read(&shared.path).map_err(io_error("read C shared object", &shared.path))?;
        if sha256(&source_bytes) != shared.sha256 {
            return Err(CompileError::Tool(format!(
                "C shared object {} changed after frontend validation",
                shared.path.display()
            )));
        }
        if destination.exists() {
            let existing = fs::read(&destination)
                .map_err(io_error("read deployed C shared object", &destination))?;
            if sha256(&existing) != shared.sha256 {
                return Err(CompileError::Tool(format!(
                    "deployed C shared object {} conflicts with required digest",
                    destination.display()
                )));
            }
            continue;
        }
        let staged = temporary.join(format!("published-shared-{index}"));
        fs::write(&staged, source_bytes).map_err(io_error("stage C shared object", &staged))?;
        staged_objects.push((staged, destination, shared.sha256.clone()));
    }
    Ok(staged_objects)
}

fn publish_shared_objects(
    staged_objects: Vec<StagedSharedObject>,
) -> Result<Vec<PathBuf>, CompileError> {
    let mut published_objects = Vec::new();
    for (staged, destination, expected_digest) in staged_objects {
        if let Err(error) = fs::hard_link(&staged, &destination) {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                let existing = fs::read(&destination).map_err(|read_error| {
                    remove_files(&published_objects);
                    CompileError::Io(format!(
                        "cannot read concurrently deployed C shared object {}: {read_error}",
                        destination.display()
                    ))
                })?;
                if sha256(&existing) == expected_digest {
                    continue;
                }
            }
            remove_files(&published_objects);
            return Err(CompileError::Io(format!(
                "cannot publish C shared object {}: {error}",
                destination.display()
            )));
        }
        published_objects.push(destination);
    }
    Ok(published_objects)
}

fn remove_files(paths: &[PathBuf]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
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
