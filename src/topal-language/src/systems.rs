//! Checked source model for the initial freestanding systems profile.
//!
//! This module deliberately stops before target lowering. It establishes the
//! sealed source vocabulary and abstract transitions which a later qualified
//! provider may consume without reinterpreting syntax.

use std::collections::BTreeMap;

use topal_source::{Diagnostic, SourceText, Span};
use topal_syntax::{Expression, FunctionClauses, ProductField, Statement, lex, parse};

pub use topal_semantics::{
    INITIAL_SYSTEMS_BOARD, INITIAL_SYSTEMS_PROFILE, INITIAL_SYSTEMS_TARGET, SYSTEMS_CONSOLE_WRITE,
    SYSTEMS_DEBUG_BREAK, SYSTEMS_FATAL, SYSTEMS_RESUME_DEBUG_BREAK,
    SystemsContextKind as CompilerSystemsContextKind,
    SystemsDisposition as CompilerSystemsDisposition, SystemsEntry as CompilerSystemsEntry,
    SystemsEntryKind as CompilerSystemsEntryKind, SystemsHandler as CompilerSystemsHandler,
    SystemsOperation as CompilerSystemsOperation, SystemsProgram as CompilerSystemsProgram,
    SystemsTargetSelection as CompilerSystemsTargetSelection,
    SystemsTransition as CompilerSystemsTransition, model_systems_transitions,
    validate_systems_program,
};

use crate::source::parse_string;

/// Check one initial systems root without making its target executable.
///
/// # Errors
///
/// Returns a source diagnostic for the wrong target, any syntax error, an
/// unknown systems member, an invalid entry signature, or an affine-context
/// violation.
pub fn analyze_systems_for_compiler(
    text: &str,
    target: &CompilerSystemsTargetSelection,
) -> Result<CompilerSystemsProgram, Diagnostic> {
    validate_target(text, target)?;
    let source = SourceText::new(text).map_err(|error| {
        Diagnostic::error(error.code, 1, 1, error.message).with_source_span(error.span)
    })?;
    let lexed = lex(&source);
    if let Some(error) = lexed.diagnostics.first() {
        return Err(source_diagnostic(
            &source,
            error.code,
            error.span,
            error.message.clone(),
        ));
    }
    let parsed = parse(&source, &lexed);
    if let Some(error) = parsed.diagnostics.first() {
        return Err(source_diagnostic(
            &source,
            error.code,
            error.span,
            error.message.clone(),
        ));
    }
    let Some((selection, remaining)) = parsed.statements.split_first() else {
        return Err(source_diagnostic(
            &source,
            "E-SYSTEMS-CONTEXT",
            Span::new(0, 0),
            "a systems root begins with `use language ( version is v0.1, features is ( systems ) )`",
        ));
    };
    validate_language_selection(&source, selection)?;
    let Some((root, declarations)) = remaining.split_last() else {
        return Err(source_diagnostic(
            &source,
            "E-SYSTEMS-ARTIFACT",
            statement_span(selection),
            "a systems root requires one `lang systems artifact`",
        ));
    };

    let functions = collect_handler_declarations(&source, declarations)?;

    let root_entries = parse_artifact_root(&source, root)?;
    if functions.len() != 2 {
        return Err(source_diagnostic(
            &source,
            "E-SYSTEMS-ENTRY-SET",
            statement_span(root),
            "the initial systems artifact requires exactly its bootstrap and debug-break handlers",
        ));
    }
    if root_entries.bootstrap == root_entries.debug_break {
        return Err(source_diagnostic(
            &source,
            "E-SYSTEMS-ENTRY-SET",
            statement_span(root),
            "bootstrap and debug-break entries require distinct handlers",
        ));
    }
    let bootstrap = analyze_named_handler(
        &source,
        &functions,
        root,
        "bootstrap",
        &root_entries.bootstrap,
        CompilerSystemsContextKind::Bootstrap,
    )?;
    let debug_break = analyze_named_handler(
        &source,
        &functions,
        root,
        "debug-break",
        &root_entries.debug_break,
        CompilerSystemsContextKind::DebugBreak,
    )?;

    Ok(CompilerSystemsProgram {
        target: target.clone(),
        bootstrap: CompilerSystemsEntry {
            kind: CompilerSystemsEntryKind::Bootstrap,
            handler: bootstrap,
        },
        debug_break: CompilerSystemsEntry {
            kind: CompilerSystemsEntryKind::SynchronousExceptionDebugBreak,
            handler: debug_break,
        },
    })
}

fn analyze_named_handler(
    source: &SourceText,
    functions: &BTreeMap<String, &Statement>,
    root: &Statement,
    entry_name: &str,
    handler_name: &str,
    context: CompilerSystemsContextKind,
) -> Result<CompilerSystemsHandler, Diagnostic> {
    let handler = functions.get(handler_name).copied().ok_or_else(|| {
        source_diagnostic(
            source,
            "E-SYSTEMS-HANDLER",
            statement_span(root),
            format!("{entry_name} entry names unknown handler `{handler_name}`"),
        )
    })?;
    analyze_handler(source, handler, context)
}

fn collect_handler_declarations<'source>(
    source: &SourceText,
    declarations: &'source [Statement],
) -> Result<BTreeMap<String, &'source Statement>, Diagnostic> {
    let mut functions = BTreeMap::new();
    for declaration in declarations {
        let Statement::Function { name, .. } = declaration else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DECLARATION",
                statement_span(declaration),
                "the initial systems root admits only entry-handler functions before its artifact",
            ));
        };
        let name_text = source.slice(*name).to_owned();
        if functions.insert(name_text.clone(), declaration).is_some() {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-BINDING",
                *name,
                format!("systems handler `{name_text}` is declared more than once"),
            ));
        }
    }
    Ok(functions)
}

fn validate_target(text: &str, target: &CompilerSystemsTargetSelection) -> Result<(), Diagnostic> {
    if target.target == INITIAL_SYSTEMS_TARGET
        && target.board == INITIAL_SYSTEMS_BOARD
        && target.profile == INITIAL_SYSTEMS_PROFILE
    {
        return Ok(());
    }
    Err(Diagnostic::error(
        "E-SYSTEMS-TARGET",
        1,
        1,
        format!(
            "systems semantic checking requires target `{INITIAL_SYSTEMS_TARGET}`, board `{INITIAL_SYSTEMS_BOARD}`, and profile `{INITIAL_SYSTEMS_PROFILE}`"
        ),
    )
    .with_source_span(Span::new(0, text.len().min(1))))
}

fn validate_language_selection(
    source: &SourceText,
    statement: &Statement,
) -> Result<(), Diagnostic> {
    let Statement::LanguageSelection {
        version,
        features,
        span,
    } = statement
    else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-CONTEXT",
            statement_span(statement),
            "a systems root begins with a language selection",
        ));
    };
    if source.slice(*version) != "v0.1"
        || features.len() != 1
        || source.slice(features[0]) != "systems"
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-CONTEXT",
            *span,
            "the initial systems root requires exactly language v0.1 with feature `systems`",
        ));
    }
    Ok(())
}

struct ArtifactEntries {
    bootstrap: String,
    debug_break: String,
}

fn parse_artifact_root(
    source: &SourceText,
    statement: &Statement,
) -> Result<ArtifactEntries, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ARTIFACT",
            statement_span(statement),
            "the final root expression must construct `lang systems artifact`",
        ));
    };
    let [lang, systems, artifact, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ARTIFACT",
            *span,
            "the final root expression must be `lang systems artifact (...)`",
        ));
    };
    if !identifier_is(source, lang, "lang")
        || !identifier_is(source, systems, "systems")
        || !identifier_is(source, artifact, "artifact")
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ARTIFACT",
            *span,
            "the final root expression must be `lang systems artifact (...)`",
        ));
    }
    let mut entries = BTreeMap::new();
    for field in fields {
        let (label, label_text, handler) = parse_artifact_field(source, field)?;
        if entries.insert(label_text.clone(), handler).is_some() {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-ENTRY-SET",
                label,
                format!("systems artifact entry `{label_text}` is duplicated"),
            ));
        }
    }
    let (Some(bootstrap), Some(debug_break)) =
        (entries.remove("bootstrap"), entries.remove("debug-break"))
    else {
        return Err(invalid_entry_set(source, *span));
    };
    if !entries.is_empty() {
        return Err(invalid_entry_set(source, *span));
    }
    Ok(ArtifactEntries {
        bootstrap,
        debug_break,
    })
}

fn parse_artifact_field(
    source: &SourceText,
    field: &ProductField,
) -> Result<(Span, String, String), Diagnostic> {
    let Some(label) = field.label else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTRY-SET",
            field.value.span(),
            "systems artifact entries require named `bootstrap` and `debug-break` fields",
        ));
    };
    let label_text = source.slice(label);
    let expected_constructor = match label_text {
        "bootstrap" => "bootstrap-entry",
        "debug-break" => "synchronous-exception-entry",
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-ENTRY-SET",
                label,
                format!("unknown initial systems artifact entry `{label_text}`"),
            ));
        }
    };
    let Expression::Application {
        items: entry_items,
        span: entry_span,
    } = &field.value
    else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTRY",
            field.value.span(),
            format!("`{label_text}` must contain a `lang systems {expected_constructor}`"),
        ));
    };
    let [entry_lang, entry_systems, constructor, handler] = entry_items.as_slice() else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTRY",
            *entry_span,
            format!("`{label_text}` must contain one handler"),
        ));
    };
    let Expression::Identifier(handler) = handler else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-HANDLER",
            handler.span(),
            "a systems entry handler must be a statically named function",
        ));
    };
    if !identifier_is(source, entry_lang, "lang")
        || !identifier_is(source, entry_systems, "systems")
        || !identifier_is(source, constructor, expected_constructor)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTRY",
            *entry_span,
            format!("`{label_text}` requires `lang systems {expected_constructor}`"),
        ));
    }
    Ok((
        label,
        label_text.to_owned(),
        source.slice(*handler).to_owned(),
    ))
}

fn invalid_entry_set(source: &SourceText, span: Span) -> Diagnostic {
    source_diagnostic(
        source,
        "E-SYSTEMS-ENTRY-SET",
        span,
        "the initial systems artifact requires exactly `bootstrap` and `debug-break` entries",
    )
}

fn analyze_handler(
    source: &SourceText,
    statement: &Statement,
    context: CompilerSystemsContextKind,
) -> Result<CompilerSystemsHandler, Diagnostic> {
    let Statement::Function {
        name,
        is_static,
        parameters,
        result,
        effect_bound,
        clauses,
        body,
        span,
    } = statement
    else {
        unreachable!("handler table retains only functions")
    };
    let (context_classifier, disposition_classifier) = match context {
        CompilerSystemsContextKind::Bootstrap => ("BootstrapContext", "BootstrapDisposition"),
        CompilerSystemsContextKind::DebugBreak => ("DebugBreakContext", "DebugBreakDisposition"),
    };
    if *is_static
        || parameters.len() != 1
        || source.slice(parameters[0].classifier) != context_classifier
        || source.slice(*result) != disposition_classifier
        || parameters[0].qualifier.is_some()
        || !parameters[0].fields.is_empty()
        || parameters[0].default.is_some()
        || effect_bound.is_some()
        || !clauses_are_empty(clauses)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-HANDLER",
            *span,
            format!(
                "handler `{}` must have exactly `fn (context : {context_classifier}) -> {disposition_classifier}`",
                source.slice(*name)
            ),
        ));
    }
    let context_name = source.slice(parameters[0].name);
    let Some((last, operations)) = body.split_last() else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DISPOSITION",
            *span,
            "a systems handler requires one final disposition",
        ));
    };
    let mut checked_operations = Vec::new();
    for operation in operations {
        checked_operations.push(analyze_operation(source, operation, context, context_name)?);
    }
    let disposition = analyze_disposition(source, last, context, context_name)?;
    let mut effects = checked_operations
        .iter()
        .map(|operation| operation.semantic_identity().to_owned())
        .collect::<Vec<_>>();
    effects.push(disposition.semantic_identity().to_owned());
    effects.sort();
    effects.dedup();
    Ok(CompilerSystemsHandler {
        name: source.slice(*name).to_owned(),
        context,
        operations: checked_operations,
        disposition,
        effects,
    })
}

fn analyze_operation(
    source: &SourceText,
    statement: &Statement,
    context: CompilerSystemsContextKind,
    context_name: &str,
) -> Result<CompilerSystemsOperation, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(affine_context_error(source, statement_span(statement)));
    };
    if let [receiver, console, write, Expression::String(text)] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, console, "console")
        && identifier_is(source, write, "write")
    {
        let text = parse_string(source.slice(*text)).ok_or_else(|| {
            source_diagnostic(
                source,
                "E-SYSTEMS-CONSOLE",
                *text,
                "invalid static console text",
            )
        })?;
        return Ok(CompilerSystemsOperation::ConsoleWrite { text: text.into() });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [receiver, debug, break_operation] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, debug, "debug")
        && identifier_is(source, break_operation, "break")
    {
        return Ok(CompilerSystemsOperation::DebugBreak);
    }
    Err(source_diagnostic(
        source,
        "E-SYSTEMS-OPERATION",
        *span,
        "operation is not admitted by this live systems entry context",
    ))
}

fn analyze_disposition(
    source: &SourceText,
    statement: &Statement,
    context: CompilerSystemsContextKind,
    context_name: &str,
) -> Result<CompilerSystemsDisposition, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DISPOSITION",
            statement_span(statement),
            "a systems handler must end in one context-consuming disposition",
        ));
    };
    if context == CompilerSystemsContextKind::DebugBreak
        && let [receiver, resume] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, resume, "resume")
    {
        return Ok(CompilerSystemsDisposition::Resume);
    }
    if let [receiver, fatal, Expression::String(message)] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, fatal, "fatal")
    {
        let message = parse_string(source.slice(*message)).ok_or_else(|| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DISPOSITION",
                *message,
                "invalid fatal text",
            )
        })?;
        return Ok(CompilerSystemsDisposition::Fatal {
            message: message.into(),
        });
    }
    Err(source_diagnostic(
        source,
        "E-SYSTEMS-DISPOSITION",
        *span,
        "final operation is not a disposition admitted by this entry context",
    ))
}

fn clauses_are_empty(clauses: &FunctionClauses) -> bool {
    clauses.requires.is_none()
        && clauses.effects.is_none()
        && clauses.guarantees.is_none()
        && clauses.result_binding.is_none()
        && clauses.ensures.is_none()
}

fn affine_context_error(source: &SourceText, span: Span) -> Diagnostic {
    source_diagnostic(
        source,
        "E-SYSTEMS-AFFINE-CONTEXT",
        span,
        "a systems context may occur only as the receiver of an admitted operation and may not be rebound, stored, passed, captured, or returned",
    )
}

fn identifier_is(source: &SourceText, expression: &Expression, expected: &str) -> bool {
    matches!(expression, Expression::Identifier(span) if source.slice(*span) == expected)
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
    use crate::analyze_for_compiler;

    const SOURCE: &str = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");

    #[test]
    fn checks_the_initial_artifact_and_models_entry_transitions() {
        // TOPAL-SYSTEMS-VOCABULARY-001, TOPAL-SYSTEMS-ENTRY-001,
        // TOPAL-SYSTEMS-DISPOSITION-001, TOPAL-SYSTEMS-OBSERVATION-001.
        let program = analyze_systems_for_compiler(
            SOURCE,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        assert_eq!(program.bootstrap.kind, CompilerSystemsEntryKind::Bootstrap);
        assert_eq!(
            program.debug_break.kind,
            CompilerSystemsEntryKind::SynchronousExceptionDebugBreak
        );
        assert_eq!(
            program.bootstrap.handler.effects,
            [SYSTEMS_CONSOLE_WRITE, SYSTEMS_FATAL, SYSTEMS_DEBUG_BREAK]
        );
        assert_eq!(
            model_systems_transitions(&program).unwrap(),
            vec![
                CompilerSystemsTransition::EnterBootstrap,
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_BOOT".into(),
                },
                CompilerSystemsTransition::ObserveDebugBreak,
                CompilerSystemsTransition::EnterDebugBreak,
                CompilerSystemsTransition::ResumeDebugBreak,
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_FAULT_RESUMED".into(),
                },
                CompilerSystemsTransition::Fatal {
                    message: "toolchain gate complete".into(),
                },
            ]
        );
    }

    #[test]
    fn rejects_hosted_and_mismatched_target_profiles() {
        // TOPAL-SYSTEMS-FEATURE-001, TOPAL-SYSTEMS-QUALIFY-001.
        let hosted = CompilerSystemsTargetSelection {
            target: "x86_64-unknown-linux-gnu".into(),
            board: INITIAL_SYSTEMS_BOARD.into(),
            profile: INITIAL_SYSTEMS_PROFILE.into(),
        };
        assert_eq!(
            analyze_systems_for_compiler(SOURCE, &hosted)
                .unwrap_err()
                .code,
            "E-SYSTEMS-TARGET"
        );
        let wrong_board = CompilerSystemsTargetSelection {
            target: INITIAL_SYSTEMS_TARGET.into(),
            board: "another-board".into(),
            profile: INITIAL_SYSTEMS_PROFILE.into(),
        };
        assert_eq!(
            analyze_systems_for_compiler(SOURCE, &wrong_board)
                .unwrap_err()
                .code,
            "E-SYSTEMS-TARGET"
        );
        let hosted_compiler = analyze_for_compiler(SOURCE).unwrap_err();
        assert_eq!(hosted_compiler.code, "E-COMPILER-UNSUPPORTED");
        assert!(
            hosted_compiler
                .message
                .contains("`systems` language feature")
        );
    }

    #[test]
    fn rejects_context_escape_unknown_operations_and_missing_dispositions() {
        // TOPAL-SYSTEMS-AUTHORITY-001, TOPAL-SYSTEMS-DISPOSITION-001.
        let escaped = SOURCE.replace(
            "context console write \"TOPAL_KERNEL_BOOT\"",
            "saved is context",
        );
        assert_eq!(
            analyze_systems_for_compiler(
                &escaped,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-AFFINE-CONTEXT"
        );

        let unknown = SOURCE.replace("context debug break", "context machine instruction");
        assert_eq!(
            analyze_systems_for_compiler(
                &unknown,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-OPERATION"
        );

        let missing = SOURCE.replace("  context fatal \"toolchain gate complete\"\n", "");
        assert_eq!(
            analyze_systems_for_compiler(
                &missing,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-DISPOSITION"
        );
    }

    #[test]
    fn rejects_wrong_handler_signatures_and_open_entry_sets() {
        // TOPAL-SYSTEMS-ENTRY-001, TOPAL-SYSTEMS-VOCABULARY-001.
        let wrong_signature =
            SOURCE.replace("context : BootstrapContext", "context : DebugBreakContext");
        assert_eq!(
            analyze_systems_for_compiler(
                &wrong_signature,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-HANDLER"
        );

        let open_entries = SOURCE.replace(
            "  debug-break is lang systems synchronous-exception-entry debug-break-handler\n",
            "  debug-break is lang systems synchronous-exception-entry debug-break-handler,\n  interrupt is lang systems interrupt-entry interrupt-handler\n",
        );
        assert_eq!(
            analyze_systems_for_compiler(
                &open_entries,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-ENTRY-SET"
        );
    }

    #[test]
    fn a_fatal_debug_break_disposition_stops_the_bootstrap_model() {
        let fatal_debug = SOURCE.replace("context resume", "context fatal \"debug fatal\"");
        let program = analyze_systems_for_compiler(
            &fatal_debug,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let transitions = model_systems_transitions(&program).unwrap();
        assert_eq!(
            transitions.last(),
            Some(&CompilerSystemsTransition::Fatal {
                message: "debug fatal".into(),
            })
        );
        assert!(
            !transitions.contains(&CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_FAULT_RESUMED".into(),
            })
        );
    }
}
