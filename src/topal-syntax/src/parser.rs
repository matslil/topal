use topal_source::{SourceText, Span};

use crate::{Lexed, SyntaxDiagnostic, Token, TokenKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Block {
        statements: Vec<Statement>,
        span: Span,
    },
    Unit(Span),
    Boolean(Span),
    Product {
        fields: Vec<ProductField>,
        span: Span,
    },
    DecisionTable {
        subject: Box<Expression>,
        rules: Vec<DecisionRule>,
        span: Span,
    },
    Integer(Span),
    Rational(Span),
    String(Span),
    Identifier(Span),
    Discard(Span),
    AnonymousFunction {
        parameters: Vec<Span>,
        body: Box<Self>,
        span: Span,
    },
    Callable {
        kind: CallableKind,
        span: Span,
    },
    Application {
        items: Vec<Self>,
        span: Span,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallableKind {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    Compare,
    Range,
    GreaterEqual,
    Plus,
    Minus,
    Multiply,
    Divide,
    QuotientModulo,
    Modulo,
    Power,
}

impl Expression {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Unit(span)
            | Self::Boolean(span)
            | Self::Integer(span)
            | Self::Rational(span)
            | Self::String(span)
            | Self::Identifier(span)
            | Self::Discard(span)
            | Self::Callable { span, .. }
            | Self::Product { span, .. }
            | Self::Block { span, .. }
            | Self::DecisionTable { span, .. }
            | Self::AnonymousFunction { span, .. }
            | Self::Application { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductField {
    pub label: Option<Span>,
    pub value: Expression,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionParameter {
    pub name: Span,
    pub classifier: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnionAlternative {
    pub name: Span,
    pub classifier: Option<Span>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionMatcher {
    Boolean {
        value: bool,
        span: Span,
    },
    Identifier(Span),
    Union {
        alternative: Span,
        binding: Span,
        span: Span,
    },
    Variant {
        type_name: Span,
        index: Span,
        binding: Span,
        span: Span,
    },
    Result {
        error: bool,
        binding: Span,
        span: Span,
    },
    Optional {
        some: bool,
        binding: Option<Span>,
        span: Span,
    },
    ListEmpty(Span),
    ListEntry {
        first: Span,
        rest: Span,
        span: Span,
    },
    ErrorCode {
        namespace: Span,
        vocabulary: Span,
        code: Span,
        span: Span,
    },
    Comparison {
        kind: CallableKind,
        operand: Expression,
        span: Span,
    },
    Otherwise(Span),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionRule {
    pub matcher: DecisionMatcher,
    pub action: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    Published {
        declaration: Box<Self>,
        span: Span,
    },
    DiagnosticControl {
        operation: DiagnosticControlKind,
        warning: Span,
        span: Span,
    },
    Binding {
        name: Span,
        classifier: Option<Span>,
        value: Expression,
    },
    Function {
        name: Span,
        is_static: bool,
        parameters: Vec<FunctionParameter>,
        result: Span,
        body: Vec<Statement>,
        span: Span,
    },
    Generator {
        name: Span,
        parameters: Vec<FunctionParameter>,
        yielded: Span,
        resumed: Span,
        result: Span,
        body: Vec<Statement>,
        span: Span,
    },
    Union {
        name: Span,
        alternatives: Vec<UnionAlternative>,
        span: Span,
    },
    Foreach {
        result: Option<(Span, Option<Span>)>,
        source: Expression,
        binding: Span,
        body: Vec<Statement>,
        span: Span,
    },
    Discard {
        span: Span,
        value: Expression,
    },
    Return {
        keyword: Span,
        value: Expression,
    },
    Expression(Expression),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticControlKind {
    DisableNext,
    Push,
    Pop,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParsedSource {
    pub statements: Vec<Statement>,
    pub diagnostics: Vec<SyntaxDiagnostic>,
}

#[must_use]
pub fn parse(source: &SourceText, lexed: &Lexed) -> ParsedSource {
    let mut parser = Parser {
        source,
        tokens: &lexed.tokens,
        cursor: 0,
        delimiter_depth: 0,
        diagnostics: lexed.diagnostics.clone(),
    };
    let mut statements = Vec::new();
    while parser.skip_separators() {
        if let Some(statement) = parser.statement() {
            statements.push(statement);
        }
        if parser
            .peek()
            .is_some_and(|token| token.kind != TokenKind::Newline)
        {
            parser.error_current("E-UNSUPPORTED-SYNTAX", "unsupported token after expression");
            parser.skip_to_newline();
        }
    }
    validate_diagnostic_controls(source, &statements, &mut parser.diagnostics);
    ParsedSource {
        statements,
        diagnostics: parser.diagnostics,
    }
}

struct Parser<'a> {
    source: &'a SourceText,
    tokens: &'a [Token],
    cursor: usize,
    delimiter_depth: usize,
    diagnostics: Vec<SyntaxDiagnostic>,
}

impl Parser<'_> {
    fn statement(&mut self) -> Option<Statement> {
        if self.foreach_separator_index().is_some() {
            return self.foreach_statement();
        }
        self.ordinary_statement()
    }

    #[allow(clippy::too_many_lines)] // Declaration forms retain localized diagnostics.
    fn ordinary_statement(&mut self) -> Option<Statement> {
        let checkpoint = self.cursor;
        let first = self.take_nontrivia()?;
        if first.kind == TokenKind::Identifier && self.source.slice(first.span) == "pub" {
            let declaration = self.ordinary_statement()?;
            if !matches!(
                declaration,
                Statement::Binding { .. }
                    | Statement::Function { .. }
                    | Statement::Generator { .. }
                    | Statement::Union { .. }
            ) {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-PUBLICATION-TARGET",
                    span: statement_span(&declaration),
                    message: "`pub` requires a declaration".into(),
                });
                return None;
            }
            let span = Span::new(first.span.start, statement_span(&declaration).end);
            return Some(Statement::Published {
                declaration: Box::new(declaration),
                span,
            });
        }
        if first.kind == TokenKind::Identifier && self.source.slice(first.span) == "lang" {
            return self.diagnostic_control(first);
        }
        if first.kind == TokenKind::Identifier && self.source.slice(first.span) == "return" {
            let Some(value) = self.expression() else {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-RETURN-VALUE",
                    span: Span::new(first.span.end, first.span.end),
                    message: "expected an expression after `return`".into(),
                });
                return None;
            };
            return Some(Statement::Return {
                keyword: first.span,
                value,
            });
        }
        if first.kind == TokenKind::Boolean
            && self.peek_nontrivia().is_some_and(|second| {
                second.kind == TokenKind::Identifier && self.source.slice(second.span) == "is"
            })
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-RESERVED-BOOLEAN-LITERAL",
                span: first.span,
                message: "a Boolean literal cannot introduce a binding".into(),
            });
            self.skip_to_newline();
            return None;
        }
        if matches!(first.kind, TokenKind::Identifier | TokenKind::Discard)
            && let Some(second) = self.peek_nontrivia()
            && second.kind == TokenKind::Identifier
            && self.source.slice(second.span) == "is"
        {
            let separator = self
                .take_nontrivia()
                .expect("peeked token remains available");
            if first.kind == TokenKind::Identifier
                && let Some(declaration) = self.peek_nontrivia()
                && declaration.kind == TokenKind::Identifier
            {
                match self.source.slice(declaration.span) {
                    "fn" => return self.function(first),
                    "generator" => return self.generator(first),
                    "Union" => return self.union(first),
                    _ => {}
                }
            }
            if let Some(value) = self.expression() {
                return Some(if first.kind == TokenKind::Discard {
                    Statement::Discard {
                        span: first.span,
                        value,
                    }
                } else {
                    Statement::Binding {
                        name: first.span,
                        classifier: None,
                        value,
                    }
                });
            }
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-EXPRESSION",
                span: Span::new(separator.span.end, separator.span.end),
                message: "expected a binding initializer".into(),
            });
            return None;
        }
        if first.kind == TokenKind::Identifier
            && self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::Colon)
        {
            self.take_nontrivia();
            let classifier_start = self.take_nontrivia()?;
            let classifier = self.classifier_from_first(classifier_start)?;
            let separator = self.take_nontrivia()?;
            if !matches!(
                classifier_start.kind,
                TokenKind::Identifier | TokenKind::LeftParen
            ) || separator.kind != TokenKind::Identifier
                || self.source.slice(separator.span) != "is"
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-CLASSIFIED-BINDING",
                    span: Span::new(first.span.start, separator.span.end),
                    message: "expected `name : Classifier is expression`".into(),
                });
                return None;
            }
            return self.expression().map(|value| Statement::Binding {
                name: first.span,
                classifier: Some(classifier),
                value,
            });
        }
        self.cursor = checkpoint;
        self.expression().map(Statement::Expression)
    }

    fn diagnostic_control(&mut self, lang: Token) -> Option<Statement> {
        let operation = self.take_nontrivia()?;
        let warning = self.take_nontrivia()?;
        let kind = match self.source.slice(operation.span) {
            "disable-warning" => DiagnosticControlKind::DisableNext,
            "push-disable-warning" => DiagnosticControlKind::Push,
            "pop-disable-warning" => DiagnosticControlKind::Pop,
            _ => {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-DIAGNOSTIC-CONTROL",
                    span: operation.span,
                    message: "expected a diagnostic-control operation after `lang`".into(),
                });
                self.skip_to_newline();
                return None;
            }
        };
        if operation.kind != TokenKind::Identifier || warning.kind != TokenKind::Identifier {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-DIAGNOSTIC-CONTROL",
                span: Span::new(lang.span.start, warning.span.end),
                message: "expected `lang diagnostic-operation warning-name`".into(),
            });
            return None;
        }
        Some(Statement::DiagnosticControl {
            operation: kind,
            warning: warning.span,
            span: Span::new(lang.span.start, warning.span.end),
        })
    }

    fn foreach_separator_index(&self) -> Option<usize> {
        self.tokens[self.cursor..]
            .iter()
            .position(|token| {
                token.kind == TokenKind::Identifier && self.source.slice(token.span) == "foreach"
            })
            .map(|offset| self.cursor + offset)
            .filter(|index| {
                !self.tokens[self.cursor..*index]
                    .iter()
                    .any(|token| token.kind == TokenKind::Newline)
            })
    }

    fn foreach_statement(&mut self) -> Option<Statement> {
        let separator_index = self.foreach_separator_index()?;
        let mut source_start = self.cursor;
        let mut result = None;
        let nontrivia = self.tokens[self.cursor..separator_index]
            .iter()
            .enumerate()
            .filter(|(_, token)| !token.kind.is_trivia())
            .collect::<Vec<_>>();
        if let Some((_, first)) = nontrivia.first()
            && first.kind == TokenKind::Identifier
            && let Some((is_position, (is_offset, _))) =
                nontrivia.iter().enumerate().find(|(_, (_, token))| {
                    token.kind == TokenKind::Identifier && self.source.slice(token.span) == "is"
                })
        {
            let classifier = if is_position == 1 {
                None
            } else if is_position >= 3 && nontrivia[1].1.kind == TokenKind::Colon {
                Some(Span::new(
                    nontrivia[2].1.span.start,
                    nontrivia[is_position - 1].1.span.end,
                ))
            } else {
                return None;
            };
            result = Some((first.span, classifier));
            source_start = self.cursor + is_offset + 1;
        }
        let mut source_parser = Self {
            source: self.source,
            tokens: &self.tokens[source_start..separator_index],
            cursor: 0,
            delimiter_depth: 0,
            diagnostics: Vec::new(),
        };
        let source_expression = source_parser.expression()?;
        self.diagnostics.extend(source_parser.diagnostics);
        self.cursor = separator_index + 1;
        let opening = self.take_nontrivia()?;
        let binding = self.take_nontrivia()?;
        let closing = self.take_nontrivia()?;
        if opening.kind != TokenKind::LeftBrace
            || binding.kind != TokenKind::Identifier
            || closing.kind != TokenKind::RightBrace
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FOREACH-BINDING",
                span: Span::new(opening.span.start, closing.span.end),
                message: "expected `source foreach { value }`".into(),
            });
            return None;
        }
        let newline = self.peek()?;
        if newline.kind != TokenKind::Newline {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FOREACH-BODY",
                span: closing.span,
                message: "expected an indented foreach body on the next line".into(),
            });
            return None;
        }
        self.cursor += 1;
        let Some(indent) = self.peek() else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FOREACH-BODY",
                span: closing.span,
                message: "expected an indented foreach body on the next line".into(),
            });
            return None;
        };
        if indent.kind != TokenKind::Whitespace {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-INDENTED-BODY",
                span: indent.span,
                message: "foreach body must be indented".into(),
            });
            return None;
        }
        let body = self.indented_function_body(indent.span.end - indent.span.start)?;
        let end = statement_span(body.last().expect("foreach body is nonempty")).end;
        let start = result.map_or(source_expression.span().start, |(name, _)| name.start);
        Some(Statement::Foreach {
            span: Span::new(start, end),
            result,
            source: source_expression,
            binding: binding.span,
            body,
        })
    }

    fn function(&mut self, name: Token) -> Option<Statement> {
        let function = self.take_nontrivia()?;
        let next = self.take_nontrivia()?;
        let is_static =
            next.kind == TokenKind::Identifier && self.source.slice(next.span) == "static";
        let opening = if is_static {
            self.take_nontrivia()?
        } else {
            next
        };
        let (parameters, closing) = self.static_function_parameters(opening)?;
        if parameters.len() > 2 {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-FUNCTION-OPERAND-COUNT",
                span: Span::new(opening.span.start, closing.span.end),
                message: "a function has at most two syntactic operands; package additional values explicitly".into(),
            });
            self.skip_to_newline();
            return None;
        }
        let arrow = self.take_nontrivia()?;
        let result_token = self.take_nontrivia()?;
        let result = self.function_result(result_token)?;
        let valid = self.source.slice(function.span) == "fn"
            && opening.kind == TokenKind::LeftParen
            && closing.kind == TokenKind::RightParen
            && arrow.kind == TokenKind::Arrow
            && matches!(
                result_token.kind,
                TokenKind::Identifier | TokenKind::LeftParen
            );
        if !valid {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNSUPPORTED-FUNCTION-HEADER",
                span: Span::new(function.span.start, result.end),
                message: "the implemented function subset requires `fn static ( name : Type, ... ) -> ResultType`".into(),
            });
            self.skip_to_newline();
            return None;
        }
        if !self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FUNCTION-BODY",
                span: Span::new(result.end, result.end),
                message: "expected an indented function body on the next line".into(),
            });
            return None;
        }
        self.cursor += 1;
        let Some(indent) = self.peek() else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FUNCTION-BODY",
                span: Span::new(result.end, result.end),
                message: "expected an indented function body on the next line".into(),
            });
            return None;
        };
        if indent.kind != TokenKind::Whitespace {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-INDENTED-BODY",
                span: indent.span,
                message: "function body must be indented".into(),
            });
            return None;
        }
        let body = self.indented_function_body(indent.span.end - indent.span.start)?;
        let body_end = statement_span(body.last().expect("function body is nonempty")).end;
        if self.source.slice(result) != "Unit"
            && matches!(
                body.last(),
                Some(Statement::Binding { .. } | Statement::Discard { .. })
            )
            && self.tokens[self.cursor..]
                .iter()
                .all(|token| token.kind.is_trivia())
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FUNCTION-BODY",
                span: Span::new(body_end, body_end),
                message: "expected a final expression producing the function result".into(),
            });
            return None;
        }
        Some(Statement::Function {
            name: name.span,
            is_static,
            parameters,
            result,
            span: Span::new(name.span.start, body_end),
            body,
        })
    }

    fn union(&mut self, name: Token) -> Option<Statement> {
        let union = self.take_nontrivia()?;
        let newline = self.peek()?;
        if newline.kind != TokenKind::Newline {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-UNION-ALTERNATIVES",
                span: union.span,
                message: "Union alternatives must begin on following indented lines".into(),
            });
            return None;
        }
        self.cursor += 1;
        let mut alternatives = Vec::new();
        let mut end = union.span.end;
        while let Some(indent) = self.peek() {
            if indent.kind != TokenKind::Whitespace {
                break;
            }
            self.cursor += 1;
            let alternative = self.take_nontrivia()?;
            if alternative.kind != TokenKind::Identifier {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-UNION-ALTERNATIVE",
                    span: alternative.span,
                    message: "expected a Union alternative name".into(),
                });
                return None;
            }
            let classifier = if self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::Colon)
            {
                self.take_nontrivia();
                let first = self.take_nontrivia()?;
                Some(self.classifier_from_first(first)?)
            } else {
                None
            };
            end = classifier.map_or(alternative.span.end, |span| span.end);
            alternatives.push(UnionAlternative {
                name: alternative.span,
                classifier,
            });
            if self
                .peek()
                .is_some_and(|token| token.kind == TokenKind::Newline)
            {
                self.cursor += 1;
            } else {
                break;
            }
        }
        if alternatives.is_empty() {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EMPTY-UNION",
                span: union.span,
                message: "a Union requires at least one alternative".into(),
            });
            return None;
        }
        Some(Statement::Union {
            name: name.span,
            alternatives,
            span: Span::new(name.span.start, end),
        })
    }

    fn generator(&mut self, name: Token) -> Option<Statement> {
        let keyword = self.take_nontrivia()?;
        let opening = self.take_nontrivia()?;
        let (parameters, closing) = self.static_function_parameters(opening)?;
        if parameters.is_empty() {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-GENERATOR-OPERAND-COUNT",
                span: Span::new(opening.span.start, closing.span.end),
                message: "a generator requires at least one initial operand".into(),
            });
            return None;
        }
        if self.source.slice(keyword.span) != "generator" || opening.kind != TokenKind::LeftParen {
            return None;
        }
        let yielded = self.generator_header_clause("yields")?;
        let resumed = self.generator_header_clause("resumes")?;
        let result = self.generator_result_clause()?;
        while self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            self.cursor += 1;
        }
        let Some(indent) = self.peek() else {
            self.error_current(
                "E-EXPECTED-GENERATOR-BODY",
                "expected an indented generator body",
            );
            return None;
        };
        if indent.kind != TokenKind::Whitespace {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-INDENTED-BODY",
                span: indent.span,
                message: "generator body must be indented".into(),
            });
            return None;
        }
        let body = self.indented_function_body(indent.span.end - indent.span.start)?;
        let body_end = statement_span(body.last().expect("generator body is nonempty")).end;
        Some(Statement::Generator {
            name: name.span,
            parameters,
            yielded,
            resumed,
            result,
            body,
            span: Span::new(name.span.start, body_end),
        })
    }

    fn generator_header_clause(&mut self, expected: &'static str) -> Option<Span> {
        self.expect_generator_header_newline()?;
        let keyword = self.take_nontrivia()?;
        let classifier = self.generator_classifier()?;
        if keyword.kind != TokenKind::Identifier || self.source.slice(keyword.span) != expected {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNSUPPORTED-GENERATOR-HEADER",
                span: Span::new(keyword.span.start, classifier.end),
                message: format!("expected `{expected} Type` in the generator header"),
            });
            return None;
        }
        Some(classifier)
    }

    fn generator_result_clause(&mut self) -> Option<Span> {
        self.expect_generator_header_newline()?;
        let arrow = self.take_nontrivia()?;
        let classifier = self.generator_classifier()?;
        if arrow.kind != TokenKind::Arrow {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNSUPPORTED-GENERATOR-HEADER",
                span: Span::new(arrow.span.start, classifier.end),
                message: "expected `-> Type` in the generator header".into(),
            });
            return None;
        }
        Some(classifier)
    }

    fn generator_classifier(&mut self) -> Option<Span> {
        let first = self.take_nontrivia()?;
        self.classifier_from_first(first)
    }

    fn classifier_from_first(&mut self, first: Token) -> Option<Span> {
        if first.kind == TokenKind::LeftParen {
            let mut depth = 1_usize;
            let mut end = first.span.end;
            while depth > 0 {
                let token = self.take_nontrivia()?;
                match token.kind {
                    TokenKind::LeftParen => depth += 1,
                    TokenKind::RightParen => depth -= 1,
                    _ => {}
                }
                end = token.span.end;
            }
            return Some(Span::new(first.span.start, end));
        }
        if first.kind != TokenKind::Identifier {
            return None;
        }
        if self.source.slice(first.span) == "Result" {
            if !self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::LeftParen)
            {
                return Some(first.span);
            }
            let opening = self.take_nontrivia()?;
            if opening.kind != TokenKind::LeftParen {
                return None;
            }
            let mut depth = 1_usize;
            let mut end = opening.span.end;
            while depth > 0 {
                let token = self.take_nontrivia()?;
                match token.kind {
                    TokenKind::LeftParen => depth += 1,
                    TokenKind::RightParen => depth -= 1,
                    _ => {}
                }
                end = token.span.end;
            }
            return Some(Span::new(first.span.start, end));
        }
        if matches!(self.source.slice(first.span), "Optional" | "Range" | "List") {
            let payload = self.generator_classifier()?;
            return Some(Span::new(first.span.start, payload.end));
        }
        if self.source.slice(first.span) == "Generator" {
            let _yielded = self.generator_classifier()?;
            let _resumed = self.generator_classifier()?;
            let returned = self.generator_classifier()?;
            return Some(Span::new(first.span.start, returned.end));
        }
        Some(first.span)
    }

    fn expect_generator_header_newline(&mut self) -> Option<()> {
        if !self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            self.error_current(
                "E-UNSUPPORTED-GENERATOR-HEADER",
                "generator header clauses must start on separate indented lines",
            );
            return None;
        }
        self.cursor += 1;
        if !self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Whitespace)
        {
            self.error_current(
                "E-EXPECTED-INDENTED-GENERATOR-HEADER",
                "generator header clauses must be indented",
            );
            return None;
        }
        self.cursor += 1;
        Some(())
    }

    fn function_result(&mut self, first: Token) -> Option<Span> {
        self.classifier_from_first(first)
    }

    fn indented_function_body(&mut self, body_indent: usize) -> Option<Vec<Statement>> {
        self.cursor += 1;
        let mut body = vec![self.statement()?];
        loop {
            if !self
                .peek()
                .is_some_and(|token| token.kind == TokenKind::Newline)
            {
                break;
            }
            let checkpoint = self.cursor;
            self.cursor += 1;
            let Some(indent) = self
                .peek()
                .filter(|token| token.kind == TokenKind::Whitespace)
            else {
                self.cursor = checkpoint;
                break;
            };
            let indent_width = indent.span.end - indent.span.start;
            if indent_width > body_indent {
                self.cursor = checkpoint;
                let Some(Statement::Expression(subject)) = body.pop() else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-DECISION-SUBJECT",
                        span: indent.span,
                        message: "a decision table must follow a subject expression".into(),
                    });
                    return None;
                };
                body.push(Statement::Expression(
                    self.decision_table(subject, indent_width)?,
                ));
                continue;
            }
            if indent_width < body_indent {
                self.cursor = checkpoint;
                break;
            }
            self.cursor += 1;
            body.push(self.statement()?);
        }
        Some(body)
    }

    #[allow(clippy::too_many_lines)] // Completeness and reachability checks stay adjacent to parsing.
    fn decision_table(&mut self, subject: Expression, rule_indent: usize) -> Option<Expression> {
        let mut rules = Vec::new();
        while self
            .tokens
            .get(self.cursor)
            .is_some_and(|token| token.kind == TokenKind::Newline)
            && self.tokens.get(self.cursor + 1).is_some_and(|token| {
                token.kind == TokenKind::Whitespace
                    && token.span.end - token.span.start == rule_indent
            })
        {
            self.cursor += 2;
            rules.push(self.decision_rule()?);
        }
        if let Some(span) = Self::rule_after_otherwise(&rules) {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNREACHABLE-DECISION-RULE",
                span,
                message: "decision rule is unreachable after `otherwise`".into(),
            });
            return None;
        }
        if let Some((span, code)) = self.duplicate_arithmetic_code(&rules) {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-DUPLICATE-ERROR-CODE-PATTERN",
                span,
                message: format!("arithmetic error code `{code}` is matched more than once"),
            });
            return None;
        }
        if let Some(span) = Self::error_code_after_fallback(&rules) {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNREACHABLE-ERROR-CODE-PATTERN",
                span,
                message: "qualified Error-code pattern is unreachable after `Error problem`".into(),
            });
            return None;
        }
        let complete = rules
            .iter()
            .any(|rule| matches!(&rule.matcher, DecisionMatcher::Otherwise(_)))
            || [false, true].into_iter().all(|value| {
                rules.iter().any(|rule| {
                    matches!(&rule.matcher, DecisionMatcher::Boolean { value: found, .. } if *found == value)
                })
            })
            || rules
                .iter()
                .all(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::Identifier(_)
                            | DecisionMatcher::Union { .. }
                            | DecisionMatcher::Variant { .. }
                    )
                })
            || [false, true].into_iter().all(|error| {
                rules.iter().any(|rule| {
                    matches!(rule.matcher, DecisionMatcher::Result { error: found, .. } if found == error)
                })
            })
            || [false, true].into_iter().all(|some| {
                rules.iter().any(|rule| {
                    matches!(rule.matcher, DecisionMatcher::Optional { some: found, .. } if found == some)
                })
            })
            || [false, true].into_iter().all(|entry| {
                rules.iter().any(|rule| {
                    matches!(rule.matcher, DecisionMatcher::ListEntry { .. } if entry)
                        || matches!(rule.matcher, DecisionMatcher::ListEmpty(_) if !entry)
                })
            })
            || self.complete_arithmetic_result(&rules);
        if !complete {
            let end = rules
                .last()
                .map_or(subject.span().end, |rule| rule.span.end);
            let missing_codes = self.missing_arithmetic_codes(&rules);
            let (code, message) = if missing_codes.len() < 4 {
                (
                    "E-INCOMPLETE-ERROR-CODE-DECISION",
                    format!(
                        "decision is missing arithmetic error code alternatives: {}",
                        missing_codes.join(", ")
                    ),
                )
            } else {
                (
                    "E-UNSUPPORTED-INCOMPLETE-DECISION",
                    "the implemented decision subset requires complete Boolean cases or `otherwise`"
                        .to_owned(),
                )
            };
            self.diagnostics.push(SyntaxDiagnostic {
                code,
                span: Span::new(subject.span().start, end),
                message,
            });
            return None;
        }
        let end = rules.last().expect("complete decision has rules").span.end;
        Some(Expression::DecisionTable {
            span: Span::new(subject.span().start, end),
            subject: Box::new(subject),
            rules,
        })
    }

    fn complete_arithmetic_result(&self, rules: &[DecisionRule]) -> bool {
        let has_ok = rules
            .iter()
            .any(|rule| matches!(rule.matcher, DecisionMatcher::Result { error: false, .. }));
        let codes = rules
            .iter()
            .filter_map(|rule| match rule.matcher {
                DecisionMatcher::ErrorCode {
                    namespace,
                    vocabulary,
                    code,
                    ..
                } if self.source.slice(namespace) == "lang"
                    && self.source.slice(vocabulary) == "arithmetic" =>
                {
                    Some(self.source.slice(code))
                }
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        has_ok
            && codes
                == [
                    "out-of-range",
                    "not-representable",
                    "division-by-zero",
                    "indeterminate",
                ]
                .into_iter()
                .collect()
    }

    fn missing_arithmetic_codes(&self, rules: &[DecisionRule]) -> Vec<&'static str> {
        let present = rules
            .iter()
            .filter_map(|rule| match rule.matcher {
                DecisionMatcher::ErrorCode {
                    namespace,
                    vocabulary,
                    code,
                    ..
                } if self.source.slice(namespace) == "lang"
                    && self.source.slice(vocabulary) == "arithmetic" =>
                {
                    Some(self.source.slice(code))
                }
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        [
            "out-of-range",
            "not-representable",
            "division-by-zero",
            "indeterminate",
        ]
        .into_iter()
        .filter(|code| !present.contains(code))
        .collect()
    }

    fn duplicate_arithmetic_code(&self, rules: &[DecisionRule]) -> Option<(Span, &str)> {
        let mut seen = std::collections::BTreeSet::new();
        rules.iter().find_map(|rule| match rule.matcher {
            DecisionMatcher::ErrorCode {
                namespace,
                vocabulary,
                code,
                ..
            } if self.source.slice(namespace) == "lang"
                && self.source.slice(vocabulary) == "arithmetic" =>
            {
                let name = self.source.slice(code);
                (!seen.insert(name)).then_some((code, name))
            }
            _ => None,
        })
    }

    fn error_code_after_fallback(rules: &[DecisionRule]) -> Option<Span> {
        let fallback = rules
            .iter()
            .position(|rule| matches!(rule.matcher, DecisionMatcher::Result { error: true, .. }))?;
        rules
            .iter()
            .skip(fallback + 1)
            .find_map(|rule| match rule.matcher {
                DecisionMatcher::ErrorCode { span, .. } => Some(span),
                _ => None,
            })
    }

    fn rule_after_otherwise(rules: &[DecisionRule]) -> Option<Span> {
        let fallback = rules
            .iter()
            .position(|rule| matches!(rule.matcher, DecisionMatcher::Otherwise(_)))?;
        rules
            .get(fallback + 1)
            .map(|rule| matcher_span(&rule.matcher))
    }

    #[allow(clippy::too_many_lines)] // Matcher-specific diagnostics remain explicit and source-located.
    fn decision_rule(&mut self) -> Option<DecisionRule> {
        let matcher_token = self.take_nontrivia()?;
        let (matcher, action) = match matcher_token.kind {
            TokenKind::Boolean => {
                let separator = self.take_nontrivia()?;
                if separator.kind != TokenKind::Identifier
                    || self.source.slice(separator.span) != "then"
                {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-THEN",
                        span: separator.span,
                        message: "expected `then` between the matcher and delayed action".into(),
                    });
                    return None;
                }
                (
                    DecisionMatcher::Boolean {
                        value: self.source.slice(matcher_token.span) == "true",
                        span: matcher_token.span,
                    },
                    self.expression()?,
                )
            }
            TokenKind::Identifier if self.source.slice(matcher_token.span) == "otherwise" => (
                DecisionMatcher::Otherwise(matcher_token.span),
                self.expression()?,
            ),
            TokenKind::Identifier if self.begins_error_code_pattern(matcher_token.span) => (
                self.error_code_matcher(matcher_token.span)?,
                self.expression()?,
            ),
            TokenKind::Identifier
                if matches!(self.source.slice(matcher_token.span), "Ok" | "Error") =>
            {
                let binding = self.take_nontrivia()?;
                let separator = self.take_nontrivia()?;
                if binding.kind != TokenKind::Identifier
                    || separator.kind != TokenKind::Identifier
                    || self.source.slice(separator.span) != "then"
                {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-RESULT-PATTERN",
                        span: Span::new(matcher_token.span.start, separator.span.end),
                        message: "expected `Ok name then` or `Error name then`".into(),
                    });
                    return None;
                }
                (
                    DecisionMatcher::Result {
                        error: self.source.slice(matcher_token.span) == "Error",
                        binding: binding.span,
                        span: Span::new(matcher_token.span.start, binding.span.end),
                    },
                    self.expression()?,
                )
            }
            TokenKind::Identifier
                if matches!(self.source.slice(matcher_token.span), "Some" | "None") =>
            {
                (self.optional_matcher(matcher_token)?, self.expression()?)
            }
            TokenKind::Identifier if self.source.slice(matcher_token.span) == "Empty" => {
                (self.list_empty_matcher(matcher_token)?, self.expression()?)
            }
            TokenKind::Identifier if self.source.slice(matcher_token.span) == "Entry" => {
                (self.list_entry_matcher(matcher_token)?, self.expression()?)
            }
            TokenKind::Identifier
                if self.peek_nontrivia().is_some_and(|token| {
                    token.kind == TokenKind::Identifier && self.source.slice(token.span) == "at"
                }) =>
            {
                (self.variant_matcher(matcher_token)?, self.expression()?)
            }
            TokenKind::Identifier => (self.identifier_matcher(matcher_token)?, self.expression()?),
            token_kind if comparison_callable(token_kind).is_some() => {
                let kind = comparison_callable(token_kind).expect("checked comparison token");
                let operand = self.expression_before_then(matcher_token.span)?;
                let span = Span::new(matcher_token.span.start, operand.span().end);
                (
                    DecisionMatcher::Comparison {
                        kind,
                        operand,
                        span,
                    },
                    self.expression()?,
                )
            }
            _ => {
                self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-UNSUPPORTED-DECISION-MATCHER",
                        span: matcher_token.span,
                        message: "the implemented decision subset accepts Boolean literals, comparisons, or `otherwise`".into(),
                    });
                return None;
            }
        };
        let span = Span::new(matcher_span(&matcher).start, action.span().end);
        Some(DecisionRule {
            matcher,
            action,
            span,
        })
    }

    fn optional_matcher(&mut self, constructor: Token) -> Option<DecisionMatcher> {
        let some = self.source.slice(constructor.span) == "Some";
        let binding = some.then(|| self.take_nontrivia()).flatten();
        let separator = self.take_nontrivia()?;
        let valid = separator.kind == TokenKind::Identifier
            && self.source.slice(separator.span) == "then"
            && binding.is_none_or(|binding| binding.kind == TokenKind::Identifier);
        if !valid {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-OPTIONAL-PATTERN",
                span: Span::new(constructor.span.start, separator.span.end),
                message: if some {
                    "expected `Some name then`"
                } else {
                    "expected `None then`"
                }
                .into(),
            });
            return None;
        }
        Some(DecisionMatcher::Optional {
            some,
            binding: binding.map(|binding| binding.span),
            span: Span::new(
                constructor.span.start,
                binding.map_or(constructor.span.end, |binding| binding.span.end),
            ),
        })
    }

    fn list_entry_matcher(&mut self, constructor: Token) -> Option<DecisionMatcher> {
        let opening = self.take_nontrivia()?;
        let first = self.take_nontrivia()?;
        let comma = self.take_nontrivia()?;
        let rest = self.take_nontrivia()?;
        let closing = self.take_nontrivia()?;
        let separator = self.take_nontrivia()?;
        if opening.kind != TokenKind::LeftParen
            || first.kind != TokenKind::Identifier
            || comma.kind != TokenKind::Comma
            || rest.kind != TokenKind::Identifier
            || closing.kind != TokenKind::RightParen
            || separator.kind != TokenKind::Identifier
            || self.source.slice(separator.span) != "then"
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-LIST-PATTERN",
                span: Span::new(constructor.span.start, separator.span.end),
                message: "expected `Entry ( first, rest ) then`".into(),
            });
            return None;
        }
        Some(DecisionMatcher::ListEntry {
            first: first.span,
            rest: rest.span,
            span: Span::new(constructor.span.start, closing.span.end),
        })
    }

    fn list_empty_matcher(&mut self, constructor: Token) -> Option<DecisionMatcher> {
        let separator = self.take_nontrivia()?;
        if separator.kind != TokenKind::Identifier || self.source.slice(separator.span) != "then" {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-LIST-PATTERN",
                span: Span::new(constructor.span.start, separator.span.end),
                message: "expected `Empty then`".into(),
            });
            return None;
        }
        Some(DecisionMatcher::ListEmpty(constructor.span))
    }

    fn identifier_matcher(&mut self, identifier: Token) -> Option<DecisionMatcher> {
        let separator = self.take_nontrivia()?;
        if separator.kind == TokenKind::Identifier && self.source.slice(separator.span) != "then" {
            let then = self.take_nontrivia()?;
            if then.kind == TokenKind::Identifier && self.source.slice(then.span) == "then" {
                return Some(DecisionMatcher::Union {
                    alternative: identifier.span,
                    binding: separator.span,
                    span: Span::new(identifier.span.start, separator.span.end),
                });
            }
        }
        if separator.kind != TokenKind::Identifier || self.source.slice(separator.span) != "then" {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-THEN",
                span: separator.span,
                message: "expected `then` between the matcher and delayed action".into(),
            });
            return None;
        }
        Some(DecisionMatcher::Identifier(identifier.span))
    }

    fn variant_matcher(&mut self, type_name: Token) -> Option<DecisionMatcher> {
        self.take_nontrivia();
        let index = self.take_nontrivia()?;
        let binding = self.take_nontrivia()?;
        let then = self.take_nontrivia()?;
        if index.kind != TokenKind::Integer
            || binding.kind != TokenKind::Identifier
            || then.kind != TokenKind::Identifier
            || self.source.slice(then.span) != "then"
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-VARIANT-PATTERN",
                span: Span::new(type_name.span.start, then.span.end),
                message: "expected `Type at index binding then`".into(),
            });
            return None;
        }
        Some(DecisionMatcher::Variant {
            type_name: type_name.span,
            index: index.span,
            binding: binding.span,
            span: Span::new(type_name.span.start, binding.span.end),
        })
    }

    fn error_code_matcher(&mut self, error: Span) -> Option<DecisionMatcher> {
        let opening = self.take_nontrivia()?;
        let field = self.take_nontrivia()?;
        let is = self.take_nontrivia()?;
        let namespace = self.take_nontrivia()?;
        let vocabulary = self.take_nontrivia()?;
        let code = self.take_nontrivia()?;
        let closing = self.take_nontrivia()?;
        let separator = self.take_nontrivia()?;
        if opening.kind != TokenKind::LeftParen
            || field.kind != TokenKind::Identifier
            || self.source.slice(field.span) != "code"
            || is.kind != TokenKind::Identifier
            || self.source.slice(is.span) != "is"
            || namespace.kind != TokenKind::Identifier
            || vocabulary.kind != TokenKind::Identifier
            || code.kind != TokenKind::Identifier
            || closing.kind != TokenKind::RightParen
            || separator.kind != TokenKind::Identifier
            || self.source.slice(separator.span) != "then"
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-ERROR-CODE-PATTERN",
                span: Span::new(error.start, separator.span.end),
                message: "expected `Error ( code is namespace vocabulary code ) then`".into(),
            });
            return None;
        }
        Some(DecisionMatcher::ErrorCode {
            namespace: namespace.span,
            vocabulary: vocabulary.span,
            code: code.span,
            span: Span::new(error.start, closing.span.end),
        })
    }

    fn begins_error_code_pattern(&self, matcher: Span) -> bool {
        self.source.slice(matcher) == "Error"
            && self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::LeftParen)
    }

    fn expression_before_then(&mut self, matcher_span: Span) -> Option<Expression> {
        let Some(separator_index) = self.tokens[self.cursor..]
            .iter()
            .take_while(|token| token.kind != TokenKind::Newline)
            .position(|token| {
                token.kind == TokenKind::Identifier && self.source.slice(token.span) == "then"
            })
            .map(|offset| self.cursor + offset)
        else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-THEN",
                span: Span::new(matcher_span.end, matcher_span.end),
                message: "expected `then` between the matcher and delayed action".into(),
            });
            return None;
        };
        let mut parser = Self {
            source: self.source,
            tokens: &self.tokens[self.cursor..separator_index],
            cursor: 0,
            delimiter_depth: 0,
            diagnostics: Vec::new(),
        };
        let operand = parser.expression();
        self.diagnostics.extend(parser.diagnostics);
        self.cursor = separator_index + 1;
        operand
    }

    fn static_function_parameters(
        &mut self,
        opening: Token,
    ) -> Option<(Vec<FunctionParameter>, Token)> {
        let delimited = opening.kind == TokenKind::LeftParen;
        self.delimiter_depth += usize::from(delimited);
        let parsed = self.static_function_parameters_inner(opening);
        self.delimiter_depth -= usize::from(delimited);
        parsed
    }

    #[allow(clippy::too_many_lines)] // Classifier forms are parsed explicitly for precise spans.
    fn static_function_parameters_inner(
        &mut self,
        opening: Token,
    ) -> Option<(Vec<FunctionParameter>, Token)> {
        let mut parameters = Vec::new();
        let mut input = self.take_nontrivia()?;
        let closing = loop {
            if input.kind == TokenKind::RightParen {
                break input;
            }
            let colon = self.take_nontrivia()?;
            let classifier_start = self.take_nontrivia()?;
            let classifier = self.classifier_from_first(classifier_start)?;
            let separator = self.take_nontrivia()?;
            if !matches!(input.kind, TokenKind::Identifier | TokenKind::Discard)
                || colon.kind != TokenKind::Colon
                || !matches!(
                    classifier_start.kind,
                    TokenKind::Identifier | TokenKind::LeftParen
                )
                || !matches!(separator.kind, TokenKind::Comma | TokenKind::RightParen)
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-UNSUPPORTED-FUNCTION-HEADER",
                    span: Span::new(opening.span.start, separator.span.end),
                    message:
                        "function parameters must have the form `pattern : Type`, separated by commas"
                            .into(),
                });
                self.skip_to_newline();
                return None;
            }
            parameters.push(FunctionParameter {
                name: input.span,
                classifier,
            });
            if separator.kind == TokenKind::RightParen {
                break separator;
            }
            input = self.take_nontrivia()?;
        };
        Some((parameters, closing))
    }

    fn expression(&mut self) -> Option<Expression> {
        let first = self.primary()?;
        let mut items = vec![first];
        loop {
            if self
                .peek_nontrivia()
                .is_none_or(|token| matches!(token.kind, TokenKind::RightParen | TokenKind::Comma))
            {
                break;
            }
            let Some(item) = self.primary() else {
                break;
            };
            items.push(item);
        }
        if items.len() == 1 {
            return items.pop();
        }
        let span = Span::new(
            items[0].span().start,
            items.last().expect("nonempty").span().end,
        );
        Some(Expression::Application { items, span })
    }

    fn primary(&mut self) -> Option<Expression> {
        let token = self.take_nontrivia()?;
        match token.kind {
            TokenKind::Boolean => Some(Expression::Boolean(token.span)),
            TokenKind::Integer => Some(Expression::Integer(token.span)),
            TokenKind::Rational => Some(Expression::Rational(token.span)),
            TokenKind::String => Some(Expression::String(token.span)),
            TokenKind::Identifier => Some(Expression::Identifier(token.span)),
            TokenKind::Discard => Some(Expression::Discard(token.span)),
            TokenKind::Equals => Some(Expression::Callable {
                kind: CallableKind::Equal,
                span: token.span,
            }),
            TokenKind::NotEquals => Some(Expression::Callable {
                kind: CallableKind::NotEqual,
                span: token.span,
            }),
            TokenKind::Less => Some(Expression::Callable {
                kind: CallableKind::Less,
                span: token.span,
            }),
            TokenKind::Greater => Some(Expression::Callable {
                kind: CallableKind::Greater,
                span: token.span,
            }),
            TokenKind::LessEqual => Some(Expression::Callable {
                kind: CallableKind::LessEqual,
                span: token.span,
            }),
            TokenKind::Compare => Some(Expression::Callable {
                kind: CallableKind::Compare,
                span: token.span,
            }),
            TokenKind::Range => Some(Expression::Callable {
                kind: CallableKind::Range,
                span: token.span,
            }),
            TokenKind::GreaterEqual => Some(Expression::Callable {
                kind: CallableKind::GreaterEqual,
                span: token.span,
            }),
            TokenKind::Plus => Some(Expression::Callable {
                kind: CallableKind::Plus,
                span: token.span,
            }),
            TokenKind::Minus => Some(Expression::Callable {
                kind: CallableKind::Minus,
                span: token.span,
            }),
            TokenKind::Star => Some(Expression::Callable {
                kind: CallableKind::Multiply,
                span: token.span,
            }),
            TokenKind::Slash => Some(Expression::Callable {
                kind: CallableKind::Divide,
                span: token.span,
            }),
            TokenKind::SlashPercent => Some(Expression::Callable {
                kind: CallableKind::QuotientModulo,
                span: token.span,
            }),
            TokenKind::Percent => Some(Expression::Callable {
                kind: CallableKind::Modulo,
                span: token.span,
            }),
            TokenKind::Caret => Some(Expression::Callable {
                kind: CallableKind::Power,
                span: token.span,
            }),
            TokenKind::LeftParen => {
                self.delimiter_depth += 1;
                let expression = self.parenthesized(token);
                self.delimiter_depth -= 1;
                expression
            }
            TokenKind::LeftBrace => self.braced_expression(token),
            _ => {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-EXPRESSION",
                    span: token.span,
                    message: "expected a literal, name, callable, or parenthesized expression"
                        .into(),
                });
                None
            }
        }
    }

    fn braced_expression(&mut self, opening: Token) -> Option<Expression> {
        let start = self.cursor;
        let mut depth = 1_usize;
        let mut closing_index = None;
        for (offset, token) in self.tokens[start..].iter().enumerate() {
            match token.kind {
                TokenKind::LeftBrace => depth += 1,
                TokenKind::RightBrace => {
                    depth -= 1;
                    if depth == 0 {
                        closing_index = Some(start + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(closing_index) = closing_index else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-RBRACE",
                span: opening.span,
                message: "expected a closing brace".into(),
            });
            return None;
        };
        if self.braces_introduce_anonymous_function(start, closing_index) {
            return self.anonymous_function(opening);
        }

        let closing = self.tokens[closing_index];
        let mut block_parser = Self {
            source: self.source,
            tokens: &self.tokens[start..closing_index],
            cursor: 0,
            delimiter_depth: 0,
            diagnostics: Vec::new(),
        };
        let mut statements = Vec::new();
        while block_parser.skip_separators() {
            if let Some(statement) = block_parser.statement() {
                statements.push(statement);
            }
            if block_parser
                .peek()
                .is_some_and(|token| token.kind != TokenKind::Newline)
            {
                block_parser.error_current(
                    "E-UNSUPPORTED-SYNTAX",
                    "unsupported token after block statement",
                );
                block_parser.skip_to_newline();
            }
        }
        validate_diagnostic_controls(self.source, &statements, &mut block_parser.diagnostics);
        self.diagnostics.extend(block_parser.diagnostics);
        self.cursor = closing_index + 1;
        Some(Expression::Block {
            statements,
            span: Span::new(opening.span.start, closing.span.end),
        })
    }

    fn braces_introduce_anonymous_function(&self, start: usize, closing: usize) -> bool {
        let content = self.tokens[start..closing]
            .iter()
            .filter(|token| !token.kind.is_trivia())
            .collect::<Vec<_>>();
        let parameters = !content.is_empty()
            && content.iter().enumerate().all(|(index, token)| {
                if index % 2 == 0 {
                    token.kind == TokenKind::Identifier
                } else {
                    token.kind == TokenKind::Comma
                }
            });
        if !parameters {
            return false;
        }
        self.tokens[closing + 1..]
            .iter()
            .take_while(|token| token.kind != TokenKind::Newline)
            .any(|token| !token.kind.is_trivia())
    }

    fn anonymous_function(&mut self, opening: Token) -> Option<Expression> {
        let mut parameters = Vec::new();
        loop {
            let token = self.take_nontrivia()?;
            if token.kind == TokenKind::RightBrace {
                if parameters.is_empty() {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EMPTY-ANONYMOUS-FUNCTION-PATTERN",
                        span: Span::new(opening.span.start, token.span.end),
                        message:
                            "an inferred anonymous function requires at least one parameter pattern"
                                .into(),
                    });
                    return None;
                }
                let Some(body) = self.expression() else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-ANONYMOUS-FUNCTION-BODY",
                        span: token.span,
                        message: "expected an anonymous-function body after the parameter pattern"
                            .into(),
                    });
                    return None;
                };
                let span = Span::new(opening.span.start, body.span().end);
                return Some(Expression::AnonymousFunction {
                    parameters,
                    body: Box::new(body),
                    span,
                });
            }
            if token.kind != TokenKind::Identifier {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-ANONYMOUS-FUNCTION-PARAMETER",
                    span: token.span,
                    message: "expected a parameter name or closing brace".into(),
                });
                return None;
            }
            parameters.push(token.span);
            let Some(separator) = self.peek_nontrivia() else {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-RBRACE",
                    span: token.span,
                    message: "expected `}` after anonymous-function parameters".into(),
                });
                return None;
            };
            if separator.kind == TokenKind::Comma {
                self.take_nontrivia();
            } else if separator.kind != TokenKind::RightBrace {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-ANONYMOUS-FUNCTION-SEPARATOR",
                    span: separator.span,
                    message: "expected `,` or `}` after anonymous-function parameter".into(),
                });
                return None;
            }
        }
    }

    fn parenthesized(&mut self, opening: Token) -> Option<Expression> {
        if self
            .peek_nontrivia()
            .is_some_and(|value| value.kind == TokenKind::RightParen)
        {
            let closing = self
                .take_nontrivia()
                .expect("peeked closing parenthesis remains available");
            return Some(Expression::Unit(Span::new(
                opening.span.start,
                closing.span.end,
            )));
        }
        let Some(first) = self.product_field() else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-RPAREN",
                span: Span::new(opening.span.end, opening.span.end),
                message: "expected expression and closing parenthesis".into(),
            });
            return None;
        };
        let product = first.label.is_some()
            || self
                .peek_nontrivia()
                .is_some_and(|value| value.kind == TokenKind::Comma);
        let mut fields = vec![first.clone()];
        if product {
            loop {
                if !self
                    .peek_nontrivia()
                    .is_some_and(|value| value.kind == TokenKind::Comma)
                {
                    break;
                }
                self.take_nontrivia();
                if self
                    .peek_nontrivia()
                    .is_some_and(|value| value.kind == TokenKind::RightParen)
                {
                    break;
                }
                let Some(field) = self.product_field() else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-RPAREN",
                        span: Span::new(opening.span.end, opening.span.end),
                        message: "expected product field or closing parenthesis".into(),
                    });
                    return None;
                };
                fields.push(field);
            }
        }
        let closing = self.take_nontrivia();
        if !closing.is_some_and(|value| value.kind == TokenKind::RightParen) {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-RPAREN",
                span: Span::new(opening.span.end, opening.span.end),
                message: "expected closing parenthesis".into(),
            });
        }
        if product {
            let first_labeled = fields[0].label.is_some();
            if let Some(mixed) = fields
                .iter()
                .find(|field| field.label.is_some() != first_labeled)
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-MIXED-PRODUCT-FIELDS",
                    span: mixed.label.unwrap_or_else(|| mixed.value.span()),
                    message: "a product cannot mix positional and labeled fields".into(),
                });
            }
            Some(Expression::Product {
                fields,
                span: Span::new(
                    opening.span.start,
                    closing.map_or(first.value.span().end, |value| value.span.end),
                ),
            })
        } else {
            Some(first.value)
        }
    }

    fn product_field(&mut self) -> Option<ProductField> {
        let checkpoint = self.cursor;
        if let Some(label) = self.take_nontrivia()
            && label.kind == TokenKind::Identifier
            && self.peek_nontrivia().is_some_and(|separator| {
                separator.kind == TokenKind::Identifier && self.source.slice(separator.span) == "is"
            })
        {
            self.take_nontrivia();
            return self.expression().map(|value| ProductField {
                label: Some(label.span),
                value,
            });
        }
        self.cursor = checkpoint;
        self.expression()
            .map(|value| ProductField { label: None, value })
    }

    fn skip_separators(&mut self) -> bool {
        while let Some(token) = self.tokens.get(self.cursor) {
            if token.kind.is_trivia() {
                self.cursor += 1;
            } else {
                break;
            }
        }
        self.cursor < self.tokens.len()
    }

    fn take_nontrivia(&mut self) -> Option<Token> {
        while let Some(token) = self.tokens.get(self.cursor).copied() {
            if token.kind == TokenKind::Newline && self.delimiter_depth == 0 {
                return None;
            }
            self.cursor += 1;
            if !token.kind.is_trivia() {
                return Some(token);
            }
        }
        None
    }

    fn peek_nontrivia(&self) -> Option<Token> {
        for token in self.tokens[self.cursor..].iter().copied() {
            if token.kind == TokenKind::Newline && self.delimiter_depth == 0 {
                return None;
            }
            if !token.kind.is_trivia() {
                return Some(token);
            }
        }
        None
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.cursor).copied()
    }

    fn error_current(&mut self, code: &'static str, message: &'static str) {
        let span = self.peek().map_or(Span::default(), |token| token.span);
        self.diagnostics.push(SyntaxDiagnostic {
            code,
            span,
            message: message.to_owned(),
        });
    }

    fn skip_to_newline(&mut self) {
        while self
            .peek()
            .is_some_and(|token| token.kind != TokenKind::Newline)
        {
            self.cursor += 1;
        }
    }
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::Binding { name, value, .. } => Span::new(name.start, value.span().end),
        Statement::Published { span, .. }
        | Statement::DiagnosticControl { span, .. }
        | Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Foreach { span, .. }
        | Statement::Discard { span, .. } => *span,
        Statement::Return { keyword, value } => Span::new(keyword.start, value.span().end),
        Statement::Expression(expression) => expression.span(),
    }
}

fn validate_diagnostic_controls(
    source: &SourceText,
    statements: &[Statement],
    diagnostics: &mut Vec<SyntaxDiagnostic>,
) {
    let mut stack = Vec::<Span>::new();
    let mut pending = None;
    for statement in statements {
        match statement {
            Statement::Published { declaration, .. } => {
                pending = None;
                validate_diagnostic_controls(
                    source,
                    std::slice::from_ref(declaration.as_ref()),
                    diagnostics,
                );
            }
            Statement::DiagnosticControl {
                operation,
                warning,
                span,
            } => match operation {
                DiagnosticControlKind::DisableNext => pending = Some((*warning, *span)),
                DiagnosticControlKind::Push => stack.push(*warning),
                DiagnosticControlKind::Pop => match stack.last().copied() {
                    None => diagnostics.push(SyntaxDiagnostic {
                        code: "E-DIAGNOSTIC-CONTROL-UNDERFLOW",
                        span: *span,
                        message: "cannot pop an empty warning-suppression stack".into(),
                    }),
                    Some(active) if source.slice(active) != source.slice(*warning) => {
                        diagnostics.push(SyntaxDiagnostic {
                            code: "E-DIAGNOSTIC-CONTROL-MISMATCH",
                            span: *warning,
                            message: format!(
                                "cannot pop warning `{}` while `{}` is active",
                                source.slice(*warning),
                                source.slice(active)
                            ),
                        });
                    }
                    Some(_) => {
                        stack.pop();
                    }
                },
            },
            Statement::Function { body, .. }
            | Statement::Generator { body, .. }
            | Statement::Foreach { body, .. } => {
                pending = None;
                validate_diagnostic_controls(source, body, diagnostics);
            }
            _ => pending = None,
        }
    }
    if let Some((warning, span)) = pending {
        diagnostics.push(SyntaxDiagnostic {
            code: "E-DIAGNOSTIC-CONTROL-TARGET",
            span,
            message: format!(
                "warning suppression for `{}` has no following statement",
                source.slice(warning)
            ),
        });
    }
    if let Some(warning) = stack.last().copied() {
        diagnostics.push(SyntaxDiagnostic {
            code: "E-DIAGNOSTIC-CONTROL-UNCLOSED",
            span: warning,
            message: format!(
                "warning suppression for `{}` remains active at the context boundary",
                source.slice(warning)
            ),
        });
    }
}

const fn matcher_span(matcher: &DecisionMatcher) -> Span {
    match matcher {
        DecisionMatcher::Boolean { span, .. }
        | DecisionMatcher::Identifier(span)
        | DecisionMatcher::Union { span, .. }
        | DecisionMatcher::Variant { span, .. }
        | DecisionMatcher::Result { span, .. }
        | DecisionMatcher::Optional { span, .. }
        | DecisionMatcher::ListEmpty(span)
        | DecisionMatcher::ListEntry { span, .. }
        | DecisionMatcher::ErrorCode { span, .. }
        | DecisionMatcher::Comparison { span, .. }
        | DecisionMatcher::Otherwise(span) => *span,
    }
}

const fn comparison_callable(kind: TokenKind) -> Option<CallableKind> {
    match kind {
        TokenKind::Equals => Some(CallableKind::Equal),
        TokenKind::NotEquals => Some(CallableKind::NotEqual),
        TokenKind::Less => Some(CallableKind::Less),
        TokenKind::Greater => Some(CallableKind::Greater),
        TokenKind::LessEqual => Some(CallableKind::LessEqual),
        TokenKind::GreaterEqual => Some(CallableKind::GreaterEqual),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex;

    #[test]
    fn retains_generic_application_order() {
        let source = SourceText::new("1 + 2 * 3").unwrap();
        let parsed = parse(&source, &lex(&source));
        let Statement::Expression(Expression::Application { items, .. }) = &parsed.statements[0]
        else {
            panic!("expected application");
        };
        assert_eq!(items.len(), 5);
        assert!(matches!(
            items[1],
            Expression::Callable {
                kind: CallableKind::Plus,
                ..
            }
        ));
        assert!(matches!(
            items[3],
            Expression::Callable {
                kind: CallableKind::Multiply,
                ..
            }
        ));
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn parses_the_zero_field_product_as_unit() {
        let source = SourceText::new("()").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(
            parsed.statements,
            vec![Statement::Expression(Expression::Unit(Span::new(0, 2)))]
        );
    }

    #[test]
    fn distinguishes_grouping_and_positional_products() {
        let grouped_source = SourceText::new("(1)").unwrap();
        let grouped = parse(&grouped_source, &lex(&grouped_source));
        assert!(matches!(
            grouped.statements[0],
            Statement::Expression(Expression::Integer(_))
        ));

        let tuple_source = SourceText::new("(1, 2,)").unwrap();
        let tuple = parse(&tuple_source, &lex(&tuple_source));
        let Statement::Expression(Expression::Product { fields, .. }) = &tuple.statements[0] else {
            panic!("expected tuple");
        };
        assert_eq!(fields.len(), 2);
        assert!(tuple.diagnostics.is_empty());
    }

    #[test]
    fn ignores_newlines_inside_parentheses() {
        let source = SourceText::new("(\n1,\n2\n)").unwrap();
        let parsed = parse(&source, &lex(&source));
        let Statement::Expression(Expression::Product { fields, .. }) = &parsed.statements[0]
        else {
            panic!("expected tuple");
        };
        assert_eq!(fields.len(), 2);
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn retains_labels_on_record_product_fields() {
        let source = SourceText::new("(name is \"Ada\", active is true)").unwrap();
        let parsed = parse(&source, &lex(&source));
        let Statement::Expression(Expression::Product { fields, .. }) = &parsed.statements[0]
        else {
            panic!("expected product");
        };
        assert_eq!(fields.len(), 2);
        assert_eq!(source.slice(fields[0].label.unwrap()), "name");
        assert_eq!(source.slice(fields[1].label.unwrap()), "active");
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn rejects_products_that_mix_positional_and_labeled_fields() {
        let source = SourceText::new("(1, name is \"Ada\")").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-MIXED-PRODUCT-FIELDS");
        assert_eq!(source.slice(parsed.diagnostics[0].span), "name");
    }

    #[test]
    fn retains_incomplete_parentheses_for_recovery() {
        let source = SourceText::new("(\n1,").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-EXPECTED-RPAREN");
    }

    #[test]
    fn retains_named_and_prefix_application_terms() {
        let named_source = SourceText::new("print value").unwrap();
        let named = parse(&named_source, &lex(&named_source));
        let Statement::Expression(Expression::Application { items, .. }) = &named.statements[0]
        else {
            panic!("expected named application");
        };
        assert_eq!(items.len(), 2);

        let prefix_source = SourceText::new("- 42").unwrap();
        let prefix = parse(&prefix_source, &lex(&prefix_source));
        let Statement::Expression(Expression::Application { items, .. }) = &prefix.statements[0]
        else {
            panic!("expected prefix application");
        };
        assert!(matches!(
            items[0],
            Expression::Callable {
                kind: CallableKind::Minus,
                ..
            }
        ));
    }

    #[test]
    fn retains_incomplete_application_for_semantic_recovery() {
        let source = SourceText::new("1 +").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(matches!(
            parsed.statements[0],
            Statement::Expression(Expression::Application { .. })
        ));
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn recovers_incomplete_binding() {
        let source = SourceText::new("answer is").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-EXPECTED-EXPRESSION");
    }

    #[test]
    fn rejects_boolean_literal_as_a_binding_name() {
        let source = SourceText::new("true is 1").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-RESERVED-BOOLEAN-LITERAL");
    }

    #[test]
    fn parses_static_nullary_function_with_indented_body() {
        let source = SourceText::new("answer is fn static () -> Int\n  40 + 2\nanswer ()").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        let Statement::Function {
            name, result, body, ..
        } = &parsed.statements[0]
        else {
            panic!("expected static function declaration");
        };
        assert_eq!(source.slice(*name), "answer");
        assert_eq!(source.slice(*result), "Int");
        assert!(matches!(
            body[0],
            Statement::Expression(Expression::Application { .. })
        ));
        assert!(matches!(parsed.statements[1], Statement::Expression(_)));
    }

    #[test]
    fn parses_one_typed_static_function_parameter() {
        let source = SourceText::new(
            "increment is fn static (input : Int) -> Int\n  input + 1\nincrement 41",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        let Statement::Function { parameters, .. } = &parsed.statements[0] else {
            panic!("expected one static parameter");
        };
        assert_eq!(parameters.len(), 1);
        assert_eq!(source.slice(parameters[0].name), "input");
        assert_eq!(source.slice(parameters[0].classifier), "Int");
    }

    #[test]
    fn parses_multiple_typed_static_function_parameters() {
        let source = SourceText::new(
            "add is fn static (left : Int, right : Int) -> Int\n  left + right\n20 add 22",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        let Statement::Function { parameters, .. } = &parsed.statements[0] else {
            panic!("expected static function parameters");
        };
        assert_eq!(parameters.len(), 2);
        assert_eq!(source.slice(parameters[0].name), "left");
        assert_eq!(source.slice(parameters[1].name), "right");
    }

    #[test]
    fn rejects_more_than_two_function_operands() {
        let source = SourceText::new(
            "invalid is fn static (one : Int, two : Int, three : Int) -> Int\n  one",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-FUNCTION-OPERAND-COUNT");
    }

    #[test]
    fn parses_a_multi_statement_function_body() {
        let source =
            SourceText::new("answer is fn static () -> Int\n  value is 40 + 2\n  value\nanswer ()")
                .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            panic!("expected static function body");
        };
        assert_eq!(body.len(), 2);
        assert!(matches!(body[0], Statement::Binding { .. }));
        assert!(matches!(body[1], Statement::Expression(_)));
        assert_eq!(parsed.statements.len(), 2);
    }

    #[test]
    fn parses_explicit_return_as_a_function_statement() {
        let source =
            SourceText::new("answer is fn static () -> Int\n  return 42\n  0\nanswer ()").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            panic!("expected static function body");
        };
        assert!(matches!(body[0], Statement::Return { .. }));
    }

    #[test]
    fn distinguishes_ordinary_and_static_function_headers() {
        let ordinary_source = SourceText::new("ordinary is fn () -> Int\n  42").unwrap();
        let ordinary = parse(&ordinary_source, &lex(&ordinary_source));
        let Statement::Function { is_static, .. } = ordinary.statements[0] else {
            panic!("expected ordinary function");
        };
        assert!(!is_static);

        let static_source = SourceText::new("compile is fn static () -> Int\n  42").unwrap();
        let static_function = parse(&static_source, &lex(&static_source));
        let Statement::Function { is_static, .. } = static_function.statements[0] else {
            panic!("expected static function");
        };
        assert!(is_static);
    }

    #[test]
    fn parses_complete_boolean_decision_table() {
        let source = SourceText::new(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    otherwise 0\nchoose true",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            panic!("expected function");
        };
        let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
            panic!("expected decision table");
        };
        assert_eq!(rules.len(), 2);
        assert!(matches!(
            rules[0].matcher,
            DecisionMatcher::Boolean { value: true, .. }
        ));
        assert!(matches!(rules[1].matcher, DecisionMatcher::Otherwise(_)));
    }

    #[test]
    fn parses_exhaustive_boolean_decision_without_otherwise() {
        let source = SourceText::new(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    false then 0\nchoose false",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            panic!("expected function");
        };
        let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
            panic!("expected decision table");
        };
        assert_eq!(rules.len(), 2);
        assert!(matches!(
            rules[0].matcher,
            DecisionMatcher::Boolean { value: true, .. }
        ));
        assert!(matches!(
            rules[1].matcher,
            DecisionMatcher::Boolean { value: false, .. }
        ));
    }

    #[test]
    fn preserves_named_enum_decision_matchers() {
        let source = SourceText::new(
            "name is fn (value : Color) -> String\n  value\n    Red then \"red\"\n    Green then \"green\"",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            panic!("expected function");
        };
        let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
            panic!("expected function decision table");
        };
        assert!(matches!(rules[0].matcher, DecisionMatcher::Identifier(_)));
        assert!(matches!(rules[1].matcher, DecisionMatcher::Identifier(_)));
    }

    #[test]
    fn parses_a_body_calling_a_later_function_declaration() {
        let source = SourceText::new(
            "first is fn (value : Int) -> Int\n  second value\nsecond is fn (value : Int) -> Int\n  value\nfirst 42",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(matches!(parsed.statements[0], Statement::Function { .. }));
        assert!(matches!(parsed.statements[1], Statement::Function { .. }));
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            unreachable!();
        };
        assert!(matches!(
            &body[0],
            Statement::Expression(Expression::Application { .. })
        ));
    }

    #[test]
    fn parses_mutually_recursive_function_declarations() {
        let source = SourceText::new(
            "even is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\neven 4",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(matches!(parsed.statements[0], Statement::Function { .. }));
        assert!(matches!(parsed.statements[1], Statement::Function { .. }));
    }

    #[test]
    fn parses_comparison_decision_matcher() {
        let source = SourceText::new(
            "minimum is fn (left : Int, right : Int) -> Int\n  left\n    < right then left\n    otherwise right\nminimum (1, 2)",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            panic!("expected function");
        };
        let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
            panic!("expected decision table");
        };
        assert!(matches!(
            &rules[0].matcher,
            DecisionMatcher::Comparison {
                kind: CallableKind::Less,
                ..
            }
        ));
    }

    #[test]
    fn parses_complete_comparison_operand_before_then() {
        let source = SourceText::new(
            "within is fn (value : Int, limit : Int) -> Boolean\n  value\n    < limit + 1 then true\n    otherwise false\n1 within 1",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            panic!("expected function");
        };
        let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
            panic!("expected decision table");
        };
        assert!(matches!(
            &rules[0].matcher,
            DecisionMatcher::Comparison {
                operand: Expression::Application { .. },
                ..
            }
        ));
    }

    #[test]
    fn parses_nested_function_declaration_in_body() {
        let source = SourceText::new(
            "answer is fn (input : Int) -> Int\n  helper is fn (value : Int) -> Int\n    value + input\n  helper 2\nanswer 40",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            panic!("expected outer function");
        };
        assert!(matches!(body[0], Statement::Function { .. }));
        assert!(matches!(body[1], Statement::Expression(_)));
    }

    #[test]
    fn parses_qualified_error_code_decision_matcher() {
        let source = SourceText::new(
            "describe is fn (attempt : Result) -> Int\n  attempt\n    Ok value then value\n    Error ( code is lang arithmetic division-by-zero ) then 0\n    Error problem then 1",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Function { body, .. } = &parsed.statements[0] else {
            panic!("expected function");
        };
        let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
            panic!("expected decision table");
        };
        let DecisionMatcher::ErrorCode {
            namespace,
            vocabulary,
            code,
            ..
        } = rules[1].matcher
        else {
            panic!("expected Error code matcher");
        };
        assert_eq!(source.slice(namespace), "lang");
        assert_eq!(source.slice(vocabulary), "arithmetic");
        assert_eq!(source.slice(code), "division-by-zero");
    }

    #[test]
    fn parses_classified_binding() {
        let source = SourceText::new("value : Rational is operation input").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Binding {
            name, classifier, ..
        } = parsed.statements[0]
        else {
            panic!("expected binding");
        };
        assert_eq!(source.slice(name), "value");
        assert_eq!(source.slice(classifier.unwrap()), "Rational");
    }

    #[test]
    fn parses_named_generator_declaration() {
        let source = SourceText::new(
            "once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Generator {
            name,
            parameters,
            yielded,
            resumed,
            result,
            body,
            ..
        } = &parsed.statements[0]
        else {
            panic!("expected generator");
        };
        assert_eq!(source.slice(*name), "once");
        assert_eq!(parameters.len(), 1);
        assert_eq!(source.slice(*yielded), "Character");
        assert_eq!(source.slice(*resumed), "Unit");
        assert_eq!(source.slice(*result), "Unit");
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn parses_multi_input_generator_declaration() {
        let source = SourceText::new(
            "select is generator ( value : Int, suffix : String )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield suffix\n  \"done\"",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Generator { parameters, .. } = &parsed.statements[0] else {
            panic!("expected generator");
        };
        assert_eq!(parameters.len(), 2);
        assert_eq!(source.slice(parameters[0].classifier), "Int");
        assert_eq!(source.slice(parameters[1].classifier), "String");
    }

    #[test]
    fn parses_foreach_result_binding() {
        let source =
            SourceText::new("result is generated foreach { value }\n  _ is value + 1").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Foreach {
            result,
            source: generator,
            ..
        } = &parsed.statements[0]
        else {
            panic!("expected foreach");
        };
        assert_eq!(source.slice(result.unwrap().0), "result");
        assert_eq!(source.slice(generator.span()), "generated");
    }

    #[test]
    fn parses_classified_foreach_result_binding() {
        let source =
            SourceText::new("result : String is generated foreach { value }\n  _ is value + 1")
                .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Foreach {
            result: Some((name, Some(classifier))),
            ..
        } = &parsed.statements[0]
        else {
            panic!("expected classified foreach result");
        };
        assert_eq!(source.slice(*name), "result");
        assert_eq!(source.slice(*classifier), "String");
    }

    #[test]
    fn parses_explicit_generator_return() {
        let source = SourceText::new(
            "done is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  return \"done\"",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Generator { body, .. } = &parsed.statements[0] else {
            panic!("expected generator");
        };
        assert!(matches!(body.as_slice(), [Statement::Return { .. }]));
    }

    #[test]
    fn parses_optional_generator_classifiers() {
        let source = SourceText::new(
            "optional is generator ( initial : Optional Int )\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n\n  _ is yield initial\n  None Int",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Generator {
            parameters,
            yielded,
            result,
            ..
        } = &parsed.statements[0]
        else {
            panic!("expected generator");
        };
        assert_eq!(source.slice(parameters[0].classifier), "Optional Int");
        assert_eq!(source.slice(*yielded), "Optional Int");
        assert_eq!(source.slice(*result), "Optional Int");
    }

    #[test]
    fn parses_product_generator_classifiers() {
        let source = SourceText::new("pair is generator ( initial : (Int, String) )\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n\n  _ is yield initial\n  (8, \"done\")").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Statement::Generator {
            parameters,
            yielded,
            result,
            ..
        } = &parsed.statements[0]
        else {
            panic!("expected generator")
        };
        assert_eq!(source.slice(parameters[0].classifier), "(Int, String)");
        assert_eq!(source.slice(*yielded), "(Int, String)");
        assert_eq!(source.slice(*result), "(Int, String)");
    }

    #[test]
    fn parses_result_generator_classifiers() {
        let source = SourceText::new(include_str!(
            "../../../examples/interpreter/custom-generator-result-values.t"
        ))
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn parses_compound_generator_function_classifiers() {
        let source = SourceText::new(include_str!(
            "../../../examples/interpreter/custom-generator-compound-function-boundaries.t"
        ))
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn parses_nested_generator_function_classifiers() {
        let source = SourceText::new(include_str!(
            "../../../examples/interpreter/custom-generator-nested-function-boundaries.t"
        ))
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn parses_list_generator_function_classifiers() {
        let source = SourceText::new(include_str!(
            "../../../examples/interpreter/custom-generator-list-values.t"
        ))
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn parses_nested_optional_generator_classifiers() {
        let source = SourceText::new(include_str!(
            "../../../examples/interpreter/custom-generator-nested-optional-values.t"
        ))
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn accepts_complete_closed_arithmetic_error_code_set() {
        let source = SourceText::new(
            "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic out-of-range ) then \"range\"\n    Error ( code is lang arithmetic not-representable ) then \"representation\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"\n    Error ( code is lang arithmetic indeterminate ) then \"indeterminate\"",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn incomplete_arithmetic_error_decision_lists_missing_codes() {
        let source = SourceText::new(
            "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        let diagnostic = parsed.diagnostics.first().unwrap();
        assert_eq!(diagnostic.code, "E-INCOMPLETE-ERROR-CODE-DECISION");
        assert!(diagnostic.message.contains("out-of-range"));
        assert!(diagnostic.message.contains("not-representable"));
        assert!(diagnostic.message.contains("indeterminate"));
    }

    #[test]
    fn duplicate_arithmetic_error_code_pattern_is_rejected() {
        let source = SourceText::new(
            "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"first\"\n    Error ( code is lang arithmetic division-by-zero ) then \"second\"\n    Error problem then \"other\"",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        let diagnostic = parsed.diagnostics.first().unwrap();
        assert_eq!(diagnostic.code, "E-DUPLICATE-ERROR-CODE-PATTERN");
        assert!(diagnostic.message.contains("division-by-zero"));
        assert_eq!(source.slice(diagnostic.span), "division-by-zero");
    }

    #[test]
    fn parses_qualified_generator_error_code_pattern() {
        let source = SourceText::new(
            "handle is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  result is yield initial\n  result\n    Error ( code is lang generator generator-closed ) then ()\n    Error problem then ()\n    Ok resumed then ()",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn error_code_pattern_after_generic_fallback_is_rejected() {
        let source = SourceText::new(
            "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error problem then \"other\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        let diagnostic = parsed.diagnostics.first().unwrap();
        assert_eq!(diagnostic.code, "E-UNREACHABLE-ERROR-CODE-PATTERN");
        assert_eq!(
            source.slice(diagnostic.span),
            "Error ( code is lang arithmetic division-by-zero )"
        );
    }

    #[test]
    fn rule_after_otherwise_is_rejected() {
        let source = SourceText::new(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    otherwise 0\n    true then 1",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        let diagnostic = parsed.diagnostics.first().unwrap();
        assert_eq!(diagnostic.code, "E-UNREACHABLE-DECISION-RULE");
        assert_eq!(source.slice(diagnostic.span), "true");
    }

    #[test]
    fn parses_list_construction_and_total_decomposition() {
        let source =
            SourceText::new(include_str!("../../../examples/interpreter/lists.t")).unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn parses_recursive_list_classifiers() {
        let source =
            SourceText::new(include_str!("../../../examples/interpreter/nested-lists.t")).unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn parses_contextual_anonymous_list_functions() {
        let source = SourceText::new(include_str!(
            "../../../examples/interpreter/anonymous-list-functions.t"
        ))
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn parses_payload_unions_and_positional_variants() {
        let source = SourceText::new(include_str!(
            "../../../examples/interpreter/unions-and-recursive-products.t"
        ))
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn malformed_source_corpus_never_panics() {
        for input in ["(", "{", "fn", "x is", "x\n  then", "\0", "😀 ( , )"] {
            if let Ok(source) = SourceText::new(input) {
                let _ = parse(&source, &lex(&source));
            }
        }
    }

    #[test]
    fn parses_balanced_diagnostic_controls() {
        let source = SourceText::new(include_str!(
            "../../../examples/interpreter/diagnostic-controls.t"
        ))
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(matches!(
            parsed.statements[0],
            Statement::DiagnosticControl {
                operation: DiagnosticControlKind::DisableNext,
                ..
            }
        ));
    }

    #[test]
    fn rejects_unbalanced_diagnostic_controls() {
        for (input, code) in [
            (
                "lang pop-disable-warning unused",
                "E-DIAGNOSTIC-CONTROL-UNDERFLOW",
            ),
            (
                "lang push-disable-warning unused\n()",
                "E-DIAGNOSTIC-CONTROL-UNCLOSED",
            ),
            ("lang disable-warning unused", "E-DIAGNOSTIC-CONTROL-TARGET"),
        ] {
            let source = SourceText::new(input).unwrap();
            let parsed = parse(&source, &lex(&source));
            assert!(parsed.diagnostics.iter().any(|error| error.code == code));
        }
    }

    #[test]
    fn preserves_publication_on_declarations() {
        let source = SourceText::new("pub answer is 42").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(matches!(
            parsed.statements.as_slice(),
            [Statement::Published { declaration, .. }]
                if matches!(declaration.as_ref(), Statement::Binding { .. })
        ));
    }

    #[test]
    fn rejects_publication_on_expressions() {
        let source = SourceText::new("pub 42").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(
            parsed
                .diagnostics
                .iter()
                .any(|error| error.code == "E-PUBLICATION-TARGET")
        );
    }
}
