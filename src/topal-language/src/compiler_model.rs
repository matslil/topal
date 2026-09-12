//! Shared checked model consumed by the native compiler.
//!
//! This first model intentionally covers a bounded, explicit language slice.
//! Extending it is how later compiler increments acquire semantics; the LLVM
//! backend never reinterprets the source syntax itself.

use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigInt;
use num_rational::BigRational;
use topal_semantics::LanguageVersion;
use topal_source::{Diagnostic, SourceText, Span};
use topal_syntax::{
    CallableKind, DecisionMatcher, Expression, FunctionClauses, FunctionParameter, Statement, lex,
    parse,
};

use crate::source::{parse_integer, parse_rational, parse_string};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerType {
    Unit,
    Boolean,
    Int,
    Rational,
    Comparison,
    Range(Box<Self>),
    String,
    Tuple(Vec<Self>),
}

impl CompilerType {
    #[must_use]
    pub const fn machine_scalar(&self) -> bool {
        matches!(
            self,
            Self::Unit
                | Self::Boolean
                | Self::Int
                | Self::Rational
                | Self::Comparison
                | Self::Range(_)
        )
    }

    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Unit => "Unit".into(),
            Self::Boolean => "Boolean".into(),
            Self::Int => "Int".into(),
            Self::Rational => "Rational".into(),
            Self::Comparison => "Comparison".into(),
            Self::Range(endpoint) => format!("Range {}", endpoint.name()),
            Self::String => "String".into(),
            Self::Tuple(fields) => format!(
                "({})",
                fields.iter().map(Self::name).collect::<Vec<_>>().join(", ")
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntRange {
    pub lower: BigInt,
    pub upper: BigInt,
}

impl IntRange {
    fn exact(value: BigInt) -> Self {
        Self {
            lower: value.clone(),
            upper: value,
        }
    }

    fn union(left: &Self, right: &Self) -> Self {
        Self {
            lower: left.lower.clone().min(right.lower.clone()),
            upper: left.upper.clone().max(right.upper.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerBinary {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    QuotientModulo,
    Power,
    Compare,
    Range,
    RangeOpen,
    RangeInclusive,
    RangeOpenInclusive,
    In,
    Contains,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Xor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerExpression {
    pub kind: CompilerExpressionKind,
    pub value_type: CompilerType,
    pub int_range: Option<IntRange>,
    pub rational_value: Option<BigRational>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerComparisonRule {
    pub operation: CompilerBinary,
    pub operand: CompilerExpression,
    pub action: CompilerExpression,
    pub subject_to_rational: bool,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerExpressionKind {
    Unit,
    Boolean(bool),
    Int(BigInt),
    Rational(BigRational),
    String(String),
    Tuple(Vec<CompilerExpression>),
    Local(String),
    Negate(Box<CompilerExpression>),
    Absolute(Box<CompilerExpression>),
    IntToRational(Box<CompilerExpression>),
    RationalConstruct {
        numerator: Box<CompilerExpression>,
        denominator: Box<CompilerExpression>,
    },
    RangeLower(Box<CompilerExpression>),
    RangeUpper(Box<CompilerExpression>),
    RangeLowerInclusive(Box<CompilerExpression>),
    RangeUpperInclusive(Box<CompilerExpression>),
    RangeEmpty(Box<CompilerExpression>),
    Not(Box<CompilerExpression>),
    Binary {
        operation: CompilerBinary,
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
    },
    Call {
        symbol: String,
        arguments: Vec<CompilerExpression>,
    },
    BooleanDecision {
        subject: Box<CompilerExpression>,
        when_true: Box<CompilerExpression>,
        when_false: Box<CompilerExpression>,
    },
    OrderedComparisonDecision {
        subject: Box<CompilerExpression>,
        rules: Vec<CompilerComparisonRule>,
        otherwise: Box<CompilerExpression>,
    },
    ComparisonValueDecision {
        subject: Box<CompilerExpression>,
        when_less: Box<CompilerExpression>,
        when_equal: Box<CompilerExpression>,
        when_greater: Box<CompilerExpression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerBinding {
    pub name: String,
    pub value: CompilerExpression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerBlock {
    pub statements: Vec<CompilerStatement>,
    pub result: CompilerExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerStatement {
    Binding(CompilerBinding),
    Discard(CompilerExpression),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerParameter {
    pub name: String,
    pub value_type: CompilerType,
    pub int_range: Option<IntRange>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerFunction {
    pub source_name: String,
    pub symbol: String,
    pub parameters: Vec<CompilerParameter>,
    pub result_type: CompilerType,
    pub body: CompilerBlock,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerProgram {
    pub source: SourceText,
    pub language_version: LanguageVersion,
    pub main: CompilerBlock,
    /// Instances are in callee-before-caller order.
    pub functions: Vec<CompilerFunction>,
}

#[derive(Clone)]
struct FunctionSource {
    name: Span,
    parameters: Vec<FunctionParameter>,
    result: Span,
    body: Vec<Statement>,
    span: Span,
}

#[derive(Clone)]
struct BindingFacts {
    value_type: CompilerType,
    int_range: Option<IntRange>,
    rational_value: Option<BigRational>,
}

struct Analyzer {
    source: SourceText,
    functions: BTreeMap<String, FunctionSource>,
    instances: Vec<CompilerFunction>,
    active_calls: BTreeSet<String>,
    next_instance: usize,
}

/// Analyze the currently implemented native-compiler subset.
///
/// # Errors
///
/// Returns a shared source diagnostic for invalid or not-yet-supported input.
pub fn analyze_for_compiler(text: &str) -> Result<CompilerProgram, Diagnostic> {
    let source = SourceText::new(text).map_err(|error| {
        Diagnostic::error(error.code, 1, 1, error.message).with_source_span(error.span)
    })?;
    let parsed = parse(&source, &lex(&source));
    if let Some(error) = parsed.diagnostics.first() {
        return Err(source_diagnostic(
            &source,
            error.code,
            error.span,
            error.message.clone(),
        ));
    }
    let Some(Statement::LanguageSelection {
        version, features, ..
    }) = parsed.statements.first()
    else {
        return Err(source_diagnostic(
            &source,
            "E-LANGUAGE-CONTEXT",
            Span::new(0, 0),
            "a source file begins with `use language ( version is v0.1 )`",
        ));
    };
    let language_version = source
        .slice(*version)
        .parse()
        .map_err(|message| source_diagnostic(&source, "E-LANGUAGE-VERSION", *version, message))?;
    if language_version != LanguageVersion::DESIGN_0 || !features.is_empty() {
        return Err(source_diagnostic(
            &source,
            "E-COMPILER-UNSUPPORTED",
            *version,
            "the native compiler increment supports language version v0.1 without optional features",
        ));
    }

    let mut functions = BTreeMap::new();
    collect_functions(&source, &parsed.statements, &mut functions)?;
    let mut analyzer = Analyzer {
        source: source.clone(),
        functions,
        instances: Vec::new(),
        active_calls: BTreeSet::new(),
        next_instance: 0,
    };
    let mut environment = BTreeMap::new();
    let main = analyzer.analyze_block(&parsed.statements, &mut environment, BlockKind::TopLevel)?;
    Ok(CompilerProgram {
        source,
        language_version,
        main,
        functions: analyzer.instances,
    })
}

fn collect_functions(
    source: &SourceText,
    statements: &[Statement],
    functions: &mut BTreeMap<String, FunctionSource>,
) -> Result<(), Diagnostic> {
    for statement in statements {
        let declaration = match statement {
            Statement::Function {
                name,
                parameters,
                result,
                body,
                span,
                is_static: false,
                effect_bound: None,
                clauses,
            } if **clauses == FunctionClauses::default() => Some(FunctionSource {
                name: *name,
                parameters: parameters.clone(),
                result: *result,
                body: body.clone(),
                span: *span,
            }),
            Statement::Published { declaration, .. } => {
                if let Statement::Function {
                    name,
                    parameters,
                    result,
                    body,
                    span,
                    is_static: false,
                    effect_bound: None,
                    clauses,
                } = declaration.as_ref()
                    && **clauses == FunctionClauses::default()
                {
                    Some(FunctionSource {
                        name: *name,
                        parameters: parameters.clone(),
                        result: *result,
                        body: body.clone(),
                        span: *span,
                    })
                } else {
                    return Err(unsupported(
                        source,
                        statement_span(statement),
                        "published declaration",
                    ));
                }
            }
            Statement::Function { .. } => {
                return Err(unsupported(
                    source,
                    statement_span(statement),
                    "static, constrained, or effectful function",
                ));
            }
            _ => None,
        };
        if let Some(function) = declaration {
            let name = source.slice(function.name).to_owned();
            if functions.insert(name.clone(), function).is_some() {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-FUNCTION",
                    statement_span(statement),
                    format!("function `{name}` is already declared"),
                ));
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BlockKind {
    TopLevel,
    Function,
}

impl Analyzer {
    #[allow(clippy::too_many_lines)] // Exhaustive statement admission keeps the subset boundary visible.
    fn analyze_block(
        &mut self,
        statements: &[Statement],
        environment: &mut BTreeMap<String, BindingFacts>,
        kind: BlockKind,
    ) -> Result<CompilerBlock, Diagnostic> {
        let mut lowered = Vec::new();
        let mut result = None;
        let executable = statements
            .iter()
            .filter(|statement| !is_declaration(statement))
            .collect::<Vec<_>>();

        for (index, statement) in executable.iter().enumerate() {
            let last = index + 1 == executable.len();
            match statement {
                Statement::Binding {
                    name,
                    classifier,
                    value,
                } => {
                    let name_text = self.source.slice(*name).to_owned();
                    if environment.contains_key(&name_text)
                        || self.functions.contains_key(&name_text)
                    {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-DUPLICATE-BINDING",
                            *name,
                            format!("`{name_text}` is already declared in this scope"),
                        ));
                    }
                    let value = self.analyze_expression(value, environment)?;
                    if let Some(classifier) = classifier {
                        let expected = parse_classifier(&self.source, *classifier)?;
                        require_same_type(&self.source, *classifier, &expected, &value.value_type)?;
                    }
                    environment.insert(
                        name_text.clone(),
                        BindingFacts {
                            value_type: value.value_type.clone(),
                            int_range: value.int_range.clone(),
                            rational_value: value.rational_value.clone(),
                        },
                    );
                    lowered.push(CompilerStatement::Binding(CompilerBinding {
                        name: name_text,
                        span: *name,
                        value,
                    }));
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Discard { value, .. } => {
                    lowered.push(CompilerStatement::Discard(
                        self.analyze_expression(value, environment)?,
                    ));
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Expression(expression) if last => {
                    result = Some(self.analyze_expression(expression, environment)?);
                }
                Statement::Expression(_) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-NONFINAL-VALUE",
                        statement_span(statement),
                        "a non-final value expression must be explicitly discarded or bound",
                    ));
                }
                Statement::Return { value, .. } if kind == BlockKind::Function && last => {
                    result = Some(self.analyze_expression(value, environment)?);
                }
                Statement::Return { .. } if kind == BlockKind::TopLevel => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-RETURN-OUTSIDE-FUNCTION",
                        statement_span(statement),
                        "`return` is available only inside a function",
                    ));
                }
                Statement::Return { .. } => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "non-final return",
                    ));
                }
                Statement::LibrarySelection { .. } => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "source library dependency",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "statement form",
                    ));
                }
            }
        }
        Ok(CompilerBlock {
            statements: lowered,
            result: result.unwrap_or_else(|| unit_expression(Span::new(0, 0))),
        })
    }

    #[allow(clippy::too_many_lines)] // Exhaustive expression admission keeps the subset boundary visible.
    fn analyze_expression(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let span = expression.span();
        match expression {
            Expression::Unit(_) => Ok(unit_expression(span)),
            Expression::Boolean(value) => Ok(CompilerExpression {
                kind: CompilerExpressionKind::Boolean(self.source.slice(*value) == "true"),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            }),
            Expression::Integer(value) => {
                let integer = parse_integer(self.source.slice(*value)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-NUMERIC-LITERAL",
                        *value,
                        "invalid integer literal",
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Int(integer.clone()),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(integer)),
                    rational_value: None,
                    span,
                })
            }
            Expression::Rational(value) => {
                let rational = parse_rational(self.source.slice(*value)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-NUMERIC-LITERAL",
                        *value,
                        "invalid rational literal",
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Rational(rational.clone()),
                    value_type: CompilerType::Rational,
                    int_range: None,
                    rational_value: Some(rational),
                    span,
                })
            }
            Expression::String(value) => {
                let value = parse_string(self.source.slice(*value)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-STRING-LITERAL",
                        *value,
                        "invalid string literal delimiter",
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::String(value.to_owned()),
                    value_type: CompilerType::String,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Product { fields, .. } => {
                if fields.iter().any(|field| field.label.is_some()) {
                    return Err(unsupported(&self.source, span, "labeled product"));
                }
                let values = fields
                    .iter()
                    .map(|field| self.analyze_expression(&field.value, environment))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(CompilerExpression {
                    value_type: CompilerType::Tuple(
                        values
                            .iter()
                            .map(|value| value.value_type.clone())
                            .collect(),
                    ),
                    kind: CompilerExpressionKind::Tuple(values),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name) => {
                let name_text = self.source.slice(*name);
                let facts = environment.get(name_text).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-UNBOUND-NAME",
                        *name,
                        format!("name `{name_text}` is not bound"),
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Local(name_text.to_owned()),
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    span,
                })
            }
            Expression::Application { items, .. } => {
                self.analyze_application(items, span, environment)
            }
            Expression::DecisionTable { subject, rules, .. } => {
                self.analyze_decision(subject, rules, span, environment)
            }
            _ => Err(unsupported(&self.source, span, "expression form")),
        }
    }

    #[allow(clippy::too_many_lines)] // Root operations are admitted explicitly and in source-selection order.
    fn analyze_application(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let [
            Expression::Identifier(operation),
            Expression::Identifier(domain),
        ] = items
            && matches!(self.source.slice(*operation), "zero" | "one")
        {
            let one = self.source.slice(*operation) == "one";
            return match self.source.slice(*domain) {
                "Int" | "Nat" => {
                    let value = BigInt::from(u8::from(one));
                    Ok(CompilerExpression {
                        kind: CompilerExpressionKind::Int(value.clone()),
                        value_type: CompilerType::Int,
                        int_range: Some(IntRange::exact(value)),
                        rational_value: None,
                        span,
                    })
                }
                "Rational" => {
                    let value = BigRational::from_integer(BigInt::from(u8::from(one)));
                    Ok(CompilerExpression {
                        kind: CompilerExpressionKind::Rational(value.clone()),
                        value_type: CompilerType::Rational,
                        int_range: None,
                        rational_value: Some(value),
                        span,
                    })
                }
                _ => Err(unsupported(
                    &self.source,
                    *domain,
                    "numeric identity domain",
                )),
            };
        }
        if let [Expression::Identifier(constructor), argument] = items
            && self.source.slice(*constructor) == "Rational"
        {
            return self.analyze_rational_constructor(argument, span, environment);
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "empty?"
                    | "range-lower"
                    | "range-upper"
                    | "range-lower-inclusive?"
                    | "range-upper-inclusive?"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_range_observation(&operation, operand, span, environment);
        }
        if let [
            Expression::Callable {
                kind: CallableKind::Minus,
                ..
            },
            operand,
        ] = items
        {
            let operand = self.analyze_expression(operand, environment)?;
            require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
            let range = (operand.value_type == CompilerType::Int)
                .then_some(operand.int_range.as_ref())
                .flatten()
                .map(|range| IntRange {
                    lower: -range.upper.clone(),
                    upper: -range.lower.clone(),
                });
            let rational_value = operand.rational_value.as_ref().map(|value| -value.clone());
            let value_type = operand.value_type.clone();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Negate(Box::new(operand)),
                value_type,
                int_range: range,
                rational_value,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(self.source.slice(*operation), "negate" | "absolute")
        {
            let negate = self.source.slice(*operation) == "negate";
            let operand = self.analyze_expression(operand, environment)?;
            require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
            let range = (operand.value_type == CompilerType::Int)
                .then_some(operand.int_range.as_ref())
                .flatten()
                .map(|range| {
                    if negate {
                        IntRange {
                            lower: -range.upper.clone(),
                            upper: -range.lower.clone(),
                        }
                    } else {
                        absolute_range(range)
                    }
                });
            let rational_value = operand.rational_value.as_ref().map(|value| {
                if negate {
                    -value.clone()
                } else {
                    rational_absolute(value)
                }
            });
            let value_type = operand.value_type.clone();
            return Ok(CompilerExpression {
                kind: if negate {
                    CompilerExpressionKind::Negate(Box::new(operand))
                } else {
                    CompilerExpressionKind::Absolute(Box::new(operand))
                },
                value_type,
                int_range: range,
                rational_value,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && self.source.slice(*operation) == "not"
        {
            let operand = self.analyze_expression(operand, environment)?;
            require_type(
                &self.source,
                operand.span,
                &CompilerType::Boolean,
                &operand.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Not(Box::new(operand)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [left, Expression::Callable { kind, .. }, right] = items {
            return self.analyze_symbolic_binary(*kind, left, right, span, environment);
        }
        if let [left, Expression::Identifier(operation), right] = items
            && matches!(
                self.source.slice(*operation),
                "and" | "or" | "xor" | "in" | "contains"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_identifier_binary(&operation, left, right, span, environment);
        }
        self.analyze_call(items, span, environment)
    }

    fn analyze_rational_constructor(
        &mut self,
        argument: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Expression::Product { fields, .. } = argument
            && fields.len() == 2
            && fields.iter().all(|field| field.label.is_none())
        {
            let numerator = self.analyze_expression(&fields[0].value, environment)?;
            let denominator = self.analyze_expression(&fields[1].value, environment)?;
            require_type(
                &self.source,
                numerator.span,
                &CompilerType::Int,
                &numerator.value_type,
            )?;
            require_type(
                &self.source,
                denominator.span,
                &CompilerType::Int,
                &denominator.value_type,
            )?;
            require_proven_nonzero_int(&self.source, denominator.span, &denominator)?;
            let rational_value = exact_int(&numerator)
                .zip(exact_int(&denominator))
                .map(|(numerator, denominator)| BigRational::new(numerator, denominator));
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::RationalConstruct {
                    numerator: Box::new(numerator),
                    denominator: Box::new(denominator),
                },
                value_type: CompilerType::Rational,
                int_range: None,
                rational_value,
                span,
            });
        }
        let value = self.analyze_expression(argument, environment)?;
        require_type(
            &self.source,
            value.span,
            &CompilerType::Int,
            &value.value_type,
        )?;
        let rational_value = exact_int(&value).map(BigRational::from_integer);
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::IntToRational(Box::new(value)),
            value_type: CompilerType::Rational,
            int_range: None,
            rational_value,
            span,
        })
    }

    fn analyze_range_observation(
        &mut self,
        operation: &str,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand = self.analyze_expression(operand, environment)?;
        let CompilerType::Range(endpoint) = &operand.value_type else {
            if operation == "empty?" {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "empty? for a non-Range value",
                ));
            }
            return Err(source_diagnostic(
                &self.source,
                "E-TYPE-MISMATCH",
                operand.span,
                format!("{operation} requires a finite exact Range operand"),
            ));
        };
        let endpoint = endpoint.as_ref().clone();
        let (kind, value_type) = match operation {
            "range-lower" => (
                CompilerExpressionKind::RangeLower(Box::new(operand)),
                endpoint,
            ),
            "range-upper" => (
                CompilerExpressionKind::RangeUpper(Box::new(operand)),
                endpoint,
            ),
            "range-lower-inclusive?" => (
                CompilerExpressionKind::RangeLowerInclusive(Box::new(operand)),
                CompilerType::Boolean,
            ),
            "range-upper-inclusive?" => (
                CompilerExpressionKind::RangeUpperInclusive(Box::new(operand)),
                CompilerType::Boolean,
            ),
            "empty?" => (
                CompilerExpressionKind::RangeEmpty(Box::new(operand)),
                CompilerType::Boolean,
            ),
            _ => unreachable!("range observation spelling selected above"),
        };
        Ok(CompilerExpression {
            kind,
            value_type,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_identifier_binary(
        &mut self,
        operation: &str,
        left: &Expression,
        right: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut left = self.analyze_expression(left, environment)?;
        let mut right = self.analyze_expression(right, environment)?;
        let binary = match operation {
            "and" => CompilerBinary::And,
            "or" => CompilerBinary::Or,
            "xor" => CompilerBinary::Xor,
            "in" => CompilerBinary::In,
            "contains" => CompilerBinary::Contains,
            _ => unreachable!("identifier binary spelling selected above"),
        };
        if matches!(binary, CompilerBinary::In | CompilerBinary::Contains) {
            let (range, value) = if binary == CompilerBinary::In {
                (&right.value_type, &mut left)
            } else {
                (&left.value_type, &mut right)
            };
            let CompilerType::Range(endpoint) = range else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-RANGE-MEMBERSHIP-OPERANDS",
                    span,
                    "range membership requires a finite exact Range operand",
                ));
            };
            require_exact_numeric(&self.source, value.span, &value.value_type)?;
            if endpoint.as_ref() == &CompilerType::Rational && value.value_type == CompilerType::Int
            {
                *value = into_rational(value.clone());
            }
            require_same_type(&self.source, value.span, endpoint, &value.value_type)?;
            return Ok(Self::finish_binary(
                binary,
                left,
                right,
                CompilerType::Boolean,
                span,
            ));
        }
        if binary == CompilerBinary::And && matches!(&left.value_type, CompilerType::Range(_)) {
            require_same_type(&self.source, span, &left.value_type, &right.value_type)?;
            let value_type = left.value_type.clone();
            return Ok(Self::finish_binary(binary, left, right, value_type, span));
        }
        require_type(
            &self.source,
            left.span,
            &CompilerType::Boolean,
            &left.value_type,
        )?;
        require_type(
            &self.source,
            right.span,
            &CompilerType::Boolean,
            &right.value_type,
        )?;
        Ok(Self::finish_binary(
            binary,
            left,
            right,
            CompilerType::Boolean,
            span,
        ))
    }

    #[allow(clippy::too_many_lines)] // Numeric coercion and fail-closed obligations stay in one selection path.
    fn analyze_symbolic_binary(
        &mut self,
        kind: CallableKind,
        left: &Expression,
        right: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operation = match kind {
            CallableKind::Plus => CompilerBinary::Add,
            CallableKind::Minus => CompilerBinary::Subtract,
            CallableKind::Multiply => CompilerBinary::Multiply,
            CallableKind::Divide => CompilerBinary::Divide,
            CallableKind::Modulo => CompilerBinary::Modulo,
            CallableKind::QuotientModulo => CompilerBinary::QuotientModulo,
            CallableKind::Power => CompilerBinary::Power,
            CallableKind::Compare => CompilerBinary::Compare,
            CallableKind::Range => CompilerBinary::Range,
            CallableKind::RangeOpen => CompilerBinary::RangeOpen,
            CallableKind::RangeInclusive => CompilerBinary::RangeInclusive,
            CallableKind::RangeOpenInclusive => CompilerBinary::RangeOpenInclusive,
            CallableKind::Equal => CompilerBinary::Equal,
            CallableKind::NotEqual => CompilerBinary::NotEqual,
            CallableKind::Less => CompilerBinary::Less,
            CallableKind::Greater => CompilerBinary::Greater,
            CallableKind::LessEqual => CompilerBinary::LessEqual,
            CallableKind::GreaterEqual => CompilerBinary::GreaterEqual,
        };
        let mut left_value = self.analyze_expression(left, environment)?;
        let mut right_value = self.analyze_expression(right, environment)?;

        if is_range_construction(operation) {
            require_exact_numeric(&self.source, left_value.span, &left_value.value_type)?;
            require_exact_numeric(&self.source, right_value.span, &right_value.value_type)?;
            let endpoint = if left_value.value_type == CompilerType::Rational
                || right_value.value_type == CompilerType::Rational
            {
                left_value = into_rational(left_value);
                right_value = into_rational(right_value);
                CompilerType::Rational
            } else {
                CompilerType::Int
            };
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                CompilerType::Range(Box::new(endpoint)),
                span,
            ));
        }

        if matches!(
            operation,
            CompilerBinary::Modulo | CompilerBinary::QuotientModulo
        ) {
            require_type(
                &self.source,
                left_value.span,
                &CompilerType::Int,
                &left_value.value_type,
            )?;
            require_type(
                &self.source,
                right_value.span,
                &CompilerType::Int,
                &right_value.value_type,
            )?;
            require_proven_nonzero_int(&self.source, right_value.span, &right_value)?;
            let result_type = if operation == CompilerBinary::Modulo {
                CompilerType::Int
            } else {
                CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int])
            };
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                result_type,
                span,
            ));
        }

        if operation == CompilerBinary::Power {
            return self.finish_power(left_value, right_value, span);
        }

        let numeric =
            is_exact_numeric(&left_value.value_type) && is_exact_numeric(&right_value.value_type);
        if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual) && !numeric {
            require_same_type(
                &self.source,
                span,
                &left_value.value_type,
                &right_value.value_type,
            )?;
            if !matches!(
                left_value.value_type,
                CompilerType::Boolean | CompilerType::Unit | CompilerType::Comparison
            ) {
                return Err(unsupported(
                    &self.source,
                    span,
                    "equality for this value type",
                ));
            }
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                CompilerType::Boolean,
                span,
            ));
        }

        require_exact_numeric(&self.source, left_value.span, &left_value.value_type)?;
        require_exact_numeric(&self.source, right_value.span, &right_value.value_type)?;
        if operation == CompilerBinary::Divide {
            require_proven_nonzero_numeric(&self.source, right_value.span, &right_value)?;
        }
        let both_int = left_value.value_type == CompilerType::Int
            && right_value.value_type == CompilerType::Int;
        let rational_result = operation == CompilerBinary::Divide
            || left_value.value_type == CompilerType::Rational
            || right_value.value_type == CompilerType::Rational;
        if rational_result {
            left_value = into_rational(left_value);
            right_value = into_rational(right_value);
        }
        let result_type = match operation {
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply => {
                if rational_result {
                    CompilerType::Rational
                } else {
                    CompilerType::Int
                }
            }
            CompilerBinary::Divide => CompilerType::Rational,
            CompilerBinary::Compare => CompilerType::Comparison,
            CompilerBinary::Equal
            | CompilerBinary::NotEqual
            | CompilerBinary::Less
            | CompilerBinary::Greater
            | CompilerBinary::LessEqual
            | CompilerBinary::GreaterEqual => CompilerType::Boolean,
            _ => unreachable!("numeric operation handled above"),
        };
        debug_assert!(!both_int || !rational_result || operation == CompilerBinary::Divide);
        Ok(Self::finish_binary(
            operation,
            left_value,
            right_value,
            result_type,
            span,
        ))
    }

    fn finish_power(
        &self,
        left: CompilerExpression,
        right: CompilerExpression,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_exact_numeric(&self.source, left.span, &left.value_type)?;
        require_type(
            &self.source,
            right.span,
            &CompilerType::Int,
            &right.value_type,
        )?;
        let Some(exponent) = exact_int(&right) else {
            return Err(unsupported(
                &self.source,
                right.span,
                "power with an exponent not proven by specialization",
            ));
        };
        if left.value_type == CompilerType::Int && exponent < BigInt::from(0) {
            return Err(source_diagnostic(
                &self.source,
                "E-TYPE-MISMATCH",
                right.span,
                "an Int exponent must satisfy Nat",
            ));
        }
        if left.value_type == CompilerType::Rational
            && exponent < BigInt::from(0)
            && is_proven_zero_numeric(&left)
        {
            return Err(division_by_zero(&self.source, left.span));
        }
        if left.value_type == CompilerType::Rational
            && exponent < BigInt::from(0)
            && !is_proven_nonzero_numeric(&left)
        {
            return Err(unsupported(
                &self.source,
                left.span,
                "negative Rational power with a base not proven nonzero",
            ));
        }
        let result_type = left.value_type.clone();
        Ok(Self::finish_binary(
            CompilerBinary::Power,
            left,
            right,
            result_type,
            span,
        ))
    }

    fn finish_binary(
        operation: CompilerBinary,
        left: CompilerExpression,
        right: CompilerExpression,
        value_type: CompilerType,
        span: Span,
    ) -> CompilerExpression {
        let int_range = match (&value_type, operation) {
            (CompilerType::Int, CompilerBinary::Add) => {
                combine_ranges(&left, &right, |a, b| IntRange {
                    lower: &a.lower + &b.lower,
                    upper: &a.upper + &b.upper,
                })
            }
            (CompilerType::Int, CompilerBinary::Subtract) => {
                combine_ranges(&left, &right, |a, b| IntRange {
                    lower: &a.lower - &b.upper,
                    upper: &a.upper - &b.lower,
                })
            }
            (CompilerType::Int, CompilerBinary::Multiply) => {
                combine_ranges(&left, &right, multiply_range)
            }
            (CompilerType::Int, CompilerBinary::Modulo) => modulo_range(&left, &right),
            (CompilerType::Int, CompilerBinary::Power) => power_range(&left, &right),
            _ => None,
        };
        let rational_value = (value_type == CompilerType::Rational)
            .then(|| exact_rational_binary(operation, &left, &right))
            .flatten();
        CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation,
                left: Box::new(left),
                right: Box::new(right),
            },
            value_type,
            int_range,
            rational_value,
            span,
        }
    }

    fn analyze_call(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let function = items.iter().enumerate().find_map(|(index, item)| {
            let Expression::Identifier(name) = item else {
                return None;
            };
            self.functions
                .contains_key(self.source.slice(*name))
                .then_some((index, self.source.slice(*name).to_owned()))
        });
        let Some((function_index, function_name)) = function else {
            return Err(unsupported(&self.source, span, "application"));
        };
        let declaration = self
            .functions
            .get(&function_name)
            .expect("selected function exists")
            .clone();
        let argument_sources = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (index != function_index).then_some(item))
            .collect::<Vec<_>>();
        let argument_sources = if declaration.parameters.is_empty()
            && matches!(argument_sources.as_slice(), [Expression::Unit(_)])
        {
            Vec::new()
        } else {
            argument_sources
        };
        if argument_sources.len() != declaration.parameters.len() {
            return Err(source_diagnostic(
                &self.source,
                "E-FUNCTION-ARITY",
                span,
                format!(
                    "function `{function_name}` requires {} operands, found {}",
                    declaration.parameters.len(),
                    argument_sources.len()
                ),
            ));
        }
        let arguments = argument_sources
            .iter()
            .map(|argument| self.analyze_expression(argument, environment))
            .collect::<Result<Vec<_>, _>>()?;
        if !self.active_calls.insert(function_name.clone()) {
            return Err(unsupported(&self.source, span, "recursive function call"));
        }
        let result = self.instantiate_function(&function_name, &declaration, &arguments);
        self.active_calls.remove(&function_name);
        let (symbol, result_type, int_range, rational_value) = result?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Call { symbol, arguments },
            value_type: result_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn instantiate_function(
        &mut self,
        function_name: &str,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
    ) -> Result<(String, CompilerType, Option<IntRange>, Option<BigRational>), Diagnostic> {
        let mut environment = BTreeMap::new();
        let mut parameters = Vec::new();
        for (parameter, argument) in declaration.parameters.iter().zip(arguments) {
            if !parameter.fields.is_empty()
                || parameter.default.is_some()
                || parameter.qualifier.is_some()
            {
                return Err(unsupported(
                    &self.source,
                    parameter.name,
                    "packaged, defaulted, or qualified parameter",
                ));
            }
            let expected = parse_classifier(&self.source, parameter.classifier)?;
            require_same_type(
                &self.source,
                parameter.classifier,
                &expected,
                &argument.value_type,
            )?;
            if !expected.machine_scalar() {
                return Err(unsupported(
                    &self.source,
                    parameter.classifier,
                    "non-scalar function parameter",
                ));
            }
            let name = self.source.slice(parameter.name).to_owned();
            environment.insert(
                name.clone(),
                BindingFacts {
                    value_type: expected.clone(),
                    int_range: argument.int_range.clone(),
                    rational_value: argument.rational_value.clone(),
                },
            );
            parameters.push(CompilerParameter {
                name,
                value_type: expected,
                int_range: argument.int_range.clone(),
                span: parameter.name,
            });
        }
        let result_type = parse_classifier(&self.source, declaration.result)?;
        if !result_type.machine_scalar() {
            return Err(unsupported(
                &self.source,
                declaration.result,
                "non-scalar function result",
            ));
        }
        let body = self.analyze_block(&declaration.body, &mut environment, BlockKind::Function)?;
        require_same_type(
            &self.source,
            declaration.result,
            &result_type,
            &body.result.value_type,
        )?;
        let symbol = format!("topal.fn.{}.{}", mangle(function_name), self.next_instance);
        self.next_instance += 1;
        let int_range = body.result.int_range.clone();
        let rational_value = body.result.rational_value.clone();
        self.instances.push(CompilerFunction {
            source_name: function_name.to_owned(),
            symbol: symbol.clone(),
            parameters,
            result_type: result_type.clone(),
            body,
            span: declaration.span,
        });
        Ok((symbol, result_type, int_range, rational_value))
    }

    fn analyze_decision(
        &mut self,
        subject: &Expression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let subject = self.analyze_expression(subject, environment)?;
        match &subject.value_type {
            CompilerType::Boolean => {
                self.analyze_boolean_decision(subject, rules, span, environment)
            }
            CompilerType::Comparison => {
                self.analyze_comparison_value_decision(subject, rules, span, environment)
            }
            CompilerType::Int | CompilerType::Rational => {
                self.analyze_ordered_comparison_decision(subject, rules, span, environment)
            }
            _ => Err(unsupported(
                &self.source,
                subject.span,
                "decision subject type",
            )),
        }
    }

    fn analyze_boolean_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_type(
            &self.source,
            subject.span,
            &CompilerType::Boolean,
            &subject.value_type,
        )?;
        let mut when_true = None;
        let mut when_false = None;
        let mut otherwise = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Boolean { value: true, .. } if when_true.is_none() => {
                    when_true = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Boolean { value: false, .. } if when_false.is_none() => {
                    when_false = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Otherwise(_) if otherwise.is_none() => {
                    otherwise = Some(self.analyze_expression(&rule.action, environment)?);
                }
                _ => return Err(unsupported(&self.source, rule.span, "decision matcher")),
            }
        }
        let when_true = when_true.or_else(|| otherwise.clone()).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `true`",
            )
        })?;
        let when_false = when_false.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `false`",
            )
        })?;
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_true, &when_false], span)?;
        Ok(CompilerExpression {
            value_type,
            kind: CompilerExpressionKind::BooleanDecision {
                subject: Box::new(subject),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
            },
            int_range,
            rational_value,
            span,
        })
    }

    fn analyze_ordered_comparison_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_exact_numeric(&self.source, subject.span, &subject.value_type)?;
        let mut lowered = Vec::new();
        let mut otherwise = None;
        for (index, rule) in rules.iter().enumerate() {
            match &rule.matcher {
                DecisionMatcher::Comparison {
                    kind,
                    operand,
                    span: matcher_span,
                } => {
                    if otherwise.is_some() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNREACHABLE-DECISION-RULE",
                            rule.span,
                            "a comparison rule cannot follow otherwise",
                        ));
                    }
                    let Some(operation) = comparison_binary(*kind) else {
                        return Err(unsupported(
                            &self.source,
                            *matcher_span,
                            "comparison decision callable",
                        ));
                    };
                    let mut operand = self.analyze_expression(operand, environment)?;
                    require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
                    let subject_to_rational = subject.value_type == CompilerType::Int
                        && operand.value_type == CompilerType::Rational;
                    if subject.value_type == CompilerType::Rational
                        && operand.value_type == CompilerType::Int
                    {
                        operand = into_rational(operand);
                    }
                    if !subject_to_rational {
                        require_same_type(
                            &self.source,
                            operand.span,
                            &subject.value_type,
                            &operand.value_type,
                        )?;
                    }
                    lowered.push(CompilerComparisonRule {
                        operation,
                        operand,
                        action: self.analyze_expression(&rule.action, environment)?,
                        subject_to_rational,
                        span: rule.span,
                    });
                }
                DecisionMatcher::Otherwise(_) if index + 1 == rules.len() => {
                    otherwise = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Otherwise(_) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNREACHABLE-DECISION-RULE",
                        rule.span,
                        "otherwise must be the final comparison rule",
                    ));
                }
                _ => return Err(unsupported(&self.source, rule.span, "decision matcher")),
            }
        }
        let otherwise = otherwise.ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "a comparison decision requires otherwise",
            )
        })?;
        let mut branches = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
        branches.push(&otherwise);
        let (value_type, int_range, rational_value) = self.decision_facts(&branches, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::OrderedComparisonDecision {
                subject: Box::new(subject),
                rules: lowered,
                otherwise: Box::new(otherwise),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn analyze_comparison_value_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut when_less = None;
        let mut when_equal = None;
        let mut when_greater = None;
        let mut otherwise = None;
        for (index, rule) in rules.iter().enumerate() {
            let destination = match &rule.matcher {
                DecisionMatcher::Identifier(name) => match self.source.slice(*name) {
                    "Less" => &mut when_less,
                    "Equal" => &mut when_equal,
                    "Greater" => &mut when_greater,
                    _ => return Err(unsupported(&self.source, *name, "Comparison alternative")),
                },
                DecisionMatcher::Otherwise(_) if index + 1 == rules.len() => &mut otherwise,
                DecisionMatcher::Otherwise(_) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNREACHABLE-DECISION-RULE",
                        rule.span,
                        "otherwise must be the final Comparison rule",
                    ));
                }
                _ => return Err(unsupported(&self.source, rule.span, "decision matcher")),
            };
            if destination.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-DECISION-RULE",
                    rule.span,
                    "a Comparison alternative appears more than once",
                ));
            }
            *destination = Some(self.analyze_expression(&rule.action, environment)?);
        }
        let missing = |name: &str| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                format!("Comparison decision does not cover {name}"),
            )
        };
        let when_less = when_less
            .or_else(|| otherwise.clone())
            .ok_or_else(|| missing("Less"))?;
        let when_equal = when_equal
            .or_else(|| otherwise.clone())
            .ok_or_else(|| missing("Equal"))?;
        let when_greater = when_greater
            .or(otherwise)
            .ok_or_else(|| missing("Greater"))?;
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_less, &when_equal, &when_greater], span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ComparisonValueDecision {
                subject: Box::new(subject),
                when_less: Box::new(when_less),
                when_equal: Box::new(when_equal),
                when_greater: Box::new(when_greater),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn decision_facts(
        &self,
        branches: &[&CompilerExpression],
        span: Span,
    ) -> Result<(CompilerType, Option<IntRange>, Option<BigRational>), Diagnostic> {
        let first = branches.first().expect("a complete decision has a branch");
        for branch in &branches[1..] {
            require_same_type(&self.source, span, &first.value_type, &branch.value_type)?;
        }
        if !first.value_type.machine_scalar() {
            return Err(unsupported(
                &self.source,
                span,
                "decision actions with non-scalar results",
            ));
        }
        let int_range = branches
            .iter()
            .try_fold(None, |range: Option<IntRange>, branch| {
                match (range, &branch.int_range) {
                    (None, Some(next)) => Some(Some(next.clone())),
                    (Some(current), Some(next)) => Some(Some(IntRange::union(&current, next))),
                    (_, None) => None,
                }
            })
            .flatten();
        let rational_value = first.rational_value.clone().filter(|value| {
            branches
                .iter()
                .all(|branch| branch.rational_value.as_ref() == Some(value))
        });
        Ok((first.value_type.clone(), int_range, rational_value))
    }
}

fn is_declaration(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::LanguageSelection { .. } | Statement::Function { .. }
    ) || matches!(statement, Statement::Published { declaration, .. } if matches!(declaration.as_ref(), Statement::Function { .. }))
}

fn parse_classifier(source: &SourceText, span: Span) -> Result<CompilerType, Diagnostic> {
    let classifier = source.slice(span).trim();
    if let Some(endpoint) = classifier.strip_prefix("Range ") {
        return match endpoint {
            "Int" => Ok(CompilerType::Range(Box::new(CompilerType::Int))),
            "Rational" => Ok(CompilerType::Range(Box::new(CompilerType::Rational))),
            _ => Err(unsupported(source, span, "Range endpoint classifier")),
        };
    }
    match classifier {
        "Unit" => Ok(CompilerType::Unit),
        "Boolean" => Ok(CompilerType::Boolean),
        "Int" => Ok(CompilerType::Int),
        "Rational" => Ok(CompilerType::Rational),
        "Comparison" => Ok(CompilerType::Comparison),
        "String" => Ok(CompilerType::String),
        _ => Err(unsupported(source, span, "classifier")),
    }
}

fn is_range_construction(operation: CompilerBinary) -> bool {
    matches!(
        operation,
        CompilerBinary::Range
            | CompilerBinary::RangeOpen
            | CompilerBinary::RangeInclusive
            | CompilerBinary::RangeOpenInclusive
    )
}

fn comparison_binary(kind: CallableKind) -> Option<CompilerBinary> {
    match kind {
        CallableKind::Equal => Some(CompilerBinary::Equal),
        CallableKind::NotEqual => Some(CompilerBinary::NotEqual),
        CallableKind::Less => Some(CompilerBinary::Less),
        CallableKind::Greater => Some(CompilerBinary::Greater),
        CallableKind::LessEqual => Some(CompilerBinary::LessEqual),
        CallableKind::GreaterEqual => Some(CompilerBinary::GreaterEqual),
        _ => None,
    }
}

fn is_exact_numeric(value_type: &CompilerType) -> bool {
    matches!(value_type, CompilerType::Int | CompilerType::Rational)
}

fn require_exact_numeric(
    source: &SourceText,
    span: Span,
    value_type: &CompilerType,
) -> Result<(), Diagnostic> {
    if is_exact_numeric(value_type) {
        Ok(())
    } else {
        Err(source_diagnostic(
            source,
            "E-TYPE-MISMATCH",
            span,
            format!("expected Int or Rational, found {}", value_type.name()),
        ))
    }
}

fn exact_int(expression: &CompilerExpression) -> Option<BigInt> {
    let range = expression.int_range.as_ref()?;
    (range.lower == range.upper).then(|| range.lower.clone())
}

fn into_rational(expression: CompilerExpression) -> CompilerExpression {
    if expression.value_type == CompilerType::Rational {
        return expression;
    }
    let span = expression.span;
    let rational_value = exact_int(&expression).map(BigRational::from_integer);
    CompilerExpression {
        kind: CompilerExpressionKind::IntToRational(Box::new(expression)),
        value_type: CompilerType::Rational,
        int_range: None,
        rational_value,
        span,
    }
}

fn rational_absolute(value: &BigRational) -> BigRational {
    if value.numer() < &BigInt::from(0) {
        -value.clone()
    } else {
        value.clone()
    }
}

fn is_proven_zero_numeric(expression: &CompilerExpression) -> bool {
    match expression.value_type {
        CompilerType::Int => exact_int(expression).is_some_and(|value| value == BigInt::from(0)),
        CompilerType::Rational => expression
            .rational_value
            .as_ref()
            .is_some_and(|value| value.numer() == &BigInt::from(0)),
        _ => false,
    }
}

fn is_proven_nonzero_numeric(expression: &CompilerExpression) -> bool {
    match expression.value_type {
        CompilerType::Int => expression
            .int_range
            .as_ref()
            .is_some_and(|range| range.upper < BigInt::from(0) || range.lower > BigInt::from(0)),
        CompilerType::Rational => expression
            .rational_value
            .as_ref()
            .is_some_and(|value| value.numer() != &BigInt::from(0)),
        _ => false,
    }
}

fn require_proven_nonzero_int(
    source: &SourceText,
    span: Span,
    expression: &CompilerExpression,
) -> Result<(), Diagnostic> {
    debug_assert_eq!(expression.value_type, CompilerType::Int);
    require_proven_nonzero_numeric(source, span, expression)
}

fn require_proven_nonzero_numeric(
    source: &SourceText,
    span: Span,
    expression: &CompilerExpression,
) -> Result<(), Diagnostic> {
    if is_proven_zero_numeric(expression) {
        Err(division_by_zero(source, span))
    } else if is_proven_nonzero_numeric(expression) {
        Ok(())
    } else {
        Err(unsupported(
            source,
            span,
            "exact division with a divisor not proven nonzero",
        ))
    }
}

fn division_by_zero(source: &SourceText, span: Span) -> Diagnostic {
    source_diagnostic(
        source,
        "E-DIVISION-BY-ZERO",
        span,
        "exact division requires a nonzero divisor",
    )
}

fn modulo_range(dividend: &CompilerExpression, divisor: &CompilerExpression) -> Option<IntRange> {
    if let (Some(dividend), Some(divisor)) = (exact_int(dividend), exact_int(divisor)) {
        let magnitude = if divisor < BigInt::from(0) {
            -divisor
        } else {
            divisor
        };
        let mut remainder = dividend % &magnitude;
        if remainder < BigInt::from(0) {
            remainder += magnitude;
        }
        return Some(IntRange::exact(remainder));
    }
    let range = divisor.int_range.as_ref()?;
    let magnitude = (-range.lower.clone()).max(range.upper.clone());
    Some(IntRange {
        lower: BigInt::from(0),
        upper: magnitude - 1,
    })
}

fn power_range(base: &CompilerExpression, exponent: &CompilerExpression) -> Option<IntRange> {
    let base = exact_int(base)?;
    let exponent = exact_int(exponent)?.to_string().parse::<u32>().ok()?;
    Some(IntRange::exact(base.pow(exponent)))
}

fn exact_rational_binary(
    operation: CompilerBinary,
    left: &CompilerExpression,
    right: &CompilerExpression,
) -> Option<BigRational> {
    let left = left.rational_value.as_ref()?;
    if operation == CompilerBinary::Power {
        let exponent = exact_int(right)?.to_string().parse::<i32>().ok()?;
        Some(left.pow(exponent))
    } else {
        let right = right.rational_value.as_ref()?;
        match operation {
            CompilerBinary::Add => Some(left + right),
            CompilerBinary::Subtract => Some(left - right),
            CompilerBinary::Multiply => Some(left * right),
            CompilerBinary::Divide => Some(left / right),
            _ => None,
        }
    }
}

fn combine_ranges(
    left: &CompilerExpression,
    right: &CompilerExpression,
    operation: impl FnOnce(&IntRange, &IntRange) -> IntRange,
) -> Option<IntRange> {
    Some(operation(
        left.int_range.as_ref()?,
        right.int_range.as_ref()?,
    ))
}

fn multiply_range(left: &IntRange, right: &IntRange) -> IntRange {
    let products = [
        &left.lower * &right.lower,
        &left.lower * &right.upper,
        &left.upper * &right.lower,
        &left.upper * &right.upper,
    ];
    IntRange {
        lower: products.iter().min().expect("four products").clone(),
        upper: products.iter().max().expect("four products").clone(),
    }
}

fn absolute_range(range: &IntRange) -> IntRange {
    let zero = BigInt::from(0);
    if range.lower >= zero {
        range.clone()
    } else if range.upper <= zero {
        IntRange {
            lower: -range.upper.clone(),
            upper: -range.lower.clone(),
        }
    } else {
        IntRange {
            lower: zero,
            upper: (-range.lower.clone()).max(range.upper.clone()),
        }
    }
}

fn require_type(
    source: &SourceText,
    span: Span,
    expected: &CompilerType,
    actual: &CompilerType,
) -> Result<(), Diagnostic> {
    require_same_type(source, span, expected, actual)
}

fn require_same_type(
    source: &SourceText,
    span: Span,
    expected: &CompilerType,
    actual: &CompilerType,
) -> Result<(), Diagnostic> {
    if expected == actual {
        Ok(())
    } else {
        Err(source_diagnostic(
            source,
            "E-TYPE-MISMATCH",
            span,
            format!("expected {}, found {}", expected.name(), actual.name()),
        ))
    }
}

fn unit_expression(span: Span) -> CompilerExpression {
    CompilerExpression {
        kind: CompilerExpressionKind::Unit,
        value_type: CompilerType::Unit,
        int_range: None,
        rational_value: None,
        span,
    }
}

fn mangle(name: &str) -> String {
    name.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                char::from(byte).to_string()
            } else {
                format!("_{byte:02x}")
            }
        })
        .collect()
}

fn unsupported(source: &SourceText, span: Span, construct: &str) -> Diagnostic {
    source_diagnostic(
        source,
        "E-COMPILER-UNSUPPORTED",
        span,
        format!("the current native compiler increment does not yet support {construct}"),
    )
}

fn source_diagnostic(
    source: &SourceText,
    code: impl Into<String>,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    let start = span.start.min(source.as_str().len());
    let end = span.end.min(source.as_str().len()).max(start);
    let position = source.position(start);
    let line = source
        .as_str()
        .lines()
        .nth(position.line.saturating_sub(1))
        .map(ToOwned::to_owned);
    let width = source.slice(Span::new(start, end)).chars().count().max(1);
    Diagnostic::error(code, position.line, position.column, message)
        .with_source_span(span)
        .with_source_excerpt(line, width)
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::LanguageSelection { span, .. }
        | Statement::LibrarySelection { span, .. }
        | Statement::Published { span, .. }
        | Statement::DiagnosticControl { span, .. }
        | Statement::Implementation { span, .. }
        | Statement::StateField { name: span, .. }
        | Statement::ContextAssignment { span, .. }
        | Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Interface { span, .. }
        | Statement::InterfaceImplementation { span, .. }
        | Statement::Foreach { span, .. }
        | Statement::Discard { span, .. } => *span,
        Statement::Binding { name, value, .. } => Span::new(name.start, value.span().end),
        Statement::Return { keyword, value } => Span::new(keyword.start, value.span().end),
        Statement::Expression(expression) => expression.span(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzes_functions_and_boolean_decisions_for_the_native_backend() {
        let source = "use language (version is v0.1)\nchoose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    otherwise 0\n(choose true, choose false)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(program.functions.len(), 2);
        assert_eq!(program.main.result.value_type.name(), "(Int, Int)");
    }

    #[test]
    fn preserves_arbitrary_integer_ranges_for_the_native_backend() {
        let source = "use language (version is v0.1)\n9223372036854775807 + 1\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.int_range,
            Some(IntRange::exact(BigInt::from(i64::MAX) + 1))
        );
    }

    #[test]
    fn models_finite_exact_conversion_division_and_power() {
        let source = "use language (version is v0.1)\n(1 + 0.5, 6 / 8, -17 % 5, 2 ^ 16, 1 <=> 2)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Rational, Rational, Int, Int, Comparison)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected a tuple")
        };
        assert_eq!(
            values[0].rational_value,
            Some(BigRational::new(BigInt::from(3), BigInt::from(2)))
        );
        assert_eq!(values[2].int_range, Some(IntRange::exact(BigInt::from(3))));
        assert_eq!(
            values[3].int_range,
            Some(IntRange::exact(BigInt::from(65_536)))
        );
    }

    #[test]
    fn rejects_statically_zero_exact_divisors() {
        let source = "use language (version is v0.1)\n1 / 0\n";
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-DIVISION-BY-ZERO"
        );
    }

    #[test]
    fn models_finite_exact_ranges_and_observations() {
        let source = "use language (version is v0.1)\ninterval is 0 ..= 2.5\npreserve is fn (value : Range Rational) -> Range Rational\n  value\nkept is preserve interval\n(1 in kept, kept contains 3, empty? (2 .. 2), range-lower kept, range-upper-inclusive? kept, kept and (1.0 <.. 4.0))\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Boolean, Boolean, Boolean, Rational, Boolean, Range Rational)"
        );
        assert!(
            program
                .functions
                .iter()
                .any(|function| function.result_type.name() == "Range Rational")
        );
    }

    #[test]
    fn models_ordered_and_comparison_value_decisions() {
        let source = "use language (version is v0.1)\nrank is fn (value : Comparison) -> Int\n  value\n    Less then -1\n    Equal then 0\n    Greater then 1\nlocate is fn (value : Int, pivot : Rational) -> Int\n  value\n    < pivot - 0.5 then -1\n    = pivot then 0\n    otherwise 1\n(rank (1 <=> 2), 0 locate 1.5)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::ComparisonValueDecision { .. }
        )));
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::OrderedComparisonDecision { .. }
        )));
    }

    #[test]
    fn rejects_a_library_until_compiled_dependency_loading_exists() {
        let source = "use language (version is v0.1)\nuse library std (version is v0.1)\n()\n";
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}
