//! Reviewed finite collection costs over the shared ordinary representation.
use super::*;
use topal_language::compiler::CompilerContainerKind;

pub(super) const MAX_ENTRIES: usize = 256;
pub(super) fn runtime_digests() -> Result<(String, String), CompileError> {
    let runtime = format!(
        "{:x}",
        Sha256::digest(include_bytes!("runtime/fundamental_containers.ll"))
    );
    let layout = format!(
        "{:x}",
        Sha256::digest(include_bytes!("runtime/list_int_layout.ll"))
    );
    if runtime != "a740ddaf6b69935187c18cdc010597f575940fa1e59a3f799c362a9fd1d176d3"
        || layout != "73ccaed62678c8170247f02ab6b46632919ba768765cb53c0ca804696cd66561"
    {
        return Err(invalid(
            "shared collection allocation certificate requires review",
        ));
    }
    Ok((runtime, layout))
}

impl Bounds<'_> {
    pub(super) fn collection_expression(
        &mut self,
        expr: &CompilerExpression,
        env: &BTreeMap<String, Value>,
    ) -> Result<(Value, u64), CompileError> {
        match &expr.kind {
            K::ListEmpty => Ok((Value::List(Vec::new()), 0)),
            K::ListEntry { value, remaining } => {
                let value = self.expression(value, env)?;
                value.value.number()?;
                let remaining = self.expression(remaining, env)?;
                let Value::List(mut entries) = remaining.value else {
                    return Err(invalid("unbounded List tail"));
                };
                if entries.len() >= MAX_ENTRIES {
                    return Err(invalid("sequence length exceeds collection certificate"));
                }
                entries.insert(0, value.value);
                Ok((
                    Value::List(entries),
                    cost_add(cost_add(value.bytes, remaining.bytes)?, 16)?,
                ))
            }
            K::ContainerCollect {
                source,
                kind: CompilerContainerKind::Array,
                ..
            } => {
                let source = self.expression(source, env)?;
                let Value::List(entries) = source.value else {
                    return Err(invalid("Array requires a bounded List"));
                };
                let CompilerType::Array { count, .. } = &expr.value_type else {
                    return Err(invalid("missing checked Array extent"));
                };
                if entries.len() != *count || entries.len() > MAX_ENTRIES {
                    return Err(invalid("Array extent does not match bounded source"));
                }
                Ok((Value::Array(entries), cost_add(source.bytes, 16)?))
            }
            K::ArrayAt { array, index } => {
                let array = self.expression(array, env)?;
                let Value::Array(entries) = array.value else {
                    return Err(invalid("checked access requires bounded Array"));
                };
                let some = entries.get(*index).cloned().map(Box::new);
                let none = some.is_none();
                Ok((Value::Optional { some, none }, cost_add(array.bytes, 16)?))
            }
            K::ContainerEntryCount(array) | K::ContainerEmpty(array) => {
                let array = self.expression(array, env)?;
                let Value::Array(entries) = array.value else {
                    return Err(invalid("observation requires bounded Array"));
                };
                if matches!(&expr.kind, K::ContainerEmpty(_)) {
                    Ok((Value::Boolean, array.bytes))
                } else {
                    let length = BigInt::from(entries.len());
                    Ok((
                        Value::Number(Range(length.clone(), length)),
                        cost_add(array.bytes, 24)?,
                    ))
                }
            }
            K::OptionalSome(value) => {
                let value = self.expression(value, env)?;
                if !self.regions || !matches!(value.value, Value::Region) {
                    value.value.number()?;
                }
                Ok((
                    Value::Optional {
                        some: Some(Box::new(value.value)),
                        none: false,
                    },
                    cost_add(value.bytes, 16)?,
                ))
            }
            K::OptionalNone => Ok((
                Value::Optional {
                    some: None,
                    none: true,
                },
                16,
            )),
            K::OptionalDecision {
                subject,
                some_binding,
                some_action,
                none_action,
            } => {
                let subject = self.expression(subject, env)?;
                let Value::Optional { some, none } = subject.value else {
                    return Err(invalid("Optional decision without bounded subject"));
                };
                let mut result: Option<Evaluation> = None;
                if let Some(some) = some {
                    let mut selected = env.clone();
                    if let Some((name, _)) = some_binding {
                        selected.insert(name.clone(), *some);
                    }
                    result = Some(self.expression(some_action, &selected)?);
                }
                if none {
                    let absent = self.expression(none_action, env)?;
                    result = Some(match result {
                        None => absent,
                        Some(present) => Evaluation {
                            value: present.value.union(&absent.value)?,
                            bytes: present.bytes.max(absent.bytes),
                        },
                    });
                }
                let result = result.ok_or_else(|| invalid("empty Optional presence bound"))?;
                Ok((result.value, cost_add(subject.bytes, result.bytes)?))
            }
            other => Err(invalid(format!(
                "unqualified numeric source expression {other:?}"
            ))),
        }
    }
}
