//! Conditional all-step ranges and per-step allocation; no native qualification.
use super::*;
use crate::SystemsTailTraversalPlan;
use topal_language::compiler::CompilerKernelRegionOperation as RegionOp;

#[path = "systems_tail_call_bounds.rs"]
mod calls;
#[path = "systems_tail_linear.rs"]
mod linear;
#[path = "systems_tail_step_bounds.rs"]
mod step;
#[cfg(test)]
#[path = "systems_tail_bounds_tests.rs"]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailValueBound {
    pub parameter: usize,
    pub classifier: String,
    pub initial_lower: String,
    pub initial_upper: String,
    pub baseline_lower: String,
    pub baseline_upper: String,
    pub lower: String,
    pub upper: String,
    pub positive_weight: String,
    pub negative_weight: String,
    pub maximum_bits: u64,
    pub bank_offset: u64,
    pub bank_slot_bytes: u64,
}

/// Allocation/range evidence only. A publisher must independently qualify the
/// actual transfer, native stack, identity admission and physical reclamation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailStorageBound {
    pub schema: String,
    pub qualification: String,
    pub source_plan: SystemsTailTraversalPlan,
    pub input_word_bits: u64,
    pub runtime_sha256: String,
    pub region_runtime_sha256: String,
    pub collection_runtime_sha256: String,
    pub collection_layout_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsigned_runtime_sha256: Option<String>,
    pub values: Vec<SystemsTailValueBound>,
    pub region_parameter: Option<usize>,
    pub maximum_operand_bits: u64,
    pub source_step_bytes: u64,
    pub terminal_step_bytes: u64,
    pub recursive_step_bytes: u64,
    pub initial_bridge_bytes: u64,
    pub bank_bytes: u64,
    pub private_metadata_bytes: u64,
    pub maximum_private_temporary_bytes: u64,
    pub limit_bytes: u64,
    pub caller_input_frame_bytes: u64,
    pub source_frames_per_step: String,
    pub source_region_loans_per_step: String,
    /// Fresh invocation: root identity 1 is live; the next counter is 2.
    /// Existing histories require a separate admission with their actual next.
    pub identity_counter_initial_next: u64,
    pub maximum_back_edges_for_identity_budget: u64,
}

fn fail(reason: impl Into<String>) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-TAIL-BOUNDS: {}", reason.into()))
}
fn word_maximum() -> BigInt {
    BigInt::from(u64::MAX)
}
fn contains(outer: &Range, inner: &Range) -> bool {
    outer.0 <= inner.0 && inner.1 <= outer.1
}
fn ceil_positive(value: &BigInt, divisor: &BigInt) -> BigInt {
    (value + divisor - 1) / divisor
}

fn identity_back_edge_limit(
    per_step: &BigInt,
    maximum: &BigInt,
    next: &BigInt,
) -> Result<BigInt, CompileError> {
    if per_step <= &BigInt::from(0) || next < &BigInt::from(2) || next > maximum {
        return Err(fail("invalid nonwrapping identity context"));
    }
    // The model advances the next counter before issuing each identity. The
    // maximum counter value may be reached but cannot itself be issued.
    let steps = (maximum - next) / per_step;
    if steps < BigInt::from(1) {
        return Err(fail("one step exceeds nonwrapping identity budget"));
    }
    Ok(steps - BigInt::from(1))
}

#[derive(Clone)]
struct Carrier {
    parameter: usize,
    kind: CompilerType,
    initial: Range,
    baseline: Range,
    positive: BigInt,
    negative: BigInt,
    global: Range,
}
impl Carrier {
    fn at_measure(&self, minimum: &BigInt) -> Range {
        let maximum = word_maximum();
        let mut range = Range(
            &self.baseline.0 - &self.negative * (&maximum - minimum),
            &self.baseline.1 + &self.positive * (&maximum - minimum),
        );
        if self.kind == CompilerType::Nat {
            range.0 = range.0.max(0.into());
        }
        range
    }
}

#[derive(Clone, Debug)]
enum Update {
    Measure,
    Region,
    Boolean,
    Affine(BigInt),
    Constant(BigInt),
    Reset,
}
struct Edge<'a> {
    expression: &'a CompilerExpression,
    decrement: BigInt,
    updates: Vec<Update>,
}

fn carriers_and_edges<'a>(
    program: &CompilerProgram,
    entry: &'a CompilerFunction,
    plan: &SystemsTailTraversalPlan,
) -> Result<(Vec<Carrier>, Vec<Edge<'a>>), CompileError> {
    let maximum = word_maximum();
    let mut carriers = entry
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| {
            let range = match parameter.value_type {
                CompilerType::Nat => Range(0.into(), maximum.clone()),
                CompilerType::Int => Range(-maximum.clone(), maximum.clone()),
                CompilerType::Boolean => Range(0.into(), 1.into()),
                _ => return None,
            };
            Some(Carrier {
                parameter: index,
                kind: parameter.value_type.clone(),
                initial: range.clone(),
                baseline: range.clone(),
                global: range,
                positive: 0.into(),
                negative: 0.into(),
            })
        })
        .collect::<Vec<_>>();
    let folded = linear::edges(program, entry)?;
    if folded.len() != plan.back_edges.len() {
        return Err(fail("source edge reconstruction differs from plan"));
    }
    let mut edges = Vec::new();
    for ((expression, values), proof) in folded.into_iter().zip(&plan.back_edges) {
        let decrement = proof
            .decrement
            .parse::<BigInt>()
            .map_err(|_| fail("invalid decrement"))?;
        let mut updates = Vec::new();
        for (index, (value, parameter)) in values.into_iter().zip(&entry.parameters).enumerate() {
            let update = if index == plan.measure_parameter {
                Update::Measure
            } else if parameter.value_type == CompilerType::KernelByteRegion {
                Update::Region
            } else if parameter.value_type == CompilerType::Boolean {
                Update::Boolean
            } else if let Some(value) = value {
                if value.coefficients.is_empty() {
                    Update::Constant(value.constant)
                } else if value.coefficients == BTreeMap::from([(index, BigInt::from(1))]) {
                    Update::Affine(value.constant)
                } else {
                    Update::Reset
                }
            } else {
                Update::Reset
            };
            if let Some(carrier) = carriers.iter_mut().find(|c| c.parameter == index) {
                match &update {
                    Update::Constant(value) => {
                        if parameter.value_type == CompilerType::Nat && value < &BigInt::from(0) {
                            return Err(fail("negative Nat reset"));
                        }
                        carrier.baseline =
                            carrier.baseline.union(&Range(value.clone(), value.clone()));
                    }
                    Update::Affine(change) if change > &BigInt::from(0) => {
                        carrier.positive = carrier
                            .positive
                            .clone()
                            .max(ceil_positive(change, &decrement));
                    }
                    Update::Affine(change) if change < &BigInt::from(0) => {
                        carrier.negative = carrier
                            .negative
                            .clone()
                            .max(ceil_positive(&(-change), &decrement));
                    }
                    _ => {}
                }
            }
            updates.push(update);
        }
        edges.push(Edge {
            expression,
            decrement,
            updates,
        });
    }
    for carrier in &mut carriers {
        carrier.global = carrier.at_measure(&BigInt::from(0));
        if carrier.kind == CompilerType::Nat {
            carrier.global.0 = carrier.global.0.clone().max(0.into());
        }
        if carrier.global.bits() > 256 {
            return Err(fail("carried range exceeds numeric proof ceiling"));
        }
    }
    Ok((carriers, edges))
}

/// Prove conditional carried ranges and allocation for native-word initial
/// facts. This neither attests external IR nor grants an executable profile.
pub fn analyze_systems_tail_storage(
    program: &CompilerProgram,
    entry_symbol: &str,
) -> Result<SystemsTailStorageBound, CompileError> {
    let plan = crate::analyze_systems_tail_traversal(program, entry_symbol)?;
    let entry = program
        .functions
        .iter()
        .find(|f| f.symbol == entry_symbol)
        .ok_or_else(|| fail("missing entry"))?;
    let (carriers, edges) = carriers_and_edges(program, entry, &plan)?;
    let runtime_sha256 = format!(
        "{:x}",
        Sha256::digest(include_bytes!("runtime/value_core.ll"))
    );
    let region_runtime_sha256 = format!(
        "{:x}",
        Sha256::digest(include_bytes!("runtime/kernel_regions.ll"))
    );
    if runtime_sha256 != CERTIFIED_RUNTIME_SHA256
        || region_runtime_sha256
            != "da1f90f9d2366262f666d5be06a835a162a224154c318dc82fac68116e8c5424"
    {
        return Err(fail("runtime allocation review is stale"));
    }
    let (collection_runtime_sha256, collection_layout_sha256) = collections::runtime_digests()?;
    let calls = calls::call_counts(program, entry)?;
    let unsigned_runtime_sha256 = if calls.unsigned {
        let hash = format!(
            "{:x}",
            Sha256::digest(include_bytes!("runtime/kernel_region_unsigned.ll"))
        );
        if hash != crate::systems_region_library::UNSIGNED_RUNTIME_SHA256 {
            return Err(fail("unsigned runtime review is stale"));
        }
        Some(hash)
    } else {
        None
    };
    let step = step::evaluate(program, entry, &plan, &carriers, &edges)?;
    let mut offset = 0;
    let mut values = Vec::new();
    for carrier in &carriers {
        let bytes = if carrier.kind == CompilerType::Boolean {
            8
        } else {
            (16 + 4 * carrier.global.bits().div_ceil(32)).div_ceil(8) * 8
        };
        values.push(SystemsTailValueBound {
            parameter: carrier.parameter,
            classifier: format!("{:?}", carrier.kind),
            initial_lower: carrier.initial.0.to_string(),
            initial_upper: carrier.initial.1.to_string(),
            baseline_lower: carrier.baseline.0.to_string(),
            baseline_upper: carrier.baseline.1.to_string(),
            lower: carrier.global.0.to_string(),
            upper: carrier.global.1.to_string(),
            positive_weight: carrier.positive.to_string(),
            negative_weight: carrier.negative.to_string(),
            maximum_bits: carrier.global.bits(),
            bank_offset: offset,
            bank_slot_bytes: bytes,
        });
        offset = cost_add(offset, bytes)?;
    }
    let bank_bytes = offset.div_ceil(16) * 16;
    let region_parameter = entry
        .parameters
        .iter()
        .position(|p| p.value_type == CompilerType::KernelByteRegion);
    let private_metadata_bytes =
        HEADER_BYTES + 64 + if region_parameter.is_some() { 64 } else { 0 };
    // Signed construction builds a magnitude and may copy with sign. Both
    // aligned allocations remain live until the first retirement.
    let initial_bridge_bytes = carriers
        .iter()
        .map(|carrier| match carrier.kind {
            CompilerType::Nat => 24,
            CompilerType::Int => 48,
            _ => 0,
        })
        .sum();
    // Full-domain Int facts use magnitude and sign. A narrower signed-word
    // bridge can separately qualify a smaller immutable input frame.
    let caller_input_frame_bytes = carriers
        .iter()
        .map(|carrier| {
            if carrier.kind == CompilerType::Int {
                16
            } else {
                8
            }
        })
        .sum();
    let maximum_private_temporary_bytes = cost_add(
        cost_add(private_metadata_bytes, cost_add(bank_bytes, bank_bytes)?)?,
        cost_add(initial_bridge_bytes, step.maximum)?,
    )?;
    let per_step = calls.frames.clone().max(calls.views.clone());
    let maximum_back_edges_for_identity_budget =
        identity_back_edge_limit(&per_step, &word_maximum(), &BigInt::from(2))?
            .to_string()
            .parse::<u64>()
            .map_err(|_| fail("invalid identity budget"))?;
    Ok(SystemsTailStorageBound {
        schema: "topal.systems-tail-storage-bound.x86_64/1".into(),
        qualification: "conditional ranges and allocation only; no native stack, transfer or lifetime qualification".into(),
        source_plan: plan, input_word_bits: 64, runtime_sha256, region_runtime_sha256,
        collection_runtime_sha256, collection_layout_sha256, unsigned_runtime_sha256,
        values, region_parameter, maximum_operand_bits: step.bits,
        source_step_bytes: step.maximum, terminal_step_bytes: step.terminal, recursive_step_bytes: step.recursive,
        initial_bridge_bytes, bank_bytes, private_metadata_bytes, maximum_private_temporary_bytes,
        limit_bytes: ARENA_BYTES, caller_input_frame_bytes,
        source_frames_per_step: calls.frames.to_string(), source_region_loans_per_step: calls.views.to_string(),
        identity_counter_initial_next: 2,
        maximum_back_edges_for_identity_budget,
    })
}
