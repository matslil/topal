//! Inactive backing only. A compilation proposal is not executable image or
//! protected-context authority. TOPAL-SYSTEMS-KERNEL-IMAGE-PREPARE-001.
use crate::{CompileError, systems_user_extent::Owner};

pub(crate) const PREPARE: &str = "topal_x86_image_backing_prepare_v1";
pub(crate) const DISCARD: &str = "topal_x86_image_backing_discard_v1";
pub(crate) const RELEASED: &str = "topal_x86_image_backing_released_v1";

#[derive(Clone, Debug)]
pub(crate) struct Plan {
    pages: u64,
}

impl Plan {
    #[cfg(test)]
    pub(crate) fn from_layout(
        layout: &crate::SystemsKernelImageLayout,
    ) -> Result<Self, CompileError> {
        Ok(Self {
            pages: crate::systems_kernel_image_layout::preparation_pages(layout)?,
        })
    }
    pub(crate) fn pages(&self) -> u64 {
        self.pages
    }
    #[cfg(test)]
    pub(crate) fn matches(
        &self,
        layout: &crate::SystemsKernelImageLayout,
    ) -> Result<bool, CompileError> {
        Ok(self.pages == crate::systems_kernel_image_layout::preparation_pages(layout)?)
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Bindings {
    pub root_object_sha256: String,
    pub token_bytes: u64,
    pub banks: [Owner; 2],
}

impl Bindings {
    pub(crate) fn validate(&self, plan: &Plan) -> Result<(), CompileError> {
        use crate::systems_frame_ownership::Purpose as P;
        if self.token_bytes == 0
            || self.token_bytes % 24 != 0
            || self.root_object_sha256.len() != 64
            || self.banks[0].slot == self.banks[1].slot
        {
            return Err(CompileError::Tool(
                "incomplete image-backing root binding".into(),
            ));
        }
        for (owner, purpose) in self
            .banks
            .iter()
            .zip([P::PreparedImageBootstrap, P::PreparedImageProcess])
        {
            if owner.count != plan.pages
                || owner.alignment != 1
                || owner.pins != 1
                || owner.purpose != purpose
                || owner.slot >= self.token_bytes / 24
            {
                return Err(CompileError::Tool(
                    "image backing requires its exact whole allocation purpose".into(),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "systems_image_backing_native.rs"]
mod native;

#[cfg(test)]
pub(crate) use native::{inspect_instructions, llvm};

#[cfg(test)]
#[path = "systems_image_backing_inspection.rs"]
mod inspection;
#[cfg(test)]
pub(crate) use inspection::inspect;

#[cfg(test)]
#[path = "../../../linux-kernel/tools/user-memory/image_backing_source.rs"]
mod source;
#[cfg(test)]
pub(crate) use source::with_image_preparation;
#[cfg(test)]
#[path = "systems_image_backing_tests.rs"]
mod tests;
