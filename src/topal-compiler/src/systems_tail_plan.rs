//! Source-shape evidence only. No executable profile consumes this plan.
use crate::CompileError;
use num_bigint::BigInt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use topal_language::compiler::{
    CompilerBinary as Op, CompilerBlock, CompilerExpression as Expression,
    CompilerExpressionKind as K, CompilerFunction, CompilerKernelRegionOperation as RegionOp,
    CompilerProgram, CompilerStatement, CompilerType,
};

#[cfg(test)]
#[path = "systems_tail_plan_tests.rs"]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailDependency {
    pub identity: String,
    pub source_name: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailEdge {
    pub source_start: usize,
    pub source_end: usize,
    pub decrement: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailFunction {
    pub symbol: String,
    pub parameter_classifiers: Vec<String>,
    pub result_classifier: String,
    /// Self tail edges are separately recorded; these are ordinary helper calls.
    pub callees: Vec<String>,
    pub reads_owner: bool,
    pub writes_owner: bool,
}

/// Diagnostic evidence from checked IR, not a source-origin or native certificate.
/// Fields are descriptive; editing/serializing them grants no authority.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailTraversalPlan {
    pub schema: String,
    pub qualification: String,
    pub source_sha256: String,
    /// Versioned compiler-internal Debug encoding, never a source/native ABI.
    pub checked_graph_sha256: String,
    pub dependencies: Vec<SystemsTailDependency>,
    pub entry_symbol: String,
    pub measure_parameter: usize,
    pub base_bound: String,
    pub minimum_decrement: String,
    pub back_edges: Vec<SystemsTailEdge>,
    pub functions: Vec<SystemsTailFunction>,
    /// Acyclic source depth within one step; not a recursive/native stack bound.
    pub maximum_one_step_function_depth: usize,
}

fn invalid(reason: &str) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-TAIL-PLAN: {reason}"))
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn scalar(ty: &CompilerType) -> bool {
    matches!(
        ty,
        CompilerType::Nat | CompilerType::Int | CompilerType::Boolean
    )
}
fn widens(from: &CompilerType, to: &CompilerType) -> bool {
    from == to || (from == &CompilerType::Nat && to == &CompilerType::Int)
}
fn value_type(ty: &CompilerType) -> bool {
    scalar(ty)
        || matches!(ty, CompilerType::Unit | CompilerType::KernelByteRegion)
        || matches!(ty, CompilerType::Optional(t) if matches!(t.as_ref(), CompilerType::Nat | CompilerType::Int))
}
fn literal(expression: &Expression) -> Option<&BigInt> {
    match &expression.kind {
        K::Int(number) => Some(number),
        _ => None,
    }
}
fn local(expression: &Expression, name: &str) -> bool {
    matches!(&expression.kind, K::Local(actual) if actual == name)
}

struct Inspection<'a> {
    functions: BTreeMap<String, &'a CompilerFunction>,
    entry: &'a CompilerFunction,
    measure: usize,
    bound: BigInt,
    edges: Vec<SystemsTailEdge>,
    decrements: Vec<BigInt>,
    calls: BTreeMap<String, BTreeSet<String>>,
    effects: BTreeMap<String, (bool, bool)>,
    visits: usize,
}

impl Inspection<'_> {
    fn shadows_parameter(&self, owner: &str, storage: &str) -> bool {
        owner == self.entry.symbol
            && (storage == self.entry.parameters[self.measure].name
                || self
                    .entry
                    .parameters
                    .iter()
                    .any(|p| p.value_type == CompilerType::KernelByteRegion && p.name == storage))
    }

    fn statement(
        &mut self,
        owner: &str,
        statement: &CompilerStatement,
        depth: usize,
    ) -> Result<(), CompileError> {
        let expression = match statement {
            CompilerStatement::Binding(binding) => {
                if self.shadows_parameter(owner, &binding.storage_name) {
                    return Err(invalid("shadowed measure or region parameter storage"));
                }
                &binding.value
            }
            CompilerStatement::Discard(expression) => expression,
        };
        self.expression(owner, expression, false, depth)
    }

    fn back_edge(
        &mut self,
        expression: &Expression,
        args: &[Expression],
    ) -> Result<(), CompileError> {
        let parameter = &self.entry.parameters[self.measure];
        let K::IntToNat(step) = &args[self.measure].kind else {
            return Err(invalid("tail measure lacks checked Nat adaptation"));
        };
        let K::Binary {
            operation: Op::Subtract,
            left,
            right,
        } = &step.kind
        else {
            return Err(invalid("tail measure is not a literal decrement"));
        };
        let decrement = literal(right).ok_or_else(|| invalid("nonliteral tail decrement"))?;
        if !local(left, &parameter.name)
            || decrement <= &BigInt::from(0)
            || decrement > &(&self.bound + 1)
        {
            return Err(invalid("unproved positive, nonnegative Nat descent"));
        }
        for (parameter, argument) in self.entry.parameters.iter().zip(args) {
            if parameter.value_type == CompilerType::KernelByteRegion
                && !local(argument, &parameter.name)
            {
                return Err(invalid("changed borrowed region on tail edge"));
            }
        }
        self.edges.push(SystemsTailEdge {
            source_start: expression.span.start,
            source_end: expression.span.end,
            decrement: decrement.to_string(),
        });
        self.decrements.push(decrement.clone());
        Ok(())
    }

    fn block(
        &mut self,
        owner: &str,
        block: &CompilerBlock,
        tail: bool,
        depth: usize,
    ) -> Result<(), CompileError> {
        for statement in &block.statements {
            self.statement(owner, statement, depth + 1)?;
        }
        self.expression(owner, &block.result, tail, depth + 1)
    }

    fn expression(
        &mut self,
        owner: &str,
        expression: &Expression,
        tail: bool,
        depth: usize,
    ) -> Result<(), CompileError> {
        self.visits += 1;
        if self.visits > 16384 || depth > 128 {
            return Err(invalid("source inspection budget exceeded"));
        }
        if !value_type(&expression.value_type) {
            return Err(invalid("unqualified source value classifier"));
        }
        match &expression.kind {
            K::Unit | K::Boolean(_) | K::Int(_) | K::Local(_) | K::OptionalNone => Ok(()),
            K::IntToNat(input) | K::Absolute(input) | K::Not(input) | K::OptionalSome(input) => {
                self.expression(owner, input, false, depth + 1)
            }
            K::Binary { left, right, .. } => {
                self.expression(owner, left, false, depth + 1)?;
                self.expression(owner, right, false, depth + 1)
            }
            K::Block(block) => self.block(owner, block, tail, depth + 1),
            K::Call { symbol, arguments } => {
                let callee = self
                    .functions
                    .get(symbol)
                    .copied()
                    .ok_or_else(|| invalid("unresolved helper"))?;
                if arguments.len() != callee.parameters.len()
                    || arguments
                        .iter()
                        .zip(&callee.parameters)
                        .any(|(arg, p)| !widens(&arg.value_type, &p.value_type))
                    || !widens(&callee.result_type, &expression.value_type)
                {
                    return Err(invalid(&format!(
                        "changed call arity or classifiers for {symbol}: {:?} -> {:?}, expected {:?} -> {:?}",
                        arguments
                            .iter()
                            .map(|arg| &arg.value_type)
                            .collect::<Vec<_>>(),
                        expression.value_type,
                        callee
                            .parameters
                            .iter()
                            .map(|p| &p.value_type)
                            .collect::<Vec<_>>(),
                        callee.result_type,
                    )));
                }
                // Argument evaluation is never a tail position.
                for argument in arguments {
                    self.expression(owner, argument, false, depth + 1)?;
                }
                if symbol == &self.entry.symbol {
                    if owner != self.entry.symbol || !tail {
                        return Err(invalid(
                            "recursive edge outside guarded direct tail position",
                        ));
                    }
                    self.back_edge(expression, arguments)
                } else {
                    self.calls.get_mut(owner).unwrap().insert(symbol.clone());
                    Ok(())
                }
            }
            K::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => {
                self.expression(owner, subject, false, depth + 1)?;
                self.expression(owner, when_true, tail, depth + 1)?;
                self.expression(owner, when_false, tail, depth + 1)
            }
            K::OrderedComparisonDecision {
                subject,
                rules,
                otherwise,
            } => {
                self.expression(owner, subject, false, depth + 1)?;
                for rule in rules {
                    self.expression(owner, &rule.operand, false, depth + 1)?;
                    self.expression(owner, &rule.action, tail, depth + 1)?;
                }
                self.expression(owner, otherwise, tail, depth + 1)
            }
            K::OptionalDecision {
                subject,
                some_binding,
                some_action,
                none_action,
                ..
            } => {
                if some_binding
                    .as_ref()
                    .is_some_and(|(name, _)| self.shadows_parameter(owner, name))
                {
                    return Err(invalid("shadowed measure or region parameter storage"));
                }
                self.expression(owner, subject, false, depth + 1)?;
                self.expression(owner, some_action, tail, depth + 1)?;
                self.expression(owner, none_action, tail, depth + 1)
            }
            K::KernelRegion {
                operation,
                region,
                arguments,
                ..
            } => {
                if matches!(operation, RegionOp::Window { .. }) {
                    return Err(invalid("view construction is not tail-qualified"));
                }
                self.expression(owner, region, false, depth + 1)?;
                for argument in arguments {
                    self.expression(owner, argument, false, depth + 1)?;
                }
                let effects = self.effects.get_mut(owner).unwrap();
                effects.0 |= matches!(operation, RegionOp::Load | RegionOp::UnsignedLoad(_));
                effects.1 |= matches!(operation, RegionOp::Store | RegionOp::UnsignedStore(_));
                Ok(())
            }
            _ => Err(invalid("unqualified tail-source operation")),
        }
    }
}

/// Inspect structural eligibility for a future systems tail-traversal profile.
///
/// The caller must obtain `program` from the shared frontend. The returned
/// metadata cannot authorize emission and does not attest externally forged IR.
pub fn analyze_systems_tail_traversal(
    program: &CompilerProgram,
    entry_symbol: &str,
) -> Result<SystemsTailTraversalPlan, CompileError> {
    if !program.main.statements.is_empty() {
        return Err(invalid("runtime initializer outside source closure"));
    }
    let functions = program
        .functions
        .iter()
        .map(|f| (f.symbol.clone(), f))
        .collect::<BTreeMap<_, _>>();
    if functions.is_empty() || functions.len() != program.functions.len() || functions.len() > 64 {
        return Err(invalid("function identity budget or duplicate symbol"));
    }
    for function in functions.values() {
        let names = function
            .parameters
            .iter()
            .map(|p| &p.name)
            .collect::<BTreeSet<_>>();
        if function.foreign.is_some()
            || function.is_static
            || !function.result_captures.is_empty()
            || !function.pattern_identities.is_empty()
            || function
                .declared_effects
                .as_ref()
                .is_some_and(|row| !row.identities.is_empty())
            || (!scalar(&function.result_type) && function.result_type != CompilerType::Unit)
            || names.len() != function.parameters.len()
            || function.parameters.iter().any(|p| {
                (!p.source_visible && !p.discarded)
                    || (!scalar(&p.value_type) && p.value_type != CompilerType::KernelByteRegion)
            })
            || function
                .parameters
                .iter()
                .filter(|p| scalar(&p.value_type))
                .count()
                > 16
            || function
                .parameters
                .iter()
                .filter(|p| p.value_type == CompilerType::KernelByteRegion)
                .count()
                > 1
        {
            return Err(invalid(
                "function classifier, capture, effect or parameter budget",
            ));
        }
    }
    let entry = functions
        .get(entry_symbol)
        .copied()
        .ok_or_else(|| invalid("missing entry"))?;
    let K::OrderedComparisonDecision {
        subject,
        rules,
        otherwise,
    } = &entry.body.result.kind
    else {
        return Err(invalid("entry lacks the natural countdown decision"));
    };
    let [rule] = rules.as_slice() else {
        return Err(invalid("countdown guard must have one rule"));
    };
    let measure = entry
        .parameters
        .iter()
        .position(|p| p.value_type == CompilerType::Nat && local(subject, &p.name))
        .ok_or_else(|| invalid("countdown subject is not a Nat parameter"))?;
    let bound = literal(&rule.operand)
        .filter(|b| *b >= &BigInt::from(0))
        .ok_or_else(|| invalid("countdown bound must be a nonnegative literal"))?;
    if rule.operation != Op::LessEqual {
        return Err(invalid("countdown guard must use inclusive <="));
    }
    let mut inspection = Inspection {
        calls: functions
            .keys()
            .map(|s| (s.clone(), BTreeSet::new()))
            .collect(),
        effects: functions
            .keys()
            .map(|s| (s.clone(), (false, false)))
            .collect(),
        functions: functions.clone(),
        entry,
        measure,
        bound: bound.clone(),
        edges: Vec::new(),
        decrements: Vec::new(),
        visits: 1,
    };
    // Pre-guard bindings and base actions cannot acquire guarded tail status.
    for statement in &entry.body.statements {
        inspection.statement(entry_symbol, statement, 1)?;
    }
    inspection.expression(entry_symbol, subject, false, 2)?;
    inspection.expression(entry_symbol, &rule.operand, false, 2)?;
    inspection.expression(entry_symbol, &rule.action, false, 2)?;
    inspection.expression(entry_symbol, otherwise, true, 2)?;
    for function in functions.values().filter(|f| f.symbol != entry_symbol) {
        inspection.block(&function.symbol, &function.body, false, 0)?;
    }
    let minimum = inspection
        .decrements
        .iter()
        .min()
        .ok_or_else(|| invalid("no tail back edge"))?;
    fn closure(
        symbol: &str,
        inspection: &Inspection<'_>,
        active: &mut BTreeSet<String>,
        seen: &mut BTreeMap<String, (usize, bool, bool)>,
    ) -> Result<(usize, bool, bool), CompileError> {
        if let Some(result) = seen.get(symbol) {
            return Ok(*result);
        }
        if !active.insert(symbol.into()) {
            return Err(invalid("helper cycle"));
        }
        let (mut read, mut write) = inspection.effects[symbol];
        let mut depth = 0;
        for callee in &inspection.calls[symbol] {
            let (nested, r, w) = closure(callee, inspection, active, seen)?;
            depth = depth.max(nested);
            read |= r;
            write |= w;
        }
        active.remove(symbol);
        let result = (depth + 1, read, write);
        seen.insert(symbol.into(), result);
        Ok(result)
    }
    let mut seen = BTreeMap::new();
    let (depth, _, _) = closure(entry_symbol, &inspection, &mut BTreeSet::new(), &mut seen)?;
    if seen.len() != functions.len() {
        return Err(invalid("unreachable function in selected closure"));
    }
    for (symbol, function) in &functions {
        if (seen[symbol].1 || seen[symbol].2)
            && function
                .declared_effects
                .as_ref()
                .is_some_and(|row| row.identities.is_empty())
        {
            return Err(invalid("owner effects exceed empty Effects row"));
        }
    }
    let mut dependency_ids = BTreeSet::new();
    let mut dependencies = Vec::new();
    for dependency in &program.dependencies {
        if !dependency_ids.insert(&dependency.identity) {
            return Err(invalid("duplicate dependency identity"));
        }
        dependencies.push(SystemsTailDependency {
            identity: dependency.identity.clone(),
            source_name: dependency.source_name.clone(),
            sha256: digest(dependency.source_text.as_bytes()),
        });
    }
    let mut graph_hash = Sha256::new();
    graph_hash.update(b"topal.checked-function-debug/1\0");
    for function in functions.values() {
        let encoded = format!("{function:?}");
        graph_hash.update((encoded.len() as u64).to_le_bytes());
        graph_hash.update(encoded.as_bytes());
    }
    Ok(SystemsTailTraversalPlan {
        schema: "topal.systems-tail-source-plan/1".into(),
        qualification: "source-shape-only; no native emission or resource certificate".into(),
        source_sha256: digest(program.source.as_str().as_bytes()),
        dependencies,
        checked_graph_sha256: format!("{:x}", graph_hash.finalize()),
        entry_symbol: entry_symbol.into(),
        measure_parameter: measure,
        base_bound: bound.to_string(),
        minimum_decrement: minimum.to_string(),
        back_edges: inspection.edges,
        functions: functions
            .iter()
            .map(|(symbol, function)| SystemsTailFunction {
                symbol: symbol.clone(),
                parameter_classifiers: function
                    .parameters
                    .iter()
                    .map(|p| format!("{:?}", p.value_type))
                    .collect(),
                result_classifier: format!("{:?}", function.result_type),
                callees: inspection.calls[symbol].iter().cloned().collect(),
                reads_owner: seen[symbol].1,
                writes_owner: seen[symbol].2,
            })
            .collect(),
        maximum_one_step_function_depth: depth,
    })
}
