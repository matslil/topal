//! Reconstruct from immutable source inputs, never from supplied native proofs.
use crate::systems_tail_linked::{self as linked, Placement, Section};
use crate::{CompileError, LlvmTools, SystemsSourceLibraryInput, SystemsSourceModuleInput};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path},
    process::Command,
};
use topal_language::compiler::{
    CompilerSystemsProgram, CompilerSystemsSourceLibraryInterface as Interface,
    CompilerSystemsTargetSelection, analyze_systems_for_compiler,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub source: String,
    pub source_name: String,
    pub text: String,
    pub modules: Vec<SystemsSourceModuleInput>,
}
impl From<&SystemsSourceLibraryInput> for Source {
    fn from(input: &SystemsSourceLibraryInput) -> Self {
        Self {
            source: input.source.clone(),
            source_name: input.source_name.clone(),
            text: input.text.clone(),
            modules: input.modules.clone(),
        }
    }
}
impl Source {
    fn input(&self) -> SystemsSourceLibraryInput {
        SystemsSourceLibraryInput {
            source: self.source.clone(),
            source_name: self.source_name.clone(),
            text: self.text.clone(),
            modules: self.modules.clone(),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeBounds {
    pub source: u64,
    pub checkpoint: u64,
    pub completion: u64,
    pub owner: u64,
    pub combined: u64,
    pub combined_with_caller: u64,
    pub retired_controller: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputObject {
    pub role: String,
    pub sha256: String,
    pub sections: Vec<Section>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub schema: String,
    pub root_source: String,
    pub library: Source,
    pub llvm_version: String,
    pub component_sha256: String,
    pub bounds: NativeBounds,
    pub objects: Vec<InputObject>,
    pub linked: Vec<Placement>,
}
pub(crate) struct Rebuilt {
    pub program: CompilerSystemsProgram,
    pub record: Binding,
    pub objects: Vec<Vec<u8>>,
}
fn invalid(message: impl std::fmt::Display) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-TAIL-BINDING: {message}"))
}
fn closed_llvm(llvm: &str) -> String {
    // Kernel inputs use a sealed static link, with no dynamic interposition.
    // LLVM must encode that fact for references to externally defined roles;
    // otherwise it may emit a GOT indirection even with static relocation.
    llvm.lines()
        .map(|line| {
            if let Some(rest) = line.strip_prefix("declare ") {
                if !line.contains("@llvm.") && !rest.starts_with("dso_local ") {
                    return format!("declare dso_local {rest}\n");
                }
            }
            if let Some(rest) = line.strip_prefix("define ") {
                if !["internal ", "private ", "dso_local "]
                    .iter()
                    .any(|prefix| rest.starts_with(prefix))
                {
                    return format!("define dso_local {rest}\n");
                }
            }
            if line.starts_with('@') && line.contains(" = external ") {
                return format!(
                    "{}\n",
                    line.replacen(" = external ", " = external dso_local ", 1)
                );
            }
            format!("{line}\n")
        })
        .collect()
}
fn run(work: &Path, label: &str, command: &mut Command) -> Result<(), CompileError> {
    let result = command.output().map_err(invalid)?;
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|s| s.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    fs::write(
        work.join(format!("{label}.command.json")),
        serde_json::to_vec_pretty(
            &serde_json::json!({"argv":argv,"exit_code":result.status.code()}),
        )
        .map_err(invalid)?,
    )
    .map_err(invalid)?;
    fs::write(work.join(format!("{label}.stdout.log")), &result.stdout).map_err(invalid)?;
    fs::write(work.join(format!("{label}.stderr.log")), &result.stderr).map_err(invalid)?;
    if !result.status.success() {
        return Err(invalid(format!(
            "{label}: {}",
            String::from_utf8_lossy(&result.stderr)
        )));
    }
    Ok(())
}

pub(crate) fn rebuild(
    root: &str,
    source: &Source,
    tools: &LlvmTools,
    work: &Path,
) -> Result<Rebuilt, CompileError> {
    if root.len() > 1024 * 1024
        || source.source_name.len() > 4096
        || source.source_name.contains('\0')
        || source.source.len() > 4096
        || source.source.is_empty()
        || Path::new(&source.source)
            .extension()
            .is_none_or(|e| e != "t")
        || Path::new(&source.source)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(invalid("unqualified source snapshot/path"));
    }
    let input = source.input();
    let normalized = Path::new(&source.source)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    if normalized != source.source {
        return Err(invalid("primary source path is not normalized"));
    }
    let modules = crate::systems_library::modules::validate(&input)?;
    let program =
        analyze_systems_for_compiler(root, &CompilerSystemsTargetSelection::initial_x86_64_qemu())
            .map_err(CompileError::Diagnostic)?;
    let requirement = program
        .region_library
        .as_ref()
        .ok_or_else(|| invalid("missing tail declaration"))?;
    if requirement.interface != Interface::BorrowedByteRegionTailActionV1
        || requirement.source != source.source
    {
        return Err(invalid("source and tail slot disagree"));
    }
    requirement.validate().map_err(|e| invalid(e.to_string()))?;
    topal_semantics::model_systems_transitions(&program).map_err(|e| invalid(e.to_string()))?;
    let body = topal_language::compiler::analyze_kernel_region_tail_source(
        &input.text,
        &modules,
        &requirement.export,
        3,
    )
    .map_err(|e| CompileError::Tool(e.render(&input.source_name)))?;
    crate::systems_region_library::check_region_isolation(&input, &body, &modules)?;
    let entries = body
        .functions
        .iter()
        .filter(|f| {
            f.module_identity.is_none()
                && topal_source::canonically_equal(&f.source_name, &requirement.export)
        })
        .collect::<Vec<_>>();
    let [entry] = entries.as_slice() else {
        return Err(invalid("ambiguous source export"));
    };
    let owner = crate::emit_systems_tail_owner_component(&body, &input.source_name, &entry.symbol)?;
    let component_sha256 = crate::artifact::sha256(&serde_json::to_vec(&owner).map_err(invalid)?);
    fs::create_dir_all(work).map_err(invalid)?;
    fs::write(
        work.join("root.o"),
        crate::systems_artifact::generate_root_object(&program)?,
    )
    .map_err(invalid)?;
    fs::write(
        work.join("provider.o"),
        crate::generate_x86_64_systems_provider_object(&program)?.bytes,
    )
    .map_err(invalid)?;
    for (name, llvm) in [
        ("source", &owner.runtime.source_step.llvm),
        ("runtime", &owner.runtime.llvm),
        ("owner", &owner.llvm),
    ] {
        let path = work.join(format!("{name}.ll"));
        fs::write(&path, llvm).map_err(invalid)?;
        let qualified = work.join(format!("{name}.native.ll"));
        fs::write(&qualified, closed_llvm(llvm)).map_err(invalid)?;
        run(
            work,
            &format!("{name}-verify"),
            Command::new(tools.tool_path("opt"))
                .args(["-passes=verify", "-disable-output"])
                .arg(&qualified),
        )?;
        let retained = work.join(format!("{name}.retained.ll"));
        let native_input = if name == "source" {
            // Shared value IR contains unreachable hosted fragments. Keep the
            // exact exported source/query closure; do not admit their unused
            // mutable task counter into a portable kernel source component.
            run(
                work,
                "source-retain",
                Command::new(tools.tool_path("opt"))
                    .args(["-S", "-passes=globaldce,verify"])
                    .arg(&qualified)
                    .arg("-o")
                    .arg(&retained),
            )?;
            let text = fs::read_to_string(&retained).map_err(invalid)?;
            let text = text
                .lines()
                .map(|line| {
                    if line.starts_with("; ModuleID = ") {
                        "; ModuleID = 'topal.tail-source.native'"
                    } else {
                        line
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            fs::write(&retained, text).map_err(invalid)?;
            &retained
        } else {
            &qualified
        };
        run(
            work,
            &format!("{name}-llc"),
            Command::new(tools.tool_path("llc"))
                .args([
                    "-filetype=obj",
                    "-O2",
                    "-mcpu=x86-64",
                    "-mtriple=x86_64-unknown-none",
                    "-relocation-model=static",
                    "-mattr=-mmx,-sse,-sse2,-avx,-avx2",
                    "--frame-pointer=all",
                    "-stack-size-section",
                    "-function-sections",
                    "--disable-tail-calls",
                ])
                .arg(native_input)
                .arg("-o")
                .arg(work.join(format!("{name}.o"))),
        )?;
        run(
            work,
            &format!("{name}-disassembly"),
            Command::new(tools.tool_path("llvm-objdump"))
                .args(["-d", "--no-show-raw-insn"])
                .arg(work.join(format!("{name}.o"))),
        )?;
        crate::systems_library::inspect_instructions(
            &fs::read_to_string(work.join(format!("{name}-disassembly.stdout.log")))
                .map_err(invalid)?,
        )?;
    }
    let inner = crate::systems_tail_runtime::inspection::inspect(
        &fs::read(work.join("source.o")).map_err(invalid)?,
        &fs::read(work.join("runtime.o")).map_err(invalid)?,
    )?;
    let outer = crate::systems_tail_owner::inspection::inspect(
        &fs::read(work.join("owner.o")).map_err(invalid)?,
        &inner,
    )?;
    let word = |v: &serde_json::Value, key: &str| {
        v[key]
            .as_u64()
            .ok_or_else(|| invalid("missing native bound"))
    };
    let own = word(&outer, "owner_stack_bytes")?;
    let combined = word(&outer, "combined_normal_stack_bytes")?;
    let bounds = NativeBounds {
        source: word(&inner, "maximum_source_stack_bytes")?,
        checkpoint: word(&inner, "maximum_checkpoint_stack_bytes")?,
        completion: word(&inner, "maximum_completion_stack_bytes")?,
        owner: own,
        combined,
        combined_with_caller: combined
            .checked_add(136)
            .ok_or_else(|| invalid("caller bound overflow"))?,
        retired_controller: own
            .checked_add(80)
            .ok_or_else(|| invalid("retirement bound overflow"))?
            .div_ceil(8)
            * 8,
    };
    fs::write(work.join("boundary.o"), &owner.runtime.boundary).map_err(invalid)?;
    fs::write(
        work.join("kernel-owner.o"),
        crate::systems_provider_object::tail_boundary::kernel::component_object(
            program.bootstrap_storage.capacity_bytes,
            own,
            combined,
        )?,
    )
    .map_err(invalid)?;
    let mut objects = Vec::new();
    let mut records = Vec::new();
    for role in [
        "root",
        "provider",
        "source",
        "runtime",
        "owner",
        "boundary",
        "kernel-owner",
    ] {
        let path = work.join(format!("{role}.o"));
        let sections = linked::prepare(tools, &path, role)?;
        let bytes = fs::read(path).map_err(invalid)?;
        records.push(InputObject {
            role: role.into(),
            sha256: crate::artifact::sha256(&bytes),
            sections,
        });
        objects.push(bytes);
    }
    Ok(Rebuilt {
        program,
        record: Binding {
            schema: "topal.tail-root-reconstruction.x86_64/1".into(),
            root_source: root.into(),
            library: source.clone(),
            llvm_version: tools.version.clone(),
            component_sha256,
            bounds,
            objects: records,
            linked: Vec::new(),
        },
        objects,
    })
}
impl Rebuilt {
    pub(crate) fn bind(&mut self, kernel: &[u8]) -> Result<(), CompileError> {
        let objects = self
            .objects
            .iter()
            .zip(&self.record.objects)
            .map(|(bytes, record)| (bytes.as_slice(), record.sections.as_slice()))
            .collect::<Vec<_>>();
        self.record.linked = linked::inspect(kernel, &objects)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "systems_tail_publication_tests.rs"]
mod tests;

#[path = "systems_tail_publication_io.rs"]
mod publication_io;
pub(crate) use publication_io::{publish_outputs, validate_boot};
