//! Checked ordinary numeric source and conservative temporary-allocation bounds.
#[path = "systems_numeric_object.rs"]
mod native_object;
use crate::{CompileError, SystemsSourceLibraryInput};
pub use native_object::SystemsNumericLibraryArtifact;
pub(crate) use native_object::{
    build_numeric_library, inspect_linked_numeric_library, inspect_numeric_provider,
};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use topal_language::compiler::{
    CompilerBinary as Op, CompilerBlock, CompilerExpression, CompilerExpressionKind as K,
    CompilerFunction, CompilerProgram, CompilerStatement, CompilerSystemsSourceLibraryInterface,
    CompilerSystemsSourceLibraryRequirement, CompilerType,
};
use topal_source::canonically_equal;

pub(crate) const FUNCTION_PREFIX: &str = "topal.numeric.";
pub(crate) const EXPORT_SYMBOL: &str = "topal_kernel_sleep_deadline_policy_v1";
// The cost certificate is invalidated by ANY change to these shared algorithms.
// Bounds include signed adjustment, zero paths and allocator alignment, rather
// than relying on measurements of the selected deadline implementation.
const CERTIFIED_RUNTIME_SHA256: &str =
    "a1c809e3b73bcd168457ca394a0dd4f1613abb68086dc0b97f1e7c168d415de6";
pub(crate) const ARENA_BYTES: u64 = 65536;
pub(crate) const HEADER_BYTES: u64 = 32;

fn invalid(message: impl Into<String>) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-NUMERIC-LIBRARY: {}", message.into()))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemsNumericAllocationBound {
    pub certificate: String,
    pub runtime_sha256: String,
    pub maximum_operand_bits: u64,
    pub source_evaluation_bytes: u64,
    pub bridge_bytes: u64,
    pub header_bytes: u64,
    pub maximum_bytes: u64,
    pub limit_bytes: u64,
}

pub(crate) struct CheckedNumericLibrary {
    pub(crate) program: CompilerProgram,
    pub(crate) source_name: String,
    pub(crate) entry_symbol: String,
    pub(crate) allocation: SystemsNumericAllocationBound,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Range(BigInt, BigInt);
impl Range {
    fn bits(&self) -> u64 {
        self.0.bits().max(self.1.bits()).max(1)
    }
    fn maximum_absolute(&self) -> BigInt {
        BigInt::from(self.0.magnitude().clone()).max(BigInt::from(self.1.magnitude().clone()))
    }
    fn union(&self, other: &Self) -> Self {
        Self(
            self.0.clone().min(other.0.clone()),
            self.1.clone().max(other.1.clone()),
        )
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Value {
    Boolean,
    Number(Range),
    Pair(Box<Value>, Box<Value>),
}
impl Value {
    fn number(&self) -> Result<&Range, CompileError> {
        if let Self::Number(range) = self {
            Ok(range)
        } else {
            Err(invalid("expected numeric bound"))
        }
    }
    fn union(&self, other: &Self) -> Result<Self, CompileError> {
        match (self, other) {
            (Self::Boolean, Self::Boolean) => Ok(Self::Boolean),
            (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a.union(b))),
            (Self::Pair(a, b), Self::Pair(c, d)) => {
                Ok(Self::Pair(Box::new(a.union(c)?), Box::new(b.union(d)?)))
            }
            _ => Err(invalid("incompatible branch bounds")),
        }
    }
}
#[derive(Clone)]
struct Evaluation {
    value: Value,
    bytes: u64,
}
struct Bounds<'a> {
    functions: BTreeMap<String, &'a CompilerFunction>,
    cache: BTreeMap<(String, Vec<Value>), Evaluation>,
    active: BTreeSet<String>,
    maximum_bits: u64,
}
fn cost_add(a: u64, b: u64) -> Result<u64, CompileError> {
    let sum = a
        .checked_add(b)
        .ok_or_else(|| invalid("allocation bound overflow"))?;
    if sum > ARENA_BYTES {
        return Err(invalid(
            "source/runtime allocation exceeds declared reserve",
        ));
    }
    Ok(sum)
}
fn numeric_type(ty: &CompilerType) -> bool {
    matches!(
        ty,
        CompilerType::Boolean | CompilerType::Nat | CompilerType::Int
    ) || matches!(ty,CompilerType::Tuple(fields) if fields.as_slice()==[CompilerType::Int,CompilerType::Int])
}
impl Bounds<'_> {
    fn function(
        &mut self,
        symbol: &str,
        arguments: Vec<Value>,
    ) -> Result<Evaluation, CompileError> {
        let key = (symbol.to_owned(), arguments.clone());
        if let Some(result) = self.cache.get(&key) {
            return Ok(result.clone());
        }
        if !self.active.insert(symbol.into()) {
            return Err(invalid("recursive source closure"));
        }
        let function = *self
            .functions
            .get(symbol)
            .ok_or_else(|| invalid("unresolved source helper"))?;
        if function.foreign.is_some()
            || !function.result_captures.is_empty()
            || !function.pattern_identities.is_empty()
            || function
                .declared_effects
                .as_ref()
                .is_some_and(|row| !row.identities.is_empty())
            || !numeric_type(&function.result_type)
            || function.parameters.len() != arguments.len()
            || function
                .parameters
                .iter()
                .any(|p| !numeric_type(&p.value_type))
        {
            return Err(invalid("unqualified function interface/effect/capture"));
        }
        let mut environment = function
            .parameters
            .iter()
            .zip(arguments)
            .map(|(p, v)| (p.name.clone(), v))
            .collect();
        let result = self.block(&function.body, &mut environment)?;
        self.active.remove(symbol);
        self.cache.insert(key, result.clone());
        Ok(result)
    }
    fn block(
        &mut self,
        block: &CompilerBlock,
        env: &mut BTreeMap<String, Value>,
    ) -> Result<Evaluation, CompileError> {
        let mut bytes = 0;
        for statement in &block.statements {
            match statement {
                CompilerStatement::Binding(binding) => {
                    let value = self.expression(&binding.value, env)?;
                    bytes = cost_add(bytes, value.bytes)?;
                    env.insert(binding.name.clone(), value.value);
                }
                CompilerStatement::Discard(expression) => {
                    bytes = cost_add(bytes, self.expression(expression, env)?.bytes)?;
                }
            }
        }
        let mut result = self.expression(&block.result, env)?;
        result.bytes = cost_add(bytes, result.bytes)?;
        Ok(result)
    }
    fn expression(
        &mut self,
        expr: &CompilerExpression,
        env: &BTreeMap<String, Value>,
    ) -> Result<Evaluation, CompileError> {
        if !numeric_type(&expr.value_type) {
            return Err(invalid("non-numeric source value"));
        }
        let (value, bytes) = match &expr.kind {
            K::Boolean(_) => (Value::Boolean, 0),
            K::Int(n) => (Value::Number(Range(n.clone(), n.clone())), 0),
            K::IntToNat(input) => {
                let value = self.expression(input, env)?;
                if value.value.number()?.0 < BigInt::from(0) {
                    return Err(invalid("unproven Nat conversion"));
                }
                (value.value, value.bytes)
            }
            K::Local(name) => (
                env.get(name)
                    .ok_or_else(|| invalid(format!("unbound {name}")))?
                    .clone(),
                0,
            ),
            K::Block(block) => {
                return self.block(block, &mut env.clone());
            }
            K::Call { symbol, arguments } => {
                let mut values = Vec::new();
                let mut bytes = 0;
                for arg in arguments {
                    let arg = self.expression(arg, env)?;
                    bytes = cost_add(bytes, arg.bytes)?;
                    values.push(arg.value);
                }
                let result = self.function(symbol, values)?;
                (result.value, cost_add(bytes, result.bytes)?)
            }
            K::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => {
                let subject = self.expression(subject, env)?;
                let a = self.expression(when_true, env)?;
                let b = self.expression(when_false, env)?;
                (
                    a.value.union(&b.value)?,
                    cost_add(subject.bytes, a.bytes.max(b.bytes))?,
                )
            }
            K::TupleField { tuple, index } => {
                let result = self.expression(tuple, env)?;
                let Value::Pair(a, b) = result.value else {
                    return Err(invalid("unqualified pair projection"));
                };
                (
                    match index {
                        0 => *a,
                        1 => *b,
                        _ => return Err(invalid("pair projection outside bounds")),
                    },
                    result.bytes,
                )
            }
            K::Binary {
                operation,
                left,
                right,
            } => {
                let a = self.expression(left, env)?;
                let b = self.expression(right, env)?;
                let left = a.value.number()?;
                let right = b.value.number()?;
                let bits = left.bits().max(right.bits());
                let mut bytes = cost_add(a.bytes, b.bytes)?;
                match operation {
                    Op::Add | Op::Multiply => {
                        let range = if *operation == Op::Add {
                            Range(&left.0 + &right.0, &left.1 + &right.1)
                        } else {
                            let endpoints = [
                                &left.0 * &right.0,
                                &left.0 * &right.1,
                                &left.1 * &right.0,
                                &left.1 * &right.1,
                            ];
                            Range(
                                endpoints.iter().min().unwrap().clone(),
                                endpoints.iter().max().unwrap().clone(),
                            )
                        };
                        // Includes signed adjustment/copy paths and an extra limb.
                        bytes =
                            cost_add(bytes, self.operation_cost(range.bits().max(bits), false)?)?;
                        (Value::Number(range), bytes)
                    }
                    Op::Modulo | Op::QuotientModulo => {
                        if right.0 <= BigInt::from(0) && right.1 >= BigInt::from(0) {
                            return Err(invalid("unproven nonzero divisor"));
                        }
                        bytes = cost_add(bytes, self.operation_cost(bits, true)?)?;
                        let q = left.maximum_absolute() + BigInt::from(1);
                        let quotient = Value::Number(Range(-q.clone(), q));
                        let remainder = Value::Number(Range(
                            BigInt::from(0),
                            right.maximum_absolute() - BigInt::from(1),
                        ));
                        (
                            if *operation == Op::Modulo {
                                remainder
                            } else {
                                Value::Pair(Box::new(quotient), Box::new(remainder))
                            },
                            bytes,
                        )
                    }
                    Op::Equal
                    | Op::NotEqual
                    | Op::Less
                    | Op::LessEqual
                    | Op::Greater
                    | Op::GreaterEqual => (Value::Boolean, bytes),
                    _ => return Err(invalid("unqualified numeric operation")),
                }
            }
            other => {
                return Err(invalid(format!(
                    "unqualified numeric source expression {other:?}"
                )));
            }
        };
        if let Value::Number(range) = &value {
            self.maximum_bits = self.maximum_bits.max(range.bits());
            if range.bits() > 256 {
                return Err(invalid("operand bit bound exceeds numeric certificate"));
            }
        }
        Ok(Evaluation { value, bytes })
    }
    fn operation_cost(&mut self, bits: u64, division: bool) -> Result<u64, CompileError> {
        if bits > 256 {
            return Err(invalid("operation bit bound exceeds numeric certificate"));
        }
        self.maximum_bits = self.maximum_bits.max(bits);
        let allocation = (16 + 4 * (bits.div_ceil(32) + 2)).div_ceil(8) * 8;
        // Reviewed shared runtime: each division bit uses at most five integer
        // allocations (quotient/remainder doubling, appended bit, subtraction,
        // quotient increment). Twenty-four covers setup, sign adjustment and
        // result pairs. Repeated IR evaluations are counted independently.
        // Eight covers all nondivision paths.
        let count = if division { 5 * (bits + 1) + 24 } else { 8 };
        count
            .checked_mul(allocation)
            .ok_or_else(|| invalid("operation allocation overflow"))
    }
}

pub(crate) fn check_numeric_library(
    requirement: &CompilerSystemsSourceLibraryRequirement,
    input: &SystemsSourceLibraryInput,
) -> Result<CheckedNumericLibrary, CompileError> {
    requirement.validate().map_err(|e| invalid(e.to_string()))?;
    if requirement.interface != CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1
        || requirement.source != input.source
        || !topal_language::modules::declared_libraries(&input.text).is_empty()
        || !topal_language::published_function_names(&input.text)
            .iter()
            .any(|n| canonically_equal(n, &requirement.export))
    {
        return Err(invalid(
            "missing/mismatched/private export or unqualified imports",
        ));
    }
    let runtime = include_str!("runtime/value_core.ll");
    let runtime_sha256 = format!("{:x}", Sha256::digest(runtime.as_bytes()));
    if runtime_sha256 != CERTIFIED_RUNTIME_SHA256 {
        return Err(invalid(
            "shared runtime allocation certificate requires review",
        ));
    }
    let client = format!(
        "{}\nseconds : Nat is 0\nnanoseconds : Nat is 1\nticks : Nat is 0\n{} (seconds, nanoseconds, ticks, false)\n",
        input.text, requirement.export
    );
    let program = topal_language::analyze_for_compiler_with_declared_exports(
        &client,
        &[requirement.export.clone()],
    )
    .map_err(|e| CompileError::Tool(e.render(&input.source_name)))?;
    for statement in &program.main.statements {
        let span = match statement {
            CompilerStatement::Binding(b) => b.value.span,
            CompilerStatement::Discard(e) => e.span,
        };
        if span.start < input.text.len() {
            return Err(invalid("runtime source initializers are unqualified"));
        }
    }
    let K::Call { symbol, .. } = &program.main.result.kind else {
        return Err(invalid("export did not select an ordinary call"));
    };
    let entry = program
        .functions
        .iter()
        .find(|f| &f.symbol == symbol)
        .ok_or_else(|| invalid("missing checked export"))?;
    if entry
        .parameters
        .iter()
        .map(|p| p.value_type.clone())
        .collect::<Vec<_>>()
        != [
            CompilerType::Nat,
            CompilerType::Nat,
            CompilerType::Nat,
            CompilerType::Boolean,
        ]
        || entry.result_type != CompilerType::Int
        || entry.parameters.iter().any(|p| p.int_range.is_some())
    {
        return Err(invalid(
            "numeric interface is not declared-type Nat/Nat/Nat/Boolean -> Int",
        ));
    }
    let entry_symbol = symbol.clone();
    let maximum = BigInt::from(i64::MAX);
    let mut bounds = Bounds {
        functions: program
            .functions
            .iter()
            .map(|f| (f.symbol.clone(), f))
            .collect(),
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
        maximum_bits: 0,
    };
    let result = bounds.function(
        &entry_symbol,
        vec![
            Value::Number(Range(BigInt::from(0), maximum)),
            Value::Number(Range(BigInt::from(0), BigInt::from(999_999_999_u64))),
            Value::Number(Range(BigInt::from(0), BigInt::from(u64::MAX))),
            Value::Boolean,
        ],
    )?;
    let bridge_bytes = 3 * 24; // three normal unsigned native-word input objects
    let maximum_bytes = cost_add(cost_add(result.bytes, bridge_bytes)?, HEADER_BYTES)?;
    let allocation = SystemsNumericAllocationBound {
        certificate: "topal.numeric-allocation/1".into(),
        runtime_sha256,
        maximum_operand_bits: bounds.maximum_bits,
        source_evaluation_bytes: result.bytes,
        bridge_bytes,
        header_bytes: HEADER_BYTES,
        maximum_bytes,
        limit_bytes: ARENA_BYTES,
    };
    Ok(CheckedNumericLibrary {
        program,
        source_name: input.source_name.clone(),
        entry_symbol,
        allocation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_profile_rejects_missing_authority_and_unbounded_operations() {
        // TOPAL-SYSTEMS-NUMERIC-RUNTIME-001, TOPAL-SYSTEMS-SLEEP-DEADLINE-LIBRARY-001
        let requirement = CompilerSystemsSourceLibraryRequirement {
            source: "policy/relative-sleep-deadline.t".into(),
            export: "sleep-deadline".into(),
            interface: CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
        };
        let text = include_str!("../../../linux-kernel/kernel/policy/relative-sleep-deadline.t");
        let input = |text: String| SystemsSourceLibraryInput {
            source: requirement.source.clone(),
            source_name: requirement.source.clone(),
            text,
        };
        let cases = [
            (
                "private",
                text.replace("pub sleep-deadline", "sleep-deadline"),
            ),
            (
                "missing",
                text.replace("pub sleep-deadline", "pub another-export"),
            ),
            ("initializer", format!("{text}\nstate : Nat is 7\n")),
            (
                "wrong signature",
                text.replace("ticks : Nat", "ticks : Int"),
            ),
            ("unproven divisor", text.replace("/% ten", "/% ticks")),
            (
                "unsupported subtraction",
                text.replace("target + nine", "target - nine"),
            ),
            (
                "cycle",
                text.replace(
                    "false then ordinary-deadline (seconds, nanoseconds, ticks)",
                    "false then sleep-deadline (seconds, nanoseconds, ticks, wrapped)",
                ),
            ),
        ];
        for (name, changed) in cases {
            assert_ne!(changed, text, "negative fixture did not change: {name}");
            assert!(
                check_numeric_library(&requirement, &input(changed)).is_err(),
                "accepted {name}"
            );
        }
        let mut mismatched = input(text.into());
        mismatched.source = "other.t".into();
        assert!(check_numeric_library(&requirement, &mismatched).is_err());
        for (stack, arena) in [(4095, Some(65536)), (4096, None), (4096, Some(65535))] {
            let mut changed = requirement.clone();
            changed.stack_limit_bytes = stack;
            changed.runtime_limit_bytes = arena;
            assert!(check_numeric_library(&changed, &input(text.into())).is_err());
        }
    }
    #[test]
    fn bounds_shared_numeric_deadline_from_full_interface_ranges() {
        // TOPAL-SYSTEMS-NUMERIC-RUNTIME-001, TOPAL-SYSTEMS-SLEEP-DEADLINE-LIBRARY-001
        let requirement = CompilerSystemsSourceLibraryRequirement {
            source: "policy/relative-sleep-deadline.t".into(),
            export: "sleep-deadline".into(),
            interface: CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
        };
        let source = SystemsSourceLibraryInput {
            source: requirement.source.clone(),
            source_name: requirement.source.clone(),
            text: include_str!("../../../linux-kernel/kernel/policy/relative-sleep-deadline.t")
                .into(),
        };
        let library = check_numeric_library(&requirement, &source).unwrap();
        eprintln!("numeric allocation bound: {:?}", library.allocation);
        let llvm = crate::codegen::emit_numeric_kernel_library(&library).unwrap();
        assert!(!llvm.contains("asm sideeffect"));
        assert!(!llvm.contains("@_start"));
        assert!(llvm.contains("call fastcc ptr"));
        assert!(library.allocation.maximum_bytes <= 65536);
        assert!(library.allocation.maximum_operand_bits >= 64);
        assert!(library.program.functions.len() > 3);
        let mut excessive = source.clone();
        excessive.text = excessive.text.replace(
            "ticks * ten + duration",
            "ticks * ticks * ticks * ticks * ticks + duration",
        );
        assert!(check_numeric_library(&requirement, &excessive).is_err());
    }
}

#[cfg(test)]
#[path = "systems_numeric_tests.rs"]
mod native_tests;
