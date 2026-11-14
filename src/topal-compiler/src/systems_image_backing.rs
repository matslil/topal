//! Inactive backing only. A compilation proposal is not executable image or
//! protected-context authority. TOPAL-SYSTEMS-KERNEL-IMAGE-PREPARE-001.
use crate::{CompileError, systems_user_extent::Owner};

pub(crate) const PREPARE: &str = "topal_x86_image_backing_prepare_v1";
pub(crate) const DISCARD: &str = "topal_x86_image_backing_discard_v1";
pub(crate) const RELEASED: &str = "topal_x86_image_backing_released_v1";
pub(crate) const TABLE_PREPARE: &str = "topal_x86_image_tables_prepare_v1";
pub(crate) const TABLE_DISCARD: &str = "topal_x86_image_tables_discard_v1";
pub(crate) const TABLE_RELEASED: &str = "topal_x86_image_tables_released_v1";
pub(crate) const TABLE_RETAIN: &str = "topal_x86_image_tables_retain_v1";
pub(crate) const RETAINED_DISCARD: &str = "topal_x86_retained_tables_discard_v1";
pub(crate) const RETAINED_RELEASED: &str = "topal_x86_retained_tables_released_v1";

#[derive(Clone, Debug)]
pub(crate) struct Plan {
    pages: u64,
    tables: bool,
    retained: bool,
    startup_storage: bool,
    #[cfg(test)]
    table_plan: Option<crate::systems_kernel_image_layout::InactiveTablePlan>,
}

impl Plan {
    #[cfg(test)]
    pub(crate) fn from_layout(
        layout: &crate::SystemsKernelImageLayout,
    ) -> Result<Self, CompileError> {
        Ok(Self {
            pages: crate::systems_kernel_image_layout::preparation_pages(layout)?,
            tables: false,
            retained: false,
            startup_storage: false,
            table_plan: None,
        })
    }
    #[cfg(test)]
    pub(crate) fn for_tables(
        layout: &crate::SystemsKernelImageLayout,
    ) -> Result<Self, CompileError> {
        let table_plan = crate::systems_kernel_image_layout::InactiveTablePlan::new(layout)?;
        Ok(Self {
            pages: table_plan.pages(),
            tables: true,
            retained: false,
            startup_storage: false,
            table_plan: Some(table_plan),
        })
    }
    pub(crate) fn is_tables(&self) -> bool {
        self.tables
    }
    pub(crate) fn is_retained(&self) -> bool {
        self.retained
    }
    pub(crate) fn has_startup_storage(&self) -> bool {
        self.startup_storage
    }
    #[cfg(test)]
    pub(crate) fn for_startup_storage(
        layout: &crate::SystemsKernelImageLayout,
    ) -> Result<Self, CompileError> {
        let mut plan = Self::for_retained_tables(layout)?;
        plan.startup_storage = true;
        Ok(plan)
    }
    #[cfg(test)]
    pub(crate) fn for_retained_tables(
        layout: &crate::SystemsKernelImageLayout,
    ) -> Result<Self, CompileError> {
        let mut plan = Self::for_tables(layout)?;
        plan.retained = true;
        Ok(plan)
    }
    #[cfg(test)]
    pub(crate) fn table_plan(
        &self,
    ) -> Result<&crate::systems_kernel_image_layout::InactiveTablePlan, CompileError> {
        self.table_plan
            .as_ref()
            .filter(|_| self.tables)
            .ok_or_else(|| {
                CompileError::Tool("table preparation requires its exact image template".into())
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
        Ok(
            self.pages == crate::systems_kernel_image_layout::preparation_pages(layout)?
                && (!self.tables || self.table_plan()?.matches(layout)?),
        )
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Bindings {
    pub root_object_sha256: String,
    pub token_bytes: u64,
    pub banks: [Owner; 2],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_user_leaf: Option<Owner>,
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
        if plan.is_tables() != self.low_user_leaf.is_some() {
            return Err(CompileError::Tool(
                "table preparation requires its separate low leaf owner".into(),
            ));
        }
        if let Some(owner) = self.low_user_leaf {
            if owner.count != 1
                || owner.alignment != 1
                || owner.pins != 1
                || owner.purpose != P::PreparedImageLowUserLeaf
                || owner.slot >= self.token_bytes / 24
                || self.banks.iter().any(|bank| bank.slot == owner.slot)
            {
                return Err(CompileError::Tool(
                    "invalid whole low USER leaf owner".into(),
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
#[path = "systems_image_tables_native.rs"]
mod tables_native;

#[cfg(test)]
pub(crate) use native::{inspect_instructions_selected, llvm};

#[cfg(test)]
#[path = "systems_image_backing_inspection.rs"]
mod inspection;
#[cfg(test)]
pub(crate) use inspection::inspect;
#[cfg(test)]
pub(crate) use inspection::inspect_selected;

#[cfg(test)]
#[path = "../../../linux-kernel/tools/user-memory/image_backing_source.rs"]
mod source;
#[cfg(test)]
pub(crate) use source::{with_image_preparation, with_image_tables, with_retained_image_tables};
#[cfg(test)]
#[path = "systems_image_backing_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "systems_image_tables_tests.rs"]
mod table_tests;

#[cfg(test)]
#[path = "systems_retained_tables_tests.rs"]
mod retained_tests;
