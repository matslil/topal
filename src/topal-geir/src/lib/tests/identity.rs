use super::*;

fn identity(name: &str) -> Identity {
    Identity {
        package: "example".into(),
        module_path: vec!["main".into()],
        declaration_path: vec![name.into()],
        language_revision: LanguageVersion::DESIGN_1,
    }
}

fn module() -> Module {
    Module {
        revision: ARTIFACT_REVISION,
        language: LanguageVersion::DESIGN_1,
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
            contracts: Some(FunctionContracts::default()),
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
    assert_eq!(u64::from(first[9]), ARTIFACT_REVISION);
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
fn revision_two_round_trips_contract_and_implementation_evidence_context() {
    let mut module = module();
    module
        .identities
        .extend([identity("compiler"), identity("lock-free")]);
    module.evidence.push(Evidence {
        identity: 2,
        calculus: "topal.progress.v1".into(),
        certificate: vec![1, 2, 3],
        status: EvidenceStatus::Verified,
        context: Some(EvidenceContext {
            kind: EvidenceKind::Implementation,
            subject: 0,
            static_parameters: vec![("capacity".into(), "64".into())],
            producer: 1,
            assumptions: vec![],
            language_revision: LanguageVersion::DESIGN_1,
            architecture_model: None,
        }),
    });
    module.functions[0].guarantees.push(0);
    module.functions[0].contracts = Some(FunctionContracts {
        precondition: Some(0),
        result_binding: Some("result".into()),
        postcondition: Some(0),
    });
    let bytes = module.validate().unwrap().canonical_bytes();
    assert_eq!(
        decode_canonical(&bytes, ArtifactLimits::default()).unwrap(),
        module
    );
}

#[test]
fn legacy_revision_remains_decodable_without_v02_fields() {
    let mut legacy = module();
    legacy.revision = LEGACY_ARTIFACT_REVISION;
    legacy.language = LanguageVersion::DESIGN_0;
    legacy.identities[0].language_revision = LanguageVersion::DESIGN_0;
    legacy.functions[0].contracts = None;
    let bytes = legacy.validate().unwrap().canonical_bytes();
    assert_eq!(
        decode_canonical(&bytes, ArtifactLimits::default()).unwrap(),
        legacy
    );
}

#[test]
fn revision_two_rejects_missing_or_impermissibly_trusted_metadata() {
    let mut missing_contracts = module();
    missing_contracts.functions[0].contracts = None;
    assert_eq!(
        missing_contracts.validate().unwrap_err().message,
        "v0.2 function contract record is missing"
    );

    let mut trusted_implementation = module();
    trusted_implementation.evidence.push(Evidence {
        identity: 0,
        calculus: "topal.progress.v1".into(),
        certificate: vec![],
        status: EvidenceStatus::TrustedUnverified,
        context: Some(EvidenceContext {
            kind: EvidenceKind::Implementation,
            subject: 0,
            static_parameters: vec![],
            producer: 0,
            assumptions: vec![],
            language_revision: LanguageVersion::DESIGN_1,
            architecture_model: None,
        }),
    });
    assert_eq!(
        trusted_implementation.validate().unwrap_err().message,
        "implementation evidence cannot be programmer-trusted"
    );

    trusted_implementation.evidence[0].status = EvidenceStatus::Verified;
    trusted_implementation.evidence[0].context = None;
    assert_eq!(
        trusted_implementation.validate().unwrap_err().message,
        "v0.2 evidence metadata is missing"
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
        ("fundamental/ordering.t".into(), "min".into()),
        ("library.t".into(), "ordering".into()),
    ]);
    let dependencies = BTreeMap::from([("example.base".into(), [7; 32])]);
    let key =
        SourcePackageKey::derive(LanguageVersion::DESIGN_0, "17.0.0", &sources, &dependencies);
    assert_eq!(key.artifact_revision, LEGACY_ARTIFACT_REVISION);
    let v02_key =
        SourcePackageKey::derive(LanguageVersion::DESIGN_1, "17.0.0", &sources, &dependencies);
    assert_eq!(v02_key.artifact_revision, ARTIFACT_REVISION);
    assert_ne!(key, v02_key);
    assert_ne!(
        key,
        SourcePackageKey::derive(LanguageVersion::DESIGN_0, "18.0.0", &sources, &dependencies,)
    );
    let mut changed = sources;
    changed.insert("fundamental/ordering.t".into(), "max".into());
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
