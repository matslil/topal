#[derive(Clone)]
enum LlValue {
    Unit,
    StaticDisplay(String),
    Completed(String),
    Effect(String),
    Boolean(String),
    Version {
        value: String,
        display: String,
    },
    SerializationStream {
        stream: String,
        expected: String,
        byte_count: String,
        value: Box<Self>,
    },
    Task {
        value: String,
        task: CompilerTaskType,
    },
    ExternalLocation {
        value: String,
        location: CompilerLocationType,
    },
    Int(String),
    Modular {
        value: String,
        modular: CompilerModularType,
    },
    Rational(String),
    Comparison(String),
    Error(String),
    ErrorCode(String),
    ErrorDomain(String),
    SourceLocation(String),
    Enum {
        value: String,
        enumeration: CompilerEnumType,
    },
    Function {
        value: String,
        enumeration: CompilerEnumType,
        captures: Vec<(String, Self)>,
    },
    Generator {
        value: String,
        generator: CompilerGeneratorType,
        captured_initial: Option<Box<Self>>,
        captured_additional_initials: Vec<Self>,
    },
    Sum {
        tag: String,
        payloads: Vec<Option<Box<Self>>>,
        sum: CompilerSumType,
    },
    Range {
        value: String,
        endpoint: CompilerType,
    },
    Result {
        value: String,
        success: CompilerType,
        function_captures: Vec<(String, Self)>,
    },
    Optional {
        value: String,
        payload: CompilerType,
        function_captures: Vec<(String, Self)>,
    },
    TraversalControl {
        value: String,
        payload: CompilerType,
    },
    List {
        value: String,
        element: CompilerType,
        function_captures: Vec<Vec<(String, Self)>>,
    },
    Container {
        value: String,
        value_type: CompilerType,
        function_captures: Vec<Vec<(String, Self)>>,
        function_capture_keys: Vec<String>,
    },
    String(String),
    Tuple(Vec<Self>),
    Record {
        fields: Vec<(String, Self)>,
        order: Vec<String>,
    },
}

fn attach_function_capture(
    value: &mut LlValue,
    path: &[CompilerAggregatePathElement],
    storage_name: String,
    capture_value: LlValue,
) {
    let Some((first, rest)) = path.split_first() else {
        match value {
            LlValue::Function { captures, .. } => {
                captures.push((storage_name, capture_value));
            }
            LlValue::Enum {
                value: tag,
                enumeration,
            } => {
                *value = LlValue::Function {
                    value: tag.clone(),
                    enumeration: enumeration.clone(),
                    captures: vec![(storage_name, capture_value)],
                };
            }
            _ => unreachable!("checked capture path ends at a Function value"),
        }
        return;
    };
    match (first, value) {
        (CompilerAggregatePathElement::Tuple(index), LlValue::Tuple(fields)) => {
            attach_function_capture(&mut fields[*index], rest, storage_name, capture_value);
        }
        (CompilerAggregatePathElement::Record(name), LlValue::Record { fields, .. }) => {
            let field = fields
                .iter_mut()
                .find_map(|(label, field)| (label == name).then_some(field))
                .expect("checked capture path names a Record field");
            attach_function_capture(field, rest, storage_name, capture_value);
        }
        (
            CompilerAggregatePathElement::ListEntry(index),
            LlValue::List {
                function_captures, ..
            },
        ) if rest.is_empty() => {
            if function_captures.len() <= *index {
                function_captures.resize_with(*index + 1, Vec::new);
            }
            function_captures[*index].push((storage_name, capture_value));
        }
        (
            CompilerAggregatePathElement::ArrayEntry(index),
            LlValue::Container {
                function_captures, ..
            },
        ) if rest.is_empty() => {
            if function_captures.len() <= *index {
                function_captures.resize_with(*index + 1, Vec::new);
            }
            function_captures[*index].push((storage_name, capture_value));
        }
        (
            CompilerAggregatePathElement::MapValue(key),
            LlValue::Container {
                function_captures,
                function_capture_keys,
                ..
            },
        ) if rest.is_empty() => {
            let index = function_capture_keys
                .iter()
                .position(|candidate| candidate == key)
                .unwrap_or_else(|| {
                    function_capture_keys.push(key.clone());
                    function_captures.push(Vec::new());
                    function_capture_keys.len() - 1
                });
            function_captures[index].push((storage_name, capture_value));
        }
        (
            CompilerAggregatePathElement::OptionalPayload,
            LlValue::Optional {
                function_captures, ..
            },
        ) if rest.is_empty() => {
            function_captures.push((storage_name, capture_value));
        }
        (
            CompilerAggregatePathElement::ResultSuccess,
            LlValue::Result {
                function_captures, ..
            },
        ) if rest.is_empty() => {
            function_captures.push((storage_name, capture_value));
        }
        (CompilerAggregatePathElement::SumPayload(name), LlValue::Sum { payloads, sum, .. }) => {
            let index = sum
                .alternatives
                .iter()
                .position(|alternative| alternative.name == *name)
                .expect("checked capture path names a Sum alternative");
            let payload = payloads[index]
                .as_deref_mut()
                .expect("checked capture path names a payload-bearing Sum alternative");
            attach_function_capture(payload, rest, storage_name, capture_value);
        }
        _ => unreachable!("checked capture path follows its aggregate representation"),
    }
}

fn function_capture_value(value: &LlValue, storage_name: &str) -> Option<LlValue> {
    match value {
        LlValue::Function { captures, .. } => captures
            .iter()
            .find_map(|(name, value)| (name == storage_name).then(|| value.clone())),
        LlValue::Tuple(fields) => fields
            .iter()
            .find_map(|field| function_capture_value(field, storage_name)),
        LlValue::Record { fields, .. } => fields
            .iter()
            .find_map(|(_, field)| function_capture_value(field, storage_name)),
        LlValue::List {
            function_captures, ..
        }
        | LlValue::Container {
            function_captures, ..
        } => function_captures.iter().find_map(|captures| {
            captures
                .iter()
                .find_map(|(name, value)| (name == storage_name).then(|| value.clone()))
        }),
        LlValue::Optional {
            function_captures, ..
        } => function_captures
            .iter()
            .find_map(|(name, value)| (name == storage_name).then(|| value.clone())),
        LlValue::Result {
            function_captures, ..
        } => function_captures
            .iter()
            .find_map(|(name, value)| (name == storage_name).then(|| value.clone())),
        LlValue::Sum { payloads, .. } => payloads.iter().find_map(|payload| {
            payload
                .as_deref()
                .and_then(|payload| function_capture_value(payload, storage_name))
        }),
        _ => None,
    }
}

fn extend_function_capture_environment(
    environment: &mut BTreeMap<String, LlValue>,
    value: &LlValue,
) {
    match value {
        LlValue::Function { captures, .. } => {
            environment.extend(captures.iter().cloned());
        }
        LlValue::Tuple(fields) => {
            for field in fields {
                extend_function_capture_environment(environment, field);
            }
        }
        LlValue::Record { fields, .. } => {
            for (_, field) in fields {
                extend_function_capture_environment(environment, field);
            }
        }
        LlValue::List {
            function_captures, ..
        }
        | LlValue::Container {
            function_captures, ..
        } => {
            for captures in function_captures {
                environment.extend(captures.iter().cloned());
            }
        }
        LlValue::Optional {
            function_captures, ..
        }
        | LlValue::Result {
            function_captures, ..
        } => {
            environment.extend(function_captures.iter().cloned());
        }
        LlValue::Sum { payloads, .. } => {
            for payload in payloads.iter().flatten() {
                extend_function_capture_environment(environment, payload);
            }
        }
        _ => {}
    }
}

impl LlValue {
    fn singleton(&self) -> &str {
        match self {
            Self::Completed(value) | Self::Effect(value) => value,
            _ => unreachable!("checked value is a zero-data singleton"),
        }
    }

    fn integer(&self) -> &str {
        match self {
            Self::Int(value) | Self::Modular { value, .. } => value,
            _ => unreachable!("checked value has an integer representation"),
        }
    }

    fn modular_pointer(&self) -> &str {
        let Self::Modular { value, .. } = self else {
            unreachable!("checked value is modular")
        };
        value
    }

    fn boolean(&self) -> &str {
        let Self::Boolean(value) = self else {
            unreachable!("checked value is Boolean")
        };
        value
    }

    fn rational(&self) -> &str {
        let Self::Rational(value) = self else {
            unreachable!("checked value is Rational")
        };
        value
    }

    fn comparison(&self) -> &str {
        let Self::Comparison(value) = self else {
            unreachable!("checked value is Comparison")
        };
        value
    }

    fn error(&self) -> &str {
        let Self::Error(value) = self else {
            unreachable!("checked value is Error")
        };
        value
    }

    fn error_code(&self) -> &str {
        let Self::ErrorCode(value) = self else {
            unreachable!("checked value is ErrorCode")
        };
        value
    }

    fn error_domain(&self) -> &str {
        let Self::ErrorDomain(value) = self else {
            unreachable!("checked value is ErrorDomain")
        };
        value
    }

    fn source_location(&self) -> &str {
        let Self::SourceLocation(value) = self else {
            unreachable!("checked value is SourceLocation")
        };
        value
    }

    fn enumeration(&self) -> &str {
        match self {
            Self::Enum { value, .. } | Self::Function { value, .. } => value,
            _ => unreachable!("checked value has an enum-like observation tag"),
        }
    }

    fn generator_token(&self) -> &str {
        let Self::Generator { value, .. } = self else {
            unreachable!("checked value is a construction-only Generator")
        };
        value
    }

    fn generator_initial(&self) -> &Self {
        let Self::Generator {
            captured_initial: Some(initial),
            ..
        } = self
        else {
            unreachable!("checked custom value Generator retains its initial value")
        };
        initial
    }

    fn generator_additional_initials(&self) -> &[Self] {
        let Self::Generator {
            captured_additional_initials,
            ..
        } = self
        else {
            unreachable!("checked custom value Generator retains its captured inputs")
        };
        captured_additional_initials
    }

    fn string(&self) -> &str {
        let Self::String(value) = self else {
            unreachable!("checked value is String")
        };
        value
    }

    fn int_or_string_pointer(&self) -> &str {
        match self {
            Self::Int(value) | Self::String(value) => value,
            _ => unreachable!("checked List pair field is Int or String"),
        }
    }

    fn aggregate_pointer(&self) -> &str {
        match self {
            Self::Int(value)
            | Self::Rational(value)
            | Self::String(value)
            | Self::List { value, .. } => value,
            _ => unreachable!("checked aggregate field has a pointer representation"),
        }
    }

    fn range(&self) -> (&str, &CompilerType) {
        let Self::Range { value, endpoint } = self else {
            unreachable!("checked value is Range")
        };
        (value, endpoint)
    }

    fn range_pointer(&self) -> &str {
        self.range().0
    }

    fn result_pointer(&self) -> &str {
        let Self::Result { value, .. } = self else {
            unreachable!("checked value is Result")
        };
        value
    }

    fn optional_pointer(&self) -> &str {
        let Self::Optional { value, .. } = self else {
            unreachable!("checked value is Optional")
        };
        value
    }

    fn traversal_control(&self) -> (&str, &CompilerType) {
        let Self::TraversalControl { value, payload } = self else {
            unreachable!("checked value is TraversalControl")
        };
        (value, payload)
    }

    fn traversal_control_pointer(&self) -> &str {
        self.traversal_control().0
    }

    fn list_pointer(&self) -> &str {
        let Self::List { value, .. } = self else {
            unreachable!("checked value is List")
        };
        value
    }

    fn container_pointer(&self) -> &str {
        let Self::Container { value, .. } = self else {
            unreachable!("checked value is a fundamental container")
        };
        value
    }

    fn serialization_stream_pointer(&self) -> &str {
        let Self::SerializationStream { stream, .. } = self else {
            unreachable!("checked value is a SerializationStream")
        };
        stream
    }

    fn task_pointer(&self) -> &str {
        let Self::Task { value, .. } = self else {
            unreachable!("checked value is a Task instance")
        };
        value
    }

    fn location_pointer(&self) -> &str {
        let Self::ExternalLocation { value, .. } = self else {
            unreachable!("checked value is an external Location")
        };
        value
    }

    fn serialization_expected_pointer(&self) -> &str {
        let Self::SerializationStream { expected, .. } = self else {
            unreachable!("checked value is a SerializationStream")
        };
        expected
    }

    fn serialization_byte_count(&self) -> &str {
        let Self::SerializationStream { byte_count, .. } = self else {
            unreachable!("checked value is a SerializationStream")
        };
        byte_count
    }

    fn argument(&self) -> String {
        match self {
            Self::Unit => "i8 0".into(),
            Self::StaticDisplay(_) => {
                unreachable!("static Capability values are not call arguments")
            }
            Self::Completed(value) | Self::Effect(value) => format!("i8 {value}"),
            Self::Boolean(value) => format!("i1 {value}"),
            Self::Version { value, .. }
            | Self::SerializationStream { stream: value, .. }
            | Self::Task { value, .. }
            | Self::ExternalLocation { value, .. }
            | Self::Int(value)
            | Self::Modular { value, .. }
            | Self::Rational(value)
            | Self::Error(value)
            | Self::ErrorDomain(value)
            | Self::SourceLocation(value)
            | Self::String(value)
            | Self::Range { value, .. }
            | Self::Result { value, .. }
            | Self::Optional { value, .. }
            | Self::TraversalControl { value, .. }
            | Self::List { value, .. }
            | Self::Container { value, .. } => {
                format!("ptr {value}")
            }
            Self::Comparison(value)
            | Self::ErrorCode(value)
            | Self::Enum { value, .. }
            | Self::Function { value, .. }
            | Self::Generator { value, .. } => {
                format!("i32 {value}")
            }
            Self::Tuple(_) | Self::Record { .. } | Self::Sum { .. } => {
                unreachable!("checked call arguments are scalar")
            }
        }
    }
}

#[allow(clippy::too_many_lines)] // Every admitted representation has an explicit neutral machine value.
fn zero_machine_value(value_type: &CompilerType) -> LlValue {
    match value_type {
        CompilerType::Unit => LlValue::Unit,
        CompilerType::Completed => LlValue::Completed("0".into()),
        CompilerType::Effect => LlValue::Effect("0".into()),
        CompilerType::Boolean => LlValue::Boolean("false".into()),
        CompilerType::Version => {
            unreachable!("Version sum payloads are not admitted")
        }
        CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat => LlValue::Int("null".into()),
        CompilerType::Modular(modular) => LlValue::Modular {
            value: "null".into(),
            modular: modular.clone(),
        },
        CompilerType::Rational | CompilerType::InfiniteRational => LlValue::Rational("null".into()),
        CompilerType::Comparison => LlValue::Comparison("0".into()),
        CompilerType::Error => LlValue::Error("null".into()),
        CompilerType::ErrorCode => LlValue::ErrorCode("0".into()),
        CompilerType::ErrorDomain => LlValue::ErrorDomain("null".into()),
        CompilerType::SourceLocation => LlValue::SourceLocation("null".into()),
        CompilerType::Enum(enumeration) => LlValue::Enum {
            value: "0".into(),
            enumeration: enumeration.clone(),
        },
        CompilerType::Generator(generator) => LlValue::Generator {
            value: "0".into(),
            generator: generator.clone(),
            captured_initial: None,
            captured_additional_initials: Vec::new(),
        },
        CompilerType::Task(task) => LlValue::Task {
            value: "null".into(),
            task: task.as_ref().clone(),
        },
        CompilerType::ExternalLocation(location) => LlValue::ExternalLocation {
            value: "null".into(),
            location: location.as_ref().clone(),
        },
        CompilerType::Range(endpoint) => LlValue::Range {
            value: "null".into(),
            endpoint: endpoint.as_ref().clone(),
        },
        CompilerType::Result(success) | CompilerType::TaskResponse(success) => LlValue::Result {
            value: "null".into(),
            success: success.as_ref().clone(),
            function_captures: Vec::new(),
        },
        CompilerType::Optional(payload) => LlValue::Optional {
            value: "null".into(),
            payload: payload.as_ref().clone(),
            function_captures: Vec::new(),
        },
        CompilerType::TraversalControl(payload) => LlValue::TraversalControl {
            value: "null".into(),
            payload: payload.as_ref().clone(),
        },
        CompilerType::List(element) => LlValue::List {
            value: "null".into(),
            element: element.as_ref().clone(),
            function_captures: Vec::new(),
        },
        value_type @ (CompilerType::Array { .. }
        | CompilerType::Set(_)
        | CompilerType::Bag(_)
        | CompilerType::Map { .. }) => LlValue::Container {
            value: "null".into(),
            value_type: value_type.clone(),
            function_captures: Vec::new(),
            function_capture_keys: Vec::new(),
        },
        CompilerType::Character | CompilerType::String => LlValue::String("null".into()),
        CompilerType::Refined { base, .. } => zero_machine_value(base),
        CompilerType::Tuple(fields) => {
            LlValue::Tuple(fields.iter().map(zero_machine_value).collect())
        }
        CompilerType::Record(fields) => LlValue::Record {
            fields: fields
                .iter()
                .map(|(label, field)| (label.clone(), zero_machine_value(field)))
                .collect(),
            order: (0..fields.len()).map(|index| index.to_string()).collect(),
        },
        CompilerType::Sum(sum) => LlValue::Sum {
            tag: "0".into(),
            payloads: sum
                .alternatives
                .iter()
                .map(|alternative| {
                    alternative
                        .payload
                        .as_ref()
                        .map(|payload| Box::new(zero_machine_value(payload)))
                })
                .collect(),
            sum: sum.clone(),
        },
        CompilerType::Type | CompilerType::Scope => LlValue::Enum {
            value: "0".into(),
            enumeration: if value_type == &CompilerType::Type {
                fundamental_type_enumeration()
            } else {
                scope_enumeration()
            },
        },
        CompilerType::Function => LlValue::Function {
            value: "0".into(),
            enumeration: CompilerEnumType {
                name: "Function".into(),
                alternatives: Vec::new(),
            },
            captures: Vec::new(),
        },
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata
        | CompilerType::SerializationStream(_)
        | CompilerType::Constraint => {
            unreachable!("static object values are not admitted in sum payloads")
        }
    }
}

fn optional_payload_pointer(value: &LlValue) -> &str {
    match value {
        LlValue::Int(value)
        | LlValue::Rational(value)
        | LlValue::Error(value)
        | LlValue::SourceLocation(value)
        | LlValue::List { value, .. }
        | LlValue::String(value) => value,
        _ => unreachable!("checked Optional payload has a pointer representation"),
    }
}

fn optional_payload_value(value: String, value_type: &CompilerType) -> LlValue {
    match value_type {
        CompilerType::Int | CompilerType::Nat => LlValue::Int(value),
        CompilerType::Rational => LlValue::Rational(value),
        CompilerType::Character | CompilerType::String => LlValue::String(value),
        CompilerType::Error => LlValue::Error(value),
        CompilerType::SourceLocation => LlValue::SourceLocation(value),
        CompilerType::List(element) => LlValue::List {
            value,
            element: element.as_ref().clone(),
            function_captures: Vec::new(),
        },
        _ => unreachable!("checked Optional payload type is supported: {value_type:?}"),
    }
}

fn numeric_pointer(value: &LlValue) -> (&str, CompilerType) {
    match value {
        LlValue::Int(value) => (value, CompilerType::Int),
        LlValue::Rational(value) => (value, CompilerType::Rational),
        _ => unreachable!("checked value is finite exact numeric"),
    }
}

fn fundamental_type_enumeration() -> CompilerEnumType {
    CompilerEnumType {
        name: "Type".into(),
        alternatives: [
            "Boolean", "Int", "Nat", "Rational", "String", "Unit", "Scope",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    }
}

fn scope_enumeration() -> CompilerEnumType {
    CompilerEnumType {
        name: "Scope".into(),
        alternatives: vec!["<namespace root>".into(), "<namespace lang lint>".into()],
    }
}

fn generator_debug_enumeration(generator: &CompilerGeneratorType) -> CompilerEnumType {
    let name = CompilerType::Generator(generator.clone()).name();
    CompilerEnumType {
        alternatives: vec![format!("<{name}>")],
        name,
    }
}

fn function_value_enumeration(program: &CompilerProgram) -> CompilerEnumType {
    CompilerEnumType {
        name: "Function".into(),
        alternatives: program.function_value_names.clone(),
    }
}

fn constraint_value_enumeration(program: &CompilerProgram) -> CompilerEnumType {
    CompilerEnumType {
        name: "Constraint".into(),
        alternatives: program
            .constraints
            .iter()
            .map(|constraint| format!("<Constraint {}>", constraint.name))
            .collect(),
    }
}

fn numeric_domain(value_type: &CompilerType) -> &'static str {
    match value_type {
        CompilerType::Int => "int",
        CompilerType::Rational => "rational",
        _ => unreachable!("checked Range endpoint is finite exact numeric"),
    }
}

struct FunctionBody {
    lines: Vec<String>,
    next_value: usize,
    next_label: usize,
    current_block: String,
    subprogram: usize,
}

impl FunctionBody {
    fn new(subprogram: usize) -> Self {
        Self {
            lines: vec!["entry:".into()],
            next_value: 0,
            next_label: 0,
            current_block: "entry".into(),
            subprogram,
        }
    }

    fn instruction(&mut self, instruction: &str, span: Span, debug: &mut DebugInfo) -> String {
        let value = self.reserve_value();
        self.define_reserved(&value, instruction, span, debug);
        value
    }

    fn instruction_without_debug(&mut self, instruction: &str) -> String {
        let value = self.reserve_value();
        self.lines.push(format!("  {value} = {instruction}"));
        value
    }

    fn reserve_value(&mut self) -> String {
        let value = format!("%v{}", self.next_value);
        self.next_value += 1;
        value
    }

    fn define_reserved(
        &mut self,
        value: &str,
        instruction: &str,
        span: Span,
        debug: &mut DebugInfo,
    ) {
        let location = debug.location(span, self.subprogram);
        self.lines
            .push(format!("  {value} = {instruction}, !dbg !{location}"));
    }

    fn effect(&mut self, instruction: &str, span: Span, debug: &mut DebugInfo) {
        let location = debug.location(span, self.subprogram);
        self.lines
            .push(format!("  {instruction}, !dbg !{location}"));
    }

    fn effect_without_debug(&mut self, instruction: &str) {
        self.lines.push(format!("  {instruction}"));
    }

    fn named_instruction(
        &mut self,
        name: &str,
        instruction: &str,
        span: Span,
        debug: &mut DebugInfo,
    ) {
        let location = debug.location(span, self.subprogram);
        self.lines
            .push(format!("  {name} = {instruction}, !dbg !{location}"));
    }

    fn terminator(&mut self, instruction: &str, location: usize) {
        self.lines
            .push(format!("  {instruction}, !dbg !{location}"));
    }

    fn debug_value(&mut self, value: &LlValue, variable: usize, location: usize) {
        let value = match value {
            LlValue::Unit => "i8 0".into(),
            LlValue::Completed(value) | LlValue::Effect(value) => format!("i8 {value}"),
            LlValue::Boolean(value) => format!("i1 {value}"),
            LlValue::Version { value, .. }
            | LlValue::SerializationStream { stream: value, .. }
            | LlValue::Task { value, .. }
            | LlValue::ExternalLocation { value, .. }
            | LlValue::Int(value)
            | LlValue::Modular { value, .. }
            | LlValue::Rational(value)
            | LlValue::Error(value)
            | LlValue::ErrorDomain(value)
            | LlValue::SourceLocation(value)
            | LlValue::String(value)
            | LlValue::Range { value, .. }
            | LlValue::Result { value, .. }
            | LlValue::Optional { value, .. }
            | LlValue::TraversalControl { value, .. }
            | LlValue::List { value, .. }
            | LlValue::Container { value, .. } => {
                format!("ptr {value}")
            }
            LlValue::Comparison(value)
            | LlValue::ErrorCode(value)
            | LlValue::Enum { value, .. }
            | LlValue::Function { value, .. }
            | LlValue::Generator { value, .. } => format!("i32 {value}"),
            LlValue::StaticDisplay(_)
            | LlValue::Tuple(_)
            | LlValue::Record { .. }
            | LlValue::Sum { .. } => return,
        };
        self.debug_value_operand(&value, variable, location);
    }

    fn debug_value_operand(&mut self, value: &str, variable: usize, location: usize) {
        self.lines.push(format!(
            "    #dbg_value({value}, !{variable}, !DIExpression(), !{location})"
        ));
    }

    fn debug_declare(&mut self, address: &str, variable: usize, location: usize) {
        self.lines.push(format!(
            "    #dbg_declare(ptr {address}, !{variable}, !DIExpression(), !{location})"
        ));
    }

    fn label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}.{}", self.next_label);
        self.next_label += 1;
        label
    }

    fn start_block(&mut self, label: &str) {
        self.lines.push(format!("{label}:"));
        self.current_block = label.into();
    }

    fn render(&self) -> String {
        self.lines.join("\n")
    }
}

struct DebugInfo {
    nodes: Vec<String>,
    file: usize,
    compile_unit: usize,
    empty: usize,
    unsigned64_type: usize,
    int_type: usize,
    nat_type: usize,
    version_type: usize,
    serialization_stream_type: usize,
    rational_type: usize,
    character_type: usize,
    string_type: usize,
    error_type: usize,
    generator_error_type: usize,
    error_code_type: usize,
    error_domain_type: usize,
    source_location_type: usize,
    int_range_type: usize,
    rational_range_type: usize,
    result_int_type: usize,
    result_nat_type: usize,
    result_rational_type: usize,
    result_string_type: usize,
    result_int_pair_type: usize,
    result_unit_generator_error_type: usize,
    result_header_pointer_type: usize,
    result_types: Vec<(CompilerType, usize)>,
    result_modular_types: Vec<(CompilerModularType, usize)>,
    optional_int_type: usize,
    optional_rational_type: usize,
    optional_character_type: usize,
    optional_string_type: usize,
    optional_error_type: usize,
    optional_source_location_type: usize,
    optional_header_pointer_type: usize,
    optional_types: Vec<(CompilerType, usize)>,
    traversal_control_types: Vec<(CompilerType, usize)>,
    comparison_type: usize,
    boolean_type: usize,
    unit_type: usize,
    completed_type: usize,
    effect_type: usize,
    enum_types: BTreeMap<String, usize>,
    modular_types: Vec<(CompilerModularType, usize)>,
    list_types: Vec<(CompilerType, usize)>,
    container_types: Vec<(CompilerType, usize)>,
    refined_types: Vec<(CompilerType, usize)>,
    tuple_types: Vec<(CompilerType, usize)>,
    record_types: Vec<(CompilerType, usize)>,
    sum_types: Vec<(CompilerType, usize)>,
    task_types: Vec<(CompilerTaskType, usize)>,
    location_types: Vec<(CompilerLocationType, usize)>,
    source: topal_source::SourceText,
    filename: String,
}
