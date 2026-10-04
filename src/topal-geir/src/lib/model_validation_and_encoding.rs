pub const LEGACY_ARTIFACT_REVISION: u64 = 1;
pub const ARTIFACT_REVISION: u64 = 2;

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
        let artifact_revision = if language_revision == LanguageVersion::DESIGN_0 {
            LEGACY_ARTIFACT_REVISION
        } else {
            ARTIFACT_REVISION
        };
        let mut hasher = Sha256::new();
        hash_field(&mut hasher, &language_revision.to_string());
        hash_field(&mut hasher, &unicode_revision);
        hasher.update(artifact_revision.to_be_bytes());
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
            artifact_revision,
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
    ExternallyAssumed,
    Refuted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceKind {
    Semantic,
    Implementation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceContext {
    pub kind: EvidenceKind,
    pub subject: usize,
    pub static_parameters: Vec<(String, String)>,
    pub producer: usize,
    pub assumptions: Vec<usize>,
    pub language_revision: LanguageVersion,
    pub architecture_model: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub identity: usize,
    pub calculus: String,
    pub certificate: Vec<u8>,
    pub status: EvidenceStatus,
    pub context: Option<EvidenceContext>,
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
    pub contracts: Option<FunctionContracts>,
    pub blocks: Vec<Block>,
    pub entry: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FunctionContracts {
    pub precondition: Option<usize>,
    pub result_binding: Option<String>,
    pub postcondition: Option<usize>,
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
    let evidence = reader.evidence(revision)?;
    let capabilities = reader.ids()?;
    let functions = reader.functions(revision)?;
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
        if !matches!(self.revision, LEGACY_ARTIFACT_REVISION | ARTIFACT_REVISION) {
            return fail(ValidationStage::Framing, "unsupported artifact revision");
        }
        let language_matches_artifact = match self.revision {
            LEGACY_ARTIFACT_REVISION => self.language == LanguageVersion::DESIGN_0,
            ARTIFACT_REVISION => self.language == LanguageVersion::DESIGN_1,
            _ => false,
        };
        if !language_matches_artifact {
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

    #[allow(clippy::too_many_lines)] // Complete function semantics stay in validation order.
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
            match (self.revision, &function.contracts) {
                (ARTIFACT_REVISION, Some(contracts)) => {
                    if contracts
                        .precondition
                        .into_iter()
                        .chain(contracts.postcondition)
                        .any(|identity| identity >= self.identities.len())
                    {
                        return fail(
                            ValidationStage::Semantics,
                            "function contract identity is out of bounds",
                        );
                    }
                    if contracts.postcondition.is_some() != contracts.result_binding.is_some() {
                        return fail(
                            ValidationStage::Semantics,
                            "a postcondition requires exactly one named result binding",
                        );
                    }
                    if contracts
                        .result_binding
                        .as_deref()
                        .is_some_and(|binding| !canonical_text(binding))
                    {
                        return fail(
                            ValidationStage::Semantics,
                            "function result binding is not canonical",
                        );
                    }
                }
                (LEGACY_ARTIFACT_REVISION, None) => {}
                (ARTIFACT_REVISION, None) => {
                    return fail(
                        ValidationStage::Semantics,
                        "v0.2 function contract record is missing",
                    );
                }
                (LEGACY_ARTIFACT_REVISION, Some(_)) => {
                    return fail(
                        ValidationStage::Semantics,
                        "legacy function contains a v0.2 contract record",
                    );
                }
                _ => unreachable!("artifact revision is validated first"),
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
        for proof in &self.evidence {
            if proof.identity >= self.identities.len() || !canonical_text(&proof.calculus) {
                return fail(ValidationStage::Evidence, "invalid proof evidence");
            }
            match (self.revision, &proof.context) {
                (LEGACY_ARTIFACT_REVISION, None)
                    if matches!(
                        proof.status,
                        EvidenceStatus::Verified | EvidenceStatus::TrustedUnverified
                    ) => {}
                (LEGACY_ARTIFACT_REVISION, _) => {
                    return fail(
                        ValidationStage::Evidence,
                        "legacy evidence contains v0.2 metadata or status",
                    );
                }
                (ARTIFACT_REVISION, Some(context)) => {
                    if context.subject >= self.identities.len()
                        || context.producer >= self.identities.len()
                        || context
                            .architecture_model
                            .is_some_and(|identity| identity >= self.identities.len())
                        || context
                            .assumptions
                            .iter()
                            .any(|identity| *identity >= self.identities.len())
                    {
                        return fail(
                            ValidationStage::Evidence,
                            "v0.2 evidence identity is out of bounds",
                        );
                    }
                    if !strictly_sorted(&context.assumptions)
                        || !context
                            .static_parameters
                            .windows(2)
                            .all(|pair| pair[0].0 < pair[1].0)
                        || context
                            .static_parameters
                            .iter()
                            .any(|(name, value)| !canonical_text(name) || !canonical_text(value))
                    {
                        return fail(
                            ValidationStage::Evidence,
                            "v0.2 evidence metadata is not canonical",
                        );
                    }
                    if context.language_revision != self.language {
                        return fail(
                            ValidationStage::Evidence,
                            "evidence language revision does not match the module",
                        );
                    }
                    if context.kind == EvidenceKind::Implementation
                        && proof.status == EvidenceStatus::TrustedUnverified
                    {
                        return fail(
                            ValidationStage::Evidence,
                            "implementation evidence cannot be programmer-trusted",
                        );
                    }
                }
                (ARTIFACT_REVISION, None) => {
                    return fail(
                        ValidationStage::Evidence,
                        "v0.2 evidence metadata is missing",
                    );
                }
                _ => unreachable!("artifact revision is validated first"),
            }
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
                EvidenceStatus::ExternallyAssumed => 2,
                EvidenceStatus::Refuted => 3,
            });
            if self.0.revision == ARTIFACT_REVISION
                && let Some(context) = &proof.context
            {
                encode_evidence_context(context, &mut output);
            }
        }
        encode_ids(&self.0.capabilities, &mut output);
        encode_u64(self.0.functions.len() as u64, &mut output);
        for function in &self.0.functions {
            encode_function(function, self.0.revision, &mut output);
        }
        encode_ids(&self.0.exports, &mut output);
        output
    }
}

fn encode_evidence_context(context: &EvidenceContext, output: &mut Vec<u8>) {
    output.push(match context.kind {
        EvidenceKind::Semantic => 0,
        EvidenceKind::Implementation => 1,
    });
    encode_u64(context.subject as u64, output);
    encode_u64(context.static_parameters.len() as u64, output);
    for (name, value) in &context.static_parameters {
        encode_text(name, output);
        encode_text(value, output);
    }
    encode_u64(context.producer as u64, output);
    encode_ids(&context.assumptions, output);
    for part in [
        context.language_revision.major,
        context.language_revision.minor,
        context.language_revision.patch,
        context.language_revision.build,
    ] {
        encode_u64(part, output);
    }
    match context.architecture_model {
        Some(identity) => {
            output.push(1);
            encode_u64(identity as u64, output);
        }
        None => output.push(0),
    }
}

fn encode_function(function: &Function, revision: u64, output: &mut Vec<u8>) {
    encode_u64(function.identity as u64, output);
    output.push(u8::from(function.visibility == Visibility::Public));
    encode_ids(&function.static_parameters, output);
    encode_ids(&function.inputs, output);
    encode_u64(function.result as u64, output);
    encode_ids(&function.effects, output);
    encode_ids(&function.guarantees, output);
    if revision == ARTIFACT_REVISION
        && let Some(contracts) = &function.contracts
    {
        encode_optional_id(contracts.precondition, output);
        encode_optional_text(contracts.result_binding.as_deref(), output);
        encode_optional_id(contracts.postcondition, output);
    }
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

fn encode_optional_id(value: Option<usize>, output: &mut Vec<u8>) {
    match value {
        Some(value) => {
            output.push(1);
            encode_u64(value as u64, output);
        }
        None => output.push(0),
    }
}

fn encode_optional_text(value: Option<&str>, output: &mut Vec<u8>) {
    match value {
        Some(value) => {
            output.push(1);
            encode_text(value, output);
        }
        None => output.push(0),
    }
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
