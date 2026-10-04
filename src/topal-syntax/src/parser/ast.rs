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
    Infinity(Span),
    Measured {
        value: Span,
        unit: Span,
        span: Span,
    },
    Rational(Span),
    String(Span),
    Identifier(Span),
    ContextIdentifier(Span),
    Discard(Span),
    AnonymousFunction {
        parameters: Vec<AnonymousPattern>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnonymousPattern {
    Binding(Span),
    Product {
        fields: Vec<AnonymousPattern>,
        span: Span,
    },
}

impl AnonymousPattern {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Binding(span) | Self::Product { span, .. } => *span,
        }
    }
}

fn anonymous_pattern_tokens(tokens: &[&Token], index: &mut usize) -> bool {
    let Some(token) = tokens.get(*index) else {
        return false;
    };
    if matches!(token.kind, TokenKind::Identifier | TokenKind::Discard) {
        *index += 1;
        return true;
    }
    if token.kind != TokenKind::LeftParen {
        return false;
    }
    *index += 1;
    let mut fields = 0_usize;
    loop {
        if !anonymous_pattern_tokens(tokens, index) {
            return false;
        }
        fields += 1;
        let Some(separator) = tokens.get(*index) else {
            return false;
        };
        match separator.kind {
            TokenKind::Comma => *index += 1,
            TokenKind::RightParen => {
                *index += 1;
                return fields > 0;
            }
            _ => return false,
        }
    }
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
    RangeOpen,
    RangeInclusive,
    RangeOpenInclusive,
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
            | Self::Infinity(span)
            | Self::Rational(span)
            | Self::String(span)
            | Self::Identifier(span)
            | Self::ContextIdentifier(span)
            | Self::Discard(span)
            | Self::Callable { span, .. }
            | Self::Product { span, .. }
            | Self::Block { span, .. }
            | Self::DecisionTable { span, .. }
            | Self::AnonymousFunction { span, .. }
            | Self::Application { span, .. }
            | Self::Measured { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductField {
    pub label: Option<Span>,
    pub value: Expression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionParameter {
    pub name: Span,
    pub classifier: Span,
    pub qualifier: Option<Span>,
    pub fields: Vec<Self>,
    pub default: Option<Expression>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FunctionClauses {
    pub requires: Option<Box<Expression>>,
    pub effects: Option<Box<Expression>>,
    pub guarantees: Option<Box<Expression>>,
    pub result_binding: Option<Span>,
    pub ensures: Option<Box<Expression>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceFunction {
    pub name: Span,
    pub parameters: Vec<FunctionParameter>,
    pub result: Span,
    pub clauses: FunctionClauses,
    pub span: Span,
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
    LanguageSelection {
        version: Span,
        features: Vec<Span>,
        span: Span,
    },
    LibrarySelection {
        name: Span,
        version: Span,
        span: Span,
    },
    Published {
        declaration: Box<Self>,
        span: Span,
    },
    DiagnosticControl {
        operation: DiagnosticControlKind,
        identity: Vec<Span>,
        span: Span,
    },
    Binding {
        name: Span,
        classifier: Option<Span>,
        value: Expression,
    },
    Implementation {
        name: Span,
        classifier: Expression,
        declarations: Vec<Self>,
        span: Span,
    },
    StateField {
        name: Span,
        classifier: Span,
    },
    ContextAssignment {
        name: Span,
        value: Expression,
        span: Span,
    },
    Function {
        name: Span,
        is_static: bool,
        parameters: Vec<FunctionParameter>,
        result: Span,
        effect_bound: Option<Span>,
        clauses: Box<FunctionClauses>,
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
    Interface {
        name: Span,
        functions: Vec<InterfaceFunction>,
        span: Span,
    },
    InterfaceImplementation {
        interface: Span,
        declarations: Vec<Statement>,
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
        current_indent: 0,
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
            parser.error_current("E-UNEXPECTED-TOKEN", "unexpected token after expression");
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
    current_indent: usize,
    diagnostics: Vec<SyntaxDiagnostic>,
}
