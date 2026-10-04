fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::Binding { name, value, .. } => Span::new(name.start, value.span().end),
        Statement::LanguageSelection { span, .. }
        | Statement::LibrarySelection { span, .. }
        | Statement::Published { span, .. }
        | Statement::DiagnosticControl { span, .. }
        | Statement::Implementation { span, .. }
        | Statement::ContextAssignment { span, .. }
        | Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Interface { span, .. }
        | Statement::InterfaceImplementation { span, .. }
        | Statement::Foreach { span, .. }
        | Statement::Discard { span, .. } => *span,
        Statement::StateField { name, classifier } => Span::new(name.start, classifier.end),
        Statement::Return { keyword, value } => Span::new(keyword.start, value.span().end),
        Statement::Expression(expression) => expression.span(),
    }
}

fn validate_diagnostic_controls(
    source: &SourceText,
    statements: &[Statement],
    diagnostics: &mut Vec<SyntaxDiagnostic>,
) {
    let mut stack = Vec::<Vec<Span>>::new();
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
                identity,
                span,
            } => match operation {
                DiagnosticControlKind::DisableNext => pending = Some((identity.clone(), *span)),
                DiagnosticControlKind::Push => stack.push(identity.clone()),
                DiagnosticControlKind::Pop => match stack.last() {
                    None => diagnostics.push(SyntaxDiagnostic {
                        code: "E-DIAGNOSTIC-CONTROL-UNDERFLOW",
                        span: *span,
                        message: "cannot pop an empty diagnostic-suppression stack".into(),
                    }),
                    Some(active) if !diagnostic_identities_equal(source, active, identity) => {
                        diagnostics.push(SyntaxDiagnostic {
                            code: "E-DIAGNOSTIC-CONTROL-MISMATCH",
                            span: identity[0],
                            message: format!(
                                "cannot pop diagnostic `{}` while `{}` is active",
                                diagnostic_identity(source, identity),
                                diagnostic_identity(source, active)
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
    if let Some((identity, span)) = pending {
        diagnostics.push(SyntaxDiagnostic {
            code: "E-DIAGNOSTIC-CONTROL-TARGET",
            span,
            message: format!(
                "diagnostic suppression for `{}` has no following statement",
                diagnostic_identity(source, &identity)
            ),
        });
    }
    if let Some(identity) = stack.last() {
        diagnostics.push(SyntaxDiagnostic {
            code: "E-DIAGNOSTIC-CONTROL-UNCLOSED",
            span: identity[0],
            message: format!(
                "diagnostic suppression for `{}` remains active at the context boundary",
                diagnostic_identity(source, identity)
            ),
        });
    }
}

fn diagnostic_identity(source: &SourceText, identity: &[Span]) -> String {
    identity
        .iter()
        .map(|component| source.slice(*component))
        .collect::<Vec<_>>()
        .join(" ")
}

fn diagnostic_identities_equal(source: &SourceText, left: &[Span], right: &[Span]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| source.slice(*left) == source.slice(*right))
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
