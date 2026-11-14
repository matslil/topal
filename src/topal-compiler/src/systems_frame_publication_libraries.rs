//! All non-frame slots are rebuilt from explicit immutable snapshots as well.
use super::*;
use std::collections::BTreeMap;
use topal_language::compiler::{
    CompilerSystemsProgram, CompilerSystemsSourceLibraryInterface as Interface,
};

pub(super) struct Library {
    pub role: String,
    pub source: Source,
    pub original_object_sha256: String,
    pub object: Vec<u8>,
}
#[derive(Clone, Copy)]
enum Kind {
    Policy,
    Numeric,
    Root,
    File,
    Instance(&'static str),
}

pub(super) fn build(
    program: &CompilerSystemsProgram,
    sources: &BTreeMap<String, Source>,
    tools: &LlvmTools,
    work: &Path,
) -> Result<Vec<Library>, CompileError> {
    use crate::systems_region_library as region;
    let slots = [
        (
            "library-kernel",
            program.kernel_library.as_ref(),
            Kind::Policy,
        ),
        ("library-time", program.time_library.as_ref(), Kind::Numeric),
        (
            "library-image",
            program.image_library.as_ref(),
            Kind::Numeric,
        ),
        (
            "library-dispatch",
            program.dispatch_library.as_ref(),
            Kind::Numeric,
        ),
        (
            "library-process-identity",
            program.process_identity_library.as_ref(),
            Kind::Numeric,
        ),
        (
            "library-region",
            program.region_library.as_ref(),
            Kind::Root,
        ),
        (
            "library-file-position",
            program.file_position_library.as_ref(),
            Kind::File,
        ),
        (
            "library-ram-page-pool",
            program.ram_page_pool_library.as_ref(),
            Kind::Instance(region::PAGE_INSTANCE),
        ),
        (
            "library-inode-mode",
            program.inode_mode_library.as_ref(),
            Kind::Instance(region::INODE_INSTANCE),
        ),
        (
            "library-namespace",
            program.namespace_library.as_ref(),
            Kind::Instance(region::NAMESPACE_INSTANCE),
        ),
    ];
    if sources
        .keys()
        .any(|role| !slots.iter().any(|(r, _, _)| role == r))
    {
        return Err(invalid(
            "extra source slot in complete kernel reconstruction",
        ));
    }
    // Check the complete association before invoking a producer or creating
    // its workspace. A missing snapshot never selects ambient/default source.
    for (role, requirement, _) in &slots {
        match (requirement, sources.get(*role)) {
            (None, None) => {}
            (Some(r), Some(s)) if r.source == s.source => {
                r.validate().map_err(invalid)?;
            }
            _ => return Err(invalid(format!("missing/mismatched source slot {role}"))),
        }
    }
    fs::create_dir_all(work).map_err(invalid)?;
    let mut result = Vec::new();
    for (role, requirement, kind) in slots {
        let Some(requirement) = requirement else {
            continue;
        };
        let source = &sources[role];
        let input = source.input();
        let directory = work.join(role);
        fs::create_dir(&directory).map_err(invalid)?;
        let object = match kind {
            Kind::Policy => {
                crate::systems_library::build_source_library(
                    requirement,
                    &input,
                    tools,
                    &directory,
                )?
                .object
            }
            Kind::Numeric => {
                crate::systems_numeric_library::build_numeric_library(
                    requirement,
                    &input,
                    tools,
                    &directory,
                )?
                .object
            }
            Kind::Root => {
                if requirement.interface == Interface::BorrowedByteRegionInstanceActionV1 {
                    crate::build_systems_region_instance_library(
                        &input,
                        &requirement.export,
                        0,
                        region::ROOT_INSTANCE,
                        tools,
                        &directory,
                    )?
                    .object
                } else {
                    region::build_systems_region_library(
                        &input,
                        &requirement.export,
                        tools,
                        &directory,
                    )?
                    .object
                }
            }
            Kind::File => {
                if requirement.interface == Interface::BorrowedByteRegionInstanceActionV1 {
                    crate::build_systems_region_instance_library(
                        &input,
                        &requirement.export,
                        2,
                        region::FILE_INSTANCE,
                        tools,
                        &directory,
                    )?
                    .object
                } else if requirement.interface == Interface::BorrowedByteRegionFramedActionV1 {
                    crate::build_systems_region_framed_action_library(
                        &input,
                        &requirement.export,
                        2,
                        tools,
                        &directory,
                    )?
                    .object
                } else {
                    crate::build_systems_region_action_library(
                        &input,
                        &requirement.export,
                        tools,
                        &directory,
                    )?
                    .object
                }
            }
            Kind::Instance(instance) => {
                crate::build_systems_region_instance_library(
                    &input,
                    &requirement.export,
                    2,
                    instance,
                    tools,
                    &directory,
                )?
                .object
            }
        };
        result.push(Library {
            role: role.into(),
            source: source.clone(),
            original_object_sha256: crate::artifact::sha256(&object),
            object,
        });
    }
    Ok(result)
}
