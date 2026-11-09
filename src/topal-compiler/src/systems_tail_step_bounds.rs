//! Evaluate a complete source step without following a certified self edge.
use super::*;

type Environment = BTreeMap<String, Value>;
#[derive(Clone, Copy, Default)]
struct Paths {
    terminal: Option<u64>,
    recursive: Option<u64>,
}
impl Paths {
    fn union(self, other: Self) -> Self {
        fn maximum(a: Option<u64>, b: Option<u64>) -> Option<u64> {
            match (a, b) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            }
        }
        Self {
            terminal: maximum(self.terminal, other.terminal),
            recursive: maximum(self.recursive, other.recursive),
        }
    }
    fn charge(self, bytes: u64) -> Result<Self, CompileError> {
        Ok(Self {
            terminal: self
                .terminal
                .map(|value| cost_add(value, bytes))
                .transpose()?,
            recursive: self
                .recursive
                .map(|value| cost_add(value, bytes))
                .transpose()?,
        })
    }
}
pub(super) struct Step {
    pub(super) maximum: u64,
    pub(super) terminal: u64,
    pub(super) recursive: u64,
    pub(super) bits: u64,
}
struct Walker<'a> {
    bounds: Bounds<'a>,
    entry: &'a CompilerFunction,
    plan: &'a SystemsTailTraversalPlan,
    carriers: &'a [Carrier],
    edges: &'a [Edge<'a>],
}
impl Walker<'_> {
    fn prefix(
        &mut self,
        block: &CompilerBlock,
        env: &mut Environment,
    ) -> Result<u64, CompileError> {
        let mut bytes = 0;
        for statement in &block.statements {
            match statement {
                CompilerStatement::Binding(binding) => {
                    let value = self.bounds.expression(&binding.value, env)?;
                    bytes = cost_add(bytes, value.bytes)?;
                    env.insert(binding.storage_name.clone(), value.value);
                }
                CompilerStatement::Discard(expression) => {
                    bytes = cost_add(bytes, self.bounds.expression(expression, env)?.bytes)?;
                }
            }
        }
        Ok(bytes)
    }
    fn edge(
        &mut self,
        expression: &CompilerExpression,
        arguments: &[CompilerExpression],
        env: &Environment,
    ) -> Result<Paths, CompileError> {
        // Borrowed AST identity disambiguates equal spans/expressions. It is
        // local to this invocation and never serialized as source authority.
        let edge = self
            .edges
            .iter()
            .find(|edge| std::ptr::eq(edge.expression, expression))
            .ok_or_else(|| fail("tail edge is absent from reconstructed proof"))?;
        let mut bytes = 0;
        for (index, (argument, update)) in arguments.iter().zip(&edge.updates).enumerate() {
            let value = self.bounds.expression(argument, env)?;
            bytes = cost_add(bytes, value.bytes)?;
            match update {
                Update::Region if matches!(value.value, Value::Region) => {}
                Update::Boolean if matches!(value.value, Value::Boolean) => {}
                Update::Measure => {
                    let current = env
                        .get(&self.entry.parameters[index].name)
                        .ok_or_else(|| fail("missing measure fact"))?
                        .number()?;
                    let expected =
                        Range(&current.0 - &edge.decrement, &current.1 - &edge.decrement);
                    if expected.0 < BigInt::from(0) || !contains(&expected, value.value.number()?) {
                        return Err(fail("successor measure differs from natural descent"));
                    }
                }
                Update::Affine(change) => {
                    let carrier = self
                        .carriers
                        .iter()
                        .find(|c| c.parameter == index)
                        .ok_or_else(|| fail("missing affine carrier"))?;
                    value.value.number()?;
                    if change - &carrier.positive * &edge.decrement > BigInt::from(0)
                        || -change - &carrier.negative * &edge.decrement > BigInt::from(0)
                    {
                        return Err(fail("affine update violates carried potential"));
                    }
                }
                Update::Constant(_) | Update::Reset => {
                    let carrier = self
                        .carriers
                        .iter()
                        .find(|c| c.parameter == index)
                        .ok_or_else(|| fail("missing reset carrier"))?;
                    if !contains(&carrier.baseline, value.value.number()?) {
                        return Err(fail("reset is not proven inside the potential baseline"));
                    }
                }
                _ => return Err(fail("tail argument has inconsistent carried classifier")),
            }
            if self.entry.parameters[index].value_type == CompilerType::Nat
                && value.value.number()?.0 < BigInt::from(0)
            {
                return Err(fail("negative successor Nat"));
            }
        }
        Ok(Paths {
            terminal: None,
            recursive: Some(bytes),
        })
    }
    fn tail(
        &mut self,
        expression: &CompilerExpression,
        env: &Environment,
    ) -> Result<Paths, CompileError> {
        match &expression.kind {
            K::Call { symbol, arguments } if symbol == &self.entry.symbol => {
                self.edge(expression, arguments, env)
            }
            K::Block(block) => {
                let mut local = env.clone();
                let bytes = self.prefix(block, &mut local)?;
                self.tail(&block.result, &local)?.charge(bytes)
            }
            K::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => {
                let tested = self.bounds.expression(subject, env)?;
                if !matches!(tested.value, Value::Boolean) {
                    return Err(fail("non-Boolean tail condition"));
                }
                self.tail(when_true, env)?
                    .union(self.tail(when_false, env)?)
                    .charge(tested.bytes)
            }
            K::OrderedComparisonDecision {
                subject,
                rules,
                otherwise,
            } => {
                let subject = self.bounds.expression(subject, env)?;
                subject.value.number()?;
                let mut bytes = subject.bytes;
                let mut paths = self.tail(otherwise, env)?;
                for rule in rules {
                    let operand = self.bounds.expression(&rule.operand, env)?;
                    operand.value.number()?;
                    bytes = cost_add(bytes, operand.bytes)?;
                    paths = paths.union(self.tail(&rule.action, env)?);
                }
                paths.charge(bytes)
            }
            K::OptionalDecision {
                subject,
                some_binding,
                some_action,
                none_action,
            } => {
                let subject = self.bounds.expression(subject, env)?;
                let Value::Optional { some, none } = subject.value else {
                    return Err(fail("unbounded tail Optional"));
                };
                let mut paths = Paths::default();
                if let Some(value) = some {
                    let mut local = env.clone();
                    if let Some((name, _)) = some_binding {
                        local.insert(name.clone(), *value);
                    }
                    paths = self.tail(some_action, &local)?;
                }
                if none {
                    paths = paths.union(self.tail(none_action, env)?);
                }
                paths.charge(subject.bytes)
            }
            _ => Ok(Paths {
                terminal: Some(self.bounds.expression(expression, env)?.bytes),
                recursive: None,
            }),
        }
    }
    fn branch(
        &mut self,
        minimum: BigInt,
        maximum: BigInt,
        action: &CompilerExpression,
        terminal_rules: bool,
    ) -> Result<Paths, CompileError> {
        if minimum > maximum {
            return Ok(Paths::default());
        }
        let mut env = Environment::new();
        for (index, parameter) in self.entry.parameters.iter().enumerate() {
            let value = match parameter.value_type {
                CompilerType::KernelByteRegion => Value::Region,
                CompilerType::Boolean => Value::Boolean,
                _ if index == self.plan.measure_parameter => {
                    Value::Number(Range(minimum.clone(), maximum.clone()))
                }
                _ => Value::Number(
                    self.carriers
                        .iter()
                        .find(|c| c.parameter == index)
                        .ok_or_else(|| fail("missing carried domain"))?
                        .at_measure(&minimum),
                ),
            };
            env.insert(parameter.name.clone(), value);
        }
        let prefix = self.prefix(&self.entry.body, &mut env)?;
        let K::OrderedComparisonDecision { subject, rules, .. } = &self.entry.body.result.kind
        else {
            return Err(fail("missing source guard"));
        };
        let subject = self.bounds.expression(subject, &env)?;
        subject.value.number()?;
        let operand = self.bounds.expression(&rules[0].operand, &env)?;
        operand.value.number()?;
        let mut paths = self.tail(action, &env)?;
        let mut tested = cost_add(subject.bytes, operand.bytes)?;
        if terminal_rules {
            for rule in &rules[1..] {
                let operand = self.bounds.expression(&rule.operand, &env)?;
                operand.value.number()?;
                tested = cost_add(tested, operand.bytes)?;
                paths = paths.union(self.tail(&rule.action, &env)?);
            }
        }
        paths.charge(cost_add(prefix, tested)?)
    }
}

pub(super) fn evaluate<'a>(
    program: &'a CompilerProgram,
    entry: &'a CompilerFunction,
    plan: &'a SystemsTailTraversalPlan,
    carriers: &'a [Carrier],
    edges: &'a [Edge<'a>],
) -> Result<Step, CompileError> {
    let mut walker = Walker {
        bounds: Bounds {
            functions: program
                .functions
                .iter()
                .map(|f| (f.symbol.clone(), f))
                .collect(),
            cache: BTreeMap::new(),
            active: BTreeSet::new(),
            maximum_bits: 0,
            regions: true,
            tail: true,
            tail_visits: 0,
        },
        entry,
        plan,
        carriers,
        edges,
    };
    let K::OrderedComparisonDecision {
        rules, otherwise, ..
    } = &entry.body.result.kind
    else {
        return Err(fail("missing guard"));
    };
    let bound = plan
        .base_bound
        .parse::<BigInt>()
        .map_err(|_| fail("invalid guard bound"))?;
    let terminal = walker.branch(
        0.into(),
        bound.clone().min(word_maximum()),
        &rules[0].action,
        false,
    )?;
    let recursive = walker.branch(bound + 1, word_maximum(), otherwise, true)?;
    let paths = terminal.union(recursive);
    let terminal = paths.terminal.unwrap_or(0);
    let recursive = paths.recursive.unwrap_or(0);
    let bits = carriers
        .iter()
        .map(|c| c.global.bits())
        .max()
        .unwrap_or(1)
        .max(walker.bounds.maximum_bits);
    Ok(Step {
        maximum: terminal.max(recursive),
        terminal,
        recursive,
        bits,
    })
}
