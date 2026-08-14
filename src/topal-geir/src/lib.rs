//! Canonical, validated Generic Export Intermediate Representation (GEIR).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use topal_semantics::LanguageVersion;
use topal_source::is_nfc;

pub const ARTIFACT_REVISION: u64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Identity {
    pub package: String,
    pub module_path: Vec<String>,
    pub declaration_path: Vec<String>,
    pub language_revision: LanguageVersion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Primitive(String),
    Tuple(Vec<usize>),
    Record(Vec<(String, usize)>),
    Variant(Vec<(String, usize)>),
    Union(Vec<usize>),
    Constraint {
        base: usize,
        predicate: usize,
    },
    Function {
        inputs: Vec<usize>,
        result: usize,
    },
    Existential(usize),
    Nominal(usize),
    Application {
        constructor: usize,
        arguments: Vec<usize>,
    },
    RecursiveRef(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceStatus {
    Verified,
    TrustedUnverified,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub identity: usize,
    pub calculus: String,
    pub certificate: Vec<u8>,
    pub status: EvidenceStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    Constant {
        result_type: usize,
        bytes: Vec<u8>,
    },
    Product {
        result_type: usize,
        values: Vec<usize>,
    },
    Construct {
        result_type: usize,
        values: Vec<usize>,
    },
    Project {
        result_type: usize,
        value: usize,
        field: usize,
    },
    Apply {
        result_type: usize,
        function: usize,
        arguments: Vec<usize>,
        effects: Vec<usize>,
    },
    Validate {
        result_type: usize,
        value: usize,
        evidence: usize,
    },
    Convert {
        result_type: usize,
        value: usize,
        evidence: usize,
    },
    Capability {
        result_type: usize,
        evidence: usize,
    },
    Effect {
        result_type: usize,
        effect: usize,
        arguments: Vec<usize>,
    },
    PackExists {
        result_type: usize,
        value: usize,
        evidence: usize,
    },
    UnpackExists {
        result_type: usize,
        value: usize,
    },
    BeginRegion,
    EndRegion,
}

impl Instruction {
    fn result_type(&self) -> Option<usize> {
        match self {
            Self::Constant { result_type, .. }
            | Self::Product { result_type, .. }
            | Self::Construct { result_type, .. }
            | Self::Project { result_type, .. }
            | Self::Apply { result_type, .. }
            | Self::Validate { result_type, .. }
            | Self::Convert { result_type, .. }
            | Self::Capability { result_type, .. }
            | Self::Effect { result_type, .. }
            | Self::PackExists { result_type, .. }
            | Self::UnpackExists { result_type, .. } => Some(*result_type),
            Self::BeginRegion | Self::EndRegion => None,
        }
    }

    fn values(&self) -> Vec<usize> {
        match self {
            Self::Product { values, .. } | Self::Construct { values, .. } => values.clone(),
            Self::Project { value, .. }
            | Self::Validate { value, .. }
            | Self::Convert { value, .. }
            | Self::PackExists { value, .. }
            | Self::UnpackExists { value, .. } => vec![*value],
            Self::Apply { arguments, .. } | Self::Effect { arguments, .. } => arguments.clone(),
            Self::Constant { .. }
            | Self::Capability { .. }
            | Self::BeginRegion
            | Self::EndRegion => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Terminator {
    Return(usize),
    Branch {
        target: usize,
        arguments: Vec<usize>,
    },
    Match {
        value: usize,
        targets: Vec<usize>,
    },
    Yield {
        value: usize,
        resume: usize,
    },
    Suspend {
        resume: usize,
    },
    TailApply {
        function: usize,
        arguments: Vec<usize>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    pub parameters: Vec<usize>,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    pub identity: usize,
    pub visibility: Visibility,
    pub static_parameters: Vec<usize>,
    pub inputs: Vec<usize>,
    pub result: usize,
    pub effects: Vec<usize>,
    pub guarantees: Vec<usize>,
    pub blocks: Vec<Block>,
    pub entry: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Module {
    pub revision: u64,
    pub language: LanguageVersion,
    pub imports: Vec<Identity>,
    pub identities: Vec<Identity>,
    pub types: Vec<Type>,
    pub capabilities: Vec<usize>,
    pub evidence: Vec<Evidence>,
    pub functions: Vec<Function>,
    pub exports: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationStage {
    Framing,
    Identity,
    TypeFormation,
    Ssa,
    Semantics,
    Evidence,
    Export,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactError {
    pub stage: ValidationStage,
    pub message: &'static str,
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.stage, self.message)
    }
}

impl std::error::Error for ArtifactError {}

impl Module {
    /// Validate the complete module in the normative GEIR stage order.
    ///
    /// # Errors
    ///
    /// Rejects the whole artifact at its first deterministic invalid stage.
    pub fn validate(&self) -> Result<ValidatedModule<'_>, ArtifactError> {
        self.validate_framing()?;
        self.validate_identities()?;
        self.validate_types()?;
        self.validate_ssa()?;
        self.validate_semantics()?;
        self.validate_evidence()?;
        self.validate_exports()?;
        Ok(ValidatedModule(self))
    }

    fn validate_framing(&self) -> Result<(), ArtifactError> {
        if self.revision != ARTIFACT_REVISION {
            return fail(ValidationStage::Framing, "unsupported artifact revision");
        }
        if self.language != LanguageVersion::DESIGN_0 {
            return fail(ValidationStage::Framing, "unsupported language revision");
        }
        Ok(())
    }

    fn validate_identities(&self) -> Result<(), ArtifactError> {
        if !strictly_sorted(&self.imports) || !strictly_sorted(&self.identities) {
            return fail(
                ValidationStage::Identity,
                "identities are not canonical and unique",
            );
        }
        if self.imports.iter().chain(&self.identities).any(|identity| {
            !canonical_text(&identity.package)
                || identity
                    .module_path
                    .iter()
                    .any(|part| !canonical_text(part))
                || identity
                    .declaration_path
                    .iter()
                    .any(|part| !canonical_text(part))
        }) {
            return fail(
                ValidationStage::Identity,
                "identity text is not canonical NFC",
            );
        }
        Ok(())
    }

    fn validate_types(&self) -> Result<(), ArtifactError> {
        for (index, value) in self.types.iter().enumerate() {
            if matches!(value, Type::Primitive(name) if !canonical_text(name)) {
                return fail(
                    ValidationStage::TypeFormation,
                    "type text is not canonical NFC",
                );
            }
            let references = match value {
                Type::Primitive(_) => Vec::new(),
                Type::Tuple(ids) | Type::Union(ids) => ids.clone(),
                Type::Record(fields) | Type::Variant(fields) => {
                    if !unique(fields.iter().map(|(label, _)| label)) {
                        return fail(ValidationStage::TypeFormation, "duplicate type label");
                    }
                    fields.iter().map(|(_, id)| *id).collect()
                }
                Type::Constraint { base, .. } => vec![*base],
                Type::Function { inputs, result } => {
                    let mut ids = inputs.clone();
                    ids.push(*result);
                    ids
                }
                Type::Existential(id) | Type::Nominal(id) => vec![*id],
                Type::Application {
                    constructor,
                    arguments,
                } => {
                    let mut ids = vec![*constructor];
                    ids.extend(arguments);
                    ids
                }
                Type::RecursiveRef(identity) => {
                    if *identity >= self.identities.len() {
                        return fail(ValidationStage::TypeFormation, "unknown recursive identity");
                    }
                    Vec::new()
                }
            };
            if references.iter().any(|reference| *reference >= index) {
                return fail(ValidationStage::TypeFormation, "forward type reference");
            }
        }
        Ok(())
    }

    fn validate_ssa(&self) -> Result<(), ArtifactError> {
        for function in &self.functions {
            if function.entry >= function.blocks.len() {
                return fail(ValidationStage::Ssa, "entry block is out of bounds");
            }
            let mut predecessors = vec![0_usize; function.blocks.len()];
            for block in &function.blocks {
                for target in terminator_targets(&block.terminator) {
                    let Some(count) = predecessors.get_mut(target) else {
                        return fail(ValidationStage::Ssa, "branch target is out of bounds");
                    };
                    *count += 1;
                }
                let mut available = block.parameters.len();
                for instruction in &block.instructions {
                    if instruction.values().iter().any(|value| *value >= available) {
                        return fail(ValidationStage::Ssa, "SSA use is not dominated");
                    }
                    available += usize::from(instruction.result_type().is_some());
                }
                if terminator_values(&block.terminator)
                    .iter()
                    .any(|value| *value >= available)
                {
                    return fail(ValidationStage::Ssa, "terminator SSA use is not dominated");
                }
            }
            if predecessors[function.entry] != 0 {
                return fail(ValidationStage::Ssa, "entry block has a predecessor");
            }
        }
        Ok(())
    }

    fn validate_semantics(&self) -> Result<(), ArtifactError> {
        if self
            .capabilities
            .iter()
            .any(|identity| *identity >= self.identities.len())
        {
            return fail(
                ValidationStage::Semantics,
                "capability identity is out of bounds",
            );
        }
        for function in &self.functions {
            let type_ok = function
                .inputs
                .iter()
                .chain([&function.result])
                .all(|id| *id < self.types.len());
            let identity_ok = function.identity < self.identities.len();
            if !type_ok
                || !identity_ok
                || function
                    .effects
                    .iter()
                    .any(|id| *id >= self.identities.len())
            {
                return fail(
                    ValidationStage::Semantics,
                    "function semantic reference is out of bounds",
                );
            }
            for block in &function.blocks {
                if block.parameters.iter().any(|id| *id >= self.types.len())
                    || block
                        .instructions
                        .iter()
                        .filter_map(Instruction::result_type)
                        .any(|id| id >= self.types.len())
                {
                    return fail(
                        ValidationStage::Semantics,
                        "instruction type is out of bounds",
                    );
                }
                for instruction in &block.instructions {
                    match instruction {
                        Instruction::Apply {
                            function, effects, ..
                        } if *function >= self.functions.len()
                            || effects.iter().any(|id| *id >= self.identities.len()) =>
                        {
                            return fail(
                                ValidationStage::Semantics,
                                "application reference is out of bounds",
                            );
                        }
                        Instruction::Effect { effect, .. } if *effect >= self.identities.len() => {
                            return fail(
                                ValidationStage::Semantics,
                                "effect identity is out of bounds",
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_evidence(&self) -> Result<(), ArtifactError> {
        if self
            .evidence
            .iter()
            .any(|proof| proof.identity >= self.identities.len() || proof.calculus.is_empty())
        {
            return fail(ValidationStage::Evidence, "invalid proof evidence");
        }
        if self
            .functions
            .iter()
            .flat_map(|function| &function.blocks)
            .flat_map(|block| &block.instructions)
            .filter_map(|instruction| match instruction {
                Instruction::Validate { evidence, .. }
                | Instruction::Convert { evidence, .. }
                | Instruction::Capability { evidence, .. }
                | Instruction::PackExists { evidence, .. } => Some(*evidence),
                _ => None,
            })
            .any(|id| id >= self.evidence.len())
        {
            return fail(
                ValidationStage::Evidence,
                "instruction evidence is out of bounds",
            );
        }
        if self
            .functions
            .iter()
            .flat_map(|function| &function.guarantees)
            .any(|id| *id >= self.evidence.len())
        {
            return fail(ValidationStage::Evidence, "unknown guarantee evidence");
        }
        Ok(())
    }

    fn validate_exports(&self) -> Result<(), ArtifactError> {
        let mut seen = BTreeSet::new();
        for export in &self.exports {
            let Some(function) = self.functions.get(*export) else {
                return fail(ValidationStage::Export, "export is out of bounds");
            };
            if function.visibility != Visibility::Public || !seen.insert(*export) {
                return fail(ValidationStage::Export, "export is private or duplicated");
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ValidatedModule<'a>(&'a Module);

impl ValidatedModule<'_> {
    #[must_use]
    pub fn canonical_bytes(self) -> Vec<u8> {
        let mut output = b"TOPALGEIR".to_vec();
        encode_u64(self.0.revision, &mut output);
        for part in [
            self.0.language.major,
            self.0.language.minor,
            self.0.language.patch,
            self.0.language.build,
        ] {
            encode_u64(part, &mut output);
        }
        encode_identities(&self.0.imports, &mut output);
        encode_identities(&self.0.identities, &mut output);
        encode_u64(self.0.types.len() as u64, &mut output);
        for value in &self.0.types {
            encode_type(value, &mut output);
        }
        encode_u64(self.0.evidence.len() as u64, &mut output);
        for proof in &self.0.evidence {
            encode_u64(proof.identity as u64, &mut output);
            encode_text(&proof.calculus, &mut output);
            encode_bytes(&proof.certificate, &mut output);
            output.push(match proof.status {
                EvidenceStatus::Verified => 0,
                EvidenceStatus::TrustedUnverified => 1,
            });
        }
        encode_ids(&self.0.capabilities, &mut output);
        encode_u64(self.0.functions.len() as u64, &mut output);
        for function in &self.0.functions {
            encode_function(function, &mut output);
        }
        encode_ids(&self.0.exports, &mut output);
        output
    }
}

fn encode_function(function: &Function, output: &mut Vec<u8>) {
    encode_u64(function.identity as u64, output);
    output.push(u8::from(function.visibility == Visibility::Public));
    encode_ids(&function.static_parameters, output);
    encode_ids(&function.inputs, output);
    encode_u64(function.result as u64, output);
    encode_ids(&function.effects, output);
    encode_ids(&function.guarantees, output);
    encode_u64(function.blocks.len() as u64, output);
    for block in &function.blocks {
        encode_ids(&block.parameters, output);
        encode_u64(block.instructions.len() as u64, output);
        for instruction in &block.instructions {
            encode_instruction(instruction, output);
        }
        encode_terminator(&block.terminator, output);
    }
    encode_u64(function.entry as u64, output);
}

fn encode_instruction(value: &Instruction, output: &mut Vec<u8>) {
    match value {
        Instruction::Constant { result_type, bytes } => {
            output.push(0);
            encode_u64(*result_type as u64, output);
            encode_bytes(bytes, output);
        }
        Instruction::Product {
            result_type,
            values,
        } => {
            output.push(1);
            encode_u64(*result_type as u64, output);
            encode_ids(values, output);
        }
        Instruction::Construct {
            result_type,
            values,
        } => {
            output.push(7);
            encode_u64(*result_type as u64, output);
            encode_ids(values, output);
        }
        Instruction::Project {
            result_type,
            value,
            field,
        } => {
            output.push(2);
            encode_u64(*result_type as u64, output);
            encode_u64(*value as u64, output);
            encode_u64(*field as u64, output);
        }
        Instruction::Apply {
            result_type,
            function,
            arguments,
            effects,
        } => {
            output.push(3);
            encode_u64(*result_type as u64, output);
            encode_u64(*function as u64, output);
            encode_ids(arguments, output);
            encode_ids(effects, output);
        }
        Instruction::Validate {
            result_type,
            value,
            evidence,
        } => {
            output.push(4);
            encode_u64(*result_type as u64, output);
            encode_u64(*value as u64, output);
            encode_u64(*evidence as u64, output);
        }
        Instruction::Convert {
            result_type,
            value,
            evidence,
        } => {
            output.push(8);
            encode_u64(*result_type as u64, output);
            encode_u64(*value as u64, output);
            encode_u64(*evidence as u64, output);
        }
        Instruction::Capability {
            result_type,
            evidence,
        } => {
            output.push(9);
            encode_u64(*result_type as u64, output);
            encode_u64(*evidence as u64, output);
        }
        Instruction::Effect {
            result_type,
            effect,
            arguments,
        } => {
            output.push(10);
            encode_u64(*result_type as u64, output);
            encode_u64(*effect as u64, output);
            encode_ids(arguments, output);
        }
        Instruction::PackExists {
            result_type,
            value,
            evidence,
        } => {
            output.push(11);
            encode_u64(*result_type as u64, output);
            encode_u64(*value as u64, output);
            encode_u64(*evidence as u64, output);
        }
        Instruction::UnpackExists { result_type, value } => {
            output.push(12);
            encode_u64(*result_type as u64, output);
            encode_u64(*value as u64, output);
        }
        Instruction::BeginRegion => output.push(5),
        Instruction::EndRegion => output.push(6),
    }
}

fn encode_terminator(value: &Terminator, output: &mut Vec<u8>) {
    match value {
        Terminator::Return(id) => {
            output.push(0);
            encode_u64(*id as u64, output);
        }
        Terminator::Branch { target, arguments } => {
            output.push(1);
            encode_u64(*target as u64, output);
            encode_ids(arguments, output);
        }
        Terminator::Match { value, targets } => {
            output.push(2);
            encode_u64(*value as u64, output);
            encode_ids(targets, output);
        }
        Terminator::Yield { value, resume } => {
            output.push(3);
            encode_u64(*value as u64, output);
            encode_u64(*resume as u64, output);
        }
        Terminator::Suspend { resume } => {
            output.push(4);
            encode_u64(*resume as u64, output);
        }
        Terminator::TailApply {
            function,
            arguments,
        } => {
            output.push(5);
            encode_u64(*function as u64, output);
            encode_ids(arguments, output);
        }
    }
}

fn encode_type(value: &Type, output: &mut Vec<u8>) {
    match value {
        Type::Primitive(name) => {
            output.push(0);
            encode_text(name, output);
        }
        Type::Tuple(ids) => {
            output.push(1);
            encode_ids(ids, output);
        }
        Type::Record(fields) => {
            output.push(2);
            encode_fields(fields, output);
        }
        Type::Variant(fields) => {
            output.push(3);
            encode_fields(fields, output);
        }
        Type::Union(ids) => {
            output.push(4);
            encode_ids(ids, output);
        }
        Type::Constraint { base, predicate } => {
            output.push(5);
            encode_u64(*base as u64, output);
            encode_u64(*predicate as u64, output);
        }
        Type::Function { inputs, result } => {
            output.push(6);
            encode_ids(inputs, output);
            encode_u64(*result as u64, output);
        }
        Type::Existential(id) => {
            output.push(7);
            encode_u64(*id as u64, output);
        }
        Type::Nominal(id) => {
            output.push(8);
            encode_u64(*id as u64, output);
        }
        Type::Application {
            constructor,
            arguments,
        } => {
            output.push(9);
            encode_u64(*constructor as u64, output);
            encode_ids(arguments, output);
        }
        Type::RecursiveRef(id) => {
            output.push(10);
            encode_u64(*id as u64, output);
        }
    }
}

fn encode_fields(fields: &[(String, usize)], output: &mut Vec<u8>) {
    encode_u64(fields.len() as u64, output);
    for (label, id) in fields {
        encode_text(label, output);
        encode_u64(*id as u64, output);
    }
}

fn encode_identities(values: &[Identity], output: &mut Vec<u8>) {
    encode_u64(values.len() as u64, output);
    for value in values {
        encode_text(&value.package, output);
        encode_texts(&value.module_path, output);
        encode_texts(&value.declaration_path, output);
        for part in [
            value.language_revision.major,
            value.language_revision.minor,
            value.language_revision.patch,
            value.language_revision.build,
        ] {
            encode_u64(part, output);
        }
    }
}

fn encode_texts(values: &[String], output: &mut Vec<u8>) {
    encode_u64(values.len() as u64, output);
    for value in values {
        encode_text(value, output);
    }
}
fn encode_ids(values: &[usize], output: &mut Vec<u8>) {
    encode_u64(values.len() as u64, output);
    for value in values {
        encode_u64(*value as u64, output);
    }
}
fn encode_text(value: &str, output: &mut Vec<u8>) {
    encode_bytes(value.as_bytes(), output);
}
fn encode_bytes(value: &[u8], output: &mut Vec<u8>) {
    encode_u64(value.len() as u64, output);
    output.extend_from_slice(value);
}
fn encode_u64(mut value: u64, output: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn terminator_targets(value: &Terminator) -> Vec<usize> {
    match value {
        Terminator::Branch { target, .. } => vec![*target],
        Terminator::Match { targets, .. } => targets.clone(),
        Terminator::Yield { resume, .. } | Terminator::Suspend { resume } => vec![*resume],
        Terminator::Return(_) | Terminator::TailApply { .. } => Vec::new(),
    }
}
fn terminator_values(value: &Terminator) -> Vec<usize> {
    match value {
        Terminator::Return(value)
        | Terminator::Match { value, .. }
        | Terminator::Yield { value, .. } => vec![*value],
        Terminator::Branch { arguments, .. } | Terminator::TailApply { arguments, .. } => {
            arguments.clone()
        }
        Terminator::Suspend { .. } => Vec::new(),
    }
}
fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}
fn unique<'a, T: Ord + ?Sized + 'a>(values: impl IntoIterator<Item = &'a T>) -> bool {
    let mut seen = BTreeSet::new();
    values.into_iter().all(|value| seen.insert(value))
}
fn canonical_text(value: &str) -> bool {
    !value.is_empty() && !value.contains('\0') && is_nfc(value)
}
fn fail<T>(stage: ValidationStage, message: &'static str) -> Result<T, ArtifactError> {
    Err(ArtifactError { stage, message })
}

/// Substitute exact type identities while retaining the evidence identities.
///
/// # Errors
///
/// Rejects a missing static argument or evidence obligation.
pub fn instantiate(
    parameters: &[usize],
    arguments: &BTreeMap<usize, usize>,
    obligations: &[usize],
    evidence: &BTreeSet<usize>,
) -> Result<Vec<usize>, ArtifactError> {
    if obligations.iter().any(|id| !evidence.contains(id)) {
        return fail(
            ValidationStage::Evidence,
            "generic evidence obligation is unsatisfied",
        );
    }
    parameters
        .iter()
        .map(|parameter| {
            arguments.get(parameter).copied().ok_or(ArtifactError {
                stage: ValidationStage::Semantics,
                message: "generic static argument is missing",
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(name: &str) -> Identity {
        Identity {
            package: "example".into(),
            module_path: vec!["main".into()],
            declaration_path: vec![name.into()],
            language_revision: LanguageVersion::DESIGN_0,
        }
    }

    fn module() -> Module {
        Module {
            revision: ARTIFACT_REVISION,
            language: LanguageVersion::DESIGN_0,
            imports: vec![],
            identities: vec![identity("answer")],
            types: vec![Type::Primitive("Int".into())],
            capabilities: vec![],
            evidence: vec![],
            functions: vec![Function {
                identity: 0,
                visibility: Visibility::Public,
                static_parameters: vec![],
                inputs: vec![0],
                result: 0,
                effects: vec![],
                guarantees: vec![],
                blocks: vec![Block {
                    parameters: vec![0],
                    instructions: vec![],
                    terminator: Terminator::Return(0),
                }],
                entry: 0,
            }],
            exports: vec![0],
        }
    }

    #[test]
    fn valid_modules_have_stable_idempotent_canonical_bytes() {
        let module = module();
        let first = module.validate().unwrap().canonical_bytes();
        let second = module.validate().unwrap().canonical_bytes();
        assert_eq!(first, second);
        assert!(first.starts_with(b"TOPALGEIR"));
    }

    #[test]
    fn validation_rejects_before_artifact_use() {
        let mut invalid = module();
        invalid.functions[0].blocks[0].terminator = Terminator::Return(1);
        assert_eq!(invalid.validate().unwrap_err().stage, ValidationStage::Ssa);
        invalid = module();
        invalid.functions[0].visibility = Visibility::Private;
        assert_eq!(
            invalid.validate().unwrap_err().stage,
            ValidationStage::Export
        );
    }
}
