fn compiler_list_string_rational_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::List(element), CompilerType::Rational]
                if element.as_ref() == &CompilerType::String)
    )
}

fn compiler_string_function_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::String, CompilerType::Function]
    )
}

fn compiler_int_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice()
                == [CompilerType::Int, CompilerType::Int, CompilerType::Int]
    )
}

fn compiler_string_string_rational_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice()
                == [CompilerType::String, CompilerType::String, CompilerType::Rational]
    )
}

fn compiler_string_rational_string_list_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::String, CompilerType::Rational, CompilerType::List(element)]
                if element.as_ref() == &CompilerType::String)
    )
}

fn compiler_string_string_list_int_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::String, CompilerType::List(element), CompilerType::Int]
                if element.as_ref() == &CompilerType::String)
    )
}

fn compiler_boolean_int_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::Boolean, CompilerType::Int]
    )
}

fn compiler_boolean_string_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::Boolean, CompilerType::String]
    )
}

fn compiler_int_int_list_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::Int, CompilerType::Int, CompilerType::List(element)]
                if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat))
    )
}

fn compiler_three_pointer_tuple(value_type: &CompilerType) -> bool {
    compiler_int_triple(value_type)
        || compiler_string_string_rational_triple(value_type)
        || compiler_string_rational_string_list_triple(value_type)
        || compiler_string_string_list_int_triple(value_type)
        || compiler_int_int_list_triple(value_type)
}

fn compiler_int_int_boolean_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice()
                == [
                    CompilerType::Int,
                    CompilerType::Int,
                    CompilerType::Boolean,
                    CompilerType::Boolean,
                ]
    )
}

fn compiler_int_int_string_int_tuple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::Int | CompilerType::Nat,
                CompilerType::String,
                CompilerType::Int | CompilerType::Nat,
            ] | [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::String,
                CompilerType::Int | CompilerType::Nat,
                CompilerType::Int | CompilerType::Nat,
            ])
    )
}

fn compiler_nested_int_string_list_element(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::List(element) if compiler_int_string_pair(element)
    )
}

fn compiler_nested_int_list_element(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::List(element)
            if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat)
                || compiler_nested_int_list_element(element.as_ref())
    )
}

fn align_bits(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

fn private_aggregate_value_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata
        | CompilerType::Generator(_)
        | CompilerType::Task(_)
        | CompilerType::ExternalLocation(_) => false,
        CompilerType::Tuple(fields) => fields.iter().all(private_aggregate_value_supported),
        CompilerType::Record(fields) => fields
            .iter()
            .all(|(_, field)| private_aggregate_value_supported(field)),
        CompilerType::Sum(sum) => sum.alternatives.iter().all(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_none_or(private_aggregate_value_supported)
        }),
        _ => true,
    }
}

fn llvm_value_type(value_type: &CompilerType) -> String {
    match value_type {
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata => {
            unreachable!("static-only compiler values have no LLVM value type")
        }
        CompilerType::Unit | CompilerType::Completed | CompilerType::Effect => "i8".into(),
        CompilerType::Boolean => "i1".into(),
        CompilerType::Version
        | CompilerType::SerializationStream(_)
        | CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat
        | CompilerType::InfiniteRational
        | CompilerType::Modular(_)
        | CompilerType::Rational
        | CompilerType::Character
        | CompilerType::Error
        | CompilerType::ErrorDomain
        | CompilerType::SourceLocation
        | CompilerType::String
        | CompilerType::Range(_)
        | CompilerType::Result(_)
        | CompilerType::TaskResponse(_)
        | CompilerType::Optional(_)
        | CompilerType::TraversalControl(_)
        | CompilerType::List(_)
        | CompilerType::Array { .. }
        | CompilerType::Set(_)
        | CompilerType::Bag(_)
        | CompilerType::Map { .. }
        | CompilerType::Task(_)
        | CompilerType::ExternalLocation(_) => "ptr".into(),
        CompilerType::Type
        | CompilerType::Scope
        | CompilerType::Function
        | CompilerType::Constraint
        | CompilerType::Comparison
        | CompilerType::ErrorCode
        | CompilerType::Enum(_)
        | CompilerType::Generator(_) => "i32".into(),
        CompilerType::Refined { base, .. } => llvm_value_type(base),
        CompilerType::Tuple(fields) => format!(
            "{{ {} }}",
            fields
                .iter()
                .map(llvm_value_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CompilerType::Record(fields) => format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|(_, value_type)| llvm_value_type(value_type))
                .chain(fields.iter().map(|_| "i32".to_owned()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CompilerType::Sum(sum) => format!(
            "{{ {} }}",
            std::iter::once("i32".to_owned())
                .chain(
                    sum.alternatives
                        .iter()
                        .filter_map(|alternative| alternative.payload.as_ref())
                        .map(llvm_value_type)
                )
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn function_parameter_value(value_type: &CompilerType, index: usize) -> LlValue {
    machine_value(value_type, format!("%arg{index}"))
}

#[allow(clippy::too_many_lines)] // Every admitted machine representation stays explicit.
fn machine_value(value_type: &CompilerType, value: String) -> LlValue {
    match value_type {
        CompilerType::Unit => LlValue::Unit,
        CompilerType::Completed => LlValue::Completed(value),
        CompilerType::Effect => LlValue::Effect(value),
        CompilerType::Type => LlValue::Enum {
            value,
            enumeration: fundamental_type_enumeration(),
        },
        CompilerType::Scope => LlValue::Enum {
            value,
            enumeration: scope_enumeration(),
        },
        CompilerType::Function => {
            unreachable!("Function values are not admitted at machine ABI reconstruction points")
        }
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata => {
            unreachable!("static-only compiler values have no machine representation")
        }
        CompilerType::SerializationStream(_) => {
            unreachable!("SerializationStream function boundaries are not admitted")
        }
        CompilerType::Version => {
            unreachable!("Version function boundaries are not admitted")
        }
        CompilerType::Constraint => {
            unreachable!("Constraint values are not admitted at machine ABI reconstruction points")
        }
        CompilerType::Boolean => LlValue::Boolean(value),
        CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat => LlValue::Int(value),
        CompilerType::Modular(modular) => LlValue::Modular {
            value,
            modular: modular.clone(),
        },
        CompilerType::Rational | CompilerType::InfiniteRational => LlValue::Rational(value),
        CompilerType::Comparison => LlValue::Comparison(value),
        CompilerType::Error => LlValue::Error(value),
        CompilerType::ErrorCode => LlValue::ErrorCode(value),
        CompilerType::ErrorDomain => LlValue::ErrorDomain(value),
        CompilerType::SourceLocation => LlValue::SourceLocation(value),
        CompilerType::Enum(enumeration) => LlValue::Enum {
            value,
            enumeration: enumeration.clone(),
        },
        CompilerType::Generator(generator) => LlValue::Generator {
            value,
            generator: generator.clone(),
            captured_initial: None,
            captured_additional_initials: Vec::new(),
        },
        CompilerType::Task(task) => LlValue::Task {
            value,
            task: task.as_ref().clone(),
        },
        CompilerType::ExternalLocation(location) => LlValue::ExternalLocation {
            value,
            location: location.as_ref().clone(),
        },
        CompilerType::Character | CompilerType::String => LlValue::String(value),
        CompilerType::Range(endpoint) => LlValue::Range {
            value,
            endpoint: endpoint.as_ref().clone(),
        },
        CompilerType::Result(success) | CompilerType::TaskResponse(success) => LlValue::Result {
            value,
            success: success.as_ref().clone(),
            function_captures: Vec::new(),
        },
        CompilerType::Optional(payload) => LlValue::Optional {
            value,
            payload: payload.as_ref().clone(),
            function_captures: Vec::new(),
        },
        CompilerType::TraversalControl(payload) => LlValue::TraversalControl {
            value,
            payload: payload.as_ref().clone(),
        },
        CompilerType::List(element) => LlValue::List {
            value,
            element: element.as_ref().clone(),
            function_captures: Vec::new(),
        },
        value_type @ (CompilerType::Array { .. }
        | CompilerType::Set(_)
        | CompilerType::Bag(_)
        | CompilerType::Map { .. }) => LlValue::Container {
            value,
            value_type: value_type.clone(),
            function_captures: Vec::new(),
            function_capture_keys: Vec::new(),
        },
        CompilerType::Refined { base, .. } => machine_value(base, value),
        CompilerType::Tuple(_) | CompilerType::Record(_) | CompilerType::Sum(_) => {
            unreachable!("aggregate machine values require structural lowering")
        }
    }
}

fn llvm_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 3);
    for byte in bytes {
        write!(encoded, "\\{byte:02X}").expect("writing to a String cannot fail");
    }
    encoded
}

fn llvm_string(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b' '..=b'~' if !matches!(byte, b'"' | b'\\') => char::from(byte).to_string(),
            _ => format!("\\{byte:02X}"),
        })
        .collect()
}

const PLATFORM_RUNTIME: &str = include_str!("../runtime/linux_x86_64.ll");
const INFINITY_RESULT_RUNTIME: &str = include_str!("../runtime/infinity_result.ll");
const LIST_BOOLEAN_CORE_RUNTIME: &str = include_str!("../runtime/list_boolean_core.ll");
const LIST_BOOLEAN_STRING_PAIR_CORE_RUNTIME: &str =
    include_str!("../runtime/list_boolean_string_pair_core.ll");
const LIST_COMPLETED_CORE_RUNTIME: &str = include_str!("../runtime/list_completed_core.ll");
const LIST_COMPARISON_CORE_RUNTIME: &str = include_str!("../runtime/list_comparison_core.ll");
const LIST_EFFECT_CORE_RUNTIME: &str = include_str!("../runtime/list_effect_core.ll");
const LIST_ENUM_CORE_RUNTIME: &str = include_str!("../runtime/list_enum_core.ll");
const LIST_ERROR_CODE_CORE_RUNTIME: &str = include_str!("../runtime/list_error_code_core.ll");
const LIST_MODULAR_CORE_RUNTIME: &str = include_str!("../runtime/list_modular_core.ll");
const LIST_OPTIONAL_INT_CORE_RUNTIME: &str = include_str!("../runtime/list_optional_int_core.ll");
const LIST_OPTIONAL_RATIONAL_CORE_RUNTIME: &str =
    include_str!("../runtime/list_optional_rational_core.ll");
const LIST_OPTIONAL_STRING_CORE_RUNTIME: &str =
    include_str!("../runtime/list_optional_string_core.ll");
const LIST_INT_PAIR_CORE_RUNTIME: &str = include_str!("../runtime/list_int_pair_core.ll");
const LIST_INT_TRIPLE_CORE_RUNTIME: &str = include_str!("../runtime/list_int_triple_core.ll");
const LIST_INT_INT_STRING_INT_CORE_RUNTIME: &str =
    include_str!("../runtime/list_int_int_string_int_core.ll");
const LIST_INT_INT_BOOLEAN_PAIR_CORE_RUNTIME: &str =
    include_str!("../runtime/list_int_int_boolean_pair_core.ll");
const LIST_INT_RANGE_CORE_RUNTIME: &str = include_str!("../runtime/list_int_range_core.ll");
const LIST_INT_STRING_PAIR_CORE_RUNTIME: &str =
    include_str!("../runtime/list_int_string_pair_core.ll");
const LIST_STRING_INT_PAIR_CORE_RUNTIME: &str =
    include_str!("../runtime/list_string_int_pair_core.ll");
const LIST_STRING_PAIR_CORE_RUNTIME: &str = include_str!("../runtime/list_string_pair_core.ll");
const LIST_RATIONAL_CORE_RUNTIME: &str = include_str!("../runtime/list_rational_core.ll");
const LIST_STRING_CORE_RUNTIME: &str = include_str!("../runtime/list_string_core.ll");
const LIST_TYPE_CORE_RUNTIME: &str = include_str!("../runtime/list_type_core.ll");
const LIST_UNIT_CORE_RUNTIME: &str = include_str!("../runtime/list_unit_core.ll");
const LIST_INT_LAYOUT: &str = include_str!("../runtime/list_int_layout.ll");
const LIST_INT_CONTAINMENT_RUNTIME: &str = include_str!("../runtime/list_int_containment.ll");
const LIST_INT_REMOVAL_RUNTIME: &str = include_str!("../runtime/list_int_removal.ll");
const LIST_INT_CORE_RUNTIME: &str = include_str!("../runtime/list_int_core.ll");
const LIST_INT_RANGE_SELECTION_RUNTIME: &str =
    include_str!("../runtime/list_int_range_selection.ll");
const LIST_INT_SEQUENCE_RUNTIME: &str = include_str!("../runtime/list_int_sequence.ll");
const FUNDAMENTAL_CONTAINERS_RUNTIME: &str = include_str!("../runtime/fundamental_containers.ll");
const LIST_NESTED_INT_CORE_RUNTIME: &str = include_str!("../runtime/list_nested_int_core.ll");
const LIST_NESTED_INT_STRING_CORE_RUNTIME: &str =
    include_str!("../runtime/list_nested_int_string_core.ll");
