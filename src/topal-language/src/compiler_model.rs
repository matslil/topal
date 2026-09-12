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
pub struct CompilerEnumType {
    pub name: String,
    pub alternatives: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerType {
    Unit,
    Completed,
    Boolean,
    Int,
    Nat,
    Rational,
    Comparison,
    Error,
    ErrorCode,
    ErrorDomain,
    Enum(CompilerEnumType),
    Range(Box<Self>),
    Result(Box<Self>),
    Optional(Box<Self>),
    String,
    Tuple(Vec<Self>),
}

impl CompilerType {
    #[must_use]
    pub const fn machine_scalar(&self) -> bool {
        matches!(
            self,
            Self::Unit
                | Self::Completed
                | Self::Boolean
                | Self::Int
                | Self::Nat
                | Self::Rational
                | Self::Comparison
                | Self::Error
                | Self::ErrorCode
                | Self::ErrorDomain
                | Self::Enum(_)
                | Self::Range(_)
                | Self::Result(_)
                | Self::Optional(_)
                | Self::String
        )
    }

    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Unit => "Unit".into(),
            Self::Completed => "Completed".into(),
            Self::Boolean => "Boolean".into(),
            Self::Int => "Int".into(),
            Self::Nat => "Nat".into(),
            Self::Rational => "Rational".into(),
            Self::Comparison => "Comparison".into(),
            Self::Error => "Error".into(),
            Self::ErrorCode => "lang arithmetic ArithmeticErrorCode".into(),
            Self::ErrorDomain => "ErrorDomain".into(),
            Self::Enum(enumeration) => enumeration.name.clone(),
            Self::Range(endpoint) => format!("Range {}", endpoint.name()),
            Self::Result(success) => format!(
                "Result ({}, lang arithmetic ArithmeticErrorCode)",
                success.name()
            ),
            Self::Optional(payload) => format!("Optional {}", payload.name()),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerFallible {
    RationalConstruct,
    RationalDivide,
    RationalPower,
    IntModulo,
    IntQuotientModulo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerValidation {
    RationalToInt,
    IntToNat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerErrorField {
    Code,
    Domain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerErrorCodeRule {
    pub code: u32,
    pub action: CompilerExpression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerEnumRule {
    pub value: u32,
    pub action: CompilerExpression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerExpressionKind {
    Unit,
    Completed,
    Boolean(bool),
    Int(BigInt),
    Rational(BigRational),
    String(String),
    StringEmpty,
    StringConcat {
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
    },
    StringEmptyPredicate(Box<CompilerExpression>),
    StringUtf8ByteCount(Box<CompilerExpression>),
    ErrorCode(u32),
    Enum(u32),
    Tuple(Vec<CompilerExpression>),
    Block(Box<CompilerBlock>),
    Local(String),
    Negate(Box<CompilerExpression>),
    Absolute(Box<CompilerExpression>),
    IntToRational(Box<CompilerExpression>),
    RationalConstruct {
        numerator: Box<CompilerExpression>,
        denominator: Box<CompilerExpression>,
    },
    RationalToInt(Box<CompilerExpression>),
    IntToNat(Box<CompilerExpression>),
    ResultSuccess(Box<CompilerExpression>),
    ResultProject(Box<CompilerExpression>),
    OptionalSome(Box<CompilerExpression>),
    OptionalNone,
    ErrorField {
        error: Box<CompilerExpression>,
        field: CompilerErrorField,
    },
    Validate {
        operation: CompilerValidation,
        value: Box<CompilerExpression>,
        error_span: Span,
    },
    Fallible {
        operation: CompilerFallible,
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
        error_span: Span,
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
    EnumDecision {
        subject: Box<CompilerExpression>,
        rules: Vec<CompilerEnumRule>,
        otherwise: Option<Box<CompilerExpression>>,
    },
    ResultDecision {
        subject: Box<CompilerExpression>,
        ok_binding: String,
        ok_binding_span: Span,
        ok_action: Box<CompilerExpression>,
        error_codes: Vec<CompilerErrorCodeRule>,
        error_fallback: Option<(String, Span, Box<CompilerExpression>)>,
    },
    OptionalDecision {
        subject: Box<CompilerExpression>,
        some_binding: Option<(String, Span)>,
        some_action: Box<CompilerExpression>,
        none_action: Box<CompilerExpression>,
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
    pub discarded: bool,
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
    pub is_static: bool,
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
    is_static: bool,
}

type EnumTypes = BTreeMap<String, (CompilerEnumType, Span)>;
type EnumAlternativeBindings = BTreeMap<String, (CompilerEnumType, u32, Span)>;

struct EnumSource {
    name: Span,
    alternatives: Vec<(String, Span)>,
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
    enums: EnumTypes,
    enum_alternatives: EnumAlternativeBindings,
    functions: BTreeMap<String, Vec<FunctionSource>>,
    instances: Vec<CompilerFunction>,
    active_calls: BTreeSet<String>,
    static_context: bool,
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

    let (enums, enum_alternatives) = collect_enums(&source, &parsed.statements)?;
    let reserved_names = enums
        .keys()
        .chain(enum_alternatives.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut functions = BTreeMap::new();
    collect_functions(&source, &parsed.statements, &reserved_names, &mut functions)?;
    let mut analyzer = Analyzer {
        source: source.clone(),
        enums,
        enum_alternatives,
        functions,
        instances: Vec::new(),
        active_calls: BTreeSet::new(),
        static_context: false,
        next_instance: 0,
    };
    let mut environment = BTreeMap::new();
    let main = analyzer.analyze_block(
        &parsed.statements,
        &mut environment,
        BlockKind::TopLevel,
        None,
    )?;
    Ok(CompilerProgram {
        source,
        language_version,
        main,
        functions: analyzer.instances,
    })
}

fn collect_enums(
    source: &SourceText,
    statements: &[Statement],
) -> Result<(EnumTypes, EnumAlternativeBindings), Diagnostic> {
    let mut enums = BTreeMap::new();
    let mut alternatives = BTreeMap::new();
    for statement in statements {
        let Some(declaration) = enum_declaration(source, statement) else {
            continue;
        };
        let EnumSource {
            name: name_span,
            alternatives: declarations,
            span: declaration_span,
        } = declaration;
        let name = source.slice(name_span).to_owned();
        if enums.contains_key(&name) || alternatives.contains_key(&name) {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-BINDING",
                name_span,
                format!("`{name}` is already declared in this scope"),
            ));
        }
        let mut local = BTreeSet::new();
        for (label, alternative_span) in &declarations {
            if label == &name
                || !local.insert(label.clone())
                || enums.contains_key(label)
                || alternatives.contains_key(label)
            {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-ENUM-ALTERNATIVE",
                    *alternative_span,
                    format!("enum alternative `{label}` is already declared in this scope"),
                ));
            }
        }
        let enumeration = CompilerEnumType {
            name: name.clone(),
            alternatives: declarations.into_iter().map(|(label, _)| label).collect(),
        };
        enums.insert(name, (enumeration.clone(), declaration_span));
        for (index, label) in enumeration.alternatives.iter().enumerate() {
            let index = u32::try_from(index).map_err(|_| {
                source_diagnostic(
                    source,
                    "E-COMPILER-UNSUPPORTED",
                    declaration_span,
                    "an Enum has more alternatives than the native tag can represent",
                )
            })?;
            alternatives.insert(
                label.clone(),
                (enumeration.clone(), index, declaration_span),
            );
        }
    }
    Ok((enums, alternatives))
}

fn enum_declaration(source: &SourceText, statement: &Statement) -> Option<EnumSource> {
    let Statement::Binding {
        name,
        classifier: None,
        value,
    } = (match statement {
        Statement::Published { declaration, .. } => declaration.as_ref(),
        statement => statement,
    })
    else {
        return None;
    };
    let Expression::Application { items, span } = value else {
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
    let alternatives = fields
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
        .collect::<Option<Vec<_>>>()?;
    Some(EnumSource {
        name: *name,
        alternatives,
        span: Span::new(name.start, span.end),
    })
}

fn collect_functions(
    source: &SourceText,
    statements: &[Statement],
    reserved_names: &BTreeSet<String>,
    functions: &mut BTreeMap<String, Vec<FunctionSource>>,
) -> Result<(), Diagnostic> {
    for statement in statements {
        let declaration = match statement {
            Statement::Function {
                name,
                parameters,
                result,
                body,
                span,
                is_static,
                effect_bound: None,
                clauses,
            } if **clauses == FunctionClauses::default() => Some(FunctionSource {
                name: *name,
                parameters: parameters.clone(),
                result: *result,
                body: body.clone(),
                span: *span,
                is_static: *is_static,
            }),
            Statement::Published { declaration, .. } => {
                if let Statement::Function {
                    name,
                    parameters,
                    result,
                    body,
                    span,
                    is_static,
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
                        is_static: *is_static,
                    })
                } else if enum_declaration(source, statement).is_some() {
                    None
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
                    "constrained or effectful function",
                ));
            }
            _ => None,
        };
        if let Some(function) = declaration {
            let name = source.slice(function.name).to_owned();
            if reserved_names.contains(&name) {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-BINDING",
                    function.name,
                    format!("`{name}` is already declared in this scope"),
                ));
            }
            let overloads = functions.entry(name.clone()).or_default();
            if overloads
                .iter()
                .any(|candidate| same_function_input_header(source, candidate, &function))
            {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-FUNCTION-OVERLOAD",
                    statement_span(statement),
                    format!("function `{name}` repeats an input signature and staticness"),
                ));
            }
            overloads.push(function);
        }
    }
    Ok(())
}

fn same_function_input_header(
    source: &SourceText,
    left: &FunctionSource,
    right: &FunctionSource,
) -> bool {
    left.is_static == right.is_static
        && left.parameters.len() == right.parameters.len()
        && left
            .parameters
            .iter()
            .zip(&right.parameters)
            .all(|(left, right)| {
                compact_classifier(source.slice(left.classifier))
                    == compact_classifier(source.slice(right.classifier))
            })
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BlockKind {
    TopLevel,
    Function,
    Lexical,
}

impl Analyzer {
    fn analyze_expression_with_expected(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        expected: Option<&CompilerType>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Expression::Identifier(name) = expression
            && self.source.slice(*name) == "None"
            && let Some(CompilerType::Optional(payload)) = expected
        {
            return self.finish_optional_none(payload.as_ref().clone(), expression.span());
        }
        self.analyze_expression(expression, environment)
    }

    fn parse_classifier(&self, span: Span) -> Result<CompilerType, Diagnostic> {
        let classifier = compact_classifier(self.source.slice(span));
        if let Some((enumeration, declaration)) = self.enums.get(&classifier)
            && declaration.end <= span.start
        {
            return Ok(CompilerType::Enum(enumeration.clone()));
        }
        parse_compact_classifier(&classifier)
            .ok_or_else(|| unsupported(&self.source, span, "classifier"))
    }

    fn is_declaration(&self, statement: &Statement) -> bool {
        if matches!(
            statement,
            Statement::LanguageSelection { .. } | Statement::Function { .. }
        ) || matches!(statement, Statement::Published { declaration, .. } if matches!(declaration.as_ref(), Statement::Function { .. }))
        {
            return true;
        }
        let Some(declaration) = enum_declaration(&self.source, statement) else {
            return false;
        };
        let name = self.source.slice(declaration.name);
        self.enums
            .get(name)
            .is_some_and(|(_, span)| *span == declaration.span)
    }

    #[allow(clippy::too_many_lines)] // Exhaustive statement admission keeps the subset boundary visible.
    fn analyze_block(
        &mut self,
        statements: &[Statement],
        environment: &mut BTreeMap<String, BindingFacts>,
        kind: BlockKind,
        enclosing_result: Option<&CompilerType>,
    ) -> Result<CompilerBlock, Diagnostic> {
        let mut lowered = Vec::new();
        let mut result = None;
        let mut declared = if kind == BlockKind::Lexical {
            BTreeSet::new()
        } else {
            environment.keys().cloned().collect()
        };
        if kind == BlockKind::Lexical
            && let Some(declaration) = statements
                .iter()
                .find(|statement| self.is_declaration(statement))
        {
            return Err(unsupported(
                &self.source,
                statement_span(declaration),
                "nested declaration",
            ));
        }
        let executable = statements
            .iter()
            .filter(|statement| !self.is_declaration(statement))
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
                    if declared.contains(&name_text)
                        || (kind == BlockKind::TopLevel
                            && (self.functions.contains_key(&name_text)
                                || self.enums.contains_key(&name_text)
                                || self.enum_alternatives.contains_key(&name_text)))
                    {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-DUPLICATE-BINDING",
                            *name,
                            format!("`{name_text}` is already declared in this scope"),
                        ));
                    }
                    let expected = classifier
                        .map(|classifier| self.parse_classifier(classifier))
                        .transpose()?;
                    let mut value = self.analyze_expression_with_expected(
                        value,
                        environment,
                        expected.as_ref(),
                    )?;
                    if let (Some(classifier), Some(expected)) = (classifier, expected) {
                        if expected == CompilerType::Int
                            && value.value_type == CompilerType::Rational
                        {
                            let span = value.span;
                            value = self.finish_int_conversion(value, span, span)?;
                        } else if expected == CompilerType::Nat
                            && value.value_type == CompilerType::Int
                        {
                            let span = value.span;
                            value = self.finish_nat_conversion(value, span, span)?;
                        }
                        if let CompilerType::Result(success) = &value.value_type
                            && success.as_ref() == &expected
                        {
                            if !matches!(enclosing_result, Some(CompilerType::Result(_))) {
                                return Err(source_diagnostic(
                                    &self.source,
                                    "E-RESULT-PROJECTION-CONTEXT",
                                    value.span,
                                    "Result success projection requires an enclosing compatible Result function",
                                ));
                            }
                            let span = value.span;
                            value = CompilerExpression {
                                kind: CompilerExpressionKind::ResultProject(Box::new(value)),
                                value_type: expected.clone(),
                                int_range: None,
                                rational_value: None,
                                span,
                            };
                        }
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
                    declared.insert(name_text.clone());
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
                    result = Some(self.analyze_expression_with_expected(
                        expression,
                        environment,
                        enclosing_result,
                    )?);
                }
                Statement::Expression(_) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-NONFINAL-VALUE",
                        statement_span(statement),
                        "a non-final value expression must be explicitly discarded or bound",
                    ));
                }
                Statement::Return { value, .. } if kind == BlockKind::Function => {
                    result = Some(self.analyze_expression_with_expected(
                        value,
                        environment,
                        enclosing_result,
                    )?);
                    break;
                }
                Statement::Return { .. } if kind == BlockKind::TopLevel => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-RETURN-OUTSIDE-FUNCTION",
                        statement_span(statement),
                        "`return` is available only inside a function",
                    ));
                }
                Statement::Return { .. } if kind == BlockKind::Lexical => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "return through a nested lexical block",
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
            Expression::Block { statements, .. } => {
                let mut nested = environment.clone();
                let mut block =
                    self.analyze_block(statements, &mut nested, BlockKind::Lexical, None)?;
                if statements.is_empty() {
                    block.result = unit_expression(span);
                }
                Ok(CompilerExpression {
                    value_type: block.result.value_type.clone(),
                    int_range: block.result.int_range.clone(),
                    rational_value: block.result.rational_value.clone(),
                    kind: CompilerExpressionKind::Block(Box::new(block)),
                    span,
                })
            }
            Expression::Unit(_) => Ok(unit_expression(span)),
            Expression::Identifier(name) if self.source.slice(*name) == "Completed" => {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Completed,
                    value_type: CompilerType::Completed,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
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
            Expression::Identifier(name)
                if !environment.contains_key(self.source.slice(*name))
                    && self
                        .enum_alternatives
                        .get(self.source.slice(*name))
                        .is_some_and(|(_, _, declaration)| declaration.end <= name.start) =>
            {
                let (enumeration, value, _) = self
                    .enum_alternatives
                    .get(self.source.slice(*name))
                    .expect("checked enum alternative exists")
                    .clone();
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Enum(value),
                    value_type: CompilerType::Enum(enumeration),
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
        if items.len() > 1
            && items
                .iter()
                .all(|item| matches!(item, Expression::String(_)))
        {
            let mut value = String::new();
            for item in items {
                let Expression::String(literal) = item else {
                    unreachable!("checked adjacent String literals")
                };
                value.push_str(parse_string(self.source.slice(*literal)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-STRING-LITERAL",
                        *literal,
                        "invalid string literal delimiter",
                    )
                })?);
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::String(value),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(operation),
            Expression::Identifier(domain),
        ] = items
            && self.source.slice(*operation) == "empty"
            && self.source.slice(*domain) == "String"
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringEmpty,
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if items.len() >= 3
            && items.len() % 2 == 1
            && items.iter().skip(1).step_by(2).all(
                |item| matches!(item, Expression::Identifier(operation) if self.source.slice(*operation) == "concat"),
            )
        {
            let mut result = self.analyze_expression(&items[0], environment)?;
            require_type(
                &self.source,
                result.span,
                &CompilerType::String,
                &result.value_type,
            )?;
            for operand in items.iter().skip(2).step_by(2) {
                let right = self.analyze_expression(operand, environment)?;
                require_type(
                    &self.source,
                    right.span,
                    &CompilerType::String,
                    &right.value_type,
                )?;
                let expression_span = Span::new(result.span.start, right.span.end);
                result = CompilerExpression {
                    kind: CompilerExpressionKind::StringConcat {
                        left: Box::new(result),
                        right: Box::new(right),
                    },
                    value_type: CompilerType::String,
                    int_range: None,
                    rational_value: None,
                    span: expression_span,
                };
            }
            result.span = span;
            return Ok(result);
        }
        if let [Expression::Identifier(constructor), value] = items
            && self.source.slice(*constructor) == "Some"
        {
            let value = self.analyze_expression(value, environment)?;
            require_optional_payload(&self.source, value.span, &value.value_type)?;
            return Ok(CompilerExpression {
                value_type: CompilerType::Optional(Box::new(value.value_type.clone())),
                kind: CompilerExpressionKind::OptionalSome(Box::new(value)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(constructor),
            Expression::Identifier(payload),
        ] = items
            && self.source.slice(*constructor) == "None"
        {
            let payload = self.parse_classifier(*payload)?;
            return self.finish_optional_none(payload, span);
        }
        if let [
            value,
            Expression::Identifier(operation),
            Expression::Identifier(encoding),
        ] = items
            && self.source.slice(*operation) == "byte-count"
        {
            if self.source.slice(*encoding) != "Utf8" {
                return Err(source_diagnostic(
                    &self.source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    *encoding,
                    "the compiler String byte-count operation requires Utf8",
                ));
            }
            let value = self.analyze_expression(value, environment)?;
            require_type(
                &self.source,
                value.span,
                &CompilerType::String,
                &value.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringUtf8ByteCount(Box::new(value)),
                value_type: CompilerType::Int,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(namespace),
            Expression::Identifier(vocabulary),
            Expression::Identifier(code),
        ] = items
            && let Some(value) = arithmetic_error_code(
                self.source.slice(*namespace),
                self.source.slice(*vocabulary),
                self.source.slice(*code),
            )
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ErrorCode(value),
                value_type: CompilerType::ErrorCode,
                int_range: None,
                rational_value: None,
                span,
            });
        }
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
                    let value_type = if self.source.slice(*domain) == "Nat" {
                        CompilerType::Nat
                    } else {
                        CompilerType::Int
                    };
                    Ok(CompilerExpression {
                        kind: CompilerExpressionKind::Int(value.clone()),
                        value_type,
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
        if let [Expression::Identifier(constructor), argument] = items {
            match self.source.slice(*constructor) {
                "Int" => return self.analyze_int_constructor(argument, span, environment),
                "Nat" => return self.analyze_nat_constructor(argument, span, environment),
                _ => {}
            }
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
        if let [error, Expression::Identifier(field)] = items
            && matches!(self.source.slice(*field), "code" | "domain")
        {
            return self.analyze_error_field(error, *field, span, environment);
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

    fn finish_optional_none(
        &self,
        payload: CompilerType,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_optional_payload(&self.source, span, &payload)?;
        Ok(CompilerExpression {
            value_type: CompilerType::Optional(Box::new(payload)),
            kind: CompilerExpressionKind::OptionalNone,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_error_field(
        &mut self,
        error: &Expression,
        field: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let error = self.analyze_expression(error, environment)?;
        if !matches!(
            &error.value_type,
            CompilerType::Error | CompilerType::Result(_)
        ) {
            return Err(source_diagnostic(
                &self.source,
                "E-TYPE-MISMATCH",
                error.span,
                format!(
                    "expected Error or Result, found {}",
                    error.value_type.name()
                ),
            ));
        }
        let (field, value_type) = match self.source.slice(field) {
            "code" => (CompilerErrorField::Code, CompilerType::ErrorCode),
            "domain" => (CompilerErrorField::Domain, CompilerType::ErrorDomain),
            _ => unreachable!("implemented Error field spelling selected above"),
        };
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ErrorField {
                error: Box::new(error),
                field,
            },
            value_type,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_int_constructor(
        &mut self,
        argument: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let value = self.analyze_expression(argument, environment)?;
        self.finish_int_conversion(value, span, argument.span())
    }

    fn finish_int_conversion(
        &self,
        value: CompilerExpression,
        span: Span,
        error_span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        if value.value_type == CompilerType::Int {
            return Ok(value);
        }
        require_type(
            &self.source,
            value.span,
            &CompilerType::Rational,
            &value.value_type,
        )?;
        if let Some(rational) = &value.rational_value {
            if rational.denom() != &BigInt::from(1) && compiler_expression_is_closed(&value) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-RATIONAL-NOT-EXACT-INT",
                    error_span,
                    format!(
                        "exact Rational operand has denominator {}, so Int cannot represent it",
                        rational.denom()
                    ),
                ));
            }
            if rational.denom() == &BigInt::from(1) {
                let numerator = rational.numer().clone();
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::RationalToInt(Box::new(value)),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(numerator)),
                    rational_value: None,
                    span,
                });
            }
        }
        Ok(Self::finish_validation(
            CompilerValidation::RationalToInt,
            value,
            CompilerType::Int,
            span,
            error_span,
        ))
    }

    fn analyze_nat_constructor(
        &mut self,
        argument: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let value = self.analyze_expression(argument, environment)?;
        self.finish_nat_conversion(value, span, argument.span())
    }

    fn finish_nat_conversion(
        &self,
        value: CompilerExpression,
        span: Span,
        error_span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_type(
            &self.source,
            value.span,
            &CompilerType::Int,
            &value.value_type,
        )?;
        if value
            .int_range
            .as_ref()
            .is_some_and(|range| range.lower >= BigInt::from(0))
        {
            let int_range = value.int_range.clone();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(value)),
                value_type: CompilerType::Nat,
                int_range,
                rational_value: None,
                span,
            });
        }
        if value
            .int_range
            .as_ref()
            .is_some_and(|range| range.upper < BigInt::from(0))
            && compiler_expression_is_closed(&value)
        {
            return Err(source_diagnostic(
                &self.source,
                "E-NAT-OUT-OF-RANGE",
                error_span,
                "a negative Int is outside the Nat constraint",
            ));
        }
        Ok(Self::finish_validation(
            CompilerValidation::IntToNat,
            value,
            CompilerType::Nat,
            span,
            error_span,
        ))
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
            if is_proven_zero_numeric(&denominator) {
                if compiler_expression_is_closed(&denominator) {
                    if is_proven_zero_numeric(&numerator) {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-INDETERMINATE-RATIONAL",
                            argument.span(),
                            "Rational (0, 0) does not determine one numeric value",
                        ));
                    }
                    return Err(division_by_zero(&self.source, argument.span()));
                }
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Fallible {
                        operation: CompilerFallible::RationalConstruct,
                        left: Box::new(numerator),
                        right: Box::new(denominator),
                        error_span: argument.span(),
                    },
                    value_type: CompilerType::Result(Box::new(CompilerType::Rational)),
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            if !is_proven_nonzero_numeric(&denominator) {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Fallible {
                        operation: CompilerFallible::RationalConstruct,
                        left: Box::new(numerator),
                        right: Box::new(denominator),
                        error_span: argument.span(),
                    },
                    value_type: CompilerType::Result(Box::new(CompilerType::Rational)),
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
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
        if operation == "empty?" && operand.value_type == CompilerType::String {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringEmptyPredicate(Box::new(operand)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        let CompilerType::Range(endpoint) = &operand.value_type else {
            if operation == "empty?" {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "empty? for this value type",
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
            let result_type = if operation == CompilerBinary::Modulo {
                CompilerType::Int
            } else {
                CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int])
            };
            if is_proven_zero_numeric(&right_value) {
                if compiler_expression_is_closed(&right_value) {
                    return Err(division_by_zero(&self.source, right_value.span));
                }
                return Ok(Self::finish_fallible_binary(
                    if operation == CompilerBinary::Modulo {
                        CompilerFallible::IntModulo
                    } else {
                        CompilerFallible::IntQuotientModulo
                    },
                    left_value,
                    right_value,
                    result_type,
                    span,
                    right.span(),
                ));
            }
            if !is_proven_nonzero_numeric(&right_value) {
                return Ok(Self::finish_fallible_binary(
                    if operation == CompilerBinary::Modulo {
                        CompilerFallible::IntModulo
                    } else {
                        CompilerFallible::IntQuotientModulo
                    },
                    left_value,
                    right_value,
                    result_type,
                    span,
                    right.span(),
                ));
            }
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
            let equatable = matches!(
                &left_value.value_type,
                CompilerType::Boolean
                    | CompilerType::Unit
                    | CompilerType::Completed
                    | CompilerType::Comparison
                    | CompilerType::ErrorCode
                    | CompilerType::String
                    | CompilerType::Enum(_)
            ) || matches!(
                &left_value.value_type,
                CompilerType::Optional(payload)
                    if matches!(payload.as_ref(), CompilerType::Int | CompilerType::String)
            );
            if !equatable {
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
        let both_int = left_value.value_type == CompilerType::Int
            && right_value.value_type == CompilerType::Int;
        let rational_result = operation == CompilerBinary::Divide
            || left_value.value_type == CompilerType::Rational
            || right_value.value_type == CompilerType::Rational;
        if rational_result {
            left_value = into_rational(left_value);
            right_value = into_rational(right_value);
        }
        if operation == CompilerBinary::Divide && is_proven_zero_numeric(&right_value) {
            if compiler_expression_is_closed(&right_value) {
                return Err(division_by_zero(&self.source, right_value.span));
            }
            if both_int {
                return Err(unsupported(
                    &self.source,
                    span,
                    "dynamic Int division Result",
                ));
            }
            return Ok(Self::finish_fallible_binary(
                CompilerFallible::RationalDivide,
                left_value,
                right_value,
                CompilerType::Rational,
                span,
                right.span(),
            ));
        }
        if operation == CompilerBinary::Divide && !is_proven_nonzero_numeric(&right_value) {
            if both_int {
                return Err(unsupported(
                    &self.source,
                    span,
                    "dynamic Int division Result",
                ));
            }
            return Ok(Self::finish_fallible_binary(
                CompilerFallible::RationalDivide,
                left_value,
                right_value,
                CompilerType::Rational,
                span,
                right.span(),
            ));
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
        let exponent = exact_int(&right);
        if left.value_type == CompilerType::Int {
            let Some(ref exponent) = exponent else {
                return Err(unsupported(
                    &self.source,
                    right.span,
                    "Int power with an exponent not proven to satisfy Nat",
                ));
            };
            if exponent < &BigInt::from(0) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-TYPE-MISMATCH",
                    right.span,
                    "an Int exponent must satisfy Nat",
                ));
            }
        }
        let can_fail = left.value_type == CompilerType::Rational
            && !is_proven_nonzero_numeric(&left)
            && exponent
                .as_ref()
                .is_none_or(|value| value < &BigInt::from(0));
        if can_fail && is_proven_zero_numeric(&left) && compiler_expression_is_closed(&left) {
            return Err(division_by_zero(&self.source, left.span));
        }
        if can_fail {
            let error_span = left.span;
            return Ok(Self::finish_fallible_binary(
                CompilerFallible::RationalPower,
                left,
                right,
                CompilerType::Rational,
                span,
                error_span,
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

    fn finish_fallible_binary(
        operation: CompilerFallible,
        left: CompilerExpression,
        right: CompilerExpression,
        success_type: CompilerType,
        span: Span,
        error_span: Span,
    ) -> CompilerExpression {
        CompilerExpression {
            kind: CompilerExpressionKind::Fallible {
                operation,
                left: Box::new(left),
                right: Box::new(right),
                error_span,
            },
            value_type: CompilerType::Result(Box::new(success_type)),
            int_range: None,
            rational_value: None,
            span,
        }
    }

    fn finish_validation(
        operation: CompilerValidation,
        value: CompilerExpression,
        success_type: CompilerType,
        span: Span,
        error_span: Span,
    ) -> CompilerExpression {
        CompilerExpression {
            kind: CompilerExpressionKind::Validate {
                operation,
                value: Box::new(value),
                error_span,
            },
            value_type: CompilerType::Result(Box::new(success_type)),
            int_range: None,
            rational_value: None,
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
            let name = self.source.slice(*name);
            (!environment.contains_key(name) && self.functions.contains_key(name))
                .then_some((index, name.to_owned()))
        });
        let Some((function_index, function_name)) = function else {
            return Err(unsupported(&self.source, span, "application"));
        };
        let declarations = self
            .functions
            .get(&function_name)
            .expect("selected overload set exists")
            .clone();
        let argument_sources = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (index != function_index).then_some(item))
            .collect::<Vec<_>>();
        let argument_sources = if let [Expression::Product { fields, .. }] =
            argument_sources.as_slice()
            && fields.iter().all(|field| field.label.is_none())
        {
            fields.iter().map(|field| &field.value).collect::<Vec<_>>()
        } else {
            argument_sources
        };
        let arguments = argument_sources
            .iter()
            .map(|argument| self.analyze_expression(argument, environment))
            .collect::<Result<Vec<_>, _>>()?;

        let mut selected = None;
        for declaration in declarations
            .iter()
            .filter(|declaration| !self.static_context || declaration.is_static)
        {
            let candidate_arguments = if declaration.parameters.is_empty()
                && matches!(argument_sources.as_slice(), [Expression::Unit(_)])
            {
                &[][..]
            } else {
                arguments.as_slice()
            };
            if candidate_arguments.len() != declaration.parameters.len() {
                continue;
            }
            let mut adapted = Vec::with_capacity(candidate_arguments.len());
            for (parameter, argument) in declaration.parameters.iter().zip(candidate_arguments) {
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
                let expected = self.parse_classifier(parameter.classifier)?;
                let Some(argument) = adapt_call_argument(&expected, argument) else {
                    adapted.clear();
                    break;
                };
                adapted.push(argument);
            }
            if adapted.len() == declaration.parameters.len() {
                selected = Some((declaration.clone(), adapted));
                break;
            }
        }
        let Some((declaration, arguments)) = selected else {
            let actual = arguments
                .iter()
                .map(|argument| argument.value_type.name())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(source_diagnostic(
                &self.source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("no overload of `{function_name}` accepts ({actual}) in this context"),
            ));
        };
        let identity = function_overload_identity(&self.source, &function_name, &declaration);
        if !self.active_calls.insert(identity.clone()) {
            return Err(unsupported(&self.source, span, "recursive function call"));
        }
        let result = self.instantiate_function(&function_name, &declaration, &arguments);
        self.active_calls.remove(&identity);
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
            let expected = self.parse_classifier(parameter.classifier)?;
            require_same_type(
                &self.source,
                parameter.classifier,
                &expected,
                &argument.value_type,
            )?;
            if !expected.machine_scalar() || !compiler_abi_type_supported(&expected) {
                return Err(unsupported(
                    &self.source,
                    parameter.classifier,
                    "non-scalar function parameter",
                ));
            }
            let name = self.source.slice(parameter.name).to_owned();
            let discarded = name == "_";
            if !discarded {
                environment.insert(
                    name.clone(),
                    BindingFacts {
                        value_type: expected.clone(),
                        int_range: argument.int_range.clone(),
                        rational_value: argument.rational_value.clone(),
                    },
                );
            }
            parameters.push(CompilerParameter {
                name,
                discarded,
                value_type: expected,
                int_range: argument.int_range.clone(),
                span: parameter.name,
            });
        }
        let result_type = self.parse_classifier(declaration.result)?;
        if !result_type.machine_scalar() || !compiler_abi_type_supported(&result_type) {
            return Err(unsupported(
                &self.source,
                declaration.result,
                "non-scalar function result",
            ));
        }
        let previous_static_context = self.static_context;
        self.static_context = declaration.is_static;
        let body = self.analyze_block(
            &declaration.body,
            &mut environment,
            BlockKind::Function,
            Some(&result_type),
        );
        self.static_context = previous_static_context;
        let mut body = body?;
        if let CompilerType::Result(success_type) = &result_type
            && body.result.value_type == **success_type
        {
            let result = body.result;
            let span = result.span;
            body.result = CompilerExpression {
                kind: CompilerExpressionKind::ResultSuccess(Box::new(result)),
                value_type: result_type.clone(),
                int_range: None,
                rational_value: None,
                span,
            };
        } else {
            require_same_type(
                &self.source,
                declaration.result,
                &result_type,
                &body.result.value_type,
            )?;
        }
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
            is_static: declaration.is_static,
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
            CompilerType::Enum(enumeration) => {
                let enumeration = enumeration.clone();
                self.analyze_enum_decision(subject, &enumeration, rules, span, environment)
            }
            CompilerType::Result(success) => {
                let success = success.as_ref().clone();
                self.analyze_result_decision(subject, &success, rules, span, environment)
            }
            CompilerType::Optional(payload) => {
                let payload = payload.as_ref().clone();
                self.analyze_optional_decision(subject, &payload, rules, span, environment)
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

    fn analyze_optional_decision(
        &mut self,
        subject: CompilerExpression,
        payload_type: &CompilerType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut some = None;
        let mut none = None;
        let mut otherwise = None;
        for rule in rules {
            if otherwise.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-UNREACHABLE-DECISION-RULE",
                    rule.span,
                    "an Optional rule cannot follow otherwise",
                ));
            }
            match rule.matcher {
                DecisionMatcher::Optional {
                    some: true,
                    binding: Some(binding),
                    ..
                } if some.is_none() => {
                    let name = self.source.slice(binding).to_owned();
                    let branch =
                        decision_binding_environment(environment, &name, payload_type.clone());
                    some = Some((
                        name,
                        binding,
                        self.analyze_expression(&rule.action, &branch)?,
                    ));
                }
                DecisionMatcher::Optional {
                    some: false,
                    binding: None,
                    ..
                } if none.is_none() => {
                    none = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Otherwise(_) => {
                    otherwise = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Optional { .. } => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-DUPLICATE-DECISION-RULE",
                        rule.span,
                        "an Optional alternative appears more than once",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "Optional decision matcher",
                    ));
                }
            }
        }
        let (some_binding, some_action) = if let Some((name, binding, action)) = some {
            (Some((name, binding)), action)
        } else if let Some(action) = otherwise.clone() {
            (None, action)
        } else {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Optional decision does not cover Some",
            ));
        };
        let none_action = none.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Optional decision does not cover None",
            )
        })?;
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&some_action, &none_action], span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::OptionalDecision {
                subject: Box::new(subject),
                some_binding,
                some_action: Box::new(some_action),
                none_action: Box::new(none_action),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn analyze_enum_decision(
        &mut self,
        subject: CompilerExpression,
        enumeration: &CompilerEnumType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut lowered = Vec::new();
        let mut seen = BTreeSet::new();
        let mut otherwise = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Identifier(matcher) if otherwise.is_none() => {
                    let label = self.source.slice(matcher);
                    let Some(value) = enumeration
                        .alternatives
                        .iter()
                        .position(|alternative| alternative == label)
                    else {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNKNOWN-ENUM-ALTERNATIVE",
                            matcher,
                            format!("`{label}` is not an alternative of `{}`", enumeration.name),
                        ));
                    };
                    let value =
                        u32::try_from(value).expect("enum declaration already fits the native tag");
                    if !seen.insert(value) {
                        // The earlier source-ordered matcher always selects this
                        // alternative, so the repeated action is unreachable.
                        continue;
                    }
                    lowered.push(CompilerEnumRule {
                        value,
                        action: self.analyze_expression(&rule.action, environment)?,
                        span: rule.span,
                    });
                }
                DecisionMatcher::Otherwise(_) if otherwise.is_none() => {
                    otherwise = Some(Box::new(
                        self.analyze_expression(&rule.action, environment)?,
                    ));
                }
                _ if otherwise.is_some() => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNREACHABLE-DECISION-RULE",
                        rule.span,
                        "an Enum rule cannot follow otherwise",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "Enum decision matcher",
                    ));
                }
            }
        }
        if otherwise.is_none() && seen.len() != enumeration.alternatives.len() {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                format!(
                    "decision does not cover every `{}` alternative",
                    enumeration.name
                ),
            ));
        }
        let mut actions = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
        if let Some(action) = &otherwise {
            actions.push(action);
        }
        let (value_type, int_range, rational_value) = self.decision_facts(&actions, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::EnumDecision {
                subject: Box::new(subject),
                rules: lowered,
                otherwise,
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Result binding, reachability, and completeness checks stay adjacent.
    fn analyze_result_decision(
        &mut self,
        subject: CompilerExpression,
        success_type: &CompilerType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut ok = None;
        let mut error_codes = Vec::new();
        let mut seen_codes = BTreeSet::new();
        let mut error_fallback = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Result {
                    error: false,
                    binding,
                    ..
                } if ok.is_none() => {
                    let name = self.source.slice(binding).to_owned();
                    let branch =
                        decision_binding_environment(environment, &name, success_type.clone());
                    ok = Some((
                        name,
                        binding,
                        self.analyze_expression(&rule.action, &branch)?,
                    ));
                }
                DecisionMatcher::Result {
                    error: true,
                    binding,
                    ..
                } if error_fallback.is_none() => {
                    let name = self.source.slice(binding).to_owned();
                    let branch =
                        decision_binding_environment(environment, &name, CompilerType::Error);
                    error_fallback = Some((
                        name,
                        binding,
                        Box::new(self.analyze_expression(&rule.action, &branch)?),
                    ));
                }
                DecisionMatcher::ErrorCode {
                    namespace,
                    vocabulary,
                    code,
                    ..
                } => {
                    if error_fallback.is_some() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNREACHABLE-ERROR-CODE-PATTERN",
                            rule.span,
                            "qualified Error-code pattern is unreachable after Error fallback",
                        ));
                    }
                    let code_value = arithmetic_error_code(
                        self.source.slice(namespace),
                        self.source.slice(vocabulary),
                        self.source.slice(code),
                    )
                    .ok_or_else(|| {
                        source_diagnostic(
                            &self.source,
                            "E-UNKNOWN-ERROR-CODE",
                            code,
                            "the compiler subset requires a qualified arithmetic Error code",
                        )
                    })?;
                    if !seen_codes.insert(code_value) {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-DUPLICATE-ERROR-CODE-PATTERN",
                            rule.span,
                            "an arithmetic Error code is matched more than once",
                        ));
                    }
                    error_codes.push(CompilerErrorCodeRule {
                        code: code_value,
                        action: self.analyze_expression(&rule.action, environment)?,
                        span: rule.span,
                    });
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "Result decision matcher",
                    ));
                }
            }
        }
        let (ok_binding, ok_binding_span, ok_action) = ok.ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Result decision does not cover Ok",
            )
        })?;
        if error_fallback.is_none() && seen_codes != BTreeSet::from([0, 1, 2, 3]) {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-ERROR-CODE-DECISION",
                span,
                "Result decision requires Error fallback or every arithmetic Error code",
            ));
        }
        let mut actions = vec![&ok_action];
        actions.extend(error_codes.iter().map(|rule| &rule.action));
        if let Some((_, _, action)) = &error_fallback {
            actions.push(action);
        }
        let (value_type, int_range, rational_value) = self.decision_facts(&actions, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ResultDecision {
                subject: Box::new(subject),
                ok_binding,
                ok_binding_span,
                ok_action: Box::new(ok_action),
                error_codes,
                error_fallback,
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
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

fn compact_classifier(classifier: &str) -> String {
    classifier
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn function_overload_identity(
    source: &SourceText,
    name: &str,
    declaration: &FunctionSource,
) -> String {
    let inputs = declaration
        .parameters
        .iter()
        .map(|parameter| compact_classifier(source.slice(parameter.classifier)))
        .collect::<Vec<_>>()
        .join(",");
    let staticness = if declaration.is_static {
        "static"
    } else {
        "ordinary"
    };
    format!("{name}:{staticness}({inputs})")
}

fn parse_compact_classifier(classifier: &str) -> Option<CompilerType> {
    if let Some(payload) = classifier.strip_prefix("Optional") {
        return Some(CompilerType::Optional(Box::new(parse_compact_classifier(
            payload,
        )?)));
    }
    if let Some(success_and_codes) = classifier
        .strip_prefix("Result(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let (success, codes) = split_classifier_once(success_and_codes)?;
        if codes != "langarithmeticArithmeticErrorCode" {
            return None;
        }
        return Some(CompilerType::Result(Box::new(parse_compact_classifier(
            success,
        )?)));
    }
    if let Some(fields) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        let fields = split_classifier_fields(fields)?;
        if fields.len() > 1 {
            return Some(CompilerType::Tuple(
                fields
                    .into_iter()
                    .map(parse_compact_classifier)
                    .collect::<Option<Vec<_>>>()?,
            ));
        }
    }
    match classifier {
        "RangeInt" => Some(CompilerType::Range(Box::new(CompilerType::Int))),
        "RangeRational" => Some(CompilerType::Range(Box::new(CompilerType::Rational))),
        "Unit" => Some(CompilerType::Unit),
        "Completed" => Some(CompilerType::Completed),
        "Boolean" => Some(CompilerType::Boolean),
        "Int" => Some(CompilerType::Int),
        "Nat" => Some(CompilerType::Nat),
        "Rational" => Some(CompilerType::Rational),
        "Comparison" => Some(CompilerType::Comparison),
        "Error" => Some(CompilerType::Error),
        "ErrorCode" | "langarithmeticArithmeticErrorCode" => Some(CompilerType::ErrorCode),
        "ErrorDomain" => Some(CompilerType::ErrorDomain),
        "String" => Some(CompilerType::String),
        _ => None,
    }
}

fn split_classifier_once(classifier: &str) -> Option<(&str, &str)> {
    let mut depth = 0_u32;
    for (index, byte) in classifier.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.checked_sub(1)?,
            b',' if depth == 0 => return Some((&classifier[..index], &classifier[index + 1..])),
            _ => {}
        }
    }
    None
}

fn split_classifier_fields(classifier: &str) -> Option<Vec<&str>> {
    let mut fields = Vec::new();
    let mut remaining = classifier;
    while let Some((field, rest)) = split_classifier_once(remaining) {
        fields.push(field);
        remaining = rest;
    }
    fields.push(remaining);
    fields
        .iter()
        .all(|field| !field.is_empty())
        .then_some(fields)
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

fn compiler_abi_type_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Optional(payload) => {
            matches!(payload.as_ref(), CompilerType::Int | CompilerType::String)
        }
        CompilerType::Result(success) => {
            matches!(
                success.as_ref(),
                CompilerType::Int
                    | CompilerType::Nat
                    | CompilerType::Rational
                    | CompilerType::String
            ) || matches!(
                success.as_ref(),
                CompilerType::Tuple(fields)
                    if matches!(fields.as_slice(), [CompilerType::Int, CompilerType::Int])
            )
        }
        _ => true,
    }
}

fn decision_binding_environment(
    environment: &BTreeMap<String, BindingFacts>,
    name: &str,
    value_type: CompilerType,
) -> BTreeMap<String, BindingFacts> {
    let mut branch = environment.clone();
    branch.insert(
        name.to_owned(),
        BindingFacts {
            value_type,
            int_range: None,
            rational_value: None,
        },
    );
    branch
}

fn arithmetic_error_code(namespace: &str, vocabulary: &str, code: &str) -> Option<u32> {
    if namespace != "lang" || vocabulary != "arithmetic" {
        return None;
    }
    match code {
        "out-of-range" => Some(0),
        "not-representable" => Some(1),
        "division-by-zero" => Some(2),
        "indeterminate" => Some(3),
        _ => None,
    }
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

fn require_optional_payload(
    source: &SourceText,
    span: Span,
    value_type: &CompilerType,
) -> Result<(), Diagnostic> {
    if matches!(value_type, CompilerType::Int | CompilerType::String) {
        Ok(())
    } else {
        Err(unsupported(source, span, "Optional payload type"))
    }
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

fn compiler_expression_is_closed(expression: &CompilerExpression) -> bool {
    compiler_expression_is_closed_with(expression, &BTreeSet::new())
}

fn compiler_expression_is_closed_with(
    expression: &CompilerExpression,
    bound: &BTreeSet<String>,
) -> bool {
    match &expression.kind {
        CompilerExpressionKind::Unit
        | CompilerExpressionKind::Completed
        | CompilerExpressionKind::Boolean(_)
        | CompilerExpressionKind::Int(_)
        | CompilerExpressionKind::Rational(_)
        | CompilerExpressionKind::String(_)
        | CompilerExpressionKind::StringEmpty
        | CompilerExpressionKind::ErrorCode(_)
        | CompilerExpressionKind::Enum(_)
        | CompilerExpressionKind::OptionalNone => true,
        CompilerExpressionKind::Tuple(values) => values
            .iter()
            .all(|value| compiler_expression_is_closed_with(value, bound)),
        CompilerExpressionKind::Block(block) => compiler_block_is_closed(block, bound),
        CompilerExpressionKind::Local(name) => bound.contains(name),
        CompilerExpressionKind::Call { .. }
        | CompilerExpressionKind::Fallible { .. }
        | CompilerExpressionKind::Validate { .. }
        | CompilerExpressionKind::ResultDecision { .. }
        | CompilerExpressionKind::OptionalDecision { .. } => false,
        CompilerExpressionKind::Negate(value)
        | CompilerExpressionKind::Absolute(value)
        | CompilerExpressionKind::IntToRational(value)
        | CompilerExpressionKind::RationalToInt(value)
        | CompilerExpressionKind::IntToNat(value)
        | CompilerExpressionKind::ResultSuccess(value)
        | CompilerExpressionKind::ResultProject(value)
        | CompilerExpressionKind::OptionalSome(value)
        | CompilerExpressionKind::StringEmptyPredicate(value)
        | CompilerExpressionKind::StringUtf8ByteCount(value)
        | CompilerExpressionKind::ErrorField { error: value, .. }
        | CompilerExpressionKind::RangeLower(value)
        | CompilerExpressionKind::RangeUpper(value)
        | CompilerExpressionKind::RangeLowerInclusive(value)
        | CompilerExpressionKind::RangeUpperInclusive(value)
        | CompilerExpressionKind::RangeEmpty(value)
        | CompilerExpressionKind::Not(value) => compiler_expression_is_closed_with(value, bound),
        CompilerExpressionKind::RationalConstruct {
            numerator,
            denominator,
        } => {
            compiler_expression_is_closed_with(numerator, bound)
                && compiler_expression_is_closed_with(denominator, bound)
        }
        CompilerExpressionKind::StringConcat { left, right }
        | CompilerExpressionKind::Binary { left, right, .. } => {
            compiler_expression_is_closed_with(left, bound)
                && compiler_expression_is_closed_with(right, bound)
        }
        CompilerExpressionKind::BooleanDecision {
            subject,
            when_true,
            when_false,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && compiler_expression_is_closed_with(when_true, bound)
                && compiler_expression_is_closed_with(when_false, bound)
        }
        CompilerExpressionKind::OrderedComparisonDecision {
            subject,
            rules,
            otherwise,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && rules.iter().all(|rule| {
                    compiler_expression_is_closed_with(&rule.operand, bound)
                        && compiler_expression_is_closed_with(&rule.action, bound)
                })
                && compiler_expression_is_closed_with(otherwise, bound)
        }
        CompilerExpressionKind::ComparisonValueDecision {
            subject,
            when_less,
            when_equal,
            when_greater,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && compiler_expression_is_closed_with(when_less, bound)
                && compiler_expression_is_closed_with(when_equal, bound)
                && compiler_expression_is_closed_with(when_greater, bound)
        }
        CompilerExpressionKind::EnumDecision {
            subject,
            rules,
            otherwise,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && rules
                    .iter()
                    .all(|rule| compiler_expression_is_closed_with(&rule.action, bound))
                && otherwise
                    .as_deref()
                    .is_none_or(|value| compiler_expression_is_closed_with(value, bound))
        }
    }
}

fn compiler_block_is_closed(block: &CompilerBlock, outer: &BTreeSet<String>) -> bool {
    let mut bound = outer.clone();
    for statement in &block.statements {
        match statement {
            CompilerStatement::Binding(binding) => {
                if !compiler_expression_is_closed_with(&binding.value, &bound) {
                    return false;
                }
                bound.insert(binding.name.clone());
            }
            CompilerStatement::Discard(expression) => {
                if !compiler_expression_is_closed_with(expression, &bound) {
                    return false;
                }
            }
        }
    }
    compiler_expression_is_closed_with(&block.result, &bound)
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

fn adapt_call_argument(
    expected: &CompilerType,
    argument: &CompilerExpression,
) -> Option<CompilerExpression> {
    if expected == &argument.value_type {
        return Some(argument.clone());
    }
    match (expected, &argument.value_type) {
        (CompilerType::Int, CompilerType::Nat) => {
            let mut value = argument.clone();
            value.value_type = CompilerType::Int;
            Some(value)
        }
        (CompilerType::Rational, CompilerType::Int | CompilerType::Nat) => {
            Some(into_rational(argument.clone()))
        }
        (CompilerType::Int, CompilerType::Rational) => {
            let rational = argument.rational_value.as_ref()?;
            (rational.denom() == &BigInt::from(1)).then(|| CompilerExpression {
                kind: CompilerExpressionKind::RationalToInt(Box::new(argument.clone())),
                value_type: CompilerType::Int,
                int_range: Some(IntRange::exact(rational.numer().clone())),
                rational_value: None,
                span: argument.span,
            })
        }
        (CompilerType::Nat, CompilerType::Int) => argument
            .int_range
            .as_ref()
            .is_some_and(|range| range.lower >= BigInt::from(0))
            .then(|| CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(argument.clone())),
                value_type: CompilerType::Nat,
                int_range: argument.int_range.clone(),
                rational_value: None,
                span: argument.span,
            }),
        (CompilerType::Nat, CompilerType::Rational) => {
            let rational = argument.rational_value.as_ref()?;
            (rational.denom() == &BigInt::from(1) && rational.numer() >= &BigInt::from(0)).then(
                || {
                    let integer = CompilerExpression {
                        kind: CompilerExpressionKind::RationalToInt(Box::new(argument.clone())),
                        value_type: CompilerType::Int,
                        int_range: Some(IntRange::exact(rational.numer().clone())),
                        rational_value: None,
                        span: argument.span,
                    };
                    CompilerExpression {
                        kind: CompilerExpressionKind::IntToNat(Box::new(integer)),
                        value_type: CompilerType::Nat,
                        int_range: Some(IntRange::exact(rational.numer().clone())),
                        rational_value: None,
                        span: argument.span,
                    }
                },
            )
        }
        _ => None,
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
    fn models_dynamic_arithmetic_results_without_reclassifying_parameter_errors_as_static() {
        // TOPAL-COMPILER-RESULT-001
        let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  Rational (numerator, denominator)\nmodulo is fn (left : Int, right : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  left % right\n(1.0 divide 2.0, 1.0 divide 0.0, 1 ratio 0, 17 modulo 0)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Result (Rational, lang arithmetic ArithmeticErrorCode), Result (Rational, lang arithmetic ArithmeticErrorCode), Result (Rational, lang arithmetic ArithmeticErrorCode), Result (Int, lang arithmetic ArithmeticErrorCode))"
        );
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::Fallible { .. }
        )));
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::ResultSuccess(_)
        )));
    }

    #[test]
    fn models_optional_construction_boundaries_decisions_and_equality() {
        // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-OPTIONAL-CONTEXT-001,
        // TOPAL-TYPE-OPTIONAL-BOUNDARY-001, TOPAL-DECISION-OPTIONAL-001,
        // TOPAL-TYPE-OPTIONAL-EQUALITY-001
        let source = "use language (version is v0.1)\nmissing : Optional Int is None\npreserve is fn (candidate : Optional Int) -> Optional Int\n  candidate\nabsent is fn () -> Optional Int\n  None\ndescribe is fn (candidate : Optional Int) -> String\n  candidate\n    Some payload then \"present\"\n    None then \"absent\"\n(Some 42, Some \"present\", None Int, None String, missing, preserve (Some 7), preserve (None Int), absent (), describe (Some 7), describe missing, (None Int) = (None Int), (Some 7) != (Some 8))\n";
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.functions.iter().any(|function| {
            function.source_name == "preserve"
                && function.parameters[0].value_type
                    == CompilerType::Optional(Box::new(CompilerType::Int))
                && function.result_type == CompilerType::Optional(Box::new(CompilerType::Int))
        }));
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::OptionalDecision { .. }
        )));
        assert!(matches!(
            program.main.result.kind,
            CompilerExpressionKind::Tuple(_)
        ));
    }

    #[test]
    fn models_prospective_utf8_string_byte_counts() {
        // TOPAL-TYPE-CALL-001, TOPAL-STRING-UTF8-BYTE-COUNT-001
        let source = include_str!("../../../examples/language/string-utf8-byte-count.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program
                .main
                .statements
                .iter()
                .filter(|statement| matches!(
                    statement,
                    CompilerStatement::Binding(CompilerBinding {
                        value: CompilerExpression {
                            kind: CompilerExpressionKind::StringUtf8ByteCount(_),
                            ..
                        },
                        ..
                    })
                ))
                .count(),
            3
        );
        assert_eq!(program.main.result.value_type.name(), "(Int, Int, Int)");
    }

    #[test]
    fn models_exact_string_and_derived_optional_string_equality() {
        // TOPAL-TYPE-EQUALITY-001, TOPAL-TYPE-OPTIONAL-EQUALITY-001
        let source = include_str!("../../../examples/language/string-exact-equality.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected equality result tuple")
        };
        assert!(values.iter().all(|value| matches!(
            value.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Equal | CompilerBinary::NotEqual,
                ..
            }
        )));
    }

    #[test]
    fn models_string_construction_concatenation_and_emptiness() {
        // TOPAL-STRING-EMPTY-001, TOPAL-STRING-LITERAL-COMPOSE-001,
        // TOPAL-STRING-CONCAT-001, TOPAL-STRING-EMPTY-PREDICATE-001
        let source = include_str!("../../../examples/language/string-construction.t");
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::StringEmpty,
                    ..
                },
                ..
            })
        )));
        assert_eq!(
            program
                .main
                .statements
                .iter()
                .filter(|statement| matches!(
                    statement,
                    CompilerStatement::Binding(CompilerBinding {
                        value: CompilerExpression {
                            kind: CompilerExpressionKind::StringConcat { .. },
                            ..
                        },
                        ..
                    })
                ))
                .count(),
            4
        );
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::String(value),
                    ..
                },
                ..
            }) if value == "adjacent literals"
        )));
        assert_eq!(
            program.main.result.value_type.name(),
            "(String, Boolean, Boolean, Boolean, String, String, String, String, String)"
        );
    }

    #[test]
    fn rejects_equality_between_distinct_optional_classifiers() {
        // TOPAL-TYPE-OPTIONAL-EQUALITY-001
        let source = "use language (version is v0.1)\n(None Int) = (None String)\n";
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
    }

    #[test]
    fn models_optional_otherwise_as_the_missing_alternative_without_a_payload_binding() {
        // TOPAL-DECISION-OPTIONAL-001
        let source = "use language (version is v0.1)\ndescribe is fn (candidate : Optional Int) -> Int\n  candidate\n    None then 0\n    otherwise 1\n(describe (Some 42), describe (None Int))\n";
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "describe")
            .unwrap();
        let CompilerExpressionKind::OptionalDecision { some_binding, .. } =
            &function.body.result.kind
        else {
            panic!("expected Optional decision")
        };
        assert!(some_binding.is_none());
    }

    #[test]
    fn models_exact_validation_and_contextual_result_projection() {
        // TOPAL-COMPILER-RESULT-001
        let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nincrement is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  quotient : Rational is 1.0 divide denominator\n  quotient + 1.0\nas-int is fn (value : Rational) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  Int value\nas-nat is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)\n  Nat value\n(increment 0.0, as-int 1.5, as-nat -1)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.functions.iter().any(|function| {
            function.body.statements.iter().any(|statement| {
                matches!(
                    statement,
                    CompilerStatement::Binding(CompilerBinding {
                        value: CompilerExpression {
                            kind: CompilerExpressionKind::ResultProject(_),
                            ..
                        },
                        ..
                    })
                )
            })
        }));

        let repeated = "use language (version is v0.1)\nColor is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"first\"\n    Red then 42\n    Green then \"green\"\nname Red\n";
        let repeated = analyze_for_compiler(repeated).unwrap();
        assert!(repeated.functions.iter().any(|function| matches!(
            &function.body.result.kind,
            CompilerExpressionKind::EnumDecision { rules, .. } if rules.len() == 2
        )));
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::Validate { .. }
        )));
    }

    #[test]
    fn models_result_decisions_and_structured_error_observation() {
        // TOPAL-DECISION-RESULT-001, TOPAL-DECISION-ERROR-CODE-001,
        // TOPAL-ERROR-FIELD-001
        let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error (code is lang arithmetic division-by-zero) then \"zero\"\n    Error problem then \"other\"\nexhaustive is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error (code is lang arithmetic out-of-range) then \"range\"\n    Error (code is lang arithmetic not-representable) then \"representation\"\n    Error (code is lang arithmetic division-by-zero) then \"zero\"\n    Error (code is lang arithmetic indeterminate) then \"indeterminate\"\nproblem is 1.0 divide 0.0\n(describe 0.0, exhaustive 0.0, problem code, problem domain)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(String, String, lang arithmetic ArithmeticErrorCode, ErrorDomain)"
        );
        assert!(program.functions.iter().any(|function| matches!(
            &function.body.result.kind,
            CompilerExpressionKind::ResultDecision {
                error_codes,
                error_fallback: Some(_),
                ..
            } if error_codes.len() == 1
        )));
        assert!(program.functions.iter().any(|function| matches!(
            &function.body.result.kind,
            CompilerExpressionKind::ResultDecision {
                error_codes,
                error_fallback: None,
                ..
            } if error_codes.len() == 4
        )));
        let CompilerExpressionKind::Tuple(fields) = &program.main.result.kind else {
            panic!("expected top-level tuple")
        };
        assert!(matches!(
            fields[2].kind,
            CompilerExpressionKind::ErrorField {
                field: CompilerErrorField::Code,
                ..
            }
        ));
        assert!(matches!(
            fields[3].kind,
            CompilerExpressionKind::ErrorField {
                field: CompilerErrorField::Domain,
                ..
            }
        ));
    }

    #[test]
    fn models_qualified_arithmetic_error_code_values() {
        // TOPAL-NUM-ARITHMETIC-ERROR-001
        let source = "use language (version is v0.1)\nretain is fn (value : ErrorCode) -> ErrorCode\n  value\n(retain (lang arithmetic division-by-zero), lang arithmetic indeterminate, (lang arithmetic out-of-range) = (lang arithmetic out-of-range))\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(lang arithmetic ArithmeticErrorCode, lang arithmetic ArithmeticErrorCode, Boolean)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected a tuple")
        };
        assert!(matches!(
            values[1].kind,
            CompilerExpressionKind::ErrorCode(3)
        ));
    }

    #[test]
    fn models_lexical_blocks_with_fresh_shadowing_scope() {
        // TOPAL-SYN-GRAMMAR-001, TOPAL-EXEC-BLOCK-001
        let source = "use language (version is v0.1)\nColor is Enum (Red, Green)\nidentity is fn (number : Int) -> Int\n  number\nempty is {}\nvalue is 40\nshadow is {\n  Red is value + 1\n  identity is Red + 1\n  (Red, identity)\n}\n(empty, shadow, Red, identity 43, value)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Unit, (Int, Int), Color, Int, Int)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected a tuple")
        };
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                name,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::Block(_),
                    ..
                },
                ..
            }) if name == "shadow"
        )));
        assert!(matches!(values[2].kind, CompilerExpressionKind::Enum(0)));
        assert!(matches!(
            values[3].kind,
            CompilerExpressionKind::Call { .. }
        ));
        assert_eq!(exact_int(&values[4]), Some(BigInt::from(40)));

        let closed_zero = "use language (version is v0.1)\n1 / {\n  zero is 0\n  zero\n}\n";
        assert_eq!(
            analyze_for_compiler(closed_zero).unwrap_err().code,
            "E-DIVISION-BY-ZERO"
        );
    }

    #[test]
    fn models_completed_as_distinct_zero_data_evidence() {
        // TOPAL-EXEC-COMPLETED-001
        let source = "use language (version is v0.1)\nfinish is fn () -> Completed\n  Completed\nretain is fn (value : Completed) -> Completed\n  value\n(finish (), retain Completed, Completed = Completed)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Completed, Completed, Boolean)"
        );
        assert!(program.functions.iter().all(|function| {
            function.result_type == CompilerType::Completed
                && function.body.result.value_type == CompilerType::Completed
        }));
        assert_ne!(CompilerType::Completed, CompilerType::Unit);
    }

    #[test]
    fn models_discarded_function_parameters_without_binding_them() {
        // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-PATTERN-001
        let source = "use language (version is v0.1)\nsecond is fn (_ : Int, value : Int) -> Int\n  value\nsecond (0, 42)\n";
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "second")
            .unwrap();
        assert!(function.parameters[0].discarded);
        assert!(!function.parameters[1].discarded);
        assert!(matches!(
            function.body.result.kind,
            CompilerExpressionKind::Local(ref name) if name == "value"
        ));

        let mismatch = "use language (version is v0.1)\nsecond is fn (_ : Int, value : Int) -> Int\n  value\nsecond (\"ignored\", 42)\n";
        assert_eq!(
            analyze_for_compiler(mismatch).unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }

    #[test]
    fn models_ordered_overloads_and_static_call_boundaries() {
        // TOPAL-FUNCTION-OVERLOAD-001, TOPAL-FUNCTION-STATIC-NULLARY-001,
        // TOPAL-FUNCTION-STATIC-UNARY-001, TOPAL-FUNCTION-STATIC-BINARY-001
        let source = "use language (version is v0.1)\ndescribe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  describe 42\nanswer is fn static () -> Int\n  42\nadd is fn static (left : Int, right : Int) -> Int\n  left + right\n(describe \"Topal\", answer (), 20 add 22)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program
                .functions
                .iter()
                .filter(|function| function.source_name == "describe")
                .count(),
            2
        );
        assert_eq!(
            program
                .functions
                .iter()
                .filter(|function| function.is_static)
                .count(),
            2
        );

        let duplicate = "use language (version is v0.1)\nsame is fn (first : Int) -> Int\n  first\nsame is fn (second : Int) -> String\n  \"duplicate\"\nsame 1\n";
        assert_eq!(
            analyze_for_compiler(duplicate).unwrap_err().code,
            "E-DUPLICATE-FUNCTION-OVERLOAD"
        );

        let invalid_static_call = "use language (version is v0.1)\nruntime is fn () -> Int\n  42\nanswer is fn static () -> Int\n  runtime ()\nanswer ()\n";
        assert_eq!(
            analyze_for_compiler(invalid_static_call).unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }

    #[test]
    fn models_explicit_early_return_and_skips_the_tail() {
        // TOPAL-FUNCTION-RETURN-001
        let source = "use language (version is v0.1)\nanswer is fn static () -> Int\n  return 40 + 2\n  0\nanswer ()\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(program.main.result.value_type, CompilerType::Int);
        assert!(program.functions.iter().any(|function| {
            function.source_name == "answer"
                && matches!(
                    function.body.result.kind,
                    CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Add,
                        ..
                    }
                )
        }));

        let outside = "use language (version is v0.1)\nreturn 42\n";
        assert_eq!(
            analyze_for_compiler(outside).unwrap_err().code,
            "E-RETURN-OUTSIDE-FUNCTION"
        );
    }

    #[test]
    fn models_nominal_enum_values_functions_and_exhaustive_decisions() {
        // TOPAL-TYPE-ENUM-001, TOPAL-DECISION-ENUM-001
        let source = "use language (version is v0.1)\nColor is Enum (Red, Green, Blue)\nnext is fn (value : Color) -> Color\n  value\n    Red then Green\n    Green then Blue\n    Blue then Red\nfavorite : Color is Red\n(next favorite, next Green, Red = Green)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Color, Color, Boolean)"
        );
        let enumeration = CompilerEnumType {
            name: "Color".into(),
            alternatives: vec!["Red".into(), "Green".into(), "Blue".into()],
        };
        assert!(program.functions.iter().any(|function| {
            function.parameters[0].value_type == CompilerType::Enum(enumeration.clone())
                && matches!(
                    &function.body.result.kind,
                    CompilerExpressionKind::EnumDecision {
                        rules,
                        otherwise: None,
                        ..
                    } if rules.len() == 3
                )
        }));
    }

    #[test]
    fn rejects_invalid_nominal_enum_declarations_and_decisions() {
        // TOPAL-TYPE-ENUM-001, TOPAL-DECISION-ENUM-001
        let duplicate = "use language (version is v0.1)\nColor is Enum (Red, Red)\nRed\n";
        assert_eq!(
            analyze_for_compiler(duplicate).unwrap_err().code,
            "E-DUPLICATE-ENUM-ALTERNATIVE"
        );

        let type_as_alternative =
            "use language (version is v0.1)\nColor is Enum (Color, Green)\nGreen\n";
        assert_eq!(
            analyze_for_compiler(type_as_alternative).unwrap_err().code,
            "E-DUPLICATE-ENUM-ALTERNATIVE"
        );

        let incomplete = "use language (version is v0.1)\nColor is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"red\"\nname Green\n";
        assert_eq!(
            analyze_for_compiler(incomplete).unwrap_err().code,
            "E-INCOMPLETE-DECISION"
        );

        let nominal_mismatch = "use language (version is v0.1)\nColor is Enum (Red, Green)\nSignal is Enum (Stop, Go)\n(Red = Stop)\n";
        assert_eq!(
            analyze_for_compiler(nominal_mismatch).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );

        let before_declaration =
            "use language (version is v0.1)\nvalue is Red\nColor is Enum (Red, Green)\nvalue\n";
        assert_eq!(
            analyze_for_compiler(before_declaration).unwrap_err().code,
            "E-UNBOUND-NAME"
        );

        let nested = "use language (version is v0.1)\nmake is fn () -> Unit\n  Local is Enum (First, Second)\n  ()\nmake ()\n";
        assert_eq!(
            analyze_for_compiler(nested).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
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
