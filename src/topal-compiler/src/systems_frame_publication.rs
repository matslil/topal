//! Fresh source/native reconstruction; this record alone is not publication authority.
use crate::systems_tail_linked::{self as linked, Placement};
use crate::systems_tail_publication::{InputObject, Source, closed_llvm, run};
use crate::{CompileError, LlvmTools};
use object::{Object, ObjectSymbol};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{
    fs,
    path::{Component, Path},
    process::Command,
};

#[path = "systems_frame_publication_coverage.rs"]
mod coverage;
#[path = "systems_frame_publication_libraries.rs"]
mod libraries;
#[path = "systems_frame_publication_sources.rs"]
mod sources;
pub use coverage::Padding as LinkerExtent;
#[path = "systems_frame_publication_io.rs"]
mod publication_io;
pub(crate) use publication_io::{publish_outputs, validate_boot};

#[cfg(test)]
#[path = "systems_frame_artifact_tests.rs"]
mod artifact_tests;
#[cfg(test)]
#[path = "systems_frame_publication_guest_tests.rs"]
mod guest_tests;
#[cfg(test)]
#[path = "systems_frame_publication_tests.rs"]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub dispatch: u64,
    pub combined_with_syscall_caller: u64,
    pub caller_reserve: u64,
    pub retired_scratch: u64,
    pub saved_registers: u64,
    pub return_and_alignment: u64,
    pub private_runtime: u64,
    pub source_stack: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub schema: String,
    pub root_source: String,
    pub library: Source,
    pub llvm_version: String,
    pub source_obligation_sha256: String,
    pub component_sha256: String,
    pub bounds: Bounds,
    pub objects: Vec<InputObject>,
    pub other_sources: BTreeMap<String, Source>,
    pub other_original_objects: BTreeMap<String, String>,
    pub linker_padding: Vec<LinkerExtent>,
    pub linked: Vec<Placement>,
}

pub(crate) struct Rebuilt {
    pub program: topal_language::compiler::CompilerSystemsProgram,
    pub record: Binding,
    pub objects: Vec<Vec<u8>>,
}
fn invalid(message: impl std::fmt::Display) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-FRAME-RECONSTRUCTION: {message}"))
}

pub(crate) fn library_sources(
    libraries: crate::SystemsKernelLibraries<'_>,
) -> BTreeMap<String, Source> {
    [
        ("library-kernel", libraries.policy),
        ("library-time", libraries.time),
        ("library-image", libraries.image),
        ("library-dispatch", libraries.dispatch),
        ("library-process-identity", libraries.process_identity),
        ("library-region", libraries.region),
        ("library-file-position", libraries.file_position),
        ("library-ram-page-pool", libraries.ram_page_pool),
        ("library-inode-mode", libraries.inode_mode),
        ("library-namespace", libraries.namespace),
    ]
    .into_iter()
    .filter_map(|(role, input)| input.map(|input| (role.into(), input.into())))
    .collect()
}

pub(crate) fn rebuild(
    root: &str,
    source: &Source,
    tools: &LlvmTools,
    work: &Path,
) -> Result<Rebuilt, CompileError> {
    if source.source.len() > 4096
        || source.source.is_empty()
        || Path::new(&source.source)
            .extension()
            .is_none_or(|s| s != "t")
        || Path::new(&source.source)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        || Path::new(&source.source)
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
            != source.source
    {
        return Err(invalid("unqualified primary source path"));
    }
    let checked = crate::systems_frame_policy_binding::check(root, &source.input())?;
    if checked.program.region_library.as_ref().is_some_and(|r|r.interface==
        topal_language::compiler::CompilerSystemsSourceLibraryInterface::BorrowedByteRegionTailActionV1)
    {
        return Err(invalid("independent tail owners require disjoint native namespaces"));
    }
    fs::create_dir_all(work).map_err(invalid)?;
    let root_object = crate::systems_artifact::generate_frame_root_checked(&checked)?;
    let parsed = object::File::parse(root_object.as_slice()).map_err(invalid)?;
    let tokens = parsed
        .symbol_by_name("topal_x86_frame_root_tokens_v1")
        .filter(|s| s.is_definition())
        .ok_or_else(|| invalid("missing owning Root tokens"))?
        .size();
    let owner = &checked.component;
    fs::write(work.join("root.o"), &root_object).map_err(invalid)?;
    fs::write(
        work.join("provider.o"),
        crate::systems_provider_object::generate_frame_memory_provider(&checked)?.bytes,
    )
    .map_err(invalid)?;
    fs::write(
        work.join("component.json"),
        serde_json::to_vec_pretty(owner).map_err(invalid)?,
    )
    .map_err(invalid)?;
    fs::write(
        work.join("source-obligation.json"),
        serde_json::to_vec_pretty(&checked.source).map_err(invalid)?,
    )
    .map_err(invalid)?;
    let target = crate::systems_frame_target::memory_llvm(owner, tokens)?;
    let dispatch = crate::systems_frame_target::memory_dispatch();
    for (role, llvm) in [
        ("source", owner.runtime.source_step.llvm.as_str()),
        ("runtime", owner.runtime.llvm.as_str()),
        ("owner", owner.llvm.as_str()),
        ("frame-controller", checked.controller.as_str()),
        ("frame-target", target.as_str()),
        ("frame-dispatch", dispatch.as_str()),
    ] {
        compile(role, llvm, tools, work)?;
    }
    let read = |role: &str| fs::read(work.join(format!("{role}.o"))).map_err(invalid);
    let inner =
        crate::systems_tail_runtime::inspection::inspect(&read("source")?, &read("runtime")?)?;
    let outer = crate::systems_tail_owner::inspection::inspect(&read("owner")?, &inner)?;
    let inspection = crate::systems_frame_target::inspection::inspect_gateway(
        &read("frame-controller")?,
        &read("frame-target")?,
        &read("frame-dispatch")?,
        &outer,
    )?;
    let body = inspection["roots"]["topal_frame_kernel_dispatch_v1"]
        .as_u64()
        .ok_or_else(|| invalid("missing complete native dispatch bound"))?;
    let combined = body
        .checked_add(80)
        .filter(|n| *n <= 4096)
        .ok_or_else(|| invalid("syscall owner exceeds its actual reserve"))?;
    fs::write(
        work.join("frame-inspection.json"),
        serde_json::to_vec_pretty(&inspection).map_err(invalid)?,
    )
    .map_err(invalid)?;
    fs::write(
        work.join("owner-inspection.json"),
        serde_json::to_vec_pretty(&outer).map_err(invalid)?,
    )
    .map_err(invalid)?;
    fs::write(work.join("boundary.o"), &owner.runtime.boundary).map_err(invalid)?;
    fs::write(
        work.join("frame-gateway.o"),
        crate::systems_provider_object::tail_boundary::frame_gateway::memory_component_object(
            body,
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
        "frame-controller",
        "frame-target",
        "frame-dispatch",
        "frame-gateway",
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
        program: checked.program,
        record: Binding {
            schema: "topal.frame-root-reconstruction.x86_64/1".into(),
            root_source: root.into(),
            library: source.clone(),
            llvm_version: tools.version.clone(),
            source_obligation_sha256: crate::artifact::sha256(
                &serde_json::to_vec(&checked.source).map_err(invalid)?,
            ),
            component_sha256: checked.source.component_sha256,
            bounds: Bounds {
                dispatch: body,
                combined_with_syscall_caller: combined,
                caller_reserve: 4096,
                retired_scratch: 4032,
                saved_registers: 48,
                return_and_alignment: 16,
                private_runtime: 65536,
                source_stack: 4096,
            },
            objects: records,
            other_sources: BTreeMap::new(),
            other_original_objects: BTreeMap::new(),
            linker_padding: vec![],
            linked: vec![],
        },
        objects,
    })
}

impl Rebuilt {
    pub(crate) fn complete_libraries(
        &mut self,
        sources: &BTreeMap<String, Source>,
        tools: &LlvmTools,
        work: &Path,
    ) -> Result<(), CompileError> {
        if !self.record.linked.is_empty() || !self.record.other_sources.is_empty() {
            return Err(invalid(
                "kernel libraries must be supplied once before linking",
            ));
        }
        let built = libraries::build(&self.program, sources, tools, &work.join("libraries"))?;
        for library in built {
            let path = work.join(format!("{}.o", library.role));
            fs::write(&path, &library.object).map_err(invalid)?;
            let sections = linked::prepare(tools, &path, &library.role)?;
            let bytes = fs::read(&path).map_err(invalid)?;
            self.record.objects.push(InputObject {
                role: library.role.clone(),
                sha256: crate::artifact::sha256(&bytes),
                sections,
            });
            self.record
                .other_sources
                .insert(library.role.clone(), library.source);
            self.record
                .other_original_objects
                .insert(library.role, library.original_object_sha256);
            self.objects.push(bytes);
        }
        Ok(())
    }
    pub(crate) fn bind(&mut self, kernel: &[u8]) -> Result<(), CompileError> {
        let objects = self
            .objects
            .iter()
            .zip(&self.record.objects)
            .map(|(bytes, record)| (bytes.as_slice(), record.sections.as_slice()))
            .collect::<Vec<_>>();
        let placements = linked::inspect(kernel, &objects)?;
        validate_reservation(kernel)?;
        let required = [
            &self.program.kernel_library,
            &self.program.time_library,
            &self.program.image_library,
            &self.program.dispatch_library,
            &self.program.process_identity_library,
            &self.program.region_library,
            &self.program.file_position_library,
            &self.program.ram_page_pool_library,
            &self.program.inode_mode_library,
            &self.program.namespace_library,
        ]
        .into_iter()
        .filter(|r| r.is_some())
        .count();
        if self.record.other_sources.len() == required {
            self.record.linker_padding = coverage::validate(kernel, &placements)?;
        }
        self.record.linked = placements;
        Ok(())
    }
}

pub(crate) fn validate_binding(
    kernel: &[u8],
    binding: &Binding,
    tools: &LlvmTools,
    work: &Path,
) -> Result<Rebuilt, CompileError> {
    if binding.schema != "topal.frame-root-reconstruction.x86_64/1"
        || binding.llvm_version != tools.version
    {
        return Err(invalid("unqualified reconstruction schema/toolchain"));
    }
    let mut fresh = rebuild(&binding.root_source, &binding.library, tools, work)?;
    if !binding.other_sources.is_empty() {
        fresh.complete_libraries(&binding.other_sources, tools, work)?;
    }
    fresh.bind(kernel)?;
    if fresh.record != *binding {
        return Err(invalid(
            "candidate source/native/caller/load record differs from fresh reconstruction",
        ));
    }
    Ok(fresh)
}

fn validate_reservation(kernel: &[u8]) -> Result<(), CompileError> {
    use object::{ObjectSection, ObjectSegment};
    let file = object::File::parse(kernel).map_err(invalid)?;
    if file.symbols().any(|s| s.is_undefined())
        || file.sections().any(|s| s.relocations().next().is_some())
    {
        return Err(invalid(
            "kernel retains an undefined dependency or relocation",
        ));
    }
    let phoff = u64::from_le_bytes(kernel[32..40].try_into().unwrap());
    let phsize = u16::from_le_bytes(kernel[54..56].try_into().unwrap()) as u64;
    let phnum = u16::from_le_bytes(kernel[56..58].try_into().unwrap()) as u64;
    for i in 0..phnum {
        let at = phoff
            .checked_add(i * phsize)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| invalid("program header overflow"))?;
        let raw = kernel
            .get(at..at + 4)
            .ok_or_else(|| invalid("truncated program header"))?;
        let kind = u32::from_le_bytes(raw.try_into().unwrap());
        if ![
            object::elf::PT_PHDR.0,
            object::elf::PT_LOAD.0,
            object::elf::PT_GNU_RELRO.0,
            object::elf::PT_GNU_STACK.0,
        ]
        .contains(&kind)
        {
            return Err(invalid("unqualified static-kernel program header"));
        }
    }
    let entry = file
        .symbol_by_name(crate::X86_SYSTEMS_KERNEL_ENTRY)
        .ok_or_else(|| invalid("missing actual Root entry"))?;
    if file.entry() != entry.address() || entry.kind() != object::SymbolKind::Text {
        return Err(invalid("ELF entry does not select the reconstructed Root"));
    }
    let end = file
        .segments()
        .map(|s| s.address().checked_add(s.size()))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| invalid("load extent overflow"))?
        .into_iter()
        .max()
        .ok_or_else(|| invalid("missing load geometry"))?
        .max(0x1000000);
    let symbol = file
        .symbol_by_name("topal_x86_kernel_reservation_end_v1")
        .ok_or_else(|| invalid("missing actual kernel reservation"))?;
    if symbol.section() != object::SymbolSection::Absolute
        || !symbol.is_global()
        || symbol.size() != 0
        || symbol.address() != end
        || end > 0x40000000
    {
        return Err(invalid(
            "allocator reservation omits or changes actual allocated image",
        ));
    }
    Ok(())
}

fn compile(role: &str, llvm: &str, tools: &LlvmTools, work: &Path) -> Result<(), CompileError> {
    fs::write(work.join(format!("{role}.ll")), llvm).map_err(invalid)?;
    let native = work.join(format!("{role}.native.ll"));
    fs::write(&native, closed_llvm(llvm)).map_err(invalid)?;
    run(
        work,
        &format!("{role}-verify"),
        Command::new(tools.tool_path("opt"))
            .args(["-passes=verify", "-disable-output"])
            .arg(&native),
    )?;
    let retained = work.join(format!("{role}.retained.ll"));
    let input = if role == "source" {
        run(
            work,
            "source-retain",
            Command::new(tools.tool_path("opt"))
                .args(["-S", "-passes=globaldce,verify"])
                .arg(&native)
                .arg("-o")
                .arg(&retained),
        )?;
        let text = fs::read_to_string(&retained).map_err(invalid)?;
        let text = text
            .lines()
            .map(|line| {
                if line.starts_with("; ModuleID = ") {
                    "; ModuleID = 'topal.frame-source.native'"
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
        &native
    };
    run(
        work,
        &format!("{role}-llc"),
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
            .arg(input)
            .arg("-o")
            .arg(work.join(format!("{role}.o"))),
    )?;
    run(
        work,
        &format!("{role}-disassembly"),
        Command::new(tools.tool_path("llvm-objdump"))
            .args(["-d", "--no-show-raw-insn"])
            .arg(work.join(format!("{role}.o"))),
    )?;
    if ["source", "runtime", "owner", "frame-controller"].contains(&role) {
        crate::systems_library::inspect_instructions(
            &fs::read_to_string(work.join(format!("{role}-disassembly.stdout.log")))
                .map_err(invalid)?,
        )?;
    }
    Ok(())
}
