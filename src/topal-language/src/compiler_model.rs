//! Shared checked model consumed by the native compiler.
//!
//! This first model intentionally covers a bounded, explicit language slice.
//! Extending it is how later compiler increments acquire semantics; the LLVM
//! backend never reinterprets the source syntax itself.

use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigInt;
use topal_semantics::LanguageVersion;
use topal_source::{Diagnostic, SourceText, Span};
use topal_syntax::{
    CallableKind, DecisionMatcher, Expression, FunctionClauses, FunctionParameter, Statement, lex,
    parse,
};

use crate::source::{parse_integer, parse_string};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerType {
    Unit,
    Boolean,
    Int,
    String,
    Tuple(Vec<Self>),
}

impl CompilerType {
    #[must_use]
    pub const fn machine_scalar(&self) -> bool {
        matches!(self, Self::Unit | Self::Boolean | Self::Int)
    }

    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Unit => "Unit".into(),
            Self::Boolean => "Boolean".into(),
            Self::Int => "Int".into(),
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
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerExpressionKind {
    Unit,
    Boolean(bool),
    Int(BigInt),
    String(String),
    Tuple(Vec<CompilerExpression>),
    Local(String),
    Negate(Box<CompilerExpression>),
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
                ensure_i64(&self.source, *value, &IntRange::exact(integer.clone()))?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Int(integer.clone()),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(integer)),
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
                    span,
                })
            }
            Expression::Application { items, .. } => {
                self.analyze_application(items, span, environment)
            }
            Expression::DecisionTable { subject, rules, .. } => {
                self.analyze_boolean_decision(subject, rules, span, environment)
            }
            _ => Err(unsupported(&self.source, span, "expression form")),
        }
    }

    fn analyze_application(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let [
            Expression::Callable {
                kind: CallableKind::Minus,
                ..
            },
            operand,
        ] = items
        {
            let operand = self.analyze_expression(operand, environment)?;
            require_type(
                &self.source,
                operand.span,
                &CompilerType::Int,
                &operand.value_type,
            )?;
            let range = operand.int_range.as_ref().map(|range| IntRange {
                lower: -range.upper.clone(),
                upper: -range.lower.clone(),
            });
            if let Some(range) = &range {
                ensure_i64(&self.source, span, range)?;
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Negate(Box::new(operand)),
                value_type: CompilerType::Int,
                int_range: range,
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
                span,
            });
        }
        if let [left, Expression::Callable { kind, .. }, right] = items {
            return self.analyze_symbolic_binary(*kind, left, right, span, environment);
        }
        if let [left, Expression::Identifier(operation), right] = items {
            let operation = match self.source.slice(*operation) {
                "and" => Some(CompilerBinary::And),
                "or" => Some(CompilerBinary::Or),
                "xor" => Some(CompilerBinary::Xor),
                _ => None,
            };
            if let Some(operation) = operation {
                return self.analyze_binary(
                    operation,
                    left,
                    right,
                    span,
                    environment,
                    &CompilerType::Boolean,
                );
            }
        }
        self.analyze_call(items, span, environment)
    }

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
            CallableKind::Equal => CompilerBinary::Equal,
            CallableKind::NotEqual => CompilerBinary::NotEqual,
            CallableKind::Less => CompilerBinary::Less,
            CallableKind::Greater => CompilerBinary::Greater,
            CallableKind::LessEqual => CompilerBinary::LessEqual,
            CallableKind::GreaterEqual => CompilerBinary::GreaterEqual,
            _ => return Err(unsupported(&self.source, span, "numeric callable")),
        };
        let left_value = self.analyze_expression(left, environment)?;
        let right_value = self.analyze_expression(right, environment)?;
        require_same_type(
            &self.source,
            span,
            &left_value.value_type,
            &right_value.value_type,
        )?;
        if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual) {
            if !matches!(
                left_value.value_type,
                CompilerType::Int | CompilerType::Boolean | CompilerType::Unit
            ) {
                return Err(unsupported(
                    &self.source,
                    span,
                    "equality for this value type",
                ));
            }
        } else {
            require_type(
                &self.source,
                span,
                &CompilerType::Int,
                &left_value.value_type,
            )?;
        }
        self.finish_binary(operation, left_value, right_value, span)
    }

    fn analyze_binary(
        &mut self,
        operation: CompilerBinary,
        left: &Expression,
        right: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        expected: &CompilerType,
    ) -> Result<CompilerExpression, Diagnostic> {
        let left = self.analyze_expression(left, environment)?;
        let right = self.analyze_expression(right, environment)?;
        require_type(&self.source, left.span, expected, &left.value_type)?;
        require_type(&self.source, right.span, expected, &right.value_type)?;
        self.finish_binary(operation, left, right, span)
    }

    fn finish_binary(
        &self,
        operation: CompilerBinary,
        left: CompilerExpression,
        right: CompilerExpression,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        let int_range = match operation {
            CompilerBinary::Add => combine_ranges(&left, &right, |a, b| IntRange {
                lower: &a.lower + &b.lower,
                upper: &a.upper + &b.upper,
            }),
            CompilerBinary::Subtract => combine_ranges(&left, &right, |a, b| IntRange {
                lower: &a.lower - &b.upper,
                upper: &a.upper - &b.lower,
            }),
            CompilerBinary::Multiply => combine_ranges(&left, &right, multiply_range),
            _ => None,
        };
        if let Some(range) = &int_range {
            ensure_i64(&self.source, span, range)?;
        }
        let value_type = if matches!(
            operation,
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply
        ) {
            CompilerType::Int
        } else {
            CompilerType::Boolean
        };
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation,
                left: Box::new(left),
                right: Box::new(right),
            },
            value_type,
            int_range,
            span,
        })
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
        let (symbol, result_type, int_range) = result?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Call { symbol, arguments },
            value_type: result_type,
            int_range,
            span,
        })
    }

    fn instantiate_function(
        &mut self,
        function_name: &str,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
    ) -> Result<(String, CompilerType, Option<IntRange>), Diagnostic> {
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
        self.instances.push(CompilerFunction {
            source_name: function_name.to_owned(),
            symbol: symbol.clone(),
            parameters,
            result_type: result_type.clone(),
            body,
            span: declaration.span,
        });
        Ok((symbol, result_type, int_range))
    }

    fn analyze_boolean_decision(
        &mut self,
        subject: &Expression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let subject = self.analyze_expression(subject, environment)?;
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
        require_same_type(
            &self.source,
            span,
            &when_true.value_type,
            &when_false.value_type,
        )?;
        let int_range = match (&when_true.int_range, &when_false.int_range) {
            (Some(left), Some(right)) => Some(IntRange::union(left, right)),
            _ => None,
        };
        Ok(CompilerExpression {
            value_type: when_true.value_type.clone(),
            kind: CompilerExpressionKind::BooleanDecision {
                subject: Box::new(subject),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
            },
            int_range,
            span,
        })
    }
}

fn is_declaration(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::LanguageSelection { .. } | Statement::Function { .. }
    ) || matches!(statement, Statement::Published { declaration, .. } if matches!(declaration.as_ref(), Statement::Function { .. }))
}

fn parse_classifier(source: &SourceText, span: Span) -> Result<CompilerType, Diagnostic> {
    match source.slice(span).trim() {
        "Unit" => Ok(CompilerType::Unit),
        "Boolean" => Ok(CompilerType::Boolean),
        "Int" => Ok(CompilerType::Int),
        "String" => Ok(CompilerType::String),
        _ => Err(unsupported(source, span, "classifier")),
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

fn ensure_i64(source: &SourceText, span: Span, range: &IntRange) -> Result<(), Diagnostic> {
    if range.lower < BigInt::from(i64::MIN) || range.upper > BigInt::from(i64::MAX) {
        return Err(source_diagnostic(
            source,
            "E-COMPILER-UNSUPPORTED",
            span,
            "the first native increment requires proof that every Int result fits signed 64-bit storage",
        ));
    }
    Ok(())
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
        format!("the first native compiler increment does not yet support {construct}"),
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
    fn rejects_an_unproved_machine_integer_representation() {
        let source = "use language (version is v0.1)\n9223372036854775807 + 1\n";
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
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
