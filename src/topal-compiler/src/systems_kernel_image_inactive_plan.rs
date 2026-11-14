//! Exact compiler proposal for inactive bytes; no runtime or load authority.
use super::{
    SystemsKernelImageLayout,
    translation_tables::{BankKind, TranslationTables},
};
use crate::CompileError;

#[derive(Clone, Debug)]
pub(crate) struct InactiveTablePlan(TranslationTables);

impl InactiveTablePlan {
    pub(crate) fn new(layout: &SystemsKernelImageLayout) -> Result<Self, CompileError> {
        Ok(Self(TranslationTables::new(layout)?))
    }
    pub(crate) fn pages(&self) -> u64 {
        self.0.pages()
    }
    pub(crate) fn matches(&self, layout: &SystemsKernelImageLayout) -> Result<bool, CompileError> {
        self.0.matches_layout(layout)
    }
    pub(crate) fn llvm(&self) -> String {
        self.0.llvm() + &self.0.verification_llvm()
    }
    pub(crate) fn encode(&self, base: u64, low: Option<u64>) -> Result<Vec<u8>, CompileError> {
        self.0.encode(
            base,
            low.map_or(BankKind::Bootstrap, |low_user_table| BankKind::Process {
                low_user_table,
            }),
        )
    }
}
