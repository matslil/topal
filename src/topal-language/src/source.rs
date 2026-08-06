use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write as _};

use num_bigint::BigInt;
use num_rational::BigRational;
use topal_source::{
    SourceText, Span, canonically_equal, case_fold, character_at, character_count, lowercase,
    normalize_nfc, normalize_nfd, uppercase,
};
use topal_syntax::{
    CallableKind, DecisionMatcher, Expression, FunctionParameter, Statement, lex, parse,
};

use crate::{ExecutionSnapshot, TraceEvent, TraceSink};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
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
    String(String),
    Tuple(Vec<Self>),
    Record(Vec<(String, Self)>),
    Enum {
        type_name: String,
        alternative: String,
    },
    ErrorDomain(String),
    Error {
        domain: String,
        code: String,
        line: usize,
        column: usize,
    },
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
            Self::ErrorDomain(domain) => formatter.write_str(domain),
            Self::Error { domain, code, .. } => {
                write!(formatter, "Error ( domain is {domain}, code is {code} )")
            }
            Self::Unit => formatter.write_str("()"),
        }
    }
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
pub struct Session {
    bindings: BTreeMap<String, Value>,
    functions: BTreeMap<String, Vec<UserFunction>>,
    declared_names: BTreeSet<String>,
    local_function_names: BTreeSet<String>,
    enum_types: BTreeMap<String, BTreeSet<String>>,
    call_stack: Vec<ActiveCall>,
    static_context: bool,
}

#[derive(Clone)]
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

#[derive(Clone)]
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

    /// Report whether a complete function declaration should await a dedented
    /// line before an interactive session submits it.
    #[must_use]
    pub fn awaits_dedent(input: &str) -> bool {
        let Ok(source) = SourceText::new(input) else {
            return false;
        };
        let parsed = parse(&source, &lex(&source));
        parsed.diagnostics.is_empty()
            && matches!(parsed.statements.as_slice(), [Statement::Function { .. }])
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
            declared_names: bindings.keys().cloned().collect(),
            local_function_names: BTreeSet::new(),
            enum_types: BTreeMap::new(),
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
                let labeled = fields.iter().filter(|field| field.label.is_some()).count();
                if labeled != 0 && labeled != fields.len() {
                    return Err(diagnostic(
                        source,
                        "E-UNSUPPORTED-MIXED-PRODUCT",
                        *span,
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
                if !enum_matchers.is_empty()
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
                let decision_rule = if has_optional_matchers {
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
                            if namespace != "lang"
                                || vocabulary != "arithmetic"
                                || !is_arithmetic_error_code(code)
                            {
                                return Err(diagnostic(
                                    source,
                                    "E-UNKNOWN-ERROR-CODE",
                                    code_span,
                                    "the implemented error-code pattern requires a code published by `lang arithmetic`",
                                ));
                            }
                            matches!(&subject, Value::Error { code: subject_code, .. } if subject_code == code)
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
                        declared_names: self.declared_names.clone(),
                        local_function_names: self.local_function_names.clone(),
                        enum_types: self.enum_types.clone(),
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
                } else {
                    self.evaluate_expression(source, &selected_rule.action, trace)
                }
            }
            Expression::Integer(span) => evaluate_integer_literal(source, *span, trace),
            Expression::Rational(span) => evaluate_rational_literal(source, *span, trace),
            Expression::String(span) => evaluate_string_literal(source, *span, trace),
            Expression::Identifier(span) => {
                let name = source.slice(*span);
                let value = self.bindings.get(name).cloned().ok_or_else(|| {
                    let error = diagnostic(source, "E-UNBOUND-NAME", *span, "name is not bound");
                    closest_name(name, self.bindings.keys())
                        .or_else(|| closest_root_operation(name))
                        .map_or(error.clone(), |candidate| {
                            error.with_help(format!("did you mean `{candidate}`?"))
                        })
                })?;
                trace.record(TraceEvent {
                    event: "binding.resolved",
                    rule: "TOPAL-SYN-BIND-001",
                    detail: name,
                });
                Ok(value)
            }
            Expression::Discard(span) => Err(diagnostic(
                source,
                "E-DISCARD-VALUE",
                *span,
                "discard is valid only in a declaration or pattern",
            )),
            Expression::Callable { span, .. } => Err(diagnostic(
                source,
                "E-UNSUPPORTED-CALLABLE-VALUE",
                *span,
                "callable values are not yet executable in isolation",
            )),
            Expression::Application { items, span } => {
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
                if let [Expression::Identifier(constructor), payload] = items.as_slice()
                    && source.slice(*constructor) == "Some"
                {
                    let value = self.evaluate_expression(source, payload, trace)?;
                    let payload_classifier = value_classifier(&value).to_owned();
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
                if let [
                    Expression::Identifier(constructor),
                    Expression::Identifier(domain),
                ] = items.as_slice()
                    && source.slice(*constructor) == "None"
                {
                    let payload_classifier = source.slice(*domain).to_owned();
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
                {
                    let (value, selection, classifier) = match source.slice(*domain) {
                        "Int" => (Value::Int(BigInt::from(1)), "root.one(Int)", "Int"),
                        "Nat" => (Value::Int(BigInt::from(1)), "root.one(Nat)", "Nat"),
                        "Rational" => (
                            Value::Rational(BigRational::from_integer(BigInt::from(1))),
                            "root.one(Rational)",
                            "Rational",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                *domain,
                                "one requires a supported numeric type",
                            ));
                        }
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
                        declared_names: BTreeSet::new(),
                        local_function_names: BTreeSet::new(),
                        enum_types: self.enum_types.clone(),
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
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "empty? requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.empty?(String)",
                    });
                    let value = Value::Boolean(text.is_empty());
                    trace.record(TraceEvent {
                        event: "string.empty.tested",
                        rule: "TOPAL-STRING-EMPTY-PREDICATE-001",
                        detail: if text.is_empty() { "true" } else { "false" },
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(source.slice(*name), "character-count" | "entry-count")
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            format!("{operation} requires a String operand"),
                        ));
                    };
                    let selection = format!("root.{operation}(String)");
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: &selection,
                    });
                    let count = character_count(&text);
                    let detail = format!("characters={count}");
                    trace.record(TraceEvent {
                        event: if operation == "entry-count" {
                            "string.entry-count"
                        } else {
                            "string.character-count"
                        },
                        rule: if operation == "entry-count" {
                            "TOPAL-STRING-ENTRY-COUNT-001"
                        } else {
                            "TOPAL-STRING-CHARACTER-COUNT-001"
                        },
                        detail: &detail,
                    });
                    let value = Value::Int(BigInt::from(count));
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
                        && let Value::Error { domain, code, .. } = &result
                    {
                        let label = source.slice(*label_span);
                        let selected = match label {
                            "code" => Value::Enum {
                                type_name: "lang arithmetic ArithmeticErrorCode".into(),
                                alternative: code.clone(),
                            },
                            "domain" => Value::ErrorDomain(domain.clone()),
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
        if !supported_value_classifier(result_text)
            && !session.enum_types.contains_key(result_text)
            && result_success_classifier(result_text).is_none()
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
                if !supported_value_classifier(classifier)
                    && !session.enum_types.contains_key(classifier)
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

    /// Execute one source statement.
    ///
    /// # Errors
    ///
    /// Returns a name-resolution or evaluation diagnostic at the failing step.
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
            Statement::Discard { span, value } => {
                session.evaluate_expression(&self.source, value, trace)?;
                trace.record(TraceEvent {
                    event: "binding.discarded",
                    rule: "TOPAL-SYN-BIND-001",
                    detail: "_",
                });
                (Value::Unit, cover(*span, value.span()))
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
                trace.record(TraceEvent {
                    event: "function.return.explicit",
                    rule: "TOPAL-FUNCTION-RETURN-001",
                    detail: value_classifier(&value),
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
                (
                    evaluate_expression_with_optional_context(
                        &self.source,
                        session,
                        expression,
                        self.return_classifier.as_deref(),
                        trace,
                    )?,
                    expression.span(),
                )
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
        let mut evaluated =
            evaluate_binding_initializer(&self.source, session, initializer, classifier, trace)?;
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
        Statement::Function { span, .. } => *span,
        Statement::Discard { span, value } => cover(*span, value.span()),
        Statement::Return { keyword, value } => cover(*keyword, value.span()),
        Statement::Expression(expression) => expression.span(),
    }
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

fn value_has_classifier(value: &Value, classifier: &str) -> bool {
    if let Value::Optional {
        payload_classifier, ..
    } = value
        && let Some(expected) = optional_payload_classifier(classifier)
    {
        return payload_classifier == expected;
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
        | (Value::String(_), "String")
        | (Value::Unit, "Unit") => true,
        (Value::String(value), "Character") => character_count(value) == 1,
        (Value::Int(value), "Nat") => value >= &BigInt::from(0),
        (Value::Enum { type_name, .. }, classifier) => type_name == classifier,
        _ => false,
    }
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

fn tuple_classifiers(classifier: &str) -> Option<Vec<&str>> {
    let contents = classifier.trim().strip_prefix('(')?.strip_suffix(')')?;
    let classifiers = contents.split(',').map(str::trim).collect::<Vec<_>>();
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

fn supported_value_classifier(classifier: &str) -> bool {
    matches!(
        classifier,
        "Boolean"
            | "Character"
            | "Comparison"
            | "Int"
            | "Nat"
            | "Range Int"
            | "Range Rational"
            | "Rational"
            | "String"
            | "Unit"
    ) || optional_payload_classifier(classifier).is_some_and(supported_value_classifier)
        || tuple_classifiers(classifier)
            .is_some_and(|items| items.into_iter().all(supported_value_classifier))
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
    trace.record(TraceEvent {
        event: "evaluation.result",
        rule: "TOPAL-SYN-GRAMMAR-001",
        detail: value_classifier(value),
    });
}

const fn value_classifier(value: &Value) -> &'static str {
    match value {
        Value::Boolean(_) => "Boolean",
        Value::Int(_) => "Int",
        Value::Rational(_) => "Rational",
        Value::IntRange { .. } | Value::RationalRange { .. } => "Range",
        Value::Optional { .. } => "Optional",
        Value::String(_) => "String",
        Value::Tuple(_) => "Tuple",
        Value::Record(_) => "Record",
        Value::Enum { .. } => "Enum",
        Value::ErrorDomain(_) => "ErrorDomain",
        Value::Error { .. } => "Error",
        Value::Unit => "Unit",
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

fn apply_binary(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    spans: (Span, Span, Span),
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (span, left_span, right_span) = spans;
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

fn values_equal(left: Value, right: Value, trace: &mut impl TraceSink) -> Option<bool> {
    match (left, right) {
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
        (Value::String(left), Value::String(right)) => Some(left == right),
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
        Value::Boolean(_)
        | Value::IntRange { .. }
        | Value::RationalRange { .. }
        | Value::Optional { .. }
        | Value::String(_)
        | Value::Tuple(_)
        | Value::Record(_)
        | Value::Enum { .. }
        | Value::ErrorDomain(_)
        | Value::Error { .. }
        | Value::Unit => Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "prefix - requires an exact numeric operand",
        )),
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

const ROOT_OPERATIONS: [&str; 15] = [
    "absolute",
    "byte-count",
    "case-fold",
    "canonically-equals",
    "character-count",
    "concat",
    "empty",
    "entry-count",
    "lower",
    "normalize",
    "upper",
    "not",
    "negate",
    "one",
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
