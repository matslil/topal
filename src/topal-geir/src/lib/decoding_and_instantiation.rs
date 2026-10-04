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

    fn evidence(&mut self, revision: u64) -> Result<Vec<Evidence>, ArtifactError> {
        let count = self.count(self.limits.table_entries)?;
        (0..count)
            .map(|_| {
                let identity = self.index()?;
                let calculus = self.text()?;
                let certificate = self.bytes()?;
                let status = match self.byte()? {
                    0 => EvidenceStatus::Verified,
                    1 => EvidenceStatus::TrustedUnverified,
                    2 if revision == ARTIFACT_REVISION => EvidenceStatus::ExternallyAssumed,
                    3 if revision == ARTIFACT_REVISION => EvidenceStatus::Refuted,
                    _ => return fail(ValidationStage::Evidence, "unknown evidence status"),
                };
                let context = (revision == ARTIFACT_REVISION)
                    .then(|| self.evidence_context())
                    .transpose()?;
                Ok(Evidence {
                    identity,
                    calculus,
                    certificate,
                    status,
                    context,
                })
            })
            .collect()
    }

    fn evidence_context(&mut self) -> Result<EvidenceContext, ArtifactError> {
        let kind = match self.byte()? {
            0 => EvidenceKind::Semantic,
            1 => EvidenceKind::Implementation,
            _ => return fail(ValidationStage::Evidence, "unknown evidence kind"),
        };
        let subject = self.index()?;
        let parameter_count = self.count(self.limits.table_entries)?;
        let static_parameters = (0..parameter_count)
            .map(|_| Ok((self.text()?, self.text()?)))
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        let producer = self.index()?;
        let assumptions = self.ids()?;
        let language_revision = LanguageVersion {
            major: self.uvarint()?,
            minor: self.uvarint()?,
            patch: self.uvarint()?,
            build: self.uvarint()?,
        };
        let architecture_model = match self.byte()? {
            0 => None,
            1 => Some(self.index()?),
            _ => {
                return fail(
                    ValidationStage::Evidence,
                    "invalid architecture-model presence tag",
                );
            }
        };
        Ok(EvidenceContext {
            kind,
            subject,
            static_parameters,
            producer,
            assumptions,
            language_revision,
            architecture_model,
        })
    }

    fn functions(&mut self, revision: u64) -> Result<Vec<Function>, ArtifactError> {
        let count = self.count(self.limits.table_entries)?;
        (0..count).map(|_| self.function(revision)).collect()
    }

    fn function(&mut self, revision: u64) -> Result<Function, ArtifactError> {
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
        let contracts = (revision == ARTIFACT_REVISION)
            .then(|| {
                Ok(FunctionContracts {
                    precondition: self.optional_index()?,
                    result_binding: self.optional_text()?,
                    postcondition: self.optional_index()?,
                })
            })
            .transpose()?;
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
            contracts,
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

    fn optional_index(&mut self) -> Result<Option<usize>, ArtifactError> {
        match self.byte()? {
            0 => Ok(None),
            1 => self.index().map(Some),
            _ => fail(ValidationStage::Framing, "invalid optional-index tag"),
        }
    }

    fn optional_text(&mut self) -> Result<Option<String>, ArtifactError> {
        match self.byte()? {
            0 => Ok(None),
            1 => self.text().map(Some),
            _ => fail(ValidationStage::Framing, "invalid optional-text tag"),
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
