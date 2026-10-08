//! Checked source model for the initial freestanding systems profile.
//!
//! This module deliberately stops before target lowering. It establishes the
//! sealed source vocabulary and abstract transitions which a later qualified
//! provider may consume without reinterpreting syntax.

use std::collections::BTreeMap;

use topal_source::{Diagnostic, SourceText, Span};
use topal_syntax::{
    CallableKind, DecisionMatcher, DecisionRule, Expression, FunctionClauses, ProductField,
    Statement, lex, parse,
};

pub use topal_semantics::{
    BootstrapRegion as CompilerBootstrapRegion,
    BootstrapStorageDescriptor as CompilerBootstrapStorageDescriptor,
    BootstrapStorageErrorCode as CompilerBootstrapStorageErrorCode,
    BootstrapStoragePlacement as CompilerBootstrapStoragePlacement,
    BootstrapStorageRequest as CompilerBootstrapStorageRequest,
    BootstrapStorageState as CompilerBootstrapStorageState,
    BootstrapStorageTransition as CompilerBootstrapStorageTransition, INITIAL_SYSTEMS_BOARD,
    INITIAL_SYSTEMS_PROFILE, INITIAL_SYSTEMS_TARGET,
    PhysicalFrameRequest as CompilerPhysicalFrameRequest, SYSTEMS_BOOT_MEMORY_DESCRIBE,
    SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE, SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
    SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE, SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE,
    SYSTEMS_BOOTSTRAP_STORAGE_EXHAUSTED, SYSTEMS_BOOTSTRAP_STORAGE_INVALID_REQUEST,
    SYSTEMS_BOOTSTRAP_STORAGE_PROVISION, SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, SYSTEMS_CONSOLE_WRITE,
    SYSTEMS_DEBUG_BREAK, SYSTEMS_FATAL, SYSTEMS_FRAME_ALLOCATOR_CREATE, SYSTEMS_FRAMES_ALLOCATE,
    SYSTEMS_FRAMES_RELEASE, SYSTEMS_RESUME_DEBUG_BREAK,
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
        Some(&root_entries.bootstrap_storage),
    )?;
    let debug_break = analyze_named_handler(
        &source,
        &functions,
        root,
        "debug-break",
        &root_entries.debug_break,
        CompilerSystemsContextKind::DebugBreak,
        None,
    )?;

    Ok(CompilerSystemsProgram {
        target: target.clone(),
        bootstrap_storage: root_entries.bootstrap_storage,
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
    bootstrap_storage: Option<&CompilerBootstrapStorageDescriptor>,
) -> Result<CompilerSystemsHandler, Diagnostic> {
    let handler = functions.get(handler_name).copied().ok_or_else(|| {
        source_diagnostic(
            source,
            "E-SYSTEMS-HANDLER",
            statement_span(root),
            format!("{entry_name} entry names unknown handler `{handler_name}`"),
        )
    })?;
    analyze_handler(source, handler, context, bootstrap_storage)
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
    bootstrap_storage: CompilerBootstrapStorageDescriptor,
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
    let mut bootstrap_storage = None;
    let mut entries = BTreeMap::new();
    for field in fields {
        let label = artifact_field_label(source, field)?;
        let label_text = source.slice(label);
        if label_text == "bootstrap-storage" {
            let descriptor = parse_bootstrap_storage(source, field)?;
            if bootstrap_storage.replace(descriptor).is_some() {
                return Err(duplicate_artifact_field(source, label, label_text));
            }
        } else if matches!(label_text, "bootstrap" | "debug-break") {
            let handler = parse_entry_field(source, field, label_text)?;
            if entries.insert(label_text.to_owned(), handler).is_some() {
                return Err(duplicate_artifact_field(source, label, label_text));
            }
        } else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-ENTRY-SET",
                label,
                format!("unknown initial systems artifact entry `{label_text}`"),
            ));
        }
    }
    let (Some(bootstrap_storage), Some(bootstrap), Some(debug_break)) = (
        bootstrap_storage,
        entries.remove("bootstrap"),
        entries.remove("debug-break"),
    ) else {
        return Err(invalid_entry_set(source, *span));
    };
    if !entries.is_empty() {
        return Err(invalid_entry_set(source, *span));
    }
    Ok(ArtifactEntries {
        bootstrap_storage,
        bootstrap,
        debug_break,
    })
}

fn artifact_field_label(source: &SourceText, field: &ProductField) -> Result<Span, Diagnostic> {
    field.label.ok_or_else(|| {
        source_diagnostic(
            source,
            "E-SYSTEMS-ENTRY-SET",
            field.value.span(),
            "systems artifact entries require named `bootstrap-storage`, `bootstrap`, and `debug-break` fields",
        )
    })
}

fn duplicate_artifact_field(source: &SourceText, label: Span, label_text: &str) -> Diagnostic {
    source_diagnostic(
        source,
        "E-SYSTEMS-ENTRY-SET",
        label,
        format!("systems artifact entry `{label_text}` is duplicated"),
    )
}

fn parse_entry_field(
    source: &SourceText,
    field: &ProductField,
    label_text: &str,
) -> Result<String, Diagnostic> {
    let expected_constructor = match label_text {
        "bootstrap" => "bootstrap-entry",
        "debug-break" => "synchronous-exception-entry",
        _ => unreachable!("entry caller admits only handler fields"),
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
    Ok(source.slice(*handler).to_owned())
}

fn parse_bootstrap_storage(
    source: &SourceText,
    field: &ProductField,
) -> Result<CompilerBootstrapStorageDescriptor, Diagnostic> {
    let Expression::Application { items, span } = &field.value else {
        return Err(storage_diagnostic(
            source,
            field.value.span(),
            "bootstrap storage must construct `lang systems bounded-bootstrap-storage`",
        ));
    };
    let [
        lang,
        systems,
        constructor,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(storage_diagnostic(
            source,
            *span,
            "bootstrap storage requires one capacity/alignment product",
        ));
    };
    if !identifier_is(source, lang, "lang")
        || !identifier_is(source, systems, "systems")
        || !identifier_is(source, constructor, "bounded-bootstrap-storage")
    {
        return Err(storage_diagnostic(
            source,
            *span,
            "bootstrap storage must construct `lang systems bounded-bootstrap-storage`",
        ));
    }
    let mut values = BTreeMap::new();
    for parameter in fields {
        let Some(label) = parameter.label else {
            return Err(storage_diagnostic(
                source,
                parameter.value.span(),
                "bootstrap storage parameters must be named",
            ));
        };
        let label_text = source.slice(label);
        if !matches!(label_text, "capacity-bytes" | "alignment-bytes") {
            return Err(storage_diagnostic(
                source,
                label,
                format!("unknown bounded bootstrap-storage parameter `{label_text}`"),
            ));
        }
        let value = parse_storage_natural(source, &parameter.value)?;
        if values.insert(label_text.to_owned(), value).is_some() {
            return Err(storage_diagnostic(
                source,
                label,
                format!("bootstrap storage parameter `{label_text}` is duplicated"),
            ));
        }
    }
    let (Some(capacity_bytes), Some(alignment_bytes)) = (
        values.remove("capacity-bytes"),
        values.remove("alignment-bytes"),
    ) else {
        return Err(storage_diagnostic(
            source,
            *span,
            "bounded bootstrap storage requires exactly `capacity-bytes` and `alignment-bytes`",
        ));
    };
    let descriptor = CompilerBootstrapStorageDescriptor {
        capacity_bytes,
        alignment_bytes,
    };
    descriptor
        .validate()
        .map_err(|error| storage_diagnostic(source, *span, error.message))?;
    Ok(descriptor)
}

fn parse_storage_natural(source: &SourceText, value: &Expression) -> Result<u64, Diagnostic> {
    let Expression::Integer(span) = value else {
        return Err(storage_diagnostic(
            source,
            value.span(),
            "bootstrap storage byte counts must be static natural-number literals",
        ));
    };
    source
        .slice(*span)
        .replace('_', "")
        .parse::<u64>()
        .map_err(|_| {
            storage_diagnostic(
                source,
                *span,
                "bootstrap storage byte count is outside the supported natural-number range",
            )
        })
}

fn storage_diagnostic(source: &SourceText, span: Span, message: impl Into<String>) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-STORAGE", span, message)
}

fn invalid_entry_set(source: &SourceText, span: Span) -> Diagnostic {
    source_diagnostic(
        source,
        "E-SYSTEMS-ENTRY-SET",
        span,
        "the initial systems artifact requires exactly `bootstrap-storage`, `bootstrap`, and `debug-break` entries",
    )
}

fn analyze_handler(
    source: &SourceText,
    statement: &Statement,
    context: CompilerSystemsContextKind,
    bootstrap_storage: Option<&CompilerBootstrapStorageDescriptor>,
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
    if context == CompilerSystemsContextKind::Bootstrap {
        let storage = bootstrap_storage.expect("bootstrap handler carries its storage descriptor");
        let checked = analyze_boot_memory_decision(source, body, context_name, storage)?;
        let mut effects = checked
            .operations
            .iter()
            .map(|operation| operation.semantic_identity().to_owned())
            .collect::<Vec<_>>();
        effects.push(checked.disposition.semantic_identity().to_owned());
        effects.sort();
        effects.dedup();
        return Ok(CompilerSystemsHandler {
            name: source.slice(*name).to_owned(),
            context,
            operations: checked.operations,
            disposition: checked.disposition,
            effects,
        });
    }
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
    let disposition = if context == CompilerSystemsContextKind::Bootstrap
        && matches!(
            last,
            Statement::Expression(Expression::DecisionTable { .. })
        ) {
        let storage = bootstrap_storage.expect("bootstrap handler carries its storage descriptor");
        let checked = analyze_bootstrap_region_decision(source, last, context_name, storage)?;
        checked_operations.extend(checked.operations);
        checked.disposition
    } else {
        analyze_disposition(source, last, context, context_name)?
    };
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

include!("systems_boot_memory.rs");
include!("systems_frames.rs");

struct CheckedBootstrapRegionDecision {
    operations: Vec<CompilerSystemsOperation>,
    disposition: CompilerSystemsDisposition,
}

fn analyze_bootstrap_region_decision(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let Statement::Expression(Expression::DecisionTable {
        subject,
        rules,
        span,
    }) = statement
    else {
        unreachable!("bootstrap region caller selects a decision table")
    };
    let request = parse_bootstrap_allocation(source, subject, context_name, storage)?;
    let (ok_binding, ok_action, error_action) = result_actions(source, rules, *span)?;
    analyze_bootstrap_region_actions(
        source,
        request,
        &ok_binding,
        ok_action,
        error_action,
        context_name,
    )
}

fn analyze_bootstrap_region_sequence(
    source: &SourceText,
    statements: &[Statement],
    context_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let [Statement::Expression(subject), ok_rule, error_rule] = statements else {
        return Err(storage_diagnostic(
            source,
            statements.first().map_or(Span::new(0, 0), statement_span),
            "bootstrap allocation requires a subject and exhaustive `Ok`/`Error` actions",
        ));
    };
    let request = parse_bootstrap_allocation(source, subject, context_name, storage)?;
    let (ok_binding, ok_action) = flattened_result_action(source, ok_rule, false)?;
    let (_, error_action) = flattened_result_action(source, error_rule, true)?;
    analyze_bootstrap_region_actions(
        source,
        request,
        &ok_binding,
        ok_action,
        error_action,
        context_name,
    )
}

fn flattened_result_action<'a>(
    source: &SourceText,
    statement: &'a Statement,
    error: bool,
) -> Result<(String, &'a Expression), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "bootstrap allocation requires exhaustive `Ok` and `Error` actions",
        ));
    };
    let [variant, binding, then, action] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "nested bootstrap allocation actions must use explicit blocks",
        ));
    };
    let expected = if error { "Error" } else { "Ok" };
    let Expression::Identifier(binding_span) = binding else {
        return Err(storage_diagnostic(
            source,
            binding.span(),
            "bootstrap allocation result action requires one payload binding",
        ));
    };
    if !identifier_is(source, variant, expected) || !identifier_is(source, then, "then") {
        return Err(storage_diagnostic(
            source,
            *span,
            "bootstrap allocation actions must appear once in `Ok`, `Error` order",
        ));
    }
    if error {
        let Expression::Block { statements, .. } = action else {
            return Err(storage_diagnostic(
                source,
                action.span(),
                "nested bootstrap allocation error action requires a block",
            ));
        };
        let [Statement::Expression(disposition)] = statements.as_slice() else {
            return Err(storage_diagnostic(
                source,
                action.span(),
                "bootstrap allocation error block requires exactly one fatal disposition",
            ));
        };
        return Ok((source.slice(*binding_span).to_owned(), disposition));
    }
    Ok((source.slice(*binding_span).to_owned(), action))
}

fn analyze_bootstrap_region_actions(
    source: &SourceText,
    request: CompilerBootstrapStorageRequest,
    ok_binding: &str,
    ok_action: &Expression,
    error_action: &Expression,
    context_name: &str,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let error_disposition = action_disposition(source, error_action, context_name)?;
    if !matches!(error_disposition, CompilerSystemsDisposition::Fatal { .. }) {
        return Err(storage_diagnostic(
            source,
            error_action.span(),
            "bootstrap allocation failure must consume the context through `fatal`",
        ));
    }
    let Expression::Block {
        statements,
        span: success_span,
    } = ok_action
    else {
        return Err(storage_diagnostic(
            source,
            ok_action.span(),
            "bootstrap allocation success requires a block using and releasing its region",
        ));
    };
    let [store, load, comparison, true_rule, false_rule] = statements.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *success_span,
            "the initial region success action requires byte store, byte load, and an exhaustive equality decision",
        ));
    };
    let (store_offset, value) = parse_region_store(source, store, ok_binding)?;
    let (loaded_name, load_offset) = parse_region_load(source, load, ok_binding)?;
    let comparison = parse_region_comparison(
        source,
        comparison,
        true_rule,
        false_rule,
        context_name,
        ok_binding,
        &loaded_name,
    )?;
    for offset in [store_offset, load_offset] {
        if offset >= request.byte_count {
            return Err(storage_diagnostic(
                source,
                *success_span,
                format!(
                    "bootstrap byte offset {offset} is outside the static region size {}",
                    request.byte_count
                ),
            ));
        }
    }
    Ok(CheckedBootstrapRegionDecision {
        operations: vec![
            CompilerSystemsOperation::BootstrapAllocate { request },
            CompilerSystemsOperation::BootstrapStoreByte {
                offset_bytes: store_offset,
                value,
            },
            CompilerSystemsOperation::BootstrapLoadByteEquals {
                offset_bytes: load_offset,
                expected: comparison.expected,
                failure_message: comparison.failure_message,
            },
            CompilerSystemsOperation::ConsoleWrite {
                text: comparison.success_text,
            },
            CompilerSystemsOperation::BootstrapRelease,
        ],
        disposition: comparison.success_disposition,
    })
}

fn parse_bootstrap_allocation(
    source: &SourceText,
    expression: &Expression,
    context_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CompilerBootstrapStorageRequest, Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(storage_diagnostic(
            source,
            expression.span(),
            "allocation decision subject must be `context bootstrap allocate (...)`",
        ));
    };
    let [
        context,
        bootstrap,
        allocate,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(storage_diagnostic(
            source,
            *span,
            "allocation decision subject must be `context bootstrap allocate (...)`",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, bootstrap, "bootstrap")
        || !identifier_is(source, allocate, "allocate")
    {
        return Err(storage_diagnostic(
            source,
            *span,
            "allocation requires the live bootstrap context",
        ));
    }
    let mut byte_count = None;
    let mut alignment_bytes = None;
    let mut placement = None;
    for field in fields {
        let label = field.label.ok_or_else(|| {
            storage_diagnostic(
                source,
                field.value.span(),
                "bootstrap allocation parameters must be named",
            )
        })?;
        match source.slice(label) {
            "byte-count" => {
                insert_once(
                    source,
                    label,
                    &mut byte_count,
                    parse_storage_natural(source, &field.value)?,
                )?;
            }
            "alignment-bytes" => {
                insert_once(
                    source,
                    label,
                    &mut alignment_bytes,
                    parse_storage_natural(source, &field.value)?,
                )?;
            }
            "placement" => {
                let Expression::Identifier(value) = &field.value else {
                    return Err(storage_diagnostic(
                        source,
                        field.value.span(),
                        "bootstrap allocation placement must be a static systems identity",
                    ));
                };
                let parsed = match source.slice(*value) {
                    "bootstrap-reclaimable" => {
                        CompilerBootstrapStoragePlacement::BootstrapReclaimable
                    }
                    "page-aligned-table" => CompilerBootstrapStoragePlacement::PageAlignedTable,
                    other => {
                        return Err(storage_diagnostic(
                            source,
                            *value,
                            format!("unknown bootstrap allocation placement `{other}`"),
                        ));
                    }
                };
                insert_once(source, label, &mut placement, parsed)?;
            }
            other => {
                return Err(storage_diagnostic(
                    source,
                    label,
                    format!("unknown bootstrap allocation parameter `{other}`"),
                ));
            }
        }
    }
    let request = CompilerBootstrapStorageRequest {
        byte_count: byte_count.ok_or_else(|| missing_allocation_fields(source, *span))?,
        alignment_bytes: alignment_bytes.ok_or_else(|| missing_allocation_fields(source, *span))?,
        placement: placement.ok_or_else(|| missing_allocation_fields(source, *span))?,
    };
    validate_static_allocation(source, *span, storage, request)?;
    Ok(request)
}

fn validate_static_allocation(
    source: &SourceText,
    span: Span,
    storage: &CompilerBootstrapStorageDescriptor,
    request: CompilerBootstrapStorageRequest,
) -> Result<(), Diagnostic> {
    let mut model = CompilerBootstrapStorageState::new(storage.clone(), "source-check")
        .map_err(|error| storage_diagnostic(source, span, error.message))?;
    model.allocate(request).map_err(|code| {
        storage_diagnostic(
            source,
            span,
            format!("static bootstrap allocation cannot succeed: {code:?}"),
        )
    })?;
    Ok(())
}

fn insert_once<T>(
    source: &SourceText,
    label: Span,
    destination: &mut Option<T>,
    value: T,
) -> Result<(), Diagnostic> {
    if destination.replace(value).is_some() {
        return Err(storage_diagnostic(
            source,
            label,
            format!(
                "bootstrap allocation parameter `{}` is duplicated",
                source.slice(label)
            ),
        ));
    }
    Ok(())
}

fn missing_allocation_fields(source: &SourceText, span: Span) -> Diagnostic {
    storage_diagnostic(
        source,
        span,
        "bootstrap allocation requires exactly `byte-count`, `alignment-bytes`, and `placement`",
    )
}

fn result_actions<'a>(
    source: &SourceText,
    rules: &'a [DecisionRule],
    span: Span,
) -> Result<(String, &'a Expression, &'a Expression), Diagnostic> {
    let mut ok = None;
    let mut error = None;
    for rule in rules {
        match rule.matcher {
            DecisionMatcher::Result {
                error: false,
                binding,
                ..
            } if ok.is_none() => ok = Some((source.slice(binding).to_owned(), &rule.action)),
            DecisionMatcher::Result { error: true, .. } if error.is_none() => {
                error = Some(&rule.action);
            }
            _ => {
                return Err(storage_diagnostic(
                    source,
                    rule.span,
                    "bootstrap allocation requires exactly one `Ok region` and one `Error problem` action",
                ));
            }
        }
    }
    let (Some((binding, ok)), Some(error)) = (ok, error) else {
        return Err(storage_diagnostic(
            source,
            span,
            "bootstrap allocation requires exhaustive `Ok` and `Error` actions",
        ));
    };
    Ok((binding, ok, error))
}

fn parse_region_store(
    source: &SourceText,
    statement: &Statement,
    region_name: &str,
) -> Result<(u64, u8), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "region success action must begin with `region byte store (...)`",
        ));
    };
    let [region, byte, store, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "region success action must begin with `region byte store (...)`",
        ));
    };
    if !identifier_is(source, region, region_name)
        || !identifier_is(source, byte, "byte")
        || !identifier_is(source, store, "store")
    {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte store requires the affine region bound by the `Ok` action",
        ));
    }
    let offset = named_static_natural(source, fields, "offset-bytes")?;
    let value = named_static_natural(source, fields, "value")?;
    if fields.len() != 2 {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte store requires exactly `offset-bytes` and `value`",
        ));
    }
    let value = u8::try_from(value).map_err(|_| {
        storage_diagnostic(
            source,
            *span,
            "byte store value must be a `Nat` from 0 through 255",
        )
    })?;
    Ok((offset, value))
}

fn parse_region_load(
    source: &SourceText,
    statement: &Statement,
    region_name: &str,
) -> Result<(String, u64), Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "byte load must bind its `Nat` result",
        ));
    };
    let [region, byte, load, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte load must be `observed : Nat is region byte load (...)`",
        ));
    };
    if source.slice(*classifier) != "Nat"
        || !identifier_is(source, region, region_name)
        || !identifier_is(source, byte, "byte")
        || !identifier_is(source, load, "load")
        || fields.len() != 1
    {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte load requires the affine region and one static `offset-bytes` field",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        named_static_natural(source, fields, "offset-bytes")?,
    ))
}

fn named_static_natural(
    source: &SourceText,
    fields: &[ProductField],
    name: &str,
) -> Result<u64, Diagnostic> {
    let mut found = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(storage_diagnostic(
                source,
                field.value.span(),
                "byte access parameters must be named",
            ));
        };
        if source.slice(label) == name {
            if found
                .replace(parse_storage_natural(source, &field.value)?)
                .is_some()
            {
                return Err(storage_diagnostic(
                    source,
                    label,
                    format!("byte access parameter `{name}` is duplicated"),
                ));
            }
        } else if !matches!(source.slice(label), "offset-bytes" | "value") {
            return Err(storage_diagnostic(
                source,
                label,
                format!("unknown byte access parameter `{}`", source.slice(label)),
            ));
        }
    }
    found.ok_or_else(|| {
        storage_diagnostic(
            source,
            fields
                .first()
                .map_or(Span::new(0, 0), |field| field.value.span()),
            format!("byte access requires `{name}`"),
        )
    })
}

struct CheckedRegionComparison {
    expected: u8,
    success_text: String,
    failure_message: String,
    success_disposition: CompilerSystemsDisposition,
}

fn parse_region_comparison(
    source: &SourceText,
    statement: &Statement,
    true_rule: &Statement,
    false_rule: &Statement,
    context_name: &str,
    region_name: &str,
    loaded_name: &str,
) -> Result<CheckedRegionComparison, Diagnostic> {
    let Statement::Expression(Expression::Application { items, .. }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "loaded byte requires an exhaustive equality decision",
        ));
    };
    let [
        loaded,
        Expression::Callable {
            kind: CallableKind::Equal,
            ..
        },
        expected,
    ] = items.as_slice()
    else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "loaded byte decision subject must use equality with a static byte",
        ));
    };
    if !identifier_is(source, loaded, loaded_name) {
        return Err(storage_diagnostic(
            source,
            loaded.span(),
            "byte comparison must use the immediately preceding load binding",
        ));
    }
    let expected = u8::try_from(parse_storage_natural(source, expected)?).map_err(|_| {
        storage_diagnostic(
            source,
            expected.span(),
            "byte comparison value must be from 0 through 255",
        )
    })?;
    let true_action = parse_boolean_rule(source, true_rule, true)?;
    let false_action = parse_boolean_rule(source, false_rule, false)?;
    let true_statements = action_block(source, true_action, "successful byte comparison")?;
    let [console, release, disposition] = true_statements else {
        return Err(storage_diagnostic(
            source,
            true_action.span(),
            "successful byte comparison requires console marker, region release, and final disposition",
        ));
    };
    let CompilerSystemsOperation::ConsoleWrite { text } = analyze_operation(
        source,
        console,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?
    else {
        unreachable!("only the console operation passes the success check")
    };
    parse_region_release(source, release, context_name, region_name)?;
    let success_disposition = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;

    let false_statements = action_block(source, false_action, "failed byte comparison")?;
    let [release, disposition] = false_statements else {
        return Err(storage_diagnostic(
            source,
            false_action.span(),
            "failed byte comparison requires region release and fatal disposition",
        ));
    };
    parse_region_release(source, release, context_name, region_name)?;
    let failure = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;
    let CompilerSystemsDisposition::Fatal {
        message: failure_message,
    } = failure
    else {
        return Err(storage_diagnostic(
            source,
            statement_span(disposition),
            "failed byte comparison must enter the fatal disposition",
        ));
    };
    Ok(CheckedRegionComparison {
        expected,
        success_text: text,
        failure_message,
        success_disposition,
    })
}

fn parse_boolean_rule<'a>(
    source: &SourceText,
    statement: &'a Statement,
    expected: bool,
) -> Result<&'a Expression, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "byte equality requires explicit `true` and `false` actions",
        ));
    };
    let [Expression::Boolean(value), then, action] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte equality requires explicit `true` and `false` actions",
        ));
    };
    if (source.slice(*value) == "true") != expected || !identifier_is(source, then, "then") {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte equality actions must appear once in `true`, `false` order",
        ));
    }
    Ok(action)
}

fn action_block<'a>(
    source: &SourceText,
    action: &'a Expression,
    description: &str,
) -> Result<&'a [Statement], Diagnostic> {
    let Expression::Block { statements, .. } = action else {
        return Err(storage_diagnostic(
            source,
            action.span(),
            format!("{description} requires a block"),
        ));
    };
    Ok(statements)
}

fn action_disposition(
    source: &SourceText,
    action: &Expression,
    context_name: &str,
) -> Result<CompilerSystemsDisposition, Diagnostic> {
    if let Expression::Block { statements, .. } = action {
        let [statement] = statements.as_slice() else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DISPOSITION",
                action.span(),
                "a systems failure block must contain exactly one context-consuming disposition",
            ));
        };
        return analyze_disposition(
            source,
            statement,
            CompilerSystemsContextKind::Bootstrap,
            context_name,
        );
    }
    let statement = Statement::Expression(action.clone());
    analyze_disposition(
        source,
        &statement,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )
}

fn parse_region_release(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    region_name: &str,
) -> Result<(), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "bootstrap region must be released exactly once",
        ));
    };
    let [context, bootstrap, release, region] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "release must be `context bootstrap release region`",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, bootstrap, "bootstrap")
        || !identifier_is(source, release, "release")
        || !identifier_is(source, region, region_name)
    {
        return Err(storage_diagnostic(
            source,
            *span,
            "release requires the live bootstrap context and its affine region",
        ));
    }
    Ok(())
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
    const STORAGE_DECLARATION: &str = "  bootstrap-storage is lang systems bounded-bootstrap-storage (\n    capacity-bytes is 65536,\n    alignment-bytes is 4096\n  ),\n";

    fn memory_source() -> String {
        SOURCE.to_owned()
    }

    #[test]
    fn checks_the_initial_artifact_and_models_entry_transitions() {
        // TOPAL-SYSTEMS-VOCABULARY-001, TOPAL-SYSTEMS-ENTRY-001,
        // TOPAL-SYSTEMS-DISPOSITION-001, TOPAL-SYSTEMS-OBSERVATION-001,
        // TOPAL-SYSTEMS-FRAMES-001.
        let program = analyze_systems_for_compiler(
            SOURCE,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        assert_eq!(program.bootstrap.kind, CompilerSystemsEntryKind::Bootstrap);
        assert_eq!(program.bootstrap_storage.capacity_bytes, 65_536);
        assert_eq!(program.bootstrap_storage.alignment_bytes, 4096);
        assert_eq!(
            program.debug_break.kind,
            CompilerSystemsEntryKind::SynchronousExceptionDebugBreak
        );
        assert_eq!(
            program.bootstrap.handler.effects,
            [
                SYSTEMS_BOOT_MEMORY_DESCRIBE,
                SYSTEMS_CONSOLE_WRITE,
                SYSTEMS_FATAL,
                SYSTEMS_FRAMES_ALLOCATE,
                SYSTEMS_FRAMES_RELEASE,
                SYSTEMS_DEBUG_BREAK,
                SYSTEMS_FRAME_ALLOCATOR_CREATE,
                SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
                SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
                SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
                SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            ]
        );
        assert_eq!(
            model_systems_transitions(&program).unwrap(),
            vec![
                CompilerSystemsTransition::ProvisionBootstrapStorage {
                    capacity_bytes: 65_536,
                    alignment_bytes: 4096,
                },
                CompilerSystemsTransition::EnterBootstrap,
                CompilerSystemsTransition::DescribeBootMemory,
                CompilerSystemsTransition::CreateFrameAllocator,
                CompilerSystemsTransition::AllocatePhysicalFrames {
                    request: CompilerPhysicalFrameRequest {
                        frame_count: 1,
                        alignment_frames: 1,
                    },
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_FRAME_ALLOCATED".into(),
                },
                CompilerSystemsTransition::ReleasePhysicalFrames,
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_MEMORY_DESCRIBED".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_BOOT".into(),
                },
                CompilerSystemsTransition::ObserveDebugBreak,
                CompilerSystemsTransition::EnterDebugBreak,
                CompilerSystemsTransition::ResumeDebugBreak,
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_FAULT_RESUMED".into(),
                },
                CompilerSystemsTransition::AllocateBootstrapRegion {
                    request: CompilerBootstrapStorageRequest {
                        byte_count: 64,
                        alignment_bytes: 8,
                        placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
                    },
                },
                CompilerSystemsTransition::StoreBootstrapByte {
                    offset_bytes: 0,
                    value: 90,
                },
                CompilerSystemsTransition::LoadBootstrapByte {
                    offset_bytes: 0,
                    value: 90,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_MEMORY_OK".into(),
                },
                CompilerSystemsTransition::ReleaseBootstrapRegion,
                CompilerSystemsTransition::Fatal {
                    message: "toolchain gate complete".into(),
                },
            ]
        );
    }

    #[test]
    fn checks_affine_bootstrap_region_access_and_models_real_byte_contents() {
        // TOPAL-SYSTEMS-STORAGE-001, TOPAL-SYSTEMS-QUALIFY-001,
        // TOPAL-SYSTEMS-FRAMES-001.
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        assert_eq!(program.bootstrap.handler.operations.len(), 14);
        assert_eq!(
            program.bootstrap.handler.effects,
            [
                SYSTEMS_BOOT_MEMORY_DESCRIBE,
                SYSTEMS_CONSOLE_WRITE,
                SYSTEMS_FATAL,
                SYSTEMS_FRAMES_ALLOCATE,
                SYSTEMS_FRAMES_RELEASE,
                SYSTEMS_DEBUG_BREAK,
                SYSTEMS_FRAME_ALLOCATOR_CREATE,
                SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
                SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
                SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
                SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            ]
        );
        let transitions = model_systems_transitions(&program).unwrap();
        assert!(
            transitions.contains(&CompilerSystemsTransition::StoreBootstrapByte {
                offset_bytes: 0,
                value: 90,
            })
        );
        assert!(
            transitions.contains(&CompilerSystemsTransition::LoadBootstrapByte {
                offset_bytes: 0,
                value: 90,
            })
        );
        assert!(transitions.contains(&CompilerSystemsTransition::ReleaseBootstrapRegion));
        assert!(
            transitions.contains(&CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_MEMORY_OK".into(),
            })
        );
    }

    #[test]
    fn rejects_unbounded_or_non_affine_bootstrap_region_forms() {
        // TOPAL-SYSTEMS-AUTHORITY-001, TOPAL-SYSTEMS-STORAGE-001.
        for (source, code, expected) in [
            (
                memory_source().replace("offset-bytes is 0, value", "offset-bytes is 64, value"),
                "E-SYSTEMS-STORAGE",
                "outside the static region size",
            ),
            (
                memory_source().replace("value is 90)", "value is 256)"),
                "E-SYSTEMS-STORAGE",
                "from 0 through 255",
            ),
            (
                memory_source().replace(
                    "memory bootstrap release region\n                      memory fatal \"toolchain gate complete\"",
                    "memory console write \"not released\"\n                      memory fatal \"toolchain gate complete\"",
                ),
                "E-SYSTEMS-STORAGE",
                "release requires",
            ),
            (
                memory_source().replace(
                    "memory fatal \"toolchain gate allocation failed\"",
                    "region byte load (offset-bytes is 0)",
                ),
                "E-SYSTEMS-DISPOSITION",
                "final operation is not a disposition",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, code);
            assert!(error.message.contains(expected), "{}", error.message);
        }
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
            "memory console write \"TOPAL_KERNEL_BOOT\"",
            "saved is memory",
        );
        assert_eq!(
            analyze_systems_for_compiler(
                &escaped,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-FRAMES"
        );

        let unknown = SOURCE.replace("memory debug break", "memory machine instruction");
        assert_eq!(
            analyze_systems_for_compiler(
                &unknown,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-OPERATION"
        );

        let missing = SOURCE.replace(
            "  context resume",
            "  context console write \"missing disposition\"",
        );
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
    fn requires_affine_exhaustive_boot_memory_refinement() {
        // TOPAL-SYSTEMS-BOOT-MEMORY-001, TOPAL-SYSTEMS-AUTHORITY-001.
        for (source, code) in [
            (
                SOURCE.replace(
                    "context boot describe memory",
                    "context console write \"skipped refinement\"",
                ),
                "E-SYSTEMS-BOOT-MEMORY",
            ),
            (
                SOURCE.replace(
                    "memory console write \"TOPAL_KERNEL_MEMORY_DESCRIBED\"",
                    "context console write \"TOPAL_KERNEL_MEMORY_DESCRIBED\"",
                ),
                "E-SYSTEMS-OPERATION",
            ),
            (
                SOURCE.replace(
                    "Error failure then failure fatal \"boot memory description failed\"",
                    "Error failure then failure resume",
                ),
                "E-SYSTEMS-DISPOSITION",
            ),
        ] {
            assert_eq!(
                analyze_systems_for_compiler(
                    &source,
                    &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
                )
                .unwrap_err()
                .code,
                code
            );
        }
    }

    #[test]
    fn requires_affine_physical_frame_allocation_and_release() {
        // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-AUTHORITY-001.
        for (source, code, expected) in [
            (
                SOURCE.replace(
                    "described memory create frame allocator",
                    "context memory create frame allocator",
                ),
                "E-SYSTEMS-FRAMES",
                "live memory-described context",
            ),
            (
                SOURCE.replace("frame-count is 1", "frame-count is 2"),
                "E-SYSTEMS-FRAMES",
                "exactly one frame",
            ),
            (
                SOURCE.replace("alignment-frames is 1", "alignment-frames is 2"),
                "E-SYSTEMS-FRAMES",
                "aligned to one frame",
            ),
            (
                SOURCE.replace(
                    "memory frames release frames",
                    "memory frames release other",
                ),
                "E-SYSTEMS-FRAMES",
                "affine extent",
            ),
            (
                SOURCE.replace(
                    "failure fatal \"frame allocator creation failed\"",
                    "failure resume",
                ),
                "E-SYSTEMS-DISPOSITION",
                "not a disposition",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, code);
            assert!(error.message.contains(expected), "{}", error.message);
        }
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
    fn rejects_missing_malformed_and_open_bootstrap_storage() {
        // TOPAL-SYSTEMS-VOCABULARY-001, TOPAL-SYSTEMS-STORAGE-001.
        let missing = SOURCE.replace(STORAGE_DECLARATION, "");
        assert_eq!(
            analyze_systems_for_compiler(
                &missing,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-ENTRY-SET"
        );

        let duplicate = SOURCE.replace(
            STORAGE_DECLARATION,
            &format!("{STORAGE_DECLARATION}{STORAGE_DECLARATION}"),
        );
        assert_eq!(
            analyze_systems_for_compiler(
                &duplicate,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-ENTRY-SET"
        );

        for malformed in [
            SOURCE.replace("    capacity-bytes is 65536,\n", ""),
            SOURCE.replace("capacity-bytes is 65536", "capacity-bytes is 0"),
            SOURCE.replace("alignment-bytes is 4096", "alignment-bytes is 3"),
            SOURCE.replace("alignment-bytes is 4096", "alignment-bytes is 131072"),
            SOURCE.replace("capacity-bytes is 65536", "capacity-bytes is capacity"),
            SOURCE.replace(
                "alignment-bytes is 4096",
                "alignment-bytes is 4096,\n    physical-address is 4096",
            ),
        ] {
            assert_eq!(
                analyze_systems_for_compiler(
                    &malformed,
                    &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
                )
                .unwrap_err()
                .code,
                "E-SYSTEMS-STORAGE"
            );
        }
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
