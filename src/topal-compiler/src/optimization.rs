use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use topal_semantics::{
    ArchitectureComponent, ArchitectureComponentKind, ArchitectureConnection, ArchitectureModel,
    ArchitectureProvenance,
};

use crate::{CompileError, DATA_LAYOUT, TARGET_TRIPLE};

pub const OPTIMIZATION_PLAN_REVISION: &str = "topal.optimization-plan/1";
pub const TARGET_REGISTRY_REVISION: &str = "topal.target-qualification/1";
pub const GENERIC_X86_64_MODEL: &str = "topal.architecture.generic-x86_64-linux/1";
pub const RUNTIME_GLOBAL_DCE: &str = "topal.runtime-global-dce/1";
pub const LLVM_DEFAULT_PIPELINE: &str = "llvm.default-pipeline/22";

const GOALS: &[&str] = &[
    "speed",
    "latency",
    "throughput",
    "code-size",
    "peak-memory",
    "energy",
    "compile-time",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TargetQualificationStatus {
    Executable,
    ModelOnly,
}

impl TargetQualificationStatus {
    const fn name(self) -> &'static str {
        match self {
            Self::Executable => "executable-qualified",
            Self::ModelOnly => "model-only",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TargetQualification {
    profile: &'static str,
    target: Option<&'static str>,
    cpu: &'static str,
    features: &'static str,
    board: Option<&'static str>,
    model: &'static str,
    source: &'static str,
    status: TargetQualificationStatus,
    missing: &'static str,
}

const TARGET_QUALIFICATIONS: &[TargetQualification] = &[
    TargetQualification {
        profile: "generic-x86-64-linux",
        target: Some(TARGET_TRIPLE),
        cpu: "generic",
        features: "x86-64-baseline",
        board: None,
        model: GENERIC_X86_64_MODEL,
        source: "architecture-models/generic-x86-64-linux.t",
        status: TargetQualificationStatus::Executable,
        missing: "none",
    },
    TargetQualification {
        profile: "example-x86-64-avx2",
        target: Some(TARGET_TRIPLE),
        cpu: "x86-64-avx2",
        features: "avx,avx2",
        board: None,
        model: "example-x86-64-avx2",
        source: "architecture-models/example-x86-64-avx2-overlay.t",
        status: TargetQualificationStatus::ModelOnly,
        missing: "instruction legality, cost, backend, runtime, ABI, and validation qualification",
    },
    TargetQualification {
        profile: "example-riscv-dsp-board",
        target: None,
        cpu: "rv64imafdc-generic",
        features: "model-declared RISC-V CPU plus DSP accelerator",
        board: Some("example-riscv-dsp-board"),
        model: "example-riscv-dsp-board",
        source: "architecture-models/example-riscv-dsp-board.t",
        status: TargetQualificationStatus::ModelOnly,
        missing: "target triple, instruction legality, cost, backend, runtime, ABI, and validation qualification",
    },
];

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
    pub explicit_level: bool,
    pub goals: Vec<String>,
    pub limits: Vec<String>,
    pub overrides: Vec<OptimizationOverride>,
    pub only: Option<String>,
    pub explain: Option<ExplanationDestination>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptimizationOverride {
    Enable(String),
    Disable(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExplanationDestination {
    StandardError,
    Path(PathBuf),
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
    pub goals: Vec<String>,
    pub limits: Vec<String>,
    pub remarks: Vec<String>,
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
                    "unsupported target `{other}`; no executable-qualified registry entry matches it (use --list-targets)"
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
            if let Some(entry) = TARGET_QUALIFICATIONS.iter().find(|entry| entry.cpu == cpu) {
                return Err(unqualified_target_error(entry));
            }
            return Err(CompileError::Tool(format!(
                "unsupported CPU profile `{cpu}`; only `generic` is executable-qualified and native features are never detected implicitly (use --list-targets)"
            )));
        }
        if let Some(board) = &target.board {
            if let Some(entry) = TARGET_QUALIFICATIONS
                .iter()
                .find(|entry| entry.board == Some(board.as_str()))
            {
                return Err(unqualified_target_error(entry));
            }
            return Err(CompileError::Tool(format!(
                "unsupported board profile `{board}`; no executable-qualified board model matches it (use --list-targets)"
            )));
        }
        if let Some(path) = &target.model {
            if let Some(entry) = TARGET_QUALIFICATIONS.iter().find(|entry| {
                path.file_name()
                    .is_some_and(|name| PathBuf::from(entry.source).file_name() == Some(name))
            }) {
                if entry.status == TargetQualificationStatus::ModelOnly {
                    return Err(unqualified_target_error(entry));
                }
            }
            return Err(CompileError::Tool(format!(
                "custom architecture model `{}` is not executable-qualified; this increment loads only `{GENERIC_X86_64_MODEL}` (use --list-targets)",
                path.display()
            )));
        }
        let model = generic_x86_64_model();
        let architecture_model_sha256 = model.canonical_sha256().map_err(|error| {
            CompileError::Tool(format!("invalid built-in architecture model: {error}"))
        })?;
        let mut level = optimization.level;
        let mut enabled_optimizations = match level {
            OptimizationLevel::O0 => Vec::new(),
            OptimizationLevel::O1 => vec![RUNTIME_GLOBAL_DCE],
            OptimizationLevel::O2
            | OptimizationLevel::O3
            | OptimizationLevel::Os
            | OptimizationLevel::Oz => vec![LLVM_DEFAULT_PIPELINE],
        };
        if let Some(identity) = &optimization.only {
            if optimization.explicit_level {
                return Err(CompileError::Tool(
                    "--only-optimization is incompatible with a standard optimization profile"
                        .into(),
                ));
            }
            let known = known_optimization(identity)?;
            enabled_optimizations = vec![known];
            level = if known == RUNTIME_GLOBAL_DCE {
                OptimizationLevel::O1
            } else {
                OptimizationLevel::O2
            };
        }
        for entry in &optimization.overrides {
            let (identity, enable) = match entry {
                OptimizationOverride::Enable(identity) => (known_optimization(identity)?, true),
                OptimizationOverride::Disable(identity) => (known_optimization(identity)?, false),
            };
            enabled_optimizations.retain(|candidate| *candidate != identity);
            if enable {
                enabled_optimizations.push(identity);
            }
        }
        if enabled_optimizations.contains(&LLVM_DEFAULT_PIPELINE) && level == OptimizationLevel::O0
        {
            return Err(CompileError::Tool(
                "llvm.default-pipeline/22 requires -O1 or a higher optimization profile".into(),
            ));
        }
        enabled_optimizations.sort_unstable();
        enabled_optimizations.dedup();
        let goals = if optimization.goals.is_empty() {
            default_goals(level)
                .iter()
                .map(|goal| (*goal).to_owned())
                .collect()
        } else {
            optimization
                .goals
                .iter()
                .map(|goal| validate_goal(goal).map(str::to_owned))
                .collect::<Result<Vec<_>, _>>()?
        };
        for limit in &optimization.limits {
            validate_limit(limit)?;
        }
        let remarks = if level == OptimizationLevel::O0 {
            Vec::new()
        } else {
            vec![
                "remark[optimization-workload]: no qualified workload profile was provided; target-independent LLVM heuristics remain conservative"
                    .into(),
                "remark[optimization-source-facts]: no verified hotness or trip-count evidence is available; future admitted source resource evidence could improve inlining and loop decisions"
                    .into(),
            ]
        };
        Ok(Self {
            revision: OPTIMIZATION_PLAN_REVISION,
            level,
            target_triple: target_triple.into(),
            cpu: "x86-64".into(),
            board: None,
            architecture_model: GENERIC_X86_64_MODEL,
            architecture_model_sha256,
            features: Vec::new(),
            enabled_optimizations,
            goals,
            limits: optimization.limits.clone(),
            remarks,
        })
    }

    /// Render the canonical, deterministic decision explanation.
    #[must_use]
    pub fn explanation(&self) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "topal.optimization-explanation/1",
            "plan_revision": self.revision,
            "profile": self.level.name(),
            "target": self.target_triple,
            "cpu": self.cpu,
            "features": self.features,
            "architecture_model": {
                "identity": self.architecture_model,
                "sha256": self.architecture_model_sha256,
            },
            "goals": self.goals,
            "hard_limits": self.limits,
            "enabled_optimizations": self.enabled_optimizations,
            "decisions": self.enabled_optimizations.iter().map(|identity| {
                serde_json::json!({"identity": identity, "decision": "enabled"})
            }).collect::<Vec<_>>(),
            "remarks": self.remarks,
        }))
        .expect("optimization explanation contains only serializable values")
            + "\n"
    }

    /// Publish an explicitly requested decision explanation.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for an explanation path which cannot be written.
    pub fn publish_explanation(
        &self,
        destination: &ExplanationDestination,
    ) -> Result<(), CompileError> {
        let explanation = self.explanation();
        match destination {
            ExplanationDestination::StandardError => eprint!("{explanation}"),
            ExplanationDestination::Path(path) => {
                fs::write(path, explanation).map_err(|error| {
                    CompileError::Io(format!(
                        "cannot write optimization explanation {}: {error}",
                        path.display()
                    ))
                })?
            }
        }
        Ok(())
    }

    /// Enforce hard limits measurable on the published artifact.
    ///
    /// # Errors
    ///
    /// Returns an error when the generated artifact exceeds a selected limit.
    pub fn validate_output_limits(&self, output: &[u8]) -> Result<(), CompileError> {
        for limit in &self.limits {
            let (dimension, quantity) = limit
                .split_once('=')
                .expect("resolved limits were validated");
            if dimension == "code-size" {
                let maximum = parse_code_size(quantity)?;
                if output.len() as u64 > maximum {
                    return Err(CompileError::Tool(format!(
                        "generated artifact is {} bytes and exceeds optimization limit `{limit}`",
                        output.len()
                    )));
                }
            }
        }
        Ok(())
    }
}

fn known_optimization(identity: &str) -> Result<&'static str, CompileError> {
    match identity {
        RUNTIME_GLOBAL_DCE => Ok(RUNTIME_GLOBAL_DCE),
        LLVM_DEFAULT_PIPELINE => Ok(LLVM_DEFAULT_PIPELINE),
        _ => Err(CompileError::Tool(format!(
            "unknown optimization `{identity}`; use --list-optimizations"
        ))),
    }
}

fn validate_goal(goal: &str) -> Result<&str, CompileError> {
    GOALS
        .iter()
        .copied()
        .find(|candidate| *candidate == goal)
        .ok_or_else(|| CompileError::Tool(format!("unknown optimization goal `{goal}`")))
}

fn validate_limit(limit: &str) -> Result<(), CompileError> {
    let (dimension, quantity) = limit.split_once('=').ok_or_else(|| {
        CompileError::Tool(format!(
            "invalid optimization limit `{limit}`; expected DIMENSION=QUANTITY"
        ))
    })?;
    validate_goal(dimension)?;
    if quantity.is_empty() {
        return Err(CompileError::Tool(format!(
            "optimization limit `{limit}` requires a quantity with an explicit unit"
        )));
    }
    if dimension != "code-size" {
        return Err(CompileError::Tool(format!(
            "optimization limit dimension `{dimension}` is not yet measurable by this compiler"
        )));
    }
    parse_code_size(quantity)?;
    Ok(())
}

fn parse_code_size(quantity: &str) -> Result<u64, CompileError> {
    let (number, multiplier) = if let Some(number) = quantity.strip_suffix("KiB") {
        (number, 1024_u64)
    } else if let Some(number) = quantity.strip_suffix("MiB") {
        (number, 1024_u64 * 1024)
    } else if let Some(number) = quantity.strip_suffix('B') {
        (number, 1)
    } else {
        return Err(CompileError::Tool(format!(
            "code-size quantity `{quantity}` requires B, KiB, or MiB"
        )));
    };
    number
        .parse::<u64>()
        .ok()
        .and_then(|value| value.checked_mul(multiplier))
        .ok_or_else(|| CompileError::Tool(format!("invalid code-size quantity `{quantity}`")))
}

const fn default_goals(level: OptimizationLevel) -> &'static [&'static str] {
    match level {
        OptimizationLevel::O0 => &["compile-time"],
        OptimizationLevel::O1 => &["compile-time", "latency", "code-size"],
        OptimizationLevel::O2 => &["latency", "throughput", "code-size", "compile-time"],
        OptimizationLevel::O3 => &["latency", "throughput"],
        OptimizationLevel::Os => &["code-size", "latency"],
        OptimizationLevel::Oz => &["code-size"],
    }
}

#[must_use]
pub fn optimization_listing() -> &'static str {
    "topal.runtime-global-dce/1\n  Remove unreachable compiler-private runtime definitions.\n  Status: implemented; default profile: O1; evidence: qualified static linkage roots.\nllvm.default-pipeline/22\n  Run the LLVM 22 default pipeline selected by O2, O3, Os, or Oz.\n  Status: implemented; default profiles: O2 O3 Os Oz; evidence: verified LLVM IR and target model.\n"
}

#[must_use]
pub fn target_listing() -> String {
    let mut listing = format!("Registry: {TARGET_REGISTRY_REVISION}\n");
    for entry in TARGET_QUALIFICATIONS {
        let target = entry.target.unwrap_or("unassigned");
        let board = entry.board.unwrap_or("none");
        listing.push_str(&format!(
            "{}\n  Target: {target}; CPU: {}; features: {}; board: {board}\n  Model: {}; source: {}\n  Status: {}; missing: {}.\n",
            entry.profile,
            entry.cpu,
            entry.features,
            entry.model,
            entry.source,
            entry.status.name(),
            entry.missing,
        ));
    }
    listing
}

fn unqualified_target_error(entry: &TargetQualification) -> CompileError {
    CompileError::Tool(format!(
        "target profile `{}` is {} and cannot produce code; missing {} (model `{}`, registry `{TARGET_REGISTRY_REVISION}`; use --list-targets)",
        entry.profile,
        entry.status.name(),
        entry.missing,
        entry.model,
    ))
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
                "../../../architecture-models/generic-x86-64-linux.t"
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
    fn rejects_unqualified_target_cpu_board_and_model() {
        let unsupported = [
            TargetSelection {
                target: Some("aarch64-unknown-linux-gnu".into()),
                ..TargetSelection::default()
            },
            TargetSelection {
                cpu: Some("x86-64-avx2".into()),
                ..TargetSelection::default()
            },
            TargetSelection {
                board: Some("example-riscv-dsp-board".into()),
                ..TargetSelection::default()
            },
            TargetSelection {
                model: Some("architecture-models/example-riscv-dsp-board.t".into()),
                ..TargetSelection::default()
            },
        ];
        for target in unsupported {
            assert!(OptimizationPlan::resolve(&target, &OptimizationRequest::default()).is_err());
        }
    }

    #[test]
    fn target_registry_distinguishes_executable_and_model_only_profiles() {
        let listing = target_listing();
        assert!(listing.starts_with("Registry: topal.target-qualification/1\n"));
        assert!(listing.contains("generic-x86-64-linux"));
        assert!(listing.contains("Status: executable-qualified"));
        assert!(listing.contains("example-x86-64-avx2"));
        assert!(listing.contains("example-riscv-dsp-board"));
        assert_eq!(listing.matches("Status: model-only").count(), 2);
        assert_eq!(listing, target_listing());
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
                ..OptimizationRequest::default()
            },
        )
        .unwrap();
        assert_eq!(plan.enabled_optimizations, [RUNTIME_GLOBAL_DCE]);
    }

    #[test]
    fn profiles_controls_and_explanations_are_deterministic() {
        let plan = OptimizationPlan::resolve(
            &TargetSelection {
                target: Some(TARGET_TRIPLE.into()),
                ..TargetSelection::default()
            },
            &OptimizationRequest {
                level: OptimizationLevel::O3,
                explicit_level: true,
                goals: vec!["throughput".into(), "code-size".into()],
                limits: vec!["code-size=64KiB".into()],
                overrides: vec![
                    OptimizationOverride::Disable(LLVM_DEFAULT_PIPELINE.into()),
                    OptimizationOverride::Enable(RUNTIME_GLOBAL_DCE.into()),
                ],
                ..OptimizationRequest::default()
            },
        )
        .unwrap();
        assert_eq!(plan.enabled_optimizations, [RUNTIME_GLOBAL_DCE]);
        assert_eq!(plan.goals, ["throughput", "code-size"]);
        assert_eq!(plan.explanation(), plan.explanation());
        assert!(plan.explanation().contains("64KiB"));
    }

    #[test]
    fn rejects_unknown_or_incompatible_controls() {
        for request in [
            OptimizationRequest {
                goals: vec!["guess".into()],
                ..OptimizationRequest::default()
            },
            OptimizationRequest {
                limits: vec!["code-size".into()],
                ..OptimizationRequest::default()
            },
            OptimizationRequest {
                overrides: vec![OptimizationOverride::Enable("unknown".into())],
                ..OptimizationRequest::default()
            },
            OptimizationRequest {
                level: OptimizationLevel::O2,
                explicit_level: true,
                only: Some(RUNTIME_GLOBAL_DCE.into()),
                ..OptimizationRequest::default()
            },
        ] {
            assert!(OptimizationPlan::resolve(&TargetSelection::default(), &request).is_err());
        }
    }
}
