//! Canonical, validated Generic Export Intermediate Representation (GEIR).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use sha2::{Digest, Sha256};
use topal_semantics::LanguageVersion;
use topal_source::is_nfc;

pub const ARTIFACT_REVISION: u64 = 1;

pub const COMPILER_ONLY_ERROR_CODE: &str = "E-COMPILER-ONLY";

/// Reproducible identity of a checked source package before compiler lowering.
/// Paths are canonical package-relative names; source and dependency ordering
/// do not affect the resulting digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourcePackageKey {
    pub language_revision: LanguageVersion,
    pub unicode_revision: String,
    pub artifact_revision: u64,
    pub digest: [u8; 32],
}

impl SourcePackageKey {
    #[must_use]
    pub fn derive(
        language_revision: LanguageVersion,
        unicode_revision: impl Into<String>,
        sources: &BTreeMap<String, String>,
        dependencies: &BTreeMap<String, [u8; 32]>,
    ) -> Self {
        let unicode_revision = unicode_revision.into();
        let mut hasher = Sha256::new();
        hash_field(&mut hasher, &language_revision.to_string());
        hash_field(&mut hasher, &unicode_revision);
        hasher.update(ARTIFACT_REVISION.to_be_bytes());
        for (path, source) in sources {
            hash_field(&mut hasher, path);
            hash_field(&mut hasher, source);
        }
        for (identity, digest) in dependencies {
            hash_field(&mut hasher, identity);
            hasher.update(digest);
        }
        Self {
            language_revision,
            unicode_revision,
            artifact_revision: ARTIFACT_REVISION,
            digest: hasher.finalize().into(),
        }
    }
}

fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_be_bytes());
    hasher.update(value.as_bytes());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolRole {
    Compiler,
    Interpreter,
    Debugger,
    LanguageServer,
    Linter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerOnlyOperation {
    ExportGenericArtifact,
    EmitObjectCode,
    OptimizeArtifact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryError {
    pub code: &'static str,
    pub operation: CompilerOnlyOperation,
    pub tool: ToolRole,
    pub message: &'static str,
}

/// Enforce that artifact production and compiler lowering never acquire an
/// accidental runtime meaning in another source tool.
///
/// # Errors
///
/// Returns the same stable diagnostic for every non-compiler caller.
pub const fn require_compiler(
    tool: ToolRole,
    operation: CompilerOnlyOperation,
) -> Result<(), BoundaryError> {
    if matches!(tool, ToolRole::Compiler) {
        Ok(())
    } else {
        Err(BoundaryError {
            code: COMPILER_ONLY_ERROR_CODE,
            operation,
            tool,
            message: "this static artifact operation is available only to the compiler",
        })
    }
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactLimits {
    pub table_entries: usize,
    pub blocks: usize,
    pub instructions: usize,
    pub bytes: usize,
}

impl Default for ArtifactLimits {
    fn default() -> Self {
        Self {
            table_entries: 1_000_000,
            blocks: 1_000_000,
            instructions: 10_000_000,
            bytes: 64 * 1_024 * 1_024,
        }
    }
}

/// Produce the stable anonymous structural identity required by
/// `TOPAL-GIR-ID-001`.
#[must_use]
pub fn structural_identity(canonical_definition: &[u8]) -> [u8; 32] {
    Sha256::digest(canonical_definition).into()
}

/// Decode, validate, and confirm the canonical representation of one GEIR
/// module before exposing it to a consumer.
///
/// # Errors
///
/// Rejects malformed, noncanonical, unsupported, or semantically invalid
/// artifacts as a whole.
pub fn decode_canonical(bytes: &[u8], limits: ArtifactLimits) -> Result<Module, ArtifactError> {
    if bytes.len() > limits.bytes {
        return fail(
            ValidationStage::Framing,
            "artifact exceeds configured byte limit",
        );
    }
    let mut reader = ArtifactReader {
        bytes,
        offset: 0,
        limits,
    };
    if reader.take(9)? != b"TOPALGEIR" {
        return fail(ValidationStage::Framing, "invalid artifact magic");
    }
    let revision = reader.uvarint()?;
    let language = LanguageVersion {
        major: reader.uvarint()?,
        minor: reader.uvarint()?,
        patch: reader.uvarint()?,
        build: reader.uvarint()?,
    };
    let imports = reader.identities()?;
    let identities = reader.identities()?;
    let types = reader.types()?;
    let evidence = reader.evidence()?;
    let capabilities = reader.ids()?;
    let functions = reader.functions()?;
    let exports = reader.ids()?;
    if reader.offset != bytes.len() {
        return fail(ValidationStage::Framing, "artifact has trailing bytes");
    }
    let module = Module {
        revision,
        language,
        imports,
        identities,
        types,
        capabilities,
        evidence,
        functions,
        exports,
    };
    let validated = module.validate()?;
    if validated.canonical_bytes() != bytes {
        return fail(
            ValidationStage::Framing,
            "artifact encoding is not canonical",
        );
    }
    Ok(module)
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

    #[allow(clippy::too_many_lines)] // Control-flow obligations remain in normative validation order.
    fn validate_ssa(&self) -> Result<(), ArtifactError> {
        for function in &self.functions {
            if function.entry >= function.blocks.len() {
                return fail(ValidationStage::Ssa, "entry block is out of bounds");
            }
            let mut predecessors = vec![0_usize; function.blocks.len()];
            if !function.blocks[function.entry]
                .parameters
                .starts_with(&function.inputs)
            {
                return fail(
                    ValidationStage::Ssa,
                    "entry block parameters do not begin with function inputs",
                );
            }
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
                let value_types = block_value_types(block);
                match &block.terminator {
                    Terminator::Return(value) if value_types[*value] != function.result => {
                        return fail(ValidationStage::Ssa, "return value has the wrong type");
                    }
                    Terminator::Branch { target, arguments } => {
                        let expected = &function.blocks[*target].parameters;
                        let actual = arguments
                            .iter()
                            .map(|value| value_types[*value])
                            .collect::<Vec<_>>();
                        if &actual != expected {
                            return fail(
                                ValidationStage::Ssa,
                                "branch arguments do not match target block parameters",
                            );
                        }
                    }
                    Terminator::Match { targets, .. }
                        if targets
                            .iter()
                            .any(|target| !function.blocks[*target].parameters.is_empty()) =>
                    {
                        return fail(
                            ValidationStage::Ssa,
                            "match target requires parameters absent from the terminator",
                        );
                    }
                    Terminator::Yield { resume, .. } | Terminator::Suspend { resume }
                        if function.blocks[*resume].parameters.len() > 1 =>
                    {
                        return fail(
                            ValidationStage::Ssa,
                            "resumption target accepts more than one protocol value",
                        );
                    }
                    Terminator::TailApply {
                        function: target,
                        arguments,
                    } => {
                        let Some(target) = self.functions.get(*target) else {
                            return fail(
                                ValidationStage::Ssa,
                                "tail application target is out of bounds",
                            );
                        };
                        let actual = arguments
                            .iter()
                            .map(|value| value_types[*value])
                            .collect::<Vec<_>>();
                        if actual != target.inputs || target.result != function.result {
                            return fail(
                                ValidationStage::Ssa,
                                "tail application signature does not match",
                            );
                        }
                    }
                    _ => {}
                }
            }
            if predecessors[function.entry] != 0 {
                return fail(ValidationStage::Ssa, "entry block has a predecessor");
            }
            if predecessors
                .iter()
                .enumerate()
                .any(|(index, count)| index != function.entry && *count == 0)
            {
                return fail(ValidationStage::Ssa, "non-entry block is unreachable");
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
                self.validate_instruction_types(block)?;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)] // Type derivation stays exhaustive over the stable opcode registry.
    fn validate_instruction_types(&self, block: &Block) -> Result<(), ArtifactError> {
        let mut values = block.parameters.clone();
        for instruction in &block.instructions {
            match instruction {
                Instruction::Product {
                    result_type,
                    values: operands,
                }
                | Instruction::Construct {
                    result_type,
                    values: operands,
                } => {
                    let expected = match &self.types[*result_type] {
                        Type::Tuple(types) => types.clone(),
                        Type::Record(fields) | Type::Variant(fields) => {
                            fields.iter().map(|(_, id)| *id).collect()
                        }
                        _ => {
                            return fail(
                                ValidationStage::Semantics,
                                "product instruction result is not a product type",
                            );
                        }
                    };
                    let actual = operands.iter().map(|id| values[*id]).collect::<Vec<_>>();
                    if actual != expected {
                        return fail(
                            ValidationStage::Semantics,
                            "product instruction operands have the wrong types",
                        );
                    }
                }
                Instruction::Project {
                    result_type,
                    value,
                    field,
                } => {
                    let projected = match &self.types[values[*value]] {
                        Type::Tuple(types) => types.get(*field).copied(),
                        Type::Record(fields) | Type::Variant(fields) => {
                            fields.get(*field).map(|(_, id)| *id)
                        }
                        _ => None,
                    };
                    if projected != Some(*result_type) {
                        return fail(
                            ValidationStage::Semantics,
                            "projection does not derive its declared result type",
                        );
                    }
                }
                Instruction::Apply {
                    result_type,
                    function,
                    arguments,
                    effects,
                } => {
                    let Some(target) = self.functions.get(*function) else {
                        return fail(
                            ValidationStage::Semantics,
                            "application target is out of bounds",
                        );
                    };
                    let actual = arguments.iter().map(|id| values[*id]).collect::<Vec<_>>();
                    if actual != target.inputs
                        || *result_type != target.result
                        || effects != &target.effects
                    {
                        return fail(
                            ValidationStage::Semantics,
                            "application signature or effects do not match",
                        );
                    }
                }
                Instruction::Validate {
                    result_type, value, ..
                }
                | Instruction::PackExists {
                    result_type, value, ..
                }
                | Instruction::UnpackExists { result_type, value }
                | Instruction::Convert {
                    result_type, value, ..
                } if *result_type == values[*value] => {}
                Instruction::Effect { effect, .. }
                    if self
                        .identities
                        .get(*effect)
                        .is_none_or(|identity| identity.declaration_path.is_empty()) =>
                {
                    return fail(
                        ValidationStage::Semantics,
                        "effect instruction has no exact effect identity",
                    );
                }
                _ => {}
            }
            if let Some(result) = instruction.result_type() {
                values.push(result);
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

struct ArtifactReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    limits: ArtifactLimits,
}

impl<'a> ArtifactReader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], ArtifactError> {
        let end = self.offset.checked_add(length).ok_or(ArtifactError {
            stage: ValidationStage::Framing,
            message: "artifact length overflows",
        })?;
        let value = self.bytes.get(self.offset..end).ok_or(ArtifactError {
            stage: ValidationStage::Framing,
            message: "artifact ends prematurely",
        })?;
        self.offset = end;
        Ok(value)
    }

    fn byte(&mut self) -> Result<u8, ArtifactError> {
        Ok(self.take(1)?[0])
    }

    fn uvarint(&mut self) -> Result<u64, ArtifactError> {
        let mut value = 0_u64;
        for index in 0..10 {
            let byte = self.byte()?;
            if index == 9 && byte > 1 {
                return fail(ValidationStage::Framing, "artifact varint overflows");
            }
            value |= u64::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                if index > 0 && byte == 0 {
                    return fail(ValidationStage::Framing, "artifact varint is nonminimal");
                }
                return Ok(value);
            }
        }
        fail(ValidationStage::Framing, "artifact varint is unterminated")
    }

    fn count(&mut self, maximum: usize) -> Result<usize, ArtifactError> {
        let count = usize::try_from(self.uvarint()?).map_err(|_| ArtifactError {
            stage: ValidationStage::Framing,
            message: "artifact count exceeds host limits",
        })?;
        if count > maximum {
            return fail(
                ValidationStage::Framing,
                "artifact count exceeds configured limit",
            );
        }
        Ok(count)
    }

    fn bytes(&mut self) -> Result<Vec<u8>, ArtifactError> {
        let length = self.count(self.limits.bytes)?;
        Ok(self.take(length)?.to_vec())
    }

    fn text(&mut self) -> Result<String, ArtifactError> {
        let bytes = self.bytes()?;
        let text = std::str::from_utf8(&bytes).map_err(|_| ArtifactError {
            stage: ValidationStage::Framing,
            message: "artifact text is not UTF-8",
        })?;
        if !canonical_text(text) {
            return fail(
                ValidationStage::Framing,
                "artifact text is not canonical NFC",
            );
        }
        Ok(text.to_owned())
    }

    fn ids(&mut self) -> Result<Vec<usize>, ArtifactError> {
        let count = self.count(self.limits.table_entries)?;
        (0..count)
            .map(|_| {
                usize::try_from(self.uvarint()?).map_err(|_| ArtifactError {
                    stage: ValidationStage::Framing,
                    message: "artifact index exceeds host limits",
                })
            })
            .collect()
    }

    fn texts(&mut self) -> Result<Vec<String>, ArtifactError> {
        let count = self.count(self.limits.table_entries)?;
        (0..count).map(|_| self.text()).collect()
    }

    fn identities(&mut self) -> Result<Vec<Identity>, ArtifactError> {
        let count = self.count(self.limits.table_entries)?;
        (0..count)
            .map(|_| {
                Ok(Identity {
                    package: self.text()?,
                    module_path: self.texts()?,
                    declaration_path: self.texts()?,
                    language_revision: LanguageVersion {
                        major: self.uvarint()?,
                        minor: self.uvarint()?,
                        patch: self.uvarint()?,
                        build: self.uvarint()?,
                    },
                })
            })
            .collect()
    }

    fn types(&mut self) -> Result<Vec<Type>, ArtifactError> {
        let count = self.count(self.limits.table_entries)?;
        (0..count).map(|_| self.type_value()).collect()
    }

    fn type_value(&mut self) -> Result<Type, ArtifactError> {
        Ok(match self.byte()? {
            0 => Type::Primitive(self.text()?),
            1 => Type::Tuple(self.ids()?),
            2 => Type::Record(self.fields()?),
            3 => Type::Variant(self.fields()?),
            4 => Type::Union(self.ids()?),
            5 => Type::Constraint {
                base: self.index()?,
                predicate: self.index()?,
            },
            6 => Type::Function {
                inputs: self.ids()?,
                result: self.index()?,
            },
            7 => Type::Existential(self.index()?),
            8 => Type::Nominal(self.index()?),
            9 => Type::Application {
                constructor: self.index()?,
                arguments: self.ids()?,
            },
            10 => Type::RecursiveRef(self.index()?),
            _ => return fail(ValidationStage::TypeFormation, "unknown GEIR type opcode"),
        })
    }

    fn fields(&mut self) -> Result<Vec<(String, usize)>, ArtifactError> {
        let count = self.count(self.limits.table_entries)?;
        (0..count)
            .map(|_| Ok((self.text()?, self.index()?)))
            .collect()
    }

    fn evidence(&mut self) -> Result<Vec<Evidence>, ArtifactError> {
        let count = self.count(self.limits.table_entries)?;
        (0..count)
            .map(|_| {
                let identity = self.index()?;
                let calculus = self.text()?;
                let certificate = self.bytes()?;
                let status = match self.byte()? {
                    0 => EvidenceStatus::Verified,
                    1 => EvidenceStatus::TrustedUnverified,
                    _ => return fail(ValidationStage::Evidence, "unknown evidence status"),
                };
                Ok(Evidence {
                    identity,
                    calculus,
                    certificate,
                    status,
                })
            })
            .collect()
    }

    fn functions(&mut self) -> Result<Vec<Function>, ArtifactError> {
        let count = self.count(self.limits.table_entries)?;
        (0..count).map(|_| self.function()).collect()
    }

    fn function(&mut self) -> Result<Function, ArtifactError> {
        let identity = self.index()?;
        let visibility = match self.byte()? {
            0 => Visibility::Private,
            1 => Visibility::Public,
            _ => return fail(ValidationStage::Framing, "invalid visibility encoding"),
        };
        let static_parameters = self.ids()?;
        let inputs = self.ids()?;
        let result = self.index()?;
        let effects = self.ids()?;
        let guarantees = self.ids()?;
        let block_count = self.count(self.limits.blocks)?;
        let blocks = (0..block_count)
            .map(|_| self.block())
            .collect::<Result<Vec<_>, _>>()?;
        let entry = self.index()?;
        Ok(Function {
            identity,
            visibility,
            static_parameters,
            inputs,
            result,
            effects,
            guarantees,
            blocks,
            entry,
        })
    }

    fn block(&mut self) -> Result<Block, ArtifactError> {
        let parameters = self.ids()?;
        let count = self.count(self.limits.instructions)?;
        let instructions = (0..count)
            .map(|_| self.instruction())
            .collect::<Result<Vec<_>, _>>()?;
        let terminator = self.terminator()?;
        Ok(Block {
            parameters,
            instructions,
            terminator,
        })
    }

    #[allow(clippy::too_many_lines)] // The stable opcode registry is intentionally explicit.
    fn instruction(&mut self) -> Result<Instruction, ArtifactError> {
        Ok(match self.byte()? {
            0 => Instruction::Constant {
                result_type: self.index()?,
                bytes: self.bytes()?,
            },
            1 => Instruction::Product {
                result_type: self.index()?,
                values: self.ids()?,
            },
            2 => Instruction::Project {
                result_type: self.index()?,
                value: self.index()?,
                field: self.index()?,
            },
            3 => Instruction::Apply {
                result_type: self.index()?,
                function: self.index()?,
                arguments: self.ids()?,
                effects: self.ids()?,
            },
            4 => Instruction::Validate {
                result_type: self.index()?,
                value: self.index()?,
                evidence: self.index()?,
            },
            5 => Instruction::BeginRegion,
            6 => Instruction::EndRegion,
            7 => Instruction::Construct {
                result_type: self.index()?,
                values: self.ids()?,
            },
            8 => Instruction::Convert {
                result_type: self.index()?,
                value: self.index()?,
                evidence: self.index()?,
            },
            9 => Instruction::Capability {
                result_type: self.index()?,
                evidence: self.index()?,
            },
            10 => Instruction::Effect {
                result_type: self.index()?,
                effect: self.index()?,
                arguments: self.ids()?,
            },
            11 => Instruction::PackExists {
                result_type: self.index()?,
                value: self.index()?,
                evidence: self.index()?,
            },
            12 => Instruction::UnpackExists {
                result_type: self.index()?,
                value: self.index()?,
            },
            _ => {
                return fail(
                    ValidationStage::Semantics,
                    "unknown GEIR instruction opcode",
                );
            }
        })
    }

    fn terminator(&mut self) -> Result<Terminator, ArtifactError> {
        Ok(match self.byte()? {
            0 => Terminator::Return(self.index()?),
            1 => Terminator::Branch {
                target: self.index()?,
                arguments: self.ids()?,
            },
            2 => Terminator::Match {
                value: self.index()?,
                targets: self.ids()?,
            },
            3 => Terminator::Yield {
                value: self.index()?,
                resume: self.index()?,
            },
            4 => Terminator::Suspend {
                resume: self.index()?,
            },
            5 => Terminator::TailApply {
                function: self.index()?,
                arguments: self.ids()?,
            },
            _ => return fail(ValidationStage::Ssa, "unknown GEIR terminator opcode"),
        })
    }

    fn index(&mut self) -> Result<usize, ArtifactError> {
        usize::try_from(self.uvarint()?).map_err(|_| ArtifactError {
            stage: ValidationStage::Framing,
            message: "artifact index exceeds host limits",
        })
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
fn block_value_types(block: &Block) -> Vec<usize> {
    block
        .parameters
        .iter()
        .copied()
        .chain(
            block
                .instructions
                .iter()
                .filter_map(Instruction::result_type),
        )
        .collect()
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
        assert_eq!(
            first,
            [
                84, 79, 80, 65, 76, 71, 69, 73, 82, 1, 0, 1, 0, 0, 0, 1, 7, 101, 120, 97, 109, 112,
                108, 101, 1, 4, 109, 97, 105, 110, 1, 6, 97, 110, 115, 119, 101, 114, 0, 1, 0, 0,
                1, 0, 3, 73, 110, 116, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0,
            ]
        );
        assert_eq!(
            decode_canonical(&first, ArtifactLimits::default()).unwrap(),
            module
        );
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

    #[test]
    fn compiler_only_boundary_is_stable_for_every_source_tool() {
        for tool in [
            ToolRole::Interpreter,
            ToolRole::Debugger,
            ToolRole::LanguageServer,
            ToolRole::Linter,
        ] {
            let error =
                require_compiler(tool, CompilerOnlyOperation::ExportGenericArtifact).unwrap_err();
            assert_eq!(error.code, COMPILER_ONLY_ERROR_CODE);
        }
        assert_eq!(
            require_compiler(
                ToolRole::Compiler,
                CompilerOnlyOperation::ExportGenericArtifact
            ),
            Ok(())
        );
    }

    #[test]
    fn decoder_rejects_every_truncated_prefix_and_noncanonical_varint() {
        let bytes = module().validate().unwrap().canonical_bytes();
        for end in 0..bytes.len() {
            assert!(decode_canonical(&bytes[..end], ArtifactLimits::default()).is_err());
        }
        let mut nonminimal = b"TOPALGEIR".to_vec();
        nonminimal.extend([0x81, 0]);
        assert_eq!(
            decode_canonical(&nonminimal, ArtifactLimits::default())
                .unwrap_err()
                .stage,
            ValidationStage::Framing
        );
    }

    #[test]
    fn structural_identity_is_normative_sha256() {
        assert_eq!(
            structural_identity(b""),
            [
                0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
                0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
                0x78, 0x52, 0xb8, 0x55,
            ]
        );
    }

    #[test]
    fn source_package_key_covers_every_reproducibility_boundary() {
        let sources = BTreeMap::from([
            ("fundamental/ordering.t".into(), "minimum".into()),
            ("library.t".into(), "ordering".into()),
        ]);
        let dependencies = BTreeMap::from([("example.base".into(), [7; 32])]);
        let key =
            SourcePackageKey::derive(LanguageVersion::DESIGN_0, "17.0.0", &sources, &dependencies);
        assert_eq!(key.artifact_revision, ARTIFACT_REVISION);
        assert_ne!(
            key,
            SourcePackageKey::derive(LanguageVersion::DESIGN_0, "18.0.0", &sources, &dependencies,)
        );
        let mut changed = sources;
        changed.insert("fundamental/ordering.t".into(), "maximum".into());
        assert_ne!(
            key,
            SourcePackageKey::derive(LanguageVersion::DESIGN_0, "17.0.0", &changed, &dependencies,)
        );
    }

    #[test]
    fn validation_rederives_return_and_branch_types() {
        let mut wrong_return = module();
        wrong_return.types.push(Type::Primitive("Boolean".into()));
        wrong_return.functions[0].blocks[0].parameters[0] = 1;
        assert_eq!(
            wrong_return.validate().unwrap_err().stage,
            ValidationStage::Ssa
        );

        let mut wrong_edge = module();
        wrong_edge.functions[0].blocks.push(Block {
            parameters: vec![0, 0],
            instructions: vec![],
            terminator: Terminator::Return(0),
        });
        wrong_edge.functions[0].blocks[0].terminator = Terminator::Branch {
            target: 1,
            arguments: vec![0],
        };
        assert_eq!(
            wrong_edge.validate().unwrap_err().message,
            "branch arguments do not match target block parameters"
        );
    }
}
