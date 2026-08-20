use topal_source::{SourceText, Span};

use crate::{FunctionParameter, Lexed, ParsedSource, Statement, Token, TokenKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentedParameter {
    pub name: String,
    pub syntax: String,
    pub documentation: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentedDeclaration {
    pub name: String,
    pub kind: String,
    pub syntax: String,
    pub documentation: Option<String>,
    pub parameters: Vec<DocumentedParameter>,
}

#[must_use]
pub fn extract_documentation(
    source: &SourceText,
    lexed: &Lexed,
    parsed: &ParsedSource,
) -> Vec<DocumentedDeclaration> {
    let mut declarations = Vec::new();
    let mut previous_end = 0;
    for statement in &parsed.statements {
        if let Some(declaration) = declaration(source, &lexed.tokens, statement, previous_end) {
            declarations.push(declaration);
        }
        previous_end = statement_extent(statement).end;
    }
    declarations
}

fn declaration(
    source: &SourceText,
    tokens: &[Token],
    statement: &Statement,
    lower_bound: usize,
) -> Option<DocumentedDeclaration> {
    let (inner, syntax_start) = match statement {
        Statement::Published { declaration, span } => (declaration.as_ref(), span.start),
        _ => (statement, statement_extent(statement).start),
    };
    let (name_span, kind, syntax_end, parameters) = match inner {
        Statement::Binding { name, value, .. } => (
            *name,
            "binding",
            line_end(source, value.span().end),
            &[][..],
        ),
        Statement::StateField { name, classifier } => {
            (*name, "field", line_end(source, classifier.end), &[][..])
        }
        Statement::Function {
            name,
            parameters,
            result,
            effect_bound,
            ..
        } => (
            *name,
            "function",
            line_end(source, effect_bound.unwrap_or(*result).end),
            parameters.as_slice(),
        ),
        Statement::Generator {
            name,
            parameters,
            result,
            ..
        } => (
            *name,
            "generator",
            line_end(source, result.end),
            parameters.as_slice(),
        ),
        Statement::Union { name, span, .. } => (*name, "type", line_end(source, span.end), &[][..]),
        Statement::Interface { name, span, .. } => {
            (*name, "interface", line_end(source, span.end), &[][..])
        }
        Statement::Implementation { name, span, .. } => {
            (*name, "implementation", line_end(source, span.end), &[][..])
        }
        _ => return None,
    };
    Some(DocumentedDeclaration {
        name: source.slice(name_span).to_owned(),
        kind: kind.to_owned(),
        syntax: source.as_str()[syntax_start..syntax_end]
            .trim_end()
            .to_owned(),
        documentation: documentation_before(source, tokens, syntax_start, lower_bound),
        parameters: documented_parameters(source, tokens, parameters, name_span.end),
    })
}

fn documented_parameters(
    source: &SourceText,
    tokens: &[Token],
    parameters: &[FunctionParameter],
    mut lower_bound: usize,
) -> Vec<DocumentedParameter> {
    parameters
        .iter()
        .map(|parameter| {
            let end = parameter
                .default
                .as_ref()
                .map_or(parameter.classifier.end, |value| value.span().end);
            let documented = DocumentedParameter {
                name: source.slice(parameter.name).to_owned(),
                syntax: source.as_str()[parameter.name.start..end].trim().to_owned(),
                documentation: documentation_before(
                    source,
                    tokens,
                    parameter.name.start,
                    lower_bound,
                ),
            };
            lower_bound = end;
            documented
        })
        .collect()
}

fn documentation_before(
    source: &SourceText,
    tokens: &[Token],
    target: usize,
    lower_bound: usize,
) -> Option<String> {
    let preceding = tokens
        .iter()
        .filter(|token| token.span.start >= lower_bound && token.span.end <= target)
        .collect::<Vec<_>>();
    let mut lines = Vec::new();
    let mut found = false;
    for token in preceding.into_iter().rev() {
        match token.kind {
            TokenKind::Documentation => {
                found = true;
                let text = source
                    .slice(token.span)
                    .strip_prefix("###")
                    .unwrap_or_default();
                lines.push(text.strip_prefix(' ').unwrap_or(text).to_owned());
            }
            TokenKind::Whitespace | TokenKind::Newline | TokenKind::Comment => {}
            _ if found => break,
            _ => return None,
        }
    }
    if lines.is_empty() {
        None
    } else {
        lines.reverse();
        Some(lines.join("\n"))
    }
}

fn line_end(source: &SourceText, offset: usize) -> usize {
    source.as_str()[offset..]
        .find('\n')
        .map_or(source.as_str().len(), |relative| offset + relative)
}

fn statement_extent(statement: &Statement) -> Span {
    match statement {
        Statement::LanguageSelection { span, .. }
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
        Statement::Binding { name, value, .. } => Span::new(name.start, value.span().end),
        Statement::StateField { name, classifier } => Span::new(name.start, classifier.end),
        Statement::Return { keyword, value } => Span::new(keyword.start, value.span().end),
        Statement::Expression(expression) => expression.span(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lex, parse};

    #[test]
    fn attaches_blocks_to_declarations_and_parameters() {
        let source = SourceText::new(
            "### Compare values.\n# Kept across an ordinary comment.\npub compare is fn (\n  ### Used when values tie.\n  left : Int,\n  right : Int\n) -> Int\n  left\n",
        )
        .unwrap();
        let lexed = lex(&source);
        assert_eq!(
            lexed
                .tokens
                .iter()
                .filter(|token| token.kind == TokenKind::Documentation)
                .count(),
            2
        );
        let parsed = parse(&source, &lexed);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let declarations = extract_documentation(&source, &lexed, &parsed);
        assert_eq!(declarations.len(), 1);
        assert_eq!(
            declarations[0].documentation.as_deref(),
            Some("Compare values.")
        );
        assert_eq!(
            declarations[0].parameters[0].documentation.as_deref(),
            Some("Used when values tie.")
        );
        assert_eq!(declarations[0].parameters[1].documentation, None);
    }
}
