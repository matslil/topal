//! Private, checked projection descriptions derived from ordinary lowering.
//! These offsets grant no source or runtime storage authority.
use crate::CompileError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use topal_language::compiler::CompilerType;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedNumber {
    pub path: Vec<String>,
    pub indices: Vec<u32>,
    pub pointer_offset: u64,
    pub node_offset: u64,
    pub minimum: u64,
    pub maximum: u64,
    pub input_maximum: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedBoolean {
    pub path: Vec<String>,
    pub indices: Vec<u32>,
    pub offset: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedRecordOrder {
    pub path: Vec<String>,
    pub indices: Vec<u32>,
    pub fields: Vec<String>,
    pub offsets: Vec<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedZeroMask {
    pub offset: u64,
    pub mask: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedValueLayout {
    pub schema: String,
    pub llvm_type: String,
    pub record_bytes: u64,
    pub graph_bytes: u64,
    pub numbers: Vec<RetainedNumber>,
    pub booleans: Vec<RetainedBoolean>,
    pub orders: Vec<RetainedRecordOrder>,
    pub zero_masks: Vec<RetainedZeroMask>,
}
fn invalid(message: &str) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-RETAINED-LAYOUT: {message}"))
}

pub(crate) fn structured_state_type() -> CompilerType {
    CompilerType::Record(vec![
        (
            "history".into(),
            CompilerType::Record(vec![
                ("creations".into(), CompilerType::Nat),
                ("wrapped".into(), CompilerType::Boolean),
            ]),
        ),
        ("next".into(), CompilerType::Nat),
    ])
}
pub(crate) fn structured_result_type() -> CompilerType {
    CompilerType::Record(vec![
        ("allocated".into(), CompilerType::Nat),
        ("state".into(), structured_state_type()),
    ])
}

impl RetainedValueLayout {
    /// A one-field order word includes its zero upper padding. The native
    /// order check validates both, so charging another mask load is unnecessary.
    pub(crate) fn separate_zero_masks(&self) -> Vec<&RetainedZeroMask> {
        self.zero_masks
            .iter()
            .filter(|mask| {
                !self.orders.iter().any(|record| {
                    record.fields.len() == 1
                        && record.offsets[0] % 8 == 0
                        && record.offsets[0] == mask.offset
                        && mask.mask == 0xffff_ffff_0000_0000
                })
            })
            .collect()
    }
    pub(crate) fn prepared_validation_words(&self) -> u64 {
        // Eight maximum header checks, then the prepared-next shadow. Each
        // numeric graph field reads pointer/sign/length/full payload once;
        // selected one-/two-field orders fit one native load each.
        9 + self.numbers.len() as u64 * 4
            + self.orders.len() as u64
            + self.separate_zero_masks().len() as u64
            + (2016 - self.graph_bytes) / 8
    }
    pub(crate) fn projection_words(&self) -> u64 {
        // Count each partial-width store as one full word operation.
        self.numbers.len() as u64 * 4
            + self.booleans.len() as u64
            + self
                .orders
                .iter()
                .map(|order| order.fields.len() as u64)
                .sum::<u64>()
    }
    pub(crate) fn identity(structured: bool) -> Result<Self, CompileError> {
        let value_type = if structured {
            structured_state_type()
        } else {
            CompilerType::Record(vec![("next".into(), CompilerType::Nat)])
        };
        let mut domains = BTreeMap::from([(
            vec!["next".to_owned()],
            (2, i32::MAX as u64, i32::MAX as u64),
        )]);
        if structured {
            domains.insert(
                vec!["history".into(), "creations".into()],
                (1, u64::MAX, u64::MAX - 1),
            );
        }
        Self::checked(&value_type, domains)
    }

    fn checked(
        value_type: &CompilerType,
        mut domains: BTreeMap<Vec<String>, (u64, u64, u64)>,
    ) -> Result<Self, CompileError> {
        fn admitted(ty: &CompilerType, depth: usize) -> bool {
            match ty {
                CompilerType::Nat | CompilerType::Boolean => true,
                CompilerType::Record(fields) => {
                    depth < 4
                        && !fields.is_empty()
                        && fields.len() <= 8
                        && fields.windows(2).all(|pair| pair[0].0 < pair[1].0)
                        && fields.iter().all(|(_, field)| admitted(field, depth + 1))
                }
                _ => false,
            }
        }
        // Reject static-only/resource/unqualified types before calling ordinary
        // machine-layout helpers, which correctly have no representation for them.
        if !matches!(value_type, CompilerType::Record(_)) || !admitted(value_type, 0) {
            return Err(invalid("unqualified or noncanonical retained value type"));
        }
        let record_bytes = crate::codegen::target_value_layout(value_type).size / 8;
        if record_bytes == 0 || record_bytes > 2016 || record_bytes % 8 != 0 {
            return Err(invalid("record does not fit a qualified version bank"));
        }
        let mut result = Self {
            schema: "topal.retained-value-layout.x86_64/1".into(),
            llvm_type: crate::codegen::llvm_value_type(value_type),
            record_bytes,
            graph_bytes: record_bytes,
            numbers: vec![],
            booleans: vec![],
            orders: vec![],
            zero_masks: vec![],
        };
        let mut occupied = vec![false; record_bytes as usize];
        fn walk(
            ty: &CompilerType,
            path: Vec<String>,
            indices: Vec<u32>,
            offset: u64,
            domains: &mut BTreeMap<Vec<String>, (u64, u64, u64)>,
            occupied: &mut [bool],
            result: &mut RetainedValueLayout,
        ) -> Result<(), CompileError> {
            let mut mark = |start: u64, count: u64| -> Result<(), CompileError> {
                let region = occupied
                    .get_mut(start as usize..(start + count) as usize)
                    .ok_or_else(|| invalid("ordinary field escapes its Record"))?;
                if region.iter().any(|used| *used) {
                    return Err(invalid("overlapping ordinary fields"));
                }
                region.fill(true);
                Ok(())
            };
            match ty {
                CompilerType::Nat => {
                    let (minimum, maximum, input_maximum) = domains
                        .remove(&path)
                        .ok_or_else(|| invalid("numeric path lacks its exact finite domain"))?;
                    if minimum == 0 || minimum > input_maximum || input_maximum > maximum {
                        return Err(invalid("invalid published/invocable numeric domain"));
                    }
                    mark(offset, 8)?;
                    let node_offset = result.graph_bytes;
                    result.graph_bytes += 24;
                    result.numbers.push(RetainedNumber {
                        path,
                        indices,
                        pointer_offset: offset,
                        node_offset,
                        minimum,
                        maximum,
                        input_maximum,
                    });
                }
                CompilerType::Boolean => {
                    mark(offset, 1)?;
                    result.booleans.push(RetainedBoolean {
                        path,
                        indices,
                        offset,
                    });
                }
                CompilerType::Record(fields) => {
                    let layout = crate::codegen::ordinary_record_layout(fields);
                    let offsets = layout
                        .order
                        .iter()
                        .map(|bits| offset + bits / 8)
                        .collect::<Vec<_>>();
                    for at in &offsets {
                        mark(*at, 4)?;
                    }
                    result.orders.push(RetainedRecordOrder {
                        path: path.clone(),
                        indices: indices.clone(),
                        fields: fields.iter().map(|(name, _)| name.clone()).collect(),
                        offsets,
                    });
                    for (index, ((name, field), bits)) in
                        fields.iter().zip(layout.fields).enumerate()
                    {
                        let mut nested_path = path.clone();
                        nested_path.push(name.clone());
                        let mut nested_indices = indices.clone();
                        nested_indices.push(index as u32);
                        walk(
                            field,
                            nested_path,
                            nested_indices,
                            offset + bits / 8,
                            domains,
                            occupied,
                            result,
                        )?;
                    }
                }
                _ => return Err(invalid("unqualified retained leaf")),
            }
            Ok(())
        }
        walk(
            value_type,
            vec![],
            vec![],
            0,
            &mut domains,
            &mut occupied,
            &mut result,
        )?;
        if !domains.is_empty() || result.graph_bytes > 2016 {
            return Err(invalid("extra domain or oversized complete graph"));
        }
        for (index, chunk) in occupied.chunks_exact(8).enumerate() {
            let mut mask = 0u64;
            for (byte, used) in chunk.iter().enumerate() {
                if !used {
                    mask |= 0xff << (byte * 8);
                }
            }
            // i1 source values have a deterministic one-byte retained encoding.
            for boolean in &result.booleans {
                if boolean.offset / 8 == index as u64 {
                    mask |= 0xfe << ((boolean.offset % 8) * 8);
                }
            }
            if mask != 0 {
                result.zero_masks.push(RetainedZeroMask {
                    offset: (index * 8) as u64,
                    mask,
                });
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_structured_source_fields_cannot_modify_the_published_or_prepared_graph() {
        // TOPAL-SYSTEMS-RETAINED-STRUCTURED-STATE-001
        use topal_language::compiler::{
            CompilerSystemsSourceLibraryInterface as Interface,
            CompilerSystemsSourceLibraryRequirement as Requirement,
        };
        let requirement = Requirement {
            source: "policy/structured-process-identity.t".into(),
            export: "process-identity".into(),
            interface: Interface::StructuredProcessIdentityV1,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
            state_limit_bytes: Some(4096),
        };
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
        let original = crate::systems_library::load_source_library(&requirement, &root).unwrap();
        let tools = crate::LlvmTools::discover(None).unwrap();
        let directory =
            std::env::temp_dir().join(format!("topal-structured-invalid-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let client = directory.join("client.c");
        let executable = directory.join("client");
        std::fs::write(
            &client,
            include_str!("../../../linux-kernel/labs/qemu/x86_64/structured_identity_client.c"),
        )
        .unwrap();
        for (field, expression) in [
            ("allocated", "0"),
            ("next", "0"),
            ("next", "2147483648"),
            ("next", "18446744073709551618"),
            ("creations", "0"),
            ("creations", "18446744073709551618"),
        ] {
            let mut changed = original.clone();
            match field {
                "allocated" => {
                    changed.text = changed.text.replace(
                        "allocated is support process-identity selected (state next)",
                        &format!("allocated : Nat is {expression}"),
                    )
                }
                "next" => {
                    let module = changed
                        .modules
                        .iter_mut()
                        .find(|m| m.identity == ["support", "process-identity"])
                        .unwrap();
                    module.text = module
                        .text
                        .replace(
                            "  one : Nat is 1",
                            &format!("  one : Nat is 1\n  bad : Nat is {expression}"),
                        )
                        .replace("false then allocated + one", "false then bad");
                }
                "creations" => {
                    let module = changed
                        .modules
                        .iter_mut()
                        .find(|m| m.identity == ["support", "identity-history"])
                        .unwrap();
                    module.text = module.text.replace(
                        "creations is (current creations) + one",
                        &format!("creations : Nat is {expression}"),
                    );
                }
                _ => unreachable!(),
            }
            crate::systems_numeric_library::build_numeric_library(
                &requirement,
                &changed,
                &tools,
                &directory,
            )
            .unwrap();
            let compiled = std::process::Command::new("cc")
                .arg("-no-pie")
                .arg(&client)
                .arg(directory.join("process-identity-library.o"))
                .arg("-o")
                .arg(&executable)
                .output()
                .unwrap();
            assert!(
                compiled.status.success(),
                "{}",
                String::from_utf8_lossy(&compiled.stderr)
            );
            let output = std::process::Command::new(&executable)
                .arg("invalid")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "field={field} expression={expression} status={:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn structured_adapter_preserves_whole_values_and_all_nested_order_combinations() {
        // TOPAL-SYSTEMS-RETAINED-STRUCTURED-STATE-001; TOPAL-SYSTEMS-STRUCTURED-PROCESS-IDENTITY-001
        use topal_language::compiler::{
            CompilerSystemsSourceLibraryInterface as Interface,
            CompilerSystemsSourceLibraryRequirement as Requirement,
        };
        let requirement = Requirement {
            source: "policy/structured-process-identity.t".into(),
            export: "process-identity".into(),
            interface: Interface::StructuredProcessIdentityV1,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
            state_limit_bytes: Some(4096),
        };
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
        let original = crate::systems_library::load_source_library(&requirement, &root).unwrap();
        let tools = crate::LlvmTools::discover(None).unwrap();
        let directory =
            std::env::temp_dir().join(format!("topal-structured-client-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let client = directory.join("client.c");
        let executable = directory.join("client");
        std::fs::write(
            &client,
            include_str!("../../../linux-kernel/labs/qemu/x86_64/structured_identity_client.c"),
        )
        .unwrap();
        for variant in 0..8 {
            let mut input = original.clone();
            let state =
                "Record (next : Nat, history : Record (creations : Nat, wrapped : Boolean))";
            if variant & 1 != 0 {
                input.text=input.text.replace(&format!("-> Record (allocated : Nat, state : {state})"),
                    &format!("-> Record (state : {state}, allocated : Nat)"))
                    .replace("(allocated is allocated, state is (next is following, history is history))",
                        "(state is (next is following, history is history), allocated is allocated)");
            }
            if variant & 2 != 0 {
                input.text=input.text.replace(state,"Record (history : Record (creations : Nat, wrapped : Boolean), next : Nat)")
                    .replace("(next is following, history is history)","(history is history, next is following)");
            }
            if variant & 4 != 0 {
                input.text = input.text.replace(
                    "Record (creations : Nat, wrapped : Boolean)",
                    "Record (wrapped : Boolean, creations : Nat)",
                );
                let module = input
                    .modules
                    .iter_mut()
                    .find(|m| m.identity == ["support", "identity-history"])
                    .unwrap();
                module.text = module
                    .text
                    .replace(
                        "Record (creations : Nat, wrapped : Boolean)",
                        "Record (wrapped : Boolean, creations : Nat)",
                    )
                    .replace(
                        "(creations is creations, wrapped is wrapped)",
                        "(wrapped is wrapped, creations is creations)",
                    );
            }
            let library = crate::systems_numeric_library::build_numeric_library(
                &requirement,
                &input,
                &tools,
                &directory,
            )
            .unwrap();
            assert_eq!(
                library.record.retained_state,
                Some(crate::SystemsRetainedStateBound::structured().unwrap())
            );
            assert!(library.record.result_order_suffix);
            assert_eq!(library.record.result_field_order, ["allocated", "state"]);
            assert_eq!(library.record.allocation.maximum_bytes, 592);
            assert!(library.record.maximum_stack_bytes <= 4096);
            let compiled = std::process::Command::new("cc")
                .arg("-no-pie")
                .arg(&client)
                .arg(directory.join("process-identity-library.o"))
                .arg("-o")
                .arg(&executable)
                .output()
                .unwrap();
            assert!(
                compiled.status.success(),
                "{}",
                String::from_utf8_lossy(&compiled.stderr)
            );
            let output = std::process::Command::new(&executable)
                .arg(variant.to_string())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "variant={variant} status={:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            );
            let facts = String::from_utf8(output.stdout)
                .unwrap()
                .split_whitespace()
                .map(|v| v.parse::<u64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(facts.len(), 3);
            assert!(facts[0] <= 592);
            assert_eq!(facts[1], 66584);
            assert_eq!(facts[2], 48);
            eprintln!(
                "structured variant={variant} stack={} bound=592 highwater={} transitions={} rejections={}",
                library.record.maximum_stack_bytes, facts[0], facts[1], facts[2]
            );
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn checked_source_nested_layout_matches_an_independent_native_c_aggregate() {
        // TOPAL-SYSTEMS-RETAINED-STRUCTURED-STATE-001; TOPAL-SYSTEMS-STRUCTURED-PROCESS-IDENTITY-001
        use topal_language::compiler::{
            CompilerSystemsSourceLibraryInterface as Interface,
            CompilerSystemsSourceLibraryRequirement as Requirement,
        };
        let requirement = Requirement {
            source: "policy/structured-process-identity.t".into(),
            export: "process-identity".into(),
            interface: Interface::StructuredProcessIdentityV1,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
            state_limit_bytes: Some(4096),
        };
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
        let input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
        let checked =
            crate::systems_numeric_library::check_numeric_library(&requirement, &input).unwrap();
        assert_eq!(input.modules.len(), 2);
        assert_eq!(checked.allocation.maximum_operand_bits, 64);
        assert!(checked.allocation.maximum_bytes <= 65536);
        let entry = checked
            .program
            .functions
            .iter()
            .find(|f| f.symbol == checked.entry_symbol)
            .unwrap();
        assert_eq!(entry.parameters[0].value_type, structured_state_type());
        assert_eq!(entry.result_type, structured_result_type());
        let layout = RetainedValueLayout::identity(true).unwrap();
        let top = layout.orders.iter().find(|r| r.path.is_empty()).unwrap();
        let history = layout
            .orders
            .iter()
            .find(|r| r.path == ["history"])
            .unwrap();
        let ordinary_outcome = crate::codegen::ordinary_record_layout(match &entry.result_type {
            CompilerType::Record(fields) => fields,
            _ => unreachable!(),
        });
        let history_type = match &entry.parameters[0].value_type {
            CompilerType::Record(fields) => &fields[0].1,
            _ => unreachable!(),
        };
        let directory =
            std::env::temp_dir().join(format!("topal-structured-layout-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let client = directory.join("layout.c");
        let executable = directory.join("layout");
        std::fs::write(
            &client,
            include_str!("../../../linux-kernel/labs/qemu/x86_64/structured_layout_client.c"),
        )
        .unwrap();
        let compiled = std::process::Command::new("cc")
            .arg(&client)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let output = std::process::Command::new(executable).output().unwrap();
        assert!(output.status.success());
        let actual = String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse::<u64>().unwrap())
            .collect::<Vec<_>>();
        let count = layout
            .numbers
            .iter()
            .find(|n| n.path == ["history", "creations"])
            .unwrap();
        let next = layout.numbers.iter().find(|n| n.path == ["next"]).unwrap();
        assert_eq!(
            actual,
            vec![
                crate::codegen::target_value_layout(history_type).size / 8,
                layout.record_bytes,
                ordinary_outcome.value.size / 8,
                24,
                count.pointer_offset,
                layout.booleans[0].offset,
                history.offsets[0],
                history.offsets[1],
                next.pointer_offset,
                top.offsets[0],
                top.offsets[1],
                ordinary_outcome.fields[1] / 8,
                ordinary_outcome.order[0] / 8,
                ordinary_outcome.order[1] / 8
            ]
        );
        assert_eq!(layout.graph_bytes, layout.record_bytes + 2 * actual[3]);
        eprintln!(
            "structured source bound={} record={} graph={} result={}",
            checked.allocation.maximum_bytes, layout.record_bytes, layout.graph_bytes, actual[2]
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
