use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use topal_semantics::{
    ArchitectureComponent, ArchitectureComponentKind, ArchitectureConnection, ArchitectureModel,
    ArchitectureProvenance,
};

use crate::{CompileError, DATA_LAYOUT, TARGET_TRIPLE};

pub const OPTIMIZATION_PLAN_REVISION: &str = "topal.optimization-plan/1";
pub const GENERIC_X86_64_MODEL: &str = "topal.architecture.generic-x86_64-linux/1";
pub const RUNTIME_GLOBAL_DCE: &str = "topal.runtime-global-dce/1";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OptimizationLevel {
    #[default]
    O0,
    O1,
    O2,
    O3,
    Os,
    Oz,
}

impl OptimizationLevel {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::O0 => "O0",
            Self::O1 => "O1",
            Self::O2 => "O2",
            Self::O3 => "O3",
            Self::Os => "Os",
            Self::Oz => "Oz",
        }
    }

    #[must_use]
    pub const fn artifact_code(self) -> u8 {
        match self {
            Self::O0 => 0,
            Self::O1 => 1,
            Self::O2 => 2,
            Self::O3 => 3,
            Self::Os => 4,
            Self::Oz => 5,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TargetSelection {
    pub target: Option<String>,
    pub cpu: Option<String>,
    pub board: Option<String>,
    pub model: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OptimizationRequest {
    pub level: OptimizationLevel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationPlan {
    pub revision: &'static str,
    pub level: OptimizationLevel,
    pub target_triple: String,
    pub cpu: String,
    pub board: Option<String>,
    pub architecture_model: &'static str,
    pub architecture_model_sha256: String,
    pub features: Vec<String>,
    pub enabled_optimizations: Vec<&'static str>,
}

impl OptimizationPlan {
    /// Resolve and validate the target and policy inputs supported by this
    /// compiler increment.
    ///
    /// # Errors
    ///
    /// Returns an explicit unsupported or incompatible selection error.
    pub fn resolve(
        target: &TargetSelection,
        optimization: &OptimizationRequest,
    ) -> Result<Self, CompileError> {
        let target_triple = match target.target.as_deref() {
            Some(TARGET_TRIPLE) => TARGET_TRIPLE,
            Some(other) => {
                return Err(CompileError::Tool(format!(
                    "unsupported target `{other}`; this increment has only the qualified `{TARGET_TRIPLE}` model"
                )));
            }
            None if cfg!(all(target_arch = "x86_64", target_os = "linux")) => TARGET_TRIPLE,
            None => {
                return Err(CompileError::Tool(format!(
                    "the build host has no qualified generic target; select a supported --target (this increment has only `{TARGET_TRIPLE}`)"
                )));
            }
        };
        let cpu = target.cpu.as_deref().unwrap_or("generic");
        if cpu != "generic" {
            return Err(CompileError::Tool(format!(
                "unsupported CPU profile `{cpu}`; this increment qualifies only `generic` and does not silently detect native features"
            )));
        }
        if let Some(board) = &target.board {
            return Err(CompileError::Tool(format!(
                "unsupported board profile `{board}`; no board model is qualified by this compiler increment"
            )));
        }
        if let Some(path) = &target.model {
            return Err(CompileError::Tool(format!(
                "custom architecture model `{}` is not yet supported; this increment loads only `{GENERIC_X86_64_MODEL}`",
                path.display()
            )));
        }
        if !matches!(
            optimization.level,
            OptimizationLevel::O0 | OptimizationLevel::O1
        ) {
            return Err(CompileError::Tool(format!(
                "optimization profile -{} is specified but not implemented by this compiler increment; use -O0 or -O1",
                optimization.level.name()
            )));
        }
        let model = generic_x86_64_model();
        let architecture_model_sha256 = model.canonical_sha256().map_err(|error| {
            CompileError::Tool(format!("invalid built-in architecture model: {error}"))
        })?;
        let enabled_optimizations = match optimization.level {
            OptimizationLevel::O0 => Vec::new(),
            OptimizationLevel::O1 => vec![RUNTIME_GLOBAL_DCE],
            _ => unreachable!("unsupported profiles were rejected"),
        };
        Ok(Self {
            revision: OPTIMIZATION_PLAN_REVISION,
            level: optimization.level,
            target_triple: target_triple.into(),
            cpu: "x86-64".into(),
            board: None,
            architecture_model: GENERIC_X86_64_MODEL,
            architecture_model_sha256,
            features: Vec::new(),
            enabled_optimizations,
        })
    }
}

fn generic_x86_64_model() -> ArchitectureModel {
    let provenance = "compiler-model";
    ArchitectureModel {
        schema_revision: 1,
        language_revision: "v0.1".into(),
        name: GENERIC_X86_64_MODEL.into(),
        imports: BTreeMap::new(),
        features: BTreeMap::from([("x86-64-baseline".into(), BTreeSet::new())]),
        components: vec![
            ArchitectureComponent {
                identity: "execution.x86_64".into(),
                kind: ArchitectureComponentKind::ExecutionArchitecture,
                definition: "x86-64 baseline ISA".into(),
                count: 1,
                properties: BTreeMap::from([("feature-set".into(), "baseline".into())]),
                provenance: provenance.into(),
            },
            ArchitectureComponent {
                identity: "platform.linux-gnu".into(),
                kind: ArchitectureComponentKind::AbiPlatform,
                definition: "qualified freestanding Linux x86-64 platform".into(),
                count: 1,
                properties: BTreeMap::from([
                    ("target-triple".into(), TARGET_TRIPLE.into()),
                    ("data-layout".into(), DATA_LAYOUT.into()),
                    ("object-format".into(), "elf64-x86-64".into()),
                    ("linkage".into(), "static".into()),
                ]),
                provenance: provenance.into(),
            },
            ArchitectureComponent {
                identity: "compute.generic".into(),
                kind: ArchitectureComponentKind::Compute,
                definition: "generic x86-64 processing element".into(),
                count: 1,
                properties: BTreeMap::from([("cpu".into(), "x86-64".into())]),
                provenance: provenance.into(),
            },
            ArchitectureComponent {
                identity: "memory.process".into(),
                kind: ArchitectureComponentKind::Memory,
                definition: "Linux process virtual memory".into(),
                count: 1,
                properties: BTreeMap::from([("pointer-bits".into(), "64".into())]),
                provenance: provenance.into(),
            },
        ],
        connections: vec![ArchitectureConnection {
            identity: "compute-memory".into(),
            source: "compute.generic".into(),
            destination: "memory.process".into(),
            kind: "load-store".into(),
            resources: BTreeSet::new(),
            provenance: provenance.into(),
        }],
        costs: Vec::new(),
        provenance: vec![ArchitectureProvenance {
            identity: provenance.into(),
            provider: "topalc".into(),
            source_class: "compiler-built-in".into(),
            revision: "1".into(),
            digest: crate::artifact::sha256(include_bytes!(
                "../../../library/std/architecture/generic-x86-64-linux.t"
            )),
            assumptions: BTreeSet::from(["generic baseline; no host feature detection".into()]),
        }],
        qualifications: BTreeSet::from([
            "schema".into(),
            "implementation".into(),
            "linux-x86_64-platform".into(),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_plan_uses_generic_host_without_native_features() {
        if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
            let plan = OptimizationPlan::resolve(
                &TargetSelection::default(),
                &OptimizationRequest::default(),
            )
            .unwrap();
            assert_eq!(plan.target_triple, TARGET_TRIPLE);
            assert_eq!(plan.cpu, "x86-64");
            assert!(plan.features.is_empty());
            assert!(plan.enabled_optimizations.is_empty());
            assert_eq!(plan.architecture_model_sha256.len(), 64);
        }
    }

    #[test]
    fn rejects_unqualified_target_cpu_board_model_and_profile() {
        let unsupported = [
            TargetSelection {
                target: Some("aarch64-unknown-linux-gnu".into()),
                ..TargetSelection::default()
            },
            TargetSelection {
                cpu: Some("native".into()),
                ..TargetSelection::default()
            },
            TargetSelection {
                board: Some("example".into()),
                ..TargetSelection::default()
            },
            TargetSelection {
                model: Some("model.t".into()),
                ..TargetSelection::default()
            },
        ];
        for target in unsupported {
            assert!(OptimizationPlan::resolve(&target, &OptimizationRequest::default()).is_err());
        }
        assert!(
            OptimizationPlan::resolve(
                &TargetSelection::default(),
                &OptimizationRequest {
                    level: OptimizationLevel::O2,
                },
            )
            .is_err()
        );
    }

    #[test]
    fn o1_enables_only_qualified_runtime_global_dce() {
        let plan = OptimizationPlan::resolve(
            &TargetSelection {
                target: Some(TARGET_TRIPLE.into()),
                ..TargetSelection::default()
            },
            &OptimizationRequest {
                level: OptimizationLevel::O1,
            },
        )
        .unwrap();
        assert_eq!(plan.enabled_optimizations, [RUNTIME_GLOBAL_DCE]);
    }
}
