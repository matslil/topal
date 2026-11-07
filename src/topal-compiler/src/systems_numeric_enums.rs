//! Finite nominal values retain identity and reachable alternatives, without allocation.
use super::*;

impl Bounds<'_> {
    pub(super) fn enum_expression(
        &mut self,
        expr: &CompilerExpression,
        env: &BTreeMap<String, Value>,
    ) -> Result<(Value, u64), CompileError> {
        match &expr.kind {
            K::Enum(tag) => {
                let CompilerType::Enum(kind) = &expr.value_type else {
                    return Err(invalid("Enum constructor without checked identity"));
                };
                if *tag as usize >= kind.alternatives.len() {
                    return Err(invalid("Enum constructor outside checked alternatives"));
                }
                Ok((
                    Value::Enum {
                        name: kind.name.clone(),
                        alternatives: kind.alternatives.clone(),
                        tags: BTreeSet::from([*tag]),
                    },
                    0,
                ))
            }
            K::EnumDecision {
                subject,
                rules,
                otherwise,
            } => {
                let subject = self.expression(subject, env)?;
                let Value::Enum {
                    alternatives,
                    mut tags,
                    ..
                } = subject.value
                else {
                    return Err(invalid("Enum decision without bounded nominal subject"));
                };
                let mut seen = BTreeSet::new();
                let mut result: Option<Value> = None;
                let mut maximum = 0;
                for rule in rules {
                    if rule.value as usize >= alternatives.len() || !seen.insert(rule.value) {
                        return Err(invalid("invalid or duplicate checked Enum decision case"));
                    }
                    // Validate even an unreachable emitted action. Its work is
                    // not charged to this subject's possible executions.
                    let action = self.expression(&rule.action, env)?;
                    if tags.remove(&rule.value) {
                        maximum = maximum.max(action.bytes);
                        result = Some(match result {
                            Some(old) => old.union(&action.value)?,
                            None => action.value,
                        });
                    }
                }
                if let Some(fallback) = otherwise {
                    let action = self.expression(fallback, env)?;
                    if !tags.is_empty() {
                        maximum = maximum.max(action.bytes);
                        result = Some(match result {
                            Some(old) => old.union(&action.value)?,
                            None => action.value,
                        });
                        tags.clear();
                    }
                }
                if !tags.is_empty() {
                    return Err(invalid("uncovered reachable Enum decision"));
                }
                Ok((
                    result.ok_or_else(|| invalid("empty nominal decision bound"))?,
                    cost_add(subject.bytes, maximum)?,
                ))
            }
            _ => Err(invalid("unqualified nominal Enum expression")),
        }
    }
}
