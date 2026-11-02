//! Source-region costs reuse the reviewed ordinary numeric/collection model.
use super::*;
use topal_language::compiler::{
    CompilerKernelRegionOperation as RegionOp, CompilerKernelRegionProfile,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRegionAllocationBound {
    pub numeric: SystemsNumericAllocationBound,
    pub maximum_view_nodes: u64,
    pub control_bytes: u64,
    pub view_bytes: u64,
    pub region_runtime_sha256: String,
    pub adapter_sha256: String,
}

impl Bounds<'_> {
    pub(super) fn region_expression(
        &mut self,
        expression: &CompilerExpression,
        env: &BTreeMap<String, Value>,
    ) -> Result<(Value, u64), CompileError> {
        let K::KernelRegion {
            operation,
            region,
            arguments,
            ..
        } = &expression.kind
        else {
            unreachable!()
        };
        let region = self.expression(region, env)?;
        if !matches!(region.value, Value::Region) {
            return Err(invalid("missing region origin"));
        }
        let mut bytes = region.bytes;
        for argument in arguments {
            let argument = self.expression(argument, env)?;
            if argument.value.number()?.0 < BigInt::from(0) {
                return Err(invalid("negative region fact"));
            }
            bytes = cost_add(bytes, argument.bytes)?;
        }
        let number = |maximum| Value::Number(Range(BigInt::from(0), BigInt::from(maximum)));
        let (value, cost) = match operation {
            RegionOp::Count => (number(65536), 24),
            RegionOp::Load => (
                Value::Optional {
                    some: Some(Box::new(number(255))),
                    none: true,
                },
                40,
            ),
            RegionOp::Store => (Value::Boolean, 0),
            RegionOp::Window { .. } => (
                Value::Optional {
                    some: Some(Box::new(Value::Region)),
                    none: true,
                },
                80,
            ),
        };
        Ok((value, cost_add(bytes, cost)?))
    }
}

pub(crate) fn bound_region_library_profile(
    program: &CompilerProgram,
    entry: &str,
    profile: CompilerKernelRegionProfile,
) -> Result<SystemsRegionAllocationBound, CompileError> {
    // Reviewed count/load/store/window costs: 24/40/0/80 bytes; root 64.
    // Changes to the primitive algorithms require re-review of these bounds.
    let region_runtime_sha256 = format!(
        "{:x}",
        Sha256::digest(include_bytes!("runtime/kernel_regions.ll"))
    );
    if region_runtime_sha256 != "da1f90f9d2366262f666d5be06a835a162a224154c318dc82fac68116e8c5424" {
        return Err(invalid(
            "region primitive allocation certificate requires review",
        ));
    }
    let (adapter_sha256, certified_adapter) = match profile {
        CompilerKernelRegionProfile::Unit => (
            crate::codegen::region_adapter_sha256(),
            "4ed4e32d16f151d0ede0bd06e58673a4e502506301c57f2b71ed0b4ada80de1e",
        ),
        CompilerKernelRegionProfile::Action => (
            crate::codegen::region_action_adapter_sha256(),
            "6b8e0e8d4b234aeae8c6d666c3db66854bb91b1031fc8c603feb76d424aaa058",
        ),
    };
    let hooks_sha256 = format!(
        "{:x}",
        Sha256::digest(include_bytes!("runtime/kernel_region_platform.ll"))
    );
    if adapter_sha256 != certified_adapter
        || hooks_sha256 != "9824eb2c9aad1d50930568754400cd68c80ce7aaa781b0e4b1b02fb0f3c7de8e"
    {
        return Err(invalid(format!(
            "region adapter/hook allocation certificate requires review: {adapter_sha256}"
        )));
    }
    let runtime_sha256 = format!(
        "{:x}",
        Sha256::digest(include_bytes!("runtime/value_core.ll"))
    );
    if runtime_sha256 != CERTIFIED_RUNTIME_SHA256 {
        return Err(invalid(
            "shared region numeric allocation certificate requires review",
        ));
    }
    let (collection_runtime_sha256, collection_layout_sha256) = collections::runtime_digests()?;
    let functions = program
        .functions
        .iter()
        .map(|f| (f.symbol.clone(), f))
        .collect::<BTreeMap<_, _>>();
    let mut bounds = Bounds {
        functions: functions.clone(),
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
        maximum_bits: 0,
        regions: true,
    };
    let (inputs, bridge_bytes) = match profile {
        CompilerKernelRegionProfile::Unit => (vec![Value::Region], 64),
        CompilerKernelRegionProfile::Action => {
            let scalar = Value::Number(Range(BigInt::from(0), BigInt::from(u64::MAX)));
            (vec![Value::Region, scalar.clone(), scalar], 112)
        }
    };
    let result = bounds.function(entry, inputs)?;
    if !matches!(
        (&profile, &result.value),
        (CompilerKernelRegionProfile::Unit, Value::Unit)
            | (CompilerKernelRegionProfile::Action, Value::Boolean)
    ) {
        return Err(invalid(
            "region entry result does not match the selected context",
        ));
    }
    let source_evaluation_bytes = result.bytes;
    let maximum_bytes = cost_add(
        cost_add(source_evaluation_bytes, bridge_bytes)?,
        HEADER_BYTES,
    )?;
    let mut views = ViewBounds {
        functions,
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    let maximum_view_nodes = cost_add(views.function(entry)?, 1)?;
    if maximum_view_nodes > 256 {
        return Err(invalid("region source exceeds issued-view bound"));
    }
    Ok(SystemsRegionAllocationBound {
        numeric: SystemsNumericAllocationBound {
            certificate: match profile {
                CompilerKernelRegionProfile::Unit => "topal.kernel-region-allocation/1",
                CompilerKernelRegionProfile::Action => "topal.kernel-region-action-allocation/1",
            }
            .into(),
            runtime_sha256,
            collection_runtime_sha256,
            collection_layout_sha256,
            maximum_operand_bits: bounds.maximum_bits,
            source_evaluation_bytes,
            bridge_bytes,
            header_bytes: HEADER_BYTES,
            maximum_bytes,
            limit_bytes: ARENA_BYTES,
        },
        maximum_view_nodes,
        control_bytes: 64,
        view_bytes: 64,
        region_runtime_sha256,
        adapter_sha256,
    })
}

struct ViewBounds<'a> {
    functions: BTreeMap<String, &'a CompilerFunction>,
    cache: BTreeMap<String, u64>,
    active: BTreeSet<String>,
}
impl ViewBounds<'_> {
    fn function(&mut self, symbol: &str) -> Result<u64, CompileError> {
        if let Some(bound) = self.cache.get(symbol) {
            return Ok(*bound);
        }
        if !self.active.insert(symbol.into()) {
            return Err(invalid("recursive region view bound"));
        }
        let function = *self
            .functions
            .get(symbol)
            .ok_or_else(|| invalid("unknown region view callee"))?;
        let bound = self.block(&function.body)?;
        self.active.remove(symbol);
        self.cache.insert(symbol.into(), bound);
        Ok(bound)
    }
    fn block(&mut self, block: &CompilerBlock) -> Result<u64, CompileError> {
        let mut count = 0;
        for statement in &block.statements {
            let expression = match statement {
                CompilerStatement::Binding(b) => &b.value,
                CompilerStatement::Discard(e) => e,
            };
            count = cost_add(count, self.expression(expression)?)?;
        }
        cost_add(count, self.expression(&block.result)?)
    }
    fn expression(&mut self, value: &CompilerExpression) -> Result<u64, CompileError> {
        match &value.kind {
            K::Unit | K::Boolean(_) | K::Int(_) | K::Local(_) | K::OptionalNone | K::ListEmpty => {
                Ok(0)
            }
            K::IntToNat(input)
            | K::Absolute(input)
            | K::Not(input)
            | K::OptionalSome(input)
            | K::ContainerEntryCount(input)
            | K::ContainerEmpty(input) => self.expression(input),
            K::KernelRegion {
                operation,
                region,
                arguments,
                ..
            } => {
                let mut count = self.expression(region)?;
                for argument in arguments {
                    count = cost_add(count, self.expression(argument)?)?;
                }
                cost_add(
                    count,
                    u64::from(matches!(operation, RegionOp::Window { .. })),
                )
            }
            K::Block(block) => self.block(block),
            K::Call { symbol, arguments } => {
                let mut count = self.function(symbol)?;
                for argument in arguments {
                    count = cost_add(count, self.expression(argument)?)?;
                }
                Ok(count)
            }
            K::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => {
                let count = self.expression(subject)?;
                let branches = self
                    .expression(when_true)?
                    .max(self.expression(when_false)?);
                cost_add(count, branches)
            }
            K::OptionalDecision {
                subject,
                some_action,
                none_action,
                ..
            } => {
                let count = self.expression(subject)?;
                let branches = self
                    .expression(some_action)?
                    .max(self.expression(none_action)?);
                cost_add(count, branches)
            }
            K::Binary { left, right, .. }
            | K::ListEntry {
                value: left,
                remaining: right,
            } => cost_add(self.expression(left)?, self.expression(right)?),
            K::Record(fields) => {
                let mut count = 0;
                for (_, field) in fields {
                    count = cost_add(count, self.expression(field)?)?;
                }
                Ok(count)
            }
            K::RecordField { record, .. } | K::TupleField { tuple: record, .. } => {
                self.expression(record)
            }
            K::RecordReconstruct { base, replacements } => {
                let mut count = self.expression(base)?;
                for (_, field) in replacements {
                    count = cost_add(count, self.expression(field)?)?;
                }
                Ok(count)
            }
            K::Tuple(fields) => {
                let mut count = 0;
                for field in fields {
                    count = cost_add(count, self.expression(field)?)?;
                }
                Ok(count)
            }
            K::ContainerCollect {
                source,
                kind: topal_language::compiler::CompilerContainerKind::Array,
                ..
            } => self.expression(source),
            K::ArrayAt { array, .. } => self.expression(array),
            _ => Err(invalid("unqualified region view-bound expression")),
        }
    }
}
