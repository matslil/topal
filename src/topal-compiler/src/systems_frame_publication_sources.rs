//! Bind legacy policy metadata to the exact inputs used by fresh reconstruction.
use super::*;

pub(super) fn validate(
    binding: &Binding,
    artifact: &crate::SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    // An original object hash alone does not associate separately serialized
    // source metadata with the snapshots consumed by its fresh producer.
    macro_rules! check {
        ($role:literal, $record:expr) => {
            match (binding.other_sources.get($role), $record.as_ref()) {
                (None, None) => {}
                (Some(source), Some(actual)) => {
                    let input = source.input();
                    if actual.source != source.source
                        || actual.source_name != source.source_name
                        || actual.source_sha256 != crate::artifact::sha256(source.text.as_bytes())
                        || actual.modules != crate::systems_library::modules::records(&input)
                        || actual.module_snapshot_sha256
                            != crate::artifact::sha256(&crate::systems_library::modules::snapshot(
                                &input,
                            )?)
                    {
                        return Err(invalid(concat!(
                            "other source/artifact snapshot association differs: ",
                            $role
                        )));
                    }
                }
                _ => {
                    return Err(invalid(concat!(
                        "other source/artifact slot differs: ",
                        $role
                    )))
                }
            }
        };
    }
    check!("library-kernel", artifact.kernel_library);
    check!("library-time", artifact.time_library);
    check!("library-image", artifact.image_library);
    check!("library-dispatch", artifact.dispatch_library);
    check!(
        "library-process-identity",
        artifact.process_identity_library
    );
    check!("library-region", artifact.region_library);
    check!("library-file-position", artifact.file_position_library);
    check!("library-ram-page-pool", artifact.ram_page_pool_library);
    check!("library-inode-mode", artifact.inode_mode_library);
    check!("library-namespace", artifact.namespace_library);
    Ok(())
}
