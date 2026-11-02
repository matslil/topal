//! Checked ordinary numeric source and conservative temporary-allocation bounds.
#[cfg(test)]
#[path = "systems_numeric_array_tests.rs"]
mod array_tests;
#[path = "systems_numeric_collections.rs"]
mod collections;
#[path = "systems_dispatch_choices.rs"]
mod dispatch_choices;
#[cfg(test)]
#[path = "systems_dispatch_tests.rs"]
mod dispatch_tests;
#[path = "systems_numeric_enums.rs"]
mod enums;
#[cfg(test)]
#[path = "systems_header_tests.rs"]
mod header_tests;
#[cfg(test)]
#[path = "systems_identity_tests.rs"]
mod identity_tests;
#[path = "systems_numeric_interface.rs"]
mod interface;
#[cfg(test)]
#[path = "systems_retained_identity_tests.rs"]
mod retained_identity_tests;
pub(crate) use interface::{NumericPolicy, namespace_llvm};
#[path = "systems_numeric_object.rs"]
mod native_object;
#[cfg(test)]
#[path = "systems_numeric_record_tests.rs"]
mod record_tests;
use crate::{CompileError, SystemsSourceLibraryInput};
pub use native_object::SystemsNumericLibraryArtifact;
#[path = "systems_region_bounds.rs"]
mod region_bounds;
#[path = "systems_retained_state_bound.rs"]
mod retained_state;
pub(crate) use native_object::object_calls_with_imports as library_object_calls;
pub(crate) use native_object::{
    build_numeric_library, inspect_linked_numeric_library, inspect_numeric_provider,
};
use num_bigint::BigInt;
pub use region_bounds::SystemsRegionAllocationBound;
pub(crate) use region_bounds::bound_region_library;
pub(crate) use retained_state::STATE_SYMBOL;
pub use retained_state::SystemsRetainedStateBound;
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
pub(crate) const ALLOCATION_CERTIFICATE: &str = "topal.numeric-allocation/8";
pub(crate) const ARENA_BYTES: u64 = 65536;
pub(crate) const HEADER_BYTES: u64 = 32;

fn invalid(message: impl Into<String>) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-NUMERIC-LIBRARY: {}", message.into()))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemsNumericAllocationBound {
    pub certificate: String,
    pub runtime_sha256: String,
    pub collection_runtime_sha256: String,
    pub collection_layout_sha256: String,
    pub maximum_operand_bits: u64,
    pub source_evaluation_bytes: u64,
    pub bridge_bytes: u64,
    pub header_bytes: u64,
    pub maximum_bytes: u64,
    pub limit_bytes: u64,
}

pub(crate) fn uses_retained_identity(
    program: &topal_language::compiler::CompilerSystemsProgram,
) -> bool {
    program.process_identity_library.as_ref().is_some_and(|r| {
        matches!(
            r.interface,
            CompilerSystemsSourceLibraryInterface::RetainedProcessIdentityV1
                | CompilerSystemsSourceLibraryInterface::StructuredProcessIdentityV1
        )
    })
}

pub(crate) fn uses_structured_identity(
    program: &topal_language::compiler::CompilerSystemsProgram,
) -> bool {
    program.process_identity_library.as_ref().is_some_and(|r| {
        r.interface == CompilerSystemsSourceLibraryInterface::StructuredProcessIdentityV1
    })
}

pub(crate) struct CheckedNumericLibrary {
    pub(crate) program: CompilerProgram,
    pub(crate) source_name: String,
    pub(crate) entry_symbol: String,
    pub(crate) allocation: SystemsNumericAllocationBound,
    pub(crate) policy: NumericPolicy,
    pub(crate) source_tags: Vec<u32>,
    pub(crate) result_field_order: Vec<String>,
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
    Unit,
    Region,
    Boolean,
    Enum {
        name: String,
        alternatives: Vec<String>,
        tags: BTreeSet<u32>,
    },
    Number(Range),
    Pair(Box<Value>, Box<Value>),
    Record(BTreeMap<String, Value>),
    List(Vec<Value>),
    Array(Vec<Value>),
    Optional {
        some: Option<Box<Value>>,
        none: bool,
    },
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
            (Self::Unit, Self::Unit) => Ok(Self::Unit),
            (Self::Region, Self::Region) => Ok(Self::Region),
            (Self::Boolean, Self::Boolean) => Ok(Self::Boolean),
            (
                Self::Enum {
                    name: a,
                    alternatives: b,
                    tags: c,
                },
                Self::Enum {
                    name: d,
                    alternatives: e,
                    tags: f,
                },
            ) if a == d && b == e => Ok(Self::Enum {
                name: a.clone(),
                alternatives: b.clone(),
                tags: c.union(f).copied().collect(),
            }),
            (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a.union(b))),
            (Self::Pair(a, b), Self::Pair(c, d)) => {
                Ok(Self::Pair(Box::new(a.union(c)?), Box::new(b.union(d)?)))
            }
            (Self::Record(a), Self::Record(b)) if a.keys().eq(b.keys()) => Ok(Self::Record(
                a.iter()
                    .map(|(label, value)| Ok((label.clone(), value.union(&b[label])?)))
                    .collect::<Result<_, CompileError>>()?,
            )),
            (Self::List(a), Self::List(b)) | (Self::Array(a), Self::Array(b))
                if a.len() == b.len() =>
            {
                let entries = a
                    .iter()
                    .zip(b)
                    .map(|(a, b)| a.union(b))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(if matches!(self, Self::Array(_)) {
                    Self::Array(entries)
                } else {
                    Self::List(entries)
                })
            }
            (Self::Optional { some: a, none: b }, Self::Optional { some: c, none: d }) => {
                Ok(Self::Optional {
                    some: match (a, c) {
                        (Some(a), Some(c)) => Some(Box::new(a.union(c)?)),
                        (Some(a), None) | (None, Some(a)) => Some(a.clone()),
                        (None, None) => None,
                    },
                    none: *b || *d,
                })
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
    regions: bool,
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
    matches!(ty, CompilerType::Enum(_)) || plain_numeric_type(ty)
}
fn plain_numeric_type(ty: &CompilerType) -> bool {
    matches!(
        ty,
        CompilerType::Boolean | CompilerType::Nat | CompilerType::Int
    ) || matches!(ty,CompilerType::Tuple(fields) if fields.as_slice()==[CompilerType::Int,CompilerType::Int])
        || matches!(ty,CompilerType::Record(fields) if fields.iter().all(|(_,field)|plain_numeric_type(field)))
        || matches!(ty,CompilerType::List(element) | CompilerType::Optional(element)
            if element.as_ref()==&CompilerType::Int)
        || matches!(ty,CompilerType::Array {count,element}
            if *count<=collections::MAX_ENTRIES && element.as_ref()==&CompilerType::Int)
}
impl Bounds<'_> {
    fn admitted_type(&self, ty: &CompilerType) -> bool {
        numeric_type(ty)
            || (self.regions
                && match ty {
                    CompilerType::Unit | CompilerType::KernelByteRegion => true,
                    CompilerType::Optional(payload) => matches!(
                        payload.as_ref(),
                        CompilerType::Nat | CompilerType::KernelByteRegion
                    ),
                    CompilerType::Record(fields) => {
                        fields.iter().all(|(_, field)| self.admitted_type(field))
                    }
                    _ => false,
                })
    }
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
            || !self.admitted_type(&function.result_type)
            || (function.module_identity.is_some()
                && (matches!(&function.result_type, CompilerType::Enum(_))
                    || function
                        .parameters
                        .iter()
                        .any(|p| matches!(&p.value_type, CompilerType::Enum(_)))))
            || function.parameters.len() != arguments.len()
            || function
                .parameters
                .iter()
                .any(|p| !self.admitted_type(&p.value_type))
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
        if !self.admitted_type(&expr.value_type) {
            return Err(invalid("non-numeric source value"));
        }
        let (value, bytes) = match &expr.kind {
            K::Unit if self.regions => (Value::Unit, 0),
            K::KernelRegion { .. } if self.regions => self.region_expression(expr, env)?,
            K::Not(input) if self.regions => {
                let input = self.expression(input, env)?;
                if !matches!(input.value, Value::Boolean) {
                    return Err(invalid("region Boolean negation type"));
                }
                (Value::Boolean, input.bytes)
            }
            K::Boolean(_) => (Value::Boolean, 0),
            K::Enum(_) | K::EnumDecision { .. } => self.enum_expression(expr, env)?,
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
            K::Record(fields) => {
                let mut values = BTreeMap::new();
                let mut bytes = 0;
                for (label, field) in fields {
                    let field = self.expression(field, env)?;
                    bytes = cost_add(bytes, field.bytes)?;
                    if values.insert(label.clone(), field.value).is_some() {
                        return Err(invalid("duplicate checked Record label"));
                    }
                }
                (Value::Record(values), bytes)
            }
            K::RecordField { record, label } => {
                let receiver = self.expression(record, env)?;
                let Value::Record(mut fields) = receiver.value else {
                    return Err(invalid("Record projection without a bounded receiver"));
                };
                (
                    fields
                        .remove(label)
                        .ok_or_else(|| invalid("missing checked Record field"))?,
                    receiver.bytes,
                )
            }
            K::RecordReconstruct { base, replacements } => {
                let base = self.expression(base, env)?;
                let Value::Record(mut fields) = base.value else {
                    return Err(invalid("Record reconstruction without a bounded base"));
                };
                let mut bytes = base.bytes;
                let mut replaced = BTreeSet::new();
                for (label, expression) in replacements {
                    if !replaced.insert(label) {
                        return Err(invalid("duplicate checked replacement"));
                    }
                    let value = self.expression(expression, env)?;
                    bytes = cost_add(bytes, value.bytes)?;
                    *fields
                        .get_mut(label)
                        .ok_or_else(|| invalid("missing checked replacement field"))? = value.value;
                }
                (Value::Record(fields), bytes)
            }
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
                if let (
                    Value::Enum {
                        name: a_name,
                        alternatives: a_kind,
                        ..
                    },
                    Value::Enum {
                        name: b_name,
                        alternatives: b_kind,
                        ..
                    },
                ) = (&a.value, &b.value)
                {
                    if a_name != b_name
                        || a_kind != b_kind
                        || !matches!(operation, Op::Equal | Op::NotEqual)
                    {
                        return Err(invalid("unqualified or mismatched nominal operation"));
                    }
                    return Ok(Evaluation {
                        value: Value::Boolean,
                        bytes: cost_add(a.bytes, b.bytes)?,
                    });
                }
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
            _ => self.collection_expression(expr, env)?,
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
    let policy = NumericPolicy::from_interface(requirement.interface)?;
    if requirement.source != input.source
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
    let (collection_runtime_sha256, collection_layout_sha256) = collections::runtime_digests()?;
    let client = format!("{}{}", input.text, policy.client(&requirement.export));
    let modules = crate::systems_library::modules::validate(input)?;
    let program = topal_language::analyze_for_compiler_with_modules_and_declared_exports(
        &client,
        &modules,
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
        != policy.parameters()
        || entry.parameters.iter().any(|p| p.int_range.is_some())
    {
        return Err(invalid(match policy {
            NumericPolicy::StructuredIdentity => {
                "structured identity interface requires its exact ordinary nested state signature"
            }
            NumericPolicy::Deadline => {
                "numeric interface is not declared-type Nat/Nat/Nat/Boolean -> Int"
            }
            NumericPolicy::Elf64Header => {
                "header interface is not declared-type Array(64,Int)/Nat/Nat -> Boolean"
            }
            NumericPolicy::ProcessIdentity | NumericPolicy::RetainedIdentity => {
                "identity interface is not declared-type Record(next:Nat) -> Record(allocated:Nat,next:Nat)"
            }
            NumericPolicy::LinuxDispatch
            | NumericPolicy::LinuxDispatchV2
            | NumericPolicy::LinuxDispatchV3
            | NumericPolicy::LinuxDispatchV4 => {
                "dispatch interface is not declared-type Nat -> StaticLinuxOperation"
            }
        }));
    }
    let source_tags = policy.source_tags(&entry.result_type)?;
    let result_field_order = policy.result_field_order(&entry.result_type);
    let entry_symbol = symbol.clone();
    let mut bounds = Bounds {
        functions: program
            .functions
            .iter()
            .map(|f| (f.symbol.clone(), f))
            .collect(),
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
        maximum_bits: 0,
        regions: false,
    };
    let result = bounds.function(&entry_symbol, policy.arguments())?;
    let bridge_bytes = policy.bridge_bytes();
    let maximum_bytes = cost_add(cost_add(result.bytes, bridge_bytes)?, HEADER_BYTES)?;
    let allocation = SystemsNumericAllocationBound {
        certificate: ALLOCATION_CERTIFICATE.into(),
        runtime_sha256,
        collection_runtime_sha256,
        collection_layout_sha256,
        maximum_operand_bits: bounds.maximum_bits,
        source_evaluation_bytes: result.bytes,
        bridge_bytes,
        header_bytes: HEADER_BYTES,
        maximum_bytes,
        limit_bytes: ARENA_BYTES,
    };
    crate::systems_library::modules::check_isolation(input, &program)?;
    Ok(CheckedNumericLibrary {
        program,
        source_name: input.source_name.clone(),
        entry_symbol,
        allocation,
        policy,
        source_tags,
        result_field_order,
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
            state_limit_bytes: None,
        };
        let text = include_str!("../../../linux-kernel/kernel/policy/relative-sleep-deadline.t");
        let input = |text: String| SystemsSourceLibraryInput {
            source: requirement.source.clone(),
            source_name: requirement.source.clone(),
            text,
            modules: crate::systems_library::load_source_library(
                &requirement,
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
            )
            .unwrap()
            .modules,
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
            (
                "cycle",
                text.replace(
                    "  support clock sleep-deadline request",
                    "  sleep-deadline (seconds, nanoseconds, ticks, wrapped)",
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
        for (from, to) in [("/% ten", "/% target"), ("target + nine", "target - nine")] {
            let mut changed = input(text.into());
            let original = changed.modules[0].text.clone();
            changed.modules[0].text = original.replace(from, to);
            assert_ne!(original, changed.modules[0].text);
            assert!(check_numeric_library(&requirement, &changed).is_err());
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
            state_limit_bytes: None,
        };
        let source = SystemsSourceLibraryInput {
            source: requirement.source.clone(),
            source_name: requirement.source.clone(),
            text: include_str!("../../../linux-kernel/kernel/policy/relative-sleep-deadline.t")
                .into(),
            modules: crate::systems_library::load_source_library(
                &requirement,
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
            )
            .unwrap()
            .modules,
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
        excessive.modules[0].text = excessive.modules[0].text.replace(
            "ticks * ten + duration",
            "ticks * ticks * ticks * ticks * ticks + duration",
        );
        assert!(check_numeric_library(&requirement, &excessive).is_err());
    }
}

#[cfg(test)]
#[path = "systems_numeric_tests.rs"]
mod native_tests;
