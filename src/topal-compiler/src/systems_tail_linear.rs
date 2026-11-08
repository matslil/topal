//! Exact linear relations through ordinary immutable bindings and helpers.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Linear {
    pub(super) coefficients: BTreeMap<usize, BigInt>,
    pub(super) constant: BigInt,
}
impl Linear {
    fn constant(value: BigInt) -> Self {
        Self {
            coefficients: BTreeMap::new(),
            constant: value,
        }
    }
    fn add(mut self, other: Self, subtract: bool) -> Self {
        let sign = if subtract {
            BigInt::from(-1)
        } else {
            BigInt::from(1)
        };
        self.constant += &sign * other.constant;
        for (parameter, coefficient) in other.coefficients {
            let value = self.coefficients.entry(parameter).or_default();
            *value += &sign * coefficient;
        }
        self.coefficients
            .retain(|_, value| value != &BigInt::from(0));
        self
    }
    fn scale(mut self, factor: &BigInt) -> Self {
        self.constant *= factor;
        for coefficient in self.coefficients.values_mut() {
            *coefficient *= factor;
        }
        self.coefficients
            .retain(|_, value| value != &BigInt::from(0));
        self
    }
}
type Environment = BTreeMap<String, Option<Linear>>;

struct Fold<'a> {
    functions: BTreeMap<String, &'a CompilerFunction>,
    entry: &'a str,
    visits: usize,
}
impl Fold<'_> {
    fn tick(&mut self) -> Result<(), CompileError> {
        self.visits += 1;
        if self.visits > 65536 {
            return Err(fail("symbolic proof budget exceeded"));
        }
        Ok(())
    }
    fn binding(
        &mut self,
        statement: &CompilerStatement,
        env: &mut Environment,
    ) -> Result<(), CompileError> {
        if let CompilerStatement::Binding(binding) = statement {
            let value = self.expression(&binding.value, env)?;
            env.insert(binding.storage_name.clone(), value);
        }
        Ok(())
    }
    fn block(
        &mut self,
        block: &CompilerBlock,
        env: &Environment,
    ) -> Result<Option<Linear>, CompileError> {
        let mut env = env.clone();
        for statement in &block.statements {
            self.binding(statement, &mut env)?;
        }
        self.expression(&block.result, &env)
    }
    fn expression(
        &mut self,
        expression: &CompilerExpression,
        env: &Environment,
    ) -> Result<Option<Linear>, CompileError> {
        self.tick()?;
        let result = match &expression.kind {
            K::Int(value) => Some(Linear::constant(value.clone())),
            K::Local(name) => env.get(name).cloned().flatten(),
            K::IntToNat(value) => self.expression(value, env)?,
            K::Absolute(value) => self.expression(value, env)?.and_then(|value| {
                value
                    .coefficients
                    .is_empty()
                    .then(|| Linear::constant(BigInt::from(value.constant.magnitude().clone())))
            }),
            K::Binary {
                operation,
                left,
                right,
            } => {
                let a = self.expression(left, env)?;
                let b = self.expression(right, env)?;
                match (operation, a, b) {
                    (Op::Add | Op::Subtract, Some(a), Some(b)) => {
                        Some(a.add(b, *operation == Op::Subtract))
                    }
                    (Op::Multiply, Some(a), Some(b)) if a.coefficients.is_empty() => {
                        Some(b.scale(&a.constant))
                    }
                    (Op::Multiply, Some(a), Some(b)) if b.coefficients.is_empty() => {
                        Some(a.scale(&b.constant))
                    }
                    _ => None,
                }
            }
            K::Block(block) => self.block(block, env)?,
            K::Call { symbol, arguments } if symbol != self.entry => {
                let function = self
                    .functions
                    .get(symbol)
                    .copied()
                    .ok_or_else(|| fail("unresolved symbolic helper"))?;
                let mut local = Environment::new();
                for (parameter, argument) in function.parameters.iter().zip(arguments) {
                    local.insert(parameter.name.clone(), self.expression(argument, env)?);
                }
                self.block(&function.body, &local)?
            }
            K::BooleanDecision {
                when_true,
                when_false,
                ..
            } => {
                let a = self.expression(when_true, env)?;
                let b = self.expression(when_false, env)?;
                if a == b { a } else { None }
            }
            K::OrderedComparisonDecision {
                rules, otherwise, ..
            } => {
                let mut result = self.expression(otherwise, env)?;
                for rule in rules {
                    if self.expression(&rule.action, env)? != result {
                        result = None;
                    }
                }
                result
            }
            K::OptionalDecision {
                some_binding,
                some_action,
                none_action,
                ..
            } => {
                let mut local = env.clone();
                if let Some((name, _)) = some_binding {
                    local.insert(name.clone(), None);
                }
                let a = self.expression(some_action, &local)?;
                let b = self.expression(none_action, env)?;
                if a == b { a } else { None }
            }
            _ => None,
        };
        if result.as_ref().is_some_and(|value| {
            value.constant.bits() > 256
                || value
                    .coefficients
                    .values()
                    .any(|coefficient| coefficient.bits() > 256)
        }) {
            return Err(fail("symbolic numeric proof ceiling exceeded"));
        }
        Ok(result)
    }
    fn tails<'a>(
        &mut self,
        expression: &'a CompilerExpression,
        env: &Environment,
        edges: &mut Vec<(&'a CompilerExpression, Vec<Option<Linear>>)>,
    ) -> Result<(), CompileError> {
        self.tick()?;
        match &expression.kind {
            K::Call { symbol, arguments } if symbol == self.entry => {
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expression(argument, env)?);
                }
                edges.push((expression, values));
            }
            K::Block(block) => {
                let mut local = env.clone();
                for statement in &block.statements {
                    self.binding(statement, &mut local)?;
                }
                self.tails(&block.result, &local, edges)?;
            }
            K::BooleanDecision {
                when_true,
                when_false,
                ..
            } => {
                self.tails(when_true, env, edges)?;
                self.tails(when_false, env, edges)?;
            }
            K::OrderedComparisonDecision {
                rules, otherwise, ..
            } => {
                for rule in rules {
                    self.tails(&rule.action, env, edges)?;
                }
                self.tails(otherwise, env, edges)?;
            }
            K::OptionalDecision {
                some_binding,
                some_action,
                none_action,
                ..
            } => {
                let mut local = env.clone();
                if let Some((name, _)) = some_binding {
                    local.insert(name.clone(), None);
                }
                self.tails(some_action, &local, edges)?;
                self.tails(none_action, env, edges)?;
            }
            _ => {}
        }
        Ok(())
    }
}

pub(super) fn edges<'a>(
    program: &CompilerProgram,
    entry: &'a CompilerFunction,
) -> Result<Vec<(&'a CompilerExpression, Vec<Option<Linear>>)>, CompileError> {
    let mut fold = Fold {
        functions: program
            .functions
            .iter()
            .map(|f| (f.symbol.clone(), f))
            .collect(),
        entry: &entry.symbol,
        visits: 0,
    };
    let mut env = entry
        .parameters
        .iter()
        .enumerate()
        .map(|(index, p)| {
            let value =
                matches!(p.value_type, CompilerType::Nat | CompilerType::Int).then(|| Linear {
                    coefficients: BTreeMap::from([(index, BigInt::from(1))]),
                    constant: BigInt::from(0),
                });
            (p.name.clone(), value)
        })
        .collect::<Environment>();
    for statement in &entry.body.statements {
        fold.binding(statement, &mut env)?;
    }
    let K::OrderedComparisonDecision { otherwise, .. } = &entry.body.result.kind else {
        return Err(fail("missing checked tail guard"));
    };
    let mut result = Vec::new();
    fold.tails(otherwise, &env, &mut result)?;
    Ok(result)
}
