//! Compiler bindings to whole source-frame allocations. These descriptors are
//! neither runtime tokens nor permission to dereference physical addresses.
//! TOPAL-SYSTEMS-KERNEL-IMAGE-OWNERS-001 / FRAME-POLICY-BINDING-001.
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Purpose {
    SourceExtent,
    PreparedImageBootstrap,
    PreparedImageProcess,
    PreparedImageLowUserLeaf,
    BootstrapHierarchy,
    TranslationEdit,
    StaticImagePayload,
    StaticImageLeaf,
    StaticImageHierarchy,
    StaticImageStack,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Allocation {
    pub count: u64,
    pub alignment: u64,
    pub purpose: Purpose,
}

impl Allocation {
    /// Refine generic extents only at the consuming construction transition.
    /// A matching numeric size alone does not assign a static-image role.
    pub(crate) fn refine_construction(
        &mut self,
        purpose: Purpose,
    ) -> Result<(), crate::CompileError> {
        let expected = match purpose {
            Purpose::StaticImagePayload => 503,
            Purpose::StaticImageLeaf => 1,
            _ => {
                return Err(crate::CompileError::Tool(
                    "invalid construction frame purpose".into(),
                ));
            }
        };
        if self.purpose != Purpose::SourceExtent || self.count != expected || self.alignment != 1 {
            return Err(crate::CompileError::Tool(
                "construction requires the exact unrefined whole frame owner".into(),
            ));
        }
        self.purpose = purpose;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EntryOwners {
    pub bootstrap: crate::systems_user_extent::Owner,
    pub process: [crate::systems_user_extent::Owner; 4],
}

#[cfg(test)]
#[path = "systems_frame_ownership_tests.rs"]
mod tests;
