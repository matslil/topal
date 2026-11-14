//! Total fresh logical identities per source step, including call multiplicity.
use super::*;

#[derive(Clone, Default)]
pub(super) struct Calls {
    pub(super) frames: BigInt,
    pub(super) views: BigInt,
    pub(super) unsigned: bool,
}
impl Calls {
    fn add(mut self, other: Self) -> Self {
        self.frames += other.frames;
        self.views += other.views;
        self.unsigned |= other.unsigned;
        self
    }
    fn union(mut self, other: Self) -> Self {
        self.frames = self.frames.max(other.frames);
        self.views = self.views.max(other.views);
        self.unsigned |= other.unsigned;
        self
    }
}
struct Counter<'a> {
    entry: &'a str,
    functions: BTreeMap<String, &'a CompilerFunction>,
    cache: BTreeMap<String, Calls>,
    active: BTreeSet<String>,
}
impl Counter<'_> {
    fn function(&mut self, symbol: &str) -> Result<Calls, CompileError> {
        if let Some(calls) = self.cache.get(symbol) {
            return Ok(calls.clone());
        }
        if !self.active.insert(symbol.into()) {
            return Err(fail("recursive identity helper"));
        }
        let function = *self
            .functions
            .get(symbol)
            .ok_or_else(|| fail("missing identity helper"))?;
        let calls = self.block(&function.body)?;
        self.active.remove(symbol);
        self.cache.insert(symbol.into(), calls.clone());
        Ok(calls)
    }
    fn block(&mut self, block: &CompilerBlock) -> Result<Calls, CompileError> {
        let mut calls = Calls::default();
        for statement in &block.statements {
            calls = calls.add(self.expression(match statement {
                CompilerStatement::Binding(binding) => &binding.value,
                CompilerStatement::Discard(expression) => expression,
            })?);
        }
        Ok(calls.add(self.expression(&block.result)?))
    }
    fn expression(&mut self, expression: &CompilerExpression) -> Result<Calls, CompileError> {
        Ok(match &expression.kind {
            K::Unit | K::Boolean(_) | K::Int(_) | K::Local(_) | K::OptionalNone => Calls::default(),
            K::IntToNat(value) | K::Absolute(value) | K::Not(value) | K::OptionalSome(value) => {
                self.expression(value)?
            }
            K::Binary { left, right, .. } => self.expression(left)?.add(self.expression(right)?),
            K::Block(block) => self.block(block)?,
            K::Call { symbol, arguments } => {
                let function = *self
                    .functions
                    .get(symbol)
                    .ok_or_else(|| fail("missing identity callee"))?;
                let mut calls = Calls {
                    frames: 1.into(),
                    views: BigInt::from(u8::from(
                        function
                            .parameters
                            .iter()
                            .any(|p| p.value_type == CompilerType::KernelByteRegion),
                    )),
                    unsigned: false,
                };
                for argument in arguments {
                    calls = calls.add(self.expression(argument)?);
                }
                if symbol != self.entry {
                    calls = calls.add(self.function(symbol)?);
                }
                calls
            }
            K::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => {
                let calls = self.expression(subject)?;
                let branches = self
                    .expression(when_true)?
                    .union(self.expression(when_false)?);
                calls.add(branches)
            }
            K::OptionalDecision {
                subject,
                some_action,
                none_action,
                ..
            } => {
                let calls = self.expression(subject)?;
                let branches = self
                    .expression(some_action)?
                    .union(self.expression(none_action)?);
                calls.add(branches)
            }
            K::OrderedComparisonDecision {
                subject,
                rules,
                otherwise,
            } => {
                let mut calls = self.expression(subject)?;
                let mut branches = self.expression(otherwise)?;
                for rule in rules {
                    calls = calls.add(self.expression(&rule.operand)?);
                    branches = branches.union(self.expression(&rule.action)?);
                }
                calls.add(branches)
            }
            K::KernelRegion {
                operation,
                region,
                arguments,
                ..
            } => {
                let mut calls = self.expression(region)?;
                calls.unsigned |= matches!(
                    operation,
                    RegionOp::UnsignedLoad(_) | RegionOp::UnsignedStore(_)
                );
                for argument in arguments {
                    calls = calls.add(self.expression(argument)?);
                }
                calls
            }
            _ => return Err(fail("unqualified identity expression")),
        })
    }
}

pub(super) fn call_counts(
    program: &CompilerProgram,
    entry: &CompilerFunction,
) -> Result<Calls, CompileError> {
    Counter {
        entry: &entry.symbol,
        functions: program
            .functions
            .iter()
            .map(|f| (f.symbol.clone(), f))
            .collect(),
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
    }
    .block(&entry.body)
}
