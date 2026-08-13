use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write as _};

use num_bigint::BigInt;
use num_rational::BigRational;
use topal_source::{
    SourceText, Span, canonically_equal, case_fold, character_at, character_count, characters,
    lowercase, normalize_nfc, normalize_nfd, uppercase,
};
use topal_syntax::{
    CallableKind, DecisionMatcher, Expression, FunctionParameter, Statement, lex, parse,
};

use crate::{ExecutionSnapshot, TraceEvent, TraceSink};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Type(String),
    Effects(Vec<String>),
    Boolean(bool),
    Int(BigInt),
    Rational(BigRational),
    IntRange {
        lower: BigInt,
        upper: BigInt,
    },
    RationalRange {
        lower: BigRational,
        upper: BigRational,
    },
    Optional {
        payload_classifier: String,
        payload: Option<Box<Self>>,
    },
    List {
        element_classifier: String,
        entries: Vec<Self>,
    },
    Callable(CallableKind),
    NamedFunction(Box<NamedFunction>),
    Namespace(Box<NamespaceValue>),
    AnonymousFunction(Box<AnonymousFunction>),
    Array {
        element_classifier: String,
        entries: Vec<Self>,
    },
    Set {
        element_classifier: String,
        entries: Vec<Self>,
    },
    Bag {
        element_classifier: String,
        entries: Vec<(Self, usize)>,
    },
    Map {
        key_classifier: String,
        value_classifier: String,
        entries: Vec<(Self, Self)>,
    },
    CharacterGenerator {
        generated: Vec<String>,
        origin: String,
    },
    CharacterReturningGenerator {
        generated: Vec<String>,
        returned: String,
        origin: String,
    },
    IterateGenerator {
        current: Box<Self>,
        next: Box<Self>,
        take_while: Option<Box<Self>>,
        classifier: String,
    },
    UnfoldGenerator {
        seed: Box<Self>,
        step: Box<Self>,
    },
    SuspendedGenerator {
        source: SourceText,
        body: Vec<Statement>,
        cursor: usize,
        bindings: BTreeMap<String, Self>,
        scope_state: Box<GeneratorScopeState>,
        pending_yield: Option<Box<Self>>,
        resume_binding: Option<String>,
        returned: Option<Box<Self>>,
        yield_classifier: String,
        return_classifier: String,
        origin: String,
    },
    String(String),
    Tuple(Vec<Self>),
    Record(Vec<(String, Self)>),
    Enum {
        type_name: String,
        alternative: String,
    },
    Union(Box<UnionValue>),
    Constraint(Box<ConstraintValue>),
    Refined {
        constraint: String,
        base_classifier: String,
        value: Box<Self>,
    },
    ModularType(Box<ModularType>),
    Modular {
        type_name: String,
        lower: BigInt,
        upper: BigInt,
        value: BigInt,
    },
    ErrorDomain(String),
    Error {
        domain: String,
        code: String,
        line: usize,
        column: usize,
    },
    Continue(Box<Self>),
    Finish(Box<Self>),
    Completed,
    Unit,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[allow(clippy::box_collection)] // Keep recursive evaluator state below the tested stack-frame ceiling.
pub struct GeneratorScopeState {
    functions: BTreeMap<String, Vec<UserFunction>>,
    declared_names: BTreeSet<String>,
    local_function_names: BTreeSet<String>,
    enum_types: BTreeMap<String, BTreeSet<String>>,
    union_types: Box<BTreeMap<String, BTreeMap<String, Option<String>>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousFunction {
    source: SourceText,
    parameters: Vec<String>,
    body: Box<Expression>,
    bindings: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedFunction {
    name: String,
    candidates: Vec<UserFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceValue {
    name: String,
    bindings: BTreeMap<String, Value>,
    functions: BTreeMap<String, Vec<UserFunction>>,
    generators: BTreeMap<String, Vec<UserGenerator>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnionValue {
    type_name: String,
    alternative: String,
    payload_classifier: Option<String>,
    payload: Option<Box<Value>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstraintValue {
    name: Option<String>,
    base_classifier: String,
    predicate: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModularType {
    name: Option<String>,
    signed: bool,
    lower: BigInt,
    upper: BigInt,
}

impl fmt::Display for Value {
    #[allow(clippy::too_many_lines)] // Every runtime value keeps an explicit stable source representation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(name) => formatter.write_str(name),
            Self::Effects(effects) => write!(formatter, "Effects ({})", effects.join(", ")),
            Self::Boolean(value) => value.fmt(formatter),
            Self::Int(value) => value.fmt(formatter),
            Self::Rational(value) => {
                write!(
                    formatter,
                    "Rational ( {}, {} )",
                    value.numer(),
                    value.denom()
                )
            }
            Self::IntRange { lower, upper } => write!(formatter, "{lower} .. {upper}"),
            Self::RationalRange { lower, upper } => write!(
                formatter,
                "Rational ( {}, {} ) .. Rational ( {}, {} )",
                lower.numer(),
                lower.denom(),
                upper.numer(),
                upper.denom()
            ),
            Self::Optional {
                payload: Some(value),
                ..
            } => write!(formatter, "Some {value}"),
            Self::Optional { payload: None, .. } => formatter.write_str("None"),
            Self::List { entries, .. } => {
                for entry in entries {
                    write!(formatter, "Entry ( {entry}, ")?;
                }
                formatter.write_str("Empty")?;
                for _ in entries {
                    formatter.write_str(" )")?;
                }
                Ok(())
            }
            Self::Callable(kind) => formatter.write_str(callable_name(*kind)),
            Self::NamedFunction(function) => write!(formatter, "<fn {}>", function.name),
            Self::Namespace(namespace) => write!(formatter, "<namespace {}>", namespace.name),
            Self::AnonymousFunction(function) => {
                write!(formatter, "<anonymous fn/{}>", function.parameters.len())
            }
            Self::Array { entries, .. } => display_collection(formatter, "Array", entries),
            Self::Set { entries, .. } => display_collection(formatter, "Set", entries),
            Self::Bag { entries, .. } => {
                formatter.write_str("Bag (")?;
                for (index, (value, count)) in entries.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "({value}, {count})")?;
                }
                formatter.write_str(")")
            }
            Self::Map { entries, .. } => {
                formatter.write_str("Map (")?;
                for (index, (key, value)) in entries.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "({key}, {value})")?;
                }
                formatter.write_str(")")
            }
            Self::CharacterGenerator { .. } => {
                formatter.write_str("<Generator Character Unit Unit>")
            }
            Self::CharacterReturningGenerator { .. } => {
                formatter.write_str("<Generator Character Unit Character>")
            }
            Self::IterateGenerator { classifier, .. } => {
                write!(formatter, "<Generator {classifier} Unit Unit>")
            }
            Self::UnfoldGenerator { .. } => formatter.write_str("<Generator Value Unit Unit>"),
            Self::SuspendedGenerator {
                yield_classifier,
                return_classifier,
                ..
            } => write!(
                formatter,
                "<Generator {yield_classifier} Unit {return_classifier}>"
            ),
            Self::String(value) => formatter.write_str(&display_string(value)),
            Self::Tuple(items) => {
                formatter.write_str("(")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    item.fmt(formatter)?;
                }
                if items.len() == 1 {
                    formatter.write_str(",")?;
                }
                formatter.write_str(")")
            }
            Self::Record(fields) => {
                formatter.write_str("(")?;
                for (index, (label, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{label} is {value}")?;
                }
                formatter.write_str(")")
            }
            Self::Enum { alternative, .. } => formatter.write_str(alternative),
            Self::Union(union) if union.payload.is_some() => write!(
                formatter,
                "{} {}",
                union.alternative,
                union.payload.as_deref().expect("present payload")
            ),
            Self::Union(union) => formatter.write_str(&union.alternative),
            Self::Constraint(constraint) => write!(
                formatter,
                "<Constraint {}>",
                constraint
                    .name
                    .as_deref()
                    .unwrap_or(&constraint.base_classifier)
            ),
            Self::Refined { value, .. } => write!(formatter, "{value}"),
            Self::ModularType(kind) => write!(
                formatter,
                "<{} {} .. {}>",
                if kind.signed { "ModInt" } else { "ModNat" },
                kind.lower,
                kind.upper
            ),
            Self::Modular {
                type_name, value, ..
            } => write!(formatter, "{type_name} {value}"),
            Self::ErrorDomain(domain) => formatter.write_str(domain),
            Self::Error { domain, code, .. } => {
                write!(formatter, "Error ( domain is {domain}, code is {code} )")
            }
            Self::Continue(value) => write!(formatter, "Continue {value}"),
            Self::Finish(value) => write!(formatter, "Finish {value}"),
            Self::Completed => formatter.write_str("Completed"),
            Self::Unit => formatter.write_str("()"),
        }
    }
}

fn display_collection(
    formatter: &mut fmt::Formatter<'_>,
    kind: &str,
    entries: &[Value],
) -> fmt::Result {
    write!(formatter, "{kind} (")?;
    for (index, entry) in entries.iter().enumerate() {
        if index != 0 {
            formatter.write_str(", ")?;
        }
        write!(formatter, "{entry}")?;
    }
    formatter.write_str(")")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub line: usize,
    pub column: usize,
    pub message: String,
    source_line: Option<String>,
    marker_width: usize,
    help: Option<String>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.render("<input>"))
    }
}

impl std::error::Error for Diagnostic {}

impl Diagnostic {
    #[must_use]
    pub fn render(&self, source_name: &str) -> String {
        let mut rendered = format!(
            "error[{}]: {}\n --> {source_name}:{}:{}",
            self.code, self.message, self.line, self.column
        );
        if let Some(source_line) = &self.source_line {
            let gutter_width = self.line.to_string().len();
            let _ = write!(
                rendered,
                "\n{empty:>gutter_width$} |\n{line:>gutter_width$} | {source_line}\n{empty:>gutter_width$} | {padding}{markers}",
                empty = "",
                line = self.line,
                padding = " ".repeat(self.column.saturating_sub(1)),
                markers = "^".repeat(self.marker_width.max(1)),
            );
        }
        if let Some(help) = &self.help {
            let _ = write!(
                rendered,
                "\n{empty:>width$} |\n{empty:>width$} = help: {help}",
                empty = "",
                width = self.line.to_string().len()
            );
        }
        rendered
    }

    fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

#[derive(Clone, Default)]
#[allow(clippy::box_collection)] // Keep recursive evaluator state below the tested stack-frame ceiling.
pub struct Session {
    bindings: BTreeMap<String, Value>,
    functions: BTreeMap<String, Vec<UserFunction>>,
    generators: BTreeMap<String, Vec<UserGenerator>>,
    declared_names: BTreeSet<String>,
    consumed_names: BTreeSet<String>,
    local_function_names: BTreeSet<String>,
    enum_types: BTreeMap<String, BTreeSet<String>>,
    union_types: Box<BTreeMap<String, BTreeMap<String, Option<String>>>>,
    call_stack: Vec<ActiveCall>,
    static_context: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UserFunction {
    source: SourceText,
    is_static: bool,
    parameters: Vec<(String, String)>,
    result: String,
    body: Vec<Statement>,
    bindings: BTreeMap<String, Value>,
    termination_rule: Option<&'static str>,
    recursion_target: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UserGenerator {
    source: SourceText,
    parameters: Vec<(String, String)>,
    yielded: String,
    result: String,
    body: Vec<Statement>,
    bindings: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveCall {
    name: String,
    signature: String,
    termination_rule: Option<&'static str>,
    recursion_target: Option<String>,
}

pub struct Execution {
    source: SourceText,
    statements: Vec<Statement>,
    cursor: usize,
    return_classifier: Option<String>,
}

#[derive(Clone, Copy)]
struct FunctionDeclaration<'a> {
    name: Span,
    is_static: bool,
    parameters: &'a [FunctionParameter],
    result: Span,
    body: &'a [Statement],
    span: Span,
}

#[derive(Clone, Copy)]
struct GeneratorDeclaration<'a> {
    name: Span,
    parameters: &'a [FunctionParameter],
    yielded: Span,
    resumed: Span,
    result: Span,
    body: &'a [Statement],
    span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionStep {
    Advanced { value: Value, span: Span },
    Complete(Value),
    Returned { value: Value, span: Span },
}

enum BindingOutcome {
    Bound(Value, Span),
    Returned(Value, Span),
}

impl Session {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Report whether a complete block statement should await a dedented line
    /// before an interactive session submits it.
    #[must_use]
    pub fn awaits_dedent(input: &str) -> bool {
        let Ok(source) = SourceText::new(input) else {
            return false;
        };
        let parsed = parse(&source, &lex(&source));
        parsed.diagnostics.is_empty()
            && matches!(
                parsed.statements.as_slice(),
                [Statement::Function { .. }
                    | Statement::Generator { .. }
                    | Statement::Union { .. }
                    | Statement::Foreach { .. }]
            )
    }

    /// Evaluate one source unit and return its final value.
    ///
    /// # Errors
    ///
    /// Returns a source, syntax, name-resolution, or evaluation diagnostic.
    pub fn evaluate(
        &mut self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut execution = self.prepare(input, trace)?;
        loop {
            match execution.step(self, trace)? {
                ExecutionStep::Complete(value) => return Ok(value),
                ExecutionStep::Advanced { .. } => {}
                ExecutionStep::Returned { .. } => {
                    unreachable!("top-level return is rejected before completing a step")
                }
            }
        }
    }

    /// Prepare a source unit for resumable execution.
    ///
    /// # Errors
    ///
    /// Returns a source or syntax diagnostic before any statement executes.
    pub fn prepare(
        &self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Execution, Diagnostic> {
        self.checkpoint(trace, None, None);
        trace.record(TraceEvent {
            event: "context.selected",
            rule: "TOPAL-SYN-UNICODE-001",
            detail: "design-0;Unicode=17.0.0",
        });
        let source = accepted_source(input, trace)?;
        let parsed = parse(&source, &lex(&source));
        if let Some(error) = parsed.diagnostics.first() {
            return Err(diagnostic(&source, error.code, error.span, &error.message));
        }
        if parsed.statements.is_empty() {
            return Err(expected_statement(input));
        }
        Ok(Execution {
            source,
            statements: parsed.statements,
            cursor: 0,
            return_classifier: None,
        })
    }

    /// Evaluate one expression against an immutable binding snapshot.
    ///
    /// # Errors
    ///
    /// Returns a diagnostic when the input is not exactly one expression or
    /// when that expression cannot be evaluated.
    pub fn inspect(
        bindings: &BTreeMap<String, Value>,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut session = Self {
            bindings: bindings.clone(),
            functions: BTreeMap::new(),
            generators: BTreeMap::new(),
            declared_names: bindings.keys().cloned().collect(),
            consumed_names: BTreeSet::new(),
            local_function_names: BTreeSet::new(),
            enum_types: BTreeMap::new(),
            union_types: Box::new(BTreeMap::new()),
            call_stack: Vec::new(),
            static_context: false,
        };
        let mut execution = session.prepare(input, trace)?;
        if !matches!(execution.statements.as_slice(), [Statement::Expression(_)]) {
            let span = execution
                .statements
                .first()
                .map_or_else(|| Span::new(0, 0), statement_span);
            return Err(diagnostic(
                &execution.source,
                "D-EXPECTED-EXPRESSION",
                span,
                "debugger inspection requires exactly one expression",
            ));
        }
        match execution.step(&mut session, trace)? {
            ExecutionStep::Complete(value) => Ok(value),
            ExecutionStep::Advanced { .. } => unreachable!("one expression completes execution"),
            ExecutionStep::Returned { .. } => unreachable!("inspection rejects return statements"),
        }
    }

    fn checkpoint(&self, trace: &mut impl TraceSink, value: Option<&Value>, span: Option<Span>) {
        trace.checkpoint(ExecutionSnapshot {
            bindings: &self.bindings,
            value,
            span,
        });
    }

    #[allow(clippy::too_many_lines)] // Keep recursive expression cases together and auditable.
    fn evaluate_expression(
        &self,
        source: &SourceText,
        expression: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let value = match expression {
            Expression::Boolean(span) => Ok(evaluate_boolean_literal(source, *span, trace)),
            Expression::Unit(_) => {
                trace.record(TraceEvent {
                    event: "product.unit",
                    rule: "TOPAL-TYPE-PRODUCT-001",
                    detail: "Tuple()",
                });
                Ok(Value::Unit)
            }
            Expression::Product { fields, span } => {
                self.evaluate_product(source, fields, *span, trace)
            }
            Expression::DecisionTable {
                subject,
                rules,
                span,
            } => {
                let subject_span = subject.span();
                let subject = self.evaluate_expression(source, subject, trace)?;
                let enum_matchers = rules
                    .iter()
                    .filter_map(|rule| match rule.matcher {
                        DecisionMatcher::Identifier(span) => Some(source.slice(span).to_owned()),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                let has_result_matchers = rules.iter().any(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::Result { .. } | DecisionMatcher::ErrorCode { .. }
                    )
                });
                let has_optional_matchers = rules
                    .iter()
                    .any(|rule| matches!(rule.matcher, DecisionMatcher::Optional { .. }));
                let has_list_matchers = rules.iter().any(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::ListEmpty(_) | DecisionMatcher::ListEntry { .. }
                    )
                });
                let has_union_matchers = rules.iter().any(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::Union { .. } | DecisionMatcher::Variant { .. }
                    )
                });
                if !enum_matchers.is_empty()
                    && !has_union_matchers
                    && !rules
                        .iter()
                        .any(|rule| matches!(rule.matcher, DecisionMatcher::Otherwise(_)))
                {
                    let Value::Enum { type_name, .. } = &subject else {
                        return Err(diagnostic(
                            source,
                            "E-DECISION-SUBJECT-TYPE",
                            subject_span,
                            "enum alternative matchers require an Enum subject",
                        ));
                    };
                    if known_enum_alternatives(self, type_name).as_ref() != Some(&enum_matchers) {
                        return Err(diagnostic(
                            source,
                            "E-INCOMPLETE-DECISION",
                            *span,
                            format!("decision does not cover every `{type_name}` alternative"),
                        ));
                    }
                }
                let decision_rule = if has_union_matchers {
                    "TOPAL-DECISION-UNION-001"
                } else if has_list_matchers {
                    "TOPAL-DECISION-LIST-001"
                } else if has_optional_matchers {
                    "TOPAL-DECISION-OPTIONAL-001"
                } else if has_result_matchers {
                    "TOPAL-DECISION-RESULT-001"
                } else if !enum_matchers.is_empty() {
                    "TOPAL-DECISION-ENUM-001"
                } else if rules
                    .iter()
                    .any(|rule| matches!(&rule.matcher, DecisionMatcher::Comparison { .. }))
                {
                    "TOPAL-DECISION-COMPARISON-001"
                } else {
                    "TOPAL-DECISION-BOOLEAN-001"
                };
                let mut selected = None;
                for (index, rule) in rules.iter().enumerate() {
                    let matches = match &rule.matcher {
                        DecisionMatcher::Boolean { value, .. } => {
                            let Value::Boolean(subject) = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "Boolean literal matchers require a Boolean subject",
                                ));
                            };
                            *value == *subject
                        }
                        DecisionMatcher::Identifier(matcher) => {
                            let name = source.slice(*matcher);
                            if let Value::Enum {
                                type_name,
                                alternative,
                            } = &subject
                                && type_name == "Comparison"
                                && matches!(name, "Less" | "Equal" | "Greater")
                            {
                                alternative == name
                            } else {
                                let Some(candidate) = self.bindings.get(name).cloned() else {
                                    return Err(diagnostic(
                                        source,
                                        "E-UNBOUND-NAME",
                                        *matcher,
                                        format!("enum matcher `{name}` is not declared"),
                                    ));
                                };
                                values_equal(subject.clone(), candidate, trace).unwrap_or(false)
                            }
                        }
                        DecisionMatcher::Union { alternative, .. } => {
                            matches!(
                                &subject,
                                Value::Union(union)
                                    if union.payload.is_some()
                                        && union.alternative == source.slice(*alternative)
                            )
                        }
                        DecisionMatcher::Variant {
                            type_name, index, ..
                        } => {
                            let alternative = format!("at {}", source.slice(*index));
                            matches!(
                                &subject,
                                Value::Union(union)
                                    if union.payload.is_some()
                                        && union.type_name == source.slice(*type_name)
                                        && union.alternative == alternative
                            )
                        }
                        DecisionMatcher::Result { error, .. } => {
                            *error == matches!(subject, Value::Error { .. })
                        }
                        DecisionMatcher::Optional { some, .. } => {
                            let Value::Optional { payload, .. } = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "Optional matchers require an Optional subject",
                                ));
                            };
                            *some == payload.is_some()
                        }
                        DecisionMatcher::ListEmpty(_) => {
                            let Value::List { entries, .. } = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "list matchers require a List subject",
                                ));
                            };
                            entries.is_empty()
                        }
                        DecisionMatcher::ListEntry { .. } => {
                            let Value::List { entries, .. } = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "list matchers require a List subject",
                                ));
                            };
                            !entries.is_empty()
                        }
                        DecisionMatcher::ErrorCode {
                            namespace,
                            vocabulary,
                            code,
                            ..
                        } => {
                            let namespace = source.slice(*namespace);
                            let vocabulary = source.slice(*vocabulary);
                            let code_span = *code;
                            let code = source.slice(code_span);
                            let known = namespace == "lang"
                                && ((vocabulary == "arithmetic" && is_arithmetic_error_code(code))
                                    || (vocabulary == "generator" && code == "generator-closed"));
                            if !known {
                                return Err(diagnostic(
                                    source,
                                    "E-UNKNOWN-ERROR-CODE",
                                    code_span,
                                    "the error-code pattern requires a code published by the qualified language namespace",
                                ));
                            }
                            let matched = matches!(&subject, Value::Error { code: subject_code, .. } if subject_code == code);
                            if vocabulary == "generator" && matched {
                                trace.record(TraceEvent {
                                    event: "generator.error.code.matched",
                                    rule: "TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001",
                                    detail: code,
                                });
                            }
                            matched
                        }
                        DecisionMatcher::Comparison {
                            kind,
                            operand,
                            span: matcher_span,
                        } => {
                            let right_span = operand.span();
                            let right = self.evaluate_expression(source, operand, trace)?;
                            matches!(
                                apply_binary(
                                    source,
                                    *kind,
                                    subject.clone(),
                                    right,
                                    (*matcher_span, subject_span, right_span),
                                    trace,
                                )?,
                                Value::Boolean(true)
                            )
                        }
                        DecisionMatcher::Otherwise(_) => true,
                    };
                    let detail = format!("rule={index};matched={matches}");
                    trace.record(TraceEvent {
                        event: "decision.rule.considered",
                        rule: decision_rule,
                        detail: &detail,
                    });
                    if matches {
                        selected = Some((index, rule));
                        break;
                    }
                }
                let Some((index, selected_rule)) = selected else {
                    return Err(diagnostic(
                        source,
                        "E-INCOMPLETE-DECISION",
                        *span,
                        "no decision rule matched the subject",
                    ));
                };
                let detail = format!("rule={index}");
                trace.record(TraceEvent {
                    event: "decision.rule.selected",
                    rule: decision_rule,
                    detail: &detail,
                });
                if let DecisionMatcher::ErrorCode { code, .. } = selected_rule.matcher {
                    trace.record(TraceEvent {
                        event: "error.code.matched",
                        rule: "TOPAL-DECISION-ERROR-CODE-001",
                        detail: source.slice(code),
                    });
                }
                if let DecisionMatcher::Result { binding, .. } = selected_rule.matcher {
                    let name = source.slice(binding);
                    let mut branch = Self {
                        bindings: self.bindings.clone(),
                        functions: self.functions.clone(),
                        generators: self.generators.clone(),
                        declared_names: self.declared_names.clone(),
                        consumed_names: self.consumed_names.clone(),
                        local_function_names: self.local_function_names.clone(),
                        enum_types: self.enum_types.clone(),
                        union_types: self.union_types.clone(),
                        call_stack: self.call_stack.clone(),
                        static_context: self.static_context,
                    };
                    branch.bindings.insert(name.to_owned(), subject);
                    trace.record(TraceEvent {
                        event: "result.payload.bound",
                        rule: "TOPAL-DECISION-RESULT-001",
                        detail: name,
                    });
                    branch.evaluate_expression(source, &selected_rule.action, trace)
                } else if let DecisionMatcher::Optional {
                    binding: Some(binding),
                    ..
                } = selected_rule.matcher
                {
                    let Value::Optional {
                        payload: Some(payload),
                        ..
                    } = subject
                    else {
                        unreachable!("Some matcher selected only for a present payload")
                    };
                    let name = source.slice(binding);
                    let mut branch = self.clone();
                    branch.bindings.insert(name.to_owned(), *payload);
                    trace.record(TraceEvent {
                        event: "optional.payload.bound",
                        rule: "TOPAL-DECISION-OPTIONAL-001",
                        detail: name,
                    });
                    branch.evaluate_expression(source, &selected_rule.action, trace)
                } else if let DecisionMatcher::ListEntry { first, rest, .. } = selected_rule.matcher
                {
                    let Value::List {
                        element_classifier,
                        mut entries,
                    } = subject
                    else {
                        unreachable!("Entry matcher selected only for a nonempty List")
                    };
                    let first_value = entries.remove(0);
                    let first = source.slice(first);
                    let rest = source.slice(rest);
                    let mut branch = self.clone();
                    branch.bindings.insert(first.to_owned(), first_value);
                    branch.bindings.insert(
                        rest.to_owned(),
                        Value::List {
                            element_classifier,
                            entries,
                        },
                    );
                    let detail = format!("first={first};rest={rest}");
                    trace.record(TraceEvent {
                        event: "list.entry.decomposed",
                        rule: "TOPAL-DECISION-LIST-001",
                        detail: &detail,
                    });
                    branch.evaluate_expression(source, &selected_rule.action, trace)
                } else if let DecisionMatcher::Union { binding, .. } = selected_rule.matcher {
                    self.evaluate_union_decision_action(
                        source,
                        subject,
                        binding,
                        &selected_rule.action,
                        trace,
                    )
                } else if let DecisionMatcher::Variant { binding, .. } = selected_rule.matcher {
                    self.evaluate_union_decision_action(
                        source,
                        subject,
                        binding,
                        &selected_rule.action,
                        trace,
                    )
                } else {
                    self.evaluate_expression(source, &selected_rule.action, trace)
                }
            }
            Expression::Integer(span) => evaluate_integer_literal(source, *span, trace),
            Expression::Rational(span) => evaluate_rational_literal(source, *span, trace),
            Expression::String(span) => evaluate_string_literal(source, *span, trace),
            Expression::Identifier(span) => self.resolve_identifier(source, *span, trace),
            Expression::Discard(span) => Err(diagnostic(
                source,
                "E-DISCARD-VALUE",
                *span,
                "discard is valid only in a declaration or pattern",
            )),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: _,
            } => Ok(self.capture_anonymous_function(source, parameters, body, trace)),
            Expression::Callable { kind, .. } => {
                trace.record(TraceEvent {
                    event: "function.callable.captured",
                    rule: "TOPAL-FUNCTION-CALLABLE-VALUE-001",
                    detail: callable_name(*kind),
                });
                Ok(Value::Callable(*kind))
            }
            Expression::Application { items, span } => {
                if Self::is_empty_effects(source, items) {
                    trace.record(TraceEvent {
                        event: "effects.empty.constructed",
                        rule: "TOPAL-EFFECT-EMPTY-001",
                        detail: "Effects ()",
                    });
                    return Ok(Value::Effects(Vec::new()));
                }
                if Self::is_use_application(source, items) {
                    return self.evaluate_use_application(source, items, *span, trace);
                }
                if self.is_bound_namespace_application(source, items) {
                    return self.evaluate_bound_namespace_application(source, items, *span, trace);
                }
                if Self::is_root_qualified_application(source, items) {
                    return self.evaluate_root_qualified_application(source, items, *span, trace);
                }
                if Self::is_unfold_construction(source, items) {
                    return self.construct_unfold_generator(source, items, *span, trace);
                }
                if Self::is_iterate_take_while_construction(source, items) {
                    return self.construct_iterate_take_while(source, items, *span, trace);
                }
                if Self::is_iterate_construction(source, items) {
                    return self.construct_iterate_generator(source, items, *span, trace);
                }
                if self.is_bound_named_function_call(source, items) {
                    return self
                        .evaluate_bound_named_function_call(source, expression, items, trace);
                }
                if self.is_bound_callable_call(source, items) {
                    return self.evaluate_bound_callable_call(source, items, *span, trace);
                }
                if Self::is_traversal_control_constructor(source, items) {
                    return self.construct_traversal_control(source, items, *span, trace);
                }
                if self.is_bound_anonymous_call(source, items) {
                    return self.evaluate_bound_anonymous_call(source, items, *span, trace);
                }
                if Self::is_record_reconstruction(source, items) {
                    return self.evaluate_record_reconstruction(source, items, *span, trace);
                }
                if self.is_bound_list_higher_order_application(source, items) {
                    return self.evaluate_list_higher_order(source, items, *span, trace);
                }
                if Self::is_range_selection(source, items) {
                    return self.evaluate_range_selection(source, items, *span, trace);
                }
                if self.is_explicit_modulo(source, items) {
                    return self.apply_explicit_modulo(source, items, trace);
                }
                if Self::is_modular_type_definition(source, items) {
                    return self.construct_modular_type(source, items, trace);
                }
                if self.is_modular_construction(source, items) {
                    return self.construct_modular_value(source, items, *span, trace);
                }
                if Self::is_constraint_definition(source, items) {
                    return self.construct_constraint(source, items, trace);
                }
                if self.is_constraint_application(source, items) {
                    return self.apply_constraint(source, items, *span, trace);
                }
                if self.application_is_union_constructor(source, items) {
                    return self.construct_union_application(source, items, *span, trace);
                }
                if matches!(
                    items.as_slice(),
                    [Expression::Identifier(operation), ..]
                        if matches!(source.slice(*operation), "unzip" | "collect" | "collect-set" | "collect-bag" | "collect-map")
                ) || matches!(
                    items.as_slice(),
                    [_, Expression::Identifier(operation), ..]
                        if matches!(source.slice(*operation), "zip-longest" | "collect")
                ) {
                    return self.evaluate_list_materialization(source, items, *span, trace);
                }
                if matches!(
                    items.as_slice(),
                    [_, Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
                        if matches!(source.slice(*operation), "map" | "select" | "remove-indexes" | "remove-values")
                ) || matches!(
                    items.as_slice(),
                    [_, Expression::Identifier(operation), _, Expression::AnonymousFunction { .. }]
                        if source.slice(*operation) == "fold"
                ) {
                    return self.evaluate_list_higher_order(source, items, *span, trace);
                }
                if Self::is_characters_application(source, items) {
                    return self.evaluate_characters_application(source, items, *span, trace);
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "canonically-equals"
                {
                    let left_span = left.span();
                    let right_span = right.span();
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    let (Value::String(left), Value::String(right)) = (left, right) else {
                        return Err(diagnostic(
                            source,
                            "E-CANONICAL-EQUALITY-OPERANDS",
                            cover(left_span, right_span),
                            "canonically-equals requires two String operands",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.canonically-equals(String,String)",
                    });
                    let value = Value::Boolean(canonically_equal(&left, &right));
                    trace.record(TraceEvent {
                        event: "string.canonical-equality.compared",
                        rule: "TOPAL-STRING-CANONICAL-EQUALITY-001",
                        detail: if matches!(value, Value::Boolean(true)) {
                            "equal"
                        } else {
                            "unequal"
                        },
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if let Some(value) = evaluate_arithmetic_error_code(source, items, trace) {
                    return Ok(value);
                }
                if let Some(value) = evaluate_generator_error_code(source, items, trace) {
                    return Ok(value);
                }
                if let [Expression::Identifier(constructor), payload] = items.as_slice()
                    && source.slice(*constructor) == "Some"
                {
                    let value = self.evaluate_expression(source, payload, trace)?;
                    let payload_classifier = structural_value_classifier(&value);
                    trace.record(TraceEvent {
                        event: "optional.some.constructed",
                        rule: "TOPAL-TYPE-OPTIONAL-CONSTRUCT-001",
                        detail: &payload_classifier,
                    });
                    return Ok(Value::Optional {
                        payload_classifier,
                        payload: Some(Box::new(value)),
                    });
                }
                if let [Expression::Identifier(constructor), domain] = items.as_slice()
                    && source.slice(*constructor) == "None"
                    && let Some(payload_classifier) = classifier_expression(source, domain)
                {
                    trace.record(TraceEvent {
                        event: "optional.none.constructed",
                        rule: "TOPAL-TYPE-OPTIONAL-CONSTRUCT-001",
                        detail: &payload_classifier,
                    });
                    return Ok(Value::Optional {
                        payload_classifier,
                        payload: None,
                    });
                }
                if let [Expression::Identifier(constructor), character] = items.as_slice()
                    && source.slice(*constructor) == "String"
                {
                    let value = self.evaluate_expression(source, character, trace)?;
                    let Value::String(text) = value else {
                        return Err(diagnostic(
                            source,
                            "E-STRING-CONSTRUCTOR-CHARACTER",
                            character.span(),
                            "String construction requires a Character value",
                        ));
                    };
                    let count = character_count(&text);
                    if count != 1 {
                        return Err(diagnostic(
                            source,
                            "E-STRING-CONSTRUCTOR-CHARACTER",
                            character.span(),
                            format!(
                                "String construction requires one Character, but the operand contains {count}"
                            ),
                        ));
                    }
                    trace.record(TraceEvent {
                        event: "string.from-character",
                        rule: "TOPAL-STRING-FROM-CHARACTER-001",
                        detail: "preserved",
                    });
                    return Ok(Value::String(text));
                }
                if let [Expression::Identifier(constructor), operand] = items.as_slice()
                    && source.slice(*constructor) == "Int"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    return construct_int(source, operand, value, trace);
                }
                if let [Expression::Identifier(constructor), operand] = items.as_slice()
                    && source.slice(*constructor) == "Nat"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    return construct_nat(source, operand, value, trace);
                }
                if let [Expression::Identifier(constructor), operand] = items.as_slice()
                    && source.slice(*constructor) == "Rational"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    return construct_rational(source, operand, value, trace);
                }
                if let [Expression::Identifier(callable), operand] = items.as_slice()
                    && source.slice(*callable) == "absolute"
                {
                    let operand_span = operand.span();
                    let value = self.evaluate_expression(source, operand, trace)?;
                    let (value, selection, classifier) = match value {
                        Value::Int(value) => (
                            Value::Int(if value < BigInt::from(0) {
                                -value
                            } else {
                                value
                            }),
                            "root.absolute(Int)",
                            "Int",
                        ),
                        Value::Rational(value) => (
                            Value::Rational(
                                if value < BigRational::from_integer(BigInt::from(0)) {
                                    -value
                                } else {
                                    value
                                },
                            ),
                            "root.absolute(Rational)",
                            "Rational",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                operand_span,
                                "absolute requires an exact numeric operand",
                            ));
                        }
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.absolute",
                        rule: "TOPAL-NUM-ABS-001",
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if let [
                    Expression::Identifier(callable),
                    Expression::Identifier(domain),
                ] = items.as_slice()
                    && source.slice(*callable) == "zero"
                {
                    let (value, selection, classifier) = match source.slice(*domain) {
                        "Int" => (Value::Int(BigInt::from(0)), "root.zero(Int)", "Int"),
                        "Nat" => (Value::Int(BigInt::from(0)), "root.zero(Nat)", "Nat"),
                        "Rational" => (
                            Value::Rational(BigRational::from_integer(BigInt::from(0))),
                            "root.zero(Rational)",
                            "Rational",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                *domain,
                                "zero requires a supported numeric type",
                            ));
                        }
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "numeric.zero.constructed",
                        rule: "TOPAL-NUM-ZERO-001",
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if let [
                    Expression::Identifier(callable),
                    Expression::Identifier(domain),
                ] = items.as_slice()
                    && source.slice(*callable) == "one"
                    && matches!(source.slice(*domain), "Int" | "Nat" | "Rational")
                {
                    let (value, selection, classifier) = match source.slice(*domain) {
                        "Int" => (Value::Int(BigInt::from(1)), "root.one(Int)", "Int"),
                        "Nat" => (Value::Int(BigInt::from(1)), "root.one(Nat)", "Nat"),
                        "Rational" => (
                            Value::Rational(BigRational::from_integer(BigInt::from(1))),
                            "root.one(Rational)",
                            "Rational",
                        ),
                        _ => unreachable!("guarded numeric one domain"),
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "numeric.one.constructed",
                        rule: "TOPAL-NUM-ONE-001",
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if is_singleton_list_construction(source, items) {
                    return evaluate_singleton_list(source, self, items, trace);
                }
                if is_explicit_empty_list_construction(source, items) {
                    return evaluate_empty_list(source, items, trace);
                }
                if let [Expression::Identifier(callable), operand] = items.as_slice()
                    && source.slice(*callable) == "negate"
                {
                    let operand_span = operand.span();
                    let value = self.evaluate_expression(source, operand, trace)?;
                    let (value, selection, classifier, rule) = match value {
                        Value::Int(value) => (
                            Value::Int(-value),
                            "root.negate(Int)",
                            "Int",
                            "TOPAL-NUM-NEG-001",
                        ),
                        Value::Rational(value) => (
                            Value::Rational(-value),
                            "root.negate(Rational)",
                            "Rational",
                            "TOPAL-NUM-RAT-NEG-001",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                operand_span,
                                "negate requires an exact numeric operand",
                            ));
                        }
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.negate",
                        rule,
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if let [Expression::Identifier(callable), operand] = items.as_slice()
                    && source.slice(*callable) == "not"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    let Value::Boolean(value) = value else {
                        return Err(diagnostic(
                            source,
                            "E-BOOLEAN-NOT-OPERAND",
                            operand.span(),
                            "not requires a Boolean operand",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.not(Boolean)",
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.logical",
                        rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
                        detail: "not",
                    });
                    return Ok(Value::Boolean(!value));
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && matches!(source.slice(*callable), "in" | "contains")
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    return apply_range_membership(
                        source,
                        source.slice(*callable),
                        left,
                        right,
                        *span,
                        trace,
                    );
                }
                if let [text, Expression::Identifier(callable), index] = items.as_slice()
                    && source.slice(*callable) == "character-at"
                {
                    let text_span = text.span();
                    let index_span = index.span();
                    let text = self.evaluate_expression(source, text, trace)?;
                    let index = self.evaluate_expression(source, index, trace)?;
                    let (Value::String(text), Value::Int(index)) = (text, index) else {
                        return Err(diagnostic(
                            source,
                            "E-CHARACTER-AT-OPERANDS",
                            cover(text_span, index_span),
                            "character-at requires a String and an Int index",
                        ));
                    };
                    let payload = usize::try_from(index)
                        .ok()
                        .and_then(|index| character_at(&text, index))
                        .map(|character| Box::new(Value::String(character.to_owned())));
                    trace.record(TraceEvent {
                        event: "string.character-at",
                        rule: "TOPAL-STRING-CHARACTER-AT-001",
                        detail: if payload.is_some() { "Some" } else { "None" },
                    });
                    return Ok(Value::Optional {
                        payload_classifier: "Character".to_owned(),
                        payload,
                    });
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "and"
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    return apply_and(source, left, right, *span, trace);
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "or"
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    let (Value::Boolean(left), Value::Boolean(right)) = (left, right) else {
                        return Err(diagnostic(
                            source,
                            "E-BOOLEAN-OR-OPERANDS",
                            *span,
                            "or requires two Boolean operands; range union is a Predicate",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.or(Boolean,Boolean)",
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.logical",
                        rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
                        detail: "or:eager",
                    });
                    return Ok(Value::Boolean(left || right));
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "xor"
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    let (Value::Boolean(left), Value::Boolean(right)) = (left, right) else {
                        return Err(diagnostic(
                            source,
                            "E-BOOLEAN-XOR-OPERANDS",
                            *span,
                            "xor requires two Boolean operands",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.xor(Boolean,Boolean)",
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.logical",
                        rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
                        detail: "xor:eager",
                    });
                    return Ok(Value::Boolean(left != right));
                }
                if items.len() == 3
                    && let Expression::Identifier(name_span) = items[1]
                    && !self.bindings.contains_key(source.slice(name_span))
                    && self.functions.contains_key(source.slice(name_span))
                {
                    let argument_span = cover(items[0].span(), items[2].span());
                    let call = Expression::Application {
                        items: vec![
                            Expression::Identifier(name_span),
                            Expression::Product {
                                fields: vec![
                                    topal_syntax::ProductField {
                                        label: None,
                                        value: items[0].clone(),
                                    },
                                    topal_syntax::ProductField {
                                        label: None,
                                        value: items[2].clone(),
                                    },
                                ],
                                span: argument_span,
                            },
                        ],
                        span: *span,
                    };
                    return self.evaluate_expression(source, &call, trace);
                }
                if items.len() == 2
                    && let Expression::Identifier(name_span) = &items[0]
                    && !self.bindings.contains_key(source.slice(*name_span))
                    && let Some(candidates) = self.generators.get(source.slice(*name_span)).cloned()
                {
                    let name = source.slice(*name_span);
                    let argument_span = items[1].span();
                    let argument = self.evaluate_expression(source, &items[1], trace)?;
                    let Some(generator) = candidates
                        .iter()
                        .find(|candidate| function_accepts(&candidate.parameters, &argument))
                        .cloned()
                    else {
                        return Err(no_applicable_generator(
                            source,
                            name,
                            argument_span,
                            &argument,
                            &candidates,
                        ));
                    };
                    let mut generator_scope = Self {
                        bindings: generator.bindings,
                        functions: self.functions.clone(),
                        generators: self.generators.clone(),
                        declared_names: BTreeSet::new(),
                        consumed_names: BTreeSet::new(),
                        local_function_names: BTreeSet::new(),
                        enum_types: self.enum_types.clone(),
                        union_types: self.union_types.clone(),
                        call_stack: self.call_stack.clone(),
                        static_context: false,
                    };
                    bind_generator_arguments(
                        &mut generator_scope,
                        &generator.parameters,
                        argument,
                        trace,
                    );
                    let signature = generator
                        .parameters
                        .iter()
                        .map(|(_, classifier)| classifier.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    trace.record(TraceEvent {
                        event: "generator.selected",
                        rule: "TOPAL-GENERATOR-OVERLOAD-001",
                        detail: &signature,
                    });
                    trace.record(TraceEvent {
                        event: "generator.started",
                        rule: "TOPAL-GENERATOR-DECLARATION-001",
                        detail: name,
                    });
                    let mut cursor = 0;
                    let mut pending_yield = None;
                    let mut resume_binding = None;
                    let mut returned = None;
                    advance_custom_generator(
                        &generator.source,
                        &generator.body,
                        &mut cursor,
                        &mut generator_scope,
                        &mut pending_yield,
                        &mut resume_binding,
                        &mut returned,
                        &generator.yielded,
                        &generator.result,
                        name,
                        trace,
                    )?;
                    let origin = format!("root.{name}");
                    let value = Value::SuspendedGenerator {
                        source: generator.source,
                        body: generator.body,
                        cursor,
                        bindings: generator_scope.bindings,
                        scope_state: Box::new(GeneratorScopeState {
                            functions: generator_scope.functions,
                            declared_names: generator_scope.declared_names,
                            local_function_names: generator_scope.local_function_names,
                            enum_types: generator_scope.enum_types,
                            union_types: generator_scope.union_types,
                        }),
                        pending_yield,
                        resume_binding,
                        returned: returned.map(Box::new),
                        yield_classifier: generator.yielded,
                        return_classifier: generator.result,
                        origin,
                    };
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name_span) = &items[0]
                    && !self.bindings.contains_key(source.slice(*name_span))
                    && let Some(candidates) = self.functions.get(source.slice(*name_span)).cloned()
                {
                    let name = source.slice(*name_span);
                    let argument_span = items[1].span();
                    let argument = self.evaluate_expression(source, &items[1], trace)?;
                    let function = candidates
                        .iter()
                        .find(|function| {
                            (!self.static_context || function.is_static)
                                && function_accepts(&function.parameters, &argument)
                        })
                        .cloned();
                    let Some(function) = function else {
                        if self.static_context
                            && candidates.iter().all(|function| !function.is_static)
                        {
                            return Err(diagnostic(
                                source,
                                "E-STATIC-CALLS-RUNTIME-FUNCTION",
                                *name_span,
                                format!("static execution cannot call ordinary function `{name}`"),
                            ));
                        }
                        return Err(no_applicable_overload(
                            source,
                            name,
                            argument_span,
                            &argument,
                            &candidates,
                            self.static_context,
                        ));
                    };
                    if matches!(
                        argument,
                        Value::CharacterGenerator { .. }
                            | Value::CharacterReturningGenerator { .. }
                            | Value::SuspendedGenerator { .. }
                    ) {
                        let classifier = structural_value_classifier(&argument);
                        trace.record(TraceEvent {
                            event: "generator.parameter.transferred",
                            rule: if matches!(argument, Value::SuspendedGenerator { .. }) {
                                "TOPAL-GENERATOR-FUNCTION-PARAMETER-001"
                            } else {
                                "TOPAL-STRING-CHARACTERS-PARAMETER-001"
                            },
                            detail: &classifier,
                        });
                    }
                    let signature = function_signature(name, &function);
                    let recursion_rule =
                        recursion_rule_for_call(&self.call_stack, name, &signature, &function);
                    if self
                        .call_stack
                        .iter()
                        .any(|active| active.signature == signature)
                        && recursion_rule.is_none()
                    {
                        return Err(diagnostic(
                            source,
                            "E-RECURSION-NOT-YET-PROVEN",
                            *name_span,
                            format!(
                                "recursive cycle returning to `{name}` requires termination proof on every call edge"
                            ),
                        ));
                    }
                    let rule = function_rule(function.is_static, function.parameters.len());
                    if let Some(recursion_rule) = recursion_rule {
                        if is_mutual_recursion_rule(recursion_rule) {
                            trace.record(TraceEvent {
                                event: "function.recursion.cycle.proven",
                                rule: recursion_rule,
                                detail: name,
                            });
                        }
                        trace.record(TraceEvent {
                            event: "function.recursion.descended",
                            rule: recursion_rule,
                            detail: name,
                        });
                    }
                    if candidates.len() > 1 {
                        trace.record(TraceEvent {
                            event: "function.overload.selected",
                            rule: "TOPAL-FUNCTION-OVERLOAD-001",
                            detail: &signature,
                        });
                    }
                    let mut function_scope = Self {
                        bindings: function.bindings,
                        functions: self.functions.clone(),
                        generators: self.generators.clone(),
                        declared_names: BTreeSet::new(),
                        consumed_names: BTreeSet::new(),
                        local_function_names: BTreeSet::new(),
                        enum_types: self.enum_types.clone(),
                        union_types: self.union_types.clone(),
                        call_stack: self.call_stack.clone(),
                        static_context: function.is_static,
                    };
                    function_scope.call_stack.push(ActiveCall {
                        name: name.to_owned(),
                        signature: signature.clone(),
                        termination_rule: function.termination_rule,
                        recursion_target: function.recursion_target.clone(),
                    });
                    bind_function_arguments(
                        &mut function_scope,
                        &function.parameters,
                        argument,
                        trace,
                        rule,
                    );
                    trace.record(TraceEvent {
                        event: "function.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: &signature,
                    });
                    trace.record(TraceEvent {
                        event: "function.entered",
                        rule,
                        detail: name,
                    });
                    let mut body_execution = Execution {
                        source: function.source.clone(),
                        statements: function.body.clone(),
                        cursor: 0,
                        return_classifier: Some(function.result.clone()),
                    };
                    let (value, result_span) = loop {
                        match body_execution.step(&mut function_scope, trace)? {
                            ExecutionStep::Advanced { .. } => {}
                            ExecutionStep::Complete(value) => {
                                break (
                                    value,
                                    statement_span(
                                        function.body.last().expect("function body is nonempty"),
                                    ),
                                );
                            }
                            ExecutionStep::Returned { value, span } => break (value, span),
                        }
                    };
                    if !function.result.starts_with("Generator ") {
                        close_remaining_character_generators(&mut function_scope, trace)?;
                    }
                    if !value_has_classifier(&value, &function.result) {
                        return Err(diagnostic(
                            &function.source,
                            "E-FUNCTION-RESULT-TYPE",
                            result_span,
                            format!(
                                "function `{name}` returned a value outside `{}`",
                                function.result
                            ),
                        ));
                    }
                    if let Value::Error { domain, code, .. } = &value
                        && result_success_classifier(&function.result).is_some()
                    {
                        let detail = format!("domain={domain};code={code}");
                        trace.record(TraceEvent {
                            event: "result.error.propagated",
                            rule: "TOPAL-TYPE-RESULT-001",
                            detail: &detail,
                        });
                    }
                    if matches!(
                        value,
                        Value::CharacterGenerator { .. }
                            | Value::CharacterReturningGenerator { .. }
                            | Value::SuspendedGenerator { .. }
                    ) {
                        let classifier = structural_value_classifier(&value);
                        trace.record(TraceEvent {
                            event: "generator.function.returned",
                            rule: if matches!(value, Value::SuspendedGenerator { .. }) {
                                "TOPAL-GENERATOR-FUNCTION-RESULT-001"
                            } else {
                                "TOPAL-STRING-CHARACTERS-RESULT-001"
                            },
                            detail: &classifier,
                        });
                    }
                    trace.record(TraceEvent {
                        event: "function.returned",
                        rule,
                        detail: name,
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "empty?")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value = apply_empty_predicate(source, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if is_list_uncons(source, items) {
                    return evaluate_list_uncons(source, self, items, *span, trace);
                }
                if is_list_projection(source, items) {
                    return evaluate_list_projection(source, self, items, *span, trace);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(source.slice(*name), "character-count" | "entry-count")
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value = apply_count(source, operation, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "upper")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "upper requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.upper(String)",
                    });
                    let value = Value::String(uppercase(&text));
                    trace.record(TraceEvent {
                        event: "string.uppercased",
                        rule: "TOPAL-STRING-UPPER-001",
                        detail: "unicode-default",
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "lower")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "lower requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.lower(String)",
                    });
                    let value = Value::String(lowercase(&text));
                    trace.record(TraceEvent {
                        event: "string.lowercased",
                        rule: "TOPAL-STRING-LOWER-001",
                        detail: "unicode-default",
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "case-fold")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "case-fold requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.case-fold(String)",
                    });
                    let value = Value::String(case_fold(&text));
                    trace.record(TraceEvent {
                        event: "string.case-folded",
                        rule: "TOPAL-STRING-CASE-FOLD-001",
                        detail: "unicode-default-full",
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "empty")
                    && matches!(&items[1], Expression::Identifier(name) if source.slice(*name) == "String")
                {
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.empty(String)",
                    });
                    trace.record(TraceEvent {
                        event: "string.empty",
                        rule: "TOPAL-STRING-EMPTY-001",
                        detail: "String",
                    });
                    let value = Value::String(String::new());
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                let (mut result, mut index) = if matches!(
                    items.first(),
                    Some(Expression::Callable {
                        kind: CallableKind::Minus,
                        ..
                    })
                ) {
                    let Some(operand) = items.get(1) else {
                        return Err(diagnostic(
                            source,
                            "E-EXPECTED-OPERAND",
                            *span,
                            "expected an operand after prefix -",
                        ));
                    };
                    let operand = self.evaluate_expression(source, operand, trace)?;
                    (apply_negate(source, operand, *span, trace)?, 2)
                } else {
                    (self.evaluate_expression(source, &items[0], trace)?, 1)
                };
                let mut composing_literals = matches!(items.first(), Some(Expression::String(_)));
                while index < items.len() {
                    if composing_literals
                        && let Expression::String(right_span) = &items[index]
                        && let Value::String(left) = &result
                    {
                        let Value::String(right) =
                            self.evaluate_expression(source, &items[index], trace)?
                        else {
                            unreachable!("string literal constructs String");
                        };
                        trace.record(TraceEvent {
                            event: "string.literals.composed",
                            rule: "TOPAL-STRING-LITERAL-COMPOSE-001",
                            detail: "String",
                        });
                        result = Value::String(format!("{left}{right}"));
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), *right_span)),
                        );
                        index += 1;
                        continue;
                    }
                    composing_literals = false;
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "normalize"
                        && let Value::String(text) = &result
                    {
                        let Some(form) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                "expected a normalization form after normalize",
                            ));
                        };
                        let form_span = form.span();
                        let Expression::Identifier(form_name) = form else {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                form_span,
                                "the implemented String normalize operation requires NFC or NFD",
                            ));
                        };
                        let form_name = source.slice(*form_name);
                        let (normalized, selection, rule) = match form_name {
                            "NFC" => (
                                normalize_nfc(text),
                                "root.normalize(String,NFC)",
                                "TOPAL-STRING-NORMALIZE-NFC-001",
                            ),
                            "NFD" => (
                                normalize_nfd(text),
                                "root.normalize(String,NFD)",
                                "TOPAL-STRING-NORMALIZE-NFD-001",
                            ),
                            _ => {
                                return Err(diagnostic(
                                    source,
                                    "E-NO-APPLICABLE-OVERLOAD",
                                    form_span,
                                    "the implemented String normalize operation requires NFC or NFD",
                                ));
                            }
                        };
                        trace.record(TraceEvent {
                            event: "operator.selected",
                            rule: "TOPAL-TYPE-CALL-001",
                            detail: selection,
                        });
                        let changed = normalized != *text;
                        trace.record(TraceEvent {
                            event: "string.normalized",
                            rule,
                            detail: if changed {
                                "changed=true"
                            } else {
                                "changed=false"
                            },
                        });
                        result = Value::String(normalized);
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), form_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "byte-count"
                        && let Value::String(text) = &result
                    {
                        let Some(encoding) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                "expected an Encoding after byte-count",
                            ));
                        };
                        let encoding_span = encoding.span();
                        if !matches!(encoding, Expression::Identifier(name) if source.slice(*name) == "Utf8")
                        {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                encoding_span,
                                "the implemented String byte-count operation requires Utf8",
                            ));
                        }
                        trace.record(TraceEvent {
                            event: "operator.selected",
                            rule: "TOPAL-TYPE-CALL-001",
                            detail: "root.byte-count(String,Utf8)",
                        });
                        let byte_count = text.len();
                        let detail = format!("bytes={byte_count}");
                        trace.record(TraceEvent {
                            event: "string.utf8-byte-count",
                            rule: "TOPAL-STRING-UTF8-BYTE-COUNT-001",
                            detail: &detail,
                        });
                        result = Value::Int(BigInt::from(byte_count));
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), encoding_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "reverse"
                        && matches!(result, Value::List { .. })
                    {
                        apply_list_reverse(&mut result, trace);
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "entries"
                        && matches!(result, Value::List { .. })
                    {
                        result = apply_list_entries_view(result, trace);
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "insert-at"
                        && let Value::List { .. } = result
                    {
                        result = self.evaluate_list_insert_at(
                            source,
                            result,
                            items.get(index + 1),
                            items.get(index + 2),
                            *callable_span,
                            trace,
                        )?;
                        index += 3;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && matches!(
                            source.slice(*callable_span),
                            "prepend"
                                | "append"
                                | "concat"
                                | "contains-entry"
                                | "contains-sequence"
                                | "contains-subsequence"
                                | "split-at"
                                | "take"
                                | "drop"
                                | "remove"
                                | "remove-indexes"
                                | "zip-exact"
                                | "zip-shortest"
                                | "remove-first"
                                | "remove-all"
                        )
                        && matches!(result, Value::List { .. })
                    {
                        let operation = source.slice(*callable_span);
                        let Some(right) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                format!("expected an operand after {operation}"),
                            ));
                        };
                        let right_span = right.span();
                        let right_is_closed = expression_is_closed(right);
                        let right = self.evaluate_expression(source, right, trace)?;
                        result = apply_list_operation(
                            source,
                            operation,
                            result,
                            right,
                            right_span,
                            right_is_closed,
                            trace,
                        )?;
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), right_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "concat"
                        && let Value::String(left) = &result
                    {
                        let Some(right) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                "expected a String after concat",
                            ));
                        };
                        let right_span = right.span();
                        let right = self.evaluate_expression(source, right, trace)?;
                        let Value::String(right) = right else {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                right_span,
                                "concat requires two String operands",
                            ));
                        };
                        trace.record(TraceEvent {
                            event: "operator.selected",
                            rule: "TOPAL-TYPE-CALL-001",
                            detail: "root.concat(String,String)",
                        });
                        trace.record(TraceEvent {
                            event: "evaluation.concat",
                            rule: "TOPAL-STRING-CONCAT-001",
                            detail: "String",
                        });
                        result = Value::String(format!("{left}{right}"));
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), right_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(label_span) = &items[index]
                        && let Value::Error {
                            domain,
                            code,
                            line,
                            column,
                        } = &result
                    {
                        let label = source.slice(*label_span);
                        let selected = match label {
                            "code" => Value::Enum {
                                type_name: "lang arithmetic ArithmeticErrorCode".into(),
                                alternative: code.clone(),
                            },
                            "domain" => Value::ErrorDomain(domain.clone()),
                            "detail" => Value::Optional {
                                payload_classifier: "String".into(),
                                payload: None,
                            },
                            "cause" => Value::Optional {
                                payload_classifier: "Error".into(),
                                payload: None,
                            },
                            "source" => Value::Optional {
                                payload_classifier: "SourceLocation".into(),
                                payload: Some(Box::new(Value::Record(vec![
                                    ("line".into(), Value::Int(BigInt::from(*line))),
                                    ("column".into(), Value::Int(BigInt::from(*column))),
                                ]))),
                            },
                            _ => {
                                return Err(diagnostic(
                                    source,
                                    "E-NO-SUCH-ERROR-FIELD",
                                    *label_span,
                                    format!("Error has no implemented field named `{label}`"),
                                ));
                            }
                        };
                        trace.record(TraceEvent {
                            event: "error.field.selected",
                            rule: "TOPAL-ERROR-FIELD-001",
                            detail: label,
                        });
                        result = selected;
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(label_span) = &items[index]
                        && let Value::Record(fields) = &result
                    {
                        let label = source.slice(*label_span);
                        let selected = fields
                            .iter()
                            .find(|(field, _)| field == label)
                            .map(|(_, value)| value.clone())
                            .ok_or_else(|| {
                                diagnostic(
                                    source,
                                    "E-NO-SUCH-RECORD-FIELD",
                                    *label_span,
                                    format!("record has no field named `{label}`"),
                                )
                            })?;
                        trace.record(TraceEvent {
                            event: "record.field.selected",
                            rule: "TOPAL-TYPE-PRODUCT-001",
                            detail: label,
                        });
                        result = selected;
                        index += 1;
                        continue;
                    }
                    let Expression::Callable {
                        kind,
                        span: operator_span,
                    } = &items[index]
                    else {
                        let mut error = diagnostic(
                            source,
                            "E-UNSUPPORTED-APPLICATION",
                            items[index].span(),
                            "the implemented subset requires a symbolic callable",
                        );
                        if let Expression::Identifier(name_span) = &items[index]
                            && let Some(candidate) =
                                closest_root_operation(source.slice(*name_span))
                        {
                            error = error.with_help(format!("did you mean `{candidate}`?"));
                        }
                        return Err(error);
                    };
                    let Some(right) = items.get(index + 1) else {
                        return Err(diagnostic(
                            source,
                            "E-EXPECTED-OPERAND",
                            Span::new(operator_span.end, operator_span.end),
                            "expected an operand after callable",
                        ));
                    };
                    let right_span = right.span();
                    let right = self.evaluate_expression(source, right, trace)?;
                    result = apply_binary(
                        source,
                        *kind,
                        result,
                        right,
                        (*span, items[0].span(), right_span),
                        trace,
                    )?;
                    self.checkpoint(
                        trace,
                        Some(&result),
                        Some(cover(items[0].span(), right_span)),
                    );
                    index += 2;
                }
                Ok(result)
            }
        }?;
        self.checkpoint(trace, Some(&value), Some(expression.span()));
        Ok(value)
    }

    fn evaluate_record(
        &self,
        source: &SourceText,
        fields: &[topal_syntax::ProductField],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut values = Vec::with_capacity(fields.len());
        for field in fields {
            let label_span = field.label.expect("record fields are labeled");
            let label = source.slice(label_span);
            if values.iter().any(|(existing, _)| existing == label) {
                return Err(diagnostic(
                    source,
                    "E-DUPLICATE-RECORD-FIELD",
                    label_span,
                    "record field label occurs more than once",
                ));
            }
            let value = self.evaluate_expression(source, &field.value, trace)?;
            values.push((label.to_owned(), value));
        }
        let detail = format!("fields={}", values.len());
        trace.record(TraceEvent {
            event: "product.record",
            rule: "TOPAL-TYPE-PRODUCT-001",
            detail: &detail,
        });
        Ok(Value::Record(values))
    }

    fn evaluate_product(
        &self,
        source: &SourceText,
        fields: &[topal_syntax::ProductField],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let labeled = fields.iter().filter(|field| field.label.is_some()).count();
        if labeled != 0 && labeled != fields.len() {
            return Err(diagnostic(
                source,
                "E-UNSUPPORTED-MIXED-PRODUCT",
                span,
                "mixed positional and labeled product fields are not yet implemented",
            ));
        }
        if labeled == 0 {
            let values = fields
                .iter()
                .map(|field| self.evaluate_expression(source, &field.value, trace))
                .collect::<Result<Vec<_>, _>>()?;
            let detail = format!("fields={}", values.len());
            trace.record(TraceEvent {
                event: "product.tuple",
                rule: "TOPAL-TYPE-PRODUCT-001",
                detail: &detail,
            });
            Ok(Value::Tuple(values))
        } else {
            self.evaluate_record(source, fields, trace)
        }
    }

    fn resolve_identifier(
        &self,
        source: &SourceText,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let name = source.slice(span);
        if matches!(name, "Little" | "Big") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-ENDIAN-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "Endian".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "ReadWrite" | "ReadOnly" | "WriteOnly" | "Reserved") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-ACCESS-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "Access".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "MostSignificantFirst" | "LeastSignificantFirst") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-BIT-ORDER-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "BitOrder".into(),
                alternative: name.into(),
            });
        }
        if matches!(
            name,
            "Boolean" | "Completed" | "Int" | "Nat" | "Rational" | "Scope" | "String" | "Unit"
        ) && name != "Completed"
        {
            trace.record(TraceEvent {
                event: "type.resolved",
                rule: "TOPAL-ABSTRACTION-TYPE-VALUE-001",
                detail: name,
            });
            return Ok(Value::Type(name.into()));
        }
        if name == "root" {
            trace.record(TraceEvent {
                event: "namespace.resolved",
                rule: "TOPAL-NAMESPACE-ROOT-001",
                detail: "root",
            });
            return Ok(Value::Namespace(Box::new(NamespaceValue {
                name: "root".into(),
                bindings: self.bindings.clone(),
                functions: self.functions.clone(),
                generators: self.generators.clone(),
            })));
        }
        if name == "Completed" {
            trace.record(TraceEvent {
                event: "completion.evidence",
                rule: "TOPAL-EXEC-COMPLETED-001",
                detail: "Completed",
            });
            return Ok(Value::Completed);
        }
        if self.consumed_names.contains(name) {
            return Err(consumed_generator_diagnostic(source, span, name));
        }
        let value = if let Some(value) = self.bindings.get(name) {
            value.clone()
        } else if let Some(candidates) = self.functions.get(name) {
            Value::NamedFunction(Box::new(NamedFunction {
                name: name.to_owned(),
                candidates: candidates.clone(),
            }))
        } else {
            let error = diagnostic(source, "E-UNBOUND-NAME", span, "name is not bound");
            return Err(closest_name(name, self.bindings.keys())
                .or_else(|| closest_name(name, self.functions.keys()))
                .or_else(|| closest_root_operation(name))
                .map_or(error.clone(), |candidate| {
                    error.with_help(format!("did you mean `{candidate}`?"))
                }));
        };
        trace.record(TraceEvent {
            event: "binding.resolved",
            rule: "TOPAL-SYN-BIND-001",
            detail: name,
        });
        Ok(value)
    }

    fn evaluate_union_decision_action(
        &self,
        source: &SourceText,
        subject: Value,
        binding: Span,
        action: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::Union(mut union) = subject else {
            unreachable!("payload Union matcher selected only for a payload alternative")
        };
        let payload = union
            .payload
            .take()
            .expect("payload matcher selected a present payload");
        let name = source.slice(binding);
        let mut branch = self.clone();
        branch.bindings.insert(name.to_owned(), *payload);
        trace.record(TraceEvent {
            event: "union.payload.bound",
            rule: "TOPAL-DECISION-UNION-001",
            detail: name,
        });
        branch.evaluate_expression(source, action, trace)
    }

    fn union_constructor(&self, name: &str) -> Option<(&str, &str)> {
        self.union_types
            .iter()
            .find_map(|(type_name, alternatives)| {
                alternatives
                    .get(name)
                    .and_then(|classifier| classifier.as_deref())
                    .map(|classifier| (type_name.as_str(), classifier))
            })
    }

    fn application_is_union_constructor(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(constructor), _] if self.union_constructor(source.slice(*constructor)).is_some())
            || matches!(
                items,
                [Expression::Identifier(type_name), Expression::Identifier(at), Expression::Integer(_), _]
                    if source.slice(*at) == "at" && self.union_types.contains_key(source.slice(*type_name))
            )
    }

    fn is_constraint_definition(source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [Expression::Identifier(_), Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
                if source.slice(*operation) == "constraint"
        )
    }

    fn is_modular_type_definition(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(kind), _] if matches!(source.slice(*kind), "ModNat" | "ModInt"))
    }

    fn is_explicit_modulo(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [_, Expression::Identifier(operation), Expression::Identifier(type_name)]
                if source.slice(*operation) == "modulo"
                    && matches!(self.bindings.get(source.slice(*type_name)), Some(Value::ModularType(_)))
        )
    }

    fn is_range_selection(source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [_, Expression::Identifier(operation), selector]
                if matches!(source.slice(*operation), "select" | "select-index")
                    && !matches!(selector, Expression::AnonymousFunction { .. })
        )
    }

    fn is_bound_list_higher_order_application(
        &self,
        source: &SourceText,
        items: &[Expression],
    ) -> bool {
        let bound_function = |expression: &Expression| {
            matches!(expression, Expression::Identifier(name)
                if matches!(self.bindings.get(source.slice(*name)), Some(Value::AnonymousFunction(_))))
        };
        matches!(
            items,
            [_, Expression::Identifier(operation), function]
                if matches!(source.slice(*operation), "map" | "select" | "remove-indexes" | "remove-values")
                    && bound_function(function)
        ) || matches!(
            items,
            [_, Expression::Identifier(operation), _, function]
                if source.slice(*operation) == "fold" && bound_function(function)
        )
    }

    fn is_bound_anonymous_call(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(self.bindings.get(source.slice(*name)), Some(Value::AnonymousFunction(_))))
    }

    fn is_bound_callable_call(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(self.bindings.get(source.slice(*name)), Some(Value::Callable(_))))
    }

    fn is_bound_named_function_call(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(self.bindings.get(source.slice(*name)), Some(Value::NamedFunction(_))))
    }

    fn is_root_qualified_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(root), Expression::Identifier(_), ..]
            if source.slice(*root) == "root")
    }

    fn is_empty_effects(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), Expression::Unit(_)] if source.slice(*name) == "Effects")
    }

    fn is_use_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(keyword), _]
            if source.slice(*keyword) == "use")
    }

    fn evaluate_use_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [_, selected] = items else {
            unreachable!("preselected use application")
        };
        let value = self.evaluate_expression(source, selected, trace)?;
        if !matches!(value, Value::Namespace(_)) {
            return Err(diagnostic(
                source,
                "E-USE-NON-NAMESPACE",
                selected.span(),
                "use requires a published namespace path",
            ));
        }
        trace.record(TraceEvent {
            event: "namespace.made-available",
            rule: "TOPAL-NAMESPACE-USE-001",
            detail: &value.to_string(),
        });
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn is_bound_namespace_application(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(alias), Expression::Identifier(_), ..]
            if matches!(self.bindings.get(source.slice(*alias)), Some(Value::Namespace(_))))
    }

    fn evaluate_bound_namespace_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            Expression::Identifier(alias),
            Expression::Identifier(member),
            remainder @ ..,
        ] = items
        else {
            unreachable!("preselected namespace alias application")
        };
        let Some(Value::Namespace(namespace)) = self.bindings.get(source.slice(*alias)) else {
            unreachable!("preselected namespace alias")
        };
        let member_name = source.slice(*member);
        if !namespace.bindings.contains_key(member_name)
            && !namespace.functions.contains_key(member_name)
            && !namespace.generators.contains_key(member_name)
        {
            let names = namespace
                .bindings
                .keys()
                .chain(namespace.functions.keys())
                .chain(namespace.generators.keys());
            let error = diagnostic(
                source,
                "E-NAMESPACE-MEMBER-NOT-FOUND",
                *member,
                format!(
                    "namespace `{}` has no member `{member_name}`",
                    namespace.name
                ),
            );
            return Err(
                closest_name(member_name, names).map_or(error.clone(), |candidate| {
                    error.with_help(format!("did you mean `{candidate}`?"))
                }),
            );
        }
        trace.record(TraceEvent {
            event: "namespace.alias.member.resolved",
            rule: "TOPAL-NAMESPACE-ALIAS-001",
            detail: member_name,
        });
        let mut qualified = self.clone();
        qualified.bindings = namespace.bindings.clone();
        qualified.functions = namespace.functions.clone();
        qualified.generators = namespace.generators.clone();
        if remainder.is_empty() {
            return qualified.resolve_identifier(source, *member, trace);
        }
        let expression = Expression::Application {
            items: std::iter::once(Expression::Identifier(*member))
                .chain(remainder.iter().cloned())
                .collect(),
            span,
        };
        qualified.evaluate_expression(source, &expression, trace)
    }

    fn evaluate_root_qualified_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            Expression::Identifier(_),
            Expression::Identifier(member),
            remainder @ ..,
        ] = items
        else {
            unreachable!("preselected root-qualified application")
        };
        let member_name = source.slice(*member);
        trace.record(TraceEvent {
            event: "namespace.member.resolved",
            rule: "TOPAL-NAMESPACE-ROOT-001",
            detail: member_name,
        });
        if remainder.is_empty() {
            return self.resolve_identifier(source, *member, trace);
        }
        let expression = Expression::Application {
            items: std::iter::once(Expression::Identifier(*member))
                .chain(remainder.iter().cloned())
                .collect(),
            span,
        };
        self.evaluate_expression(source, &expression, trace)
    }

    fn evaluate_bound_named_function_call(
        &self,
        source: &SourceText,
        expression: &Expression,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(alias), _] = items else {
            unreachable!("preselected bound named function call")
        };
        let alias = source.slice(*alias);
        let Some(Value::NamedFunction(function)) = self.bindings.get(alias) else {
            unreachable!("preselected named function binding")
        };
        let mut invocation = self.clone();
        invocation.bindings.remove(alias);
        invocation
            .functions
            .insert(alias.to_owned(), function.candidates.clone());
        trace.record(TraceEvent {
            event: "function.value.called",
            rule: "TOPAL-FUNCTION-VALUE-001",
            detail: &function.name,
        });
        invocation.evaluate_expression(source, expression, trace)
    }

    fn evaluate_bound_callable_call(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name), argument] = items else {
            unreachable!("preselected bound callable call")
        };
        let Value::Callable(kind) = self.resolve_identifier(source, *name, trace)? else {
            unreachable!("preselected callable binding")
        };
        let argument_span = argument.span();
        let argument = self.evaluate_expression(source, argument, trace)?;
        trace.record(TraceEvent {
            event: "function.callable.called",
            rule: "TOPAL-FUNCTION-CALLABLE-VALUE-001",
            detail: callable_name(kind),
        });
        match argument {
            Value::Tuple(mut operands) if operands.len() == 2 => {
                let right = operands.pop().expect("two operands");
                let left = operands.pop().expect("two operands");
                apply_binary(
                    source,
                    kind,
                    left,
                    right,
                    (span, argument_span, argument_span),
                    trace,
                )
            }
            operand if kind == CallableKind::Minus => apply_negate(source, operand, span, trace),
            value => Err(diagnostic(
                source,
                "E-CALLABLE-ARGUMENT-PACKAGE",
                argument_span,
                format!(
                    "callable `{}` requires a two-field positional product, found `{}`",
                    callable_name(kind),
                    structural_value_classifier(&value)
                ),
            )),
        }
    }

    fn is_traversal_control_constructor(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(source.slice(*name), "Continue" | "Finish"))
    }

    fn is_iterate_construction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [_, Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
            if source.slice(*operation) == "iterate")
    }

    fn is_unfold_construction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [_, Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
            if source.slice(*operation) == "unfold")
    }

    fn construct_unfold_generator(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [seed, _, step_expression] = items else {
            unreachable!("preselected unfold construction")
        };
        let seed = self.evaluate_expression(source, seed, trace)?;
        let step = self.evaluate_expression(source, step_expression, trace)?;
        if !matches!(&step, Value::AnonymousFunction(function) if function.parameters.len() == 1) {
            return Err(diagnostic(
                source,
                "E-UNFOLD-FUNCTION-ARITY",
                step_expression.span(),
                "unfold step function requires exactly one seed parameter",
            ));
        }
        trace.record(TraceEvent {
            event: "generator.unfold.constructed",
            rule: "TOPAL-GENERATOR-UNFOLD-001",
            detail: &structural_value_classifier(&seed),
        });
        let value = Value::UnfoldGenerator {
            seed: Box::new(seed),
            step: Box::new(step),
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn is_iterate_take_while_construction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items,
            [_, Expression::Identifier(iterate), Expression::AnonymousFunction { .. }, Expression::Identifier(take_while), Expression::AnonymousFunction { .. }]
                if source.slice(*iterate) == "iterate" && source.slice(*take_while) == "take-while")
    }

    fn construct_iterate_take_while(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [initial, _, next_expression, _, predicate_expression] = items else {
            unreachable!("preselected iterate take-while construction")
        };
        let current = self.evaluate_expression(source, initial, trace)?;
        let classifier = structural_value_classifier(&current);
        let next = self.evaluate_expression(source, next_expression, trace)?;
        let predicate = self.evaluate_expression(source, predicate_expression, trace)?;
        for (value, expression, role) in [
            (&next, next_expression, "next"),
            (&predicate, predicate_expression, "predicate"),
        ] {
            if !matches!(value, Value::AnonymousFunction(function) if function.parameters.len() == 1)
            {
                return Err(diagnostic(
                    source,
                    "E-GENERATED-TRAVERSAL-FUNCTION-ARITY",
                    expression.span(),
                    format!("iterate {role} function requires exactly one parameter"),
                ));
            }
        }
        trace.record(TraceEvent {
            event: "generator.take-while.constructed",
            rule: "TOPAL-GENERATOR-TAKE-WHILE-001",
            detail: &classifier,
        });
        let value = Value::IterateGenerator {
            current: Box::new(current),
            next: Box::new(next),
            take_while: Some(Box::new(predicate)),
            classifier,
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn construct_iterate_generator(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [initial, _, next_expression] = items else {
            unreachable!("preselected iterate construction")
        };
        let current = self.evaluate_expression(source, initial, trace)?;
        let classifier = structural_value_classifier(&current);
        let next = self.evaluate_expression(source, next_expression, trace)?;
        let Value::AnonymousFunction(function) = &next else {
            unreachable!("iterate syntax requires an anonymous function")
        };
        if function.parameters.len() != 1 {
            return Err(diagnostic(
                source,
                "E-ITERATE-FUNCTION-ARITY",
                next_expression.span(),
                "iterate next function requires exactly one parameter",
            ));
        }
        trace.record(TraceEvent {
            event: "generator.iterate.constructed",
            rule: "TOPAL-GENERATOR-ITERATE-001",
            detail: &classifier,
        });
        let value = Value::IterateGenerator {
            current: Box::new(current),
            next: Box::new(next),
            take_while: None,
            classifier,
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn construct_traversal_control(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name), payload] = items else {
            unreachable!("preselected traversal-control constructor")
        };
        let payload = self.evaluate_expression(source, payload, trace)?;
        let constructor = source.slice(*name);
        trace.record(TraceEvent {
            event: "traversal.control.constructed",
            rule: "TOPAL-EXEC-TRAVERSAL-CONTROL-001",
            detail: constructor,
        });
        let value = if constructor == "Continue" {
            Value::Continue(Box::new(payload))
        } else {
            Value::Finish(Box::new(payload))
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn evaluate_bound_anonymous_call(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name), argument_expression] = items else {
            unreachable!("preselected bound anonymous call")
        };
        let function = self.resolve_identifier(source, *name, trace)?;
        let arity = match &function {
            Value::AnonymousFunction(function) => function.parameters.len(),
            _ => unreachable!("preselected anonymous binding"),
        };
        let argument = self.evaluate_expression(source, argument_expression, trace)?;
        let arguments = match (arity, argument) {
            (1, value) => vec![value],
            (_, Value::Tuple(values)) => values,
            (_, value) => {
                return Err(diagnostic(
                    source,
                    "E-ANONYMOUS-ARGUMENT-PACKAGE",
                    argument_expression.span(),
                    format!(
                        "anonymous function expects {arity} arguments packaged as a tuple, found `{}`",
                        structural_value_classifier(&value)
                    ),
                ));
            }
        };
        self.invoke_anonymous_function(&function, arguments, span, trace)
    }

    fn is_record_reconstruction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [_, Expression::Identifier(operation), Expression::Product { fields, .. }]
                if source.slice(*operation) == "with"
                    && !fields.is_empty()
                    && fields.iter().all(|field| field.label.is_some())
        )
    }

    fn evaluate_record_reconstruction(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            base,
            _,
            Expression::Product {
                fields: replacements,
                ..
            },
        ] = items
        else {
            unreachable!("preselected record reconstruction")
        };
        let Value::Record(mut fields) = self.evaluate_expression(source, base, trace)? else {
            return Err(diagnostic(
                source,
                "E-RECONSTRUCT-NON-RECORD",
                base.span(),
                "`with` reconstruction requires a labeled product",
            ));
        };
        let mut replaced = BTreeSet::new();
        for replacement in replacements {
            let label_span = replacement.label.expect("preselected labeled replacement");
            let label = source.slice(label_span);
            if !replaced.insert(label) {
                return Err(diagnostic(
                    source,
                    "E-DUPLICATE-RECONSTRUCTION-FIELD",
                    label_span,
                    format!("field `{label}` is replaced more than once"),
                ));
            }
            let Some((_, value)) = fields.iter_mut().find(|(name, _)| name == label) else {
                return Err(diagnostic(
                    source,
                    "E-NO-SUCH-RECORD-FIELD",
                    label_span,
                    format!("record has no field named `{label}`"),
                ));
            };
            *value = self.evaluate_expression(source, &replacement.value, trace)?;
            trace.record(TraceEvent {
                event: "record.field.replaced",
                rule: "TOPAL-TYPE-RECONSTRUCT-001",
                detail: label,
            });
        }
        let result = Value::Record(fields);
        self.checkpoint(trace, Some(&result), Some(span));
        Ok(result)
    }

    fn evaluate_range_selection(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [collection, Expression::Identifier(operation), selector] = items else {
            unreachable!("preselected range selection")
        };
        let collection_value = self.evaluate_expression(source, collection, trace)?;
        let selector_value = self.evaluate_expression(source, selector, trace)?;
        let Value::IntRange { lower, upper } = selector_value else {
            return Err(diagnostic(
                source,
                "E-SELECTION-RANGE",
                selector.span(),
                "range selection requires Range Int",
            ));
        };
        let operation = source.slice(*operation);
        let result = match collection_value {
            Value::List {
                element_classifier,
                entries,
            } if operation == "select-index" => {
                let entries = entries
                    .into_iter()
                    .enumerate()
                    .filter(|(index, _)| {
                        let index = BigInt::from(*index);
                        index >= lower && index <= upper
                    })
                    .map(|(_, value)| value)
                    .collect();
                Value::List {
                    element_classifier,
                    entries,
                }
            }
            Value::List {
                element_classifier,
                entries,
            } if operation == "select" => {
                let entries = entries
                    .into_iter()
                    .filter(|value| {
                        matches!(value, Value::Int(candidate) if candidate >= &lower && candidate <= &upper)
                    })
                    .collect();
                Value::List {
                    element_classifier,
                    entries,
                }
            }
            Value::String(text) if operation == "select-index" => {
                let selected = characters(&text)
                    .enumerate()
                    .filter(|(index, _)| {
                        let index = BigInt::from(*index);
                        index >= lower && index <= upper
                    })
                    .map(|(_, character)| character)
                    .collect::<String>();
                Value::String(selected)
            }
            value => {
                return Err(diagnostic(
                    source,
                    "E-SELECTION-SOURCE",
                    collection.span(),
                    format!(
                        "{operation} range has no overload for `{}`",
                        structural_value_classifier(&value)
                    ),
                ));
            }
        };
        trace.record(TraceEvent {
            event: "collection.range.selected",
            rule: if operation == "select-index" {
                "TOPAL-RANGE-INDEX-SELECTION-001"
            } else {
                "TOPAL-RANGE-VALUE-SELECTION-001"
            },
            detail: operation,
        });
        self.checkpoint(trace, Some(&result), Some(span));
        Ok(result)
    }

    fn apply_explicit_modulo(
        &self,
        source: &SourceText,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [operand, _, Expression::Identifier(type_name)] = items else {
            unreachable!("preselected explicit modular reduction")
        };
        let operand_value = self.evaluate_expression(source, operand, trace)?;
        let Value::Int(value) = operand_value else {
            return Err(diagnostic(
                source,
                "E-MODULO-OPERAND",
                operand.span(),
                "explicit modulo construction requires Int",
            ));
        };
        let name = source.slice(*type_name);
        let Value::ModularType(kind) = self.bindings.get(name).expect("known modular type") else {
            unreachable!("preselected modular type")
        };
        let value = reduce_modular(value, &kind.lower, &kind.upper);
        trace.record(TraceEvent {
            event: "numeric.modular.reduced",
            rule: "TOPAL-NUM-MODULAR-REDUCE-001",
            detail: name,
        });
        Ok(Value::Modular {
            type_name: name.into(),
            lower: kind.lower.clone(),
            upper: kind.upper.clone(),
            value,
        })
    }

    fn construct_modular_type(
        &self,
        source: &SourceText,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(kind), range] = items else {
            unreachable!("preselected modular type definition")
        };
        let signed = source.slice(*kind) == "ModInt";
        let range_value = self.evaluate_expression(source, range, trace)?;
        let Value::IntRange { lower, upper } = range_value else {
            return Err(diagnostic(
                source,
                "E-MODULAR-RANGE",
                range.span(),
                "ModNat and ModInt require a finite Int range",
            ));
        };
        if lower > BigInt::from(0)
            || upper < BigInt::from(0)
            || (!signed && lower != BigInt::from(0))
        {
            return Err(diagnostic(
                source,
                "E-MODULAR-RANGE",
                range.span(),
                "modular range must contain zero and ModNat must begin at zero",
            ));
        }
        trace.record(TraceEvent {
            event: "numeric.modular.type.constructed",
            rule: "TOPAL-NUM-MODULAR-TYPE-001",
            detail: source.slice(*kind),
        });
        Ok(Value::ModularType(Box::new(ModularType {
            name: None,
            signed,
            lower,
            upper,
        })))
    }

    fn is_modular_construction(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _] if matches!(self.bindings.get(source.slice(*name)), Some(Value::ModularType(_))))
    }

    fn construct_modular_value(
        &self,
        source: &SourceText,
        items: &[Expression],
        _span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name_span), operand] = items else {
            unreachable!("preselected modular construction")
        };
        let name = source.slice(*name_span);
        let Value::ModularType(kind) = self.bindings.get(name).expect("known modular type") else {
            unreachable!("preselected modular type")
        };
        let operand_value = self.evaluate_expression(source, operand, trace)?;
        let Value::Int(value) = operand_value else {
            return Err(diagnostic(
                source,
                "E-MODULAR-CONSTRUCTION-OPERAND",
                operand.span(),
                "modular construction requires Int",
            ));
        };
        if value < kind.lower || value > kind.upper {
            if expression_is_closed(operand) {
                return Err(diagnostic(
                    source,
                    "E-MODULAR-OUT-OF-RANGE",
                    operand.span(),
                    format!("value is outside `{name}` canonical range"),
                ));
            }
            let position = source.position(operand.span().start);
            return Ok(Value::Error {
                domain: format!("root.{name}(Int)"),
                code: "out-of-range".into(),
                line: position.line,
                column: position.column,
            });
        }
        trace.record(TraceEvent {
            event: "numeric.modular.constructed",
            rule: "TOPAL-NUM-MODULAR-CONSTRUCT-001",
            detail: name,
        });
        Ok(Value::Modular {
            type_name: name.into(),
            lower: kind.lower.clone(),
            upper: kind.upper.clone(),
            value,
        })
    }

    fn construct_constraint(
        &self,
        source: &SourceText,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(base), _, predicate] = items else {
            unreachable!("preselected constraint definition")
        };
        let base_classifier = source.slice(*base);
        if !matches!(
            base_classifier,
            "Boolean" | "Int" | "Nat" | "Rational" | "String"
        ) {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-BASE",
                *base,
                "constraint base must be a supported value classifier",
            ));
        }
        let predicate = self.evaluate_expression(source, predicate, trace)?;
        trace.record(TraceEvent {
            event: "constraint.constructed",
            rule: "TOPAL-TYPE-CONSTRAINT-001",
            detail: base_classifier,
        });
        Ok(Value::Constraint(Box::new(ConstraintValue {
            name: None,
            base_classifier: base_classifier.into(),
            predicate,
        })))
    }

    fn is_constraint_application(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _] if matches!(self.bindings.get(source.slice(*name)), Some(Value::Constraint(_))))
    }

    fn apply_constraint(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name_span), operand] = items else {
            unreachable!("preselected constraint application")
        };
        let name = source.slice(*name_span);
        let Value::Constraint(constraint) = self.bindings.get(name).expect("known constraint")
        else {
            unreachable!("preselected constraint value")
        };
        let value = self.evaluate_expression(source, operand, trace)?;
        if !value_has_classifier(&value, &constraint.base_classifier) {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-OPERAND",
                operand.span(),
                format!(
                    "constraint `{name}` requires `{}`",
                    constraint.base_classifier
                ),
            ));
        }
        let decision = self.invoke_anonymous_function(
            &constraint.predicate,
            vec![value.clone()],
            span,
            trace,
        )?;
        let Value::Boolean(accepted) = decision else {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-PREDICATE-RESULT",
                operand.span(),
                "constraint predicate must return Boolean",
            ));
        };
        trace.record(TraceEvent {
            event: "constraint.validated",
            rule: "TOPAL-TYPE-CONSTRAINT-VALIDATE-001",
            detail: if accepted { "accepted" } else { "rejected" },
        });
        if accepted {
            return Ok(Value::Refined {
                constraint: name.into(),
                base_classifier: constraint.base_classifier.clone(),
                value: Box::new(value),
            });
        }
        if expression_is_closed(operand) {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-REJECTED",
                operand.span(),
                format!("value does not satisfy constraint `{name}`"),
            ));
        }
        let position = source.position(operand.span().start);
        Ok(Value::Error {
            domain: format!("root.{name}({})", constraint.base_classifier),
            code: "out-of-range".into(),
            line: position.line,
            column: position.column,
        })
    }

    fn is_characters_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items.first(), Some(Expression::Identifier(operation)) if source.slice(*operation) == "characters")
    }

    fn evaluate_characters_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let text = items.get(1).expect("characters application has text");
        let text_span = text.span();
        let text = self.evaluate_expression(source, text, trace)?;
        let Value::String(text) = text else {
            return Err(diagnostic(
                source,
                "E-CHARACTERS-OPERAND",
                text_span,
                "characters requires a String operand",
            ));
        };
        let value = if items.len() == 4 {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.characters(String)",
            });
            let mut collected = String::new();
            for character in characters(&text) {
                trace.record(TraceEvent {
                    event: "generator.yielded",
                    rule: "TOPAL-STRING-CHARACTERS-COLLECT-001",
                    detail: character,
                });
                collected.push_str(character);
            }
            trace.record(TraceEvent {
                event: "string.characters.collected",
                rule: "TOPAL-STRING-CHARACTERS-COLLECT-001",
                detail: "String",
            });
            Value::String(collected)
        } else {
            trace.record(TraceEvent {
                event: "generator.started",
                rule: "TOPAL-STRING-CHARACTERS-GENERATOR-001",
                detail: "Generator Character Unit Unit",
            });
            Value::CharacterGenerator {
                generated: characters(&text).map(str::to_owned).collect(),
                origin: "root.characters".to_owned(),
            }
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn construct_union_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [Expression::Identifier(constructor), payload] = items {
            return self.construct_union(source, *constructor, payload, span, trace);
        }
        let [
            Expression::Identifier(type_name),
            _,
            Expression::Integer(index),
            payload,
        ] = items
        else {
            unreachable!("preselected positional Variant constructor application")
        };
        let index_text = source.slice(*index);
        let key = format!("at {index_text}");
        let type_text = source.slice(*type_name);
        let classifier = self
            .union_types
            .get(type_text)
            .and_then(|alternatives| alternatives.get(&key))
            .and_then(Option::as_deref)
            .ok_or_else(|| {
                diagnostic(
                    source,
                    "E-VARIANT-INDEX",
                    *index,
                    "Variant alternative index is outside its declared bounds",
                )
            })?;
        let value = self.evaluate_expression(source, payload, trace)?;
        if !value_has_classifier(&value, classifier) {
            return Err(diagnostic(
                source,
                "E-VARIANT-PAYLOAD-CLASSIFIER",
                payload.span(),
                format!("Variant alternative {index_text} requires `{classifier}`"),
            ));
        }
        trace.record(TraceEvent {
            event: "variant.constructed",
            rule: "TOPAL-TYPE-VARIANT-001",
            detail: index_text,
        });
        Ok(Value::Union(Box::new(UnionValue {
            type_name: type_text.into(),
            alternative: key,
            payload_classifier: Some(classifier.into()),
            payload: Some(Box::new(value)),
        })))
    }

    fn construct_union(
        &self,
        source: &SourceText,
        constructor: Span,
        payload: &Expression,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let name = source.slice(constructor);
        let (type_name, classifier) = self
            .union_constructor(name)
            .expect("preselected payload Union constructor");
        let value = self.evaluate_expression(source, payload, trace)?;
        if !value_has_classifier(&value, classifier) {
            return Err(diagnostic(
                source,
                "E-UNION-PAYLOAD-CLASSIFIER",
                payload.span(),
                format!(
                    "Union constructor `{name}` requires `{classifier}`, found `{}`",
                    structural_value_classifier(&value)
                ),
            ));
        }
        trace.record(TraceEvent {
            event: "union.constructed",
            rule: "TOPAL-TYPE-UNION-001",
            detail: name,
        });
        let result = Value::Union(Box::new(UnionValue {
            type_name: type_name.to_owned(),
            alternative: name.to_owned(),
            payload_classifier: Some(classifier.to_owned()),
            payload: Some(Box::new(value)),
        }));
        self.checkpoint(trace, Some(&result), Some(span));
        Ok(result)
    }

    fn invoke_anonymous_function(
        &self,
        function: &Value,
        arguments: Vec<Value>,
        call_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::AnonymousFunction(function) = function else {
            unreachable!("anonymous invocation is dispatched only for an anonymous function")
        };
        let AnonymousFunction {
            source,
            parameters,
            body,
            bindings,
        } = function.as_ref();
        if parameters.len() != arguments.len() {
            return Err(diagnostic(
                source,
                "E-ANONYMOUS-FUNCTION-ARITY",
                call_span,
                format!(
                    "anonymous function expects {} arguments, found {}",
                    parameters.len(),
                    arguments.len()
                ),
            ));
        }
        let mut invocation = self.clone();
        invocation.bindings = bindings.clone();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            invocation.bindings.insert(parameter.clone(), argument);
        }
        let detail = format!("arguments={}", parameters.len());
        trace.record(TraceEvent {
            event: "function.anonymous.called",
            rule: "TOPAL-FUNCTION-ANONYMOUS-001",
            detail: &detail,
        });
        invocation.evaluate_expression(source, body, trace)
    }

    fn capture_anonymous_function(
        &self,
        source: &SourceText,
        parameters: &[Span],
        body: &Expression,
        trace: &mut impl TraceSink,
    ) -> Value {
        let parameters = parameters
            .iter()
            .map(|parameter| source.slice(*parameter).to_owned())
            .collect::<Vec<_>>();
        let detail = format!("parameters={}", parameters.len());
        trace.record(TraceEvent {
            event: "function.anonymous.captured",
            rule: "TOPAL-FUNCTION-ANONYMOUS-001",
            detail: &detail,
        });
        Value::AnonymousFunction(Box::new(AnonymousFunction {
            source: source.clone(),
            parameters,
            body: Box::new(body.clone()),
            bindings: self.bindings.clone(),
        }))
    }

    fn evaluate_list_insert_at(
        &self,
        source: &SourceText,
        list: Value,
        boundary: Option<&Expression>,
        inserted: Option<&Expression>,
        operation_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Some(boundary) = boundary else {
            return Err(diagnostic(
                source,
                "E-EXPECTED-OPERAND",
                operation_span,
                "expected a boundary after insert-at",
            ));
        };
        let Some(inserted) = inserted else {
            return Err(diagnostic(
                source,
                "E-EXPECTED-OPERAND",
                boundary.span(),
                "expected a value or List after the insertion boundary",
            ));
        };
        let boundary_value = self.evaluate_expression(source, boundary, trace)?;
        let inserted_value = self.evaluate_expression(source, inserted, trace)?;
        apply_list_insert_at(
            source,
            list,
            boundary_value,
            boundary.span(),
            inserted_value,
            inserted.span(),
            trace,
        )
    }

    #[allow(clippy::too_many_lines)] // Collection laws remain explicit in one isolated frame.
    fn evaluate_list_higher_order(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [collection, Expression::Identifier(operation_span), function] = items
            && matches!(
                source.slice(*operation_span),
                "map" | "select" | "remove-indexes" | "remove-values"
            )
        {
            return (|| {
                let collection_span = collection.span();
                let collection = self.evaluate_expression(source, collection, trace)?;
                let Value::List {
                    element_classifier,
                    entries,
                } = collection
                else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECTION-OPERATION-SOURCE",
                        collection_span,
                        format!(
                            "{} requires a homogeneous collection",
                            source.slice(*operation_span)
                        ),
                    ));
                };
                let function_span = function.span();
                let function = self.evaluate_expression(source, function, trace)?;
                let operation = source.slice(*operation_span);
                let mut output = Vec::new();
                for (index, entry) in entries.into_iter().enumerate() {
                    let input = entry.clone();
                    let argument = if operation == "remove-indexes" {
                        Value::Int(BigInt::from(index))
                    } else {
                        entry
                    };
                    let transformed =
                        self.invoke_anonymous_function(&function, vec![argument], span, trace)?;
                    if matches!(operation, "select" | "remove-indexes" | "remove-values") {
                        let Value::Boolean(retain) = transformed else {
                            return Err(diagnostic(
                                source,
                                "E-SELECT-PREDICATE-RESULT",
                                function_span,
                                format!("{operation} predicate must return Boolean"),
                            ));
                        };
                        if retain == (operation == "select") {
                            output.push(input);
                        }
                    } else {
                        output.push(transformed);
                    }
                }
                let output_classifier = if operation != "map" || output.is_empty() {
                    element_classifier
                } else {
                    let classifier = structural_value_classifier(&output[0]);
                    if output
                        .iter()
                        .any(|value| structural_value_classifier(value) != classifier)
                    {
                        return Err(diagnostic(
                            source,
                            "E-MAP-RESULT-CLASSIFIER",
                            function_span,
                            "map transformation returned values with different classifiers",
                        ));
                    }
                    classifier
                };
                let selection = format!("root.{operation}(List {output_classifier})");
                trace.record(TraceEvent {
                    event: "operator.selected",
                    rule: "TOPAL-TYPE-CALL-001",
                    detail: &selection,
                });
                trace.record(TraceEvent {
                    event: match operation {
                        "map" => "list.mapped",
                        "select" => "list.selected",
                        _ => "list.entries.removed",
                    },
                    rule: match operation {
                        "map" => "TOPAL-COLLECTION-MAP-001",
                        "select" => "TOPAL-COLLECTION-SELECT-001",
                        "remove-indexes" => "TOPAL-LIST-REMOVE-INDEXES-001",
                        "remove-values" => "TOPAL-LIST-REMOVE-VALUES-001",
                        _ => unreachable!("known higher-order List operation"),
                    },
                    detail: &output_classifier,
                });
                let result = Value::List {
                    element_classifier: output_classifier,
                    entries: output,
                };
                self.checkpoint(trace, Some(&result), Some(span));
                Ok(result)
            })();
        }
        if let [
            collection,
            Expression::Identifier(operation),
            initial,
            function,
        ] = items
            && source.slice(*operation) == "fold"
        {
            return (|| {
                let collection_span = collection.span();
                let collection = self.evaluate_expression(source, collection, trace)?;
                let Value::List { entries, .. } = collection else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECTION-OPERATION-SOURCE",
                        collection_span,
                        "fold requires an ordered homogeneous collection",
                    ));
                };
                let mut state = self.evaluate_expression(source, initial, trace)?;
                let expected = structural_value_classifier(&state);
                let function_span = function.span();
                let function = self.evaluate_expression(source, function, trace)?;
                for entry in entries {
                    let transformed = self.invoke_anonymous_function(
                        &function,
                        vec![state.clone(), entry],
                        span,
                        trace,
                    )?;
                    state = match transformed {
                        Value::Continue(next) => *next,
                        Value::Finish(result) => {
                            let result = *result;
                            if !value_has_classifier(&result, &expected) {
                                return Err(diagnostic(
                                    source,
                                    "E-FOLD-FINISH-CLASSIFIER",
                                    function_span,
                                    format!("Finish result must satisfy `{expected}`"),
                                ));
                            }
                            trace.record(TraceEvent {
                                event: "traversal.finished",
                                rule: "TOPAL-EXEC-TRAVERSAL-CONTROL-001",
                                detail: "fold",
                            });
                            return Ok(result);
                        }
                        value => value,
                    };
                    if !value_has_classifier(&state, &expected) {
                        return Err(diagnostic(
                            source,
                            "E-FOLD-STATE-CLASSIFIER",
                            function_span,
                            format!("fold step must preserve state classifier `{expected}`"),
                        ));
                    }
                }
                trace.record(TraceEvent {
                    event: "list.folded",
                    rule: "TOPAL-COLLECTION-FOLD-001",
                    detail: &expected,
                });
                self.checkpoint(trace, Some(&state), Some(span));
                Ok(state)
            })();
        }
        unreachable!("higher-order List operation is preselected by its application shape")
    }

    #[allow(clippy::too_many_lines)] // Collector spellings and their distinct laws remain auditable together.
    fn evaluate_list_materialization(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [Expression::Identifier(operation), pairs] = items
            && source.slice(*operation) == "unzip"
        {
            let pairs_span = pairs.span();
            let pairs = self.evaluate_expression(source, pairs, trace)?;
            return apply_list_unzip(source, pairs, pairs_span, trace);
        }
        if let [Expression::Identifier(operation), collection] = items
            && source.slice(*operation) == "collect"
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            if let Value::IterateGenerator { .. } = value {
                return self.collect_iterate_generator(
                    source,
                    value,
                    collection.span(),
                    span,
                    trace,
                );
            }
            if let Value::UnfoldGenerator { .. } = value {
                return self.collect_unfold_generator(
                    source,
                    value,
                    collection.span(),
                    span,
                    trace,
                );
            }
            if matches!(value, Value::List { .. }) {
                trace.record(TraceEvent {
                    event: "list.collected",
                    rule: "TOPAL-COLLECTION-COLLECT-LIST-001",
                    detail: "List",
                });
                return Ok(value);
            }
            return Err(diagnostic(
                source,
                "E-COLLECT-SOURCE",
                collection.span(),
                "unary collect requires a finite homogeneous traversal",
            ));
        }
        if let [
            collection,
            Expression::Identifier(operation),
            Expression::Identifier(target),
        ] = items
            && source.slice(*operation) == "collect"
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            if source.slice(*target) == "Array" {
                let Value::List {
                    element_classifier,
                    entries,
                } = value
                else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECT-ARRAY-SOURCE",
                        collection.span(),
                        "Array collection requires a finite List",
                    ));
                };
                trace.record(TraceEvent {
                    event: "array.collected",
                    rule: "TOPAL-ARRAY-COLLECT-001",
                    detail: &format!("count={}", entries.len()),
                });
                return Ok(Value::Array {
                    element_classifier,
                    entries,
                });
            }
            if source.slice(*target) != "String" {
                return Err(diagnostic(
                    source,
                    "E-COLLECT-TARGET",
                    *target,
                    "implemented collectors are Array and String",
                ));
            }
            let Value::List { entries, .. } = value else {
                return Err(diagnostic(
                    source,
                    "E-COLLECT-SOURCE",
                    collection.span(),
                    "String collection requires a finite List of Character or String entries",
                ));
            };
            let mut text = String::new();
            for entry in entries {
                let Value::String(fragment) = entry else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECT-STRING-ENTRY",
                        collection.span(),
                        "String collection requires Character or String entries",
                    ));
                };
                text.push_str(&fragment);
            }
            trace.record(TraceEvent {
                event: "string.collected",
                rule: "TOPAL-COLLECTION-COLLECT-STRING-001",
                detail: "String",
            });
            return Ok(Value::String(text));
        }
        if let [Expression::Identifier(operation), collection] = items
            && matches!(source.slice(*operation), "collect-set" | "collect-bag")
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            return collect_unordered(
                source,
                source.slice(*operation),
                value,
                collection.span(),
                trace,
            );
        }
        if let [
            Expression::Identifier(operation),
            collection,
            Expression::Identifier(resolving),
            Expression::Identifier(policy),
        ] = items
            && source.slice(*operation) == "collect-map"
            && source.slice(*resolving) == "resolving"
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            return collect_map(
                source,
                value,
                source.slice(*policy),
                collection.span(),
                trace,
            );
        }
        if let [
            left_with_default,
            Expression::Identifier(operation),
            right_with_default,
        ] = items
            && source.slice(*operation) == "zip-longest"
        {
            let left = self.evaluate_expression(source, left_with_default, trace)?;
            let right = self.evaluate_expression(source, right_with_default, trace)?;
            return apply_list_zip_longest(source, left, right, span, trace);
        }
        Err(diagnostic(
            source,
            "E-UNSUPPORTED-COLLECTION-APPLICATION",
            span,
            "unsupported collection materialization form",
        ))
    }

    fn collect_iterate_generator(
        &self,
        source: &SourceText,
        generator: Value,
        source_span: Span,
        result_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::IterateGenerator {
            mut current,
            next,
            take_while,
            classifier,
        } = generator
        else {
            unreachable!("generated collection requires iterate generator")
        };
        let Some(predicate) = take_while else {
            return Err(diagnostic(
                source,
                "E-UNBOUNDED-GENERATOR-COLLECT",
                source_span,
                "collect requires a statically finite generated traversal",
            ));
        };
        let mut entries = Vec::new();
        loop {
            let accepted = self.invoke_anonymous_function(
                &predicate,
                vec![(*current).clone()],
                result_span,
                trace,
            )?;
            let Value::Boolean(accepted) = accepted else {
                return Err(diagnostic(
                    source,
                    "E-TAKE-WHILE-PREDICATE-RESULT",
                    source_span,
                    "take-while predicate must return Boolean",
                ));
            };
            if !accepted {
                break;
            }
            entries.push((*current).clone());
            let next_value =
                self.invoke_anonymous_function(&next, vec![*current], result_span, trace)?;
            if !value_has_classifier(&next_value, &classifier) {
                return Err(diagnostic(
                    source,
                    "E-ITERATE-NEXT-CLASSIFIER",
                    source_span,
                    format!("iterate next function must return `{classifier}`"),
                ));
            }
            *current = next_value;
        }
        trace.record(TraceEvent {
            event: "generator.collected",
            rule: "TOPAL-GENERATOR-COLLECT-001",
            detail: &classifier,
        });
        let value = Value::List {
            element_classifier: classifier,
            entries,
        };
        self.checkpoint(trace, Some(&value), Some(result_span));
        Ok(value)
    }

    fn collect_unfold_generator(
        &self,
        source: &SourceText,
        generator: Value,
        source_span: Span,
        result_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::UnfoldGenerator { mut seed, step } = generator else {
            unreachable!("unfold collection requires unfold generator")
        };
        let seed_classifier = structural_value_classifier(&seed);
        let mut element_classifier = None;
        let mut entries = Vec::new();
        loop {
            let result = self.invoke_anonymous_function(&step, vec![*seed], result_span, trace)?;
            let Value::Optional { payload, .. } = result else {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-STEP-RESULT",
                    source_span,
                    "unfold step must return Optional (Yield, Seed)",
                ));
            };
            let Some(payload) = payload else {
                break;
            };
            let Value::Tuple(mut pair) = *payload else {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-STEP-RESULT",
                    source_span,
                    "unfold Some payload must be a two-field positional product",
                ));
            };
            if pair.len() != 2 {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-STEP-RESULT",
                    source_span,
                    "unfold Some payload must contain yielded value and next seed",
                ));
            }
            let next_seed = pair.pop().expect("two-field unfold payload");
            let yielded = pair.pop().expect("two-field unfold payload");
            if !value_has_classifier(&next_seed, &seed_classifier) {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-SEED-CLASSIFIER",
                    source_span,
                    format!("unfold next seed must satisfy `{seed_classifier}`"),
                ));
            }
            let yielded_classifier = structural_value_classifier(&yielded);
            if element_classifier
                .as_ref()
                .is_some_and(|expected| expected != &yielded_classifier)
            {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-YIELD-CLASSIFIER",
                    source_span,
                    "unfold step yielded inconsistent value classifiers",
                ));
            }
            element_classifier.get_or_insert(yielded_classifier);
            trace.record(TraceEvent {
                event: "generator.yielded",
                rule: "TOPAL-GENERATOR-UNFOLD-COLLECT-001",
                detail: &yielded.to_string(),
            });
            entries.push(yielded);
            *seed = next_seed;
        }
        let element_classifier = element_classifier.unwrap_or_else(|| "Value".into());
        trace.record(TraceEvent {
            event: "generator.collected",
            rule: "TOPAL-GENERATOR-UNFOLD-COLLECT-001",
            detail: &element_classifier,
        });
        let value = Value::List {
            element_classifier,
            entries,
        };
        self.checkpoint(trace, Some(&value), Some(result_span));
        Ok(value)
    }
}

fn known_enum_alternatives(session: &Session, type_name: &str) -> Option<BTreeSet<String>> {
    if type_name == "Comparison" {
        return Some(
            ["Less", "Equal", "Greater"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        );
    }
    session.enum_types.get(type_name).cloned()
}

impl Execution {
    #[allow(clippy::too_many_lines)] // Traversal keeps consumption, action, resume, and return auditable together.
    fn execute_foreach(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        source: &Expression,
        binding: Span,
        body: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let (generated, origin, returned, returned_classifier) = match source {
            Expression::Identifier(name) => {
                let name_text = self.source.slice(*name);
                let value = session.bindings.remove(name_text).ok_or_else(|| {
                    if session.consumed_names.contains(name_text) {
                        consumed_generator_diagnostic(&self.source, *name, name_text)
                    } else {
                        diagnostic(&self.source, "E-UNBOUND-NAME", *name, "name is not bound")
                    }
                })?;
                if let Value::SuspendedGenerator { .. } = value {
                    session.declared_names.remove(name_text);
                    session.consumed_names.insert(name_text.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.consumed",
                        rule: "TOPAL-GENERATOR-DECLARATION-001",
                        detail: name_text,
                    });
                    return self
                        .execute_suspended_foreach(session, trace, value, binding, body, span);
                }
                if let Value::IterateGenerator { .. } = value {
                    session.declared_names.remove(name_text);
                    session.consumed_names.insert(name_text.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.consumed",
                        rule: "TOPAL-GENERATOR-ITERATE-FOREACH-001",
                        detail: name_text,
                    });
                    return self
                        .execute_iterate_foreach(session, trace, value, binding, body, span);
                }
                let (generated, origin, returned, returned_classifier) = match value {
                    Value::CharacterGenerator { generated, origin } => (
                        generated.into_iter().map(Value::String).collect(),
                        origin,
                        Value::Unit,
                        "Unit",
                    ),
                    Value::CharacterReturningGenerator {
                        generated,
                        returned,
                        origin,
                    } => (
                        generated.into_iter().map(Value::String).collect(),
                        origin,
                        Value::String(returned),
                        "Character",
                    ),
                    Value::List {
                        element_classifier,
                        entries,
                    } => {
                        session.bindings.insert(
                            name_text.to_owned(),
                            Value::List {
                                element_classifier,
                                entries: entries.clone(),
                            },
                        );
                        session.declared_names.insert(name_text.to_owned());
                        (entries, "root.List".into(), Value::Unit, "Unit")
                    }
                    _ => return Err(foreach_source_diagnostic(&self.source, source.span())),
                };
                if origin != "root.List" {
                    session.declared_names.remove(name_text);
                    session.consumed_names.insert(name_text.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.consumed",
                        rule: "TOPAL-STRING-CHARACTERS-GENERATOR-001",
                        detail: name_text,
                    });
                }
                (generated, origin, returned, returned_classifier)
            }
            Expression::Application { items, .. } => {
                let [Expression::Identifier(operation), text] = items.as_slice() else {
                    return Err(foreach_source_diagnostic(&self.source, source.span()));
                };
                if self.source.slice(*operation) != "characters" {
                    return Err(foreach_source_diagnostic(&self.source, source.span()));
                }
                let text_value = session.evaluate_expression(&self.source, text, trace)?;
                let Value::String(text_value) = text_value else {
                    return Err(diagnostic(
                        &self.source,
                        "E-CHARACTERS-OPERAND",
                        text.span(),
                        "characters requires a String operand",
                    ));
                };
                (
                    characters(&text_value)
                        .map(|character| Value::String(character.to_owned()))
                        .collect(),
                    "root.characters".to_owned(),
                    Value::Unit,
                    "Unit",
                )
            }
            _ => return Err(foreach_source_diagnostic(&self.source, source.span())),
        };
        let traversal_rule = if origin == "root.characters" {
            "TOPAL-STRING-CHARACTERS-FOREACH-001"
        } else if origin == "root.List" {
            "TOPAL-COLLECTION-FOREACH-001"
        } else {
            "TOPAL-GENERATOR-FOREACH-001"
        };
        let binding_name = self.source.slice(binding).to_owned();
        for entry in &generated {
            let mut iteration = session.clone();
            iteration
                .bindings
                .insert(binding_name.clone(), entry.clone());
            iteration.declared_names.insert(binding_name.clone());
            trace.record(TraceEvent {
                event: "generator.yielded",
                rule: traversal_rule,
                detail: &entry.to_string(),
            });
            let mut body_execution = Self {
                source: self.source.clone(),
                statements: body.to_vec(),
                cursor: 0,
                return_classifier: None,
            };
            loop {
                match body_execution.step(&mut iteration, trace)? {
                    ExecutionStep::Advanced { .. } => {}
                    ExecutionStep::Complete(Value::Unit) => break,
                    ExecutionStep::Complete(_) => {
                        return Err(diagnostic(
                            &self.source,
                            "E-FOREACH-ACTION-RESULT",
                            statement_span(body.last().expect("foreach body is nonempty")),
                            "foreach action must return Unit",
                        ));
                    }
                    ExecutionStep::Returned { .. } => {
                        unreachable!("foreach body has no function return context")
                    }
                }
            }
            trace.record(TraceEvent {
                event: "generator.resumed",
                rule: traversal_rule,
                detail: "Unit",
            });
        }
        trace.record(TraceEvent {
            event: "generator.returned",
            rule: generator_return_rule(
                &origin,
                generated.is_empty(),
                returned_classifier,
                traversal_rule,
            ),
            detail: returned_classifier,
        });
        Ok((returned, span))
    }

    fn execute_iterate_foreach(
        &self,
        session: &Session,
        trace: &mut impl TraceSink,
        generator: Value,
        binding: Span,
        body: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let Value::IterateGenerator {
            mut current,
            next,
            take_while,
            classifier,
        } = generator
        else {
            unreachable!("iterate traversal requires iterate generator")
        };
        let Some(predicate) = take_while else {
            return Err(diagnostic(
                &self.source,
                "E-UNBOUNDED-GENERATOR-TRAVERSAL",
                span,
                "complete foreach traversal of unbounded iterate requires a stopping transformation",
            ));
        };
        let binding_name = self.source.slice(binding).to_owned();
        loop {
            let accepted = session.invoke_anonymous_function(
                &predicate,
                vec![(*current).clone()],
                span,
                trace,
            )?;
            let Value::Boolean(accepted) = accepted else {
                return Err(diagnostic(
                    &self.source,
                    "E-TAKE-WHILE-PREDICATE-RESULT",
                    span,
                    "take-while predicate must return Boolean",
                ));
            };
            if !accepted {
                trace.record(TraceEvent {
                    event: "generator.returned",
                    rule: "TOPAL-GENERATOR-TAKE-WHILE-001",
                    detail: "Unit",
                });
                return Ok((Value::Unit, span));
            }
            trace.record(TraceEvent {
                event: "generator.yielded",
                rule: "TOPAL-GENERATOR-ITERATE-FOREACH-001",
                detail: &current.to_string(),
            });
            let mut iteration = session.clone();
            iteration
                .bindings
                .insert(binding_name.clone(), (*current).clone());
            iteration.declared_names.insert(binding_name.clone());
            let mut body_execution = Self {
                source: self.source.clone(),
                statements: body.to_vec(),
                cursor: 0,
                return_classifier: None,
            };
            loop {
                match body_execution.step(&mut iteration, trace)? {
                    ExecutionStep::Advanced { .. } => {}
                    ExecutionStep::Complete(Value::Unit) => break,
                    ExecutionStep::Complete(_) => {
                        return Err(diagnostic(
                            &self.source,
                            "E-FOREACH-ACTION-RESULT",
                            statement_span(body.last().expect("foreach body is nonempty")),
                            "foreach action must return Unit",
                        ));
                    }
                    ExecutionStep::Returned { .. } => {
                        unreachable!("foreach body has no function return context")
                    }
                }
            }
            let next_value =
                session.invoke_anonymous_function(&next, vec![*current], span, trace)?;
            if !value_has_classifier(&next_value, &classifier) {
                return Err(diagnostic(
                    &self.source,
                    "E-ITERATE-NEXT-CLASSIFIER",
                    span,
                    format!("iterate next function must return `{classifier}`"),
                ));
            }
            *current = next_value;
            trace.record(TraceEvent {
                event: "generator.resumed",
                rule: "TOPAL-GENERATOR-ITERATE-FOREACH-001",
                detail: "Unit",
            });
        }
    }

    #[allow(clippy::too_many_lines)] // State restoration and suspension order remain explicit and auditable.
    fn execute_suspended_foreach(
        &self,
        session: &Session,
        trace: &mut impl TraceSink,
        mut generator: Value,
        binding: Span,
        body: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let Value::SuspendedGenerator {
            source,
            body: generator_body,
            ref mut cursor,
            ref mut bindings,
            ref mut scope_state,
            ref mut pending_yield,
            ref mut resume_binding,
            ref mut returned,
            yield_classifier,
            return_classifier,
            origin,
        } = generator
        else {
            unreachable!("caller selects a suspended generator")
        };
        let binding_name = self.source.slice(binding).to_owned();
        let mut yielded_any = pending_yield.is_some();
        loop {
            if let Some(yielded) = pending_yield.take() {
                yielded_any = true;
                let detail = yielded.to_string();
                trace.record(TraceEvent {
                    event: "generator.yielded",
                    rule: "TOPAL-GENERATOR-FOREACH-001",
                    detail: &detail,
                });
                let mut iteration = session.clone();
                iteration.bindings.insert(binding_name.clone(), *yielded);
                iteration.declared_names.insert(binding_name.clone());
                let mut action = Self {
                    source: self.source.clone(),
                    statements: body.to_vec(),
                    cursor: 0,
                    return_classifier: None,
                };
                loop {
                    match action.step(&mut iteration, trace)? {
                        ExecutionStep::Advanced { .. } => {}
                        ExecutionStep::Complete(Value::Unit) => break,
                        ExecutionStep::Complete(_) => {
                            return Err(diagnostic(
                                &self.source,
                                "E-FOREACH-ACTION-RESULT",
                                statement_span(body.last().expect("foreach body is nonempty")),
                                "foreach action must return Unit",
                            ));
                        }
                        ExecutionStep::Returned { .. } => unreachable!("foreach cannot return"),
                    }
                }
                trace.record(TraceEvent {
                    event: "generator.resumed",
                    rule: "TOPAL-GENERATOR-FOREACH-001",
                    detail: "Unit",
                });
                let mut scope = session.clone();
                scope.bindings = std::mem::take(bindings);
                scope.functions = std::mem::take(&mut scope_state.functions);
                scope.declared_names = std::mem::take(&mut scope_state.declared_names);
                scope.local_function_names = std::mem::take(&mut scope_state.local_function_names);
                scope.enum_types = std::mem::take(&mut scope_state.enum_types);
                if let Some(name) = resume_binding.take() {
                    scope.bindings.insert(name.clone(), Value::Unit);
                    scope.declared_names.insert(name.clone());
                    trace.record(TraceEvent {
                        event: "generator.resume.bound",
                        rule: "TOPAL-GENERATOR-RESUME-BINDING-001",
                        detail: &name,
                    });
                }
                let mut next_returned = returned.take().map(|value| *value);
                advance_custom_generator(
                    &source,
                    &generator_body,
                    cursor,
                    &mut scope,
                    pending_yield,
                    resume_binding,
                    &mut next_returned,
                    &yield_classifier,
                    &return_classifier,
                    origin.rsplit('.').next().unwrap_or(&origin),
                    trace,
                )?;
                *bindings = scope.bindings;
                scope_state.functions = scope.functions;
                scope_state.declared_names = scope.declared_names;
                scope_state.local_function_names = scope.local_function_names;
                scope_state.enum_types = scope.enum_types;
                *returned = next_returned.map(Box::new);
                continue;
            }
            let value = returned.take().map_or(Value::Unit, |value| *value);
            trace.record(TraceEvent {
                event: "generator.returned",
                rule: generator_return_rule(
                    &origin,
                    !yielded_any,
                    &return_classifier,
                    "TOPAL-GENERATOR-FOREACH-001",
                ),
                detail: &return_classifier,
            });
            return Ok((value, span));
        }
    }

    fn execute_discard(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        span: Span,
        value: &Expression,
    ) -> Result<(Value, Span), Diagnostic> {
        session.evaluate_expression(&self.source, value, trace)?;
        trace.record(TraceEvent {
            event: "binding.discarded",
            rule: "TOPAL-SYN-BIND-001",
            detail: "_",
        });
        Ok((Value::Unit, cover(span, value.span())))
    }

    #[allow(clippy::too_many_lines)] // Declaration validation and trace setup stay auditable together.
    fn declare_function(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        declaration: FunctionDeclaration<'_>,
    ) -> Result<(Value, Span), Diagnostic> {
        let FunctionDeclaration {
            name,
            is_static,
            parameters,
            result,
            body,
            span,
        } = declaration;
        let name_text = self.source.slice(name);
        if session.declared_names.contains(name_text)
            && !session.local_function_names.contains(name_text)
        {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                name,
                "name is already bound in this scope",
            ));
        }
        let result_text = self.source.slice(result);
        if !supported_value_classifier(result_text, &session.enum_types)
            && !session.union_types.contains_key(result_text)
        {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-RESULT-CLASSIFIER",
                result,
                "the result classifier is not supported by this interpreter subset",
            ));
        }
        validate_parameter_names(&self.source, parameters)?;
        let parameters = parameters
            .iter()
            .map(|parameter| {
                let classifier = self.source.slice(parameter.classifier);
                if !supported_value_classifier(classifier, &session.enum_types)
                    && !session.union_types.contains_key(classifier)
                {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNSUPPORTED-PARAMETER-CLASSIFIER",
                        parameter.classifier,
                        "the parameter classifier is not supported by this interpreter subset",
                    ));
                }
                Ok((
                    self.source.slice(parameter.name).to_owned(),
                    classifier.to_owned(),
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if session.local_function_names.contains(name_text)
            && session.functions[name_text].iter().any(|function| {
                function.is_static == is_static
                    && function
                        .parameters
                        .iter()
                        .map(|(_, classifier)| classifier)
                        .eq(parameters.iter().map(|(_, classifier)| classifier))
            })
        {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-FUNCTION-OVERLOAD",
                name,
                "an overload with the same input classifiers and staticness already exists",
            ));
        }
        let direct_termination_rule =
            prove_int_recursion(&self.source, name_text, &parameters, body);
        let mutual_edge = direct_termination_rule
            .is_none()
            .then(|| prove_mutual_int_recursion_edge(&self.source, name_text, &parameters, body))
            .flatten();
        let recursion_target = mutual_edge.as_ref().map(|(target, _)| target.clone());
        let termination_rule =
            direct_termination_rule.or_else(|| mutual_edge.as_ref().map(|(_, rule)| *rule));
        let rule = function_rule(is_static, parameters.len());
        let function = UserFunction {
            source: self.source.clone(),
            is_static,
            parameters,
            result: result_text.to_owned(),
            body: body.to_vec(),
            bindings: session.bindings.clone(),
            termination_rule,
            recursion_target: recursion_target.clone(),
        };
        if session.local_function_names.contains(name_text) {
            session.functions.get_mut(name_text).unwrap().push(function);
        } else {
            session
                .functions
                .insert(name_text.to_owned(), vec![function]);
        }
        session.bindings.remove(name_text);
        session.declared_names.insert(name_text.to_owned());
        session.local_function_names.insert(name_text.to_owned());
        trace.record(TraceEvent {
            event: "function.declared",
            rule,
            detail: name_text,
        });
        if result_success_classifier(result_text).is_some() {
            trace.record(TraceEvent {
                event: "function.result.contract",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: result_text,
            });
        }
        if let Some(termination_rule) = direct_termination_rule {
            trace.record(TraceEvent {
                event: "function.recursion.proven",
                rule: termination_rule,
                detail: name_text,
            });
        } else if let Some(target) = recursion_target {
            let detail = format!("{name_text}->{target}");
            trace.record(TraceEvent {
                event: "function.recursion.edge.candidate",
                rule: termination_rule.expect("a mutual edge has a termination rule"),
                detail: &detail,
            });
        }
        Ok((Value::Unit, span))
    }

    fn declare_generator(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        declaration: GeneratorDeclaration<'_>,
    ) -> Result<(Value, Span), Diagnostic> {
        let GeneratorDeclaration {
            name,
            parameters,
            yielded,
            resumed,
            result,
            body,
            span,
        } = declaration;
        if !session.call_stack.is_empty() {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-GENERATOR-SCOPE",
                name,
                "the implemented generator subset requires a root-namespace declaration",
            ));
        }
        let name_text = self.source.slice(name);
        if session.declared_names.contains(name_text) && !session.generators.contains_key(name_text)
        {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                name,
                "name is already bound in this scope",
            ));
        }
        validate_parameter_names(&self.source, parameters)?;
        let parameters = parameters
            .iter()
            .map(|parameter| {
                (
                    self.source.slice(parameter.name).to_owned(),
                    self.source.slice(parameter.classifier).to_owned(),
                )
            })
            .collect::<Vec<_>>();
        let yield_classifier = self.source.slice(yielded);
        let result_classifier = self.source.slice(result);
        if !parameters.iter().all(|(_, classifier)| {
            supported_generator_value_classifier(classifier, &session.enum_types)
        }) || !supported_generator_value_classifier(yield_classifier, &session.enum_types)
            || self.source.slice(resumed) != "Unit"
            || !supported_generator_value_classifier(result_classifier, &session.enum_types)
        {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-GENERATOR-SIGNATURE",
                span,
                "the implemented generator subset requires supported scalar, Optional, Range, or declared enum input/yield/return classifiers and Unit resume",
            ));
        }
        if !supported_generator_body(&self.source, body) {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-GENERATOR-BODY",
                span,
                "the implemented generator subset requires bindings, discarded computations, or yield statements followed by a final expression",
            ));
        }
        let overloads = session.generators.entry(name_text.to_owned()).or_default();
        if overloads.iter().any(|candidate| {
            candidate.parameters.len() == parameters.len()
                && candidate
                    .parameters
                    .iter()
                    .zip(&parameters)
                    .all(|((_, left), (_, right))| left == right)
        }) {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-GENERATOR-OVERLOAD",
                name,
                format!("generator overload `{name_text}` has the same input classifiers"),
            ));
        }
        overloads.push(UserGenerator {
            source: self.source.clone(),
            parameters,
            yielded: yield_classifier.to_owned(),
            result: result_classifier.to_owned(),
            body: body.to_vec(),
            bindings: session.bindings.clone(),
        });
        session.declared_names.insert(name_text.to_owned());
        trace.record(TraceEvent {
            event: "generator.declared",
            rule: "TOPAL-GENERATOR-DECLARATION-001",
            detail: name_text,
        });
        let classifier = format!("Generator {yield_classifier} Unit {result_classifier}");
        trace.record(TraceEvent {
            event: "generator.classified",
            rule: "TOPAL-GENERATOR-DECLARATION-001",
            detail: &classifier,
        });
        Ok((Value::Unit, span))
    }

    /// Execute one source statement.
    ///
    /// # Errors
    ///
    /// Returns a name-resolution or evaluation diagnostic at the failing step.
    #[allow(clippy::too_many_lines)] // Statement dispatch remains explicit and exhaustively typed.
    pub fn step(
        &mut self,
        session: &mut Session,
        trace: &mut impl TraceSink,
    ) -> Result<ExecutionStep, Diagnostic> {
        let statement = &self.statements[self.cursor];
        let (value, span) = match statement {
            Statement::Binding {
                name,
                classifier,
                value,
            } => match self.execute_binding(session, trace, *name, *classifier, value)? {
                BindingOutcome::Bound(value, span) => (value, span),
                BindingOutcome::Returned(value, span) => {
                    return Ok(ExecutionStep::Returned { value, span });
                }
            },
            Statement::Function {
                name,
                is_static,
                parameters,
                result,
                body,
                span,
            } => self.declare_function(
                session,
                trace,
                FunctionDeclaration {
                    name: *name,
                    is_static: *is_static,
                    parameters,
                    result: *result,
                    body,
                    span: *span,
                },
            )?,
            Statement::Generator {
                name,
                parameters,
                yielded,
                resumed,
                result,
                body,
                span,
            } => self.declare_generator(
                session,
                trace,
                GeneratorDeclaration {
                    name: *name,
                    parameters,
                    yielded: *yielded,
                    resumed: *resumed,
                    result: *result,
                    body,
                    span: *span,
                },
            )?,
            Statement::Union {
                name,
                alternatives,
                span,
            } => declare_union(&self.source, session, *name, alternatives, *span, trace)?,
            Statement::Foreach {
                result,
                source,
                binding,
                body,
                span,
            } => {
                let (value, span) =
                    self.execute_foreach(session, trace, source, *binding, body, *span)?;
                if let Some((result, classifier)) = result {
                    let name = self.source.slice(*result);
                    if session.declared_names.contains(name) {
                        return Err(diagnostic(
                            &self.source,
                            "E-DUPLICATE-BINDING",
                            *result,
                            "name is already bound in this scope",
                        ));
                    }
                    if let Some(classifier) = classifier {
                        let expected = self.source.slice(*classifier);
                        if !value_has_classifier(&value, expected) {
                            let found = structural_value_classifier(&value);
                            return Err(diagnostic(
                                &self.source,
                                "E-FOREACH-RESULT-CLASSIFIER",
                                *classifier,
                                format!(
                                    "foreach returned `{found}`, but binding `{name}` requires `{expected}`"
                                ),
                            )
                            .with_help(format!(
                                "use classifier `{found}` here or traverse a generator returning `{expected}`"
                            )));
                        }
                    }
                    session.bindings.insert(name.to_owned(), value.clone());
                    session.declared_names.insert(name.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.foreach.result.bound",
                        rule: "TOPAL-GENERATOR-FOREACH-RESULT-001",
                        detail: name,
                    });
                }
                (value, span)
            }
            Statement::Discard { span, value } => {
                self.execute_discard(session, trace, *span, value)?
            }
            Statement::Return { keyword, value } => {
                if self.return_classifier.is_none() {
                    return Err(diagnostic(
                        &self.source,
                        "E-RETURN-OUTSIDE-FUNCTION",
                        *keyword,
                        "`return` is valid only inside a function body",
                    ));
                }
                let span = cover(*keyword, value.span());
                let value = evaluate_expression_with_optional_context(
                    &self.source,
                    session,
                    value,
                    self.return_classifier.as_deref(),
                    trace,
                )?;
                let classifier = structural_value_classifier(&value);
                trace.record(TraceEvent {
                    event: "function.return.explicit",
                    rule: "TOPAL-FUNCTION-RETURN-001",
                    detail: &classifier,
                });
                session.checkpoint(trace, Some(&value), Some(span));
                self.cursor = self.statements.len();
                return Ok(ExecutionStep::Returned { value, span });
            }
            Statement::Expression(expression) => {
                if self.cursor + 1 != self.statements.len() {
                    return Err(diagnostic(
                        &self.source,
                        "E-DISCARDED-VALUE",
                        expression.span(),
                        "a non-final expression value cannot be discarded",
                    ));
                }
                let value = evaluate_expression_with_optional_context(
                    &self.source,
                    session,
                    expression,
                    self.return_classifier.as_deref(),
                    trace,
                )?;
                consume_generator_argument(&self.source, session, expression);
                (value, expression.span())
            }
        };
        self.cursor += 1;
        if self.cursor == self.statements.len() {
            record_result(trace, &value);
            session.checkpoint(trace, Some(&value), Some(span));
            Ok(ExecutionStep::Complete(value))
        } else {
            session.checkpoint(trace, Some(&value), Some(span));
            Ok(ExecutionStep::Advanced { value, span })
        }
    }

    #[allow(clippy::too_many_lines)] // Declaration specializations remain ordered before ordinary projection.
    fn execute_binding(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        name: Span,
        classifier: Option<Span>,
        initializer: &Expression,
    ) -> Result<BindingOutcome, Diagnostic> {
        let name_text = self.source.slice(name);
        if session.declared_names.contains(name_text) {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                name,
                "name is already bound in this scope",
            ));
        }
        if let Some((value, span)) = declare_enum(&self.source, name, initializer, session, trace)?
        {
            return Ok(BindingOutcome::Bound(value, span));
        }
        if let Some((value, span)) =
            declare_variant(&self.source, name, initializer, session, trace)
        {
            return Ok(BindingOutcome::Bound(value, span));
        }
        let mut evaluated =
            evaluate_binding_initializer(&self.source, session, initializer, classifier, trace)?;
        consume_generator_argument(&self.source, session, initializer);
        if let Some(classifier) = classifier {
            let classifier_text = self.source.slice(classifier);
            evaluated = narrow_rational_to_int(
                &self.source,
                initializer,
                evaluated,
                classifier_text,
                self.return_classifier.as_deref(),
                trace,
            )?;
            if matches!(evaluated, Value::Error { .. }) {
                let Some(return_classifier) = &self.return_classifier else {
                    return Err(diagnostic(
                        &self.source,
                        "E-RESULT-PROJECTION-OUTSIDE-FUNCTION",
                        initializer.span(),
                        "a failed Result cannot propagate from top-level execution",
                    ));
                };
                if result_success_classifier(return_classifier).is_none() {
                    return Err(diagnostic(
                        &self.source,
                        "E-RESULT-PROJECTION-INFALLIBLE",
                        initializer.span(),
                        format!(
                            "cannot propagate a failed Result from a function returning `{return_classifier}`"
                        ),
                    ));
                }
                trace.record(TraceEvent {
                    event: "result.error.projected",
                    rule: "TOPAL-TYPE-RESULT-PROJECT-001",
                    detail: name_text,
                });
                return Ok(BindingOutcome::Returned(evaluated, initializer.span()));
            }
            if !value_has_classifier(&evaluated, classifier_text) {
                if classifier_text == "Character"
                    && let Value::String(text) = &evaluated
                {
                    let count = character_count(text);
                    return Err(diagnostic(
                        &self.source,
                        "E-CHARACTER-CLASSIFIER",
                        initializer.span(),
                        format!(
                            "Character requires exactly one user-perceived character, but this String contains {count}"
                        ),
                    ));
                }
                return Err(diagnostic(
                    &self.source,
                    "E-BINDING-CLASSIFIER",
                    initializer.span(),
                    format!("initializer does not satisfy `{classifier_text}`"),
                ));
            }
            trace.record(TraceEvent {
                event: "result.success.projected",
                rule: "TOPAL-TYPE-RESULT-PROJECT-001",
                detail: name_text,
            });
        }
        if let Value::Constraint(constraint) = &mut evaluated {
            constraint.name = Some(name_text.to_owned());
        }
        if let Value::ModularType(kind) = &mut evaluated {
            kind.name = Some(name_text.to_owned());
        }
        session.bindings.insert(name_text.to_owned(), evaluated);
        session.functions.remove(name_text);
        session.declared_names.insert(name_text.to_owned());
        trace.record(TraceEvent {
            event: "binding.created",
            rule: "TOPAL-SYN-BIND-001",
            detail: name_text,
        });
        Ok(BindingOutcome::Bound(
            Value::Unit,
            cover(name, initializer.span()),
        ))
    }
}

fn evaluate_binding_initializer(
    source: &SourceText,
    session: &mut Session,
    initializer: &Expression,
    classifier: Option<Span>,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    evaluate_expression_with_optional_context(
        source,
        session,
        initializer,
        classifier.map(|classifier| source.slice(classifier)),
        trace,
    )
}

fn evaluate_expression_with_optional_context(
    source: &SourceText,
    session: &mut Session,
    expression: &Expression,
    expected_classifier: Option<&str>,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if let Some(element_classifier) = expected_classifier.and_then(list_element_classifier)
        && let Some(list) =
            evaluate_list_expression(source, session, expression, element_classifier, trace)?
    {
        return Ok(list);
    }
    let contextual_none = expected_classifier
        .and_then(optional_payload_classifier)
        .filter(
            |_| matches!(expression, Expression::Identifier(span) if source.slice(*span) == "None"),
        );
    let Some(payload_classifier) = contextual_none else {
        return session.evaluate_expression(source, expression, trace);
    };
    trace.record(TraceEvent {
        event: "optional.none.constructed",
        rule: "TOPAL-TYPE-OPTIONAL-CONTEXT-001",
        detail: payload_classifier,
    });
    Ok(Value::Optional {
        payload_classifier: payload_classifier.to_owned(),
        payload: None,
    })
}

fn evaluate_list_expression(
    source: &SourceText,
    session: &mut Session,
    expression: &Expression,
    element_classifier: &str,
    trace: &mut impl TraceSink,
) -> Result<Option<Value>, Diagnostic> {
    if matches!(expression, Expression::Identifier(span) if source.slice(*span) == "Empty") {
        trace.record(TraceEvent {
            event: "list.empty.constructed",
            rule: "TOPAL-TYPE-LIST-CONSTRUCT-001",
            detail: element_classifier,
        });
        return Ok(Some(Value::List {
            element_classifier: element_classifier.to_owned(),
            entries: Vec::new(),
        }));
    }
    let Expression::Application { items, span } = expression else {
        return Ok(None);
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Ok(None);
    };
    if source.slice(*constructor) != "Entry" {
        return Ok(None);
    }
    if fields.len() != 2 || fields.iter().any(|field| field.label.is_some()) {
        return Err(diagnostic(
            source,
            "E-LIST-ENTRY-SHAPE",
            *span,
            "Entry requires exactly `(value, remaining-list)`",
        )
        .with_help("write `Entry ( value, remaining-list )`"));
    }
    let entry = session.evaluate_expression(source, &fields[0].value, trace)?;
    if !value_has_classifier(&entry, element_classifier) {
        let found = structural_value_classifier(&entry);
        return Err(diagnostic(
            source,
            "E-LIST-ENTRY-CLASSIFIER",
            fields[0].value.span(),
            format!(
                "list entry has classifier `{found}`, but this list requires `{element_classifier}`"
            ),
        )
        .with_help(format!("use a `{element_classifier}` value for this entry")));
    }
    let Some(Value::List { mut entries, .. }) =
        evaluate_list_expression(source, session, &fields[1].value, element_classifier, trace)?
    else {
        return Err(diagnostic(
            source,
            "E-LIST-REMAINDER",
            fields[1].value.span(),
            "Entry requires another List as its remaining value",
        )
        .with_help("end the constructor chain with `Empty`"));
    };
    entries.insert(0, entry);
    trace.record(TraceEvent {
        event: "list.entry.constructed",
        rule: "TOPAL-TYPE-LIST-CONSTRUCT-001",
        detail: element_classifier,
    });
    Ok(Some(Value::List {
        element_classifier: element_classifier.to_owned(),
        entries,
    }))
}

fn expression_is_closed(expression: &Expression) -> bool {
    match expression {
        Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Integer(_)
        | Expression::Rational(_)
        | Expression::String(_)
        | Expression::Callable { .. } => true,
        Expression::Product { fields, .. } => fields
            .iter()
            .all(|field| expression_is_closed(&field.value)),
        Expression::Application { items, .. } => items.iter().all(expression_is_closed),
        Expression::AnonymousFunction { body, .. } => expression_is_closed(body),
        Expression::DecisionTable { .. } | Expression::Identifier(_) | Expression::Discard(_) => {
            false
        }
    }
}

fn narrow_rational_to_int(
    source: &SourceText,
    initializer: &Expression,
    value: Value,
    classifier: &str,
    return_classifier: Option<&str>,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if classifier != "Int" {
        return Ok(value);
    }
    let Value::Rational(value) = value else {
        return Ok(value);
    };
    if value.denom() != &BigInt::from(1) {
        if !expression_is_closed(initializer)
            && return_classifier.and_then(result_success_classifier) == Some("Int")
        {
            let position = source.position(initializer.span().start);
            trace.record(TraceEvent {
                event: "result.error.constructed",
                rule: "TOPAL-NUM-RATIONAL-INT-VALIDATE-001",
                detail: "root.Int(Rational);not-representable",
            });
            return Ok(Value::Error {
                domain: "root.Int(Rational)".to_owned(),
                code: "not-representable".to_owned(),
                line: position.line,
                column: position.column,
            });
        }
        return Err(diagnostic(
            source,
            "E-RATIONAL-NOT-EXACT-INT",
            initializer.span(),
            format!(
                "exact Rational result has denominator {}, so it cannot satisfy Int",
                value.denom()
            ),
        ));
    }
    trace.record(TraceEvent {
        event: "conversion.applied",
        rule: if expression_is_closed(initializer) {
            "TOPAL-NUM-RATIONAL-INT-EXACT-001"
        } else {
            "TOPAL-NUM-RATIONAL-INT-VALIDATE-001"
        },
        detail: if expression_is_closed(initializer) {
            "Rational->Int:exact"
        } else {
            "Rational->Int:validated"
        },
    });
    Ok(Value::Int(value.numer().clone()))
}

fn construct_int(
    source: &SourceText,
    operand: &Expression,
    value: Value,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Rational(value) = value else {
        if matches!(value, Value::Int(_)) {
            trace.record(TraceEvent {
                event: "numeric.int.constructed",
                rule: "TOPAL-NUM-INT-CONSTRUCT-001",
                detail: "Int->Int:identity",
            });
            return Ok(value);
        }
        return Err(diagnostic(
            source,
            "E-INT-CONSTRUCTOR-OPERAND",
            operand.span(),
            "Int construction requires an exact numeric operand",
        ));
    };
    if value.denom() == &BigInt::from(1) {
        trace.record(TraceEvent {
            event: "numeric.int.constructed",
            rule: "TOPAL-NUM-INT-CONSTRUCT-001",
            detail: "Rational->Int:exact",
        });
        return Ok(Value::Int(value.numer().clone()));
    }
    if expression_is_closed(operand) {
        return Err(diagnostic(
            source,
            "E-RATIONAL-NOT-EXACT-INT",
            operand.span(),
            format!(
                "exact Rational operand has denominator {}, so Int cannot represent it",
                value.denom()
            ),
        ));
    }
    let position = source.position(operand.span().start);
    trace.record(TraceEvent {
        event: "result.error.constructed",
        rule: "TOPAL-NUM-INT-CONSTRUCT-001",
        detail: "root.Int(Rational);not-representable",
    });
    Ok(Value::Error {
        domain: "root.Int(Rational)".to_owned(),
        code: "not-representable".to_owned(),
        line: position.line,
        column: position.column,
    })
}

fn construct_nat(
    source: &SourceText,
    operand: &Expression,
    value: Value,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Int(value) = value else {
        return Err(diagnostic(
            source,
            "E-NAT-CONSTRUCTOR-OPERAND",
            operand.span(),
            "Nat construction requires an Int operand",
        ));
    };
    if value >= BigInt::from(0) {
        trace.record(TraceEvent {
            event: "numeric.nat.constructed",
            rule: "TOPAL-NUM-NAT-CONSTRUCT-001",
            detail: "Int->Nat:nonnegative",
        });
        return Ok(Value::Int(value));
    }
    if expression_is_closed(operand) {
        return Err(diagnostic(
            source,
            "E-NAT-OUT-OF-RANGE",
            operand.span(),
            "a negative Int is outside the Nat constraint",
        ));
    }
    let position = source.position(operand.span().start);
    trace.record(TraceEvent {
        event: "result.error.constructed",
        rule: "TOPAL-NUM-NAT-CONSTRUCT-001",
        detail: "root.Nat(Int);out-of-range",
    });
    Ok(Value::Error {
        domain: "root.Nat(Int)".to_owned(),
        code: "out-of-range".to_owned(),
        line: position.line,
        column: position.column,
    })
}

fn construct_rational(
    source: &SourceText,
    operand: &Expression,
    value: Value,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if let Value::Int(value) = value {
        trace.record(TraceEvent {
            event: "numeric.rational.constructed",
            rule: "TOPAL-NUM-INT-RATIONAL-CONVERT-001",
            detail: "Int->Rational:explicit",
        });
        return Ok(Value::Rational(BigRational::from_integer(value)));
    }
    let Value::Tuple(values) = value else {
        return Err(diagnostic(
            source,
            "E-RATIONAL-CONSTRUCTOR-PRODUCT",
            operand.span(),
            "Rational construction requires a positional (numerator, denominator) product",
        ));
    };
    let [Value::Int(numerator), Value::Int(denominator)] = values.as_slice() else {
        return Err(diagnostic(
            source,
            "E-RATIONAL-CONSTRUCTOR-COMPONENTS",
            operand.span(),
            "Rational numerator and denominator must both be Int values",
        ));
    };
    if denominator == &BigInt::from(0) {
        let code = if numerator == &BigInt::from(0) {
            "indeterminate"
        } else {
            "division-by-zero"
        };
        if expression_is_closed(operand) {
            let (diagnostic_code, message) = if code == "indeterminate" {
                (
                    "E-INDETERMINATE-RATIONAL",
                    "Rational (0, 0) does not determine one numeric value",
                )
            } else {
                (
                    "E-DIVISION-BY-ZERO",
                    "a finite Rational constructor requires a nonzero denominator",
                )
            };
            return Err(diagnostic(source, diagnostic_code, operand.span(), message));
        }
        let position = source.position(operand.span().start);
        trace.record(TraceEvent {
            event: "result.error.constructed",
            rule: "TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001",
            detail: if code == "indeterminate" {
                "root.Rational(Int,Int);indeterminate"
            } else {
                "root.Rational(Int,Int);division-by-zero"
            },
        });
        return Ok(Value::Error {
            domain: "root.Rational(Int,Int)".to_owned(),
            code: code.to_owned(),
            line: position.line,
            column: position.column,
        });
    }
    let value = BigRational::new(numerator.clone(), denominator.clone());
    trace.record(TraceEvent {
        event: "numeric.rational.constructed",
        rule: if expression_is_closed(operand) {
            "TOPAL-NUM-RATIONAL-CONSTRUCT-001"
        } else {
            "TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001"
        },
        detail: if expression_is_closed(operand) {
            "canonical"
        } else {
            "canonical:validated"
        },
    });
    Ok(Value::Rational(value))
}

const fn cover(first: Span, second: Span) -> Span {
    Span {
        start: first.start,
        end: second.end,
    }
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::Binding { name, value, .. } => cover(*name, value.span()),
        Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Foreach { span, .. } => *span,
        Statement::Discard { span, value } => cover(*span, value.span()),
        Statement::Return { keyword, value } => cover(*keyword, value.span()),
        Statement::Expression(expression) => expression.span(),
    }
}

fn supported_generator_body(source: &SourceText, body: &[Statement]) -> bool {
    if !matches!(
        body.last(),
        Some(Statement::Expression(_) | Statement::Return { .. })
    ) {
        return false;
    }
    for statement in &body[..body.len().saturating_sub(1)] {
        if yielded_statement(source, statement).is_none()
            && !matches!(
                statement,
                Statement::Binding { .. }
                    | Statement::Discard { .. }
                    | Statement::Function { .. }
                    | Statement::Return { .. }
            )
        {
            return false;
        }
    }
    true
}

fn discarded_yield_expression<'a>(
    source: &SourceText,
    statement: &'a Statement,
) -> Option<&'a Expression> {
    let Statement::Discard {
        value: Expression::Application { items, .. },
        ..
    } = statement
    else {
        return None;
    };
    let [Expression::Identifier(keyword), yielded] = items.as_slice() else {
        return None;
    };
    (source.slice(*keyword) == "yield").then_some(yielded)
}

fn yielded_statement<'a>(
    source: &SourceText,
    statement: &'a Statement,
) -> Option<(Option<Span>, &'a Expression)> {
    if let Some(expression) = discarded_yield_expression(source, statement) {
        return Some((None, expression));
    }
    let Statement::Binding {
        name,
        value: Expression::Application { items, .. },
        ..
    } = statement
    else {
        return None;
    };
    let [Expression::Identifier(keyword), yielded] = items.as_slice() else {
        return None;
    };
    (source.slice(*keyword) == "yield").then_some((Some(*name), yielded))
}

#[allow(clippy::too_many_arguments)]
fn advance_custom_generator(
    source: &SourceText,
    body: &[Statement],
    cursor: &mut usize,
    scope: &mut Session,
    pending_yield: &mut Option<Box<Value>>,
    resume_binding: &mut Option<String>,
    returned: &mut Option<Value>,
    yield_classifier: &str,
    return_classifier: &str,
    name: &str,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    while *cursor < body.len() {
        let statement = &body[*cursor];
        *cursor += 1;
        if let Some((binding, expression)) = yielded_statement(source, statement) {
            let value = scope.evaluate_expression(source, expression, trace)?;
            if !value_has_classifier(&value, yield_classifier) {
                return Err(generator_classifier_diagnostic(
                    source,
                    "E-GENERATOR-YIELD-TYPE",
                    expression.span(),
                    name,
                    "yielded",
                    yield_classifier,
                    &value,
                ));
            }
            *pending_yield = Some(Box::new(value));
            *resume_binding = binding.map(|span| source.slice(span).to_owned());
            trace.record(TraceEvent {
                event: "generator.suspended",
                rule: "TOPAL-GENERATOR-SUSPEND-001",
                detail: name,
            });
            return Ok(());
        }
        if let Statement::Expression(expression) = statement {
            let value = scope.evaluate_expression(source, expression, trace)?;
            if !value_has_classifier(&value, return_classifier) {
                return Err(generator_classifier_diagnostic(
                    source,
                    "E-GENERATOR-RETURN-TYPE",
                    expression.span(),
                    name,
                    "returned",
                    return_classifier,
                    &value,
                ));
            }
            *returned = Some(value);
            return Ok(());
        }
        if let Statement::Return {
            value: expression, ..
        } = statement
        {
            let value = scope.evaluate_expression(source, expression, trace)?;
            if !value_has_classifier(&value, return_classifier) {
                return Err(generator_classifier_diagnostic(
                    source,
                    "E-GENERATOR-RETURN-TYPE",
                    expression.span(),
                    name,
                    "returned",
                    return_classifier,
                    &value,
                ));
            }
            trace.record(TraceEvent {
                event: "generator.return.explicit",
                rule: "TOPAL-GENERATOR-EXPLICIT-RETURN-001",
                detail: return_classifier,
            });
            *returned = Some(value);
            *cursor = body.len();
            return Ok(());
        }
        let mut execution = Execution {
            source: source.clone(),
            statements: vec![statement.clone()],
            cursor: 0,
            return_classifier: None,
        };
        match execution.step(scope, trace)? {
            ExecutionStep::Advanced { .. } | ExecutionStep::Complete(_) => {}
            ExecutionStep::Returned { .. } => unreachable!("generator bindings cannot return"),
        }
    }
    Ok(())
}

fn generator_return_rule(
    origin: &str,
    empty: bool,
    returned: &str,
    traversal_rule: &'static str,
) -> &'static str {
    if origin != "root.characters" && returned != "Unit" {
        "TOPAL-GENERATOR-FINAL-RETURN-001"
    } else if origin != "root.characters" && empty {
        "TOPAL-GENERATOR-EARLY-RETURN-001"
    } else {
        traversal_rule
    }
}

fn foreach_source_diagnostic(source: &SourceText, span: Span) -> Diagnostic {
    diagnostic(
        source,
        "E-FOREACH-SOURCE",
        span,
        "the implemented foreach subset requires `characters text`",
    )
}

fn consumed_generator_diagnostic(source: &SourceText, span: Span, name: &str) -> Diagnostic {
    diagnostic(
        source,
        "E-GENERATOR-CONSUMED",
        span,
        format!("generator `{name}` was already consumed"),
    )
    .with_help("construct a fresh generator before traversing it again")
}

fn consume_generator_argument(source: &SourceText, session: &mut Session, expression: &Expression) {
    let Expression::Application { items, .. } = expression else {
        return;
    };
    let [
        Expression::Identifier(function),
        Expression::Identifier(argument),
    ] = items.as_slice()
    else {
        return;
    };
    let function_name = source.slice(*function);
    let argument_name = source.slice(*argument);
    let accepts_generator = session
        .functions
        .get(function_name)
        .is_some_and(|candidates| {
            candidates.iter().any(|candidate| {
                matches!(
                    candidate.parameters.as_slice(),
                    [(_, classifier)] if classifier.starts_with("Generator ")
                )
            })
        });
    if accepts_generator
        && matches!(
            session.bindings.get(argument_name),
            Some(
                Value::CharacterGenerator { .. }
                    | Value::CharacterReturningGenerator { .. }
                    | Value::SuspendedGenerator { .. }
            )
        )
    {
        session.bindings.remove(argument_name);
        session.declared_names.remove(argument_name);
        session.consumed_names.insert(argument_name.to_owned());
    }
}

#[allow(clippy::too_many_lines)] // Close delivery, handler execution, and trace order stay auditable together.
fn close_remaining_character_generators(
    session: &mut Session,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    let generators = session
        .bindings
        .iter()
        .filter(|(_, value)| {
            matches!(
                value,
                Value::CharacterGenerator { .. }
                    | Value::CharacterReturningGenerator { .. }
                    | Value::SuspendedGenerator { .. }
            )
        })
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    for name in generators {
        let value = session
            .bindings
            .remove(&name)
            .expect("collected binding exists");
        session.declared_names.remove(&name);
        session.consumed_names.insert(name.clone());
        let origin = match &value {
            Value::CharacterGenerator { origin, .. }
            | Value::CharacterReturningGenerator { origin, .. }
            | Value::SuspendedGenerator { origin, .. } => origin.clone(),
            _ => unreachable!("only generators were collected"),
        };
        let detail = format!("domain=root;code=generator-closed;generator={origin}");
        trace.record(TraceEvent {
            event: "generator.close.signaled",
            rule: "TOPAL-GENERATOR-ERROR-CODE-001",
            detail: &detail,
        });
        if let Value::SuspendedGenerator {
            source,
            body,
            mut cursor,
            bindings,
            scope_state,
            pending_yield: _,
            resume_binding,
            returned,
            return_classifier,
            yield_classifier,
            ..
        } = value
            && let Some(resume_binding) = resume_binding
        {
            let mut pending_yield = None;
            let yield_span = body
                .get(cursor.saturating_sub(1))
                .map_or(Span::new(0, 0), statement_span);
            let position = source.position(yield_span.start);
            let mut scope = session.clone();
            scope.bindings = bindings;
            scope.functions = scope_state.functions;
            scope.declared_names = scope_state.declared_names;
            scope.local_function_names = scope_state.local_function_names;
            scope.enum_types = scope_state.enum_types;
            scope.bindings.insert(
                resume_binding.clone(),
                Value::Error {
                    domain: "root".into(),
                    code: "generator-closed".into(),
                    line: position.line,
                    column: position.column,
                },
            );
            scope.declared_names.insert(resume_binding.clone());
            trace.record(TraceEvent {
                event: "generator.close.bound",
                rule: "TOPAL-GENERATOR-CLOSE-HANDLER-001",
                detail: &resume_binding,
            });
            let mut handled_return = returned.map(|value| *value);
            let mut next_resume_binding = None;
            advance_custom_generator(
                &source,
                &body,
                &mut cursor,
                &mut scope,
                &mut pending_yield,
                &mut next_resume_binding,
                &mut handled_return,
                &yield_classifier,
                &return_classifier,
                origin.rsplit('.').next().unwrap_or(&origin),
                trace,
            )?;
            if pending_yield.is_some() {
                return Err(diagnostic(
                    &source,
                    "E-GENERATOR-YIELD-AFTER-CLOSE",
                    body.get(cursor.saturating_sub(1))
                        .map_or(yield_span, statement_span),
                    "a generator cannot yield again after observing `generator-closed`",
                ));
            }
        }
        trace.record(TraceEvent {
            event: "generator.closed",
            rule: if origin == "root.characters" {
                "TOPAL-STRING-CHARACTERS-CLOSE-001"
            } else {
                "TOPAL-GENERATOR-CLOSE-001"
            },
            detail: &origin,
        });
    }
    Ok(())
}

fn enum_alternatives(source: &SourceText, expression: &Expression) -> Option<Vec<(String, Span)>> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*constructor) != "Enum" {
        return None;
    }
    fields
        .iter()
        .map(|field| {
            let Expression::Identifier(alternative) = &field.value else {
                return None;
            };
            field
                .label
                .is_none()
                .then(|| (source.slice(*alternative).to_owned(), *alternative))
        })
        .collect()
}

fn variant_alternatives(source: &SourceText, expression: &Expression) -> Option<Vec<String>> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*constructor) != "Variant" {
        return None;
    }
    fields
        .iter()
        .map(|field| {
            field
                .label
                .is_none()
                .then(|| classifier_expression(source, &field.value))
                .flatten()
        })
        .collect()
}

fn evaluate_arithmetic_error_code(
    source: &SourceText,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Option<Value> {
    let [
        Expression::Identifier(lang),
        Expression::Identifier(arithmetic),
        Expression::Identifier(code),
    ] = items
    else {
        return None;
    };
    if source.slice(*lang) != "lang" || source.slice(*arithmetic) != "arithmetic" {
        return None;
    }
    let code = source.slice(*code);
    if !matches!(
        code,
        "out-of-range" | "not-representable" | "division-by-zero" | "indeterminate"
    ) {
        return None;
    }
    trace.record(TraceEvent {
        event: "namespace.member.selected",
        rule: "TOPAL-NUM-ARITHMETIC-ERROR-001",
        detail: code,
    });
    Some(Value::Enum {
        type_name: "lang arithmetic ArithmeticErrorCode".to_owned(),
        alternative: code.to_owned(),
    })
}

fn evaluate_generator_error_code(
    source: &SourceText,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Option<Value> {
    let [
        Expression::Identifier(lang),
        Expression::Identifier(generator),
        Expression::Identifier(code),
    ] = items
    else {
        return None;
    };
    if source.slice(*lang) != "lang"
        || source.slice(*generator) != "generator"
        || source.slice(*code) != "generator-closed"
    {
        return None;
    }
    trace.record(TraceEvent {
        event: "namespace.member.selected",
        rule: "TOPAL-GENERATOR-ERROR-CODE-001",
        detail: "generator-closed",
    });
    Some(Value::Enum {
        type_name: "lang generator GeneratorErrorCode".to_owned(),
        alternative: "generator-closed".to_owned(),
    })
}

fn declare_enum(
    source: &SourceText,
    name: Span,
    expression: &Expression,
    session: &mut Session,
    trace: &mut impl TraceSink,
) -> Result<Option<(Value, Span)>, Diagnostic> {
    let Some(alternatives) = enum_alternatives(source, expression) else {
        return Ok(None);
    };
    let name_text = source.slice(name);
    let mut seen = BTreeSet::new();
    for (alternative, span) in &alternatives {
        if !seen.insert(alternative.as_str())
            || session.declared_names.contains(alternative)
            || alternative == name_text
        {
            return Err(diagnostic(
                source,
                "E-DUPLICATE-ENUM-ALTERNATIVE",
                *span,
                format!("enum alternative `{alternative}` is already declared in this scope"),
            ));
        }
    }
    session.declared_names.insert(name_text.to_owned());
    session.enum_types.insert(
        name_text.to_owned(),
        alternatives
            .iter()
            .map(|(alternative, _)| alternative.clone())
            .collect(),
    );
    for (alternative, _) in alternatives {
        session.bindings.insert(
            alternative.clone(),
            Value::Enum {
                type_name: name_text.to_owned(),
                alternative: alternative.clone(),
            },
        );
        session.declared_names.insert(alternative);
    }
    trace.record(TraceEvent {
        event: "enum.declared",
        rule: "TOPAL-TYPE-ENUM-001",
        detail: name_text,
    });
    Ok(Some((Value::Unit, cover(name, expression.span()))))
}

fn declare_union(
    source: &SourceText,
    session: &mut Session,
    name: Span,
    alternatives: &[topal_syntax::UnionAlternative],
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<(Value, Span), Diagnostic> {
    let type_name = source.slice(name);
    if session.declared_names.contains(type_name) {
        return Err(diagnostic(
            source,
            "E-DUPLICATE-UNION",
            name,
            "name is already declared",
        ));
    }
    let mut declared = BTreeMap::new();
    for alternative in alternatives {
        let alternative_name = source.slice(alternative.name);
        if declared.contains_key(alternative_name) {
            return Err(diagnostic(
                source,
                "E-DUPLICATE-UNION-ALTERNATIVE",
                alternative.name,
                "Union alternative occurs more than once",
            ));
        }
        let classifier = alternative
            .classifier
            .map(|classifier| source.slice(classifier).to_owned());
        declared.insert(alternative_name.to_owned(), classifier.clone());
        if classifier.is_none() {
            session.bindings.insert(
                alternative_name.to_owned(),
                Value::Union(Box::new(UnionValue {
                    type_name: type_name.to_owned(),
                    alternative: alternative_name.to_owned(),
                    payload_classifier: None,
                    payload: None,
                })),
            );
        }
        session.declared_names.insert(alternative_name.to_owned());
    }
    session.union_types.insert(type_name.to_owned(), declared);
    session.declared_names.insert(type_name.to_owned());
    trace.record(TraceEvent {
        event: "union.declared",
        rule: "TOPAL-TYPE-UNION-001",
        detail: type_name,
    });
    Ok((Value::Unit, span))
}

fn declare_variant(
    source: &SourceText,
    name: Span,
    expression: &Expression,
    session: &mut Session,
    trace: &mut impl TraceSink,
) -> Option<(Value, Span)> {
    let alternatives = variant_alternatives(source, expression)?;
    let type_name = source.slice(name);
    let declared = alternatives
        .into_iter()
        .enumerate()
        .map(|(index, classifier)| (format!("at {index}"), Some(classifier)))
        .collect();
    session.union_types.insert(type_name.to_owned(), declared);
    session.declared_names.insert(type_name.to_owned());
    trace.record(TraceEvent {
        event: "variant.declared",
        rule: "TOPAL-TYPE-VARIANT-001",
        detail: type_name,
    });
    Some((Value::Unit, expression.span()))
}

fn value_has_classifier(value: &Value, classifier: &str) -> bool {
    if let Value::Refined {
        constraint,
        base_classifier,
        value,
    } = value
    {
        return classifier == constraint
            || (classifier == base_classifier && value_has_classifier(value, base_classifier));
    }
    if let Value::SuspendedGenerator {
        yield_classifier,
        return_classifier,
        ..
    } = value
    {
        return classifier == format!("Generator {yield_classifier} Unit {return_classifier}");
    }
    if let Value::IterateGenerator {
        classifier: yielded,
        ..
    } = value
    {
        return classifier == format!("Generator {yielded} Unit Unit");
    }
    if matches!(value, Value::UnfoldGenerator { .. }) {
        return classifier == "Generator Value Unit Unit";
    }
    if let Value::Optional {
        payload_classifier, ..
    } = value
        && let Some(expected) = optional_payload_classifier(classifier)
    {
        return payload_classifier == expected;
    }
    if let Value::List {
        element_classifier,
        entries,
    } = value
        && let Some(expected) = list_element_classifier(classifier)
    {
        return element_classifier == expected
            && entries
                .iter()
                .all(|entry| value_has_classifier(entry, expected));
    }
    if let Some(success) = result_success_classifier(classifier) {
        return matches!(value, Value::Error { code, .. } if is_arithmetic_error_code(code))
            || value_has_classifier(value, success);
    }
    if let (Value::Tuple(values), Some(classifiers)) = (value, tuple_classifiers(classifier)) {
        return values.len() == classifiers.len()
            && values
                .iter()
                .zip(classifiers)
                .all(|(value, classifier)| value_has_classifier(value, classifier));
    }
    match (value, classifier) {
        (Value::Boolean(_), "Boolean")
        | (Value::Int(_), "Int")
        | (Value::Rational(_), "Rational")
        | (Value::IntRange { .. }, "Range Int")
        | (Value::RationalRange { .. }, "Range Rational")
        | (Value::CharacterGenerator { .. }, "Generator Character Unit Unit")
        | (Value::CharacterReturningGenerator { .. }, "Generator Character Unit Character")
        | (Value::String(_), "String")
        | (Value::Namespace(_), "Scope")
        | (Value::Type(_), "Type")
        | (Value::Effects(_), "Effect")
        | (
            Value::Callable(_) | Value::NamedFunction(_) | Value::AnonymousFunction(_),
            "Function",
        )
        | (Value::Constraint(_), "Constraint")
        | (Value::Continue(_) | Value::Finish(_), "TraversalControl")
        | (Value::Completed, "Completed")
        | (Value::Unit, "Unit") => true,
        (Value::String(value), "Character") => character_count(value) == 1,
        (Value::Int(value), "Nat") => value >= &BigInt::from(0),
        (Value::Enum { type_name, .. } | Value::Modular { type_name, .. }, classifier) => {
            type_name == classifier
        }
        (Value::Union(union), classifier) => union.type_name == classifier,
        _ => false,
    }
}

fn supported_generator_value_classifier(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    matches!(
        classifier,
        "Unit"
            | "Boolean"
            | "Character"
            | "Comparison"
            | "Constraint"
            | "Effect"
            | "Int"
            | "Nat"
            | "Rational"
            | "Scope"
            | "String"
            | "Range Int"
            | "Range Rational"
    ) || enum_types.contains_key(classifier)
        || optional_payload_classifier(classifier)
            .is_some_and(|payload| supported_generator_value_classifier(payload, enum_types))
        || list_element_classifier(classifier)
            .is_some_and(|element| supported_generator_value_classifier(element, enum_types))
        || tuple_classifiers(classifier).is_some_and(|items| {
            items
                .into_iter()
                .all(|item| supported_generator_value_classifier(item, enum_types))
        })
        || result_success_classifier(classifier)
            .is_some_and(|success| supported_generator_value_classifier(success, enum_types))
}

fn is_arithmetic_error_code(code: &str) -> bool {
    matches!(
        code,
        "out-of-range" | "not-representable" | "division-by-zero" | "indeterminate"
    )
}

fn result_success_classifier(classifier: &str) -> Option<&str> {
    let contents = classifier
        .trim()
        .strip_prefix("Result")?
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')?;
    let comma = top_level_comma(contents)?;
    let (success, errors) = (&contents[..comma], &contents[comma + 1..]);
    let errors = errors.split_whitespace().collect::<Vec<_>>().join(" ");
    (errors == "lang arithmetic ArithmeticErrorCode").then(|| success.trim())
}

fn optional_payload_classifier(classifier: &str) -> Option<&str> {
    classifier.trim().strip_prefix("Optional ").map(str::trim)
}

fn list_element_classifier(classifier: &str) -> Option<&str> {
    classifier.trim().strip_prefix("List ").map(str::trim)
}

fn tuple_classifiers(classifier: &str) -> Option<Vec<&str>> {
    let contents = classifier.trim().strip_prefix('(')?.strip_suffix(')')?;
    let mut classifiers = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0_usize;
    for (offset, character) in contents.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                classifiers.push(contents[start..offset].trim());
                start = offset + character.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    classifiers.push(contents[start..].trim());
    (classifiers.len() > 1 && classifiers.iter().all(|item| !item.is_empty()))
        .then_some(classifiers)
}

fn top_level_comma(text: &str) -> Option<usize> {
    let mut depth = 0_usize;
    for (offset, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => return Some(offset),
            _ => {}
        }
    }
    None
}

const fn function_rule(is_static: bool, parameter_count: usize) -> &'static str {
    if !is_static {
        return "TOPAL-FUNCTION-ORDINARY-001";
    }
    match parameter_count {
        0 => "TOPAL-FUNCTION-STATIC-NULLARY-001",
        1 => "TOPAL-FUNCTION-STATIC-UNARY-001",
        _ => "TOPAL-FUNCTION-STATIC-BINARY-001",
    }
}

fn function_accepts(parameters: &[(String, String)], argument: &Value) -> bool {
    match parameters {
        [] => matches!(argument, Value::Unit),
        [(_, classifier)] => value_has_classifier(argument, classifier),
        parameters => {
            let Value::Tuple(arguments) = argument else {
                return false;
            };
            arguments.len() == parameters.len()
                && parameters
                    .iter()
                    .zip(arguments)
                    .all(|((_, classifier), argument)| value_has_classifier(argument, classifier))
        }
    }
}

fn bind_function_arguments(
    scope: &mut Session,
    parameters: &[(String, String)],
    argument: Value,
    trace: &mut impl TraceSink,
    rule: &'static str,
) {
    let arguments = match (parameters, argument) {
        ([], Value::Unit) => return,
        ([_], argument) => vec![argument],
        (_, Value::Tuple(arguments)) => arguments,
        _ => unreachable!("selected overload has already validated its argument"),
    };
    for ((parameter, _), argument) in parameters.iter().zip(arguments) {
        scope.bindings.insert(parameter.clone(), argument);
        scope.declared_names.insert(parameter.clone());
        trace.record(TraceEvent {
            event: "function.argument.bound",
            rule,
            detail: parameter,
        });
    }
}

fn bind_generator_arguments(
    scope: &mut Session,
    parameters: &[(String, String)],
    argument: Value,
    trace: &mut impl TraceSink,
) {
    let arguments = match (parameters, argument) {
        ([_], argument) => vec![argument],
        (_, Value::Tuple(arguments)) => arguments,
        _ => unreachable!("selected generator overload has validated its argument"),
    };
    for ((parameter, _), argument) in parameters.iter().zip(arguments) {
        scope.bindings.insert(parameter.clone(), argument);
        scope.declared_names.insert(parameter.clone());
        trace.record(TraceEvent {
            event: "generator.argument.bound",
            rule: "TOPAL-GENERATOR-OVERLOAD-001",
            detail: parameter,
        });
    }
}

fn no_applicable_generator(
    source: &SourceText,
    name: &str,
    argument_span: Span,
    argument: &Value,
    candidates: &[UserGenerator],
) -> Diagnostic {
    let found = structural_value_classifier(argument);
    let expected = candidates
        .iter()
        .map(|candidate| {
            candidate
                .parameters
                .iter()
                .map(|(_, classifier)| classifier.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .collect::<Vec<_>>()
        .join(" or ");
    diagnostic(
        source,
        "E-NO-APPLICABLE-GENERATOR",
        argument_span,
        format!("no `{name}` generator overload accepts `{found}`"),
    )
    .with_help(format!("available input classifiers: {expected}"))
}

fn no_applicable_overload(
    source: &SourceText,
    name: &str,
    argument_span: Span,
    argument: &Value,
    candidates: &[UserFunction],
    static_context: bool,
) -> Diagnostic {
    let eligible = candidates
        .iter()
        .filter(|function| !static_context || function.is_static)
        .collect::<Vec<_>>();
    if let [function] = eligible.as_slice() {
        match function.parameters.as_slice() {
            [] => {
                return diagnostic(
                    source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    argument_span,
                    format!("nullary function `{name}` requires ()"),
                );
            }
            [(parameter, classifier)] => {
                return diagnostic(
                    source,
                    "E-FUNCTION-ARGUMENT-TYPE",
                    argument_span,
                    format!("argument for `{parameter}` is outside `{classifier}`"),
                );
            }
            parameters => {
                let Value::Tuple(arguments) = argument else {
                    return diagnostic(
                        source,
                        "E-FUNCTION-ARGUMENT-SHAPE",
                        argument_span,
                        format!(
                            "function `{name}` requires a positional product with {} fields",
                            parameters.len()
                        ),
                    );
                };
                if arguments.len() != parameters.len() {
                    return diagnostic(
                        source,
                        "E-FUNCTION-ARGUMENT-ARITY",
                        argument_span,
                        format!(
                            "function `{name}` requires {} arguments but received {}",
                            parameters.len(),
                            arguments.len()
                        ),
                    );
                }
                if let Some(((parameter, classifier), _)) = parameters
                    .iter()
                    .zip(arguments)
                    .find(|((_, classifier), argument)| !value_has_classifier(argument, classifier))
                {
                    return diagnostic(
                        source,
                        "E-FUNCTION-ARGUMENT-TYPE",
                        argument_span,
                        format!("argument for `{parameter}` is outside `{classifier}`"),
                    );
                }
            }
        }
    }
    let signatures = eligible
        .iter()
        .map(|function| {
            let inputs = function
                .parameters
                .iter()
                .map(|(_, classifier)| classifier.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name} ({inputs})")
        })
        .collect::<Vec<_>>()
        .join(", ");
    diagnostic(
        source,
        "E-NO-APPLICABLE-OVERLOAD",
        argument_span,
        format!(
            "no overload of `{name}` accepts {} in this context",
            value_classifier(argument)
        ),
    )
    .with_help(format!("available overloads: {signatures}"))
}

fn function_signature(name: &str, function: &UserFunction) -> String {
    let inputs = function
        .parameters
        .iter()
        .map(|(_, classifier)| classifier.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let staticness = if function.is_static { " static" } else { "" };
    format!("{name}{staticness} ({inputs})")
}

fn validate_parameter_names(
    source: &SourceText,
    parameters: &[FunctionParameter],
) -> Result<(), Diagnostic> {
    for (index, parameter) in parameters.iter().enumerate() {
        let name = source.slice(parameter.name);
        if parameters[..index]
            .iter()
            .any(|earlier| source.slice(earlier.name) == name)
        {
            return Err(diagnostic(
                source,
                "E-DUPLICATE-FUNCTION-PARAMETER",
                parameter.name,
                format!("parameter `{name}` is already declared in this function"),
            ));
        }
    }
    Ok(())
}

fn prove_int_recursion(
    source: &SourceText,
    function_name: &str,
    parameters: &[(String, String)],
    body: &[Statement],
) -> Option<&'static str> {
    let [(parameter, classifier)] = parameters else {
        return None;
    };
    if classifier != "Int" && classifier != "Nat" {
        return None;
    }
    let [Statement::Expression(Expression::DecisionTable { subject, rules, .. })] = body else {
        return None;
    };
    if !matches!(subject.as_ref(), Expression::Identifier(span) if source.slice(*span) == parameter)
    {
        return None;
    }
    let [base, recursive] = rules.as_slice() else {
        return None;
    };
    let (step, proof_rule) = match (&**classifier, &base.matcher) {
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(bound),
                ..
            },
        ) if parse_integer(source.slice(*bound)).is_some_and(|value| value >= BigInt::from(0)) => {
            (CallableKind::Minus, "TOPAL-FUNCTION-RECURSION-NAT-001")
        }
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001",
        ),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (CallableKind::Minus, "TOPAL-FUNCTION-RECURSION-INT-001"),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-INT-INCREASING-001",
        ),
        _ => return None,
    };
    if !matches!(&recursive.matcher, DecisionMatcher::Otherwise(_))
        || contains_self_call(source, function_name, &base.action)
    {
        return None;
    }
    let (found, valid) =
        bounded_self_calls(source, function_name, parameter, step, &recursive.action);
    let nat_step_limit = nat_decrement_step_limit(source, &base.matcher);
    let preserves_nat = classifier != "Nat"
        || step == CallableKind::Plus
        || nat_step_limit.is_some_and(|limit| {
            recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                &recursive.action,
                &limit,
            )
        });
    (found && valid && preserves_nat).then_some(proof_rule)
}

fn nat_decrement_step_limit(source: &SourceText, matcher: &DecisionMatcher) -> Option<BigInt> {
    let DecisionMatcher::Comparison {
        kind: CallableKind::LessEqual,
        operand: Expression::Integer(bound),
        ..
    } = matcher
    else {
        return None;
    };
    parse_integer(source.slice(*bound)).map(|bound| bound + BigInt::from(1))
}

fn recursive_calls_fit_nat_bound(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    expression: &Expression,
    maximum_step: &BigInt,
) -> bool {
    match expression {
        Expression::Application { items, .. } if matches!(items.first(), Some(Expression::Identifier(span)) if source.slice(*span) == function_name) =>
        {
            matches!(items.as_slice(), [_, Expression::Application { items, .. }]
                if matches!(items.as_slice(), [Expression::Identifier(name), Expression::Callable { kind: CallableKind::Minus, .. }, Expression::Integer(amount)]
                    if source.slice(*name) == parameter && parse_integer(source.slice(*amount)).is_some_and(|step| step > BigInt::from(0) && step <= *maximum_step)))
        }
        Expression::Application { items, .. } => items.iter().all(|item| {
            recursive_calls_fit_nat_bound(source, function_name, parameter, item, maximum_step)
        }),
        Expression::Product { fields, .. } => fields.iter().all(|field| {
            recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                &field.value,
                maximum_step,
            )
        }),
        Expression::DecisionTable { subject, rules, .. } => {
            recursive_calls_fit_nat_bound(source, function_name, parameter, subject, maximum_step)
                && rules.iter().all(|rule| {
                    recursive_calls_fit_nat_bound(
                        source,
                        function_name,
                        parameter,
                        &rule.action,
                        maximum_step,
                    )
                })
        }
        _ => true,
    }
}

fn prove_mutual_int_recursion_edge(
    source: &SourceText,
    function_name: &str,
    parameters: &[(String, String)],
    body: &[Statement],
) -> Option<(String, &'static str)> {
    let [(parameter, classifier)] = parameters else {
        return None;
    };
    if classifier != "Int" && classifier != "Nat" {
        return None;
    }
    let [Statement::Expression(Expression::DecisionTable { subject, rules, .. })] = body else {
        return None;
    };
    if !matches!(subject.as_ref(), Expression::Identifier(span) if source.slice(*span) == parameter)
    {
        return None;
    }
    let [base, recursive] = rules.as_slice() else {
        return None;
    };
    let (step, proof_rule) = match (&**classifier, &base.matcher) {
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(bound),
                ..
            },
        ) if parse_integer(source.slice(*bound)).is_some_and(|value| value >= BigInt::from(0)) => (
            CallableKind::Minus,
            "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001",
        ),
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001",
        ),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Minus,
            "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001",
        ),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001",
        ),
        _ => return None,
    };
    if !matches!(&recursive.matcher, DecisionMatcher::Otherwise(_)) {
        return None;
    }
    let (target, valid) =
        mutual_call_target(source, function_name, parameter, step, &recursive.action);
    let target = target?;
    if !valid
        || contains_self_call(source, &target, &base.action)
        || (classifier == "Nat"
            && step == CallableKind::Minus
            && !nat_decrement_step_limit(source, &base.matcher).is_some_and(|limit| {
                recursive_calls_fit_nat_bound(source, &target, parameter, &recursive.action, &limit)
            }))
    {
        return None;
    }
    Some((target, proof_rule))
}

fn mutual_call_target(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> (Option<String>, bool) {
    match expression {
        Expression::Application { items, .. }
            if matches!(items.as_slice(), [Expression::Identifier(_), _]) =>
        {
            let [Expression::Identifier(target), argument] = items.as_slice() else {
                unreachable!("guard established a unary named application");
            };
            let target = source.slice(*target);
            (
                Some(target.to_owned()),
                target != function_name
                    && is_positive_literal_step(source, parameter, step, argument),
            )
        }
        Expression::Application { items, .. } => combine_mutual_call_targets(
            items
                .iter()
                .map(|item| mutual_call_target(source, function_name, parameter, step, item)),
        ),
        Expression::Product { fields, .. } => {
            combine_mutual_call_targets(fields.iter().map(|field| {
                mutual_call_target(source, function_name, parameter, step, &field.value)
            }))
        }
        Expression::DecisionTable { subject, rules, .. } => combine_mutual_call_targets(
            std::iter::once(mutual_call_target(
                source,
                function_name,
                parameter,
                step,
                subject,
            ))
            .chain(rules.iter().map(|rule| {
                mutual_call_target(source, function_name, parameter, step, &rule.action)
            })),
        ),
        _ => (None, true),
    }
}

fn combine_mutual_call_targets(
    checks: impl Iterator<Item = (Option<String>, bool)>,
) -> (Option<String>, bool) {
    checks.fold(
        (None, true),
        |(target, valid), (next_target, next_valid)| match (target, next_target) {
            (Some(target), Some(next)) => {
                let same = target == next;
                (Some(target), valid && next_valid && same)
            }
            (Some(target), None) | (None, Some(target)) => (Some(target), valid && next_valid),
            (None, None) => (None, valid && next_valid),
        },
    )
}

const MUTUAL_INT_RECURSION_RULE: &str = "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001";
const MUTUAL_INCREASING_INT_RECURSION_RULE: &str =
    "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001";
const MUTUAL_NAT_RECURSION_RULE: &str = "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001";
const MUTUAL_INCREASING_NAT_RECURSION_RULE: &str =
    "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001";

fn is_mutual_recursion_rule(rule: &str) -> bool {
    matches!(
        rule,
        MUTUAL_INT_RECURSION_RULE
            | MUTUAL_INCREASING_INT_RECURSION_RULE
            | MUTUAL_NAT_RECURSION_RULE
            | MUTUAL_INCREASING_NAT_RECURSION_RULE
    )
}

fn recursion_rule_for_call(
    call_stack: &[ActiveCall],
    target: &str,
    target_signature: &str,
    function: &UserFunction,
) -> Option<&'static str> {
    let cycle_start = call_stack
        .iter()
        .position(|active| active.signature == target_signature)?;
    let cycle = &call_stack[cycle_start..];
    if function.recursion_target.is_none() {
        return function.termination_rule;
    }
    let cycle_rule = function.termination_rule?;
    if !is_mutual_recursion_rule(cycle_rule)
        || cycle
            .iter()
            .any(|active| active.termination_rule != Some(cycle_rule))
    {
        return None;
    }
    let internal_edges_match = cycle
        .windows(2)
        .all(|pair| pair[0].recursion_target.as_deref() == Some(pair[1].name.as_str()));
    let closes_cycle = cycle
        .last()
        .and_then(|active| active.recursion_target.as_deref())
        == Some(target);
    (internal_edges_match && closes_cycle).then_some(cycle_rule)
}

fn contains_self_call(source: &SourceText, function_name: &str, expression: &Expression) -> bool {
    let (found, _) = bounded_self_calls(source, function_name, "", CallableKind::Minus, expression);
    found
}

fn bounded_self_calls(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> (bool, bool) {
    match expression {
        Expression::Application { items, .. } if matches!(items.first(), Some(Expression::Identifier(span)) if source.slice(*span) == function_name) =>
        {
            let valid = matches!(items.as_slice(), [_, argument] if is_positive_literal_step(source, parameter, step, argument));
            (true, valid)
        }
        Expression::Application { items, .. } => combine_call_checks(
            items
                .iter()
                .map(|item| bounded_self_calls(source, function_name, parameter, step, item)),
        ),
        Expression::Product { fields, .. } => {
            combine_call_checks(fields.iter().map(|field| {
                bounded_self_calls(source, function_name, parameter, step, &field.value)
            }))
        }
        Expression::DecisionTable { subject, rules, .. } => combine_call_checks(
            std::iter::once(bounded_self_calls(
                source,
                function_name,
                parameter,
                step,
                subject,
            ))
            .chain(rules.iter().map(|rule| {
                bounded_self_calls(source, function_name, parameter, step, &rule.action)
            })),
        ),
        _ => (false, true),
    }
}

fn combine_call_checks(checks: impl Iterator<Item = (bool, bool)>) -> (bool, bool) {
    checks.fold((false, true), |(found, valid), (next_found, next_valid)| {
        (found || next_found, valid && next_valid)
    })
}

fn is_positive_literal_step(
    source: &SourceText,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> bool {
    matches!(
        expression,
        Expression::Application { items, .. }
            if matches!(
                items.as_slice(),
                [
                    Expression::Identifier(name),
                    Expression::Callable { kind, .. },
                    Expression::Integer(amount)
                ] if source.slice(*name) == parameter
                    && *kind == step
                    && parse_integer(source.slice(*amount)).is_some_and(|value| value > BigInt::from(0_u8))
            )
    )
}

fn supported_value_classifier(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    matches!(
        classifier,
        "Boolean"
            | "Character"
            | "Completed"
            | "Comparison"
            | "Constraint"
            | "Effect"
            | "Generator Character Unit Unit"
            | "Generator Character Unit Character"
            | "Generator String Unit Unit"
            | "Generator String Unit Character"
            | "Generator String Unit String"
            | "Generator Character Unit String"
            | "Function"
            | "Int"
            | "Nat"
            | "Range Int"
            | "Range Rational"
            | "Rational"
            | "Scope"
            | "String"
            | "Type"
            | "Unit"
    ) || enum_types.contains_key(classifier)
        || generator_classifiers(classifier).is_some_and(|(yielded, resumed, returned)| {
            resumed == "Unit"
                && supported_generator_value_classifier(yielded, enum_types)
                && supported_generator_value_classifier(returned, enum_types)
        })
        || optional_payload_classifier(classifier)
            .is_some_and(|payload| supported_value_classifier(payload, enum_types))
        || list_element_classifier(classifier)
            .is_some_and(|element| supported_value_classifier(element, enum_types))
        || tuple_classifiers(classifier).is_some_and(|items| {
            items
                .into_iter()
                .all(|item| supported_value_classifier(item, enum_types))
        })
        || result_success_classifier(classifier)
            .is_some_and(|success| supported_value_classifier(success, enum_types))
}

fn generator_classifiers(classifier: &str) -> Option<(&str, &str, &str)> {
    let contents = classifier.trim().strip_prefix("Generator")?;
    let (yielded, contents) = take_classifier(contents)?;
    let (resumed, contents) = take_classifier(contents)?;
    let (returned, remainder) = take_classifier(contents)?;
    remainder
        .trim()
        .is_empty()
        .then_some((yielded, resumed, returned))
}

fn take_classifier(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start();
    if text.starts_with('(') {
        let end = parenthesized_end(text)?;
        return Some((&text[..end], &text[end..]));
    }
    let head_end = text.find(char::is_whitespace).unwrap_or(text.len());
    let head = &text[..head_end];
    if head.is_empty() {
        return None;
    }
    if head == "Result" {
        let tail = text[head_end..].trim_start();
        let result_end = parenthesized_end(tail)?;
        let end = text.len() - tail.len() + result_end;
        return Some((&text[..end], &text[end..]));
    }
    let arity = match head {
        "Optional" | "Range" | "List" => 1,
        "Generator" => 3,
        _ => 0,
    };
    let mut remainder = &text[head_end..];
    for _ in 0..arity {
        let (_, next) = take_classifier(remainder)?;
        remainder = next;
    }
    let end = text.len() - remainder.len();
    Some((text[..end].trim_end(), remainder))
}

fn parenthesized_end(text: &str) -> Option<usize> {
    let mut depth = 0_usize;
    for (offset, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(offset + character.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}

fn accepted_source(input: &str, trace: &mut impl TraceSink) -> Result<SourceText, Diagnostic> {
    let source = SourceText::new(input).map_err(|error| {
        let (line, column) = raw_position(input, error.span.start);
        Diagnostic {
            code: error.code,
            line,
            column,
            message: error.message.into(),
            source_line: raw_source_line(input, line),
            marker_width: marker_width(input, error.span),
            help: diagnostic_help(error.code).map(str::to_owned),
        }
    })?;
    trace.record(TraceEvent {
        event: "source.accepted",
        rule: "TOPAL-SYN-SOURCE-001",
        detail: "unicode source normalized",
    });
    Ok(source)
}

fn expected_statement(input: &str) -> Diagnostic {
    Diagnostic {
        code: "E-EXPECTED-EXPRESSION",
        line: 1,
        column: 1,
        message: "expected a statement".into(),
        source_line: raw_source_line(input, 1),
        marker_width: 1,
        help: diagnostic_help("E-EXPECTED-EXPRESSION").map(str::to_owned),
    }
}

fn record_result(trace: &mut impl TraceSink, value: &Value) {
    let classifier = structural_value_classifier(value);
    trace.record(TraceEvent {
        event: "evaluation.result",
        rule: "TOPAL-SYN-GRAMMAR-001",
        detail: &classifier,
    });
}

fn value_classifier(value: &Value) -> &'static str {
    match value {
        Value::Boolean(_) => "Boolean",
        Value::Type(_) | Value::ModularType(_) => "Type",
        Value::Effects(_) => "Effect",
        Value::Int(_) => "Int",
        Value::Rational(_) => "Rational",
        Value::IntRange { .. } | Value::RationalRange { .. } => "Range",
        Value::Optional { .. } => "Optional",
        Value::List { .. } => "List",
        Value::Callable(_) | Value::NamedFunction(_) | Value::AnonymousFunction(_) => "Function",
        Value::Namespace(_) => "Scope",
        Value::Array { .. } => "Array",
        Value::Set { .. } => "Set",
        Value::Bag { .. } => "Bag",
        Value::Map { .. } => "Map",
        Value::CharacterReturningGenerator { .. } => "Generator Character Unit Character",
        Value::IterateGenerator { .. } | Value::UnfoldGenerator { .. } => "Generator",
        Value::SuspendedGenerator {
            yield_classifier,
            return_classifier,
            ..
        } if yield_classifier == "String" && return_classifier == "String" => {
            "Generator String Unit String"
        }
        Value::SuspendedGenerator {
            yield_classifier,
            return_classifier,
            ..
        } if yield_classifier == "String" && return_classifier == "Character" => {
            "Generator String Unit Character"
        }
        Value::SuspendedGenerator {
            yield_classifier, ..
        } if yield_classifier == "String" => "Generator String Unit Unit",
        Value::SuspendedGenerator {
            return_classifier, ..
        } if return_classifier == "String" => "Generator Character Unit String",
        Value::SuspendedGenerator {
            return_classifier, ..
        } if return_classifier == "Character" => "Generator Character Unit Character",
        Value::CharacterGenerator { .. } | Value::SuspendedGenerator { .. } => {
            "Generator Character Unit Unit"
        }
        Value::String(_) => "String",
        Value::Tuple(_) => "Tuple",
        Value::Record(_) => "Record",
        Value::Enum { .. } => "Enum",
        Value::Union(_) => "Union",
        Value::Constraint(_) => "Constraint",
        Value::Refined { .. } => "Refined",
        Value::Modular { .. } => "Modular",
        Value::ErrorDomain(_) => "ErrorDomain",
        Value::Error { .. } => "Error",
        Value::Continue(_) | Value::Finish(_) => "TraversalControl",
        Value::Completed => "Completed",
        Value::Unit => "Unit",
    }
}

fn structural_value_classifier(value: &Value) -> String {
    match value {
        Value::IntRange { .. } => "Range Int".into(),
        Value::RationalRange { .. } => "Range Rational".into(),
        Value::Tuple(values) => format!(
            "({})",
            values
                .iter()
                .map(structural_value_classifier)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Optional {
            payload_classifier, ..
        } => format!("Optional {payload_classifier}"),
        Value::List {
            element_classifier, ..
        } => format!("List {element_classifier}"),
        Value::Array {
            element_classifier,
            entries,
        } => format!("Array {} {element_classifier}", entries.len()),
        Value::Set {
            element_classifier, ..
        } => format!("Set {element_classifier}"),
        Value::Bag {
            element_classifier, ..
        } => format!("Bag {element_classifier}"),
        Value::Map {
            key_classifier,
            value_classifier,
            ..
        } => format!("Map ({key_classifier}, {value_classifier})"),
        Value::SuspendedGenerator {
            yield_classifier,
            return_classifier,
            ..
        } => format!("Generator {yield_classifier} Unit {return_classifier}"),
        Value::IterateGenerator { classifier, .. } => {
            format!("Generator {classifier} Unit Unit")
        }
        Value::UnfoldGenerator { .. } => "Generator Value Unit Unit".into(),
        Value::Enum { type_name, .. } | Value::Modular { type_name, .. } => type_name.clone(),
        Value::Union(union) => union.type_name.clone(),
        Value::Constraint(constraint) => format!("Constraint {}", constraint.base_classifier),
        Value::Refined { constraint, .. } => constraint.clone(),
        Value::Type(name) => name.clone(),
        Value::Effects(_) => "Effect".into(),
        Value::ModularType(kind) => kind.name.clone().unwrap_or_else(|| "Type".into()),
        _ => value_classifier(value).to_owned(),
    }
}

fn generator_classifier_diagnostic(
    source: &SourceText,
    code: &'static str,
    span: Span,
    name: &str,
    action: &str,
    expected: &str,
    value: &Value,
) -> Diagnostic {
    let found = structural_value_classifier(value);
    diagnostic(
        source,
        code,
        span,
        format!("generator `{name}` {action} `{found}`, but its declaration requires `{expected}`"),
    )
    .with_help(format!(
        "produce `{expected}` here or change the generator's declared classifier from `{expected}`"
    ))
}

fn classifier_expression(source: &SourceText, expression: &Expression) -> Option<String> {
    match expression {
        Expression::Identifier(span) => Some(source.slice(*span).to_owned()),
        Expression::Product { fields, .. }
            if fields.len() > 1 && fields.iter().all(|field| field.label.is_none()) =>
        {
            Some(format!(
                "({})",
                fields
                    .iter()
                    .map(|field| classifier_expression(source, &field.value))
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            ))
        }
        _ => None,
    }
}

fn evaluate_boolean_literal(source: &SourceText, span: Span, trace: &mut impl TraceSink) -> Value {
    let lexeme = source.slice(span);
    trace.record(TraceEvent {
        event: "token.boolean",
        rule: "TOPAL-TYPE-BOOLEAN-001",
        detail: lexeme,
    });
    Value::Boolean(lexeme == "true")
}

fn evaluate_integer_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let text = source.slice(span);
    let value = parse_integer(text)
        .ok_or_else(|| diagnostic(source, "E-NUMERIC-LITERAL", span, "invalid integer literal"))?;
    trace.record(TraceEvent {
        event: "token.integer",
        rule: "TOPAL-NUM-LITERAL-001",
        detail: text,
    });
    Ok(Value::Int(value))
}

fn evaluate_rational_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let text = source.slice(span);
    let value = parse_rational(text).ok_or_else(|| {
        diagnostic(
            source,
            "E-NUMERIC-LITERAL",
            span,
            "invalid rational literal",
        )
    })?;
    trace.record(TraceEvent {
        event: "token.rational",
        rule: "TOPAL-NUM-RATIONAL-LITERAL-001",
        detail: text,
    });
    Ok(Value::Rational(value))
}

fn evaluate_string_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let lexeme = source.slice(span);
    let value = parse_string(lexeme).ok_or_else(|| {
        diagnostic(
            source,
            "E-STRING-LITERAL",
            span,
            "invalid string literal delimiter",
        )
    })?;
    trace.record(TraceEvent {
        event: "token.string",
        rule: "TOPAL-SYN-STRING-001",
        detail: lexeme,
    });
    Ok(Value::String(value.to_owned()))
}

fn parse_string(lexeme: &str) -> Option<&str> {
    let opening = lexeme.find('"')?;
    let closing_length = opening + 1;
    (lexeme.len() >= opening + 1 + closing_length)
        .then(|| &lexeme[opening + 1..lexeme.len() - closing_length])
}

fn display_string(value: &str) -> String {
    if !value.contains('"') {
        return format!("\"{value}\"");
    }
    let mut tag = "text".to_owned();
    while value.contains(&format!("\"{tag}")) {
        tag.push('_');
    }
    format!("{tag}\"{value}\"{tag}")
}

#[allow(clippy::too_many_lines)] // Numeric domains keep explicit, non-coercing dispatch arms.
fn apply_binary(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    spans: (Span, Span, Span),
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (span, left_span, right_span) = spans;
    let left = forget_refinement(left, trace, "constraint->base:left");
    let right = forget_refinement(right, trace, "constraint->base:right");
    if matches!(kind, CallableKind::Equal | CallableKind::NotEqual) {
        return apply_equality(source, kind, left, right, span, trace);
    }
    if kind == CallableKind::Compare {
        let Some(ordering) = values_compare(left, right, trace) else {
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "the operand types do not share an applicable TotalOrder",
            ));
        };
        let alternative = match ordering {
            Ordering::Less => "Less",
            Ordering::Equal => "Equal",
            Ordering::Greater => "Greater",
        };
        trace.record(TraceEvent {
            event: "operator.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: "root.<=>(TotalOrder,TotalOrder)",
        });
        trace.record(TraceEvent {
            event: "comparison.result",
            rule: "TOPAL-NUM-THREE-WAY-COMPARE-001",
            detail: alternative,
        });
        return Ok(Value::Enum {
            type_name: "Comparison".to_owned(),
            alternative: alternative.to_owned(),
        });
    }
    if kind == CallableKind::Range {
        return apply_range(source, left, right, span, trace);
    }
    if matches!(
        kind,
        CallableKind::Less
            | CallableKind::Greater
            | CallableKind::LessEqual
            | CallableKind::GreaterEqual
    ) {
        return apply_comparison(source, kind, left, right, span, trace);
    }
    match (left, right) {
        (
            Value::Modular {
                type_name: left_type,
                lower,
                upper,
                value: left,
            },
            Value::Modular {
                type_name: right_type,
                value: right,
                ..
            },
        ) if left_type == right_type => {
            let raw = match kind {
                CallableKind::Plus => left + right,
                CallableKind::Minus => left - right,
                CallableKind::Multiply => left * right,
                _ => {
                    return Err(diagnostic(
                        source,
                        "E-NO-APPLICABLE-OVERLOAD",
                        span,
                        "modular values support settled wrapping +, -, and * operations",
                    ));
                }
            };
            let value = reduce_modular(raw, &lower, &upper);
            trace.record(TraceEvent {
                event: "numeric.modular.wrapped",
                rule: "TOPAL-NUM-MODULAR-ARITHMETIC-001",
                detail: &left_type,
            });
            Ok(Value::Modular {
                type_name: left_type,
                lower,
                upper,
                value,
            })
        }
        (Value::Int(left), Value::Int(right)) => {
            apply_int_binary(source, kind, left, right, right_span, trace)
        }
        (Value::Rational(left), Value::Rational(right)) => {
            apply_rational_binary(source, kind, left, right, span, right_span, trace)
        }
        (Value::Rational(left), Value::Int(right)) if kind == CallableKind::Power => {
            apply_rational_power(source, left, right, left_span, right_span, trace)
        }
        (Value::Int(left), Value::Rational(right)) if kind != CallableKind::Power => {
            trace_conversion(trace, "Int->Rational:left");
            apply_rational_binary(
                source,
                kind,
                BigRational::from_integer(left),
                right,
                span,
                right_span,
                trace,
            )
        }
        (Value::Rational(left), Value::Int(right)) if kind != CallableKind::Power => {
            trace_conversion(trace, "Int->Rational:right");
            apply_rational_binary(
                source,
                kind,
                left,
                BigRational::from_integer(right),
                span,
                right_span,
                trace,
            )
        }
        _ => Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "the implemented subset requires operands from one exact numeric domain",
        )),
    }
}

fn reduce_modular(value: BigInt, lower: &BigInt, upper: &BigInt) -> BigInt {
    let modulus = upper - lower + BigInt::from(1);
    let offset = value - lower;
    let reduced = ((offset % &modulus) + &modulus) % &modulus;
    reduced + lower
}

fn forget_refinement(value: Value, trace: &mut impl TraceSink, detail: &'static str) -> Value {
    if let Value::Refined { value, .. } = value {
        trace_conversion(trace, detail);
        *value
    } else {
        value
    }
}

fn apply_range(
    source: &SourceText,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (range, nonempty) = match (left, right) {
        (Value::Int(lower), Value::Int(upper)) => {
            let nonempty = lower <= upper;
            (Value::IntRange { lower, upper }, nonempty)
        }
        (Value::Rational(lower), Value::Rational(upper)) => {
            let nonempty = lower <= upper;
            (Value::RationalRange { lower, upper }, nonempty)
        }
        (Value::Int(lower), Value::Rational(upper)) => {
            trace_conversion(trace, "Int->Rational:left");
            let lower = BigRational::from_integer(lower);
            let nonempty = lower <= upper;
            (Value::RationalRange { lower, upper }, nonempty)
        }
        (Value::Rational(lower), Value::Int(upper)) => {
            trace_conversion(trace, "Int->Rational:right");
            let upper = BigRational::from_integer(upper);
            let nonempty = lower <= upper;
            (Value::RationalRange { lower, upper }, nonempty)
        }
        _ => {
            return Err(diagnostic(
                source,
                "E-RANGE-ENDPOINTS",
                span,
                "range endpoints require finite Int or Rational values",
            ));
        }
    };
    trace.record(TraceEvent {
        event: "range.constructed",
        rule: "TOPAL-RANGE-INCLUSIVE-001",
        detail: if nonempty { "nonempty" } else { "empty" },
    });
    Ok(range)
}

fn apply_range_membership(
    source: &SourceText,
    callable: &str,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let operands = match (callable, left, right) {
        ("in", Value::Int(value), Value::IntRange { lower, upper })
        | ("contains", Value::IntRange { lower, upper }, Value::Int(value)) => Some((
            BigRational::from_integer(value),
            BigRational::from_integer(lower),
            BigRational::from_integer(upper),
        )),
        ("in", Value::Rational(value), Value::RationalRange { lower, upper })
        | ("contains", Value::RationalRange { lower, upper }, Value::Rational(value)) => {
            Some((value, lower, upper))
        }
        ("in", Value::Int(value), Value::RationalRange { lower, upper })
        | ("contains", Value::RationalRange { lower, upper }, Value::Int(value)) => {
            trace_conversion(trace, "Int->Rational:membership");
            Some((BigRational::from_integer(value), lower, upper))
        }
        _ => None,
    };
    let Some((value, lower, upper)) = operands else {
        return Err(diagnostic(
            source,
            "E-RANGE-MEMBERSHIP-OPERANDS",
            span,
            "range membership requires compatible exact numeric operands",
        ));
    };
    let accepted = lower <= value && value <= upper;
    trace.record(TraceEvent {
        event: "range.membership.tested",
        rule: "TOPAL-RANGE-MEMBERSHIP-001",
        detail: if accepted { "accepted" } else { "rejected" },
    });
    Ok(Value::Boolean(accepted))
}

fn apply_and(
    source: &SourceText,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if let (Value::Boolean(left), Value::Boolean(right)) = (&left, &right) {
        trace.record(TraceEvent {
            event: "operator.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: "root.and(Boolean,Boolean)",
        });
        trace.record(TraceEvent {
            event: "evaluation.logical",
            rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
            detail: "and:eager",
        });
        return Ok(Value::Boolean(*left && *right));
    }
    let result = match (left, right) {
        (
            Value::IntRange {
                lower: left_lower,
                upper: left_upper,
            },
            Value::IntRange {
                lower: right_lower,
                upper: right_upper,
            },
        ) => Value::IntRange {
            lower: left_lower.max(right_lower),
            upper: left_upper.min(right_upper),
        },
        (
            Value::RationalRange {
                lower: left_lower,
                upper: left_upper,
            },
            Value::RationalRange {
                lower: right_lower,
                upper: right_upper,
            },
        ) => Value::RationalRange {
            lower: left_lower.max(right_lower),
            upper: left_upper.min(right_upper),
        },
        _ => {
            return Err(diagnostic(
                source,
                "E-RANGE-INTERSECTION-OPERANDS",
                span,
                "and requires two Booleans or ranges from the same endpoint domain",
            ));
        }
    };
    trace.record(TraceEvent {
        event: "range.intersection.constructed",
        rule: "TOPAL-RANGE-INTERSECTION-001",
        detail: "conjunction",
    });
    Ok(result)
}

fn apply_comparison(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let tuple = matches!((&left, &right), (Value::Tuple(_), Value::Tuple(_)));
    let Some(ordering) = values_compare(left, right, trace) else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "ordering requires operands with shared TotalOrder evidence",
        ));
    };
    let (callable, result) = match kind {
        CallableKind::Less => ("root.<(TotalOrder,TotalOrder)", ordering == Ordering::Less),
        CallableKind::Greater => (
            "root.>(TotalOrder,TotalOrder)",
            ordering == Ordering::Greater,
        ),
        CallableKind::LessEqual => (
            "root.<=(TotalOrder,TotalOrder)",
            ordering != Ordering::Greater,
        ),
        CallableKind::GreaterEqual => {
            ("root.>=(TotalOrder,TotalOrder)", ordering != Ordering::Less)
        }
        _ => unreachable!("comparison dispatch accepts only ordering predicates"),
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: callable,
    });
    trace.record(TraceEvent {
        event: "comparison.result",
        rule: if tuple {
            "TOPAL-TYPE-ORDERING-001"
        } else {
            "TOPAL-NUM-COMPARE-001"
        },
        detail: match ordering {
            Ordering::Less => "Less",
            Ordering::Equal => "Equal",
            Ordering::Greater => "Greater",
        },
    });
    Ok(Value::Boolean(result))
}

fn values_compare(left: Value, right: Value, trace: &mut impl TraceSink) -> Option<Ordering> {
    match (left, right) {
        (Value::Refined { value, .. }, right) => values_compare(*value, right, trace),
        (left, Value::Refined { value, .. }) => values_compare(left, *value, trace),
        (
            Value::Modular {
                type_name: left_type,
                value: left,
                ..
            },
            Value::Modular {
                type_name: right_type,
                value: right,
                ..
            },
        ) if left_type == right_type => Some(left.cmp(&right)),
        (Value::Int(left), Value::Int(right)) => Some(left.cmp(&right)),
        (Value::Rational(left), Value::Rational(right)) => Some(left.cmp(&right)),
        (Value::Int(left), Value::Rational(right)) => {
            trace_conversion(trace, "Int->Rational:left");
            Some(BigRational::from_integer(left).cmp(&right))
        }
        (Value::Rational(left), Value::Int(right)) => {
            trace_conversion(trace, "Int->Rational:right");
            Some(left.cmp(&BigRational::from_integer(right)))
        }
        (Value::Tuple(left), Value::Tuple(right)) if left.len() == right.len() => {
            for (left, right) in left.into_iter().zip(right) {
                let ordering = values_compare(left, right, trace)?;
                if ordering != Ordering::Equal {
                    return Some(ordering);
                }
            }
            Some(Ordering::Equal)
        }
        _ => None,
    }
}

fn apply_equality(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Some(equal) = values_equal(left, right, trace) else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "the operand types do not share an applicable Equality operation",
        ));
    };
    let equal = if kind == CallableKind::NotEqual {
        !equal
    } else {
        equal
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: if kind == CallableKind::NotEqual {
            "root.!=(Equality,Equality)"
        } else {
            "root.=(Equality,Equality)"
        },
    });
    trace.record(TraceEvent {
        event: "evaluation.equal",
        rule: "TOPAL-TYPE-EQUALITY-001",
        detail: if equal { "true" } else { "false" },
    });
    Ok(Value::Boolean(equal))
}

fn is_singleton_list_construction(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(callable), _] if source.slice(*callable) == "one")
}

fn evaluate_singleton_list(
    source: &SourceText,
    session: &Session,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [_, entry] = items else {
        unreachable!("singleton construction shape checked")
    };
    let entry = session.evaluate_expression(source, entry, trace)?;
    Ok(construct_singleton_list(entry, trace))
}

fn is_explicit_empty_list_construction(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(empty), Expression::Identifier(list), _]
        if source.slice(*empty) == "empty" && source.slice(*list) == "List")
}

fn evaluate_empty_list(
    source: &SourceText,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [
        Expression::Identifier(empty),
        Expression::Identifier(list),
        element,
    ] = items
    else {
        unreachable!("empty List construction shape checked")
    };
    debug_assert_eq!(source.slice(*empty), "empty");
    debug_assert_eq!(source.slice(*list), "List");
    let Some(element_classifier) = classifier_expression(source, element) else {
        return Err(diagnostic(
            source,
            "E-LIST-ELEMENT-CLASSIFIER",
            element.span(),
            "empty List requires a supported element classifier",
        ));
    };
    Ok(construct_empty_list(element_classifier, trace))
}

fn construct_singleton_list(entry: Value, trace: &mut impl TraceSink) -> Value {
    let element_classifier = structural_value_classifier(&entry);
    let selection = format!("root.one({element_classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: "list.singleton.constructed",
        rule: "TOPAL-LIST-ONE-001",
        detail: &element_classifier,
    });
    Value::List {
        element_classifier,
        entries: vec![entry],
    }
}

fn construct_empty_list(element_classifier: String, trace: &mut impl TraceSink) -> Value {
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.empty({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: "list.empty.constructed",
        rule: "TOPAL-LIST-EMPTY-001",
        detail: &element_classifier,
    });
    Value::List {
        element_classifier,
        entries: Vec::new(),
    }
}

fn apply_empty_predicate(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (is_empty, classifier, event, rule) = match operand {
        Value::String(text) => (
            text.is_empty(),
            "String".to_owned(),
            "string.empty.tested",
            "TOPAL-STRING-EMPTY-PREDICATE-001",
        ),
        Value::List {
            element_classifier,
            entries,
        } => (
            entries.is_empty(),
            format!("List {element_classifier}"),
            "list.empty.tested",
            "TOPAL-LIST-EMPTY-PREDICATE-001",
        ),
        Value::Array { entries, .. } | Value::Set { entries, .. } => (
            entries.is_empty(),
            "Collection".into(),
            "collection.empty.tested",
            "TOPAL-COLLECTION-EMPTY-PREDICATE-001",
        ),
        Value::Bag { entries, .. } => (
            entries.is_empty(),
            "Collection".into(),
            "collection.empty.tested",
            "TOPAL-COLLECTION-EMPTY-PREDICATE-001",
        ),
        Value::Map { entries, .. } => (
            entries.is_empty(),
            "Collection".into(),
            "collection.empty.tested",
            "TOPAL-COLLECTION-EMPTY-PREDICATE-001",
        ),
        value => {
            let found = structural_value_classifier(&value);
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("empty? requires a String or List operand, found `{found}`"),
            ));
        }
    };
    let selection = format!("root.empty?({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event,
        rule,
        detail: if is_empty { "true" } else { "false" },
    });
    Ok(Value::Boolean(is_empty))
}

fn is_list_uncons(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(name), _] if source.slice(*name) == "uncons")
}

fn evaluate_list_uncons(
    source: &SourceText,
    session: &Session,
    items: &[Expression],
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [_, operand] = items else {
        unreachable!("uncons expression shape checked")
    };
    let operand_span = operand.span();
    let operand = session.evaluate_expression(source, operand, trace)?;
    let value = apply_list_uncons(source, operand, operand_span, trace)?;
    session.checkpoint(trace, Some(&value), Some(span));
    Ok(value)
}

fn apply_list_uncons(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        mut entries,
    } = operand
    else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "uncons requires a List operand",
        ));
    };
    let payload_classifier = format!("({element_classifier}, List {element_classifier})");
    let payload = if entries.is_empty() {
        None
    } else {
        let first = entries.remove(0);
        Some(Box::new(Value::Tuple(vec![
            first,
            Value::List {
                element_classifier: element_classifier.clone(),
                entries,
            },
        ])))
    };
    let selection = format!("root.uncons(List {element_classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: "list.uncons",
        rule: "TOPAL-LIST-UNCONS-001",
        detail: if payload.is_some() { "Some" } else { "None" },
    });
    Ok(Value::Optional {
        payload_classifier,
        payload,
    })
}

fn is_list_projection(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(name), _]
        if matches!(source.slice(*name), "first" | "rest"))
}

fn evaluate_list_projection(
    source: &SourceText,
    session: &Session,
    items: &[Expression],
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [Expression::Identifier(operation), operand] = items else {
        unreachable!("List projection expression shape checked")
    };
    let operation = source.slice(*operation);
    let operand_span = operand.span();
    let operand = session.evaluate_expression(source, operand, trace)?;
    let Value::List {
        element_classifier,
        mut entries,
    } = operand
    else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            operand_span,
            format!("{operation} requires a List operand"),
        ));
    };
    let (payload_classifier, payload) = if operation == "first" {
        (
            element_classifier.clone(),
            (!entries.is_empty()).then(|| Box::new(entries.remove(0))),
        )
    } else {
        let payload_classifier = format!("List {element_classifier}");
        let payload = if entries.is_empty() {
            None
        } else {
            entries.remove(0);
            Some(Box::new(Value::List {
                element_classifier: element_classifier.clone(),
                entries,
            }))
        };
        (payload_classifier, payload)
    };
    let selection = format!("root.{operation}(List {element_classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: if operation == "first" {
            "list.first"
        } else {
            "list.rest"
        },
        rule: if operation == "first" {
            "TOPAL-LIST-FIRST-001"
        } else {
            "TOPAL-LIST-REST-001"
        },
        detail: if payload.is_some() { "Some" } else { "None" },
    });
    let value = Value::Optional {
        payload_classifier,
        payload,
    };
    session.checkpoint(trace, Some(&value), Some(span));
    Ok(value)
}

fn apply_count(
    source: &SourceText,
    operation: &str,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (count, classifier, event, rule) = match operand {
        Value::String(text) => (
            character_count(&text),
            "String".to_owned(),
            if operation == "entry-count" {
                "string.entry-count"
            } else {
                "string.character-count"
            },
            if operation == "entry-count" {
                "TOPAL-STRING-ENTRY-COUNT-001"
            } else {
                "TOPAL-STRING-CHARACTER-COUNT-001"
            },
        ),
        Value::List {
            element_classifier,
            entries,
        } if operation == "entry-count" => (
            entries.len(),
            format!("List {element_classifier}"),
            "list.entry-count",
            "TOPAL-LIST-ENTRY-COUNT-001",
        ),
        Value::Array { entries, .. } | Value::Set { entries, .. } if operation == "entry-count" => {
            (
                entries.len(),
                "Collection".into(),
                "collection.entry-count",
                "TOPAL-COLLECTION-ENTRY-COUNT-001",
            )
        }
        Value::Bag { entries, .. } if operation == "entry-count" => (
            entries.iter().map(|(_, count)| count).sum(),
            "Bag".into(),
            "collection.entry-count",
            "TOPAL-COLLECTION-ENTRY-COUNT-001",
        ),
        Value::Map { entries, .. } if operation == "entry-count" => (
            entries.len(),
            "Map".into(),
            "collection.entry-count",
            "TOPAL-COLLECTION-ENTRY-COUNT-001",
        ),
        value => {
            let found = structural_value_classifier(&value);
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("{operation} has no overload accepting `{found}`"),
            ));
        }
    };
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    let detail = if event.starts_with("string.") {
        format!("characters={count}")
    } else {
        format!("entries={count}")
    };
    trace.record(TraceEvent {
        event,
        rule,
        detail: &detail,
    });
    Ok(Value::Int(BigInt::from(count)))
}

fn apply_list_reverse(value: &mut Value, trace: &mut impl TraceSink) {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        unreachable!("List reverse dispatched only for a List")
    };
    entries.reverse();
    let classifier = format!("List {element_classifier}");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.reverse(List)",
    });
    trace.record(TraceEvent {
        event: "list.reversed",
        rule: "TOPAL-LIST-REVERSE-001",
        detail: &classifier,
    });
}

#[allow(clippy::too_many_lines)] // Keep ordered List operation dispatch together.
fn apply_list_operation(
    source: &SourceText,
    operation: &str,
    left: Value,
    right: Value,
    right_span: Span,
    right_is_closed: bool,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        mut entries,
    } = left
    else {
        unreachable!("List operation is dispatched only for a List left operand")
    };
    if operation.starts_with("contains-") {
        return apply_list_containment(
            source,
            operation,
            &element_classifier,
            &entries,
            right,
            right_span,
            trace,
        );
    }
    if matches!(operation, "remove-first" | "remove-all") {
        return apply_list_value_removal(
            source,
            operation,
            element_classifier,
            entries,
            &right,
            right_span,
            trace,
        );
    }
    if matches!(
        operation,
        "split-at" | "take" | "drop" | "remove" | "remove-indexes"
    ) {
        return apply_list_index_operation(
            source,
            operation,
            element_classifier,
            entries,
            right,
            right_span,
            right_is_closed,
            trace,
        );
    }
    if matches!(operation, "zip-exact" | "zip-shortest") {
        return apply_list_zip(
            source,
            operation,
            &element_classifier,
            entries,
            right,
            right_span,
            trace,
        );
    }
    match operation {
        "prepend" | "append" => {
            if !value_has_classifier(&right, &element_classifier) {
                let found = structural_value_classifier(&right);
                return Err(diagnostic(
                    source,
                    "E-LIST-ENTRY-CLASSIFIER",
                    right_span,
                    format!(
                        "{operation} received `{found}`, but this list requires `{element_classifier}`"
                    ),
                )
                .with_help(format!("use a `{element_classifier}` value here")));
            }
            if operation == "prepend" {
                entries.insert(0, right);
            } else {
                entries.push(right);
            }
        }
        "concat" => {
            let Value::List {
                element_classifier: right_classifier,
                entries: right_entries,
            } = right
            else {
                return Err(diagnostic(
                    source,
                    "E-LIST-CONCAT-OPERAND",
                    right_span,
                    "List concat requires another List",
                ));
            };
            if right_classifier != element_classifier {
                return Err(diagnostic(
                    source,
                    "E-LIST-CONCAT-CLASSIFIER",
                    right_span,
                    format!(
                        "cannot concatenate `List {right_classifier}` with `List {element_classifier}`"
                    ),
                )
                .with_help("use Lists with the same element classifier"));
            }
            entries.extend(right_entries);
        }
        _ => unreachable!("known List operation"),
    }
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: match operation {
            "prepend" => "list.prepended",
            "append" => "list.appended",
            "concat" => "list.concatenated",
            _ => unreachable!("known List operation"),
        },
        rule: match operation {
            "prepend" => "TOPAL-LIST-PREPEND-001",
            "append" => "TOPAL-LIST-APPEND-001",
            "concat" => "TOPAL-LIST-CONCAT-001",
            _ => unreachable!("known List operation"),
        },
        detail: &classifier,
    });
    Ok(Value::List {
        element_classifier,
        entries,
    })
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Bounds, source evidence, and trace context remain explicit.
fn apply_list_index_operation(
    source: &SourceText,
    operation: &str,
    element_classifier: String,
    mut entries: Vec<Value>,
    operand: Value,
    operand_span: Span,
    operand_is_closed: bool,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if operation == "remove-indexes"
        && let Value::IntRange { lower, upper } = operand
    {
        let Ok(lower) = usize::try_from(lower) else {
            return list_boundary_failure(
                source,
                operation,
                operand_span,
                operand_is_closed,
                trace,
            );
        };
        let Ok(upper) = usize::try_from(upper) else {
            return list_boundary_failure(
                source,
                operation,
                operand_span,
                operand_is_closed,
                trace,
            );
        };
        if lower > upper || upper >= entries.len() {
            return list_boundary_failure(
                source,
                operation,
                operand_span,
                operand_is_closed,
                trace,
            );
        }
        entries.drain(lower..=upper);
        trace.record(TraceEvent {
            event: "list.entries.removed",
            rule: "TOPAL-LIST-REMOVE-INDEXES-001",
            detail: &format!("lower={lower};upper={upper}"),
        });
        return Ok(Value::List {
            element_classifier,
            entries,
        });
    }
    let Value::Int(index) = operand else {
        return Err(diagnostic(
            source,
            "E-LIST-INDEX-CLASSIFIER",
            operand_span,
            format!("{operation} requires a Nat operand"),
        ));
    };
    let Ok(index) = usize::try_from(index) else {
        return list_boundary_failure(source, operation, operand_span, operand_is_closed, trace);
    };
    let valid = if operation == "remove" {
        index < entries.len()
    } else {
        index <= entries.len()
    };
    if !valid {
        return list_boundary_failure(source, operation, operand_span, operand_is_closed, trace);
    }
    let classifier = format!("List {element_classifier}");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &format!("root.{operation}({classifier},Nat)"),
    });
    let value = match operation {
        "split-at" => {
            let suffix = entries.split_off(index);
            Value::Tuple(vec![
                Value::List {
                    element_classifier: element_classifier.clone(),
                    entries,
                },
                Value::List {
                    element_classifier,
                    entries: suffix,
                },
            ])
        }
        "take" => {
            entries.truncate(index);
            Value::List {
                element_classifier,
                entries,
            }
        }
        "drop" => Value::List {
            element_classifier,
            entries: entries.split_off(index),
        },
        "remove" => {
            entries.remove(index);
            Value::List {
                element_classifier,
                entries,
            }
        }
        _ => unreachable!("known indexed List operation"),
    };
    trace.record(TraceEvent {
        event: "list.region.selected",
        rule: match operation {
            "split-at" => "TOPAL-LIST-SPLIT-AT-001",
            "take" => "TOPAL-LIST-TAKE-001",
            "drop" => "TOPAL-LIST-DROP-001",
            "remove" => "TOPAL-LIST-REMOVE-INDEX-001",
            _ => unreachable!("known indexed List operation"),
        },
        detail: &format!("index={index}"),
    });
    Ok(value)
}

fn apply_list_insert_at(
    source: &SourceText,
    list: Value,
    boundary: Value,
    boundary_span: Span,
    inserted: Value,
    inserted_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        mut entries,
    } = list
    else {
        unreachable!("insert-at is dispatched only for List")
    };
    let Value::Int(boundary) = boundary else {
        return Err(diagnostic(
            source,
            "E-LIST-BOUNDARY-CLASSIFIER",
            boundary_span,
            "insert-at boundary must be Nat",
        ));
    };
    let Ok(boundary) = usize::try_from(boundary) else {
        return Ok(list_boundary_error(
            source,
            "insert-at",
            boundary_span,
            trace,
        ));
    };
    if boundary > entries.len() {
        return Ok(list_boundary_error(
            source,
            "insert-at",
            boundary_span,
            trace,
        ));
    }
    let inserted_entries = match inserted {
        Value::List {
            element_classifier: classifier,
            entries,
        } => {
            if classifier != element_classifier {
                return Err(diagnostic(
                    source,
                    "E-LIST-INSERT-CLASSIFIER",
                    inserted_span,
                    format!("cannot insert `List {classifier}` into `List {element_classifier}`"),
                ));
            }
            entries
        }
        value if value_has_classifier(&value, &element_classifier) => vec![value],
        value => {
            return Err(diagnostic(
                source,
                "E-LIST-INSERT-CLASSIFIER",
                inserted_span,
                format!(
                    "insert-at requires `{element_classifier}` or `List {element_classifier}`, found `{}`",
                    structural_value_classifier(&value)
                ),
            ));
        }
    };
    let inserted_count = inserted_entries.len();
    entries.splice(boundary..boundary, inserted_entries);
    trace.record(TraceEvent {
        event: "list.inserted",
        rule: "TOPAL-LIST-INSERT-AT-001",
        detail: &format!("boundary={boundary};count={inserted_count}"),
    });
    Ok(Value::List {
        element_classifier,
        entries,
    })
}

fn apply_list_entries_view(list: Value, trace: &mut impl TraceSink) -> Value {
    let Value::List {
        element_classifier,
        entries,
    } = list
    else {
        unreachable!("entries view is dispatched only for List")
    };
    let entry_classifier = format!("IndexedEntry {element_classifier}");
    let entries = entries
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            Value::Record(vec![
                ("index".into(), Value::Int(BigInt::from(index))),
                ("value".into(), value),
            ])
        })
        .collect();
    trace.record(TraceEvent {
        event: "list.entries.viewed",
        rule: "TOPAL-COLLECTION-ENTRIES-001",
        detail: &entry_classifier,
    });
    Value::List {
        element_classifier: entry_classifier,
        entries,
    }
}

fn list_boundary_error(
    source: &SourceText,
    operation: &str,
    span: Span,
    trace: &mut impl TraceSink,
) -> Value {
    let position = source.position(span.start);
    trace.record(TraceEvent {
        event: "list.boundary.rejected",
        rule: "TOPAL-LIST-BOUNDARY-CHECK-001",
        detail: operation,
    });
    Value::Error {
        domain: if operation.starts_with("zip-") {
            format!("root.{operation}(List,List)")
        } else {
            format!("root.{operation}(List,Nat)")
        },
        code: "out-of-range".into(),
        line: position.line,
        column: position.column,
    }
}

fn list_boundary_failure(
    source: &SourceText,
    operation: &str,
    span: Span,
    operand_is_closed: bool,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if operand_is_closed {
        return Err(diagnostic(
            source,
            "E-LIST-BOUNDARY-OUT-OF-RANGE",
            span,
            format!("{operation} operand is outside the List's valid bounds"),
        )
        .with_help(
            "use a boundary no greater than the entry count, or an existing index for remove",
        ));
    }
    Ok(list_boundary_error(source, operation, span, trace))
}

fn apply_list_zip(
    source: &SourceText,
    operation: &str,
    left_classifier: &str,
    left: Vec<Value>,
    right: Value,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier: right_classifier,
        entries: right,
    } = right
    else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-OPERAND",
            right_span,
            format!("{operation} requires another List"),
        ));
    };
    if operation == "zip-exact" && left.len() != right.len() {
        return Ok(list_boundary_error(source, operation, right_span, trace));
    }
    let entries = left
        .into_iter()
        .zip(right)
        .map(|(left, right)| Value::Tuple(vec![left, right]))
        .collect();
    let pair_classifier = format!("({left_classifier}, {right_classifier})");
    trace.record(TraceEvent {
        event: "list.zipped",
        rule: if operation == "zip-exact" {
            "TOPAL-LIST-ZIP-EXACT-001"
        } else {
            "TOPAL-LIST-ZIP-SHORTEST-001"
        },
        detail: operation,
    });
    Ok(Value::List {
        element_classifier: pair_classifier,
        entries,
    })
}

fn apply_list_unzip(
    source: &SourceText,
    pairs: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List { entries, .. } = pairs else {
        return Err(diagnostic(
            source,
            "E-LIST-UNZIP-SOURCE",
            span,
            "unzip requires a List of two-field products",
        ));
    };
    let mut left = Vec::with_capacity(entries.len());
    let mut right = Vec::with_capacity(entries.len());
    for entry in entries {
        let Value::Tuple(mut fields) = entry else {
            return Err(diagnostic(
                source,
                "E-LIST-UNZIP-ENTRY",
                span,
                "unzip requires every List entry to be a two-field product",
            ));
        };
        if fields.len() != 2 {
            return Err(diagnostic(
                source,
                "E-LIST-UNZIP-ENTRY",
                span,
                "unzip requires every List entry to contain exactly two fields",
            ));
        }
        right.push(fields.pop().expect("two fields"));
        left.push(fields.pop().expect("two fields"));
    }
    let left_classifier = left
        .first()
        .map_or_else(|| "Object".into(), structural_value_classifier);
    let right_classifier = right
        .first()
        .map_or_else(|| "Object".into(), structural_value_classifier);
    trace.record(TraceEvent {
        event: "list.unzipped",
        rule: "TOPAL-LIST-UNZIP-001",
        detail: &format!("count={}", left.len()),
    });
    Ok(Value::Tuple(vec![
        Value::List {
            element_classifier: left_classifier,
            entries: left,
        },
        Value::List {
            element_classifier: right_classifier,
            entries: right,
        },
    ]))
}

fn apply_list_zip_longest(
    source: &SourceText,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Tuple(mut left_fields) = left else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-LEFT",
            span,
            "zip-longest left operand must be `(List, default)`",
        ));
    };
    let Value::Tuple(mut right_fields) = right else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-RIGHT",
            span,
            "zip-longest right operand must be `(List, default)`",
        ));
    };
    if left_fields.len() != 2 || right_fields.len() != 2 {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-OPERAND",
            span,
            "zip-longest operands must each contain a List and its default",
        ));
    }
    let left_default = left_fields.pop().expect("two fields");
    let right_default = right_fields.pop().expect("two fields");
    let Value::List {
        element_classifier: left_classifier,
        entries: left_entries,
    } = left_fields.pop().expect("two fields")
    else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-LEFT",
            span,
            "first left field must be a List",
        ));
    };
    let Value::List {
        element_classifier: right_classifier,
        entries: right_entries,
    } = right_fields.pop().expect("two fields")
    else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-RIGHT",
            span,
            "first right field must be a List",
        ));
    };
    if !value_has_classifier(&left_default, &left_classifier)
        || !value_has_classifier(&right_default, &right_classifier)
    {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-DEFAULT",
            span,
            "each zip-longest default must match its List element classifier",
        ));
    }
    let count = left_entries.len().max(right_entries.len());
    let entries = (0..count)
        .map(|index| {
            Value::Tuple(vec![
                left_entries
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| left_default.clone()),
                right_entries
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| right_default.clone()),
            ])
        })
        .collect();
    trace.record(TraceEvent {
        event: "list.zipped",
        rule: "TOPAL-LIST-ZIP-LONGEST-001",
        detail: &format!("count={count}"),
    });
    Ok(Value::List {
        element_classifier: format!("({left_classifier}, {right_classifier})"),
        entries,
    })
}

fn collect_unordered(
    source: &SourceText,
    operation: &str,
    value: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        return Err(diagnostic(
            source,
            "E-UNORDERED-COLLECT-SOURCE",
            span,
            format!("{operation} requires a finite List"),
        ));
    };
    let mut distinct: Vec<(Value, usize)> = Vec::new();
    for entry in entries {
        let mut found = None;
        for (index, (candidate, _)) in distinct.iter().enumerate() {
            if values_equal(candidate.clone(), entry.clone(), trace).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-UNORDERED-COLLECT-EQUALITY",
                    span,
                    format!("`{element_classifier}` must provide equality for {operation}"),
                )
            })? {
                found = Some(index);
                break;
            }
        }
        if let Some(index) = found {
            distinct[index].1 += 1;
        } else {
            distinct.push((entry, 1));
        }
    }
    let count = distinct.len();
    trace.record(TraceEvent {
        event: if operation == "collect-set" {
            "set.collected"
        } else {
            "bag.collected"
        },
        rule: if operation == "collect-set" {
            "TOPAL-SET-COLLECT-001"
        } else {
            "TOPAL-BAG-COLLECT-001"
        },
        detail: &format!("distinct={count}"),
    });
    if operation == "collect-set" {
        Ok(Value::Set {
            element_classifier,
            entries: distinct.into_iter().map(|(value, _)| value).collect(),
        })
    } else {
        Ok(Value::Bag {
            element_classifier,
            entries: distinct,
        })
    }
}

fn collect_map(
    source: &SourceText,
    value: Value,
    policy: &str,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if !matches!(policy, "reject" | "keep-first" | "keep-last") {
        return Err(diagnostic(
            source,
            "E-MAP-COLLISION-POLICY",
            span,
            "collect-map policy must be reject, keep-first, or keep-last",
        ));
    }
    let Value::List { entries, .. } = value else {
        return Err(diagnostic(
            source,
            "E-MAP-COLLECT-SOURCE",
            span,
            "collect-map requires a List of key/value products",
        ));
    };
    let mut mapping: Vec<(Value, Value)> = Vec::new();
    for entry in entries {
        let Value::Tuple(mut pair) = entry else {
            return Err(diagnostic(
                source,
                "E-MAP-COLLECT-ENTRY",
                span,
                "collect-map entries must be two-field products",
            ));
        };
        if pair.len() != 2 {
            return Err(diagnostic(
                source,
                "E-MAP-COLLECT-ENTRY",
                span,
                "collect-map entries must have exactly two fields",
            ));
        }
        let value = pair.pop().expect("two fields");
        let key = pair.pop().expect("two fields");
        let mut collision = None;
        for (index, (candidate, _)) in mapping.iter().enumerate() {
            if values_equal(candidate.clone(), key.clone(), trace).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-MAP-KEY-EQUALITY",
                    span,
                    "map keys must provide equality",
                )
            })? {
                collision = Some(index);
                break;
            }
        }
        match (collision, policy) {
            (Some(_), "reject") => {
                return Err(diagnostic(
                    source,
                    "E-MAP-KEY-COLLISION",
                    span,
                    "collect-map encountered a duplicate key under reject policy",
                ));
            }
            (Some(_), "keep-first") => {}
            (Some(index), "keep-last") => mapping[index].1 = value,
            (None, _) => mapping.push((key, value)),
            _ => unreachable!("validated collision policy"),
        }
    }
    let key_classifier = mapping.first().map_or_else(
        || "Object".into(),
        |(key, _)| structural_value_classifier(key),
    );
    let value_classifier = mapping.first().map_or_else(
        || "Object".into(),
        |(_, value)| structural_value_classifier(value),
    );
    trace.record(TraceEvent {
        event: "map.collected",
        rule: "TOPAL-MAP-COLLECT-001",
        detail: policy,
    });
    Ok(Value::Map {
        key_classifier,
        value_classifier,
        entries: mapping,
    })
}

fn apply_list_value_removal(
    source: &SourceText,
    operation: &str,
    element_classifier: String,
    entries: Vec<Value>,
    target: &Value,
    target_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if !value_has_classifier(target, &element_classifier) {
        let found = structural_value_classifier(target);
        return Err(diagnostic(
            source,
            "E-LIST-REMOVAL-CLASSIFIER",
            target_span,
            format!("{operation} requires `{element_classifier}`, found `{found}`"),
        ));
    }
    let mut removed = false;
    let mut retained = Vec::with_capacity(entries.len());
    for entry in entries {
        let equal = values_equal(entry.clone(), target.clone(), trace).ok_or_else(|| {
            diagnostic(
                source,
                "E-LIST-REMOVAL-EQUALITY",
                target_span,
                format!("`{element_classifier}` does not provide equality required by {operation}"),
            )
        })?;
        if equal && (operation == "remove-all" || !removed) {
            removed = true;
        } else {
            retained.push(entry);
        }
    }
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: if operation == "remove-first" {
            "list.first.removed"
        } else {
            "list.all.removed"
        },
        rule: if operation == "remove-first" {
            "TOPAL-LIST-REMOVE-FIRST-001"
        } else {
            "TOPAL-LIST-REMOVE-ALL-001"
        },
        detail: if removed {
            "removed=true"
        } else {
            "removed=false"
        },
    });
    Ok(Value::List {
        element_classifier,
        entries: retained,
    })
}

fn apply_list_containment(
    source: &SourceText,
    operation: &str,
    element_classifier: &str,
    entries: &[Value],
    right: Value,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let contained = if operation == "contains-entry" {
        if !value_has_classifier(&right, element_classifier) {
            let found = structural_value_classifier(&right);
            return Err(diagnostic(
                source,
                "E-LIST-CONTAINMENT-CLASSIFIER",
                right_span,
                format!("contains-entry requires `{element_classifier}`, found `{found}`"),
            ));
        }
        entries.iter().try_fold(false, |found, entry| {
            values_equal(entry.clone(), right.clone(), trace).map(|equal| found || equal)
        })
    } else {
        let Value::List {
            element_classifier: right_classifier,
            entries: pattern,
        } = right
        else {
            return Err(diagnostic(
                source,
                "E-LIST-CONTAINMENT-OPERAND",
                right_span,
                format!("{operation} requires another List"),
            ));
        };
        if right_classifier != element_classifier {
            return Err(diagnostic(
                source,
                "E-LIST-CONTAINMENT-CLASSIFIER",
                right_span,
                format!("{operation} requires `List {element_classifier}`, found `List {right_classifier}`"),
            ));
        }
        if operation == "contains-sequence" {
            contains_consecutive(entries, &pattern, trace)
        } else {
            contains_ordered_subsequence(entries, &pattern, trace)
        }
    }
    .ok_or_else(|| {
        diagnostic(
            source,
            "E-LIST-CONTAINMENT-EQUALITY",
            right_span,
            format!("`{element_classifier}` does not provide equality required by {operation}"),
        )
    })?;
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    let rule = match operation {
        "contains-entry" => "TOPAL-LIST-CONTAINS-ENTRY-001",
        "contains-sequence" => "TOPAL-LIST-CONTAINS-SEQUENCE-001",
        "contains-subsequence" => "TOPAL-LIST-CONTAINS-SUBSEQUENCE-001",
        _ => unreachable!("known List containment operation"),
    };
    trace.record(TraceEvent {
        event: "list.containment.tested",
        rule,
        detail: if contained { "true" } else { "false" },
    });
    Ok(Value::Boolean(contained))
}

fn contains_consecutive(
    entries: &[Value],
    pattern: &[Value],
    trace: &mut impl TraceSink,
) -> Option<bool> {
    if pattern.is_empty() {
        return Some(true);
    }
    entries
        .windows(pattern.len())
        .try_fold(false, |found, window| {
            window
                .iter()
                .zip(pattern)
                .try_fold(true, |equal, (left, right)| {
                    values_equal(left.clone(), right.clone(), trace).map(|item| equal && item)
                })
                .map(|equal| found || equal)
        })
}

fn contains_ordered_subsequence(
    entries: &[Value],
    pattern: &[Value],
    trace: &mut impl TraceSink,
) -> Option<bool> {
    let mut matched = 0;
    for entry in entries {
        if let Some(expected) = pattern.get(matched)
            && values_equal(entry.clone(), expected.clone(), trace)?
        {
            matched += 1;
        }
    }
    Some(matched == pattern.len())
}

#[allow(clippy::too_many_lines)] // Every recursively derived equality remains explicit.
fn values_equal(left: Value, right: Value, trace: &mut impl TraceSink) -> Option<bool> {
    match (left, right) {
        (
            Value::Refined {
                constraint: left_constraint,
                value: left,
                ..
            },
            Value::Refined {
                constraint: right_constraint,
                value: right,
                ..
            },
        ) if left_constraint == right_constraint => values_equal(*left, *right, trace),
        (Value::Refined { value, .. }, right) => values_equal(*value, right, trace),
        (left, Value::Refined { value, .. }) => values_equal(left, *value, trace),
        (Value::Type(left), Value::Type(right)) | (Value::String(left), Value::String(right)) => {
            Some(left == right)
        }
        (Value::Effects(left), Value::Effects(right)) => Some(left == right),
        (Value::Boolean(left), Value::Boolean(right)) => Some(left == right),
        (Value::Int(left), Value::Int(right)) => Some(left == right),
        (Value::Rational(left), Value::Rational(right)) => Some(left == right),
        (Value::Int(left), Value::Rational(right)) => {
            trace_conversion(trace, "Int->Rational:left");
            Some(BigRational::from_integer(left) == right)
        }
        (Value::Rational(left), Value::Int(right)) => {
            trace_conversion(trace, "Int->Rational:right");
            Some(left == BigRational::from_integer(right))
        }
        (
            Value::Modular {
                type_name: left_type,
                value: left,
                ..
            },
            Value::Modular {
                type_name: right_type,
                value: right,
                ..
            },
        ) if left_type == right_type => Some(left == right),
        (
            Value::List {
                element_classifier: left_classifier,
                entries: left,
            },
            Value::List {
                element_classifier: right_classifier,
                entries: right,
            },
        ) if left_classifier == right_classifier && left.len() == right.len() => {
            trace.record(TraceEvent {
                event: "equality.list",
                rule: "TOPAL-TYPE-LIST-EQUALITY-001",
                detail: &left_classifier,
            });
            left.into_iter()
                .zip(right)
                .try_fold(true, |equal, (left, right)| {
                    values_equal(left, right, trace).map(|entry_equal| equal && entry_equal)
                })
        }
        (
            Value::Enum {
                type_name: left_type,
                alternative: left,
            },
            Value::Enum {
                type_name: right_type,
                alternative: right,
            },
        ) if left_type == right_type => Some(left == right),
        (Value::Union(left), Value::Union(right))
            if left.type_name == right.type_name && left.alternative == right.alternative =>
        {
            match (left.payload, right.payload) {
                (None, None) => Some(true),
                (Some(left), Some(right)) => values_equal(*left, *right, trace),
                _ => Some(false),
            }
        }
        (
            Value::Optional {
                payload_classifier: left_classifier,
                payload: left,
            },
            Value::Optional {
                payload_classifier: right_classifier,
                payload: right,
            },
        ) if left_classifier == right_classifier => {
            trace.record(TraceEvent {
                event: "equality.optional",
                rule: "TOPAL-TYPE-OPTIONAL-EQUALITY-001",
                detail: &left_classifier,
            });
            match (left, right) {
                (None, None) => Some(true),
                (Some(left), Some(right)) => values_equal(*left, *right, trace),
                _ => Some(false),
            }
        }
        (Value::Unit, Value::Unit) => Some(true),
        (Value::Tuple(left), Value::Tuple(right)) if left.len() == right.len() => left
            .into_iter()
            .zip(right)
            .try_fold(true, |equal, (left, right)| {
                values_equal(left, right, trace).map(|field_equal| equal && field_equal)
            }),
        (Value::Record(left), Value::Record(right)) if left.len() == right.len() => {
            left.into_iter().try_fold(true, |equal, (label, left)| {
                let right = right
                    .iter()
                    .find(|(right_label, _)| right_label == &label)
                    .map(|(_, value)| value.clone())?;
                values_equal(left, right, trace).map(|field_equal| equal && field_equal)
            })
        }
        _ => None,
    }
}

fn trace_conversion(trace: &mut impl TraceSink, detail: &'static str) {
    trace.record(TraceEvent {
        event: "conversion.applied",
        rule: "TOPAL-TYPE-CONVERT-001",
        detail,
    });
}

fn apply_int_binary(
    source: &SourceText,
    kind: CallableKind,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    match kind {
        CallableKind::Equal
        | CallableKind::NotEqual
        | CallableKind::Compare
        | CallableKind::Less
        | CallableKind::Greater
        | CallableKind::LessEqual
        | CallableKind::GreaterEqual => {
            unreachable!("comparison is dispatched before numeric operations")
        }
        CallableKind::Range => unreachable!("range is dispatched before numeric operations"),
        CallableKind::Plus => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.+(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.add",
                rule: "TOPAL-NUM-ADD-001",
                detail: "Int",
            });
            Ok(Value::Int(left + right))
        }
        CallableKind::Minus => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.subtract",
                rule: "TOPAL-NUM-SUB-001",
                detail: "Int",
            });
            Ok(Value::Int(left - right))
        }
        CallableKind::Multiply => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.*(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.multiply",
                rule: "TOPAL-NUM-MUL-001",
                detail: "Int",
            });
            Ok(Value::Int(left * right))
        }
        CallableKind::Divide => apply_divide(source, left, right, right_span, trace),
        CallableKind::QuotientModulo => {
            apply_quotient_modulo(source, left, right, right_span, trace)
        }
        CallableKind::Modulo => apply_modulo(source, left, &right, right_span, trace),
        CallableKind::Power => apply_power(source, left, right, right_span, trace),
    }
}

fn apply_modulo(
    source: &SourceText,
    left: BigInt,
    right: &BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right == &BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-DIVZERO-001",
            detail: "divisor.nonzero",
        });
        if parse_integer(source.slice(right_span)).is_none() {
            let position = source.position(right_span.start);
            trace.record(TraceEvent {
                event: "result.error.constructed",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: "root.%(Int,Int);division-by-zero",
            });
            return Ok(Value::Error {
                domain: "root.%(Int,Int)".to_owned(),
                code: "division-by-zero".to_owned(),
                line: position.line,
                column: position.column,
            });
        }
        return Err(diagnostic(
            source,
            "E-DIVISION-BY-ZERO",
            right_span,
            "statically evident modulo by zero",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-DIVZERO-001",
        detail: "divisor.nonzero",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.%(Int,Int)",
    });
    let remainder = euclidean_remainder(left, right);
    trace.record(TraceEvent {
        event: "evaluation.modulo",
        rule: "TOPAL-NUM-INT-MODULO-001",
        detail: "Euclidean",
    });
    Ok(Value::Int(remainder))
}

fn apply_quotient_modulo(
    source: &SourceText,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right == BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-DIVZERO-001",
            detail: "divisor.nonzero",
        });
        if parse_integer(source.slice(right_span)).is_none() {
            let position = source.position(right_span.start);
            trace.record(TraceEvent {
                event: "result.error.constructed",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: "root./%(Int,Int);division-by-zero",
            });
            return Ok(Value::Error {
                domain: "root./%(Int,Int)".to_owned(),
                code: "division-by-zero".to_owned(),
                line: position.line,
                column: position.column,
            });
        }
        return Err(diagnostic(
            source,
            "E-DIVISION-BY-ZERO",
            right_span,
            "statically evident quotient/modulo by zero",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-DIVZERO-001",
        detail: "divisor.nonzero",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root./%(Int,Int)",
    });
    let remainder = euclidean_remainder(left.clone(), &right);
    let quotient = (left - &remainder) / right;
    trace.record(TraceEvent {
        event: "evaluation.quotient-modulo",
        rule: "TOPAL-NUM-INT-QUOTIENT-MODULO-001",
        detail: "Euclidean",
    });
    Ok(Value::Tuple(vec![
        Value::Int(quotient),
        Value::Int(remainder),
    ]))
}

fn euclidean_remainder(left: BigInt, right: &BigInt) -> BigInt {
    let mut remainder = left % right;
    if remainder < BigInt::from(0) {
        remainder += if right < &BigInt::from(0) {
            -right
        } else {
            right.clone()
        };
    }
    remainder
}

fn apply_rational_binary(
    source: &SourceText,
    kind: CallableKind,
    left: BigRational,
    right: BigRational,
    span: Span,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if matches!(kind, CallableKind::Modulo | CallableKind::QuotientModulo) {
        return Err(discrete_operand_diagnostic(source, span));
    }
    let (callable, event, rule, result) = match kind {
        CallableKind::Equal
        | CallableKind::NotEqual
        | CallableKind::Compare
        | CallableKind::Less
        | CallableKind::Greater
        | CallableKind::LessEqual
        | CallableKind::GreaterEqual => {
            unreachable!("comparison is dispatched before numeric operations")
        }
        CallableKind::Range => unreachable!("range is dispatched before numeric operations"),
        CallableKind::Plus => (
            "root.+(Rational,Rational)",
            "evaluation.add",
            "TOPAL-NUM-RAT-ADD-001",
            left + right,
        ),
        CallableKind::Minus => (
            "root.-(Rational,Rational)",
            "evaluation.subtract",
            "TOPAL-NUM-RAT-SUB-001",
            left - right,
        ),
        CallableKind::Multiply => (
            "root.*(Rational,Rational)",
            "evaluation.multiply",
            "TOPAL-NUM-RAT-MUL-001",
            left * right,
        ),
        CallableKind::Modulo | CallableKind::QuotientModulo => {
            unreachable!("discrete operations are rejected before Rational dispatch")
        }
        CallableKind::Divide => {
            if right.numer() == &BigInt::from(0) {
                trace.record(TraceEvent {
                    event: "obligation.refuted",
                    rule: "TOPAL-NUM-DIVZERO-001",
                    detail: "divisor.nonzero",
                });
                if parse_rational(source.slice(right_span)).is_none()
                    && parse_integer(source.slice(right_span)).is_none()
                {
                    let position = source.position(right_span.start);
                    trace.record(TraceEvent {
                        event: "result.error.constructed",
                        rule: "TOPAL-TYPE-RESULT-001",
                        detail: "root./(Rational,Rational);division-by-zero",
                    });
                    return Ok(Value::Error {
                        domain: "root./(Rational,Rational)".to_owned(),
                        code: "division-by-zero".to_owned(),
                        line: position.line,
                        column: position.column,
                    });
                }
                return Err(diagnostic(
                    source,
                    "E-DIVISION-BY-ZERO",
                    right_span,
                    "statically evident division by zero",
                ));
            }
            trace.record(TraceEvent {
                event: "obligation.proved",
                rule: "TOPAL-NUM-DIVZERO-001",
                detail: "divisor.nonzero",
            });
            (
                "root./(Rational,Rational)",
                "evaluation.divide",
                "TOPAL-NUM-RAT-DIV-001",
                left / right,
            )
        }
        CallableKind::Power => {
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "Rational exponentiation is not in the implemented subset",
            ));
        }
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: callable,
    });
    trace.record(TraceEvent {
        event,
        rule,
        detail: "Rational",
    });
    Ok(Value::Rational(result))
}

fn discrete_operand_diagnostic(source: &SourceText, span: Span) -> Diagnostic {
    diagnostic(
        source,
        "E-NO-APPLICABLE-OVERLOAD",
        span,
        "Euclidean modulo requires discrete Int operands",
    )
}

fn apply_divide(
    source: &SourceText,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right == BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-DIVZERO-001",
            detail: "divisor.nonzero",
        });
        return Err(diagnostic(
            source,
            "E-DIVISION-BY-ZERO",
            right_span,
            "statically evident division by zero",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-DIVZERO-001",
        detail: "divisor.nonzero",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root./(Int,Int)",
    });
    trace.record(TraceEvent {
        event: "evaluation.divide",
        rule: "TOPAL-NUM-DIV-001",
        detail: "Rational",
    });
    Ok(Value::Rational(BigRational::new(left, right)))
}

fn apply_power(
    source: &SourceText,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right < BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-POW-001",
            detail: "exponent.finite-nat",
        });
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            right_span,
            "Int exponentiation requires a finite Nat exponent",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-POW-001",
        detail: "exponent.finite-nat",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.^(Int,Nat)",
    });
    trace.record(TraceEvent {
        event: "evaluation.power",
        rule: "TOPAL-NUM-POW-001",
        detail: "Int",
    });
    Ok(Value::Int(pow_int(left, right)))
}

fn apply_rational_power(
    source: &SourceText,
    left: BigRational,
    right: BigInt,
    left_span: Span,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right < BigInt::from(0) {
        if left.numer() == &BigInt::from(0) {
            trace.record(TraceEvent {
                event: "obligation.refuted",
                rule: "TOPAL-NUM-RAT-NEG-POW-001",
                detail: "base.nonzero",
            });
            if parse_rational(source.slice(left_span)).is_none()
                && parse_integer(source.slice(left_span)).is_none()
            {
                let position = source.position(left_span.start);
                trace.record(TraceEvent {
                    event: "result.error.constructed",
                    rule: "TOPAL-TYPE-RESULT-001",
                    detail: "root.^(Rational,Int);division-by-zero",
                });
                return Ok(Value::Error {
                    domain: "root.^(Rational,Int)".to_owned(),
                    code: "division-by-zero".to_owned(),
                    line: position.line,
                    column: position.column,
                });
            }
            return Err(diagnostic(
                source,
                "E-DIVISION-BY-ZERO",
                right_span,
                "a zero Rational base cannot be raised to a negative exponent",
            ));
        }
        trace.record(TraceEvent {
            event: "obligation.proved",
            rule: "TOPAL-NUM-RAT-NEG-POW-001",
            detail: "base.nonzero",
        });
        trace.record(TraceEvent {
            event: "operator.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: "root.^(Rational,Int)",
        });
        trace.record(TraceEvent {
            event: "evaluation.power",
            rule: "TOPAL-NUM-RAT-NEG-POW-001",
            detail: "Rational",
        });
        let power = pow_rational(left, -right);
        return Ok(Value::Rational(BigRational::new(
            power.denom().clone(),
            power.numer().clone(),
        )));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-RAT-POW-001",
        detail: "exponent.finite-nat",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.^(Rational,Nat)",
    });
    trace.record(TraceEvent {
        event: "evaluation.power",
        rule: "TOPAL-NUM-RAT-POW-001",
        detail: "Rational",
    });
    Ok(Value::Rational(pow_rational(left, right)))
}

fn pow_int(mut base: BigInt, mut exponent: BigInt) -> BigInt {
    let zero = BigInt::from(0);
    let one = BigInt::from(1);
    let two = BigInt::from(2);
    let mut result = one.clone();
    while exponent > zero {
        if &exponent % &two == one {
            result *= &base;
        }
        exponent /= &two;
        if exponent > zero {
            base = &base * &base;
        }
    }
    result
}

fn pow_rational(mut base: BigRational, mut exponent: BigInt) -> BigRational {
    let zero = BigInt::from(0);
    let one = BigInt::from(1);
    let two = BigInt::from(2);
    let mut result = BigRational::from_integer(one.clone());
    while exponent > zero {
        if &exponent % &two == one {
            result *= &base;
        }
        exponent /= &two;
        if exponent > zero {
            base = &base * &base;
        }
    }
    result
}

fn apply_negate(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    match operand {
        Value::Int(operand) => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.negate",
                rule: "TOPAL-NUM-NEG-001",
                detail: "Int",
            });
            Ok(Value::Int(-operand))
        }
        Value::Rational(operand) => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Rational)",
            });
            trace.record(TraceEvent {
                event: "evaluation.negate",
                rule: "TOPAL-NUM-RAT-NEG-001",
                detail: "Rational",
            });
            Ok(Value::Rational(-operand))
        }
        Value::Modular {
            type_name,
            lower,
            upper,
            value,
        } => {
            let value = reduce_modular(-value, &lower, &upper);
            trace.record(TraceEvent {
                event: "numeric.modular.wrapped",
                rule: "TOPAL-NUM-MODULAR-ARITHMETIC-001",
                detail: &type_name,
            });
            Ok(Value::Modular {
                type_name,
                lower,
                upper,
                value,
            })
        }
        Value::Boolean(_)
        | Value::Type(_)
        | Value::Effects(_)
        | Value::IntRange { .. }
        | Value::RationalRange { .. }
        | Value::Optional { .. }
        | Value::List { .. }
        | Value::Callable(_)
        | Value::NamedFunction(_)
        | Value::Namespace(_)
        | Value::AnonymousFunction(_)
        | Value::Array { .. }
        | Value::Set { .. }
        | Value::Bag { .. }
        | Value::Map { .. }
        | Value::CharacterGenerator { .. }
        | Value::CharacterReturningGenerator { .. }
        | Value::IterateGenerator { .. }
        | Value::UnfoldGenerator { .. }
        | Value::SuspendedGenerator { .. }
        | Value::String(_)
        | Value::Tuple(_)
        | Value::Record(_)
        | Value::Enum { .. }
        | Value::Union(_)
        | Value::Constraint(_)
        | Value::Refined { .. }
        | Value::ModularType(_)
        | Value::ErrorDomain(_)
        | Value::Error { .. }
        | Value::Continue(_)
        | Value::Finish(_)
        | Value::Completed
        | Value::Unit => Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "prefix - requires an exact numeric operand",
        )),
    }
}

const fn callable_name(kind: CallableKind) -> &'static str {
    match kind {
        CallableKind::Equal => "=",
        CallableKind::NotEqual => "/=",
        CallableKind::Less => "<",
        CallableKind::Greater => ">",
        CallableKind::LessEqual => "<=",
        CallableKind::Compare => "<=>",
        CallableKind::Range => "..",
        CallableKind::GreaterEqual => ">=",
        CallableKind::Plus => "+",
        CallableKind::Minus => "-",
        CallableKind::Multiply => "*",
        CallableKind::Divide => "/",
        CallableKind::QuotientModulo => "/%",
        CallableKind::Modulo => "%",
        CallableKind::Power => "^",
    }
}

fn diagnostic(
    source: &SourceText,
    code: &'static str,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    let position = source.position(span.start);
    Diagnostic {
        code,
        line: position.line,
        column: position.column,
        message: message.into(),
        source_line: source
            .as_str()
            .lines()
            .nth(position.line - 1)
            .map(str::to_owned),
        marker_width: marker_width(source.as_str(), span),
        help: diagnostic_help(code).map(str::to_owned),
    }
}

fn closest_name<'a>(name: &str, candidates: impl Iterator<Item = &'a String>) -> Option<&'a str> {
    let maximum = 2.max(name.chars().count() / 3);
    candidates
        .map(|candidate| (edit_distance(name, candidate), candidate.as_str()))
        .filter(|(distance, _)| *distance <= maximum)
        .min()
        .map(|(_, candidate)| candidate)
}

const ROOT_OPERATIONS: [&str; 21] = [
    "absolute",
    "byte-count",
    "case-fold",
    "canonically-equals",
    "characters",
    "character-count",
    "concat",
    "collect",
    "empty",
    "entry-count",
    "first",
    "lower",
    "normalize",
    "upper",
    "uncons",
    "not",
    "negate",
    "one",
    "rest",
    "reverse",
    "zero",
];

fn closest_root_operation(name: &str) -> Option<&'static str> {
    if name == "concatenate" {
        return Some("concat");
    }
    let maximum = 2.max(name.chars().count() / 3);
    ROOT_OPERATIONS
        .into_iter()
        .map(|candidate| (edit_distance(name, candidate), candidate))
        .filter(|(distance, _)| *distance <= maximum)
        .min()
        .map(|(_, candidate)| candidate)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (left_index, left_character) in left.chars().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_character) in right.iter().enumerate() {
            current.push(
                (current[right_index] + 1)
                    .min(previous[right_index + 1] + 1)
                    .min(previous[right_index] + usize::from(left_character != *right_character)),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

fn diagnostic_help(code: &str) -> Option<&'static str> {
    match code {
        "E-UNKNOWN-TOKEN" => Some("remove this character or use a symbol declared by design-0"),
        "E-UNBOUND-NAME" => Some("declare this name earlier in the same source session"),
        "E-EXPECTED-RPAREN" => Some("add a closing `)` for this parenthesized expression"),
        "E-UNTERMINATED-STRING" => Some("add the literal's matching closing quote and tag"),
        "E-DIVISION-BY-ZERO" => Some("use a divisor that is provably nonzero"),
        "E-NO-APPLICABLE-OVERLOAD" => {
            Some("use operands supported by one overload or apply an explicit conversion")
        }
        "E-RESERVED-BOOLEAN-LITERAL" => {
            Some("choose an identifier other than the reserved literals `true` and `false`")
        }
        "E-MIXED-PRODUCT-FIELDS" => {
            Some("nest a tuple in a labeled field, or place a record inside a tuple")
        }
        "E-RESULT-PROJECTION-INFALLIBLE" => {
            Some("change the function result to `Result (T, Codes)`, or match the Error explicitly")
        }
        "E-RESULT-PROJECTION-OUTSIDE-FUNCTION" => Some("match the Result explicitly at top level"),
        "E-INCOMPLETE-ERROR-CODE-DECISION" => {
            Some("add each missing qualified code pattern, or add an `Error problem` fallback")
        }
        "E-DUPLICATE-ERROR-CODE-PATTERN" => {
            Some("remove the later duplicate pattern or replace it with a missing alternative")
        }
        "E-UNREACHABLE-ERROR-CODE-PATTERN" => {
            Some("move qualified code patterns before the generic `Error problem` fallback")
        }
        "E-UNREACHABLE-DECISION-RULE" => Some("move `otherwise` after every specific matcher"),
        "E-CHARACTER-CLASSIFIER" => {
            Some("use a String containing exactly one Unicode grapheme cluster")
        }
        "E-STRING-CONSTRUCTOR-CHARACTER" => {
            Some("classify a one-character String as Character before construction")
        }
        "E-RATIONAL-NOT-EXACT-INT" => {
            Some("use an exactly divisible expression or keep the result classified as Rational")
        }
        "E-NAT-OUT-OF-RANGE" => Some("use a provably nonnegative Int or handle dynamic validation"),
        "E-INDETERMINATE-RATIONAL" => {
            Some("use a nonzero denominator or handle dynamic Rational construction")
        }
        _ => None,
    }
}

fn raw_source_line(source: &str, line: usize) -> Option<String> {
    source.lines().nth(line - 1).map(str::to_owned)
}

fn marker_width(source: &str, span: Span) -> usize {
    source
        .get(span.start..span.end)
        .unwrap_or("")
        .split(['\r', '\n'])
        .next()
        .unwrap_or("")
        .chars()
        .count()
        .max(1)
}

fn raw_position(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit_once('\n')
        .map_or(before, |(_, tail)| tail)
        .chars()
        .count()
        + 1;
    (line, column)
}

fn parse_integer(token: &str) -> Option<BigInt> {
    if let Some(unsigned) = token.strip_prefix('-') {
        return parse_unsigned_integer(unsigned).map(std::ops::Neg::neg);
    }
    parse_unsigned_integer(token)
}

fn parse_rational(token: &str) -> Option<BigRational> {
    if let Some(unsigned) = token.strip_prefix('-') {
        return parse_unsigned_rational(unsigned).map(std::ops::Neg::neg);
    }
    parse_unsigned_rational(token)
}

fn parse_unsigned_rational(token: &str) -> Option<BigRational> {
    let (mantissa, exponent) = if let Some(offset) = token.find(['e', 'E']) {
        (
            &token[..offset],
            parse_signed_decimal_integer(&token[offset + 1..])?,
        )
    } else {
        (token, BigInt::from(0))
    };
    let (integer, fractional) = mantissa
        .split_once('.')
        .map_or((mantissa, ""), |parts| parts);
    if !valid_decimal_integer(integer) || (!fractional.is_empty() && !valid_fractional(fractional))
    {
        return None;
    }
    if fractional.is_empty() && !token.contains(['e', 'E']) {
        return None;
    }
    let integer_digits = integer.replace('_', "");
    let fractional_digits = fractional.replace('_', "");
    let numerator = format!("{integer_digits}{fractional_digits}")
        .parse::<BigInt>()
        .ok()?;
    let scale = BigInt::from(fractional_digits.len()) - exponent;
    if scale >= BigInt::from(0) {
        Some(BigRational::new(
            numerator,
            pow_int(BigInt::from(10), scale),
        ))
    } else {
        Some(BigRational::from_integer(
            numerator * pow_int(BigInt::from(10), -scale),
        ))
    }
}

fn parse_signed_decimal_integer(token: &str) -> Option<BigInt> {
    let (negative, unsigned) = token
        .strip_prefix('-')
        .map_or((false, token), |value| (true, value));
    let unsigned = unsigned.strip_prefix('+').unwrap_or(unsigned);
    if !valid_decimal_integer(unsigned) {
        return None;
    }
    let value = unsigned.replace('_', "").parse::<BigInt>().ok()?;
    Some(if negative { -value } else { value })
}

fn valid_fractional(token: &str) -> bool {
    if !token.contains('_') {
        return token.bytes().all(|byte| byte.is_ascii_digit());
    }
    let groups = token.split('_').collect::<Vec<_>>();
    groups
        .first()
        .is_some_and(|group| group.len() == 3 && group.bytes().all(|byte| byte.is_ascii_digit()))
        && groups.iter().skip(1).enumerate().all(|(index, group)| {
            let final_group = index + 2 == groups.len();
            (group.len() == 3 || (final_group && (1..=2).contains(&group.len())))
                && group.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn parse_unsigned_integer(token: &str) -> Option<BigInt> {
    if valid_decimal_integer(token) {
        return token.replace('_', "").parse().ok();
    }
    let (radix, digits) = if let Some(digits) = token.strip_prefix("0b") {
        (2, digits)
    } else if let Some(digits) = token.strip_prefix("0o") {
        (8, digits)
    } else {
        (16, token.strip_prefix("0x")?)
    };
    valid_based_digits(digits, radix)
        .then(|| BigInt::parse_bytes(digits.replace('_', "").as_bytes(), radix))
        .flatten()
}

fn valid_decimal_integer(token: &str) -> bool {
    if token == "0" {
        return true;
    }
    if token.starts_with('0')
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'_')
    {
        return false;
    }
    if !token.contains('_') {
        return true;
    }
    let mut groups = token.split('_');
    let first = groups.next().unwrap_or_default();
    (1..=3).contains(&first.len())
        && first.bytes().all(|byte| byte.is_ascii_digit())
        && groups.all(|group| group.len() == 3 && group.bytes().all(|byte| byte.is_ascii_digit()))
}

fn valid_based_digits(digits: &str, radix: u32) -> bool {
    if digits.is_empty() {
        return false;
    }
    let valid_group =
        |group: &str| !group.is_empty() && group.chars().all(|character| character.is_digit(radix));
    if !digits.contains('_') {
        return valid_group(digits);
    }
    let mut groups = digits.split('_');
    let first = groups.next().unwrap_or_default();
    (1..=4).contains(&first.len())
        && valid_group(first)
        && groups.all(|group| group.len() == 4 && valid_group(group))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(source: &str) -> Result<Value, Diagnostic> {
        Session::new().evaluate(source, &mut std::io::sink())
    }

    #[test]
    fn advances_a_prepared_execution_one_statement_at_a_time() {
        let mut session = Session::new();
        let mut trace = Vec::new();
        let mut execution = session
            .prepare("answer is 40\nanswer + 2\n", &mut trace)
            .unwrap();

        let first = execution.step(&mut session, &mut trace).unwrap();
        assert!(matches!(
            first,
            ExecutionStep::Advanced {
                value: Value::Unit,
                ..
            }
        ));
        assert!(
            !trace
                .iter()
                .any(|event| event.contains("evaluation.result"))
        );

        let second = execution.step(&mut session, &mut trace).unwrap();
        assert!(matches!(second, ExecutionStep::Complete(Value::Int(_))));
        assert!(trace.last().unwrap().contains("evaluation.result"));
    }

    #[test]
    fn evaluates_discard_without_introducing_a_binding() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("_ is 20 + 22\n7\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "7");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("binding.discarded"))
        );
        assert!(
            Session::new()
                .evaluate("_\n", &mut std::io::sink())
                .is_err()
        );
    }

    #[test]
    fn evaluates_labeled_record_products_in_field_order() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("(name is \"Ada\", active is true)\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "(name is \"Ada\", active is true)");
        assert!(trace.iter().any(|event| event.contains("product.record")));

        let duplicate = Session::new()
            .evaluate("(name is 1, name is 2)\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(duplicate.code, "E-DUPLICATE-RECORD-FIELD");

        let mixed = Session::new()
            .evaluate("(1, name is 2)\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(mixed.code, "E-MIXED-PRODUCT-FIELDS");
    }

    #[test]
    fn selects_record_fields_without_resolving_the_label_as_a_name() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate(
                "person is (name is \"Ada\", active is true)\nperson name\n",
                &mut trace,
            )
            .unwrap();
        assert_eq!(value.to_string(), "\"Ada\"");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("record.field.selected"))
        );

        let error = Session::new()
            .evaluate("(name is \"Ada\") age\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-SUCH-RECORD-FIELD");
    }

    #[test]
    fn derives_equality_for_records_with_the_same_shape() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate(
                "(name is \"Ada\", score is 1) = (score is 1.0, name is \"Ada\")\n",
                &mut trace,
            )
            .unwrap();
        assert_eq!(value.to_string(), "true");
        assert!(trace.iter().any(|event| event.contains("Int->Rational")));

        let error = Session::new()
            .evaluate(
                "(name is \"Ada\") = (name is \"Ada\", active is true)\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn concatenates_plain_strings_without_normalizing_the_join() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"e\" concat \"\u{301}\"\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"e\u{301}\"");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-CONCAT-001"))
        );

        let error = Session::new()
            .evaluate("\"value\" concat 1\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn composes_only_adjacent_string_literals_implicitly() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"Hello, \" \"Topal\"\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"Hello, Topal\"");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-LITERAL-COMPOSE-001"))
        );

        let value = Session::new()
            .evaluate(
                "left is \"Hello, \"\nright is \"Topal\"\nleft concat right\n",
                &mut std::io::sink(),
            )
            .unwrap();
        assert_eq!(value.to_string(), "\"Hello, Topal\"");

        let error = Session::new()
            .evaluate(
                "left is \"Hello, \"\nright is \"Topal\"\nleft right\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
        assert_eq!(error.code, "E-UNSUPPORTED-APPLICATION");
    }

    #[test]
    fn constructs_the_unique_empty_plain_string() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("empty String\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"\"");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-EMPTY-001"))
        );
    }

    #[test]
    fn tests_plain_string_emptiness() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("empty? (empty String)\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "true");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-EMPTY-PREDICATE-001"))
        );
        assert_eq!(
            Session::new()
                .evaluate("empty? \"Topal\"\n", &mut std::io::sink())
                .unwrap()
                .to_string(),
            "false"
        );
    }

    #[test]
    fn counts_unicode_user_perceived_characters() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("character-count \"a\u{301}👩‍🔬🇸🇪\"\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "3");
        assert!(trace.iter().any(|event| {
            event.contains("TOPAL-STRING-CHARACTER-COUNT-001") && event.contains("characters=3")
        }));

        let error = Session::new()
            .evaluate("character-count 1\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn string_entry_count_agrees_with_character_count() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("entry-count \"a\u{301}👩‍🔬🇸🇪\"\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "3");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-ENTRY-COUNT-001"))
        );
    }

    #[test]
    fn counts_prospective_utf8_bytes_without_normalizing() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"e\u{301}👩‍🔬\" byte-count Utf8\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "14");
        assert!(trace.iter().any(|event| {
            event.contains("TOPAL-STRING-UTF8-BYTE-COUNT-001") && event.contains("bytes=14")
        }));

        let error = Session::new()
            .evaluate("\"text\" byte-count Utf16\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn normalizes_plain_strings_to_nfc_explicitly() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"e\u{301}\" normalize NFC\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"é\"");
        assert!(trace.iter().any(|event| {
            event.contains("TOPAL-STRING-NORMALIZE-NFC-001") && event.contains("changed=true")
        }));
    }

    #[test]
    fn normalizes_plain_strings_to_nfd_explicitly() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"é\" normalize NFD\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"e\u{301}\"");
        assert!(trace.iter().any(|event| {
            event.contains("TOPAL-STRING-NORMALIZE-NFD-001") && event.contains("changed=true")
        }));

        let error = Session::new()
            .evaluate("\"text\" normalize NFKD\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn adds_signed_arbitrary_precision_integers() {
        assert_eq!(
            evaluate("-1 + 123456789012345678901234567890")
                .unwrap()
                .to_string(),
            "123456789012345678901234567889"
        );
    }

    #[test]
    fn follows_left_association() {
        assert_eq!(evaluate("1 + 2 + 3").unwrap().to_string(), "6");
    }

    #[test]
    fn negates_and_subtracts_exact_integers() {
        assert_eq!(evaluate("- 42").unwrap().to_string(), "-42");
        assert_eq!(evaluate("10 - 3 - 2").unwrap().to_string(), "5");
        assert_eq!(evaluate("10 - -2").unwrap().to_string(), "12");
    }

    #[test]
    fn multiplies_without_hidden_precedence() {
        assert_eq!(evaluate("2 + 3 * 4").unwrap().to_string(), "20");
        assert_eq!(evaluate("2 + (3 * 4)").unwrap().to_string(), "14");
        assert_eq!(
            evaluate("99999999999999999999 * 99999999999999999999")
                .unwrap()
                .to_string(),
            "9999999999999999999800000000000000000001"
        );
    }

    #[test]
    fn divides_to_canonical_rational() {
        assert_eq!(evaluate("6 / 8").unwrap().to_string(), "Rational ( 3, 4 )");
        assert_eq!(
            evaluate("6 / -8").unwrap().to_string(),
            "Rational ( -3, 4 )"
        );
        assert_eq!(evaluate("6 / 3").unwrap().to_string(), "Rational ( 2, 1 )");
    }

    #[test]
    fn rejects_statically_evident_zero_divisor() {
        assert_eq!(evaluate("1 / 0").unwrap_err().code, "E-DIVISION-BY-ZERO");
    }

    #[test]
    fn renders_unicode_aligned_actionable_diagnostics() {
        let error = evaluate("α is 1\nα + missing").unwrap_err();
        assert_eq!(
            error.render("example.t"),
            "error[E-UNBOUND-NAME]: name is not bound\n --> example.t:2:5\n  |\n2 | α + missing\n  |     ^^^^^^^\n  |\n  = help: declare this name earlier in the same source session"
        );
    }

    #[test]
    fn edit_distance_counts_unicode_scalars() {
        assert_eq!(edit_distance("räknare", "räknaren"), 1);
        assert_eq!(edit_distance("αβ", "βα"), 2);
    }

    #[test]
    fn diagnostics_suggest_root_operations_and_the_concat_migration() {
        let error = Session::new()
            .evaluate("charcter-count \"Topal\"\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-UNBOUND-NAME");
        assert_eq!(
            error.help.as_deref(),
            Some("did you mean `character-count`?")
        );

        let error = Session::new()
            .evaluate("\"a\" concatenate \"b\"\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-UNSUPPORTED-APPLICATION");
        assert_eq!(error.help.as_deref(), Some("did you mean `concat`?"));
    }

    #[test]
    fn raises_integer_to_natural_power_exactly() {
        assert_eq!(
            evaluate("2 ^ 100").unwrap().to_string(),
            "1267650600228229401496703205376"
        );
        assert_eq!(evaluate("0 ^ 0").unwrap().to_string(), "1");
        assert_eq!(evaluate("-2 ^ 3").unwrap().to_string(), "-8");
    }

    #[test]
    fn exponentiation_uses_ordinary_left_association() {
        assert_eq!(evaluate("2 + 3 ^ 2").unwrap().to_string(), "25");
        assert_eq!(evaluate("2 + (3 ^ 2)").unwrap().to_string(), "11");
    }

    #[test]
    fn raises_rationals_to_natural_powers_exactly() {
        assert_eq!(
            evaluate("1.5 ^ 3").unwrap().to_string(),
            "Rational ( 27, 8 )"
        );
        assert_eq!(
            evaluate("0.0 ^ 0").unwrap().to_string(),
            "Rational ( 1, 1 )"
        );
        assert_eq!(
            evaluate("1.5 ^ -2").unwrap().to_string(),
            "Rational ( 4, 9 )"
        );
        assert_eq!(
            evaluate("1.5 ^ 2.0").unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
        assert_eq!(evaluate("0.0 ^ -1").unwrap_err().code, "E-DIVISION-BY-ZERO");
    }

    #[test]
    fn rejects_negative_integer_exponent() {
        assert_eq!(
            evaluate("2 ^ -1").unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }

    #[test]
    fn constructs_exact_rational_literals() {
        assert_eq!(evaluate("0.1").unwrap().to_string(), "Rational ( 1, 10 )");
        assert_eq!(
            evaluate("1.25e3").unwrap().to_string(),
            "Rational ( 1250, 1 )"
        );
        assert_eq!(
            evaluate("-6.022e-24").unwrap().to_string(),
            "Rational ( -3011, 500000000000000000000000000 )"
        );
        assert_eq!(
            evaluate("1_000.000_125").unwrap().to_string(),
            "Rational ( 8000001, 8000 )"
        );
    }

    #[test]
    fn rejects_malformed_rational_literal() {
        assert_eq!(evaluate("1.2e").unwrap_err().code, "E-NUMERIC-LITERAL");
    }

    #[test]
    fn evaluates_exact_rational_arithmetic() {
        assert_eq!(
            evaluate("0.5 + 0.25").unwrap().to_string(),
            "Rational ( 3, 4 )"
        );
        assert_eq!(
            evaluate("- 1.5 - 0.25").unwrap().to_string(),
            "Rational ( -7, 4 )"
        );
        assert_eq!(
            evaluate("1.5 * 0.5").unwrap().to_string(),
            "Rational ( 3, 4 )"
        );
        assert_eq!(
            evaluate("1.5 / 0.25").unwrap().to_string(),
            "Rational ( 6, 1 )"
        );
    }

    #[test]
    fn converts_int_for_mixed_exact_arithmetic() {
        assert_eq!(
            evaluate("1 + 0.5").unwrap().to_string(),
            "Rational ( 3, 2 )"
        );
        assert_eq!(
            evaluate("0.5 * 2").unwrap().to_string(),
            "Rational ( 1, 1 )"
        );
        assert_eq!(
            evaluate("1 / 0.5").unwrap().to_string(),
            "Rational ( 2, 1 )"
        );
    }

    #[test]
    fn rejects_rational_zero_divisor() {
        assert_eq!(
            evaluate("1.0 / 0.0").unwrap_err().code,
            "E-DIVISION-BY-ZERO"
        );
    }

    #[test]
    fn preserves_ordinary_and_tagged_string_contents() {
        assert_eq!(
            evaluate(r#""plain\n{value}""#).unwrap().to_string(),
            r#""plain\n{value}""#
        );
        assert_eq!(
            evaluate(r#"text"He said "hello"."text"#)
                .unwrap()
                .to_string(),
            r#"text"He said "hello"."text"#
        );
        assert_eq!(
            evaluate("\"first\nsecond\"").unwrap().to_string(),
            "\"first\nsecond\""
        );
    }

    #[test]
    fn display_extends_colliding_string_tag() {
        assert_eq!(
            evaluate(r#"tag"contains "text closing"tag"#)
                .unwrap()
                .to_string(),
            r#"text_"contains "text closing"text_"#
        );
    }

    #[test]
    fn parentheses_group_addition() {
        assert_eq!(evaluate("1 + (2 + 3)").unwrap().to_string(), "6");
    }

    #[test]
    fn evaluates_binding_and_lookup() {
        assert_eq!(
            evaluate("answer is 40 + 2\nanswer").unwrap().to_string(),
            "42"
        );
    }

    #[test]
    fn rejects_incomplete_grouping() {
        assert_eq!(evaluate("12_34").unwrap_err().code, "E-NUMERIC-LITERAL");
    }
}
#[test]
fn declares_and_calls_static_nullary_functions() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn static () -> Int\n  40 + 2\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let declared = trace
        .iter()
        .position(|event| event.contains("function.declared"))
        .unwrap();
    let entered = trace
        .iter()
        .position(|event| event.contains("function.entered"))
        .unwrap();
    let returned = trace
        .iter()
        .position(|event| event.contains("function.returned"))
        .unwrap();
    assert!(declared < entered && entered < returned);
}

#[test]
fn static_function_body_uses_declaration_order_lexical_bindings() {
    let value = Session::new()
        .evaluate(
            "base is 40\nanswer is fn static () -> Int\n  base + 2\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");

    let error = Session::new()
        .evaluate(
            "answer is fn static () -> Int\n  later + 2\nlater is 40\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn static_unary_function_binds_a_typed_local_parameter() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "increment is fn static (input : Int) -> Int\n  input + 1\nincrement 41\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("function.argument.bound") && event.contains("input") })
    );

    let error = Session::new()
        .evaluate(
            "increment is fn static (input : Int) -> Int\n  input + 1\ninput\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn static_product_function_binds_typed_parameters_in_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "subtract is fn static (left : Int, right : Int) -> Int\n  left - right\n50 subtract 8\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let bindings = trace
        .iter()
        .filter(|event| event.contains("function.argument.bound"))
        .collect::<Vec<_>>();
    assert!(bindings[0].contains("left"));
    assert!(bindings[1].contains("right"));

    let error = Session::new()
        .evaluate(
            "bad is fn static (value : Int, value : Int) -> Int\n  value\nbad (1, 2)\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-DUPLICATE-FUNCTION-PARAMETER");
}

#[test]
fn function_block_bindings_are_local_to_each_invocation() {
    let mut trace = Vec::new();
    let mut session = Session::new();
    let value = session
        .evaluate(
            "answer is fn static () -> Int\n  local is 40 + 2\n  local\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let created = trace
        .iter()
        .position(|event| event.contains("binding.created") && event.contains("local"))
        .unwrap();
    let resolved = trace
        .iter()
        .position(|event| event.contains("binding.resolved") && event.contains("local"))
        .unwrap();
    assert!(created < resolved);

    let error = session
        .evaluate("local\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");

    let error = session
        .evaluate(
            "invalid is fn static () -> Int\n  1\n  2\ninvalid ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-DISCARDED-VALUE");
}

#[test]
fn explicit_return_skips_later_function_statements() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn static () -> Int\n  return 40 + 2\n  missing\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.return.explicit"))
    );
    assert!(!trace.iter().any(|event| event.contains("missing")));

    let error = Session::new()
        .evaluate("return 42\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn ordinary_runtime_function_uses_ordinary_trace_rule() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "subtract is fn (left : Int, right : Int) -> Int\n  left - right\n50 subtract 8\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .filter(|event| event.contains("function."))
            .all(|event| event.contains("TOPAL-FUNCTION-ORDINARY-001")
                || event.contains("TOPAL-TYPE-CALL-001"))
    );
}

#[test]
fn nat_classifiers_accept_only_nonnegative_int_values() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "identity is fn (value : Nat) -> Nat\n  value\nidentity 42\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(trace.iter().any(|event| event.contains("identity (Nat)")));

    let argument_error = Session::new()
        .evaluate(
            "identity is fn (value : Nat) -> Nat\n  value\nidentity -1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(argument_error.code, "E-FUNCTION-ARGUMENT-TYPE");

    let result_error = Session::new()
        .evaluate(
            "negative is fn () -> Nat\n  -1\nnegative ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(result_error.code, "E-FUNCTION-RESULT-TYPE");
}

#[test]
fn proves_unit_step_nat_recursion_without_overshoot() {
    let source = "count-down is fn (value : Nat) -> Nat\n  value\n    <= 0 then 0\n    otherwise count-down (value - 1)\ncount-down 3\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "0"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-001"))
    );

    let error = Session::new()
        .evaluate(
            &source.replace("value - 1", "value - 2"),
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RECURSION-NOT-YET-PROVEN");
}

#[test]
fn proves_nat_decrement_when_the_bound_prevents_overshoot() {
    let safe = "count-down is fn (value : Nat) -> Nat\n  value\n    <= 2 then value\n    otherwise count-down (value - 3)\ncount-down 8\n";
    assert_eq!(
        Session::new()
            .evaluate(safe, &mut std::io::sink())
            .unwrap()
            .to_string(),
        "2"
    );
    let error = Session::new()
        .evaluate(
            &safe.replace("value - 3", "value - 4"),
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RECURSION-NOT-YET-PROVEN");
}

#[test]
fn proves_increasing_nat_recursion_with_positive_steps() {
    let source = "advance is fn (value : Nat) -> Nat\n  value\n    >= 5 then value\n    otherwise advance (value + 2)\nadvance 0\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "6"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"))
    );
}

#[test]
fn proves_closed_mutual_nat_recursion() {
    let source = "even is fn (value : Nat) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Nat) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\n(even 6, odd 6)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(true, false)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"))
    );
}

#[test]
fn proves_closed_mutual_increasing_nat_recursion() {
    let source = "even is fn (value : Nat) -> Boolean\n  value\n    >= 6 then true\n    otherwise odd (value + 1)\nodd is fn (value : Nat) -> Boolean\n  value\n    >= 6 then false\n    otherwise even (value + 1)\n(even 0, odd 0)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(true, false)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"))
    );
}

#[test]
fn declares_nominal_payload_free_enum_values() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "Color is Enum (Red, Green, Blue)\n(Red, Green, Red = Red, Red = Green)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Red, Green, true, false)");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-TYPE-ENUM-001"))
    );
}

#[test]
fn validates_enum_function_parameters_and_results() {
    let source = "Color is Enum (Red, Green)\nidentity is fn (value : Color) -> Color\n  value\n(identity Red, identity Green)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(Red, Green)"
    );
    assert!(trace.iter().any(|event| event.contains("identity (Color)")));
}

#[test]
fn executes_only_complete_enum_decisions() {
    let source = "Color is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"red\"\n    Green then \"green\"\nname Green\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "\"green\""
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-DECISION-ENUM-001"))
    );
}

#[test]
fn resolves_namespaced_arithmetic_error_codes_without_a_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(lang arithmetic division-by-zero) = (lang arithmetic division-by-zero)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(true));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-NUM-ARITHMETIC-ERROR-001"))
    );
}

#[test]
fn matches_both_result_paths_exhaustively() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error problem then \"error\"\n(describe 2.0, describe 0.0)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(\"ok\", \"error\")"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-DECISION-RESULT-001"))
    );
}

#[test]
fn nested_function_calls_preserve_staticness_and_detect_cycles() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  increment 41\nincrement is fn (input : Int) -> Int\n  input + 1\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let outer_entry = trace
        .iter()
        .position(|event| event.contains("function.entered") && event.contains("answer"))
        .unwrap();
    let inner_entry = trace
        .iter()
        .position(|event| event.contains("function.entered") && event.contains("increment"))
        .unwrap();
    let inner_return = trace
        .iter()
        .position(|event| event.contains("function.returned") && event.contains("increment"))
        .unwrap();
    let outer_return = trace
        .iter()
        .position(|event| event.contains("function.returned") && event.contains("answer"))
        .unwrap();
    assert!(outer_entry < inner_entry && inner_entry < inner_return && inner_return < outer_return);

    let recursion = Session::new()
        .evaluate(
            "again is fn () -> Int\n  again ()\nagain ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(recursion.code, "E-RECURSION-NOT-YET-PROVEN");
}

#[test]
fn function_local_binding_shadows_capture_without_leaking() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "value is 40\nanswer is fn () -> Int\n  value is 42\n  value\n(answer (), value)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 40)");

    let duplicate = Session::new()
        .evaluate(
            "bad is fn (value : Int) -> Int\n  value is 42\n  value\nbad 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(duplicate.code, "E-DUPLICATE-BINDING");
}

#[test]
fn overload_selection_uses_input_classifier_and_rejects_duplicate_signature() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  value\n(describe 42, describe \"Topal\")\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"integer\", \"Topal\")");
    assert!(trace.iter().any(|event| event.contains("describe (Int)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("describe (String)"))
    );

    let duplicate = Session::new()
        .evaluate(
            "same is fn (first : Int) -> Int\n  first\nsame is fn (second : Int) -> String\n  \"duplicate\"\nsame 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(duplicate.code, "E-DUPLICATE-FUNCTION-OVERLOAD");
}

#[test]
fn boolean_decision_evaluates_only_selected_action() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    otherwise missing\nchoose true\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
    assert!(!trace.iter().any(|event| event.contains("missing")));
}

#[test]
fn exhaustive_boolean_decision_selects_both_literal_cases() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    false then 0\n(choose true, choose false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 0)");
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=1") })
    );
}

#[test]
fn earlier_function_body_calls_later_declaration() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Int\n  second value\nsecond is fn (value : Int) -> Int\n  value + 1\nfirst 41\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let first = trace
        .iter()
        .position(|event| event.contains("function.entered") && event.contains("first"))
        .unwrap();
    let second = trace
        .iter()
        .position(|event| event.contains("function.entered") && event.contains("second"))
        .unwrap();
    assert!(first < second);
}

#[test]
fn mutual_int_recursion_executes_only_when_every_cycle_edge_decreases() {
    let source = "even is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\n(even 6, odd 6)\n";
    let mut trace = Vec::new();
    let value = Session::new().evaluate(source, &mut trace).unwrap();
    assert_eq!(value.to_string(), "(true, false)");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.recursion.cycle.proven"))
    );

    let three_member = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise third (value - 1)\nthird is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first (value - 1)\nfirst 3\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(three_member.to_string(), "true");

    let invalid = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first value\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(invalid.code, "E-RECURSION-NOT-YET-PROVEN");
}

#[test]
fn mutual_increasing_int_recursion_requires_one_direction_for_the_complete_cycle() {
    let source = "even-up is fn (value : Int) -> Boolean\n  value\n    >= 0 then true\n    otherwise odd-up (value + 1)\nodd-up is fn (value : Int) -> Boolean\n  value\n    >= 0 then false\n    otherwise even-up (value + 1)\n(even-up (-6), odd-up (-6))\n";
    let mut trace = Vec::new();
    let value = Session::new().evaluate(source, &mut trace).unwrap();
    assert_eq!(value.to_string(), "(true, false)");
    assert!(trace.iter().any(|event| {
        event.contains("function.recursion.cycle.proven")
            && event.contains("TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001")
    }));

    let mixed = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    >= 10 then false\n    otherwise first (value + 1)\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(mixed.code, "E-RECURSION-NOT-YET-PROVEN");
}

#[test]
fn same_named_distinct_overloads_are_not_recursive() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  (describe 42) concat \":\" concat value\ndescribe \"Topal\"\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "\"integer:Topal\"");
    let string = trace
        .iter()
        .position(|event| event.contains("describe (String)"))
        .unwrap();
    let integer = trace
        .iter()
        .position(|event| event.contains("describe (Int)"))
        .unwrap();
    assert!(string < integer);
}

#[test]
fn bounded_int_recursion_accepts_only_positive_literal_progress() {
    let value = Session::new()
        .evaluate(
            "down is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise 1 + (down (value - 3))\nup is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise 1 + (up (value + 2))\n(down 7, up (-5))\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(3, 3)");

    let mutual = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 2)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first (value - 3)\nfirst 7\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(mutual.to_string(), "false");

    for invalid_step in ["0", "-1"] {
        let source = format!(
            "stuck is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise stuck (value - {invalid_step})\nstuck 1\n"
        );
        let error = Session::new()
            .evaluate(&source, &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-RECURSION-NOT-YET-PROVEN");
    }
}

#[test]
fn every_recursive_call_in_one_action_must_progress() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "branch-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (branch-count (value - 1)) + (branch-count (value - 2))\nbranch-count 3\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "5");
    assert!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count()
            > 2
    );

    let error = Session::new()
        .evaluate(
            "unsafe-branch is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (unsafe-branch (value - 1)) + (unsafe-branch value)\nunsafe-branch 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RECURSION-NOT-YET-PROVEN");
}

#[test]
fn every_call_on_one_mutual_edge_must_share_target_and_progress() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "first-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second-count (value - 1)) + (second-count (value - 2))\nsecond-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first-count (value - 1)\nfirst-count 3\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "3");
    assert!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count()
            > 1
    );

    let error = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second (value - 1)) + (second value)\nsecond is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first (value - 1)\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RECURSION-NOT-YET-PROVEN");

    let different_target = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second (value - 1)) + (third (value - 1))\nsecond is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first (value - 1)\nthird is fn (value : Int) -> Int\n  value\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(different_target.code, "E-RECURSION-NOT-YET-PROVEN");
}

#[test]
fn comparison_decision_uses_subject_as_left_operand() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "minimum is fn (left : Int, right : Int) -> Int\n  left\n    < right then left\n    otherwise missing\n42 minimum 50\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(trace.iter().any(|event| {
        event.contains("decision.rule.selected") && event.contains("TOPAL-DECISION-COMPARISON-001")
    }));
    assert!(!trace.iter().any(|event| event.contains("missing")));
}

#[test]
fn decreasing_int_recursion_executes_only_after_structural_proof() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "sum-down is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise value + (sum-down (value - 1))\nsum-down 5\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "15");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.recursion.proven"))
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count(),
        5
    );

    let unproven = Session::new()
        .evaluate(
            "wrong is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise wrong (value + 1)\nwrong 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(unproven.code, "E-RECURSION-NOT-YET-PROVEN");
}

#[test]
fn increasing_int_recursion_executes_only_after_structural_proof() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "distance-up is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise 1 + (distance-up (value + 1))\ndistance-up (-5)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "5");
    assert!(trace.iter().any(|event| {
        event.contains("function.recursion.proven")
            && event.contains("TOPAL-FUNCTION-RECURSION-INT-INCREASING-001")
    }));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count(),
        5
    );
}

#[test]
fn comparison_matcher_evaluates_complete_operand_expression() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "within is fn (value : Int, limit : Int) -> Boolean\n  value\n    < limit + 1 then true\n    otherwise false\n(5 within 5, 6 within 5)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false)");
    let addition = trace
        .iter()
        .position(|event| event.contains("root.+(Int,Int)"))
        .unwrap();
    let comparison = trace
        .iter()
        .position(|event| event.contains("root.<(TotalOrder,TotalOrder)"))
        .unwrap();
    assert!(addition < comparison);
}

#[test]
fn nested_function_captures_outer_parameter_without_leaking() {
    let mut session = Session::new();
    let mut trace = Vec::new();
    let value = session
        .evaluate(
            "answer is fn (input : Int) -> Int\n  add-input is fn (value : Int) -> Int\n    value + input\n  add-input 2\nanswer 40\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let outer_entry = trace
        .iter()
        .position(|event| event.contains("function.entered") && event.contains("answer"))
        .unwrap();
    let nested_declaration = trace
        .iter()
        .position(|event| event.contains("function.declared") && event.contains("add-input"))
        .unwrap();
    let nested_entry = trace
        .iter()
        .position(|event| event.contains("function.entered") && event.contains("add-input"))
        .unwrap();
    assert!(outer_entry < nested_declaration && nested_declaration < nested_entry);

    let error = session
        .evaluate("add-input 2\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn structured_error_fields_retain_code_type_and_domain_identity() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nproblem is 1.0 divide 0.0\n(problem code, problem domain)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value,
        Value::Tuple(vec![
            Value::Enum {
                type_name: "lang arithmetic ArithmeticErrorCode".into(),
                alternative: "division-by-zero".into(),
            },
            Value::ErrorDomain("root./(Rational,Rational)".into()),
        ])
    );
    assert_eq!(
        value.to_string(),
        "(division-by-zero, root./(Rational,Rational))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("error.field.selected"))
            .count(),
        2
    );
}

#[test]
fn qualified_error_code_pattern_selects_without_using_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"\n    Error problem then \"other\"\ndescribe 0.0\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("zero".into()));
    assert!(trace.iter().any(|event| {
        event.contains("error.code.matched") && event.contains("TOPAL-DECISION-ERROR-CODE-001")
    }));
}

#[test]
fn classified_binding_projects_success_and_propagates_error() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nproject is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  quotient : Rational is 1.0 divide denominator\n  quotient + 1.0\n(project 2.0, project 0.0)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 3, 2 ), Error ( domain is root./(Rational,Rational), code is division-by-zero ))"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("result.success.projected"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("result.error.projected"))
    );
}

#[test]
fn classified_binding_rejects_error_propagation_from_infallible_function() {
    let error = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nbad is fn (denominator : Rational) -> Rational\n  quotient : Rational is 1.0 divide denominator\n  quotient\nbad 0.0\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RESULT-PROJECTION-INFALLIBLE");
    assert!(error.message.contains("returning `Rational`"));
    assert!(
        error
            .help
            .is_some_and(|help| help.contains("match the Error"))
    );
}

#[test]
fn character_classifier_uses_pinned_grapheme_segmentation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "identity is fn (value : Character) -> Character\n  value\ncomposed : Character is \"a\u{301}\"\n(String (identity \"🙂\"), String composed)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"🙂\", \"a\u{301}\")");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-STRING-FROM-CHARACTER-001"))
            .count(),
        2
    );

    let error = Session::new()
        .evaluate("invalid : Character is \"ab\"\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-CHARACTER-CLASSIFIER");
    assert!(error.message.contains("contains 2"));
}

#[test]
fn int_modulo_is_euclidean_and_dynamic_zero_returns_error() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "modulo is fn (left : Int, right : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  left % right\nquotient-modulo is fn (left : Int, right : Int) -> Result ((Int, Int), lang arithmetic ArithmeticErrorCode)\n  left /% right\n(17 % 5, -17 % 5, 17 % -5, -17 /% 5, 17 /% -5, 17 modulo 0, 17 quotient-modulo 0)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(2, 3, 2, (-4, 3), (-3, 2), Error ( domain is root.%(Int,Int), code is division-by-zero ), Error ( domain is root./%(Int,Int), code is division-by-zero ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-INT-MODULO-001"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-INT-QUOTIENT-MODULO-001"))
            .count(),
        2
    );
}

#[test]
fn exact_numeric_absolute_retains_operand_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(absolute -42, absolute 42, absolute -1.25, absolute 1.25)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(42, 42, Rational ( 5, 4 ), Rational ( 5, 4 ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-ABS-001"))
            .count(),
        4
    );
}

#[test]
fn named_numeric_negate_matches_exact_additive_inverse() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(negate 42, negate -42, negate 1.25, negate -1.25)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(-42, 42, Rational ( -5, 4 ), Rational ( 5, 4 ))"
    );
    assert!(trace.iter().any(|event| event.contains("root.negate(Int)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.negate(Rational)"))
    );
}

#[test]
fn exact_numeric_zero_uses_explicit_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(zero Int, zero Nat, zero Rational, one Int, one Nat, one Rational)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(0, 0, Rational ( 0, 1 ), 1, 1, Rational ( 1, 1 ))"
    );
    assert!(trace.iter().any(|event| event.contains("root.zero(Int)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.zero(Rational)"))
    );
    assert!(trace.iter().any(|event| event.contains("root.one(Int)")));
    assert!(trace.iter().any(|event| event.contains("root.zero(Nat)")));
    assert!(trace.iter().any(|event| event.contains("root.one(Nat)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.one(Rational)"))
    );
}

#[test]
fn exact_three_way_comparison_returns_nominal_alternatives() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("describe is fn (value : Comparison) -> String\n  value\n    Less then \"less\"\n    Equal then \"equal\"\n    Greater then \"greater\"\n(1 <=> 2, 2 <=> 2, 3 <=> 2, 1 <=> 1.5, describe (1 <=> 2), describe (2 <=> 2), describe (3 <=> 2))\n", &mut trace)
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Less, Equal, Greater, Less, \"less\", \"equal\", \"greater\")"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-THREE-WAY-COMPARE-001"))
            .count(),
        7
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("decision.rule.selected"))
            .filter(|event| event.contains("TOPAL-DECISION-ENUM-001"))
            .count(),
        3
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Int->Rational:left"))
    );
}

#[test]
fn closed_exact_rational_narrows_to_int_without_rounding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "fifty : Int is 100 / 2\nnegative-three : Int is -9 / 3\n(fifty, negative-three)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(50, -3)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-RATIONAL-INT-EXACT-001"))
            .count(),
        2
    );

    let error = Session::new()
        .evaluate("half : Int is 1 / 2\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RATIONAL-NOT-EXACT-INT");
    assert!(error.message.contains("denominator 2"));
}

#[test]
fn dynamic_rational_to_int_validation_returns_typed_result() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "halve is fn (value : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  half : Int is value / 2\n  half\n(halve 100, halve 3)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(50, Error ( domain is root.Int(Rational), code is not-representable ))"
    );
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-NUM-RATIONAL-INT-VALIDATE-001")
            && event.contains("Rational->Int:validated")
    }));
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-NUM-RATIONAL-INT-VALIDATE-001")
            && event.contains("root.Int(Rational);not-representable")
    }));
}

#[test]
fn checked_int_construction_is_exact_and_fallible() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "as-int is fn (value : Rational) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  Int value\n(Int 7, as-int 6.0, as-int 1.5)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(7, 6, Error ( domain is root.Int(Rational), code is not-representable ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-INT-CONSTRUCT-001"))
            .count(),
        3
    );

    let error = Session::new()
        .evaluate("Int 1.5\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RATIONAL-NOT-EXACT-INT");
}

#[test]
fn checked_nat_construction_validates_the_nonnegative_constraint() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "as-nat is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)\n  Nat value\n(Nat 7, as-nat 6, as-nat -1)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(7, 6, Error ( domain is root.Nat(Int), code is out-of-range ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-NAT-CONSTRUCT-001"))
            .count(),
        3
    );

    let error = Session::new()
        .evaluate("Nat -1\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NAT-OUT-OF-RANGE");
}

#[test]
fn closed_rational_construction_canonicalizes_components() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(Rational 7, Rational (2, 4), Rational (2, -4), Rational (0, 5))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 7, 1 ), Rational ( 1, 2 ), Rational ( -1, 2 ), Rational ( 0, 1 ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-001"))
            .count(),
        3
    );
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-NUM-INT-RATIONAL-CONVERT-001")
            && event.contains("Int->Rational:explicit")
    }));

    let error = Session::new()
        .evaluate("Rational (1, 0)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-DIVISION-BY-ZERO");
}

#[test]
fn dynamic_rational_construction_distinguishes_zero_failures() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "ratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  Rational (numerator, denominator)\n(1 ratio 2, 1 ratio 0, 0 ratio 0)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 1, 2 ), Error ( domain is root.Rational(Int,Int), code is division-by-zero ), Error ( domain is root.Rational(Int,Int), code is indeterminate ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001"))
            .count(),
        3
    );

    let error = Session::new()
        .evaluate("Rational (0, 0)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-INDETERMINATE-RATIONAL");
}

#[test]
fn inclusive_int_ranges_preserve_bounds_and_allow_empty_ranges() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("interval is 0 .. 10\nempty-interval is 10 .. 0\n(interval, 5 in interval, interval contains 11, 5 in empty-interval)\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "(0 .. 10, true, false, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-RANGE-INCLUSIVE-001"))
            .count(),
        2
    );
    assert!(trace.iter().any(|event| event.contains("empty")));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-RANGE-MEMBERSHIP-001"))
            .count(),
        3
    );
}

#[test]
fn boolean_not_is_eager_and_type_checked() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("(not true, not false, not (not true))\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "(false, true, true)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-BOOLEAN-LOGIC-001"))
            .count(),
        4
    );
    let error = Session::new()
        .evaluate("not 1\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-BOOLEAN-NOT-OPERAND");
}

#[test]
fn boolean_and_implements_the_eager_truth_table() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(true and true, true and false, false and true, false and false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false, false, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("and:eager"))
            .count(),
        4
    );
}

#[test]
fn boolean_or_implements_the_eager_truth_table() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(true or true, true or false, false or true, false or false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, true, true, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("or:eager"))
            .count(),
        4
    );
}

#[test]
fn boolean_xor_implements_the_eager_truth_table() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(true xor true, true xor false, false xor true, false xor false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(false, true, true, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("xor:eager"))
            .count(),
        4
    );
}

#[test]
fn explicit_optional_constructors_preserve_payload_classifiers() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(Some 42, Some \"present\", None Int, None String)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Some 42, Some \"present\", None, None)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-OPTIONAL-CONSTRUCT-001"))
            .count(),
        4
    );
}

#[test]
fn contextual_none_uses_the_binding_classifier() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("missing : Optional Int is None\nmissing\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "None");
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001") && event.contains("Int")
    }));

    let error = Session::new()
        .evaluate("None\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn optional_values_cross_matching_function_boundaries() {
    let value = Session::new()
        .evaluate(
            "preserve is fn (candidate : Optional Int) -> Optional Int\n  candidate\n(preserve (Some 7), preserve (None Int))\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Some 7, None)");

    let error = Session::new()
        .evaluate(
            "preserve is fn (candidate : Optional Int) -> Optional Int\n  candidate\npreserve (None String)\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-FUNCTION-ARGUMENT-TYPE");
}

#[test]
fn contextual_none_uses_function_result_classifiers() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "implicit is fn () -> Optional Int\n  None\nexplicit is fn () -> Optional String\n  return None\n(implicit (), explicit ())\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(None, None)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001"))
            .count(),
        2
    );
}

#[test]
fn optional_decisions_bind_only_present_payloads() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (candidate : Optional Int) -> String\n  candidate\n    Some payload then \"present\"\n    None then \"absent\"\n(describe (Some 7), describe (None Int))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"present\", \"absent\")");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("decision.rule.selected"))
            .filter(|event| event.contains("TOPAL-DECISION-OPTIONAL-001"))
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("optional.payload.bound"))
            .count(),
        1
    );
}

#[test]
fn optional_equality_uses_nominal_payload_identity() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "((None Int) = (None Int), (Some 7) = (Some 7), (Some 7) = (None Int), (Some 7) != (Some 8))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, true, false, true)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-OPTIONAL-EQUALITY-001"))
            .count(),
        4
    );

    let error = Session::new()
        .evaluate("(None Int) = (None String)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn string_character_at_returns_optional_grapheme_clusters() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "text is \"a\u{301}👩‍🔬🇸🇪\"\n(text character-at 0, text character-at 1, text character-at 2, text character-at -1, text character-at 3)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Some \"a\u{301}\", Some \"👩‍🔬\", Some \"🇸🇪\", None, None)"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-STRING-CHARACTER-AT-001"))
            .count(),
        5
    );
}

#[test]
fn optional_decisions_consume_indexed_characters() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (candidate : Optional Character) -> String\n  candidate\n    Some character then String character\n    None then \"missing\"\n(describe (\"👩‍🔬\" character-at 0), describe (\"👩‍🔬\" character-at 1))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"👩‍🔬\", \"missing\")");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-DECISION-OPTIONAL-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-FROM-CHARACTER-001"))
    );
}

#[test]
fn upper_uses_locale_independent_unicode_mapping() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("upper \"Straße σς\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"STRASSE ΣΣ\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-UPPER-001"))
    );
}

#[test]
fn lower_uses_locale_independent_unicode_mapping() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("lower \"İΣ\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"i\u{307}ς\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-LOWER-001"))
    );
}

#[test]
fn case_fold_uses_full_locale_independent_unicode_mapping() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("case-fold \"Straße Σς\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"strasse σσ\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CASE-FOLD-001"))
    );
}

#[test]
fn canonical_string_equality_preserves_exact_equality_distinction() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "composed is \"é\"\ndecomposed is \"e\u{301}\"\n(composed = decomposed, composed canonically-equals decomposed, composed canonically-equals \"e\")\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(false, true, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-STRING-CANONICAL-EQUALITY-001"))
            .count(),
        2
    );
}

#[test]
fn character_traversal_collects_the_exact_preserved_string() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("characters \"a\u{301}👩‍🔬🇸🇪\" collect String\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"a\u{301}👩‍🔬🇸🇪\"");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        3
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CHARACTERS-COLLECT-001"))
    );
}

#[test]
fn foreach_consumes_character_generator_with_unit() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "characters \"a\u{301}👩‍🔬🇸🇪\" foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.resumed"))
            .count(),
        3
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.returned") && event.contains("Unit"))
    );
}

#[test]
fn foreach_rejects_non_unit_action_result() {
    let error = Session::new()
        .evaluate(
            "characters \"Topal\" foreach { character }\n  String character\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-FOREACH-ACTION-RESULT");
}

#[test]
fn named_character_generator_is_consumed_linearly() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "generated is characters \"a\u{301}👩‍🔬🇸🇪\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.started"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.consumed"))
    );
}

#[test]
fn character_generator_accepts_its_explicit_classifier() {
    let value = Session::new()
        .evaluate(
            "generated : Generator Character Unit Unit is characters \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
}

#[test]
fn function_returns_fresh_character_generator() {
    let value = Session::new()
        .evaluate(
            "generate is fn (text : String) -> Generator Character Unit Unit\n  characters text\ngenerated is generate \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
}

#[test]
fn reused_character_generator_reports_consumption() {
    let error = Session::new()
        .evaluate(
            "generated is characters \"Topal\"\ngenerated foreach { character }\n  _ is String character\ngenerated foreach { character }\n  _ is String character\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-GENERATOR-CONSUMED");
    assert_eq!(
        error.help.as_deref(),
        Some("construct a fresh generator before traversing it again")
    );
}

#[test]
fn generator_parameter_transfers_linear_continuation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "consume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is characters \"Topal\"\nconsume generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.parameter.transferred"))
    );
}

#[test]
fn abandoned_generator_parameter_closes_at_function_boundary() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "ignore is fn (generated : Generator Character Unit Unit) -> Unit\n  ()\ngenerated is characters \"Topal\"\nignore generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CHARACTERS-CLOSE-001"))
    );
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-GENERATOR-ERROR-CODE-001")
            && event.contains("domain=root")
            && event.contains("generator=root.characters")
    }));
}

#[test]
fn generator_error_code_has_qualified_nominal_identity() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "code is lang generator generator-closed\n(code, code = (lang generator generator-closed))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(generator-closed, true)");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-ERROR-CODE-001"))
    );
}

#[test]
fn named_single_yield_generator_is_traversable() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  _ is yield initial\n  ()\ngenerated is once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.declared"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.started"))
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        2
    );
}

#[test]
fn named_generator_yield_reads_local_binding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "copy-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  copy : Character is initial\n  _ is yield copy\n  ()\ngenerated is copy-once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(trace.iter().any(|event| event.contains("binding.created")));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        1
    );
}

#[test]
fn named_generator_can_return_before_first_yield() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "nothing is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  ()\ngenerated is nothing \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("generator.yielded"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.returned"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-EARLY-RETURN-001"))
    );
}

#[test]
fn named_generator_returns_character_after_yields() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "yield-then-return is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Character\n\n  _ is yield initial\n  \"R\"\ngenerated is yield-then-return \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("R".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn custom_generator_defers_post_yield_binding_until_resume() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "pause-twice is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  copy : Character is initial\n  _ is yield copy\n  ()\ngenerated is pause-twice \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let local = trace
        .iter()
        .position(|event| event.contains("binding.created") && event.contains("copy"))
        .unwrap();
    let second_suspend = trace
        .iter()
        .rposition(|event| event.contains("generator.suspended"))
        .unwrap();
    assert!(resumed < local && local < second_suspend);
}

#[test]
fn custom_generator_binds_unit_resume_after_yield() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "bind-resume is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  resumed is yield initial\n  resumed\ngenerated is bind-resume \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let bound = trace
        .iter()
        .position(|event| event.contains("generator.resume.bound"))
        .unwrap();
    let resolved = trace
        .iter()
        .rposition(|event| event.contains("binding.resolved") && event.contains("resumed"))
        .unwrap();
    assert!(resumed < bound && bound < resolved);
}

#[test]
fn abandoned_custom_generator_keeps_domain_separate_from_provenance() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is pause-once initial\n  ()\nabandon \"T\"\n",
            &mut trace,
        )
        .unwrap();
    assert!(trace.iter().any(|event| {
        event.contains("domain=root;code=generator-closed;generator=root.pause-once")
    }));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-CLOSE-001"))
    );
}

#[test]
fn abandoned_custom_generator_handles_close_result() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "handle-close is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  resume-result is yield initial\n  resume-result\n    Error problem then ()\n    Ok resumed then ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is handle-close initial\n  ()\nabandon \"T\"\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.close.bound"))
    );
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
}

#[test]
fn custom_generator_matches_qualified_close_code() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "handle-code is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  result is yield initial\n  result\n    Error ( code is lang generator generator-closed ) then ()\n    Error problem then ()\n    Ok resumed then ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is handle-code initial\n  ()\nabandon \"T\"\n",
            &mut trace,
        )
        .unwrap();
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
}

#[test]
fn function_transfers_custom_generator_result_to_caller() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nmake is fn ( initial : Character ) -> Generator Character Unit Unit\n  pause-once initial\ngenerated is make \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"))
    );
    assert!(
        !trace.iter().any(|event| {
            event.contains("generator.closed") && event.contains("root.pause-once")
        })
    );
}

#[test]
fn function_parameter_receives_custom_generator_ownership() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nconsume is fn ( generated : Generator Character Unit Unit ) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is pause-once \"T\"\nconsume generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"))
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        1
    );
}

#[test]
fn function_closes_unconsumed_custom_generator_parameter() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nignore is fn ( generated : Generator Character Unit Unit ) -> Unit\n  ()\ngenerated is pause-once \"T\"\nignore generated\n",
            &mut trace,
        )
        .unwrap();
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-GENERATOR-CLOSE-001") && event.contains("root.pause-once")
    }));
    assert!(trace.iter().any(|event| {
        event.contains("domain=root;code=generator-closed;generator=root.pause-once")
    }));
}

#[test]
fn function_parameter_preserves_generator_final_character() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "yield-return is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Character\n\n  _ is yield initial\n  \"R\"\nconsume is fn ( generated : Generator Character Unit Character ) -> Character\n  generated foreach { character }\n    _ is String character\ngenerated is yield-return \"Y\"\nconsume generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("R".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn function_result_preserves_generator_final_character() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "yield-return is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Character\n\n  _ is yield initial\n  \"R\"\nmake is fn ( initial : Character ) -> Generator Character Unit Character\n  yield-return initial\ngenerated is make \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("R".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn custom_generator_accepts_string_initial_input() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "from-text is generator ( initial : String )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  initial-is-empty : Boolean is empty? initial\n  _ is yield \"T\"\n  ()\ngenerated is from-text \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.empty?(String)"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.suspended"))
    );
}

#[test]
fn custom_generator_yields_strings() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "texts is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  _ is yield \"\"\n  ()\ngenerated is texts \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        2
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator String Unit Unit"))
    );
}

#[test]
fn custom_generator_returns_distinct_string() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "text-result is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  \"done\"\ngenerated is text-result \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator String Unit String"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn custom_generator_returns_explicitly_before_yielding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "done is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  return \"done\"\ngenerated is done \"unused\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-EXPLICIT-RETURN-001"))
    );
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("generator.yielded"))
    );
}

#[test]
fn custom_generator_returns_explicitly_after_resuming() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "finish is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  return \"done\"\ngenerated is finish \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let returned = trace
        .iter()
        .position(|event| event.contains("generator.return.explicit"))
        .unwrap();
    assert!(resumed < returned);
}

#[test]
fn custom_generator_transfers_boolean_values() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "invert is generator ( initial : Boolean )\n  yields Boolean\n  resumes Unit\n  -> Boolean\n\n  _ is yield initial\n  not initial\ngenerated is invert true\ngenerated foreach { value }\n  _ is not value\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(false));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Boolean Unit Boolean"))
    );
}

#[test]
fn custom_generator_preserves_int_values() {
    let value = Session::new().evaluate("next is generator ( initial : Int )\n  yields Int\n  resumes Unit\n  -> Int\n\n  _ is yield initial\n  initial + 1\ngenerated is next 999999999999999999999999999999\ngenerated foreach { value }\n  _ is value + 1\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "1000000000000000000000000000000");
}

#[test]
fn custom_generator_preserves_rational_values() {
    let value = Session::new().evaluate("next is generator ( initial : Rational )\n  yields Rational\n  resumes Unit\n  -> Rational\n\n  _ is yield initial\n  initial + (Rational (1, 3))\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (1, 3))\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "Rational ( 2, 3 )");
}

#[test]
fn custom_generator_transfers_unit_values() {
    let mut trace = Vec::new();
    let value = Session::new().evaluate("pulse is generator ( initial : Unit )\n  yields Unit\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\ngenerated is pulse ()\ngenerated foreach { signal }\n  signal\n", &mut trace).unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Unit Unit Unit"))
    );
}

#[test]
fn custom_generator_preserves_optional_values() {
    let mut trace = Vec::new();
    let value = Session::new().evaluate("optional is generator ( initial : Optional Int )\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n\n  _ is yield initial\n  None Int\ngenerated is optional (Some 7)\ngenerated foreach { candidate }\n  _ is candidate = (Some 7)\n", &mut trace).unwrap();
    assert_eq!(value.to_string(), "None");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Optional Int Unit Optional Int"))
    );
}

#[test]
fn custom_generator_preserves_range_values() {
    let value = Session::new().evaluate("narrow is generator ( initial : Range Int )\n  yields Range Int\n  resumes Unit\n  -> Range Int\n\n  _ is yield initial\n  initial and (5 .. 15)\ngenerated is narrow (0 .. 10)\ngenerated foreach { interval }\n  _ is 5 in interval\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "5 .. 10");
}

#[test]
fn custom_generator_preserves_nat_constraint() {
    let value = Session::new().evaluate("next is generator ( initial : Nat )\n  yields Nat\n  resumes Unit\n  -> Nat\n\n  _ is yield initial\n  initial + 1\ngenerated is next (Nat 7)\ngenerated foreach { value }\n  _ is value + 1\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "8");
}

#[test]
fn custom_generator_preserves_enum_identity() {
    let mut trace = Vec::new();
    let value = Session::new().evaluate("Choice is Enum ( First, Second )\nchoose is generator ( initial : Choice )\n  yields Choice\n  resumes Unit\n  -> Choice\n\n  _ is yield initial\n  Second\ngenerated is choose First\ngenerated foreach { choice }\n  _ is choice = First\n", &mut trace).unwrap();
    assert_eq!(value.to_string(), "Second");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Choice Unit Choice"))
    );
}

#[test]
fn custom_generator_preserves_product_values() {
    let value = Session::new().evaluate("pair is generator ( initial : (Int, String) )\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"item\")\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
}

#[test]
fn custom_generator_returns_structured_result_error() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/custom-generator-result-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert!(matches!(value, Value::Error { ref code, .. } if code == "division-by-zero"));
}

#[test]
fn custom_generator_preserves_comparison_identity() {
    let value = Session::new().evaluate("order is generator ( initial : Comparison )\n  yields Comparison\n  resumes Unit\n  -> Comparison\n\n  _ is yield initial\n  3 <=> 2\ngenerated is order (1 <=> 2)\ngenerated foreach { comparison }\n  _ is comparison = (1 <=> 2)\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "Greater");
}

#[test]
fn custom_generator_preserves_nested_optional_product() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/custom-generator-nested-optional-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "Some (8, \"done\")");
}

#[test]
fn custom_generator_preserves_nested_result_product() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/custom-generator-nested-result-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
}

#[test]
fn custom_generator_preserves_nested_absent_optional() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/custom-generator-nested-none-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert!(
        matches!(value, Value::Optional { ref payload_classifier, payload: None } if payload_classifier == "(Int, String)")
    );
}

#[test]
fn custom_generators_preserve_recursive_nominal_classifiers() {
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../examples/interpreter/custom-generator-recursive-nominal-values.t"
            ),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Some Second, Second)");
}

#[test]
fn custom_generator_selects_final_decision_after_resuming() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/custom-generator-final-decision.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("accepted".into()));
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let selected = trace
        .iter()
        .position(|event| event.contains("decision.rule.selected"))
        .unwrap();
    let returned = trace
        .iter()
        .position(|event| event.contains("generator.returned"))
        .unwrap();
    assert!(resumed < selected && selected < returned);
}

#[test]
fn generator_return_mismatch_reports_expected_and_found_classifiers() {
    let error = Session::new()
        .evaluate(
            "invalid is generator ( initial : Boolean )\n  yields Boolean\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  42\ngenerated is invalid true\ngenerated foreach { value }\n  _ is not value\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-GENERATOR-RETURN-TYPE");
    assert!(error.message.contains("returned `Int`"));
    assert!(error.message.contains("requires `String`"));
    assert!(error.help.as_deref().unwrap().contains("produce `String`"));
}

#[test]
fn custom_generator_retains_local_function_across_resumption() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/custom-generator-local-function.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("accepted".into()));
    let declared_enum = trace
        .iter()
        .position(|event| event.contains("enum.declared") && event.contains("Choice"))
        .unwrap();
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let called = trace
        .iter()
        .rposition(|event| event.contains("function.entered"))
        .unwrap();
    assert!(declared_enum < resumed && resumed < called);
}

#[test]
fn custom_generator_restores_local_declarations_during_close() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/custom-generator-local-close-handler.t"),
            &mut trace,
        )
        .unwrap();
    let close_bound = trace
        .iter()
        .position(|event| event.contains("generator.close.bound"))
        .unwrap();
    let entered = trace
        .iter()
        .rposition(|event| event.contains("function.entered"))
        .unwrap();
    let closed = trace
        .iter()
        .position(|event| event.contains("generator.closed"))
        .unwrap();
    assert!(close_bound < entered && entered < closed);
}

#[test]
fn custom_generator_selects_unary_and_binary_overloads() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/custom-generator-overloads.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"unary\", \"binary\")");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.selected"))
            .count(),
        2
    );
    assert!(trace.iter().any(|event| event.contains("Int, String")));
}

#[test]
fn duplicate_generator_input_signature_is_rejected() {
    let error = Session::new()
        .evaluate(
            "same is generator ( value : Int )\n  yields Int\n  resumes Unit\n  -> Unit\n\n  _ is yield value\n  ()\nsame is generator ( other : Int )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield \"duplicate\"\n  \"duplicate\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-DUPLICATE-GENERATOR-OVERLOAD");
}

#[test]
fn generator_overload_error_lists_available_inputs() {
    let error = Session::new()
        .evaluate(
            "select is generator ( value : Int )\n  yields Int\n  resumes Unit\n  -> Unit\n\n  _ is yield value\n  ()\ngenerated is select true\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-GENERATOR");
    assert!(error.message.contains("Boolean"));
    assert!(error.help.as_deref().unwrap().contains("Int"));
}

#[test]
fn foreach_result_binding_is_available_to_later_statements() {
    let value = Session::new()
        .evaluate(
            "once is generator ( initial : Int )\n  yields Int\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  \"done\"\ngenerated is once 7\nresult is generated foreach { value }\n  _ is value + 1\nempty? result\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(false));
}

#[test]
fn classified_foreach_result_reports_mismatch() {
    let error = Session::new()
        .evaluate(
            "once is generator ( initial : Int )\n  yields Int\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  \"done\"\ngenerated is once 7\nresult : Int is generated foreach { value }\n  _ is value + 1\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-FOREACH-RESULT-CLASSIFIER");
    assert!(error.message.contains("returned `String`"));
    assert!(error.message.contains("requires `Int`"));
}

#[test]
fn custom_generator_crosses_generic_function_boundaries() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../examples/interpreter/custom-generator-generic-function-boundaries.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.function.returned"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.parameter.transferred"))
    );
}

#[test]
fn compound_generator_crosses_function_boundaries() {
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../examples/interpreter/custom-generator-compound-function-boundaries.t"
            ),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
}

#[test]
fn nested_generator_crosses_function_boundaries() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../examples/interpreter/custom-generator-nested-function-boundaries.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
    let classifier = "Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)";
    assert!(trace.iter().any(|event| {
        event.contains("generator.function.returned") && event.contains(classifier)
    }));
    assert!(trace.iter().any(|event| {
        event.contains("generator.parameter.transferred") && event.contains(classifier)
    }));
}

#[test]
fn list_generator_crosses_function_boundaries() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/custom-generator-list-values.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "Entry ( 7, Entry ( 9, Empty ) )");
    assert!(trace.iter().any(|event| {
        event.contains("generator.function.returned")
            && event.contains("Generator List Int Unit List Int")
    }));
    assert!(trace.iter().any(|event| {
        event.contains("generator.parameter.transferred")
            && event.contains("Generator List Int Unit List Int")
    }));
}

#[test]
fn custom_generator_executes_discard_after_resume() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "inspect-between is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  _ is empty? initial\n  _ is yield \"\"\n  ()\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let tested = trace
        .iter()
        .enumerate()
        .skip(resumed + 1)
        .find_map(|(index, event)| event.contains("string.empty.tested").then_some(index))
        .unwrap();
    let suspended = trace
        .iter()
        .rposition(|event| event.contains("generator.suspended"))
        .unwrap();
    assert!(resumed < tested && tested < suspended);
}

#[test]
fn custom_generator_cannot_yield_after_close_result() {
    let error = Session::new()
        .evaluate(
            "invalid-close is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  resume-result is yield initial\n  _ is yield initial\n  ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is invalid-close initial\n  ()\nabandon \"T\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-GENERATOR-YIELD-AFTER-CLOSE");
    assert!(error.message.contains("cannot yield again"));
}

#[test]
fn rational_ranges_use_exact_canonical_conversion() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("interval is 0 .. 2.5\n(interval, 1.5 in interval, interval contains 2, 3 in interval)\n", &mut trace)
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 0, 1 ) .. Rational ( 5, 2 ), true, true, false)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Int->Rational:left"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Int->Rational:membership"))
    );
}

#[test]
fn lists_construct_compare_and_decompose() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/lists.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Some 6, Some 6, Some Entry ( 7, Entry ( 8, Entry ( 9, Entry ( 10, Empty ) ) ) ), None, None, 5, false, true, true, Some (6, Entry ( 7, Entry ( 8, Entry ( 9, Entry ( 10, Empty ) ) ) )), Some 10, true)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("list.entry.constructed"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("list.entry.decomposed"))
    );
    assert!(trace.iter().any(|event| event.contains("equality.list")));
    for event in [
        "list.prepended",
        "list.appended",
        "list.concatenated",
        "list.entry-count",
        "list.empty.tested",
        "list.empty.constructed",
        "list.singleton.constructed",
        "list.uncons",
        "list.first",
        "list.rest",
        "list.reversed",
    ] {
        assert!(trace.iter().any(|record| record.contains(event)), "{event}");
    }
}

#[test]
fn first_and_rest_reject_non_lists() {
    for operation in ["first", "rest"] {
        let error = Session::new()
            .evaluate(&format!("{operation} 7\n"), &mut Vec::new())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
        assert!(error.message.contains("requires a List"));
    }
}

#[test]
fn recursive_list_classifiers_cross_function_boundaries() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/nested-lists.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Some Entry ( (7, \"seven\"), Empty ), 1, true)"
    );
}

#[test]
fn list_containment_distinguishes_entry_sequence_and_subsequence() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/list-containment.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false, true, true, false, false)");
    for rule in [
        "TOPAL-LIST-CONTAINS-ENTRY-001",
        "TOPAL-LIST-CONTAINS-SEQUENCE-001",
        "TOPAL-LIST-CONTAINS-SUBSEQUENCE-001",
    ] {
        assert!(trace.iter().any(|event| event.contains(rule)), "{rule}");
    }
}

#[test]
fn list_containment_requires_compatible_classifiers() {
    let error = Session::new()
        .evaluate(
            "numbers : List Int is one 1\ntexts : List String is one \"one\"\nnumbers contains-sequence texts\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-CONTAINMENT-CLASSIFIER");
    assert!(error.message.contains("List String"));
}

#[test]
fn list_value_removal_preserves_retained_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/interpreter/list-removal.t"),
            &mut trace,
        )
        .unwrap();
    assert!(
        value
            .to_string()
            .contains("Entry ( 1, Entry ( 3, Entry ( 2")
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-LIST-REMOVE-FIRST-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-LIST-REMOVE-ALL-001"))
    );
}

#[test]
fn list_value_removal_rejects_wrong_classifier() {
    let error = Session::new()
        .evaluate(
            "values : List Int is one 1\nvalues remove-first \"1\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-REMOVAL-CLASSIFIER");
}

#[test]
fn uncons_is_total_for_empty_lists_and_rejects_other_values() {
    let value = Session::new()
        .evaluate("uncons (empty List Int)\n", &mut Vec::new())
        .unwrap();
    assert_eq!(value.to_string(), "None");

    let error = Session::new()
        .evaluate("uncons 7\n", &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    assert!(error.message.contains("requires a List"));
}

#[test]
fn explicit_empty_and_singleton_lists_preserve_numeric_one() {
    let value = Session::new()
        .evaluate(
            "empty-values is empty List String\nsingleton is one \"Topal\"\n(empty-values, singleton, one Int)\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Empty, Entry ( \"Topal\", Empty ), 1)");
}

#[test]
fn list_operations_reject_incompatible_classifiers() {
    let entry = Session::new()
        .evaluate(
            "values : List Int is Empty\nvalues append \"bad\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(entry.code, "E-LIST-ENTRY-CLASSIFIER");
    assert!(entry.message.contains("requires `Int`"));

    let concat = Session::new()
        .evaluate(
            "numbers : List Int is Empty\ntexts : List String is Empty\nnumbers concat texts\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(concat.code, "E-LIST-CONCAT-CLASSIFIER");
    assert!(concat.message.contains("List String"));
}

#[test]
fn list_entry_classifier_mismatch_is_precise() {
    let error = Session::new()
        .evaluate(
            "values : List Int is Entry ( \"bad\", Empty )\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-ENTRY-CLASSIFIER");
    assert!(error.message.contains("requires `Int`"));
    assert!(error.help.unwrap().contains("use a `Int` value"));
}

#[test]
fn list_remainder_must_be_a_list() {
    let error = Session::new()
        .evaluate("values : List Int is Entry ( 7, 8 )\n", &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-REMAINDER");
    assert!(error.help.unwrap().contains("Empty"));
}
