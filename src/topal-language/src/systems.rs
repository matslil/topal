//! Checked source model for the initial freestanding systems profile.
//!
//! This module deliberately stops before target lowering. It establishes the
//! sealed source vocabulary and abstract transitions which a later qualified
//! provider may consume without reinterpreting syntax.

use std::collections::BTreeMap;

use topal_source::{Diagnostic, SourceText, Span};
use topal_syntax::{
    CallableKind, DecisionMatcher, DecisionRule, Expression, FunctionClauses, FunctionParameter,
    ProductField, Statement, lex, parse,
};

pub use topal_semantics::{
    AtomicCompareExchangeResult as CompilerAtomicCompareExchangeResult,
    AtomicDomain as CompilerAtomicDomain, AtomicOrder as CompilerAtomicOrder,
    AtomicWordRequest as CompilerAtomicWordRequest, BootstrapRegion as CompilerBootstrapRegion,
    BootstrapStorageDescriptor as CompilerBootstrapStorageDescriptor,
    BootstrapStorageErrorCode as CompilerBootstrapStorageErrorCode,
    BootstrapStoragePlacement as CompilerBootstrapStoragePlacement,
    BootstrapStorageRequest as CompilerBootstrapStorageRequest,
    BootstrapStorageState as CompilerBootstrapStorageState,
    BootstrapStorageTransition as CompilerBootstrapStorageTransition,
    CriticalDomain as CompilerCriticalDomain,
    DmaBufferIntegerLoadRequest as CompilerDmaBufferIntegerLoadRequest,
    DmaBufferIntegerStoreRequest as CompilerDmaBufferIntegerStoreRequest,
    DmaBufferPreparationRequest as CompilerDmaBufferPreparationRequest,
    DmaBufferReferenceRequest as CompilerDmaBufferReferenceRequest,
    DmaBufferRequest as CompilerDmaBufferRequest, DmaBufferRole as CompilerDmaBufferRole,
    DmaDirection as CompilerDmaDirection, DmaVisibilityPolicy as CompilerDmaVisibilityPolicy,
    EntropyGeneratorExtractRequest as CompilerEntropyGeneratorExtractRequest,
    EntropyGeneratorRequest as CompilerEntropyGeneratorRequest,
    EntropyOutputDeliveryRequest as CompilerEntropyOutputDeliveryRequest,
    EntropyOutputReservationRequest as CompilerEntropyOutputReservationRequest,
    EntropyPoolCreditRequest as CompilerEntropyPoolCreditRequest,
    EntropyPoolMixRequest as CompilerEntropyPoolMixRequest,
    EntropyPoolRequest as CompilerEntropyPoolRequest,
    EntropySampleCaptureRequest as CompilerEntropySampleCaptureRequest,
    EntropySampleQualityRequest as CompilerEntropySampleQualityRequest,
    EntropyUserRandomServiceRequest as CompilerEntropyUserRandomServiceRequest,
    EntropyUserRandomSmallRequest as CompilerEntropyUserRandomSmallRequest, INITIAL_SYSTEMS_BOARD,
    INITIAL_SYSTEMS_PROFILE, INITIAL_SYSTEMS_TARGET,
    KernelExecutionPolicy as CompilerKernelExecutionPolicy,
    KernelMappingRequest as CompilerKernelMappingRequest,
    KernelMappingRights as CompilerKernelMappingRights,
    KernelMemoryKind as CompilerKernelMemoryKind,
    PciConfigurationDwordRequest as CompilerPciConfigurationDwordRequest,
    PciConfigurationWordUpdateRequest as CompilerPciConfigurationWordUpdateRequest,
    PciRegionDmaReferenceRequest as CompilerPciRegionDmaReferenceRequest,
    PciRegionRegisterDirection as CompilerPciRegionRegisterDirection,
    PciRegionRegisterRequest as CompilerPciRegionRegisterRequest,
    PhysicalFrameRequest as CompilerPhysicalFrameRequest, SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
    SYSTEMS_ATOMIC_END, SYSTEMS_ATOMIC_LOAD, SYSTEMS_ATOMIC_WORD_CREATE,
    SYSTEMS_BOOT_MEMORY_DESCRIBE, SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
    SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE, SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
    SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE, SYSTEMS_BOOTSTRAP_STORAGE_EXHAUSTED,
    SYSTEMS_BOOTSTRAP_STORAGE_INVALID_REQUEST, SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
    SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, SYSTEMS_CONSOLE_WRITE, SYSTEMS_CRITICAL_ENTER,
    SYSTEMS_CRITICAL_RESTORE, SYSTEMS_DEADLINE_AFTER, SYSTEMS_DEADLINE_ARM,
    SYSTEMS_DEADLINE_COMPLETE, SYSTEMS_DEADLINE_WAIT, SYSTEMS_DEBUG_BREAK,
    SYSTEMS_DMA_BUFFER_BIND_REFERENCE, SYSTEMS_DMA_BUFFER_COMPLETE_CPU, SYSTEMS_DMA_BUFFER_CREATE,
    SYSTEMS_DMA_BUFFER_LOAD_INTEGER, SYSTEMS_DMA_BUFFER_PREPARE_DEVICE,
    SYSTEMS_DMA_BUFFER_STORE_INTEGER, SYSTEMS_ENTROPY_GENERATOR_EXTRACT,
    SYSTEMS_ENTROPY_GENERATOR_INITIALIZE, SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM,
    SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM, SYSTEMS_ENTROPY_POOL_CREATE,
    SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION, SYSTEMS_ENTROPY_POOL_MIX_SAMPLE,
    SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY, SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA,
    SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE, SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS,
    SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS, SYSTEMS_FATAL,
    SYSTEMS_FRAME_ALLOCATOR_CREATE, SYSTEMS_FRAMES_ALLOCATE, SYSTEMS_FRAMES_RELEASE,
    SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE, SYSTEMS_KERNEL_COMPLETION_CONSUME,
    SYSTEMS_KERNEL_COMPLETION_CREATE, SYSTEMS_KERNEL_COMPLETION_SIGNAL,
    SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION, SYSTEMS_KERNEL_CONTEXT_CREATE,
    SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT, SYSTEMS_KERNEL_CONTEXT_RECLAIM,
    SYSTEMS_KERNEL_CONTEXT_RETIRE, SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
    SYSTEMS_KERNEL_CONTEXT_TRANSFER, SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE,
    SYSTEMS_KERNEL_MAP, SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
    SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME, SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE,
    SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE, SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE,
    SYSTEMS_KERNEL_UNMAP, SYSTEMS_LOCAL_NOTIFICATION_COMPLETE, SYSTEMS_LOCAL_NOTIFICATION_SEND,
    SYSTEMS_LOCAL_NOTIFICATION_WAIT, SYSTEMS_MONOTONIC_CLOCK_NOW,
    SYSTEMS_PCI_CONFIGURATION_READ_DWORD, SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD,
    SYSTEMS_PCI_REGION_READ, SYSTEMS_PCI_REGION_READ_EVENTUALLY, SYSTEMS_PCI_REGION_WRITE,
    SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE, SYSTEMS_RESUME_DEADLINE, SYSTEMS_RESUME_DEBUG_BREAK,
    SYSTEMS_RESUME_LOCAL_NOTIFICATION, SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PROGRAM_HEADERS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_ACQUISITION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_HANDOFF,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DELIVER_ENTROPY_INITIAL_RANDOM,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_BUS_MASTER,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DEVICE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_BUFFERS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_PREPARATION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_NEGOTIATION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_POOL_LIFECYCLE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_REQUEST_COMPLETION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_CAPTURE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_QUALITY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SPLIT_RING,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_TRANSPORT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_RETAINED_DISJOINT_WORK,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PROGRAM_HEADERS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_PREPARED_PROCESS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RETAIN_PREPARED_PROCESS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_ACQUISITION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_HANDOFF,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_INITIAL_RANDOM,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING, SYSTEMS_STATIC_IMAGE_PROCESS_ENTER,
    SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY, SYSTEMS_TRANSLATION_ACTIVATE,
    SYSTEMS_TRANSLATION_BEGIN, SYSTEMS_TRANSLATION_COMMIT, SYSTEMS_TRANSLATION_EDIT_BEGIN,
    SYSTEMS_TRANSLATION_EDIT_COMMIT, SYSTEMS_TRANSLATION_EDIT_MAP, SYSTEMS_TRANSLATION_EDIT_UNMAP,
    SYSTEMS_USER_ADDRESS_SPACE_MAP, SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
    SYSTEMS_USER_ADDRESS_SPACE_RECLAIM, SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
    SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS, SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
    SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING, SYSTEMS_USER_CONTEXT_COMPLETE,
    SYSTEMS_USER_CONTEXT_ENTER, SYSTEMS_USER_CONTEXT_PREPARE, SYSTEMS_USER_COPY_FROM,
    SYSTEMS_USER_COPY_TO, SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE,
    SYSTEMS_USER_MAPPING_LOAD_BYTE, SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
    SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA, SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
    SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA, SYSTEMS_USER_MAPPING_STORE_BYTE,
    SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE, SYSTEMS_USER_SYSCALL_DISPATCH,
    SYSTEMS_USER_TRANSFER_BEGIN, SYSTEMS_USER_TRANSFER_END,
    SystemsContextKind as CompilerSystemsContextKind,
    SystemsDisposition as CompilerSystemsDisposition, SystemsEntry as CompilerSystemsEntry,
    SystemsEntryKind as CompilerSystemsEntryKind, SystemsHandler as CompilerSystemsHandler,
    SystemsOperation as CompilerSystemsOperation, SystemsProgram as CompilerSystemsProgram,
    SystemsTargetSelection as CompilerSystemsTargetSelection,
    SystemsTransition as CompilerSystemsTransition,
    TranslationEditKind as CompilerTranslationEditKind,
    TranslationMappingRequest as CompilerTranslationMappingRequest,
    TranslationPagePolicy as CompilerTranslationPagePolicy,
    TranslationPlacement as CompilerTranslationPlacement,
    TranslationTemplate as CompilerTranslationTemplate,
    TranslationUpdateRequest as CompilerTranslationUpdateRequest, model_systems_transitions,
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
    let (source, statements) = parse_systems_source(text)?;
    let Some((selection, remaining)) = statements.split_first() else {
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
    if functions.len() != 8 {
        return Err(source_diagnostic(
            &source,
            "E-SYSTEMS-ENTRY-SET",
            statement_span(root),
            "the systems artifact requires exactly its bootstrap, debug-break, local-notification, deadline-notification, preemptible, completion-blocking, terminal, and user-syscall handlers",
        ));
    }
    if root_entries.bootstrap == root_entries.debug_break
        || root_entries.bootstrap == root_entries.local_notification
        || root_entries.bootstrap == root_entries.deadline_notification
        || root_entries.debug_break == root_entries.local_notification
        || root_entries.debug_break == root_entries.deadline_notification
        || root_entries.debug_break == root_entries.kernel_thread
        || root_entries.local_notification == root_entries.deadline_notification
        || root_entries.local_notification == root_entries.kernel_thread
        || root_entries.deadline_notification == root_entries.kernel_thread
        || root_entries.bootstrap == root_entries.completion_thread
        || root_entries.debug_break == root_entries.completion_thread
        || root_entries.local_notification == root_entries.completion_thread
        || root_entries.deadline_notification == root_entries.completion_thread
        || root_entries.kernel_thread == root_entries.completion_thread
        || root_entries.bootstrap == root_entries.kernel_thread
        || root_entries.bootstrap == root_entries.terminal_thread
        || root_entries.debug_break == root_entries.terminal_thread
        || root_entries.local_notification == root_entries.terminal_thread
        || root_entries.deadline_notification == root_entries.terminal_thread
        || root_entries.completion_thread == root_entries.terminal_thread
        || root_entries.kernel_thread == root_entries.terminal_thread
        || root_entries.bootstrap == root_entries.user_syscall
        || root_entries.debug_break == root_entries.user_syscall
        || root_entries.local_notification == root_entries.user_syscall
        || root_entries.deadline_notification == root_entries.user_syscall
        || root_entries.kernel_thread == root_entries.user_syscall
        || root_entries.completion_thread == root_entries.user_syscall
        || root_entries.terminal_thread == root_entries.user_syscall
    {
        return Err(source_diagnostic(
            &source,
            "E-SYSTEMS-ENTRY-SET",
            statement_span(root),
            "all eight initial systems entries require distinct handlers",
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
    let local_notification = analyze_named_handler(
        &source,
        &functions,
        root,
        "local-notification",
        &root_entries.local_notification,
        CompilerSystemsContextKind::LocalNotificationInterrupt,
        None,
    )?;
    let deadline_notification = analyze_named_handler(
        &source,
        &functions,
        root,
        "deadline-notification",
        &root_entries.deadline_notification,
        CompilerSystemsContextKind::DeadlineInterrupt,
        None,
    )?;
    let kernel_thread = analyze_named_handler(
        &source,
        &functions,
        root,
        "kernel-thread",
        &root_entries.kernel_thread,
        CompilerSystemsContextKind::KernelThread,
        None,
    )?;
    let terminal_thread = analyze_named_handler(
        &source,
        &functions,
        root,
        "terminal-thread",
        &root_entries.terminal_thread,
        CompilerSystemsContextKind::KernelThread,
        None,
    )?;
    let completion_thread = analyze_named_handler(
        &source,
        &functions,
        root,
        "completion-thread",
        &root_entries.completion_thread,
        CompilerSystemsContextKind::KernelThread,
        None,
    )?;
    let user_syscall = analyze_named_handler(
        &source,
        &functions,
        root,
        "user-syscall",
        &root_entries.user_syscall,
        CompilerSystemsContextKind::UserSyscall,
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
        local_notification: CompilerSystemsEntry {
            kind: CompilerSystemsEntryKind::ExternalInterruptLocalNotification,
            handler: local_notification,
        },
        deadline_notification: CompilerSystemsEntry {
            kind: CompilerSystemsEntryKind::ExternalInterruptDeadline,
            handler: deadline_notification,
        },
        kernel_thread: CompilerSystemsEntry {
            kind: CompilerSystemsEntryKind::ResumedKernelThread,
            handler: kernel_thread,
        },
        completion_thread: CompilerSystemsEntry {
            kind: CompilerSystemsEntryKind::ResumedKernelThread,
            handler: completion_thread,
        },
        terminal_thread: CompilerSystemsEntry {
            kind: CompilerSystemsEntryKind::ResumedKernelThread,
            handler: terminal_thread,
        },
        user_syscall: CompilerSystemsEntry {
            kind: CompilerSystemsEntryKind::UserSyscall,
            handler: user_syscall,
        },
    })
}

fn parse_systems_source(text: &str) -> Result<(SourceText, Vec<Statement>), Diagnostic> {
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
    Ok((source, parsed.statements))
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
    local_notification: String,
    deadline_notification: String,
    kernel_thread: String,
    completion_thread: String,
    terminal_thread: String,
    user_syscall: String,
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
        } else if matches!(
            label_text,
            "bootstrap"
                | "debug-break"
                | "local-notification"
                | "deadline-notification"
                | "kernel-thread"
                | "completion-thread"
                | "terminal-thread"
                | "user-syscall"
        ) {
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
    let (
        Some(bootstrap_storage),
        Some(bootstrap),
        Some(debug_break),
        Some(local_notification),
        Some(deadline_notification),
        Some(kernel_thread),
        Some(completion_thread),
        Some(terminal_thread),
        Some(user_syscall),
    ) = (
        bootstrap_storage,
        entries.remove("bootstrap"),
        entries.remove("debug-break"),
        entries.remove("local-notification"),
        entries.remove("deadline-notification"),
        entries.remove("kernel-thread"),
        entries.remove("completion-thread"),
        entries.remove("terminal-thread"),
        entries.remove("user-syscall"),
    )
    else {
        return Err(invalid_entry_set(source, *span));
    };
    if !entries.is_empty() {
        return Err(invalid_entry_set(source, *span));
    }
    Ok(ArtifactEntries {
        bootstrap_storage,
        bootstrap,
        debug_break,
        local_notification,
        deadline_notification,
        kernel_thread,
        completion_thread,
        terminal_thread,
        user_syscall,
    })
}

fn artifact_field_label(source: &SourceText, field: &ProductField) -> Result<Span, Diagnostic> {
    field.label.ok_or_else(|| {
        source_diagnostic(
            source,
            "E-SYSTEMS-ENTRY-SET",
            field.value.span(),
            "systems artifact entries require named `bootstrap-storage`, `bootstrap`, `debug-break`, `local-notification`, `deadline-notification`, `kernel-thread`, and `terminal-thread` fields",
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
        "local-notification" => "external-interrupt-entry",
        "deadline-notification" => "external-interrupt-entry",
        "kernel-thread" | "completion-thread" | "terminal-thread" => "resumed-thread-entry",
        "user-syscall" => "user-syscall-entry",
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

fn atomic_diagnostic(source: &SourceText, span: Span, message: impl Into<String>) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-ATOMIC", span, message)
}

fn local_interrupt_diagnostic(
    source: &SourceText,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-LOCAL-INTERRUPT", span, message)
}

fn monotonic_clock_diagnostic(
    source: &SourceText,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-MONOTONIC-CLOCK", span, message)
}

fn deadline_diagnostic(source: &SourceText, span: Span, message: impl Into<String>) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-DEADLINE-EVENT", span, message)
}

fn kernel_context_diagnostic(
    source: &SourceText,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-CONTEXT-TRANSFER", span, message)
}

fn user_context_diagnostic(
    source: &SourceText,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-USER-CONTEXT", span, message)
}

fn field_natural(
    source: &SourceText,
    field: &ProductField,
    expected_label: &str,
) -> Result<u64, Diagnostic> {
    let Some(label) = field.label else {
        return Err(user_context_diagnostic(
            source,
            field.value.span(),
            format!("user syscall dispatch requires named `{expected_label}`"),
        ));
    };
    if source.slice(label) != expected_label {
        return Err(user_context_diagnostic(
            source,
            label,
            format!("user syscall dispatch requires `{expected_label}` in source order"),
        ));
    }
    parse_storage_natural(source, &field.value)
}

fn field_static_string(
    source: &SourceText,
    field: &ProductField,
    expected_label: &str,
) -> Result<String, Diagnostic> {
    let Some(label) = field.label else {
        return Err(user_context_diagnostic(
            source,
            field.value.span(),
            format!("user syscall dispatch requires named `{expected_label}`"),
        ));
    };
    if source.slice(label) != expected_label {
        return Err(user_context_diagnostic(
            source,
            label,
            format!("user syscall dispatch requires `{expected_label}` in source order"),
        ));
    }
    let Expression::String(value) = field.value else {
        return Err(user_context_diagnostic(
            source,
            field.value.span(),
            format!("user syscall dispatch requires static string `{expected_label}`"),
        ));
    };
    parse_string(source.slice(value))
        .map(str::to_owned)
        .ok_or_else(|| user_context_diagnostic(source, value, "invalid static uname field"))
}

fn invalid_entry_set(source: &SourceText, span: Span) -> Diagnostic {
    source_diagnostic(
        source,
        "E-SYSTEMS-ENTRY-SET",
        span,
        "the cooperative systems artifact requires exactly `bootstrap-storage`, `bootstrap`, `debug-break`, `local-notification`, `deadline-notification`, `kernel-thread`, and `terminal-thread` entries",
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
    if context == CompilerSystemsContextKind::KernelThread {
        return analyze_kernel_thread_handler(
            source,
            source.slice(*name),
            *is_static,
            parameters,
            *result,
            effect_bound.as_ref(),
            clauses,
            body,
            *span,
        );
    }
    let (context_classifier, disposition_classifier) = match context {
        CompilerSystemsContextKind::Bootstrap => ("BootstrapContext", "BootstrapDisposition"),
        CompilerSystemsContextKind::DebugBreak => ("DebugBreakContext", "DebugBreakDisposition"),
        CompilerSystemsContextKind::LocalNotificationInterrupt => (
            "LocalNotificationInterruptContext",
            "LocalNotificationInterruptDisposition",
        ),
        CompilerSystemsContextKind::DeadlineInterrupt => (
            "DeadlineInterruptContext InitialMonotonicClock",
            "DeadlineInterruptDisposition InitialMonotonicClock",
        ),
        CompilerSystemsContextKind::UserSyscall => {
            ("InitialUserSyscallContext", "InitialUserSyscallDisposition")
        }
        CompilerSystemsContextKind::KernelThread => unreachable!("handled above"),
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
        return analyze_bootstrap_handler(
            source,
            source.slice(*name),
            body,
            context_name,
            bootstrap_storage.expect("bootstrap handler carries its storage descriptor"),
        );
    }
    if context == CompilerSystemsContextKind::LocalNotificationInterrupt {
        return analyze_local_notification_handler(source, source.slice(*name), body, context_name);
    }
    if context == CompilerSystemsContextKind::DeadlineInterrupt {
        return analyze_deadline_handler(source, source.slice(*name), body, context_name);
    }
    if context == CompilerSystemsContextKind::UserSyscall {
        return analyze_user_syscall_handler(source, source.slice(*name), body, context_name);
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
    effects.push(disposition.semantic_identity(context).to_owned());
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

fn analyze_user_syscall_handler(
    source: &SourceText,
    name: &str,
    body: &[Statement],
    context_name: &str,
) -> Result<CompilerSystemsHandler, Diagnostic> {
    let [Statement::Expression(Expression::Application { items, span })] = body else {
        return Err(user_context_diagnostic(
            source,
            body.first().map_or(Span::new(0, 0), statement_span),
            "the initial user-syscall handler requires one exhaustive sealed dispatch disposition",
        ));
    };
    let [
        receiver,
        linux,
        syscall,
        dispatch,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user syscall dispatch requires the sealed implemented-call fields and unsupported-syscall error",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, linux, "linux")
        || !identifier_is(source, syscall, "syscall")
        || !identifier_is(source, dispatch, "dispatch")
        || fields.len() != 65
        || field_natural(source, &fields[0], "getpid-result")? != 1
        || field_static_string(source, &fields[1], "uname-sysname")? != "Linux"
        || field_static_string(source, &fields[2], "uname-nodename")? != "topal"
        || field_static_string(source, &fields[3], "uname-release")? != "7.2.9-topal"
        || field_static_string(source, &fields[4], "uname-version")? != "#1 Topal"
        || field_static_string(source, &fields[5], "uname-machine")? != "x86_64"
        || field_static_string(source, &fields[6], "uname-domainname")? != "(none)"
        || field_natural(source, &fields[7], "nanosleep-seconds")? != 0
        || field_natural(source, &fields[8], "nanosleep-nanoseconds")? != 0
        || field_natural(source, &fields[9], "nanosleep-remainder-candidate")? != 0
        || field_natural(source, &fields[10], "clock-gettime-result")? != 0
        || field_natural(source, &fields[11], "clock-getres-result")? != 0
        || field_natural(source, &fields[12], "write-descriptor")? != 1
        || field_natural(source, &fields[13], "write-maximum-bytes")? != 256
        || field_natural(source, &fields[14], "read-descriptor")? != 0
        || field_natural(source, &fields[15], "read-maximum-bytes")? != 256
        || field_natural(source, &fields[16], "brk-data-page-bytes")? != 4096
        || field_natural(source, &fields[17], "getppid-result")? != 0
        || field_natural(source, &fields[18], "gettid-result")? != 1
        || field_natural(source, &fields[19], "getuid-result")? != 0
        || field_natural(source, &fields[20], "geteuid-result")? != 0
        || field_natural(source, &fields[21], "getgid-result")? != 0
        || field_natural(source, &fields[22], "getegid-result")? != 0
        || field_natural(source, &fields[23], "arch-set-fs-result")? != 0
        || field_natural(source, &fields[24], "arch-get-fs-result")? != 0
        || field_natural(source, &fields[25], "set-tid-address-result")? != 1
        || field_natural(source, &fields[26], "set-robust-list-result")? != 0
        || field_natural(source, &fields[27], "getrlimit-result")? != 0
        || field_natural(source, &fields[28], "prlimit64-result")? != 0
        || field_natural(source, &fields[29], "rt-sigprocmask-result")? != 0
        || field_natural(source, &fields[30], "anonymous-page-bytes")? != 4096
        || field_natural(source, &fields[31], "madvise-result")? != 0
        || field_natural(source, &fields[32], "mprotect-result")? != 0
        || field_natural(source, &fields[33], "munmap-result")? != 0
        || field_natural(source, &fields[34], "exit-group-status")? != 0
        || field_natural(source, &fields[35], "unsupported-syscall-error")? != 38
        || field_natural(source, &fields[36], "sched-yield-result")? != 0
        || field_natural(source, &fields[37], "initial-descriptor-count")? != 3
        || field_natural(source, &fields[38], "bad-descriptor-error")? != 9
        || field_natural(source, &fields[39], "too-many-open-files-error")? != 24
        || field_natural(source, &fields[40], "console-input-description")? != 1
        || field_natural(source, &fields[41], "console-output-description")? != 2
        || field_natural(source, &fields[42], "console-error-description")? != 3
        || field_natural(source, &fields[43], "invalid-argument-error")? != 22
        || field_natural(source, &fields[44], "fcntl-getfd-command")? != 1
        || field_natural(source, &fields[45], "fcntl-setfd-command")? != 2
        || field_natural(source, &fields[46], "fcntl-getfl-command")? != 3
        || field_natural(source, &fields[47], "close-on-exec-flag")? != 1
        || field_natural(source, &fields[48], "read-only-status")? != 0
        || field_natural(source, &fields[49], "write-only-status")? != 1
        || field_natural(source, &fields[50], "stat-layout-bytes")? != 144
        || field_natural(source, &fields[51], "console-device")? != 0
        || field_natural(source, &fields[52], "console-inode")? != 1
        || field_natural(source, &fields[53], "console-mode")? != 8576
        || field_natural(source, &fields[54], "console-special-device")? != 17408
        || field_natural(source, &fields[55], "console-block-size")? != 4096
        || field_natural(source, &fields[56], "terminal-layout-bytes")? != 36
        || field_natural(source, &fields[57], "terminal-get-attributes-request")? != 21505
        || field_natural(source, &fields[58], "terminal-control-flags")? != 2239
        || field_natural(source, &fields[59], "terminal-minimum-input")? != 1
        || field_natural(source, &fields[60], "not-a-terminal-error")? != 25
        || field_natural(source, &fields[61], "window-size-layout-bytes")? != 8
        || field_natural(source, &fields[62], "window-size-query-request")? != 21523
        || field_natural(source, &fields[63], "seek-maximum-whence")? != 4
        || field_natural(source, &fields[64], "illegal-seek-error")? != 29
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user syscall dispatch must exhaustively implement the sealed calls, return ENOSYS otherwise, and terminate exit_group with status zero",
        ));
    }
    let disposition = CompilerSystemsDisposition::DispatchInitialUserSyscall {
        getpid_result: 1,
        uname_sysname: "Linux".into(),
        uname_nodename: "topal".into(),
        uname_release: "7.2.9-topal".into(),
        uname_version: "#1 Topal".into(),
        uname_machine: "x86_64".into(),
        uname_domainname: "(none)".into(),
        nanosleep_seconds: 0,
        nanosleep_nanoseconds: 0,
        nanosleep_remainder_candidate: 0,
        clock_gettime_result: 0,
        clock_getres_result: 0,
        write_descriptor: 1,
        write_maximum_bytes: 256,
        read_descriptor: 0,
        read_maximum_bytes: 256,
        initial_descriptor_count: 3,
        bad_descriptor_error: 9,
        too_many_open_files_error: 24,
        console_input_description: 1,
        console_output_description: 2,
        console_error_description: 3,
        invalid_argument_error: 22,
        fcntl_getfd_command: 1,
        fcntl_setfd_command: 2,
        fcntl_getfl_command: 3,
        close_on_exec_flag: 1,
        read_only_status: 0,
        write_only_status: 1,
        stat_layout_bytes: 144,
        console_device: 0,
        console_inode: 1,
        console_mode: 8576,
        console_special_device: 17408,
        console_block_size: 4096,
        terminal_layout_bytes: 36,
        terminal_get_attributes_request: 21505,
        terminal_control_flags: 2239,
        terminal_minimum_input: 1,
        not_a_terminal_error: 25,
        window_size_layout_bytes: 8,
        window_size_query_request: 21523,
        seek_maximum_whence: 4,
        illegal_seek_error: 29,
        brk_data_page_bytes: 4096,
        getppid_result: 0,
        gettid_result: 1,
        getuid_result: 0,
        geteuid_result: 0,
        getgid_result: 0,
        getegid_result: 0,
        arch_set_fs_result: 0,
        arch_get_fs_result: 0,
        set_tid_address_result: 1,
        set_robust_list_result: 0,
        getrlimit_result: 0,
        prlimit64_result: 0,
        rt_sigprocmask_result: 0,
        anonymous_page_bytes: 4096,
        madvise_result: 0,
        mprotect_result: 0,
        munmap_result: 0,
        exit_group_status: 0,
        unsupported_syscall_error: 38,
        sched_yield_result: 0,
    };
    Ok(CompilerSystemsHandler {
        name: name.to_owned(),
        context: CompilerSystemsContextKind::UserSyscall,
        operations: vec![
            CompilerSystemsOperation::CopyInitialUnameToUser,
            CompilerSystemsOperation::CopyInitialTimespecFromUser,
        ],
        effects: vec![
            SYSTEMS_USER_SYSCALL_DISPATCH.to_owned(),
            SYSTEMS_USER_COPY_FROM.to_owned(),
            SYSTEMS_USER_COPY_TO.to_owned(),
        ],
        disposition,
    })
}

#[allow(clippy::too_many_arguments)]
fn analyze_kernel_thread_handler(
    source: &SourceText,
    name: &str,
    is_static: bool,
    parameters: &[FunctionParameter],
    result: Span,
    effect_bound: Option<&Span>,
    clauses: &FunctionClauses,
    body: &[Statement],
    span: Span,
) -> Result<CompilerSystemsHandler, Diagnostic> {
    if is_static
        || parameters.len() != 2
        || source.slice(parameters[0].classifier) != "KernelThreadContext InitialProcessor"
        || source.slice(parameters[1].classifier) != "SuspendedKernelContext InitialProcessor"
        || source.slice(result) != "KernelThreadDisposition InitialProcessor"
        || parameters.iter().any(|parameter| {
            parameter.qualifier.is_some()
                || !parameter.fields.is_empty()
                || parameter.default.is_some()
        })
        || effect_bound.is_some()
        || !clauses_are_empty(clauses)
    {
        return Err(kernel_context_diagnostic(
            source,
            span,
            format!(
                "handler `{name}` must have exactly `fn (context : KernelThreadContext InitialProcessor, caller : SuspendedKernelContext InitialProcessor) -> KernelThreadDisposition InitialProcessor`"
            ),
        ));
    }
    let terminal = name == "terminal-thread-handler";
    let cooperative = name == "kernel-thread-handler";
    let completion_blocking = name == "completion-thread-handler";
    if !terminal && !cooperative && !completion_blocking {
        return Err(kernel_context_diagnostic(
            source,
            span,
            "the profile requires its preemptible, completion-blocking, and terminal thread handlers",
        ));
    }
    let context_name = source.slice(parameters[0].name);
    let caller_name = source.slice(parameters[1].name);
    let (_marker, retirement, returned_caller, operations) = if cooperative || completion_blocking {
        let [marker, await_operation, resumed_marker, retirement] = body else {
            return Err(kernel_context_diagnostic(
                source,
                span,
                "a suspendable kernel thread must enter, await its exact event, resume, and retire",
            ));
        };
        let entered = analyze_operation(
            source,
            marker,
            CompilerSystemsContextKind::KernelThread,
            context_name,
        )?;
        let expected_entered = if cooperative {
            "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED"
        } else {
            "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_ENTERED"
        };
        if entered
            != (CompilerSystemsOperation::ConsoleWrite {
                text: expected_entered.into(),
            })
        {
            return Err(kernel_context_diagnostic(
                source,
                statement_span(marker),
                "the suspendable kernel thread requires its exact entry marker",
            ));
        }
        let await_operation_span = statement_span(await_operation);
        let await_operation = analyze_operation(
            source,
            await_operation,
            CompilerSystemsContextKind::KernelThread,
            context_name,
        )?;
        let expected_await = if cooperative {
            CompilerSystemsOperation::AwaitDeadlinePreemption
        } else {
            CompilerSystemsOperation::AwaitKernelCompletion
        };
        if await_operation != expected_await {
            return Err(kernel_context_diagnostic(
                source,
                await_operation_span,
                "the suspendable kernel thread requires its sealed wait operation",
            ));
        }
        let resumed = analyze_operation(
            source,
            resumed_marker,
            CompilerSystemsContextKind::KernelThread,
            context_name,
        )?;
        let expected_resumed = if cooperative {
            "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED"
        } else {
            "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RESUMED"
        };
        if resumed
            != (CompilerSystemsOperation::ConsoleWrite {
                text: expected_resumed.into(),
            })
        {
            return Err(kernel_context_diagnostic(
                source,
                statement_span(resumed_marker),
                "the suspendable kernel thread requires its exact resumed marker",
            ));
        }
        (
            marker,
            retirement,
            caller_name.to_owned(),
            vec![entered, await_operation, resumed],
        )
    } else {
        let [marker, retirement] = body else {
            return Err(kernel_context_diagnostic(
                source,
                span,
                "the terminal kernel thread must write its entry marker and retire",
            ));
        };
        let entered = analyze_operation(
            source,
            marker,
            CompilerSystemsContextKind::KernelThread,
            context_name,
        )?;
        if entered
            != (CompilerSystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED".into(),
            })
        {
            return Err(kernel_context_diagnostic(
                source,
                statement_span(marker),
                "the terminal kernel thread requires its exact entry marker",
            ));
        }
        (marker, retirement, caller_name.to_owned(), vec![entered])
    };
    let Statement::Expression(Expression::Application {
        items,
        span: retirement_span,
    }) = retirement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(retirement),
            "kernel-thread retirement must be `context kernel context retire to caller`",
        ));
    };
    let [context, kernel, context_word, retire, to, caller] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *retirement_span,
            "kernel-thread retirement must be `context kernel context retire to caller`",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, retire, "retire")
        || !identifier_is(source, to, "to")
        || !identifier_is(source, caller, &returned_caller)
    {
        return Err(kernel_context_diagnostic(
            source,
            *retirement_span,
            "kernel-thread retirement must consume its live context and matching suspended caller",
        ));
    }
    let mut effects = vec![
        SYSTEMS_CONSOLE_WRITE.to_owned(),
        SYSTEMS_KERNEL_CONTEXT_RETIRE.to_owned(),
    ];
    if cooperative {
        effects.push(SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION.to_owned());
    } else if completion_blocking {
        effects.push(SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE.to_owned());
    }
    effects.sort();
    Ok(CompilerSystemsHandler {
        name: name.to_owned(),
        context: CompilerSystemsContextKind::KernelThread,
        operations,
        disposition: CompilerSystemsDisposition::RetireKernelContextToCaller,
        effects,
    })
}

fn analyze_bootstrap_handler(
    source: &SourceText,
    name: &str,
    body: &[Statement],
    context_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CompilerSystemsHandler, Diagnostic> {
    let checked = analyze_boot_memory_decision(source, body, context_name, storage)?;
    let mut effects = checked
        .operations
        .iter()
        .map(|operation| operation.semantic_identity().to_owned())
        .collect::<Vec<_>>();
    effects.push(
        checked
            .disposition
            .semantic_identity(CompilerSystemsContextKind::Bootstrap)
            .to_owned(),
    );
    effects.sort();
    effects.dedup();
    Ok(CompilerSystemsHandler {
        name: name.to_owned(),
        context: CompilerSystemsContextKind::Bootstrap,
        operations: checked.operations,
        disposition: checked.disposition,
        effects,
    })
}

fn analyze_local_notification_handler(
    source: &SourceText,
    name: &str,
    body: &[Statement],
    context_name: &str,
) -> Result<CompilerSystemsHandler, Diagnostic> {
    let [complete, disposition] = body else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-LOCAL-INTERRUPT",
            body.first().map_or(Span::new(0, 0), statement_span),
            "local-notification handler requires consuming completion followed by resume",
        ));
    };
    let (completed, items, span) = affine_application_binding(source, complete, "completion")?;
    let [receiver, local, notification, complete_operation] = items else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-LOCAL-INTERRUPT",
            span,
            "completion must bind `completed is context local notification complete`",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, local, "local")
        || !identifier_is(source, notification, "notification")
        || !identifier_is(source, complete_operation, "complete")
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-LOCAL-INTERRUPT",
            span,
            "completion must consume the live local-notification interrupt context",
        ));
    }
    let completed_name = source.slice(completed);
    let disposition = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::LocalNotificationInterrupt,
        completed_name,
    )?;
    if disposition != CompilerSystemsDisposition::Resume {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-LOCAL-INTERRUPT",
            statement_span(body.last().expect("two statements")),
            "local-notification completion must end in resume",
        ));
    }
    let operations = vec![CompilerSystemsOperation::CompleteLocalNotification];
    let mut effects = vec![
        SYSTEMS_LOCAL_NOTIFICATION_COMPLETE.to_owned(),
        disposition
            .semantic_identity(CompilerSystemsContextKind::LocalNotificationInterrupt)
            .to_owned(),
    ];
    effects.sort();
    Ok(CompilerSystemsHandler {
        name: name.to_owned(),
        context: CompilerSystemsContextKind::LocalNotificationInterrupt,
        operations,
        disposition,
        effects,
    })
}

fn analyze_deadline_handler(
    source: &SourceText,
    name: &str,
    body: &[Statement],
    context_name: &str,
) -> Result<CompilerSystemsHandler, Diagnostic> {
    let [complete, disposition] = body else {
        return Err(deadline_diagnostic(
            source,
            body.first().map_or(Span::new(0, 0), statement_span),
            "deadline handler requires consuming completion followed by bound-context preemption",
        ));
    };
    let (completed, items, span) = affine_application_binding(source, complete, "completion")?;
    let [receiver, deadline, notification, complete_operation] = items else {
        return Err(deadline_diagnostic(
            source,
            span,
            "completion must bind `completed is context deadline notification complete`",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, deadline, "deadline")
        || !identifier_is(source, notification, "notification")
        || !identifier_is(source, complete_operation, "complete")
    {
        return Err(deadline_diagnostic(
            source,
            span,
            "completion must consume the live deadline interrupt context",
        ));
    }
    let completed_name = source.slice(completed);
    let disposition = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::DeadlineInterrupt,
        completed_name,
    )?;
    if disposition != CompilerSystemsDisposition::PreemptKernelContext {
        return Err(deadline_diagnostic(
            source,
            statement_span(body.last().expect("two statements")),
            "deadline completion must preempt the bound kernel context",
        ));
    }
    let operations = vec![CompilerSystemsOperation::CompleteDeadline];
    let mut effects = vec![
        SYSTEMS_DEADLINE_COMPLETE.to_owned(),
        SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT.to_owned(),
    ];
    effects.sort();
    Ok(CompilerSystemsHandler {
        name: name.to_owned(),
        context: CompilerSystemsContextKind::DeadlineInterrupt,
        operations,
        disposition,
        effects,
    })
}

fn affine_application_binding<'a>(
    source: &SourceText,
    statement: &'a Statement,
    operation: &str,
) -> Result<(Span, &'a [Expression], Span), Diagnostic> {
    match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => Ok((*name, items, *span)),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => Ok((*name, items, *span)),
        _ => Err(local_interrupt_diagnostic(
            source,
            statement_span(statement),
            format!("{operation} must use one unclassified affine binding"),
        )),
    }
}

include!("systems_boot_memory.rs");
include!("systems_frames.rs");
include!("systems_mapping.rs");
include!("systems_translation.rs");
include!("systems_translation_edit.rs");
include!("systems_critical.rs");

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
        storage,
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
        storage,
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
    storage: &CompilerBootstrapStorageDescriptor,
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
        RegionComparisonContext {
            context_name,
            region_name: ok_binding,
            loaded_name: &loaded_name,
            region_request: request,
            storage,
        },
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
    let mut operations = vec![
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
    ];
    operations.extend(comparison.success_operations);
    Ok(CheckedBootstrapRegionDecision {
        operations,
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

fn raw_result_action<'a>(
    source: &SourceText,
    statement: &'a Statement,
    expected: &str,
) -> Result<(String, &'a Expression), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "nested context allocation requires explicit `Ok` and `Error` actions",
        ));
    };
    let [constructor, Expression::Identifier(binding), then, action] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "nested context allocation action must bind its result before `then`",
        ));
    };
    if !identifier_is(source, constructor, expected) || !identifier_is(source, then, "then") {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            format!("nested context allocation requires the `{expected}` action in order"),
        ));
    }
    Ok((source.slice(*binding).to_owned(), action))
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
    success_operations: Vec<CompilerSystemsOperation>,
    failure_message: String,
    success_disposition: CompilerSystemsDisposition,
}

#[derive(Clone, Copy)]
struct RegionComparisonContext<'a> {
    context_name: &'a str,
    region_name: &'a str,
    loaded_name: &'a str,
    region_request: CompilerBootstrapStorageRequest,
    storage: &'a CompilerBootstrapStorageDescriptor,
}

fn parse_region_comparison(
    source: &SourceText,
    statement: &Statement,
    true_rule: &Statement,
    false_rule: &Statement,
    context: RegionComparisonContext<'_>,
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
    if !identifier_is(source, loaded, context.loaded_name) {
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
    let atomic = parse_atomic_word_sequence(
        source,
        true_statements,
        context.context_name,
        context.region_name,
        context.region_request,
        context.storage,
    )?;

    let false_statements = action_block(source, false_action, "failed byte comparison")?;
    let [release, disposition] = false_statements else {
        return Err(storage_diagnostic(
            source,
            false_action.span(),
            "failed byte comparison requires region release and fatal disposition",
        ));
    };
    parse_region_release(source, release, context.context_name, context.region_name)?;
    let failure = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        context.context_name,
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
        success_operations: atomic.operations,
        failure_message,
        success_disposition: atomic.disposition,
    })
}

struct CheckedAtomicSequence {
    operations: Vec<CompilerSystemsOperation>,
    disposition: CompilerSystemsDisposition,
}

fn parse_atomic_word_sequence(
    source: &SourceText,
    statements: &[Statement],
    context_name: &str,
    region_name: &str,
    region_request: CompilerBootstrapStorageRequest,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedAtomicSequence, Diagnostic> {
    let [create] = statements else {
        return Err(atomic_diagnostic(
            source,
            statements.first().map_or(Span::new(0, 0), statement_span),
            "atomic word lifecycle requires create and exhaustive `Exchanged`/`Observed` compare/exchange actions",
        ));
    };
    let (atomic_name, request, continuation) =
        parse_atomic_word_create(source, create, region_name)?;
    if request != CompilerAtomicWordRequest::initial()
        || request.offset_bytes % 8 != 0
        || request
            .offset_bytes
            .checked_add(8)
            .is_none_or(|end| end > region_request.byte_count)
    {
        return Err(atomic_diagnostic(
            source,
            statement_span(create),
            "the initial atomic word requires aligned offset 8, initial value 41, cpu-shared domain, and eight in-bounds bytes",
        ));
    }
    let [
        Statement::Expression(Expression::DecisionTable {
            subject: compare_exchange,
            rules,
            span,
        }),
    ] = continuation
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(create),
            "atomic creation must continue directly into one exhaustive compare/exchange decision",
        ));
    };
    let (expected, desired, success_order, failure_order) =
        parse_atomic_compare_exchange(source, compare_exchange, &atomic_name)?;
    if (expected, desired, success_order, failure_order)
        != (
            41,
            42,
            CompilerAtomicOrder::AcquireRelease,
            CompilerAtomicOrder::Acquire,
        )
    {
        return Err(atomic_diagnostic(
            source,
            compare_exchange.span(),
            "the initial compare/exchange requires expected 41, desired 42, acquire-release success, and acquire failure",
        ));
    }
    let (exchanged_action, observed_action) = atomic_result_actions(source, rules, *span)?;
    let exchanged = parse_atomic_exchanged_action(
        source,
        exchanged_action,
        context_name,
        region_name,
        &atomic_name,
        storage,
    )?;
    let compare_failure_message = parse_atomic_observed_action(
        source,
        observed_action,
        context_name,
        region_name,
        &atomic_name,
    )?;

    let mut operations = vec![
        CompilerSystemsOperation::AtomicWordCreate { request },
        CompilerSystemsOperation::AtomicCompareExchangeEquals {
            expected,
            desired,
            success_order,
            failure_order,
            failure_message: compare_failure_message,
        },
        CompilerSystemsOperation::AtomicLoadEquals {
            order: exchanged.load_order,
            expected: exchanged.expected,
            failure_message: exchanged.load_failure_message,
        },
        CompilerSystemsOperation::AtomicWordEnd,
    ];
    operations.extend(exchanged.success_operations);
    Ok(CheckedAtomicSequence {
        operations,
        disposition: exchanged.success_disposition,
    })
}

fn parse_atomic_observed_action(
    source: &SourceText,
    action: &Expression,
    context_name: &str,
    region_name: &str,
    atomic_name: &str,
) -> Result<String, Diagnostic> {
    let statements = action_block(source, action, "observed compare/exchange")?;
    let [end] = statements else {
        return Err(atomic_diagnostic(
            source,
            action.span(),
            "observed compare/exchange requires atomic end, region release, and fatal disposition",
        ));
    };
    let continuation = parse_atomic_end(source, end, atomic_name, region_name)?;
    let [release, disposition] = continuation else {
        return Err(atomic_diagnostic(
            source,
            action.span(),
            "observed compare/exchange requires region release and fatal disposition after atomic end",
        ));
    };
    parse_region_release(source, release, context_name, region_name)?;
    let CompilerSystemsDisposition::Fatal { message } = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(disposition),
            "observed compare/exchange must enter the fatal disposition",
        ));
    };
    Ok(message)
}

fn parse_atomic_word_create<'a>(
    source: &SourceText,
    statement: &'a Statement,
    region_name: &str,
) -> Result<(String, CompilerAtomicWordRequest, &'a [Statement]), Diagnostic> {
    let Statement::Implementation {
        name,
        classifier: Expression::Application { items, span },
        declarations,
        ..
    } = statement
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(statement),
            "atomic creation must bind `atomic is region atomic word create (...)`",
        ));
    };
    let [
        region,
        atomic,
        word,
        create,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic creation must bind `atomic is region atomic word create (...)`",
        ));
    };
    if !identifier_is(source, region, region_name)
        || !identifier_is(source, atomic, "atomic")
        || !identifier_is(source, word, "word")
        || !identifier_is(source, create, "create")
    {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic word creation must consume the live ordinary region",
        ));
    }
    let offset_bytes = atomic_natural_field(source, fields, "offset-bytes")?;
    let initial_value = atomic_natural_field(source, fields, "initial-value")?;
    let domain = atomic_identity_field(source, fields, "domain", "cpu-shared")?;
    if fields.len() != 3 {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic word creation requires exactly `offset-bytes`, `initial-value`, and `domain`",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        CompilerAtomicWordRequest {
            offset_bytes,
            initial_value,
            domain: match domain.as_str() {
                "cpu-shared" => CompilerAtomicDomain::CpuShared,
                _ => unreachable!("identity helper checks the sealed domain"),
            },
        },
        declarations,
    ))
}

fn parse_atomic_compare_exchange(
    source: &SourceText,
    expression: &Expression,
    atomic_name: &str,
) -> Result<(u64, u64, CompilerAtomicOrder, CompilerAtomicOrder), Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(atomic_diagnostic(
            source,
            expression.span(),
            "compare/exchange must use the live atomic location",
        ));
    };
    let [
        atomic,
        compare,
        exchange,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(atomic_diagnostic(
            source,
            *span,
            "compare/exchange must be `atomic compare exchange (...)`",
        ));
    };
    if !identifier_is(source, atomic, atomic_name)
        || !identifier_is(source, compare, "compare")
        || !identifier_is(source, exchange, "exchange")
        || fields.len() != 4
    {
        return Err(atomic_diagnostic(
            source,
            *span,
            "compare/exchange requires the live atomic location and four named parameters",
        ));
    }
    Ok((
        atomic_natural_field(source, fields, "expected")?,
        atomic_natural_field(source, fields, "desired")?,
        atomic_order_field(source, fields, "success-order")?,
        atomic_order_field(source, fields, "failure-order")?,
    ))
}

fn atomic_result_actions<'a>(
    source: &SourceText,
    rules: &'a [DecisionRule],
    span: Span,
) -> Result<(&'a Expression, &'a Expression), Diagnostic> {
    let [exchanged, observed] = rules else {
        return Err(atomic_diagnostic(
            source,
            span,
            "compare/exchange requires exactly one `Exchanged` and one `Observed` action",
        ));
    };
    match (&exchanged.matcher, &observed.matcher) {
        (
            DecisionMatcher::Union {
                alternative: exchanged_name,
                ..
            },
            DecisionMatcher::Union {
                alternative: observed_name,
                ..
            },
        ) if source.slice(*exchanged_name) == "Exchanged"
            && source.slice(*observed_name) == "Observed" =>
        {
            Ok((&exchanged.action, &observed.action))
        }
        _ => Err(atomic_diagnostic(
            source,
            span,
            "compare/exchange actions must appear once in `Exchanged`, `Observed` order and bind the observed value",
        )),
    }
}

struct CheckedAtomicExchanged {
    load_order: CompilerAtomicOrder,
    expected: u64,
    load_failure_message: String,
    success_operations: Vec<CompilerSystemsOperation>,
    success_disposition: CompilerSystemsDisposition,
}

fn parse_atomic_exchanged_action(
    source: &SourceText,
    action: &Expression,
    context_name: &str,
    region_name: &str,
    atomic_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedAtomicExchanged, Diagnostic> {
    let statements = action_block(source, action, "exchanged compare/exchange")?;
    let [load, comparison, true_rule, false_rule] = statements else {
        return Err(atomic_diagnostic(
            source,
            action.span(),
            "exchanged compare/exchange requires acquire load and exhaustive equality actions",
        ));
    };
    let (loaded_name, load_order) = parse_atomic_load(source, load, atomic_name)?;
    let expected = parse_atomic_equality(source, comparison, &loaded_name)?;
    let true_action = parse_boolean_rule(source, true_rule, true)?;
    let false_action = parse_boolean_rule(source, false_rule, false)?;
    let (success_operations, success_disposition) = parse_atomic_exchanged_success(
        source,
        true_action,
        context_name,
        region_name,
        atomic_name,
        storage,
    )?;
    let load_failure_message = parse_atomic_exchanged_failure(
        source,
        false_action,
        context_name,
        region_name,
        atomic_name,
    )?;
    Ok(CheckedAtomicExchanged {
        load_order,
        expected,
        load_failure_message,
        success_operations,
        success_disposition,
    })
}

#[allow(clippy::too_many_lines)] // The sealed atomic-to-interrupt-to-deadline success path is audited together.
fn parse_atomic_exchanged_success(
    source: &SourceText,
    true_action: &Expression,
    context_name: &str,
    region_name: &str,
    atomic_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<(Vec<CompilerSystemsOperation>, CompilerSystemsDisposition), Diagnostic> {
    let success = action_block(source, true_action, "successful atomic load")?;
    let [end] = success else {
        return Err(atomic_diagnostic(
            source,
            true_action.span(),
            "successful atomic load requires atomic end, atomic and memory markers, region release, and final disposition",
        ));
    };
    let success_continuation = parse_atomic_end(source, end, atomic_name, region_name)?;
    let [
        atomic_console,
        memory_console,
        release,
        send,
        wait,
        interrupt_console,
        first_clock,
        second_clock,
        time_console,
        deadline,
        context_decision,
    ] = success_continuation
    else {
        return Err(atomic_diagnostic(
            source,
            true_action.span(),
            "successful atomic load requires atomic and memory markers, region release, local-notification completion, two monotonic-clock observations, one deadline-preemptible context lifecycle, and final disposition after atomic end",
        ));
    };
    let atomic_marker = analyze_operation(
        source,
        atomic_console,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;
    let memory_marker = analyze_operation(
        source,
        memory_console,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;
    if !matches!(atomic_marker, CompilerSystemsOperation::ConsoleWrite { .. })
        || !matches!(memory_marker, CompilerSystemsOperation::ConsoleWrite { .. })
    {
        return Err(atomic_diagnostic(
            source,
            true_action.span(),
            "successful atomic load markers must be console writes",
        ));
    }
    parse_region_release(source, release, context_name, region_name)?;
    let (pending_name, send_operation) = parse_local_notification_send(source, send, context_name)?;
    let (resumed_name, wait_operation) =
        parse_local_notification_wait(source, wait, &pending_name)?;
    let interrupt_marker = analyze_operation(
        source,
        interrupt_console,
        CompilerSystemsContextKind::Bootstrap,
        &resumed_name,
    )?;
    if interrupt_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_INTERRUPT_OK".into(),
        })
    {
        return Err(local_interrupt_diagnostic(
            source,
            statement_span(interrupt_console),
            "local-notification wait must be followed by the exact interrupt success marker",
        ));
    }
    let (first_name, first_clock_operation) =
        parse_monotonic_clock_now(source, first_clock, &resumed_name)?;
    let (second_name, second_clock_operation) =
        parse_monotonic_clock_now(source, second_clock, &resumed_name)?;
    if first_name == second_name {
        return Err(monotonic_clock_diagnostic(
            source,
            statement_span(second_clock),
            "monotonic-clock observations require distinct instant bindings",
        ));
    }
    let time_marker = analyze_operation(
        source,
        time_console,
        CompilerSystemsContextKind::Bootstrap,
        &resumed_name,
    )?;
    if time_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_TIME_OK".into(),
        })
    {
        return Err(monotonic_clock_diagnostic(
            source,
            statement_span(time_console),
            "the two clock observations must be followed by the exact time success marker",
        ));
    }
    let (deadline_name, deadline_operation) = parse_deadline_after(source, deadline, &second_name)?;
    let checked_context = analyze_kernel_context_decision(
        source,
        context_decision,
        &resumed_name,
        &deadline_name,
        storage,
    )?;
    let mut operations = vec![
        atomic_marker,
        memory_marker,
        CompilerSystemsOperation::BootstrapRelease,
        send_operation,
        wait_operation,
        interrupt_marker,
        first_clock_operation,
        second_clock_operation,
        time_marker,
        deadline_operation,
    ];
    operations.extend(checked_context.operations);
    Ok((operations, checked_context.disposition))
}

fn analyze_kernel_context_decision(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    deadline_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let Statement::Expression(Expression::DecisionTable {
        subject,
        rules,
        span,
    }) = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "kernel-context stack allocation requires a subject and exhaustive `Ok`/`Error` actions",
        ));
    };
    let request = parse_bootstrap_allocation(source, subject, context_name, storage)?;
    if request
        != (CompilerBootstrapStorageRequest {
            byte_count: 16_384,
            alignment_bytes: 16,
            placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            subject.span(),
            "the initial kernel context requires one 16 KiB, 16-byte-aligned bootstrap-reclaimable stack",
        ));
    }
    let (stack_name, ok_action, error_action) = result_actions(source, rules, *span)?;
    let CompilerSystemsDisposition::Fatal { .. } =
        action_disposition(source, error_action, context_name)?
    else {
        return Err(kernel_context_diagnostic(
            source,
            error_action.span(),
            "kernel-context stack allocation failure must enter the fatal disposition",
        ));
    };
    let statements = action_block(source, ok_action, "kernel-context stack allocation")?;
    let [
        create_completion,
        create_cooperative,
        completion_subject_statement,
        completion_ok_statement,
        completion_error_statement,
    ] = statements
    else {
        return Err(kernel_context_diagnostic(
            source,
            ok_action.span(),
            "context success requires completion creation, the preemptible worker, and the completion-worker stack allocation",
        ));
    };
    let completion_name = parse_kernel_completion_create(source, create_completion, context_name)?;
    let cooperative_name = parse_kernel_context_create(
        source,
        create_cooperative,
        context_name,
        &stack_name,
        "kernel-thread",
    )?;
    let Statement::Expression(completion_subject) = completion_subject_statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(completion_subject_statement),
            "preemptible context creation must be followed by the completion-worker stack allocation",
        ));
    };
    let completion_request =
        parse_bootstrap_allocation(source, completion_subject, context_name, storage)?;
    if completion_request != request {
        return Err(kernel_context_diagnostic(
            source,
            completion_subject.span(),
            "all worker contexts require disjoint 16 KiB bootstrap-reclaimable stacks",
        ));
    }
    let (completion_stack_name, completion_ok) =
        raw_result_action(source, completion_ok_statement, "Ok")?;
    let (_, completion_error) = raw_result_action(source, completion_error_statement, "Error")?;
    let CompilerSystemsDisposition::Fatal { .. } =
        action_disposition(source, completion_error, context_name)?
    else {
        return Err(kernel_context_diagnostic(
            source,
            completion_error.span(),
            "completion-worker stack allocation failure must enter the fatal disposition",
        ));
    };
    let completion_statements = action_block(
        source,
        completion_ok,
        "completion kernel-context stack allocation",
    )?;
    let [
        create_completion_worker,
        terminal_subject_statement,
        terminal_ok_statement,
        terminal_error_statement,
    ] = completion_statements
    else {
        return Err(kernel_context_diagnostic(
            source,
            completion_ok.span(),
            "completion context success requires its create followed by the terminal stack allocation",
        ));
    };
    let completion_worker_name = parse_kernel_context_create(
        source,
        create_completion_worker,
        context_name,
        &completion_stack_name,
        "completion-thread",
    )?;
    let Statement::Expression(terminal_subject) = terminal_subject_statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(terminal_subject_statement),
            "completion context creation must be followed by the terminal stack allocation",
        ));
    };
    let terminal_request =
        parse_bootstrap_allocation(source, terminal_subject, context_name, storage)?;
    if terminal_request != request {
        return Err(kernel_context_diagnostic(
            source,
            terminal_subject.span(),
            "all worker contexts require disjoint 16 KiB bootstrap-reclaimable stacks",
        ));
    }
    let (terminal_stack_name, terminal_ok) =
        raw_result_action(source, terminal_ok_statement, "Ok")?;
    let (_, terminal_error) = raw_result_action(source, terminal_error_statement, "Error")?;
    let CompilerSystemsDisposition::Fatal { .. } =
        action_disposition(source, terminal_error, context_name)?
    else {
        return Err(kernel_context_diagnostic(
            source,
            terminal_error.span(),
            "terminal stack allocation failure must enter the fatal disposition",
        ));
    };
    let terminal_statements = action_block(source, terminal_ok, "terminal stack allocation")?;
    let [
        create_terminal,
        create_queue,
        enqueue_cooperative,
        enqueue_completion,
        enqueue_terminal,
        dequeue_cooperative,
        preemptible_transfer,
        take_preempted,
        preemption_marker_statement,
        enqueue_preempted,
        dequeue_completion,
        transfer_completion_until_signal,
        dequeue_terminal,
        transfer_terminal,
        terminal_marker_statement,
        reclaim_terminal,
    ] = terminal_statements
    else {
        return Err(kernel_context_diagnostic(
            source,
            terminal_ok.span(),
            "the FIFO dispatcher requires three workers, deadline preemption, completion blocking, exact source order, retirement, and reclamation",
        ));
    };
    let terminal_name = parse_kernel_context_create(
        source,
        create_terminal,
        context_name,
        &terminal_stack_name,
        "terminal-thread",
    )?;
    let queue_name = parse_runnable_queue_create(source, create_queue, context_name)?;
    parse_runnable_queue_enqueue(source, enqueue_cooperative, &queue_name, &cooperative_name)?;
    parse_runnable_queue_enqueue(
        source,
        enqueue_completion,
        &queue_name,
        &completion_worker_name,
    )?;
    parse_runnable_queue_enqueue(source, enqueue_terminal, &queue_name, &terminal_name)?;
    let selected_cooperative =
        parse_runnable_queue_dequeue(source, dequeue_cooperative, &queue_name)?;
    let preempted_name = parse_kernel_context_transfer_until_deadline(
        source,
        preemptible_transfer,
        context_name,
        &selected_cooperative,
        deadline_name,
    )?;
    let preempted_worker =
        parse_take_preempted_kernel_context(source, take_preempted, &preempted_name)?;
    let preemption_marker = analyze_operation(
        source,
        preemption_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;
    if preemption_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_DEADLINE_PREEMPTED".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(preemption_marker_statement),
            "the deadline-preemptible transfer requires its exact success marker",
        ));
    }
    parse_runnable_queue_enqueue(source, enqueue_preempted, &queue_name, &preempted_worker)?;
    let selected_completion =
        parse_runnable_queue_dequeue(source, dequeue_completion, &queue_name)?;
    let blocked_completion = parse_kernel_context_transfer_until_completion(
        source,
        transfer_completion_until_signal,
        context_name,
        &selected_completion,
        &completion_name,
    )?;
    let selected_terminal = parse_runnable_queue_dequeue(source, dequeue_terminal, &queue_name)?;
    let terminal_completed = parse_kernel_context_transfer(
        source,
        transfer_terminal,
        context_name,
        &selected_terminal,
        "CompletedKernelContextTransfer InitialProcessor",
    )?;
    let terminal_marker = analyze_operation(
        source,
        terminal_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;
    if terminal_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(terminal_marker_statement),
            "terminal retirement requires its exact marker",
        ));
    }
    let (after_terminal, after_terminal_statements) =
        parse_kernel_context_reclaim(source, reclaim_terminal, &terminal_completed)?;
    let [
        signal_completion,
        enqueue_completion,
        dequeue_cooperative_final,
        transfer_cooperative,
        cooperative_marker_statement,
        reclaim_cooperative,
    ] = after_terminal_statements
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(reclaim_terminal),
            "terminal reclamation must signal and enqueue the blocked worker before restoring the preempted worker",
        ));
    };
    let awakened_completion = parse_signal_blocked_kernel_completion(
        source,
        signal_completion,
        &blocked_completion,
        &completion_name,
    )?;
    parse_runnable_queue_enqueue(
        source,
        enqueue_completion,
        &queue_name,
        &awakened_completion,
    )?;
    let selected_cooperative_final =
        parse_runnable_queue_dequeue(source, dequeue_cooperative_final, &queue_name)?;
    let cooperative_completed = parse_kernel_context_transfer(
        source,
        transfer_cooperative,
        &after_terminal,
        &selected_cooperative_final,
        "CompletedKernelContextTransfer InitialProcessor",
    )?;
    let cooperative_marker = analyze_operation(
        source,
        cooperative_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &after_terminal,
    )?;
    if cooperative_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(cooperative_marker_statement),
            "preemptible-worker retirement requires its exact marker",
        ));
    }
    let (resumed_name, continuation) =
        parse_kernel_context_reclaim(source, reclaim_cooperative, &cooperative_completed)?;
    let [
        dequeue_completion_final,
        transfer_completion,
        completion_marker_statement,
        reclaim_completion,
    ] = continuation
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(reclaim_cooperative),
            "preemptible reclamation must resume and reclaim the awakened completion worker",
        ));
    };
    let selected_completion_final =
        parse_runnable_queue_dequeue(source, dequeue_completion_final, &queue_name)?;
    let completion_completed = parse_kernel_context_transfer(
        source,
        transfer_completion,
        &resumed_name,
        &selected_completion_final,
        "CompletedKernelContextTransfer InitialProcessor",
    )?;
    let completion_marker = analyze_operation(
        source,
        completion_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &resumed_name,
    )?;
    if completion_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RETIRED".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(completion_marker_statement),
            "completion-worker retirement requires its exact marker",
        ));
    }
    let (completion_resumed, completion_continuation) =
        parse_kernel_context_reclaim(source, reclaim_completion, &completion_completed)?;
    let [consume_completion, consume_queue, user_allocation @ ..] = completion_continuation else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(reclaim_completion),
            "completion-worker reclamation must consume completion and empty queue, complete one user syscall round trip, then reach disposition",
        ));
    };
    parse_kernel_completion_consume(
        source,
        consume_completion,
        &completion_resumed,
        &completion_name,
    )?;
    parse_runnable_queue_consume(source, consume_queue, &completion_resumed, &queue_name)?;
    let (subject, user_frames, success, _problem_name, failure, span) =
        result_sequence(source, user_allocation, "user backing allocation")?;
    let backing_request = parse_physical_frame_allocation(source, subject, &completion_resumed)?;
    if backing_request
        != (CompilerPhysicalFrameRequest {
            frame_count: 3,
            alignment_frames: 1,
        })
    {
        return Err(user_context_diagnostic(
            source,
            subject.span(),
            "initial user backing requires exactly three frames aligned to one frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: backing_failure,
    } = action_disposition(source, failure, &completion_resumed)?
    else {
        return Err(user_context_diagnostic(
            source,
            failure.span(),
            "user backing allocation failure must preserve the allocator context and enter `fatal`",
        ));
    };
    let user_statements = action_block(source, success, "user backing allocation success")?;
    let [prepare_address_space] = user_statements else {
        return Err(user_context_diagnostic(
            source,
            span,
            "user backing success must begin with one affine user-address-space preparation scope",
        ));
    };
    let (user_space, address_space_continuation) = parse_user_address_space_prepare(
        source,
        prepare_address_space,
        &completion_resumed,
        &user_frames,
    )?;
    let (scratch_subject, scratch_frames, scratch_success, _scratch_problem, scratch_failure, _) =
        result_sequence(
            source,
            address_space_continuation,
            "disjoint frame allocation",
        )?;
    let scratch_request =
        parse_physical_frame_allocation(source, scratch_subject, &completion_resumed)?;
    if scratch_request
        != (CompilerPhysicalFrameRequest {
            frame_count: 2,
            alignment_frames: 1,
        })
    {
        return Err(user_context_diagnostic(
            source,
            scratch_subject.span(),
            "the disjoint executable extent requires exactly two frames aligned to one frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: scratch_failure_message,
    } = action_disposition(source, scratch_failure, &completion_resumed)?
    else {
        return Err(user_context_diagnostic(
            source,
            scratch_failure.span(),
            "disjoint frame allocation failure must preserve the allocator context and enter `fatal`",
        ));
    };
    let scratch_statements =
        action_block(source, scratch_success, "disjoint frame allocation success")?;
    let [map_mapping, scratch_after_map @ ..] = scratch_statements else {
        return Err(user_context_diagnostic(
            source,
            span,
            "disjoint frame allocation success must map, store, load, unmap, and release its exact extent before consuming the prepared user address space",
        ));
    };
    let (mapped_scratch, map_continuation) = parse_user_address_space_map(
        source,
        map_mapping,
        &completion_resumed,
        &user_space,
        &scratch_frames,
        "probe-code",
    )?;
    let after_map =
        select_user_mapping_continuation(source, span, scratch_after_map, map_continuation, "map")?;
    let [store_mapping, scratch_after_store @ ..] = after_map else {
        return Err(user_context_diagnostic(
            source,
            span,
            "code mapping publication must be followed by entry installation, protection, verification, and executable promotion",
        ));
    };
    let (stored_scratch, store_continuation) = parse_user_mapping_entry_trampoline(
        source,
        store_mapping,
        &completion_resumed,
        &mapped_scratch,
        "install",
    )?;
    let after_store = select_user_mapping_continuation(
        source,
        span,
        scratch_after_store,
        store_continuation,
        "store",
    )?;
    let [protect_mapping, scratch_after_protect @ ..] = after_store else {
        return Err(user_context_diagnostic(
            source,
            span,
            "entry-trampoline installation must be followed by protection, verification, and executable promotion",
        ));
    };
    let (protected_scratch, protect_continuation) = parse_user_mapping_protection(
        source,
        protect_mapping,
        &completion_resumed,
        &stored_scratch,
    )?;
    let after_protect = select_user_mapping_continuation(
        source,
        span,
        scratch_after_protect,
        protect_continuation,
        "protect",
    )?;
    let [load_mapping, scratch_after_load @ ..] = after_protect else {
        return Err(user_context_diagnostic(
            source,
            span,
            "protected code mapping must have its entry trampoline verified before executable promotion",
        ));
    };
    let (verified_scratch, load_continuation) = parse_user_mapping_entry_trampoline(
        source,
        load_mapping,
        &completion_resumed,
        &protected_scratch,
        "verify",
    )?;
    let after_load = select_user_mapping_continuation(
        source,
        span,
        scratch_after_load,
        load_continuation,
        "load",
    )?;
    let [execute_mapping, scratch_after_execute @ ..] = after_load else {
        return Err(user_context_diagnostic(
            source,
            span,
            "entry-trampoline verification must be followed by execute promotion",
        ));
    };
    let (executable_scratch, execute_continuation) = parse_user_mapping_execute_promotion(
        source,
        execute_mapping,
        &completion_resumed,
        &verified_scratch,
    )?;
    let after_execute = select_user_mapping_continuation(
        source,
        span,
        scratch_after_execute,
        execute_continuation,
        "execute promotion",
    )?;
    let (data_subject, data_frames, data_success, _data_problem, data_failure, data_span) =
        result_sequence(source, after_execute, "data frame allocation")?;
    let data_request = parse_physical_frame_allocation(source, data_subject, &completion_resumed)?;
    if data_request
        != (CompilerPhysicalFrameRequest {
            frame_count: 1,
            alignment_frames: 1,
        })
    {
        return Err(user_context_diagnostic(
            source,
            data_subject.span(),
            "resident immutable data requires exactly one separately allocated frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: data_failure_message,
    } = action_disposition(source, data_failure, &completion_resumed)?
    else {
        return Err(user_context_diagnostic(
            source,
            data_failure.span(),
            "data frame allocation failure must preserve the allocator context and enter `fatal`",
        ));
    };
    let data_statements = action_block(source, data_success, "data frame allocation success")?;
    let [map_mapping_again, after_second_map @ ..] = data_statements else {
        return Err(user_context_diagnostic(
            source,
            data_span,
            "data frame allocation success must construct and revoke immutable data before executable code",
        ));
    };
    let (mapped_scratch_again, map_again_continuation) = parse_user_address_space_map(
        source,
        map_mapping_again,
        &completion_resumed,
        &user_space,
        &data_frames,
        "probe-data",
    )?;
    let after_second_map = select_user_mapping_continuation(
        source,
        span,
        after_second_map,
        map_again_continuation,
        "second map",
    )?;
    let [store_mapping_again, after_second_store @ ..] = after_second_map else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second user mapping publication must be followed by store, protect, load, and unmap",
        ));
    };
    let (stored_scratch_again, store_again_continuation) = parse_user_mapping_refinement(
        source,
        store_mapping_again,
        &completion_resumed,
        &mapped_scratch_again,
        "store",
    )?;
    let after_second_store = select_user_mapping_continuation(
        source,
        span,
        after_second_store,
        store_again_continuation,
        "second store",
    )?;
    let [protect_mapping_again, after_second_protect @ ..] = after_second_store else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second user mapping store must be followed by protect, load, and unmap",
        ));
    };
    let (protected_scratch_again, protect_again_continuation) = parse_user_mapping_protection(
        source,
        protect_mapping_again,
        &completion_resumed,
        &stored_scratch_again,
    )?;
    let after_second_protect = select_user_mapping_continuation(
        source,
        span,
        after_second_protect,
        protect_again_continuation,
        "second protect",
    )?;
    let [load_mapping_again, after_second_load @ ..] = after_second_protect else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second protected user mapping must be loaded before unmap",
        ));
    };
    let (verified_scratch_again, load_again_continuation) = parse_user_mapping_refinement(
        source,
        load_mapping_again,
        &completion_resumed,
        &protected_scratch_again,
        "load",
    )?;
    let after_second_load = select_user_mapping_continuation(
        source,
        span,
        after_second_load,
        load_again_continuation,
        "second load",
    )?;
    let [seal_mapping_again, after_second_seal @ ..] = after_second_load else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second user mapping load must be followed by writable-data promotion and unmap",
        ));
    };
    let (writable_data_scratch_again, seal_again_continuation) =
        parse_user_mapping_writable_data_promotion(
            source,
            seal_mapping_again,
            &completion_resumed,
            &verified_scratch_again,
        )?;
    let after_second_seal = select_user_mapping_continuation(
        source,
        span,
        after_second_seal,
        seal_again_continuation,
        "second writable-data promotion",
    )?;
    let (
        anonymous_subject,
        anonymous_frames,
        anonymous_success,
        _anonymous_problem,
        anonymous_failure,
        anonymous_span,
    ) = result_sequence(source, after_second_seal, "anonymous frame allocation")?;
    let anonymous_request =
        parse_physical_frame_allocation(source, anonymous_subject, &completion_resumed)?;
    if anonymous_request
        != (CompilerPhysicalFrameRequest {
            frame_count: 34,
            alignment_frames: 1,
        })
    {
        return Err(user_context_diagnostic(
            source,
            anonymous_subject.span(),
            "the bounded anonymous reservation requires exactly 35 frames aligned to one frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: anonymous_failure_message,
    } = action_disposition(source, anonymous_failure, &completion_resumed)?
    else {
        return Err(user_context_diagnostic(
            source,
            anonymous_failure.span(),
            "anonymous frame allocation failure must preserve the allocator context and enter `fatal`",
        ));
    };
    let anonymous_statements = action_block(
        source,
        anonymous_success,
        "anonymous frame allocation success",
    )?;
    let [reserve_anonymous, anonymous_continuation @ ..] = anonymous_statements else {
        return Err(user_context_diagnostic(
            source,
            anonymous_span,
            "anonymous frame allocation success must reserve its exact frame before user entry",
        ));
    };
    let (anonymous_page, reserve_continuation) = parse_user_address_space_reserve_anonymous(
        source,
        reserve_anonymous,
        &completion_resumed,
        &user_space,
        &anonymous_frames,
    )?;
    let after_anonymous_reserve = select_user_mapping_continuation(
        source,
        anonymous_span,
        anonymous_continuation,
        reserve_continuation,
        "anonymous reservation",
    )?;
    let [
        image_ready_marker_statement,
        boot_resource_marker_statement,
        prepare_user,
        after_prepare_user @ ..,
    ] = after_anonymous_reserve
    else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second writable-data user mapping must publish readiness before user preparation",
        ));
    };
    let image_ready_marker = analyze_operation(
        source,
        image_ready_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if image_ready_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_READY".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(image_ready_marker_statement),
            "resident user image requires its exact readiness marker before teardown",
        ));
    }
    let boot_resource_marker = analyze_operation(
        source,
        boot_resource_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if boot_resource_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_BOOT_RESOURCE_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(boot_resource_marker_statement),
            "initial user image requires its exact boot-resource marker before user preparation",
        ));
    }
    let (user_probe, probe_continuation) =
        parse_user_probe_prepare(source, prepare_user, &completion_resumed, &user_space)?;
    let user_continuation = match (after_prepare_user.is_empty(), probe_continuation.is_empty()) {
        (true, false) => probe_continuation,
        (false, true) => after_prepare_user,
        _ => {
            return Err(user_context_diagnostic(
                source,
                span,
                "prepared user address space must transfer exactly one continuation into its user probe",
            ));
        }
    };
    let [
        queue_enable_write_statement,
        queue_enable_read_statement,
        driver_ok_write_statement,
        driver_ok_read_statement,
        queue_activation_marker_statement,
        queue_notification_statement,
        queue_notification_marker_statement,
        completion_signal_statement,
        completion_signal_marker_statement,
        complete_control_statement,
        used_flags_statement,
        used_index_statement,
        used_identifier_statement,
        used_length_statement,
        used_ring_marker_statement,
        complete_payload_statement,
        payload_completion_marker_statement,
        entropy_sample_statement,
        entropy_sample_marker_statement,
        entropy_quality_statement,
        entropy_quality_marker_statement,
        entropy_pool_statement,
        entropy_pool_marker_statement,
        entropy_mix_statement,
        entropy_mix_marker_statement,
        entropy_credit_statement,
        entropy_credit_marker_statement,
        entropy_generator_statement,
        entropy_generator_marker_statement,
        entropy_extract_statement,
        entropy_extract_marker_statement,
        entropy_delivery_statement,
        auxv_random_marker_statement,
        entropy_getrandom_extract_statement,
        entropy_getrandom_reservation_statement,
        entropy_getrandom_binding_statement,
        entropy_getrandom_small_request_statement,
        entropy_getrandom_large_request_statement,
        enter_user,
        multipage_executable_marker_statement,
        user_getrandom_marker_statement,
        user_getrandom_repeated_marker_statement,
        user_getrandom_full_output_marker_statement,
        user_getrandom_small_request_marker_statement,
        user_getrandom_large_request_marker_statement,
        user_read_marker_statement,
        user_close_marker_statement,
        user_dup_marker_statement,
        user_fcntl_marker_statement,
        user_fstat_marker_statement,
        user_tcgets_marker_statement,
        user_winsize_marker_statement,
        user_lseek_marker_statement,
        user_copy_to_marker_statement,
        user_copy_from_marker_statement,
        user_marker_statement,
        entry_marker_statement,
        stack_marker_statement,
        auxv_marker_statement,
        auxv_entry_marker_statement,
        auxv_execfn_marker_statement,
        auxv_credentials_marker_statement,
        elf_marker_statement,
        external_elf_marker_statement,
        external_elf_copy_marker_statement,
        runtime_extent_marker_statement,
        runtime_entry_marker_statement,
        runtime_data_marker_statement,
        external_bss_marker_statement,
        data_marker_statement,
        writable_data_marker_statement,
        image_copy_to_marker_statement,
        image_copy_from_marker_statement,
        image_syscall_marker_statement,
        retained_marker_statement,
        release_anonymous,
        release_anonymous_frame,
        unmap_mapping_again,
        release_data,
        unmap_code,
        release_code,
        mapping_marker_statement,
        disjoint_marker_statement,
        complete_user,
        reclaim_user_space,
        release_user_frames,
        backing_marker_statement,
        disposition,
    ] = user_continuation
    else {
        return Err(user_context_diagnostic(
            source,
            span,
            "user probe must return with its image resident before ordered teardown, completion, reclaim, and disposition",
        ));
    };
    let user_completed =
        parse_user_context_enter(source, enter_user, &completion_resumed, &user_probe)?;
    let multipage_executable_marker = analyze_operation(
        source,
        multipage_executable_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if multipage_executable_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(multipage_executable_marker_statement),
            "returned user execution requires its exact multipage-executable marker",
        ));
    }
    let user_getrandom_marker = analyze_operation(
        source,
        user_getrandom_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_marker_statement),
            "returned user execution requires its exact bounded getrandom marker",
        ));
    }
    let user_getrandom_repeated_marker = analyze_operation(
        source,
        user_getrandom_repeated_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_repeated_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_repeated_marker_statement),
            "returned user execution requires its exact repeated getrandom marker",
        ));
    }
    let user_getrandom_full_output_marker = analyze_operation(
        source,
        user_getrandom_full_output_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_full_output_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_full_output_marker_statement),
            "returned user execution requires its exact full-output getrandom marker",
        ));
    }
    let user_getrandom_small_request_marker = analyze_operation(
        source,
        user_getrandom_small_request_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_small_request_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_small_request_marker_statement),
            "returned user execution requires its exact small-request getrandom marker",
        ));
    }
    let user_getrandom_large_request_marker = analyze_operation(
        source,
        user_getrandom_large_request_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_large_request_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_large_request_marker_statement),
            "returned user execution requires its exact large-request getrandom marker",
        ));
    }
    let user_read_marker = analyze_operation(
        source,
        user_read_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_read_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_READ_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_read_marker_statement),
            "returned user execution requires its exact constructed read marker",
        ));
    }
    let user_close_marker = analyze_operation(
        source,
        user_close_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_close_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_CLOSE_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_close_marker_statement),
            "returned user execution requires its exact descriptor-close marker",
        ));
    }
    let user_dup_marker = analyze_operation(
        source,
        user_dup_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_dup_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_DUP_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_dup_marker_statement),
            "returned user execution requires its exact descriptor-duplication marker",
        ));
    }
    let user_fcntl_marker = analyze_operation(
        source,
        user_fcntl_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_fcntl_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_FCNTL_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_fcntl_marker_statement),
            "returned user execution requires its exact descriptor-metadata marker",
        ));
    }
    let user_fstat_marker = analyze_operation(
        source,
        user_fstat_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_fstat_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_FSTAT_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_fstat_marker_statement),
            "returned user execution requires its exact descriptor-stat marker",
        ));
    }
    let user_tcgets_marker = analyze_operation(
        source,
        user_tcgets_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_tcgets_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_TCGETS_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_tcgets_marker_statement),
            "returned user execution requires its exact terminal-attributes marker",
        ));
    }
    let user_winsize_marker = analyze_operation(
        source,
        user_winsize_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_winsize_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_WINSIZE_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_winsize_marker_statement),
            "returned user execution requires its exact console-geometry marker",
        ));
    }
    let user_lseek_marker = analyze_operation(
        source,
        user_lseek_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_lseek_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_LSEEK_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_lseek_marker_statement),
            "returned user execution requires its exact console-seek marker",
        ));
    }
    let user_copy_to_marker = analyze_operation(
        source,
        user_copy_to_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_copy_to_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_COPY_TO_OK".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(user_copy_to_marker_statement),
            "user copy_to_user recovery requires its exact marker",
        ));
    }
    let user_copy_from_marker = analyze_operation(
        source,
        user_copy_from_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_copy_from_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_COPY_FROM_OK".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(user_copy_from_marker_statement),
            "user copy_from_user recovery requires its exact marker",
        ));
    }
    let user_marker = analyze_operation(
        source,
        user_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_SYSCALL_OK".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(user_marker_statement),
            "user syscall round trip requires its exact marker",
        ));
    }
    let entry_marker = analyze_operation(
        source,
        entry_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entry_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entry_marker_statement),
            "returned user execution requires its exact constructed-image entry marker",
        ));
    }
    let stack_marker = analyze_operation(
        source,
        stack_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if stack_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_STACK_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(stack_marker_statement),
            "returned user execution requires its exact constructed-image initial-stack marker",
        ));
    }
    let auxv_marker = analyze_operation(
        source,
        auxv_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_marker_statement),
            "returned user execution requires its exact constructed-image auxiliary-vector marker",
        ));
    }
    let auxv_entry_marker = analyze_operation(
        source,
        auxv_entry_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_entry_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_entry_marker_statement),
            "returned user execution requires its exact constructed-image auxiliary-vector entry marker",
        ));
    }
    let auxv_execfn_marker = analyze_operation(
        source,
        auxv_execfn_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_execfn_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_execfn_marker_statement),
            "returned user execution requires its exact constructed-image auxiliary-vector executable-name marker",
        ));
    }
    let auxv_credentials_marker = analyze_operation(
        source,
        auxv_credentials_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_credentials_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_credentials_marker_statement),
            "returned user execution requires its exact constructed-image auxiliary-vector credentials marker",
        ));
    }
    let elf_marker = analyze_operation(
        source,
        elf_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if elf_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_ELF_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(elf_marker_statement),
            "returned user execution requires its exact constructed-image ELF identity marker",
        ));
    }
    let external_elf_marker = analyze_operation(
        source,
        external_elf_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if external_elf_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(external_elf_marker_statement),
            "returned user execution requires its exact external ELF-load marker",
        ));
    }
    let external_elf_copy_marker = analyze_operation(
        source,
        external_elf_copy_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if external_elf_copy_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(external_elf_copy_marker_statement),
            "returned user execution requires its exact external ELF-copy marker",
        ));
    }
    let runtime_extent_marker = analyze_operation(
        source,
        runtime_extent_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if runtime_extent_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(runtime_extent_marker_statement),
            "returned user execution requires its exact runtime-derived executable-extent marker",
        ));
    }
    let runtime_entry_marker = analyze_operation(
        source,
        runtime_entry_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if runtime_entry_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(runtime_entry_marker_statement),
            "returned user execution requires its exact runtime-derived ELF-entry marker",
        ));
    }
    let runtime_data_marker = analyze_operation(
        source,
        runtime_data_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if runtime_data_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(runtime_data_marker_statement),
            "returned user execution requires its exact runtime-derived data/BSS-extent marker",
        ));
    }
    let external_bss_marker = analyze_operation(
        source,
        external_bss_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if external_bss_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_BSS_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(external_bss_marker_statement),
            "returned user execution requires its exact external ELF data/BSS marker",
        ));
    }
    let data_marker = analyze_operation(
        source,
        data_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if data_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_DATA_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(data_marker_statement),
            "returned user execution requires its exact constructed-image data marker",
        ));
    }
    let writable_data_marker = analyze_operation(
        source,
        writable_data_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if writable_data_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(writable_data_marker_statement),
            "returned user execution requires its exact writable-data marker",
        ));
    }
    let image_copy_to_marker = analyze_operation(
        source,
        image_copy_to_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if image_copy_to_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(image_copy_to_marker_statement),
            "returned user execution requires its exact constructed-image copy-to-user marker",
        ));
    }
    let image_copy_from_marker = analyze_operation(
        source,
        image_copy_from_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if image_copy_from_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(image_copy_from_marker_statement),
            "returned user execution requires its exact constructed-image copy-from-user marker",
        ));
    }
    let image_syscall_marker = analyze_operation(
        source,
        image_syscall_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if image_syscall_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(image_syscall_marker_statement),
            "returned user execution requires its exact constructed-image syscall marker",
        ));
    }
    let retained_marker = analyze_operation(
        source,
        retained_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if retained_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_RETAINED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(retained_marker_statement),
            "returned user execution requires its exact retained-image marker before teardown",
        ));
    }
    let (returned_anonymous, release_anonymous_continuation) =
        parse_user_address_space_release_anonymous(
            source,
            release_anonymous,
            &completion_resumed,
            &user_space,
            &anonymous_page,
        )?;
    if !release_anonymous_continuation.is_empty() {
        return Err(user_context_diagnostic(
            source,
            statement_span(release_anonymous),
            "anonymous release continuation must remain after returned user execution",
        ));
    }
    parse_physical_frame_release(
        source,
        release_anonymous_frame,
        &completion_resumed,
        &returned_anonymous,
    )?;
    let (returned_scratch_again, unmap_again_continuation) = parse_user_address_space_unmap(
        source,
        unmap_mapping_again,
        &completion_resumed,
        &user_space,
        &writable_data_scratch_again,
    )?;
    if !unmap_again_continuation.is_empty() {
        return Err(user_context_diagnostic(
            source,
            statement_span(unmap_mapping_again),
            "writable-data unmap continuation must remain after returned user execution",
        ));
    }
    parse_physical_frame_release(
        source,
        release_data,
        &completion_resumed,
        &returned_scratch_again,
    )?;
    let (returned_code, code_unmap_continuation) = parse_user_address_space_unmap(
        source,
        unmap_code,
        &completion_resumed,
        &user_space,
        &executable_scratch,
    )?;
    if !code_unmap_continuation.is_empty() {
        return Err(user_context_diagnostic(
            source,
            statement_span(unmap_code),
            "executable-code unmap continuation must remain after returned user execution",
        ));
    }
    parse_physical_frame_release(source, release_code, &completion_resumed, &returned_code)?;
    let mapping_marker = analyze_operation(
        source,
        mapping_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if mapping_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_MAPPING_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(mapping_marker_statement),
            "user mapping round trip requires its exact marker",
        ));
    }
    let disjoint_marker = analyze_operation(
        source,
        disjoint_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if disjoint_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_FRAME_DISJOINT_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(disjoint_marker_statement),
            "disjoint frame release requires its exact marker",
        ));
    }
    let revoked_space =
        parse_user_execution_complete(source, complete_user, &completion_resumed, &user_completed)?;
    let returned_frames = parse_user_address_space_reclaim(
        source,
        reclaim_user_space,
        &completion_resumed,
        &revoked_space,
    )?;
    parse_physical_frame_release(
        source,
        release_user_frames,
        &completion_resumed,
        &returned_frames,
    )?;
    let backing_marker = analyze_operation(
        source,
        backing_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if backing_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_BACKING_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(backing_marker_statement),
            "user backing release requires its exact marker",
        ));
    }
    let mut queue_activation_operations = Vec::new();
    for (statement, request) in [
        queue_enable_write_statement,
        queue_enable_read_statement,
        driver_ok_write_statement,
        driver_ok_read_statement,
    ]
    .into_iter()
    .zip(CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION)
    {
        let operation = analyze_operation(
            source,
            statement,
            CompilerSystemsContextKind::Bootstrap,
            &completion_resumed,
        )?;
        if operation != (CompilerSystemsOperation::AccessPciRegionRegister { request }) {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user teardown must be followed by the exact virtio queue-activation sequence",
            ));
        }
        queue_activation_operations.push(operation);
    }
    let queue_activation_marker = analyze_operation(
        source,
        queue_activation_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if queue_activation_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(queue_activation_marker_statement),
            "virtio queue activation requires its exact completion marker",
        ));
    }
    let queue_notification = analyze_operation(
        source,
        queue_notification_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if queue_notification
        != (CompilerSystemsOperation::AccessPciRegionRegister {
            request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION[0],
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(queue_notification_statement),
            "verified virtio queue activation must be followed by the exact queue-zero notification",
        ));
    }
    let queue_notification_marker = analyze_operation(
        source,
        queue_notification_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if queue_notification_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(queue_notification_marker_statement),
            "virtio queue notification requires its exact completion marker",
        ));
    }
    let completion_signal = analyze_operation(
        source,
        completion_signal_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if completion_signal
        != (CompilerSystemsOperation::AccessPciRegionRegister {
            request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL[0],
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(completion_signal_statement),
            "virtio queue notification must be followed by the exact completion-signal observation",
        ));
    }
    let completion_signal_marker = analyze_operation(
        source,
        completion_signal_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if completion_signal_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(completion_signal_marker_statement),
            "virtio completion-signal observation requires its exact marker",
        ));
    }
    let complete_control = analyze_operation(
        source,
        complete_control_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if complete_control
        != (CompilerSystemsOperation::CompleteDmaBufferForCpu {
            request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(complete_control_statement),
            "virtio used-ring validation requires exact control-buffer CPU completion",
        ));
    }
    let used_ring_statements = [
        used_flags_statement,
        used_index_statement,
        used_identifier_statement,
        used_length_statement,
    ];
    let mut used_ring_operations = Vec::with_capacity(4);
    for (statement, request) in used_ring_statements
        .iter()
        .zip(CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING)
    {
        let operation = analyze_operation(
            source,
            statement,
            CompilerSystemsContextKind::Bootstrap,
            &completion_resumed,
        )?;
        if operation != (CompilerSystemsOperation::LoadDmaBufferIntegerEquals { request }) {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "virtio used-ring validation requires the exact ordered control-field reads",
            ));
        }
        used_ring_operations.push(operation);
    }
    let used_ring_marker = analyze_operation(
        source,
        used_ring_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if used_ring_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_USED_RING_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(used_ring_marker_statement),
            "virtio used-ring validation requires its exact marker",
        ));
    }
    let complete_payload = analyze_operation(
        source,
        complete_payload_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if complete_payload
        != (CompilerSystemsOperation::CompleteDmaBufferForCpu {
            request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(complete_payload_statement),
            "virtio used-ring validation must be followed by exact payload-buffer CPU completion",
        ));
    }
    let payload_completion_marker = analyze_operation(
        source,
        payload_completion_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if payload_completion_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(payload_completion_marker_statement),
            "virtio payload completion requires its exact marker",
        ));
    }
    let entropy_sample = analyze_operation(
        source,
        entropy_sample_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_sample
        != (CompilerSystemsOperation::CaptureEntropySample {
            request: CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_sample_statement),
            "virtio payload completion must be followed by exact uncredited entropy-sample capture",
        ));
    }
    let entropy_sample_marker = analyze_operation(
        source,
        entropy_sample_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_sample_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_sample_marker_statement),
            "entropy-sample capture requires its exact marker",
        ));
    }
    let entropy_quality = analyze_operation(
        source,
        entropy_quality_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_quality
        != (CompilerSystemsOperation::AssessEntropySampleQuality {
            request: CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_quality_statement),
            "entropy-sample capture must be followed by exact Q35 quality assessment",
        ));
    }
    let entropy_quality_marker = analyze_operation(
        source,
        entropy_quality_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_quality_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_quality_marker_statement),
            "entropy-sample quality assessment requires its exact marker",
        ));
    }
    let entropy_pool = analyze_operation(
        source,
        entropy_pool_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_pool
        != (CompilerSystemsOperation::CreateEntropyPool {
            request: CompilerEntropyPoolRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_pool_statement),
            "entropy-sample assessment must be followed by exact empty-pool creation",
        ));
    }
    let entropy_pool_marker = analyze_operation(
        source,
        entropy_pool_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_pool_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_POOL_CREATED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_pool_marker_statement),
            "entropy-pool creation requires its exact marker",
        ));
    }
    let entropy_mix = analyze_operation(
        source,
        entropy_mix_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_mix
        != (CompilerSystemsOperation::MixEntropyPoolSample {
            request: CompilerEntropyPoolMixRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_mix_statement),
            "empty-pool creation must be followed by the exact first sample mix",
        ));
    }
    let entropy_mix_marker = analyze_operation(
        source,
        entropy_mix_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_mix_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_POOL_MIXED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_mix_marker_statement),
            "entropy-pool mixing requires its exact marker",
        ));
    }
    let entropy_credit = analyze_operation(
        source,
        entropy_credit_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_credit
        != (CompilerSystemsOperation::CreditEntropyPoolContribution {
            request: CompilerEntropyPoolCreditRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_credit_statement),
            "the first sample mix must be followed by its exact contribution-credit transfer",
        ));
    }
    let entropy_credit_marker = analyze_operation(
        source,
        entropy_credit_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_credit_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_POOL_CREDITED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_credit_marker_statement),
            "entropy-credit transfer requires its exact marker",
        ));
    }
    let entropy_generator = analyze_operation(
        source,
        entropy_generator_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_generator
        != (CompilerSystemsOperation::InitializeEntropyGenerator {
            request: CompilerEntropyGeneratorRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_generator_statement),
            "entropy credit must be followed by exact generation-one CRNG initialization",
        ));
    }
    let entropy_generator_marker = analyze_operation(
        source,
        entropy_generator_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_generator_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_GENERATOR_READY".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_generator_marker_statement),
            "entropy-generator initialization requires its exact marker",
        ));
    }
    let entropy_extract = analyze_operation(
        source,
        entropy_extract_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_extract
        != (CompilerSystemsOperation::ExtractEntropyGenerator {
            request: CompilerEntropyGeneratorExtractRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_extract_statement),
            "ready generation one must be followed by the exact first CRNG extraction",
        ));
    }
    let entropy_extract_marker = analyze_operation(
        source,
        entropy_extract_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_extract_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_OUTPUT_READY".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_extract_marker_statement),
            "entropy-generator extraction requires its exact marker",
        ));
    }
    let entropy_delivery = analyze_operation(
        source,
        entropy_delivery_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_delivery
        != (CompilerSystemsOperation::DeliverInitialAuxvRandom {
            request: CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_delivery_statement),
            "first CRNG extraction must be followed by exact affine initial AT_RANDOM delivery",
        ));
    }
    let auxv_random_marker = analyze_operation(
        source,
        auxv_random_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_random_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_RANDOM_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_random_marker_statement),
            "initial AT_RANDOM delivery requires its exact completion marker before user entry",
        ));
    }
    let entropy_getrandom_extract = analyze_operation(
        source,
        entropy_getrandom_extract_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_extract
        != (CompilerSystemsOperation::ExtractEntropyGenerator {
            request: CompilerEntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_extract_statement),
            "initial AT_RANDOM delivery must be followed by the exact second CRNG extraction",
        ));
    }
    let entropy_getrandom_reservation = analyze_operation(
        source,
        entropy_getrandom_reservation_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_reservation
        != (CompilerSystemsOperation::ReserveInitialUserRandom {
            request: CompilerEntropyOutputReservationRequest::INITIAL_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_reservation_statement),
            "the second CRNG output must be reserved by the exact initial getrandom capability before user entry",
        ));
    }
    let entropy_getrandom_binding = analyze_operation(
        source,
        entropy_getrandom_binding_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_binding
        != (CompilerSystemsOperation::BindReusableUserRandom {
            request: CompilerEntropyUserRandomServiceRequest::INITIAL_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_binding_statement),
            "the initial reservation and generation-three generator must bind the exact reusable getrandom service before user entry",
        ));
    }
    let entropy_getrandom_small_request = analyze_operation(
        source,
        entropy_getrandom_small_request_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_small_request
        != (CompilerSystemsOperation::ExpandSmallUserRandomRequests {
            request: CompilerEntropyUserRandomSmallRequest::INITIAL_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_small_request_statement),
            "the reusable getrandom service must receive its exact bounded small-request refinement before user entry",
        ));
    }
    let entropy_getrandom_large_request = analyze_operation(
        source,
        entropy_getrandom_large_request_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_large_request
        != (CompilerSystemsOperation::ExpandLargeUserRandomRequests {
            request: CompilerEntropyUserRandomSmallRequest::LARGE_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_large_request_statement),
            "the small-request getrandom service must receive its exact bounded large-request refinement before user entry",
        ));
    }
    let disposition = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    let mut operations = vec![
        CompilerSystemsOperation::BootstrapAllocate { request },
        CompilerSystemsOperation::CreateKernelCompletion,
        CompilerSystemsOperation::CreateKernelContext,
        CompilerSystemsOperation::BootstrapAllocate {
            request: completion_request,
        },
        CompilerSystemsOperation::CreateKernelContext,
        CompilerSystemsOperation::BootstrapAllocate {
            request: terminal_request,
        },
        CompilerSystemsOperation::CreateKernelContext,
        CompilerSystemsOperation::CreateRunnableQueue { capacity: 3 },
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContextUntilDeadline,
        CompilerSystemsOperation::TakePreemptedKernelContext,
        preemption_marker,
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContextUntilCompletion,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContext,
        terminal_marker,
        CompilerSystemsOperation::ReclaimKernelContext,
        CompilerSystemsOperation::SignalBlockedKernelCompletion,
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContext,
        cooperative_marker,
        CompilerSystemsOperation::ReclaimKernelContext,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContext,
        completion_marker,
        CompilerSystemsOperation::ReclaimKernelContext,
        CompilerSystemsOperation::ConsumeKernelCompletion,
        CompilerSystemsOperation::ConsumeEmptyRunnableQueue,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: backing_request,
            failure_message: backing_failure,
        },
        CompilerSystemsOperation::PrepareUserAddressSpace,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: scratch_request,
            failure_message: scratch_failure_message,
        },
        CompilerSystemsOperation::MapUserAddressSpace,
        CompilerSystemsOperation::InstallUserMappingEntryTrampoline,
        CompilerSystemsOperation::ProtectUserMappingReadOnly,
        CompilerSystemsOperation::VerifyUserMappingEntryTrampoline,
        CompilerSystemsOperation::PermitUserMappingExecute,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: data_request,
            failure_message: data_failure_message,
        },
        CompilerSystemsOperation::MapUserAddressSpace,
        CompilerSystemsOperation::StoreUserMappingByte,
        CompilerSystemsOperation::ProtectUserMappingReadOnly,
        CompilerSystemsOperation::LoadUserMappingByte,
        CompilerSystemsOperation::PermitUserMappingWritableData,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: anonymous_request,
            failure_message: anonymous_failure_message,
        },
        CompilerSystemsOperation::ReserveAnonymousUserMapping,
        image_ready_marker,
        boot_resource_marker,
        CompilerSystemsOperation::PrepareUserProbe {
            process_identity: 1,
        },
    ];
    operations.extend(queue_activation_operations);
    operations.push(queue_activation_marker);
    operations.push(queue_notification);
    operations.push(queue_notification_marker);
    operations.push(completion_signal);
    operations.push(completion_signal_marker);
    operations.push(complete_control);
    operations.extend(used_ring_operations);
    operations.push(used_ring_marker);
    operations.push(complete_payload);
    operations.push(payload_completion_marker);
    operations.push(entropy_sample);
    operations.push(entropy_sample_marker);
    operations.push(entropy_quality);
    operations.push(entropy_quality_marker);
    operations.push(entropy_pool);
    operations.push(entropy_pool_marker);
    operations.push(entropy_mix);
    operations.push(entropy_mix_marker);
    operations.push(entropy_credit);
    operations.push(entropy_credit_marker);
    operations.push(entropy_generator);
    operations.push(entropy_generator_marker);
    operations.push(entropy_extract);
    operations.push(entropy_extract_marker);
    operations.push(entropy_delivery);
    operations.push(auxv_random_marker);
    operations.push(entropy_getrandom_extract);
    operations.push(entropy_getrandom_reservation);
    operations.push(entropy_getrandom_binding);
    operations.push(entropy_getrandom_small_request);
    operations.push(entropy_getrandom_large_request);
    operations.extend([
        CompilerSystemsOperation::EnterUserContext,
        multipage_executable_marker,
        user_getrandom_marker,
        user_getrandom_repeated_marker,
        user_getrandom_full_output_marker,
        user_getrandom_small_request_marker,
        user_getrandom_large_request_marker,
        user_read_marker,
        user_close_marker,
        user_dup_marker,
        user_fcntl_marker,
        user_fstat_marker,
        user_tcgets_marker,
        user_winsize_marker,
        user_lseek_marker,
        user_copy_to_marker,
        user_copy_from_marker,
        user_marker,
        entry_marker,
        stack_marker,
        auxv_marker,
        auxv_entry_marker,
        auxv_execfn_marker,
        auxv_credentials_marker,
        elf_marker,
        external_elf_marker,
        external_elf_copy_marker,
        runtime_extent_marker,
        runtime_entry_marker,
        runtime_data_marker,
        external_bss_marker,
        data_marker,
        writable_data_marker,
        image_copy_to_marker,
        image_copy_from_marker,
        image_syscall_marker,
        retained_marker,
        CompilerSystemsOperation::ReleaseAnonymousUserMapping,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        CompilerSystemsOperation::UnmapUserAddressSpace,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        CompilerSystemsOperation::UnmapUserAddressSpace,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        mapping_marker,
        disjoint_marker,
        CompilerSystemsOperation::CompleteUserExecution,
        CompilerSystemsOperation::ReclaimUserAddressSpace,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        backing_marker,
    ]);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition,
    })
}

fn parse_kernel_context_create(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    stack_name: &str,
    entry_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "kernel-context creation must bind `SuspendedKernelContext InitialProcessor`",
        ));
    };
    let [
        context,
        kernel,
        context_word,
        create,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context creation requires named `stack` and `entry` arguments",
        ));
    };
    if source.slice(*classifier) != "SuspendedKernelContext InitialProcessor"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, create, "create")
        || !kernel_context_create_fields(source, fields, stack_name, entry_name)?
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            format!(
                "kernel-context creation requires the live context, allocated stack, and typed `{entry_name}` entry"
            ),
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_user_address_space_prepare<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    backing_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let Statement::Implementation {
        name,
        classifier: Expression::Application { items, span },
        declarations,
        ..
    } = statement
    else {
        return Err(user_context_diagnostic(
            source,
            statement_span(statement),
            "user-address-space preparation must bind one opaque affine space",
        ));
    };
    let [
        receiver,
        user,
        address,
        space,
        prepare,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user-address-space preparation requires named `backing` and `layout` parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, prepare, "prepare")
        || fields.len() != 2
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user address space requires the exact three-frame backing and sealed initial-probe layout",
        ));
    }
    let mut backing = None;
    let mut layout = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user-address-space preparation parameters must be named",
            ));
        };
        let Expression::Identifier(value) = field.value else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user-address-space preparation values must use the live backing and sealed layout identity",
            ));
        };
        match source.slice(label) {
            "backing" => backing = Some(source.slice(value)),
            "layout" => layout = Some(source.slice(value)),
            _ => {
                return Err(user_context_diagnostic(
                    source,
                    label,
                    "unknown user-address-space preparation parameter",
                ));
            }
        }
    }
    if backing != Some(backing_name) || layout != Some("initial-probe") {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user address space requires the exact three-frame backing and sealed initial-probe layout",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_address_space_map<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
    backing_name: &str,
    layout_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping publication must bind one fresh affine mapping",
            ));
        }
    };
    let [
        receiver,
        user,
        address,
        space,
        map,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping publication requires named address-space, backing, and layout parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, map, "map")
        || fields.len() != 3
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user mapping requires the exact prepared space, disjoint backing, and probe-guard layout",
        ));
    }
    let mut address_space = None;
    let mut backing = None;
    let mut layout = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user mapping publication parameters must be named",
            ));
        };
        let Expression::Identifier(value) = field.value else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user mapping publication values must use live affine identities",
            ));
        };
        match source.slice(label) {
            "address-space" => address_space = Some(source.slice(value)),
            "backing" => backing = Some(source.slice(value)),
            "layout" => layout = Some(source.slice(value)),
            _ => {
                return Err(user_context_diagnostic(
                    source,
                    label,
                    "unknown user mapping publication parameter",
                ));
            }
        }
    }
    if address_space != Some(address_space_name)
        || backing != Some(backing_name)
        || layout != Some(layout_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user mapping requires the exact prepared space, disjoint backing, and probe-guard layout",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_mapping_refinement<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
    operation: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping access must bind its refined affine mapping",
            ));
        }
    };
    let [receiver, user, mapping, access, byte, live_mapping] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping access requires the exact live mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, access, operation)
        || !identifier_is(source, byte, "byte")
        || !identifier_is(source, live_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            format!("user mapping {operation} requires the exact preceding mapping refinement"),
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_mapping_entry_trampoline<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
    operation: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user entry-trampoline operation must bind its affine mapping refinement",
            ));
        }
    };
    let [
        receiver,
        user,
        mapping,
        action,
        entry,
        trampoline,
        live_mapping,
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user entry-trampoline operation requires the exact preceding mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, action, operation)
        || !identifier_is(source, entry, "entry")
        || !identifier_is(source, trampoline, "trampoline")
        || !identifier_is(source, live_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            format!(
                "user mapping {operation} entry trampoline requires the exact preceding mapping refinement"
            ),
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_mapping_protection<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping protection must bind its read-only affine refinement",
            ));
        }
    };
    let [receiver, user, mapping, protect, read, only, stored_mapping] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping protection requires the exact stored mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, protect, "protect")
        || !identifier_is(source, read, "read")
        || !identifier_is(source, only, "only")
        || !identifier_is(source, stored_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping protect read only requires the exact stored mapping refinement",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_mapping_execute_promotion<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping execute promotion must bind its affine refinement",
            ));
        }
    };
    let [receiver, user, mapping, permit, execute, verified_mapping] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping execute promotion requires the exact verified mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, permit, "permit")
        || !identifier_is(source, execute, "execute")
        || !identifier_is(source, verified_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping permit execute requires the exact verified mapping refinement",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_mapping_writable_data_promotion<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping writable-data promotion must bind its affine refinement",
            ));
        }
    };
    let [
        receiver,
        user,
        mapping,
        permit,
        writable,
        data,
        verified_mapping,
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping writable-data promotion requires the exact verified mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, permit, "permit")
        || !identifier_is(source, writable, "writable")
        || !identifier_is(source, data, "data")
        || !identifier_is(source, verified_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping permit writable data requires the second verified refinement",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_address_space_reserve_anonymous<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
    backing_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "anonymous user-page reservation must bind its affine reservation",
            ));
        }
    };
    let [
        receiver,
        user,
        address,
        space,
        reserve,
        anonymous,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "anonymous user-page reservation requires named address-space, backing, and layout parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, reserve, "reserve")
        || !identifier_is(source, anonymous, "anonymous")
        || fields.len() != 3
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the bounded anonymous reservation requires its exact prepared space, frame, and layout",
        ));
    }
    let mut address_space = false;
    let mut backing = false;
    let mut layout = false;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "anonymous reservation parameters must be named",
            ));
        };
        match source.slice(label) {
            "address-space" => {
                address_space = identifier_is(source, &field.value, address_space_name)
            }
            "backing" => backing = identifier_is(source, &field.value, backing_name),
            "layout" => {
                layout = identifier_is(source, &field.value, "post-random-anonymous-growth-extent")
            }
            _ => {}
        }
    }
    if !address_space || !backing || !layout {
        return Err(user_context_diagnostic(
            source,
            *span,
            "anonymous reservation must consume the exact extent into `post-random-anonymous-growth-extent`",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_address_space_release_anonymous<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
    reservation_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "anonymous user-page release must bind its returned affine frame",
            ));
        }
    };
    let [
        receiver,
        user,
        address,
        space,
        release,
        anonymous,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "anonymous user-page release requires named address-space and reservation parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, release, "release")
        || !identifier_is(source, anonymous, "anonymous")
        || fields.len() != 2
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the bounded anonymous release requires its exact prepared space and reservation",
        ));
    }
    let mut address_space = false;
    let mut reservation = false;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "anonymous release parameters must be named",
            ));
        };
        match source.slice(label) {
            "address-space" => {
                address_space = identifier_is(source, &field.value, address_space_name)
            }
            "reservation" => reservation = identifier_is(source, &field.value, reservation_name),
            _ => {}
        }
    }
    if !address_space || !reservation {
        return Err(user_context_diagnostic(
            source,
            *span,
            "anonymous release must consume the exact live reservation",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_address_space_unmap<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
    mapping_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping revocation must bind the returned affine extent",
            ));
        }
    };
    let [
        receiver,
        user,
        address,
        space,
        unmap,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping revocation requires named address-space and mapping parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, unmap, "unmap")
        || fields.len() != 2
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user unmap requires the exact prepared space and verified mapping",
        ));
    }
    let mut address_space = None;
    let mut mapping_identity = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user mapping revocation parameters must be named",
            ));
        };
        let Expression::Identifier(value) = field.value else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user mapping revocation values must use live affine identities",
            ));
        };
        match source.slice(label) {
            "address-space" => address_space = Some(source.slice(value)),
            "mapping" => mapping_identity = Some(source.slice(value)),
            _ => {
                return Err(user_context_diagnostic(
                    source,
                    label,
                    "unknown user mapping revocation parameter",
                ));
            }
        }
    }
    if address_space != Some(address_space_name) || mapping_identity != Some(mapping_name) {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user unmap requires the exact prepared space and verified mapping",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn select_user_mapping_continuation<'a>(
    source: &SourceText,
    span: Span,
    following: &'a [Statement],
    nested: &'a [Statement],
    operation: &str,
) -> Result<&'a [Statement], Diagnostic> {
    match (following.is_empty(), nested.is_empty()) {
        (true, false) => Ok(nested),
        (false, true) => Ok(following),
        _ => Err(user_context_diagnostic(
            source,
            span,
            format!("user mapping {operation} must transfer exactly one continuation"),
        )),
    }
}

fn parse_user_probe_prepare<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user probe preparation must bind one classified suspended user context",
            ));
        }
    };
    let [
        receiver,
        user,
        probe,
        prepare,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user probe preparation must use the sealed process-identity request",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, probe, "probe")
        || !identifier_is(source, prepare, "prepare")
        || fields.len() != 2
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial suspended user probe requires process identity one and exact processor, address-space, thread, and ABI classifiers",
        ));
    }
    let mut process_identity = None;
    let mut address_space = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user probe preparation parameters must be named",
            ));
        };
        match source.slice(label) {
            "process-identity" => {
                process_identity = Some(parse_storage_natural(source, &field.value)?);
            }
            "address-space" => {
                let Expression::Identifier(name) = field.value else {
                    return Err(user_context_diagnostic(
                        source,
                        field.value.span(),
                        "user probe address space must be the live opaque prepared space",
                    ));
                };
                address_space = Some(source.slice(name));
            }
            _ => {
                return Err(user_context_diagnostic(
                    source,
                    label,
                    "unknown user probe preparation parameter",
                ));
            }
        }
    }
    if process_identity != Some(1) || address_space != Some(address_space_name) {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial suspended user probe requires process identity one and its exact prepared address space",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_context_enter(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    probe_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(user_context_diagnostic(
            source,
            statement_span(statement),
            "user entry must bind one classified completed user execution",
        ));
    };
    let [receiver, user, context_word, enter, probe] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user entry must be `context user context enter probe`",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, enter, "enter")
        || !identifier_is(source, probe, probe_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user entry must consume the exact suspended probe and preserve all identities",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_user_execution_complete(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    completed_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(user_context_diagnostic(
            source,
            statement_span(statement),
            "user execution completion must consume the terminal result",
        ));
    };
    let [receiver, user, execution, complete, completed] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user completion must be `context user execution complete completed`",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, execution, "execution")
        || !identifier_is(source, complete, "complete")
        || !identifier_is(source, completed, completed_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user completion must consume the exact terminal execution result",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_user_address_space_reclaim(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    revoked_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(user_context_diagnostic(
            source,
            statement_span(statement),
            "user-address-space reclaim must bind the returned affine frame extent",
        ));
    };
    let [receiver, user, address, space, reclaim, revoked] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user-address-space reclaim must consume one revoked space",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, reclaim, "reclaim")
        || !identifier_is(source, revoked, revoked_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user-address-space reclaim must consume the exact revoked space",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_completion_create(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "kernel-completion creation must bind its affine one-shot state",
        ));
    };
    let [context, kernel, completion, create] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-completion creation requires its live dispatcher",
        ));
    };
    if source.slice(*classifier) != "KernelCompletion InitialProcessor"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, completion, "completion")
        || !identifier_is(source, create, "create")
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-completion creation requires the exact affine classifier and live dispatcher",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_context_transfer_until_completion(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    worker_name: &str,
    completion_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "completion-bound transfer must bind its exact blocked outcome",
        ));
    };
    let [
        context,
        kernel,
        context_word,
        transfer,
        worker,
        until,
        completion,
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion-bound transfer requires a selected worker and one-shot completion",
        ));
    };
    if source.slice(*classifier) != "BlockedKernelContextTransfer InitialProcessor"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, transfer, "transfer")
        || !identifier_is(source, worker, worker_name)
        || !identifier_is(source, until, "until")
        || !identifier_is(source, completion, completion_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion-bound transfer requires the exact dispatcher, selected worker, and completion identity",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_signal_blocked_kernel_completion(
    source: &SourceText,
    statement: &Statement,
    blocked_name: &str,
    completion_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "completion signal must bind the exact awakened suspended context",
        ));
    };
    let [blocked, kernel, completion, signal, completion_value] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion signal must consume the blocked outcome and matching signal endpoint",
        ));
    };
    if source.slice(*classifier) != "SuspendedKernelContext InitialProcessor"
        || !identifier_is(source, blocked, blocked_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, completion, "completion")
        || !identifier_is(source, signal, "signal")
        || !identifier_is(source, completion_value, completion_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion signal requires the matching blocked context and one-shot completion",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_completion_consume(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    completion_name: &str,
) -> Result<(), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "completed one-shot state must be consumed before disposition",
        ));
    };
    let [context, kernel, completion, consume, completion_value] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion consumption requires the live dispatcher and completed state",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, completion, "completion")
        || !identifier_is(source, consume, "consume")
        || !identifier_is(source, completion_value, completion_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion consumption requires the exact completed one-shot identity",
        ));
    }
    Ok(())
}

fn parse_runnable_queue_create(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "runnable-queue creation must bind its exact bounded affine classifier",
        ));
    };
    let [
        context,
        kernel,
        runnable,
        queue,
        create,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable-queue creation requires one named capacity",
        ));
    };
    let [field] = fields.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "the initial runnable queue requires exactly `capacity is 3`",
        ));
    };
    let capacity_is_three = field
        .label
        .is_some_and(|label| source.slice(label) == "capacity")
        && matches!(&field.value, Expression::Integer(value) if source.slice(*value) == "3");
    if source.slice(*classifier) != "BoundedKernelRunnableQueue InitialProcessor"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, runnable, "runnable")
        || !identifier_is(source, queue, "queue")
        || !identifier_is(source, create, "create")
        || !capacity_is_three
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "the initial runnable queue requires its live context, capacity three, and exact bounded classifier",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_runnable_queue_enqueue(
    source: &SourceText,
    statement: &Statement,
    queue_name: &str,
    context_name: &str,
) -> Result<(), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "runnable enqueue must refine the affine queue in place",
        ));
    };
    let [queue, kernel, runnable, enqueue, context] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable enqueue must consume one queue and one suspended context",
        ));
    };
    if !identifier_is(source, queue, queue_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, runnable, "runnable")
        || !identifier_is(source, enqueue, "enqueue")
        || !identifier_is(source, context, context_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable enqueue must refine the queue and consume the selected suspended context",
        ));
    }
    Ok(())
}

fn parse_runnable_queue_dequeue(
    source: &SourceText,
    statement: &Statement,
    queue_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "runnable dequeue must bind one suspended context",
        ));
    };
    let [queue, kernel, runnable, dequeue] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable dequeue must consume the current queue state",
        ));
    };
    if source.slice(*classifier) != "SuspendedKernelContext InitialProcessor"
        || !identifier_is(source, queue, queue_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, runnable, "runnable")
        || !identifier_is(source, dequeue, "dequeue")
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable dequeue must select the FIFO head from the exact queue binding",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_runnable_queue_consume(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    queue_name: &str,
) -> Result<(), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "empty runnable-queue consumption must be an explicit operation",
        ));
    };
    let [
        context,
        kernel,
        runnable,
        queue,
        consume,
        empty,
        queue_value,
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "empty runnable-queue consumption requires the live context and queue",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, runnable, "runnable")
        || !identifier_is(source, queue, "queue")
        || !identifier_is(source, consume, "consume")
        || !identifier_is(source, empty, "empty")
        || !identifier_is(source, queue_value, queue_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable-queue consumption requires the exact empty queue before disposition",
        ));
    }
    Ok(())
}

fn kernel_context_create_fields(
    source: &SourceText,
    fields: &[ProductField],
    stack_name: &str,
    entry_name: &str,
) -> Result<bool, Diagnostic> {
    let mut stack = None;
    let mut entry = None;
    for field in fields {
        let Some(label) = field.label else {
            return Ok(false);
        };
        match source.slice(label) {
            "stack" if stack.is_none() => stack = Some(&field.value),
            "entry" if entry.is_none() => entry = Some(&field.value),
            _ => return Ok(false),
        }
    }
    Ok(
        matches!(stack, Some(value) if identifier_is(source, value, stack_name))
            && matches!(entry, Some(value) if identifier_is(source, value, entry_name)),
    )
}

fn parse_kernel_context_transfer(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    worker_name: &str,
    result_classifier: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            format!("kernel-context transfer must bind `{result_classifier}`"),
        ));
    };
    let [context, kernel, context_word, transfer, worker] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context transfer must consume one suspended context",
        ));
    };
    if source.slice(*classifier) != result_classifier
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, transfer, "transfer")
        || !identifier_is(source, worker, worker_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context transfer requires the live caller and matching suspended context",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_context_transfer_until_deadline(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    worker_name: &str,
    deadline_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "deadline-preemptible transfer must bind its exact affine outcome",
        ));
    };
    let [
        context,
        kernel,
        context_word,
        transfer,
        worker,
        until,
        deadline,
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "deadline-preemptible transfer must consume one selected context and deadline",
        ));
    };
    if source.slice(*classifier)
        != "PreemptedKernelContextTransfer InitialProcessor InitialMonotonicClock"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, transfer, "transfer")
        || !identifier_is(source, worker, worker_name)
        || !identifier_is(source, until, "until")
        || !identifier_is(source, deadline, deadline_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "deadline-preemptible transfer requires the live dispatcher, FIFO-selected context, and exact deadline",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_take_preempted_kernel_context(
    source: &SourceText,
    statement: &Statement,
    preempted_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "taking a preempted outcome must bind one suspended kernel context",
        ));
    };
    let [receiver, kernel, context_word, take, preempted] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "taking a preempted outcome must consume the exact affine result",
        ));
    };
    if source.slice(*classifier) != "SuspendedKernelContext InitialProcessor"
        || !identifier_is(source, receiver, preempted_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, take, "take")
        || !identifier_is(source, preempted, "preempted")
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "taking preemption requires the exact completed outcome before runnable enqueue",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_context_reclaim<'a>(
    source: &SourceText,
    statement: &'a Statement,
    completed_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let Statement::Implementation {
        name,
        classifier: Expression::Application { items, span },
        declarations,
        ..
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "kernel-context reclaim must bind the resumed caller and continue into its final disposition",
        ));
    };
    let [completed, kernel, context, reclaim] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context reclaim must consume one completed transfer",
        ));
    };
    if !identifier_is(source, completed, completed_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context, "context")
        || !identifier_is(source, reclaim, "reclaim")
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context reclaim must consume the matching completed transfer",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_deadline_after(
    source: &SourceText,
    statement: &Statement,
    instant_name: &str,
) -> Result<(String, CompilerSystemsOperation), Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(deadline_diagnostic(
            source,
            statement_span(statement),
            "deadline construction must bind `Deadline InitialMonotonicClock`",
        ));
    };
    let [
        instant,
        deadline,
        after,
        Expression::Measured { value, unit, .. },
    ] = items.as_slice()
    else {
        return Err(deadline_diagnostic(
            source,
            *span,
            "deadline construction must be `deadline : Deadline InitialMonotonicClock is instant deadline after 1[ms]`",
        ));
    };
    if source.slice(*classifier) != "Deadline InitialMonotonicClock"
        || !identifier_is(source, instant, instant_name)
        || !identifier_is(source, deadline, "deadline")
        || !identifier_is(source, after, "after")
        || source.slice(*value) != "1"
        || source.slice(*unit) != "ms"
    {
        return Err(deadline_diagnostic(
            source,
            *span,
            "the initial deadline must derive from the second observation and exact duration 1[ms]",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        CompilerSystemsOperation::ConstructDeadline {
            duration_nanoseconds: 1_000_000,
        },
    ))
}

fn parse_monotonic_clock_now(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
) -> Result<(String, CompilerSystemsOperation), Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(monotonic_clock_diagnostic(
            source,
            statement_span(statement),
            "clock observation must bind `Instant InitialMonotonicClock`",
        ));
    };
    let [context, monotonic, clock, now] = items.as_slice() else {
        return Err(monotonic_clock_diagnostic(
            source,
            *span,
            "clock observation must be `instant : Instant InitialMonotonicClock is context monotonic clock now`",
        ));
    };
    if source.slice(*classifier) != "Instant InitialMonotonicClock"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, monotonic, "monotonic")
        || !identifier_is(source, clock, "clock")
        || !identifier_is(source, now, "now")
    {
        return Err(monotonic_clock_diagnostic(
            source,
            *span,
            "clock observation requires the resumed context and initial monotonic clock classifier",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        CompilerSystemsOperation::ObserveMonotonicClock,
    ))
}

fn parse_atomic_exchanged_failure(
    source: &SourceText,
    false_action: &Expression,
    context_name: &str,
    region_name: &str,
    atomic_name: &str,
) -> Result<String, Diagnostic> {
    let failure = action_block(source, false_action, "failed atomic load")?;
    let [end] = failure else {
        return Err(atomic_diagnostic(
            source,
            false_action.span(),
            "failed atomic load requires atomic end, region release, and fatal disposition",
        ));
    };
    let failure_continuation = parse_atomic_end(source, end, atomic_name, region_name)?;
    let [release, disposition] = failure_continuation else {
        return Err(atomic_diagnostic(
            source,
            false_action.span(),
            "failed atomic load requires region release and fatal disposition after atomic end",
        ));
    };
    parse_region_release(source, release, context_name, region_name)?;
    let CompilerSystemsDisposition::Fatal {
        message: load_failure_message,
    } = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(disposition),
            "failed atomic load must enter the fatal disposition",
        ));
    };
    Ok(load_failure_message)
}

fn parse_local_notification_send(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
) -> Result<(String, CompilerSystemsOperation), Diagnostic> {
    let (name, items, span) = affine_application_binding(source, statement, "send")?;
    let [context, local, notification, send] = items else {
        return Err(local_interrupt_diagnostic(
            source,
            span,
            "send must bind `pending is context local notification send`",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, local, "local")
        || !identifier_is(source, notification, "notification")
        || !identifier_is(source, send, "send")
    {
        return Err(local_interrupt_diagnostic(
            source,
            span,
            "local-notification send must consume the current processor context",
        ));
    }
    Ok((
        source.slice(name).to_owned(),
        CompilerSystemsOperation::SendLocalNotification,
    ))
}

fn parse_local_notification_wait(
    source: &SourceText,
    statement: &Statement,
    pending_name: &str,
) -> Result<(String, CompilerSystemsOperation), Diagnostic> {
    let (name, items, span) = affine_application_binding(source, statement, "wait")?;
    let [pending, local, notification, wait] = items else {
        return Err(local_interrupt_diagnostic(
            source,
            span,
            "wait must bind `resumed is pending local notification wait`",
        ));
    };
    if !identifier_is(source, pending, pending_name)
        || !identifier_is(source, local, "local")
        || !identifier_is(source, notification, "notification")
        || !identifier_is(source, wait, "wait")
    {
        return Err(local_interrupt_diagnostic(
            source,
            span,
            "local-notification wait must consume the matching pending session",
        ));
    }
    Ok((
        source.slice(name).to_owned(),
        CompilerSystemsOperation::WaitLocalNotification,
    ))
}

fn parse_atomic_load(
    source: &SourceText,
    statement: &Statement,
    atomic_name: &str,
) -> Result<(String, CompilerAtomicOrder), Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(statement),
            "atomic load must bind its `Nat` result",
        ));
    };
    let [atomic, load, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic load must be `observed : Nat is atomic load (order is acquire)`",
        ));
    };
    if source.slice(*classifier) != "Nat"
        || !identifier_is(source, atomic, atomic_name)
        || !identifier_is(source, load, "load")
        || fields.len() != 1
    {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic load requires the live location and one named order",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        atomic_order_field(source, fields, "order")?,
    ))
}

fn parse_atomic_equality(
    source: &SourceText,
    statement: &Statement,
    loaded_name: &str,
) -> Result<u64, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(atomic_diagnostic(
            source,
            statement_span(statement),
            "atomic load requires an exhaustive equality decision",
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
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic load decision must compare its binding with a static natural",
        ));
    };
    if !identifier_is(source, loaded, loaded_name) {
        return Err(atomic_diagnostic(
            source,
            loaded.span(),
            "atomic comparison must use the immediately preceding load binding",
        ));
    }
    parse_storage_natural(source, expected)
}

fn parse_atomic_end<'a>(
    source: &SourceText,
    statement: &'a Statement,
    atomic_name: &str,
    region_name: &str,
) -> Result<&'a [Statement], Diagnostic> {
    let Statement::Implementation {
        name,
        classifier: Expression::Application { items, span },
        declarations,
        ..
    } = statement
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(statement),
            "atomic lifecycle must return its region through `region is atomic end`",
        ));
    };
    let [atomic, end] = items.as_slice() else {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic lifecycle must return its region through `region is atomic end`",
        ));
    };
    if source.slice(*name) != region_name
        || !identifier_is(source, atomic, atomic_name)
        || !identifier_is(source, end, "end")
    {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic end must consume the live location and restore the original region binding",
        ));
    }
    Ok(declarations)
}

fn atomic_natural_field(
    source: &SourceText,
    fields: &[ProductField],
    name: &str,
) -> Result<u64, Diagnostic> {
    let field = unique_atomic_field(source, fields, name)?;
    parse_storage_natural(source, &field.value)
}

fn atomic_identity_field(
    source: &SourceText,
    fields: &[ProductField],
    name: &str,
    required: &str,
) -> Result<String, Diagnostic> {
    let field = unique_atomic_field(source, fields, name)?;
    let Expression::Identifier(value) = field.value else {
        return Err(atomic_diagnostic(
            source,
            field.value.span(),
            format!("atomic parameter `{name}` must be a static identity"),
        ));
    };
    let value = source.slice(value).to_owned();
    if value != required {
        return Err(atomic_diagnostic(
            source,
            field.value.span(),
            format!("atomic parameter `{name}` requires `{required}`"),
        ));
    }
    Ok(value)
}

fn atomic_order_field(
    source: &SourceText,
    fields: &[ProductField],
    name: &str,
) -> Result<CompilerAtomicOrder, Diagnostic> {
    let field = unique_atomic_field(source, fields, name)?;
    let Expression::Identifier(value) = field.value else {
        return Err(atomic_diagnostic(
            source,
            field.value.span(),
            format!("atomic order `{name}` must be a static identity"),
        ));
    };
    match source.slice(value) {
        "atomic-only" => Ok(CompilerAtomicOrder::AtomicOnly),
        "acquire" => Ok(CompilerAtomicOrder::Acquire),
        "release" => Ok(CompilerAtomicOrder::Release),
        "acquire-release" => Ok(CompilerAtomicOrder::AcquireRelease),
        "sequential" => Ok(CompilerAtomicOrder::Sequential),
        other => Err(atomic_diagnostic(
            source,
            value,
            format!("unknown atomic order `{other}`"),
        )),
    }
}

fn unique_atomic_field<'a>(
    source: &SourceText,
    fields: &'a [ProductField],
    name: &str,
) -> Result<&'a ProductField, Diagnostic> {
    let mut found = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(atomic_diagnostic(
                source,
                field.value.span(),
                "atomic parameters must be named",
            ));
        };
        if source.slice(label) == name && found.replace(field).is_some() {
            return Err(atomic_diagnostic(
                source,
                label,
                format!("atomic parameter `{name}` is duplicated"),
            ));
        }
    }
    found.ok_or_else(|| {
        atomic_diagnostic(
            source,
            fields
                .first()
                .map_or(Span::new(0, 0), |field| field.value.span()),
            format!("atomic operation requires `{name}`"),
        )
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
        && let [
            receiver,
            pci,
            region,
            register,
            read,
            eventually,
            equals,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, region, "region")
        && identifier_is(source, register, "register")
        && identifier_is(source, read, "read")
        && identifier_is(source, eventually, "eventually")
        && identifier_is(source, equals, "equals")
    {
        let request = parse_pci_region_register_request(
            source,
            fields,
            *span,
            CompilerPciRegionRegisterDirection::ReadEventuallyEquals,
        )?;
        return Ok(CompilerSystemsOperation::AccessPciRegionRegister { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            configuration,
            dword,
            read,
            equals,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, configuration, "configuration")
        && identifier_is(source, dword, "dword")
        && identifier_is(source, read, "read")
        && identifier_is(source, equals, "equals")
    {
        let request = parse_pci_configuration_dword_request(source, fields, *span)?;
        if !CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
            .contains(&request)
        {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                *span,
                "the initial Q35 profile admits only requests from the sealed segment-zero 00:05.0 virtio-rng capability-discovery sequence",
            ));
        }
        return Ok(CompilerSystemsOperation::ObservePciConfigurationDword { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            configuration,
            word,
            update,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, configuration, "configuration")
        && identifier_is(source, word, "word")
        && identifier_is(source, update, "update")
    {
        let request = parse_pci_configuration_word_update_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::UpdatePciConfigurationWord { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            create,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, create, "create")
    {
        let request = parse_dma_buffer_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::CreateDmaBuffer { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            bind,
            reference,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, bind, "bind")
        && identifier_is(source, reference, "reference")
    {
        let request = parse_dma_buffer_reference_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::BindDmaBufferReference { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            integer,
            store,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, integer, "integer")
        && identifier_is(source, store, "store")
    {
        let request = parse_dma_buffer_integer_store_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::StoreDmaBufferInteger { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            integer,
            read,
            equals,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, integer, "integer")
        && identifier_is(source, read, "read")
        && identifier_is(source, equals, "equals")
    {
        let request = parse_dma_buffer_integer_load_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::LoadDmaBufferIntegerEquals { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            prepare,
            device,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, prepare, "prepare")
        && identifier_is(source, device, "device")
    {
        let request = parse_dma_buffer_preparation_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::PrepareDmaBufferForDevice { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            complete,
            cpu,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, complete, "complete")
        && identifier_is(source, cpu, "cpu")
    {
        let request = parse_dma_buffer_preparation_request(source, fields, *span)?;
        if !CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.contains(&request) {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-COMPLETE",
                *span,
                "the initial Q35 profile admits only completion of its sealed virtio-rng DMA buffers",
            ));
        }
        return Ok(CompilerSystemsOperation::CompleteDmaBufferForCpu { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            sample,
            capture,
            dma,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, sample, "sample")
        && identifier_is(source, capture, "capture")
        && identifier_is(source, dma, "dma")
    {
        let request = parse_entropy_sample_capture_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::CaptureEntropySample { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            sample,
            assess,
            quality,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, sample, "sample")
        && identifier_is(source, assess, "assess")
        && identifier_is(source, quality, "quality")
    {
        let request = parse_entropy_sample_quality_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::AssessEntropySampleQuality { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            pool,
            create,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, pool, "pool")
        && identifier_is(source, create, "create")
    {
        let request = parse_entropy_pool_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::CreateEntropyPool { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            pool,
            mix,
            sample,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, pool, "pool")
        && identifier_is(source, mix, "mix")
        && identifier_is(source, sample, "sample")
    {
        let request = parse_entropy_pool_mix_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::MixEntropyPoolSample { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            pool,
            credit,
            contribution,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, pool, "pool")
        && identifier_is(source, credit, "credit")
        && identifier_is(source, contribution, "contribution")
    {
        let request = parse_entropy_pool_credit_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::CreditEntropyPoolContribution { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            generator,
            initialize,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, generator, "generator")
        && identifier_is(source, initialize, "initialize")
    {
        let request = parse_entropy_generator_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::InitializeEntropyGenerator { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            generator,
            extract,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, generator, "generator")
        && identifier_is(source, extract, "extract")
    {
        let request = parse_entropy_generator_extract_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::ExtractEntropyGenerator { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            output,
            deliver,
            initial,
            auxv,
            random,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, output, "output")
        && identifier_is(source, deliver, "deliver")
        && identifier_is(source, initial, "initial")
        && identifier_is(source, auxv, "auxv")
        && identifier_is(source, random, "random")
    {
        let request = parse_entropy_output_delivery_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::DeliverInitialAuxvRandom { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            output,
            reserve,
            initial,
            user,
            random,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, output, "output")
        && identifier_is(source, reserve, "reserve")
        && identifier_is(source, initial, "initial")
        && identifier_is(source, user, "user")
        && identifier_is(source, random, "random")
    {
        let request = parse_entropy_output_reservation_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::ReserveInitialUserRandom { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            user,
            random,
            bind,
            reusable,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, user, "user")
        && identifier_is(source, random, "random")
        && identifier_is(source, bind, "bind")
        && identifier_is(source, reusable, "reusable")
    {
        let request = parse_entropy_user_random_service_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::BindReusableUserRandom { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            user,
            random,
            expand,
            small,
            requests,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, user, "user")
        && identifier_is(source, random, "random")
        && identifier_is(source, expand, "expand")
        && identifier_is(source, small, "small")
        && identifier_is(source, requests, "requests")
    {
        let request = parse_entropy_user_random_small_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::ExpandSmallUserRandomRequests { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            user,
            random,
            expand,
            large,
            requests,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, user, "user")
        && identifier_is(source, random, "random")
        && identifier_is(source, expand, "expand")
        && identifier_is(source, large, "large")
        && identifier_is(source, requests, "requests")
    {
        let request = parse_entropy_user_random_large_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::ExpandLargeUserRandomRequests { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            region,
            write,
            dma,
            reference,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, region, "region")
        && identifier_is(source, write, "write")
        && identifier_is(source, dma, "dma")
        && identifier_is(source, reference, "reference")
    {
        let request = parse_pci_region_dma_reference_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::PublishPciRegionDmaReference { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            region,
            register,
            direction,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, region, "region")
        && identifier_is(source, register, "register")
        && identifier_is(source, direction, "write")
    {
        let request = parse_pci_region_register_request(
            source,
            fields,
            *span,
            CompilerPciRegionRegisterDirection::Write,
        )?;
        return Ok(CompilerSystemsOperation::AccessPciRegionRegister { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            region,
            register,
            read,
            equals,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, region, "region")
        && identifier_is(source, register, "register")
        && identifier_is(source, read, "read")
        && identifier_is(source, equals, "equals")
    {
        let request = parse_pci_region_register_request(
            source,
            fields,
            *span,
            CompilerPciRegionRegisterDirection::ReadEquals,
        )?;
        return Ok(CompilerSystemsOperation::AccessPciRegionRegister { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [receiver, debug, break_operation] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, debug, "debug")
        && identifier_is(source, break_operation, "break")
    {
        return Ok(CompilerSystemsOperation::DebugBreak);
    }
    if context == CompilerSystemsContextKind::KernelThread
        && let [
            receiver,
            kernel,
            context_word,
            await_word,
            deadline,
            preemption,
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, kernel, "kernel")
        && identifier_is(source, context_word, "context")
        && identifier_is(source, await_word, "await")
        && identifier_is(source, deadline, "deadline")
        && identifier_is(source, preemption, "preemption")
    {
        return Ok(CompilerSystemsOperation::AwaitDeadlinePreemption);
    }
    if context == CompilerSystemsContextKind::KernelThread
        && let [receiver, kernel, completion, await_word] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, kernel, "kernel")
        && identifier_is(source, completion, "completion")
        && identifier_is(source, await_word, "await")
    {
        return Ok(CompilerSystemsOperation::AwaitKernelCompletion);
    }
    Err(source_diagnostic(
        source,
        "E-SYSTEMS-OPERATION",
        *span,
        "operation is not admitted by this live systems entry context",
    ))
}

fn parse_pci_configuration_dword_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerPciConfigurationDwordRequest, Diagnostic> {
    const LABELS: [&str; 6] = [
        "segment",
        "bus",
        "device",
        "function",
        "register-offset-bytes",
        "expected-value",
    ];
    if fields.len() != LABELS.len() {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-CONFIG",
            span,
            "PCI configuration dword observation requires exactly segment, bus, device, function, register-offset-bytes, and expected-value",
        ));
    }
    let mut values = [0_u64; 6];
    for ((field, expected_label), value) in fields.iter().zip(LABELS).zip(&mut values) {
        let Some(label) = field.label else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                field.value.span(),
                format!("PCI configuration dword observation requires named `{expected_label}`"),
            ));
        };
        if source.slice(label) != expected_label {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                label,
                format!(
                    "PCI configuration dword observation requires `{expected_label}` in source order"
                ),
            ));
        }
        *value = parse_storage_natural(source, &field.value)?;
    }
    let request = CompilerPciConfigurationDwordRequest {
        segment: u16::try_from(values[0]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI segment exceeds 65535",
            )
        })?,
        bus: u8::try_from(values[1]).map_err(|_| {
            source_diagnostic(source, "E-SYSTEMS-PCI-CONFIG", span, "PCI bus exceeds 255")
        })?,
        device: u8::try_from(values[2]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI device exceeds 31",
            )
        })?,
        function: u8::try_from(values[3]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI function exceeds 7",
            )
        })?,
        register_offset_bytes: u8::try_from(values[4]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI register offset exceeds 252",
            )
        })?,
        expected_value: u32::try_from(values[5]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI expected value exceeds 32 bits",
            )
        })?,
    };
    if !request.is_well_formed() {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-CONFIG",
            span,
            "PCI configuration request requires device 0..31, function 0..7, and a four-byte-aligned register offset in 0..252",
        ));
    }
    Ok(request)
}

fn parse_pci_configuration_word_update_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerPciConfigurationWordUpdateRequest, Diagnostic> {
    const LABELS: [&str; 8] = [
        "segment",
        "bus",
        "device",
        "function",
        "register-offset-bytes",
        "mask",
        "expected-masked-value",
        "desired-masked-value",
    ];
    if fields.len() != LABELS.len() {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-CONFIG-UPDATE",
            span,
            "PCI configuration-word update requires exactly the sealed function, offset, mask, pre-state, and post-state fields",
        ));
    }
    let mut values = [0_u64; 8];
    for ((field, expected_label), value) in fields.iter().zip(LABELS).zip(&mut values) {
        let Some(label) = field.label else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                field.value.span(),
                format!("PCI configuration-word update requires named `{expected_label}`"),
            ));
        };
        if source.slice(label) != expected_label {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                label,
                format!(
                    "PCI configuration-word update requires `{expected_label}` in source order"
                ),
            ));
        }
        *value = parse_storage_natural(source, &field.value)?;
    }
    let request = CompilerPciConfigurationWordUpdateRequest {
        segment: u16::try_from(values[0]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI segment exceeds 65535",
            )
        })?,
        bus: u8::try_from(values[1]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI bus exceeds 255",
            )
        })?,
        device: u8::try_from(values[2]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI device exceeds 31",
            )
        })?,
        function: u8::try_from(values[3]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI function exceeds 7",
            )
        })?,
        register_offset_bytes: u8::try_from(values[4]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI configuration-word offset exceeds 255",
            )
        })?,
        mask: u16::try_from(values[5]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI configuration-word mask exceeds 65535",
            )
        })?,
        expected_masked_value: u16::try_from(values[6]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI configuration-word expected value exceeds 65535",
            )
        })?,
        desired_masked_value: u16::try_from(values[7]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI configuration-word desired value exceeds 65535",
            )
        })?,
    };
    if !request.is_well_formed()
        || request != CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-CONFIG-UPDATE",
            span,
            "the initial Q35 profile admits only the sealed virtio-rng bus-master command transition",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferRequest, Diagnostic> {
    if fields.len() != 8 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-BUFFER",
            span,
            "DMA-buffer creation requires exactly identity, device, translation-domain, size, alignment, direction, visibility, and role fields",
        ));
    }
    let direction_value = field_static_string(source, &fields[5], "direction")?;
    let direction = match direction_value.as_str() {
        "bidirectional" => CompilerDmaDirection::Bidirectional,
        "device-to-memory" => CompilerDmaDirection::DeviceToMemory,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-BUFFER",
                span,
                "DMA-buffer direction must be `bidirectional` or `device-to-memory`",
            ));
        }
    };
    let visibility_value = field_static_string(source, &fields[6], "visibility")?;
    let visibility = match visibility_value.as_str() {
        "provider-managed" => CompilerDmaVisibilityPolicy::ProviderManaged,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-BUFFER",
                span,
                "DMA-buffer visibility must be `provider-managed`",
            ));
        }
    };
    let role_value = field_static_string(source, &fields[7], "role")?;
    let role = match role_value.as_str() {
        "virtqueue-control" => CompilerDmaBufferRole::VirtqueueControl,
        "entropy-payload" => CompilerDmaBufferRole::EntropyPayload,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-BUFFER",
                span,
                "DMA-buffer role must be `virtqueue-control` or `entropy-payload`",
            ));
        }
    };
    let request = CompilerDmaBufferRequest {
        buffer_identity: field_natural(source, &fields[0], "buffer-identity")?,
        device_identity: field_natural(source, &fields[1], "device-identity")?,
        translation_domain_identity: field_natural(
            source,
            &fields[2],
            "translation-domain-identity",
        )?,
        byte_count: field_natural(source, &fields[3], "byte-count")?,
        alignment_bytes: field_natural(source, &fields[4], "alignment-bytes")?,
        direction,
        visibility,
        role,
    };
    if !request.is_well_formed()
        || !CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-BUFFER",
            span,
            "the initial Q35 profile admits only the two sealed virtio-rng DMA-buffer requests",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_reference_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferReferenceRequest, Diagnostic> {
    if fields.len() != 5 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-CONSTRUCT",
            span,
            "DMA-reference binding requires destination identity/offset, referenced identity/offset, and width",
        ));
    }
    let width = field_natural(source, &fields[4], "width-bytes")?;
    let request = CompilerDmaBufferReferenceRequest {
        destination_buffer_identity: field_natural(
            source,
            &fields[0],
            "destination-buffer-identity",
        )?,
        destination_offset_bytes: field_natural(source, &fields[1], "destination-offset-bytes")?,
        referenced_buffer_identity: field_natural(
            source,
            &fields[2],
            "referenced-buffer-identity",
        )?,
        referenced_offset_bytes: field_natural(source, &fields[3], "referenced-offset-bytes")?,
        width_bytes: u8::try_from(width).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-CONSTRUCT",
                span,
                "DMA-reference width exceeds 255",
            )
        })?,
    };
    if !request.is_well_formed()
        || request != CompilerDmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-CONSTRUCT",
            span,
            "the initial Q35 profile admits only the sealed virtio-rng descriptor reference",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_integer_store_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferIntegerStoreRequest, Diagnostic> {
    if fields.len() != 4 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-CONSTRUCT",
            span,
            "DMA integer store requires buffer identity, offset, width, and value",
        ));
    }
    let width = field_natural(source, &fields[2], "width-bytes")?;
    let request = CompilerDmaBufferIntegerStoreRequest {
        buffer_identity: field_natural(source, &fields[0], "buffer-identity")?,
        offset_bytes: field_natural(source, &fields[1], "offset-bytes")?,
        width_bytes: u8::try_from(width).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-CONSTRUCT",
                span,
                "DMA integer-store width exceeds 255",
            )
        })?,
        value: field_natural(source, &fields[3], "value")?,
    };
    if !request.is_well_formed()
        || !CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING
            .contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-CONSTRUCT",
            span,
            "the initial Q35 profile admits only stores from the sealed virtio-rng split-ring construction",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_preparation_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferPreparationRequest, Diagnostic> {
    if fields.len() != 6 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-PREPARE",
            span,
            "DMA device preparation requires buffer, device, translation-domain, direction, visibility, and role fields",
        ));
    }
    let direction = match field_static_string(source, &fields[3], "direction")?.as_str() {
        "bidirectional" => CompilerDmaDirection::Bidirectional,
        "device-to-memory" => CompilerDmaDirection::DeviceToMemory,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PREPARE",
                span,
                "DMA preparation direction must be `bidirectional` or `device-to-memory`",
            ));
        }
    };
    let visibility = match field_static_string(source, &fields[4], "visibility")?.as_str() {
        "provider-managed" => CompilerDmaVisibilityPolicy::ProviderManaged,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PREPARE",
                span,
                "DMA preparation visibility must be `provider-managed`",
            ));
        }
    };
    let role = match field_static_string(source, &fields[5], "role")?.as_str() {
        "virtqueue-control" => CompilerDmaBufferRole::VirtqueueControl,
        "entropy-payload" => CompilerDmaBufferRole::EntropyPayload,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PREPARE",
                span,
                "DMA preparation role must be `virtqueue-control` or `entropy-payload`",
            ));
        }
    };
    let request = CompilerDmaBufferPreparationRequest {
        buffer_identity: field_natural(source, &fields[0], "buffer-identity")?,
        device_identity: field_natural(source, &fields[1], "device-identity")?,
        translation_domain_identity: field_natural(
            source,
            &fields[2],
            "translation-domain-identity",
        )?,
        direction,
        visibility,
        role,
    };
    if !request.is_well_formed()
        || !CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-PREPARE",
            span,
            "the initial Q35 profile admits only the sealed payload and control DMA-preparation requests",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_integer_load_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferIntegerLoadRequest, Diagnostic> {
    if fields.len() != 4 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-USED-RING",
            span,
            "DMA integer read requires buffer identity, offset, width, and expected value",
        ));
    }
    let width = field_natural(source, &fields[2], "width-bytes")?;
    let request = CompilerDmaBufferIntegerLoadRequest {
        buffer_identity: field_natural(source, &fields[0], "buffer-identity")?,
        offset_bytes: field_natural(source, &fields[1], "offset-bytes")?,
        width_bytes: u8::try_from(width).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-USED-RING",
                span,
                "DMA integer-read width exceeds 255",
            )
        })?,
        expected_value: field_natural(source, &fields[3], "expected-value")?,
    };
    if !request.is_well_formed()
        || !CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-USED-RING",
            span,
            "the initial Q35 profile admits only the sealed virtio-rng used-ring reads",
        ));
    }
    Ok(request)
}

fn parse_entropy_sample_capture_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropySampleCaptureRequest, Diagnostic> {
    if fields.len() != 10 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-SAMPLE",
            span,
            "entropy-sample capture requires sample, buffer, range, device, domain, direction, visibility, role, and credit fields",
        ));
    }
    let direction = match field_static_string(source, &fields[6], "direction")?.as_str() {
        "device-to-memory" => CompilerDmaDirection::DeviceToMemory,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-ENTROPY-SAMPLE",
                span,
                "entropy-sample DMA direction must be `device-to-memory`",
            ));
        }
    };
    let visibility = match field_static_string(source, &fields[7], "visibility")?.as_str() {
        "provider-managed" => CompilerDmaVisibilityPolicy::ProviderManaged,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-ENTROPY-SAMPLE",
                span,
                "entropy-sample DMA visibility must be `provider-managed`",
            ));
        }
    };
    let role = match field_static_string(source, &fields[8], "role")?.as_str() {
        "entropy-payload" => CompilerDmaBufferRole::EntropyPayload,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-ENTROPY-SAMPLE",
                span,
                "entropy-sample DMA role must be `entropy-payload`",
            ));
        }
    };
    let request = CompilerEntropySampleCaptureRequest {
        sample_identity: field_natural(source, &fields[0], "sample-identity")?,
        buffer_identity: field_natural(source, &fields[1], "buffer-identity")?,
        offset_bytes: field_natural(source, &fields[2], "offset-bytes")?,
        byte_count: field_natural(source, &fields[3], "byte-count")?,
        device_identity: field_natural(source, &fields[4], "device-identity")?,
        translation_domain_identity: field_natural(
            source,
            &fields[5],
            "translation-domain-identity",
        )?,
        direction,
        visibility,
        role,
        credited_entropy_bits: field_natural(source, &fields[9], "credited-entropy-bits")?,
    };
    if !request.is_well_formed()
        || request != CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-SAMPLE",
            span,
            "the initial Q35 profile admits only the sealed 32-byte uncredited virtio-rng sample capture",
        ));
    }
    Ok(request)
}

fn parse_entropy_sample_quality_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropySampleQualityRequest, Diagnostic> {
    if fields.len() != 6 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-QUALITY",
            span,
            "entropy-sample quality assessment requires sample, source, trust evidence, extent, quality, and credit fields",
        ));
    }
    let request = CompilerEntropySampleQualityRequest {
        sample_identity: field_natural(source, &fields[0], "sample-identity")?,
        source_identity: field_natural(source, &fields[1], "source-identity")?,
        trust_evidence_identity: field_natural(source, &fields[2], "trust-evidence-identity")?,
        byte_count: field_natural(source, &fields[3], "byte-count")?,
        quality_parts_per_1024: field_natural(
            source,
            &fields[4],
            "quality-parts-per-thousand-twenty-four",
        )?,
        credited_entropy_bits: field_natural(source, &fields[5], "credited-entropy-bits")?,
    };
    if !request.is_well_formed()
        || request != CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-QUALITY",
            span,
            "the initial Q35 profile admits only the sealed fully credited 32-byte virtio-rng sample assessment",
        ));
    }
    Ok(request)
}

fn parse_entropy_pool_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyPoolRequest, Diagnostic> {
    if fields.len() != 3 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-POOL",
            span,
            "entropy-pool creation requires identity, credit-capacity, and initial-credit fields",
        ));
    }
    let request = CompilerEntropyPoolRequest {
        pool_identity: field_natural(source, &fields[0], "pool-identity")?,
        credit_capacity_bits: field_natural(source, &fields[1], "credit-capacity-bits")?,
        initial_credited_entropy_bits: field_natural(
            source,
            &fields[2],
            "initial-credited-entropy-bits",
        )?,
    };
    if !request.is_well_formed() || request != CompilerEntropyPoolRequest::INITIAL_Q35 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-POOL",
            span,
            "the initial Q35 profile admits only the sealed empty 256-bit entropy pool",
        ));
    }
    Ok(request)
}

fn parse_entropy_pool_mix_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyPoolMixRequest, Diagnostic> {
    if fields.len() != 6 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-MIX",
            span,
            "entropy-pool mixing requires pool, sample, conditioner, contribution, extent, and contribution-credit fields",
        ));
    }
    let request = CompilerEntropyPoolMixRequest {
        pool_identity: field_natural(source, &fields[0], "pool-identity")?,
        sample_identity: field_natural(source, &fields[1], "sample-identity")?,
        conditioner_identity: field_natural(source, &fields[2], "conditioner-identity")?,
        contribution_identity: field_natural(source, &fields[3], "contribution-identity")?,
        byte_count: field_natural(source, &fields[4], "byte-count")?,
        contribution_entropy_bits: field_natural(source, &fields[5], "contribution-entropy-bits")?,
    };
    if !request.is_well_formed() || request != CompilerEntropyPoolMixRequest::INITIAL_Q35 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-MIX",
            span,
            "the initial Q35 profile admits only the sealed first BLAKE2s-256 entropy-pool mix",
        ));
    }
    Ok(request)
}

fn parse_entropy_pool_credit_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyPoolCreditRequest, Diagnostic> {
    if fields.len() != 7 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-CREDIT",
            span,
            "entropy-credit transfer requires pool, contribution, origin, prior, transfer, capacity, and result fields",
        ));
    }
    let request = CompilerEntropyPoolCreditRequest {
        pool_identity: field_natural(source, &fields[0], "pool-identity")?,
        contribution_identity: field_natural(source, &fields[1], "contribution-identity")?,
        conditioner_origin_identity: field_natural(
            source,
            &fields[2],
            "conditioner-origin-identity",
        )?,
        prior_credited_entropy_bits: field_natural(
            source,
            &fields[3],
            "prior-credited-entropy-bits",
        )?,
        contribution_entropy_bits: field_natural(source, &fields[4], "contribution-entropy-bits")?,
        credit_capacity_bits: field_natural(source, &fields[5], "credit-capacity-bits")?,
        resulting_credited_entropy_bits: field_natural(
            source,
            &fields[6],
            "resulting-credited-entropy-bits",
        )?,
    };
    if !request.is_well_formed() || request != CompilerEntropyPoolCreditRequest::INITIAL_Q35 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-CREDIT",
            span,
            "the initial Q35 profile admits only the sealed zero-to-256 contribution transfer",
        ));
    }
    Ok(request)
}

fn parse_entropy_generator_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyGeneratorRequest, Diagnostic> {
    if fields.len() != 7 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-GENERATOR",
            span,
            "entropy-generator initialization requires pool, generator, algorithm, generation, key extent, required credit, and observed credit fields",
        ));
    }
    let request = CompilerEntropyGeneratorRequest {
        pool_identity: field_natural(source, &fields[0], "pool-identity")?,
        generator_identity: field_natural(source, &fields[1], "generator-identity")?,
        algorithm_identity: field_natural(source, &fields[2], "algorithm-identity")?,
        generation: field_natural(source, &fields[3], "generation")?,
        key_byte_count: field_natural(source, &fields[4], "key-byte-count")?,
        required_entropy_bits: field_natural(source, &fields[5], "required-entropy-bits")?,
        observed_pool_entropy_bits: field_natural(
            source,
            &fields[6],
            "observed-pool-entropy-bits",
        )?,
    };
    if !request.is_well_formed() || request != CompilerEntropyGeneratorRequest::INITIAL_Q35 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-GENERATOR",
            span,
            "the initial Q35 profile admits only the sealed generation-one 32-byte ChaCha20 initialization",
        ));
    }
    Ok(request)
}

fn parse_entropy_generator_extract_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyGeneratorExtractRequest, Diagnostic> {
    if fields.len() != 10 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-EXTRACT",
            span,
            "entropy-generator extraction requires generator, algorithm, generations, output, block, split, counter, and nonce fields",
        ));
    }
    let request = CompilerEntropyGeneratorExtractRequest {
        generator_identity: field_natural(source, &fields[0], "generator-identity")?,
        algorithm_identity: field_natural(source, &fields[1], "algorithm-identity")?,
        prior_generation: field_natural(source, &fields[2], "prior-generation")?,
        resulting_generation: field_natural(source, &fields[3], "resulting-generation")?,
        output_identity: field_natural(source, &fields[4], "output-identity")?,
        block_byte_count: field_natural(source, &fields[5], "block-byte-count")?,
        output_offset_bytes: field_natural(source, &fields[6], "output-offset-bytes")?,
        output_byte_count: field_natural(source, &fields[7], "output-byte-count")?,
        counter: field_natural(source, &fields[8], "counter")?,
        nonce_identity: field_natural(source, &fields[9], "nonce-identity")?,
    };
    if !request.is_well_formed()
        || (request != CompilerEntropyGeneratorExtractRequest::INITIAL_Q35
            && request != CompilerEntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-EXTRACT",
            span,
            "the initial Q35 profile admits only its two sealed consecutive 32-byte ChaCha20 extractions",
        ));
    }
    Ok(request)
}

fn parse_entropy_output_reservation_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyOutputReservationRequest, Diagnostic> {
    if fields.len() != 12 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-RESERVE",
            span,
            "initial getrandom reservation requires output, extent, process, interface, service, flag-policy, and single-use fields",
        ));
    }
    let request = CompilerEntropyOutputReservationRequest {
        output_identity: field_natural(source, &fields[0], "output-identity")?,
        source_byte_count: field_natural(source, &fields[1], "source-byte-count")?,
        selected_offset_bytes: field_natural(source, &fields[2], "selected-offset-bytes")?,
        maximum_delivery_byte_count: field_natural(
            source,
            &fields[3],
            "maximum-delivery-byte-count",
        )?,
        discarded_offset_bytes: field_natural(source, &fields[4], "discarded-offset-bytes")?,
        discarded_byte_count: field_natural(source, &fields[5], "discarded-byte-count")?,
        process_identity: field_natural(source, &fields[6], "process-identity")?,
        interface_identity: field_natural(source, &fields[7], "interface-identity")?,
        service_identity: field_natural(source, &fields[8], "service-identity")?,
        supported_flag_mask: field_natural(source, &fields[9], "supported-flag-mask")?,
        incompatible_flag_mask: field_natural(source, &fields[10], "incompatible-flag-mask")?,
        single_use: field_natural(source, &fields[11], "single-use")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyOutputReservationRequest::INITIAL_GETRANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-RESERVE",
            span,
            "the initial profile admits only the sealed single-use 16-byte getrandom reservation from output two",
        ));
    }
    Ok(request)
}

fn parse_entropy_user_random_service_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyUserRandomServiceRequest, Diagnostic> {
    if fields.len() != 13 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-SERVICE",
            span,
            "reusable getrandom binding requires generator, generation, output, extents, process, interface, service, flag-policy, first-output, and reusable fields",
        ));
    }
    let request = CompilerEntropyUserRandomServiceRequest {
        generator_identity: field_natural(source, &fields[0], "generator-identity")?,
        generation: field_natural(source, &fields[1], "generation")?,
        initial_output_identity: field_natural(source, &fields[2], "initial-output-identity")?,
        block_byte_count: field_natural(source, &fields[3], "block-byte-count")?,
        output_byte_count: field_natural(source, &fields[4], "output-byte-count")?,
        maximum_delivery_byte_count: field_natural(
            source,
            &fields[5],
            "maximum-delivery-byte-count",
        )?,
        process_identity: field_natural(source, &fields[6], "process-identity")?,
        interface_identity: field_natural(source, &fields[7], "interface-identity")?,
        service_identity: field_natural(source, &fields[8], "service-identity")?,
        supported_flag_mask: field_natural(source, &fields[9], "supported-flag-mask")?,
        incompatible_flag_mask: field_natural(source, &fields[10], "incompatible-flag-mask")?,
        reserved_first_output: field_natural(source, &fields[11], "reserved-first-output")? == 1,
        reusable: field_natural(source, &fields[12], "reusable")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyUserRandomServiceRequest::INITIAL_GETRANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-SERVICE",
            span,
            "the initial profile admits only its sealed generation-three reusable 32-byte getrandom service",
        ));
    }
    Ok(request)
}

fn parse_entropy_user_random_small_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyUserRandomSmallRequest, Diagnostic> {
    if fields.len() != 8 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-SMALL-REQUEST",
            span,
            "small-request getrandom refinement requires process, interface, service, algorithm, output, count, delivery, and completion fields",
        ));
    }
    let request = CompilerEntropyUserRandomSmallRequest {
        process_identity: field_natural(source, &fields[0], "process-identity")?,
        interface_identity: field_natural(source, &fields[1], "interface-identity")?,
        service_identity: field_natural(source, &fields[2], "service-identity")?,
        algorithm_identity: field_natural(source, &fields[3], "algorithm-identity")?,
        output_byte_count: field_natural(source, &fields[4], "output-byte-count")?,
        maximum_output_count: field_natural(source, &fields[5], "maximum-output-count")?,
        maximum_delivery_byte_count: field_natural(
            source,
            &fields[6],
            "maximum-delivery-byte-count",
        )?,
        complete_request: field_natural(source, &fields[7], "complete-request")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyUserRandomSmallRequest::INITIAL_GETRANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-SMALL-REQUEST",
            span,
            "the initial profile admits only its sealed eight-output 256-byte complete getrandom service",
        ));
    }
    Ok(request)
}

fn parse_entropy_user_random_large_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyUserRandomSmallRequest, Diagnostic> {
    if fields.len() != 8 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-LARGE-REQUEST",
            span,
            "large-request getrandom refinement requires process, interface, service, algorithm, output, count, delivery, and completion fields",
        ));
    }
    let request = CompilerEntropyUserRandomSmallRequest {
        process_identity: field_natural(source, &fields[0], "process-identity")?,
        interface_identity: field_natural(source, &fields[1], "interface-identity")?,
        service_identity: field_natural(source, &fields[2], "service-identity")?,
        algorithm_identity: field_natural(source, &fields[3], "algorithm-identity")?,
        output_byte_count: field_natural(source, &fields[4], "output-byte-count")?,
        maximum_output_count: field_natural(source, &fields[5], "maximum-output-count")?,
        maximum_delivery_byte_count: field_natural(
            source,
            &fields[6],
            "maximum-delivery-byte-count",
        )?,
        complete_request: field_natural(source, &fields[7], "complete-request")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyUserRandomSmallRequest::LARGE_GETRANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-LARGE-REQUEST",
            span,
            "the initial profile admits only its sealed 16-output 512-byte complete getrandom service",
        ));
    }
    Ok(request)
}

fn parse_entropy_output_delivery_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyOutputDeliveryRequest, Diagnostic> {
    if fields.len() != 10 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-DELIVER",
            span,
            "initial AT_RANDOM delivery requires output, source, selected, discarded, address-space, interface, tag, and single-use fields",
        ));
    }
    let request = CompilerEntropyOutputDeliveryRequest {
        output_identity: field_natural(source, &fields[0], "output-identity")?,
        source_byte_count: field_natural(source, &fields[1], "source-byte-count")?,
        selected_offset_bytes: field_natural(source, &fields[2], "selected-offset-bytes")?,
        delivered_byte_count: field_natural(source, &fields[3], "delivered-byte-count")?,
        discarded_offset_bytes: field_natural(source, &fields[4], "discarded-offset-bytes")?,
        discarded_byte_count: field_natural(source, &fields[5], "discarded-byte-count")?,
        address_space_identity: field_natural(source, &fields[6], "address-space-identity")?,
        interface_identity: field_natural(source, &fields[7], "interface-identity")?,
        auxiliary_tag: field_natural(source, &fields[8], "auxiliary-tag")?,
        single_use: field_natural(source, &fields[9], "single-use")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-DELIVER",
            span,
            "the initial profile admits only single-use delivery of the first 16 bytes from output one to Linux AT_RANDOM with the remainder discarded",
        ));
    }
    Ok(request)
}

fn parse_pci_region_register_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
    direction: CompilerPciRegionRegisterDirection,
) -> Result<CompilerPciRegionRegisterRequest, Diagnostic> {
    const LABELS: [&str; 11] = [
        "segment",
        "bus",
        "device",
        "function",
        "access-capability-offset-bytes",
        "bar",
        "region-offset-bytes",
        "region-length-bytes",
        "register-offset-bytes",
        "width-bytes",
        "value",
    ];
    if fields.len() != LABELS.len() {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-REGION",
            span,
            "PCI region-register access requires exactly the sealed function, capability, BAR, region, register, width, and value fields",
        ));
    }
    let mut values = [0_u64; 11];
    for ((field, expected_label), value) in fields.iter().zip(LABELS).zip(&mut values) {
        let Some(label) = field.label else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                field.value.span(),
                format!("PCI region-register access requires named `{expected_label}`"),
            ));
        };
        if source.slice(label) != expected_label {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                label,
                format!("PCI region-register access requires `{expected_label}` in source order"),
            ));
        }
        *value = parse_storage_natural(source, &field.value)?;
    }
    let request = CompilerPciRegionRegisterRequest {
        segment: u16::try_from(values[0]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI segment exceeds 65535",
            )
        })?,
        bus: u8::try_from(values[1]).map_err(|_| {
            source_diagnostic(source, "E-SYSTEMS-PCI-REGION", span, "PCI bus exceeds 255")
        })?,
        device: u8::try_from(values[2]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI device exceeds 31",
            )
        })?,
        function: u8::try_from(values[3]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI function exceeds 7",
            )
        })?,
        access_capability_offset_bytes: u8::try_from(values[4]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI access capability offset exceeds 255",
            )
        })?,
        bar: u8::try_from(values[5]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI BAR index exceeds 255",
            )
        })?,
        region_offset_bytes: u32::try_from(values[6]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI region offset exceeds 32 bits",
            )
        })?,
        region_length_bytes: u32::try_from(values[7]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI region length exceeds 32 bits",
            )
        })?,
        register_offset_bytes: u32::try_from(values[8]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI register offset exceeds 32 bits",
            )
        })?,
        width_bytes: u8::try_from(values[9]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI register width exceeds 255",
            )
        })?,
        direction,
        value: u32::try_from(values[10]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI register value exceeds 32 bits",
            )
        })?,
    };
    if !request.is_well_formed()
        || !(CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
            .contains(&request)
            || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                .contains(&request)
            || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                .contains(&request)
            || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                .contains(&request)
            || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                .contains(&request))
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-REGION",
            span,
            "the initial Q35 profile admits only requests from the sealed virtio-rng feature-negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences",
        ));
    }
    Ok(request)
}

fn parse_pci_region_dma_reference_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerPciRegionDmaReferenceRequest, Diagnostic> {
    if fields.len() != 14 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-PUBLISH",
            span,
            "PCI-region DMA-reference publication requires exactly fourteen region, buffer, and domain fields",
        ));
    }
    let width = field_natural(source, &fields[9], "width-bytes")?;
    let request = CompilerPciRegionDmaReferenceRequest {
        segment: u16::try_from(field_natural(source, &fields[0], "segment")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI segment exceeds 16 bits",
            )
        })?,
        bus: u8::try_from(field_natural(source, &fields[1], "bus")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI bus exceeds 8 bits",
            )
        })?,
        device: u8::try_from(field_natural(source, &fields[2], "device")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI device exceeds 8 bits",
            )
        })?,
        function: u8::try_from(field_natural(source, &fields[3], "function")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI function exceeds 8 bits",
            )
        })?,
        access_capability_offset_bytes: u8::try_from(field_natural(
            source,
            &fields[4],
            "access-capability-offset-bytes",
        )?)
        .map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI access-capability offset exceeds 8 bits",
            )
        })?,
        bar: u8::try_from(field_natural(source, &fields[5], "bar")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI BAR exceeds 8 bits",
            )
        })?,
        region_offset_bytes: u32::try_from(field_natural(
            source,
            &fields[6],
            "region-offset-bytes",
        )?)
        .map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI region offset exceeds 32 bits",
            )
        })?,
        region_length_bytes: u32::try_from(field_natural(
            source,
            &fields[7],
            "region-length-bytes",
        )?)
        .map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI region length exceeds 32 bits",
            )
        })?,
        register_offset_bytes: u32::try_from(field_natural(
            source,
            &fields[8],
            "register-offset-bytes",
        )?)
        .map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI register offset exceeds 32 bits",
            )
        })?,
        width_bytes: u8::try_from(width).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "DMA-reference width exceeds 8 bits",
            )
        })?,
        buffer_identity: field_natural(source, &fields[10], "buffer-identity")?,
        buffer_offset_bytes: field_natural(source, &fields[11], "buffer-offset-bytes")?,
        device_identity: field_natural(source, &fields[12], "device-identity")?,
        translation_domain_identity: field_natural(
            source,
            &fields[13],
            "translation-domain-identity",
        )?,
    };
    if !request.is_well_formed()
        || !CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
            .contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-PUBLISH",
            span,
            "the initial Q35 profile admits only the three sealed virtio-rng queue-address publications",
        ));
    }
    Ok(request)
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
    if matches!(
        context,
        CompilerSystemsContextKind::DebugBreak
            | CompilerSystemsContextKind::LocalNotificationInterrupt
            | CompilerSystemsContextKind::DeadlineInterrupt
    ) && let [receiver, resume] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, resume, "resume")
    {
        return Ok(CompilerSystemsDisposition::Resume);
    }
    if context == CompilerSystemsContextKind::DeadlineInterrupt
        && let [receiver, preempt, current, kernel, context_word] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, preempt, "preempt")
        && identifier_is(source, current, "current")
        && identifier_is(source, kernel, "kernel")
        && identifier_is(source, context_word, "context")
    {
        return Ok(CompilerSystemsDisposition::PreemptKernelContext);
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

    fn is_queue_finalization(operation: &CompilerSystemsOperation) -> bool {
        matches!(
            operation,
            CompilerSystemsOperation::AccessPciRegionRegister { request }
                if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                    .contains(request)
                    || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                        .contains(request)
                    || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                        .contains(request)
        ) || matches!(
            operation,
            CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
                | CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
                | CompilerSystemsOperation::CaptureEntropySample { .. }
                | CompilerSystemsOperation::AssessEntropySampleQuality { .. }
                | CompilerSystemsOperation::CreateEntropyPool { .. }
                | CompilerSystemsOperation::MixEntropyPoolSample { .. }
                | CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
                | CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                | CompilerSystemsOperation::BindReusableUserRandom { .. }
                | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
        )
    }

    fn remove_entropy_sample_capture(program: &mut CompilerSystemsProgram) {
        program.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::CaptureEntropySample { .. }
                    | CompilerSystemsOperation::AssessEntropySampleQuality { .. }
                    | CompilerSystemsOperation::CreateEntropyPool { .. }
                    | CompilerSystemsOperation::MixEntropyPoolSample { .. }
                    | CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
                    | CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                    | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                    | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                    | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                    | CompilerSystemsOperation::BindReusableUserRandom { .. }
                    | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                    | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
            )
        });
        program.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA
                && *effect != SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY
                && *effect != SYSTEMS_ENTROPY_POOL_CREATE
                && *effect != SYSTEMS_ENTROPY_POOL_MIX_SAMPLE
                && *effect != SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION
                && *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
                && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
                && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
                && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
        });
    }

    macro_rules! assert_initial_transition_shape {
        ($actual:expr, $legacy_expected:expr) => {{
            let transitions = $actual;
            let _ = stringify!($legacy_expected);
            assert!(matches!(
                transitions.first(),
                Some(CompilerSystemsTransition::ProvisionBootstrapStorage { .. })
            ));
            assert!(matches!(
                transitions.last(),
                Some(CompilerSystemsTransition::Fatal { message })
                    if message == "toolchain gate complete"
            ));
            let delivery = transitions
                .iter()
                .position(|transition| matches!(
                    transition,
                    CompilerSystemsTransition::DeliverInitialAuxvRandom { request }
                        if *request == CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM
                ))
                .expect("modeled initial AT_RANDOM delivery");
            let entry = transitions
                .iter()
                .position(|transition| matches!(
                    transition,
                    CompilerSystemsTransition::EnterUserExecution { .. }
                ))
                .expect("modeled initial user entry");
            let reservation = transitions
                .iter()
                .position(|transition| matches!(
                    transition,
                    CompilerSystemsTransition::ReserveInitialUserRandom { request }
                        if *request == CompilerEntropyOutputReservationRequest::INITIAL_GETRANDOM
                ))
                .expect("modeled initial getrandom reservation");
            let binding = transitions
                .iter()
                .position(|transition| matches!(
                    transition,
                    CompilerSystemsTransition::BindReusableUserRandom { request }
                        if *request == CompilerEntropyUserRandomServiceRequest::INITIAL_GETRANDOM
                ))
                .expect("modeled reusable getrandom binding");
            assert!(delivery < reservation && reservation < binding && binding < entry);
        }};
    }

    #[test]
    #[allow(clippy::too_many_lines)] // One assertion retains the complete ordered systems trace.
    fn checks_the_initial_artifact_and_models_entry_transitions() {
        // TOPAL-SYSTEMS-VOCABULARY-001, TOPAL-SYSTEMS-ENTRY-001,
        // TOPAL-SYSTEMS-DISPOSITION-001, TOPAL-SYSTEMS-OBSERVATION-001,
        // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-MONOTONIC-CLOCK-001,
        // TOPAL-SYSTEMS-DEADLINE-EVENT-001.
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
                SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
                SYSTEMS_ATOMIC_WORD_CREATE,
                SYSTEMS_ATOMIC_END,
                SYSTEMS_ATOMIC_LOAD,
                SYSTEMS_BOOT_MEMORY_DESCRIBE,
                SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
                SYSTEMS_KERNEL_COMPLETION_CONSUME,
                SYSTEMS_KERNEL_COMPLETION_CREATE,
                SYSTEMS_KERNEL_COMPLETION_SIGNAL,
                SYSTEMS_KERNEL_CONTEXT_CREATE,
                SYSTEMS_KERNEL_CONTEXT_RECLAIM,
                SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME,
                SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE,
                SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE,
                SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE,
                SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
                SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE,
                SYSTEMS_KERNEL_CONTEXT_TRANSFER,
                SYSTEMS_USER_CONTEXT_COMPLETE,
                SYSTEMS_USER_CONTEXT_ENTER,
                SYSTEMS_USER_CONTEXT_PREPARE,
                SYSTEMS_CRITICAL_ENTER,
                SYSTEMS_CRITICAL_RESTORE,
                SYSTEMS_CONSOLE_WRITE,
                SYSTEMS_PCI_CONFIGURATION_READ_DWORD,
                SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD,
                SYSTEMS_PCI_REGION_READ_EVENTUALLY,
                SYSTEMS_PCI_REGION_READ,
                SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE,
                SYSTEMS_PCI_REGION_WRITE,
                SYSTEMS_FATAL,
                SYSTEMS_DMA_BUFFER_BIND_REFERENCE,
                SYSTEMS_DMA_BUFFER_COMPLETE_CPU,
                SYSTEMS_DMA_BUFFER_CREATE,
                SYSTEMS_DMA_BUFFER_LOAD_INTEGER,
                SYSTEMS_DMA_BUFFER_PREPARE_DEVICE,
                SYSTEMS_DMA_BUFFER_STORE_INTEGER,
                SYSTEMS_ENTROPY_GENERATOR_EXTRACT,
                SYSTEMS_ENTROPY_GENERATOR_INITIALIZE,
                SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM,
                SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM,
                SYSTEMS_ENTROPY_POOL_CREATE,
                SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION,
                SYSTEMS_ENTROPY_POOL_MIX_SAMPLE,
                SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY,
                SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA,
                SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE,
                SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS,
                SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS,
                SYSTEMS_FRAMES_ALLOCATE,
                SYSTEMS_FRAMES_RELEASE,
                SYSTEMS_LOCAL_NOTIFICATION_SEND,
                SYSTEMS_LOCAL_NOTIFICATION_WAIT,
                SYSTEMS_DEBUG_BREAK,
                SYSTEMS_KERNEL_MAPPING_LOAD_BYTE,
                SYSTEMS_KERNEL_MAP,
                SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
                SYSTEMS_KERNEL_UNMAP,
                SYSTEMS_FRAME_ALLOCATOR_CREATE,
                SYSTEMS_USER_ADDRESS_SPACE_MAP,
                SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
                SYSTEMS_USER_ADDRESS_SPACE_RECLAIM,
                SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
                SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS,
                SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
                SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE,
                SYSTEMS_USER_MAPPING_LOAD_BYTE,
                SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
                SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA,
                SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
                SYSTEMS_USER_MAPPING_STORE_BYTE,
                SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PROGRAM_HEADERS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_ACQUISITION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_HANDOFF,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DELIVER_ENTROPY_INITIAL_RANDOM,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_BUS_MASTER,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DEVICE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_BUFFERS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_PREPARATION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_NEGOTIATION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_POOL_LIFECYCLE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_REQUEST_COMPLETION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_CAPTURE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_QUALITY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SPLIT_RING,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_TRANSPORT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_RETAINED_DISJOINT_WORK,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PROGRAM_HEADERS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_PREPARED_PROCESS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RETAIN_PREPARED_PROCESS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_ACQUISITION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_HANDOFF,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_INITIAL_RANDOM,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING,
                SYSTEMS_STATIC_IMAGE_PROCESS_ENTER,
                SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY,
                SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
                SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
                SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
                SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
                SYSTEMS_DEADLINE_AFTER,
                SYSTEMS_MONOTONIC_CLOCK_NOW,
                SYSTEMS_TRANSLATION_ACTIVATE,
                SYSTEMS_TRANSLATION_BEGIN,
                SYSTEMS_TRANSLATION_COMMIT,
                SYSTEMS_TRANSLATION_EDIT_BEGIN,
                SYSTEMS_TRANSLATION_EDIT_COMMIT,
                SYSTEMS_TRANSLATION_EDIT_MAP,
                SYSTEMS_TRANSLATION_EDIT_UNMAP,
            ]
        );
        assert_initial_transition_shape!(
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
                CompilerSystemsTransition::MapKernelFrames {
                    request: CompilerKernelMappingRequest::initial_read_write(),
                },
                CompilerSystemsTransition::StoreKernelMappingByte {
                    offset_bytes: 0,
                    value: 165,
                },
                CompilerSystemsTransition::LoadKernelMappingByte {
                    offset_bytes: 0,
                    value: 165,
                },
                CompilerSystemsTransition::UnmapKernelFrames,
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_FRAME_MAPPED".into(),
                },
                CompilerSystemsTransition::ReleasePhysicalFrames,
                CompilerSystemsTransition::BeginTranslationUpdate {
                    request: CompilerTranslationUpdateRequest::initial_bootstrap_equivalent(),
                },
                CompilerSystemsTransition::CommitTranslationUpdate,
                CompilerSystemsTransition::ActivateTranslationSpace,
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_TRANSLATION_ACTIVE".into(),
                },
                CompilerSystemsTransition::AllocatePhysicalFrames {
                    request: CompilerPhysicalFrameRequest {
                        frame_count: 1,
                        alignment_frames: 1,
                    },
                },
                CompilerSystemsTransition::BeginTranslationEdit {
                    kind: CompilerTranslationEditKind::Map,
                },
                CompilerSystemsTransition::MapTranslationFrames {
                    request: CompilerTranslationMappingRequest::initial_read_write(),
                },
                CompilerSystemsTransition::CommitTranslationEdit {
                    kind: CompilerTranslationEditKind::Map,
                },
                CompilerSystemsTransition::StoreKernelMappingByte {
                    offset_bytes: 0,
                    value: 60,
                },
                CompilerSystemsTransition::LoadKernelMappingByte {
                    offset_bytes: 0,
                    value: 60,
                },
                CompilerSystemsTransition::BeginTranslationEdit {
                    kind: CompilerTranslationEditKind::Unmap,
                },
                CompilerSystemsTransition::UnmapTranslationMapping,
                CompilerSystemsTransition::CommitTranslationEdit {
                    kind: CompilerTranslationEditKind::Unmap,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_TRANSLATION_EDITED".into(),
                },
                CompilerSystemsTransition::ReleasePhysicalFrames,
                CompilerSystemsTransition::AllocatePhysicalFrames {
                    request: CompilerPhysicalFrameRequest {
                        frame_count: 3,
                        alignment_frames: 1,
                    },
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_FRAME_EXTENT_OK".into(),
                },
                CompilerSystemsTransition::ReleasePhysicalFrames,
                CompilerSystemsTransition::EnterCritical {
                    domain: CompilerCriticalDomain::LocalMaskableInterrupts,
                    nesting_identity: 1,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_INTERRUPTS_MASKED".into(),
                },
                CompilerSystemsTransition::RestoreCritical {
                    domain: CompilerCriticalDomain::LocalMaskableInterrupts,
                    nesting_identity: 1,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_MEMORY_DESCRIBED".into(),
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 1,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [0],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 2,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [1],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 3,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [2],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 4,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [3],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 5,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [4],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 6,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [5],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 7,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [6],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 8,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [7],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 9,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [8],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 10,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [9],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 11,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [10],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 12,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [11],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 13,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [12],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 14,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [13],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 15,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [14],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 16,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [15],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 17,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [16],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 18,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [17],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 19,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [18],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 20,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [19],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 21,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [20],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 22,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [21],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 23,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [22],
                },
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity: 24,
                    request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                        [23],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_PCI_VIRTIO_RNG_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_PCI_CAPABILITIES_OK".into(),
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 1,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [0],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 2,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [1],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 3,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [2],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 4,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [3],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 5,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [4],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 6,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [5],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 7,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [6],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 8,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [7],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 9,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [8],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 10,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [9],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 11,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [10],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 12,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [11],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 13,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [12],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 14,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [13],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 15,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [14],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 16,
                    request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        [15],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_PCI_NEGOTIATED".into(),
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 17,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[0],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 18,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[1],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 19,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[2],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 20,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[3],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 21,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[4],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_QUEUE_DISCOVERED".into(),
                },
                CompilerSystemsTransition::UpdatePciConfigurationWord {
                    transaction_identity: 1,
                    request:
                        CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_PCI_BUS_MASTER_OK".into(),
                },
                CompilerSystemsTransition::CreateDmaBuffer {
                    request: CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[0],
                },
                CompilerSystemsTransition::CreateDmaBuffer {
                    request: CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[1],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_DMA_BUFFERS_OK".into(),
                },
                CompilerSystemsTransition::BindDmaBufferReference {
                    request: CompilerDmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR,
                },
                CompilerSystemsTransition::StoreDmaBufferInteger {
                    request:
                        CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[0],
                },
                CompilerSystemsTransition::StoreDmaBufferInteger {
                    request:
                        CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[1],
                },
                CompilerSystemsTransition::StoreDmaBufferInteger {
                    request:
                        CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[2],
                },
                CompilerSystemsTransition::StoreDmaBufferInteger {
                    request:
                        CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[3],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_SPLIT_RING_OK".into(),
                },
                CompilerSystemsTransition::PrepareDmaBufferForDevice {
                    request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
                },
                CompilerSystemsTransition::PrepareDmaBufferForDevice {
                    request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_DMA_DEVICE_OWNED".into(),
                },
                CompilerSystemsTransition::PublishPciRegionDmaReference {
                    transaction_identity: 22,
                    request:
                        CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                            [0],
                },
                CompilerSystemsTransition::PublishPciRegionDmaReference {
                    transaction_identity: 23,
                    request:
                        CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                            [1],
                },
                CompilerSystemsTransition::PublishPciRegionDmaReference {
                    transaction_identity: 24,
                    request:
                        CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                            [2],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_QUEUE_ADDRESSES_OK".into(),
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
                CompilerSystemsTransition::CreateAtomicWord {
                    request: CompilerAtomicWordRequest::initial(),
                },
                CompilerSystemsTransition::CompareExchangeAtomicWord {
                    expected: 41,
                    desired: 42,
                    observed: 41,
                    exchanged: true,
                    success_order: CompilerAtomicOrder::AcquireRelease,
                    failure_order: CompilerAtomicOrder::Acquire,
                },
                CompilerSystemsTransition::LoadAtomicWord {
                    value: 42,
                    order: CompilerAtomicOrder::Acquire,
                },
                CompilerSystemsTransition::EndAtomicWord,
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_ATOMIC_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_MEMORY_OK".into(),
                },
                CompilerSystemsTransition::ReleaseBootstrapRegion,
                CompilerSystemsTransition::SendLocalNotification { event_identity: 1 },
                CompilerSystemsTransition::BeginLocalNotificationWait { event_identity: 1 },
                CompilerSystemsTransition::ObserveLocalNotification { event_identity: 1 },
                CompilerSystemsTransition::EnterLocalNotificationInterrupt { event_identity: 1 },
                CompilerSystemsTransition::CompleteLocalNotificationInterrupt { event_identity: 1 },
                CompilerSystemsTransition::ResumeLocalNotificationInterrupt { event_identity: 1 },
                CompilerSystemsTransition::EndLocalNotificationWait { event_identity: 1 },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_INTERRUPT_OK".into(),
                },
                CompilerSystemsTransition::ObserveMonotonicClock {
                    observation_identity: 1,
                },
                CompilerSystemsTransition::ObserveMonotonicClock {
                    observation_identity: 2,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_TIME_OK".into(),
                },
                CompilerSystemsTransition::ConstructDeadline {
                    source_observation_identity: 2,
                    duration_nanoseconds: 1_000_000,
                },
                CompilerSystemsTransition::AllocateBootstrapRegion {
                    request: CompilerBootstrapStorageRequest {
                        byte_count: 16_384,
                        alignment_bytes: 16,
                        placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
                    },
                },
                CompilerSystemsTransition::CreateKernelCompletion {
                    completion_identity: 1,
                },
                CompilerSystemsTransition::CreateKernelContext {
                    context_identity: 1,
                },
                CompilerSystemsTransition::AllocateBootstrapRegion {
                    request: CompilerBootstrapStorageRequest {
                        byte_count: 16_384,
                        alignment_bytes: 16,
                        placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
                    },
                },
                CompilerSystemsTransition::CreateKernelContext {
                    context_identity: 2,
                },
                CompilerSystemsTransition::AllocateBootstrapRegion {
                    request: CompilerBootstrapStorageRequest {
                        byte_count: 16_384,
                        alignment_bytes: 16,
                        placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
                    },
                },
                CompilerSystemsTransition::CreateKernelContext {
                    context_identity: 3,
                },
                CompilerSystemsTransition::CreateRunnableQueue { capacity: 3 },
                CompilerSystemsTransition::EnqueueRunnableContext {
                    context_identity: 1,
                },
                CompilerSystemsTransition::EnqueueRunnableContext {
                    context_identity: 2,
                },
                CompilerSystemsTransition::EnqueueRunnableContext {
                    context_identity: 3,
                },
                CompilerSystemsTransition::DequeueRunnableContext {
                    context_identity: 1,
                },
                CompilerSystemsTransition::ArmDeadline { event_identity: 1 },
                CompilerSystemsTransition::TransferKernelContextUntilDeadline {
                    context_identity: 1,
                    caller_identity: 1,
                    event_identity: 1,
                },
                CompilerSystemsTransition::EnterKernelThread {
                    context_identity: 1,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED".into(),
                },
                CompilerSystemsTransition::AwaitDeadlinePreemption {
                    context_identity: 1,
                },
                CompilerSystemsTransition::ObserveDeadline { event_identity: 1 },
                CompilerSystemsTransition::EnterDeadlineInterrupt { event_identity: 1 },
                CompilerSystemsTransition::CompleteDeadlineInterrupt { event_identity: 1 },
                CompilerSystemsTransition::PreemptKernelContext {
                    event_identity: 1,
                    context_identity: 1,
                    caller_identity: 1,
                },
                CompilerSystemsTransition::ResumePreemptionDispatcher { caller_identity: 1 },
                CompilerSystemsTransition::TakePreemptedKernelContext {
                    context_identity: 1,
                    event_identity: 1,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_DEADLINE_PREEMPTED".into(),
                },
                CompilerSystemsTransition::EnqueueRunnableContext {
                    context_identity: 1,
                },
                CompilerSystemsTransition::DequeueRunnableContext {
                    context_identity: 2,
                },
                CompilerSystemsTransition::TransferKernelContextUntilCompletion {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                CompilerSystemsTransition::EnterKernelThread {
                    context_identity: 2,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_ENTERED".into(),
                },
                CompilerSystemsTransition::AwaitKernelCompletion {
                    context_identity: 2,
                    completion_identity: 1,
                },
                CompilerSystemsTransition::BlockKernelContext {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                CompilerSystemsTransition::ResumeCompletionDispatcher { caller_identity: 2 },
                CompilerSystemsTransition::DequeueRunnableContext {
                    context_identity: 3,
                },
                CompilerSystemsTransition::TransferKernelContext {
                    context_identity: 3,
                    caller_identity: 3,
                },
                CompilerSystemsTransition::EnterKernelThread {
                    context_identity: 3,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED".into(),
                },
                CompilerSystemsTransition::RetireKernelContextToCaller {
                    context_identity: 3,
                    caller_identity: 3,
                },
                CompilerSystemsTransition::ResumeKernelContextCaller { caller_identity: 3 },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED".into(),
                },
                CompilerSystemsTransition::ReclaimKernelContext {
                    context_identity: 3,
                },
                CompilerSystemsTransition::SignalBlockedKernelCompletion {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                CompilerSystemsTransition::EnqueueRunnableContext {
                    context_identity: 2,
                },
                CompilerSystemsTransition::DequeueRunnableContext {
                    context_identity: 1,
                },
                CompilerSystemsTransition::TransferKernelContext {
                    context_identity: 1,
                    caller_identity: 4,
                },
                CompilerSystemsTransition::RestorePreemptedKernelContext {
                    context_identity: 1,
                    caller_identity: 4,
                    event_identity: 1,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED".into(),
                },
                CompilerSystemsTransition::RetireKernelContextToCaller {
                    context_identity: 1,
                    caller_identity: 4,
                },
                CompilerSystemsTransition::ResumeKernelContextCaller { caller_identity: 4 },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED".into(),
                },
                CompilerSystemsTransition::ReclaimKernelContext {
                    context_identity: 1,
                },
                CompilerSystemsTransition::DequeueRunnableContext {
                    context_identity: 2,
                },
                CompilerSystemsTransition::TransferKernelContext {
                    context_identity: 2,
                    caller_identity: 5,
                },
                CompilerSystemsTransition::RestoreBlockedKernelContext {
                    context_identity: 2,
                    caller_identity: 5,
                    completion_identity: 1,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RESUMED".into(),
                },
                CompilerSystemsTransition::RetireKernelContextToCaller {
                    context_identity: 2,
                    caller_identity: 5,
                },
                CompilerSystemsTransition::ResumeKernelContextCaller { caller_identity: 5 },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RETIRED".into(),
                },
                CompilerSystemsTransition::ReclaimKernelContext {
                    context_identity: 2,
                },
                CompilerSystemsTransition::ConsumeKernelCompletion {
                    completion_identity: 1,
                },
                CompilerSystemsTransition::ConsumeEmptyRunnableQueue,
                CompilerSystemsTransition::AllocatePhysicalFrames {
                    request: CompilerPhysicalFrameRequest {
                        frame_count: 3,
                        alignment_frames: 1,
                    },
                },
                CompilerSystemsTransition::PrepareUserAddressSpace {
                    address_space_identity: 1,
                },
                CompilerSystemsTransition::AllocatePhysicalFrames {
                    request: CompilerPhysicalFrameRequest {
                        frame_count: 2,
                        alignment_frames: 1,
                    },
                },
                CompilerSystemsTransition::MapUserAddressSpace {
                    address_space_identity: 1,
                    mapping_identity: 1,
                },
                CompilerSystemsTransition::InstallUserMappingEntryTrampoline {
                    mapping_identity: 1,
                },
                CompilerSystemsTransition::ProtectUserMappingReadOnly {
                    mapping_identity: 1,
                },
                CompilerSystemsTransition::VerifyUserMappingEntryTrampoline {
                    mapping_identity: 1,
                },
                CompilerSystemsTransition::PermitUserMappingExecute {
                    mapping_identity: 1,
                },
                CompilerSystemsTransition::AllocatePhysicalFrames {
                    request: CompilerPhysicalFrameRequest {
                        frame_count: 1,
                        alignment_frames: 1,
                    },
                },
                CompilerSystemsTransition::MapUserAddressSpace {
                    address_space_identity: 1,
                    mapping_identity: 2,
                },
                CompilerSystemsTransition::StoreUserMappingByte {
                    mapping_identity: 2,
                },
                CompilerSystemsTransition::ProtectUserMappingReadOnly {
                    mapping_identity: 2,
                },
                CompilerSystemsTransition::LoadUserMappingByte {
                    mapping_identity: 2,
                },
                CompilerSystemsTransition::PermitUserMappingWritableData {
                    mapping_identity: 2,
                },
                CompilerSystemsTransition::AllocatePhysicalFrames {
                    request: CompilerPhysicalFrameRequest {
                        frame_count: 34,
                        alignment_frames: 1,
                    },
                },
                CompilerSystemsTransition::ReserveAnonymousUserMapping {
                    address_space_identity: 1,
                    reservation_identity: 3,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_READY".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_BOOT_RESOURCE_OK".into(),
                },
                CompilerSystemsTransition::PrepareUserProbe {
                    process_identity: 1,
                    probe_identity: 1,
                },
                CompilerSystemsTransition::EnterUserExecution {
                    process_identity: 1,
                    kernel_continuation_identity: 1,
                },
                CompilerSystemsTransition::EnterUserSyscall {
                    entry_identity: 1,
                    number: 39,
                    arguments: [0; 6],
                },
                CompilerSystemsTransition::ReturnToUser {
                    entry_identity: 1,
                    result: 1,
                },
                CompilerSystemsTransition::EnterUserSyscall {
                    entry_identity: 2,
                    number: 63,
                    arguments: [6_299_136, 0, 0, 0, 0, 0],
                },
                CompilerSystemsTransition::BeginUserTransfer {
                    entry_identity: 2,
                    transfer_identity: 1,
                    candidate: 6_299_136,
                    extent: 390,
                },
                CompilerSystemsTransition::CopyToUserComplete {
                    transfer_identity: 1,
                    copied: 390,
                },
                CompilerSystemsTransition::EndUserTransfer {
                    entry_identity: 2,
                    transfer_identity: 1,
                },
                CompilerSystemsTransition::ReturnToUser {
                    entry_identity: 2,
                    result: 0,
                },
                CompilerSystemsTransition::EnterUserSyscall {
                    entry_identity: 3,
                    number: 63,
                    arguments: [6_299_392, 0, 0, 0, 0, 0],
                },
                CompilerSystemsTransition::BeginUserTransfer {
                    entry_identity: 3,
                    transfer_identity: 2,
                    candidate: 6_299_392,
                    extent: 390,
                },
                CompilerSystemsTransition::CopyToUserFault {
                    transfer_identity: 2,
                    copied_prefix: 256,
                },
                CompilerSystemsTransition::EndUserTransfer {
                    entry_identity: 3,
                    transfer_identity: 2,
                },
                CompilerSystemsTransition::ReturnToUser {
                    entry_identity: 3,
                    result: u64::MAX - 13,
                },
                CompilerSystemsTransition::EnterUserSyscall {
                    entry_identity: 4,
                    number: 35,
                    arguments: [6_299_632, 0, 0, 0, 0, 0],
                },
                CompilerSystemsTransition::BeginUserTransfer {
                    entry_identity: 4,
                    transfer_identity: 3,
                    candidate: 6_299_632,
                    extent: 16,
                },
                CompilerSystemsTransition::CopyFromUserComplete {
                    transfer_identity: 3,
                    copied: 16,
                    seconds: 0,
                    nanoseconds: 0,
                },
                CompilerSystemsTransition::EndUserTransfer {
                    entry_identity: 4,
                    transfer_identity: 3,
                },
                CompilerSystemsTransition::ReturnToUser {
                    entry_identity: 4,
                    result: 0,
                },
                CompilerSystemsTransition::EnterUserSyscall {
                    entry_identity: 5,
                    number: 35,
                    arguments: [6_299_640, 0, 0, 0, 0, 0],
                },
                CompilerSystemsTransition::BeginUserTransfer {
                    entry_identity: 5,
                    transfer_identity: 4,
                    candidate: 6_299_640,
                    extent: 16,
                },
                CompilerSystemsTransition::CopyFromUserFault {
                    transfer_identity: 4,
                    copied_prefix: 8,
                },
                CompilerSystemsTransition::EndUserTransfer {
                    entry_identity: 5,
                    transfer_identity: 4,
                },
                CompilerSystemsTransition::ReturnToUser {
                    entry_identity: 5,
                    result: u64::MAX - 13,
                },
                CompilerSystemsTransition::EnterUserSyscall {
                    entry_identity: 6,
                    number: 231,
                    arguments: [0; 6],
                },
                CompilerSystemsTransition::TerminateUser {
                    entry_identity: 6,
                    status: 0,
                },
                CompilerSystemsTransition::ResumeUserKernelContinuation {
                    kernel_continuation_identity: 1,
                    status: 0,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_READ_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_CLOSE_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_DUP_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_FCNTL_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_FSTAT_OK".into(),
                },
                CompilerSystemsOperation::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_TCGETS_OK".into(),
                },
                CompilerSystemsOperation::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_WINSIZE_OK".into(),
                },
                CompilerSystemsOperation::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_LSEEK_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_COPY_TO_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_COPY_FROM_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_SYSCALL_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_STACK_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_AUXV_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_ELF_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_BSS_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_DATA_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_RETAINED".into(),
                },
                CompilerSystemsTransition::ReleaseAnonymousUserMapping {
                    address_space_identity: 1,
                    reservation_identity: 3,
                },
                CompilerSystemsTransition::ReleasePhysicalFrames,
                CompilerSystemsTransition::UnmapUserAddressSpace {
                    address_space_identity: 1,
                    mapping_identity: 2,
                },
                CompilerSystemsTransition::ReleasePhysicalFrames,
                CompilerSystemsTransition::UnmapUserAddressSpace {
                    address_space_identity: 1,
                    mapping_identity: 1,
                },
                CompilerSystemsTransition::ReleasePhysicalFrames,
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_MAPPING_OK".into(),
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_FRAME_DISJOINT_OK".into(),
                },
                CompilerSystemsTransition::CompleteUserExecution {
                    process_identity: 1,
                },
                CompilerSystemsTransition::ReclaimUserAddressSpace {
                    address_space_identity: 1,
                },
                CompilerSystemsTransition::ReleasePhysicalFrames,
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_BACKING_OK".into(),
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 25,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION[0],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 26,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION[1],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 27,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION[2],
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 28,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION[3],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE".into(),
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 29,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                            [0],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED".into(),
                },
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity: 30,
                    request:
                        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                            [0],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL".into(),
                },
                CompilerSystemsTransition::CompleteDmaBufferForCpu {
                    request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                },
                CompilerSystemsTransition::LoadDmaBufferIntegerEquals {
                    request: CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                        [0],
                },
                CompilerSystemsTransition::LoadDmaBufferIntegerEquals {
                    request: CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                        [1],
                },
                CompilerSystemsTransition::LoadDmaBufferIntegerEquals {
                    request: CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                        [2],
                },
                CompilerSystemsTransition::LoadDmaBufferIntegerEquals {
                    request: CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                        [3],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_USED_RING_OK".into(),
                },
                CompilerSystemsTransition::CompleteDmaBufferForCpu {
                    request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED".into(),
                },
                CompilerSystemsTransition::CaptureEntropySample {
                    request: CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED".into(),
                },
                CompilerSystemsTransition::AssessEntropySampleQuality {
                    request: CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED".into(),
                },
                CompilerSystemsTransition::CreateEntropyPool {
                    request: CompilerEntropyPoolRequest::INITIAL_Q35,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_ENTROPY_POOL_CREATED".into(),
                },
                CompilerSystemsTransition::MixEntropyPoolSample {
                    request: CompilerEntropyPoolMixRequest::INITIAL_Q35,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_ENTROPY_POOL_MIXED".into(),
                },
                CompilerSystemsTransition::CreditEntropyPoolContribution {
                    request: CompilerEntropyPoolCreditRequest::INITIAL_Q35,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_ENTROPY_POOL_CREDITED".into(),
                },
                CompilerSystemsTransition::InitializeEntropyGenerator {
                    request: CompilerEntropyGeneratorRequest::INITIAL_Q35,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_ENTROPY_GENERATOR_READY".into(),
                },
                CompilerSystemsTransition::ExtractEntropyGenerator {
                    request: CompilerEntropyGeneratorExtractRequest::INITIAL_Q35,
                },
                CompilerSystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_ENTROPY_OUTPUT_READY".into(),
                },
                CompilerSystemsTransition::Fatal {
                    message: "toolchain gate complete".into(),
                },
            ]
        );
    }

    #[test]
    fn checks_affine_bootstrap_region_access_and_models_real_byte_contents() {
        // TOPAL-SYSTEMS-STORAGE-001, TOPAL-SYSTEMS-QUALIFY-001,
        // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-STATIC-ELF-MAPPING-RESOURCES-001,
        // TOPAL-SYSTEMS-STATIC-ELF-LEAVES-001.
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        assert_eq!(program.bootstrap.handler.operations.len(), 422);
        assert_eq!(
            program.bootstrap.handler.effects,
            [
                SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
                SYSTEMS_ATOMIC_WORD_CREATE,
                SYSTEMS_ATOMIC_END,
                SYSTEMS_ATOMIC_LOAD,
                SYSTEMS_BOOT_MEMORY_DESCRIBE,
                SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
                SYSTEMS_KERNEL_COMPLETION_CONSUME,
                SYSTEMS_KERNEL_COMPLETION_CREATE,
                SYSTEMS_KERNEL_COMPLETION_SIGNAL,
                SYSTEMS_KERNEL_CONTEXT_CREATE,
                SYSTEMS_KERNEL_CONTEXT_RECLAIM,
                SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME,
                SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE,
                SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE,
                SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE,
                SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
                SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE,
                SYSTEMS_KERNEL_CONTEXT_TRANSFER,
                SYSTEMS_USER_CONTEXT_COMPLETE,
                SYSTEMS_USER_CONTEXT_ENTER,
                SYSTEMS_USER_CONTEXT_PREPARE,
                SYSTEMS_CRITICAL_ENTER,
                SYSTEMS_CRITICAL_RESTORE,
                SYSTEMS_CONSOLE_WRITE,
                SYSTEMS_PCI_CONFIGURATION_READ_DWORD,
                SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD,
                SYSTEMS_PCI_REGION_READ_EVENTUALLY,
                SYSTEMS_PCI_REGION_READ,
                SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE,
                SYSTEMS_PCI_REGION_WRITE,
                SYSTEMS_FATAL,
                SYSTEMS_DMA_BUFFER_BIND_REFERENCE,
                SYSTEMS_DMA_BUFFER_COMPLETE_CPU,
                SYSTEMS_DMA_BUFFER_CREATE,
                SYSTEMS_DMA_BUFFER_LOAD_INTEGER,
                SYSTEMS_DMA_BUFFER_PREPARE_DEVICE,
                SYSTEMS_DMA_BUFFER_STORE_INTEGER,
                SYSTEMS_ENTROPY_GENERATOR_EXTRACT,
                SYSTEMS_ENTROPY_GENERATOR_INITIALIZE,
                SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM,
                SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM,
                SYSTEMS_ENTROPY_POOL_CREATE,
                SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION,
                SYSTEMS_ENTROPY_POOL_MIX_SAMPLE,
                SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY,
                SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA,
                SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE,
                SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS,
                SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS,
                SYSTEMS_FRAMES_ALLOCATE,
                SYSTEMS_FRAMES_RELEASE,
                SYSTEMS_LOCAL_NOTIFICATION_SEND,
                SYSTEMS_LOCAL_NOTIFICATION_WAIT,
                SYSTEMS_DEBUG_BREAK,
                SYSTEMS_KERNEL_MAPPING_LOAD_BYTE,
                SYSTEMS_KERNEL_MAP,
                SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
                SYSTEMS_KERNEL_UNMAP,
                SYSTEMS_FRAME_ALLOCATOR_CREATE,
                SYSTEMS_USER_ADDRESS_SPACE_MAP,
                SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
                SYSTEMS_USER_ADDRESS_SPACE_RECLAIM,
                SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
                SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS,
                SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
                SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE,
                SYSTEMS_USER_MAPPING_LOAD_BYTE,
                SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
                SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA,
                SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
                SYSTEMS_USER_MAPPING_STORE_BYTE,
                SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PROGRAM_HEADERS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_ACQUISITION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_HANDOFF,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DELIVER_ENTROPY_INITIAL_RANDOM,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_BUS_MASTER,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DEVICE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_BUFFERS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_PREPARATION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_NEGOTIATION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_POOL_LIFECYCLE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_REQUEST_COMPLETION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_CAPTURE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_QUALITY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SPLIT_RING,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_TRANSPORT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_RETAINED_DISJOINT_WORK,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PROGRAM_HEADERS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_PREPARED_PROCESS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RETAIN_PREPARED_PROCESS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_ACQUISITION,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_HANDOFF,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_INITIAL_RANDOM,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD,
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING,
                SYSTEMS_STATIC_IMAGE_PROCESS_ENTER,
                SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY,
                SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
                SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
                SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
                SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
                SYSTEMS_DEADLINE_AFTER,
                SYSTEMS_MONOTONIC_CLOCK_NOW,
                SYSTEMS_TRANSLATION_ACTIVATE,
                SYSTEMS_TRANSLATION_BEGIN,
                SYSTEMS_TRANSLATION_COMMIT,
                SYSTEMS_TRANSLATION_EDIT_BEGIN,
                SYSTEMS_TRANSLATION_EDIT_COMMIT,
                SYSTEMS_TRANSLATION_EDIT_MAP,
                SYSTEMS_TRANSLATION_EDIT_UNMAP,
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
            transitions.contains(&CompilerSystemsTransition::MapKernelFrames {
                request: CompilerKernelMappingRequest::initial_read_write(),
            })
        );
        assert!(
            transitions.contains(&CompilerSystemsTransition::StoreKernelMappingByte {
                offset_bytes: 0,
                value: 165,
            })
        );
        assert!(
            transitions.contains(&CompilerSystemsTransition::LoadKernelMappingByte {
                offset_bytes: 0,
                value: 165,
            })
        );
        assert!(transitions.contains(&CompilerSystemsTransition::UnmapKernelFrames));
        assert!(
            transitions.contains(&CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_MEMORY_OK".into(),
            })
        );
    }

    #[test]
    fn requires_the_sealed_bounded_pci_configuration_observation() {
        // TOPAL-SYSTEMS-PCI-CONFIG-001, TOPAL-SYSTEMS-VIRTIO-PCI-DISCOVERY-001.
        for source in [
            memory_source().replace("expected-value is 272898804", "expected-value is 0"),
            memory_source().replace(
                "register-offset-bytes is 0,\n                                                                    expected-value",
                "register-offset-bytes is 2,\n                                                                    expected-value",
            ),
            memory_source().replacen("device is 5", "device is 32", 1),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-PCI-CONFIG");
        }
    }

    #[test]
    fn requires_the_complete_ordered_virtio_pci_capability_discovery() {
        // TOPAL-SYSTEMS-VIRTIO-PCI-DISCOVERY-001.
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let pci_indices = program
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(
                    operation,
                    CompilerSystemsOperation::ObservePciConfigurationDword { .. }
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(pci_indices.len(), 24);

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(pci_indices[1]);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-PCI-CONFIG"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations[pci_indices[1]] =
            duplicated.bootstrap.handler.operations[pci_indices[0]].clone();
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-PCI-CONFIG"
        );

        let mut reordered = program;
        reordered
            .bootstrap
            .handler
            .operations
            .swap(pci_indices[1], pci_indices[2]);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-PCI-CONFIG"
        );
    }

    #[test]
    fn requires_the_complete_ordered_virtio_pci_transport_observation() {
        // TOPAL-SYSTEMS-PCI-REGION-001, TOPAL-SYSTEMS-VIRTIO-PCI-NEGOTIATION-001,
        // TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-DISCOVERY-001.
        let mut malformed = CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[0];
        malformed.register_offset_bytes = 1;
        malformed.width_bytes = 4;
        assert!(!malformed.is_well_formed());
        malformed = CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[0];
        malformed.region_length_bytes = 20;
        assert!(!malformed.is_well_formed());
        malformed = CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[0];
        malformed.value = 65_536;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let indices = program
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(
                    operation,
                    CompilerSystemsOperation::AccessPciRegionRegister { .. }
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(indices.len(), 27);

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(indices[17]);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-PCI-REGION"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations[indices[17]] =
            duplicated.bootstrap.handler.operations[indices[16]].clone();
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-PCI-REGION"
        );

        let mut reordered = program;
        reordered
            .bootstrap
            .handler
            .operations
            .swap(indices[17], indices[18]);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-PCI-REGION"
        );
    }

    #[test]
    fn requires_the_exact_pci_bus_master_authorization() {
        // TOPAL-SYSTEMS-PCI-CONFIG-UPDATE-001.
        let mut malformed =
            CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
        malformed.mask = 0;
        assert!(!malformed.is_well_formed());
        malformed = CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
        malformed.expected_masked_value = 1;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::UpdatePciConfigurationWord { .. }
                )
            })
            .unwrap();

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        missing.bootstrap.handler.operations.retain(|operation| {
            !is_queue_finalization(operation)
                && !matches!(
                    operation,
                    CompilerSystemsOperation::CreateDmaBuffer { .. }
                        | CompilerSystemsOperation::BindDmaBufferReference { .. }
                        | CompilerSystemsOperation::StoreDmaBufferInteger { .. }
                        | CompilerSystemsOperation::PrepareDmaBufferForDevice { .. }
                        | CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
                )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD
                && *effect != SYSTEMS_DMA_BUFFER_CREATE
                && *effect != SYSTEMS_DMA_BUFFER_BIND_REFERENCE
                && *effect != SYSTEMS_DMA_BUFFER_STORE_INTEGER
                && *effect != SYSTEMS_DMA_BUFFER_PREPARE_DEVICE
                && *effect != SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
                && *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
                && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
                && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-PCI-CONFIG-UPDATE"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::UpdatePciConfigurationWord { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.desired_masked_value = 0;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-PCI-CONFIG-UPDATE"
        );
    }

    #[test]
    fn requires_the_exact_cpu_owned_dma_buffers() {
        // TOPAL-SYSTEMS-DMA-BUFFER-001.
        let mut malformed = CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[0];
        malformed.alignment_bytes = 0;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let indices = program
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(operation, CompilerSystemsOperation::CreateDmaBuffer { .. })
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(indices.len(), 2);

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(indices[1]);
        missing.bootstrap.handler.operations.retain(|operation| {
            !is_queue_finalization(operation)
                && !matches!(
                    operation,
                    CompilerSystemsOperation::BindDmaBufferReference { .. }
                        | CompilerSystemsOperation::StoreDmaBufferInteger { .. }
                        | CompilerSystemsOperation::PrepareDmaBufferForDevice { .. }
                        | CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
                )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_DMA_BUFFER_BIND_REFERENCE
                && *effect != SYSTEMS_DMA_BUFFER_STORE_INTEGER
                && *effect != SYSTEMS_DMA_BUFFER_PREPARE_DEVICE
                && *effect != SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
                && *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
                && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
                && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            indices[1],
            duplicated.bootstrap.handler.operations[indices[0]].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-DMA-BUFFER"
        );

        let mut reordered = program;
        reordered
            .bootstrap
            .handler
            .operations
            .swap(indices[0], indices[1]);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-DMA-BUFFER"
        );
    }

    #[test]
    fn requires_the_exact_cpu_owned_split_ring_construction() {
        // TOPAL-SYSTEMS-DMA-CONSTRUCT-001.
        let mut malformed =
            CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[0];
        malformed.width_bytes = 3;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let bind_index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::BindDmaBufferReference { .. }
                )
            })
            .unwrap();
        let store_indices = program
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(
                    operation,
                    CompilerSystemsOperation::StoreDmaBufferInteger { .. }
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(store_indices.len(), 4);

        let mut missing = program.clone();
        missing
            .bootstrap
            .handler
            .operations
            .remove(store_indices[3]);
        missing.bootstrap.handler.operations.retain(|operation| {
            !is_queue_finalization(operation)
                && !matches!(
                    operation,
                    CompilerSystemsOperation::PrepareDmaBufferForDevice { .. }
                        | CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
                )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_DMA_BUFFER_PREPARE_DEVICE
                && *effect != SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
                && *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
                && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
                && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            store_indices[1],
            duplicated.bootstrap.handler.operations[store_indices[0]].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-DMA-CONSTRUCT"
        );

        let mut reordered = program;
        reordered
            .bootstrap
            .handler
            .operations
            .swap(bind_index, store_indices[0]);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-DMA-CONSTRUCT"
        );
    }

    #[test]
    fn requires_the_exact_dma_device_preparation_sequence() {
        // TOPAL-SYSTEMS-DMA-PREPARE-001.
        let mut malformed = CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0];
        malformed.buffer_identity = 0;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let indices = program
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(
                    operation,
                    CompilerSystemsOperation::PrepareDmaBufferForDevice { .. }
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(indices.len(), 2);

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(indices[1]);
        missing.bootstrap.handler.operations.retain(|operation| {
            !is_queue_finalization(operation)
                && !matches!(
                    operation,
                    CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
                )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
                && *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
                && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
                && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            indices[1],
            duplicated.bootstrap.handler.operations[indices[0]].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-DMA-PREPARE"
        );

        let mut reordered = program;
        reordered
            .bootstrap
            .handler
            .operations
            .swap(indices[0], indices[1]);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-DMA-PREPARE"
        );
    }

    #[test]
    fn requires_the_exact_virtio_queue_address_publication_sequence() {
        // TOPAL-SYSTEMS-DMA-PUBLISH-001.
        let mut malformed =
            CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[0];
        malformed.width_bytes = 4;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let indices = program
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(
                    operation,
                    CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(indices.len(), 3);

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(indices[2]);
        missing
            .bootstrap
            .handler
            .operations
            .retain(|operation| !is_queue_finalization(operation));
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
                && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
                && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            indices[1],
            duplicated.bootstrap.handler.operations[indices[0]].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-DMA-PUBLISH"
        );

        let mut reordered = program;
        reordered
            .bootstrap
            .handler
            .operations
            .swap(indices[0], indices[1]);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-DMA-PUBLISH"
        );
    }

    #[test]
    fn requires_the_exact_virtio_queue_activation_sequence() {
        // TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-ACTIVATE-001.
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let indices = program
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(
                    operation,
                    CompilerSystemsOperation::AccessPciRegionRegister { request }
                        if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                            .contains(request)
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(indices.len(), 4);

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(indices[3]);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::AccessPciRegionRegister { request }
                    if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                        .contains(request)
                        || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                            .contains(request)
            ) && !matches!(
                operation,
                CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
                    | CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
                && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
                && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            indices[1],
            duplicated.bootstrap.handler.operations[indices[0]].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-PCI-REGION"
        );

        let mut reordered = program;
        reordered
            .bootstrap
            .handler
            .operations
            .swap(indices[0], indices[1]);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-PCI-REGION"
        );
    }

    #[test]
    fn requires_the_exact_virtio_queue_notification() {
        // TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-NOTIFY-001.
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::AccessPciRegionRegister { request }
                        if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                            .contains(request)
                )
            })
            .unwrap();

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::AccessPciRegionRegister { request }
                    if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                        .contains(request)
            ) && !matches!(
                operation,
                CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
                    | CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
                && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
                && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-PCI-REGION"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::AccessPciRegionRegister { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.value = 1;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-PCI-REGION"
        );
    }

    #[test]
    fn requires_the_exact_virtio_completion_signal() {
        // TOPAL-SYSTEMS-VIRTIO-PCI-COMPLETION-SIGNAL-001.
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::AccessPciRegionRegister { request }
                        if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                            .contains(request)
                )
            })
            .unwrap();

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
                    | CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
                && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
                && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-PCI-REGION"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::AccessPciRegionRegister { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.value = 0;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-PCI-REGION"
        );
    }

    #[test]
    fn requires_the_exact_virtio_used_ring_validation() {
        // TOPAL-SYSTEMS-DMA-COMPLETE-001, TOPAL-SYSTEMS-VIRTIO-PCI-USED-RING-001.
        let mut malformed =
            CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[0];
        malformed.width_bytes = 3;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let complete_index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
                )
            })
            .unwrap();
        let read_indices = program
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(
                    operation,
                    CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(read_indices.len(), 4);

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(read_indices[3]);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::CompleteDmaBufferForCpu { request }
                    if *request == CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0]
            ) && !matches!(
                operation,
                CompilerSystemsOperation::CaptureEntropySample { .. }
            )
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            read_indices[1],
            duplicated.bootstrap.handler.operations[read_indices[0]].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-DMA-USED-RING"
        );

        let mut reordered = program.clone();
        reordered
            .bootstrap
            .handler
            .operations
            .swap(read_indices[0], read_indices[1]);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-DMA-USED-RING"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::CompleteDmaBufferForCpu { request } =
            &mut substituted.bootstrap.handler.operations[complete_index]
        {
            *request = CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0];
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-DMA-COMPLETE"
        );
    }

    #[test]
    fn requires_the_exact_virtio_payload_completion() {
        // TOPAL-SYSTEMS-DMA-COMPLETE-001,
        // TOPAL-SYSTEMS-VIRTIO-PCI-PAYLOAD-COMPLETE-001.
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let indices = program
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(indices.len(), 2);

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(indices[1]);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::CaptureEntropySample { .. }
            )
        });
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            indices[1],
            duplicated.bootstrap.handler.operations[indices[1]].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-DMA-COMPLETE"
        );

        let mut reordered = program.clone();
        let last_read = reordered
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::LoadDmaBufferIntegerEquals { request }
                        if *request == CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[3]
                )
            })
            .unwrap();
        reordered
            .bootstrap
            .handler
            .operations
            .swap(last_read, indices[1]);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-DMA-COMPLETE"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::CompleteDmaBufferForCpu { request } =
            &mut substituted.bootstrap.handler.operations[indices[1]]
        {
            *request = CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1];
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-DMA-COMPLETE"
        );
    }

    #[test]
    fn requires_the_exact_uncredited_entropy_sample_capture() {
        // TOPAL-SYSTEMS-ENTROPY-SAMPLE-001,
        // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-CAPTURE-001.
        let mut malformed = CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG;
        malformed.sample_identity = 0;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CaptureEntropySample { .. }
                )
            })
            .unwrap();

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        remove_entropy_sample_capture(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-SAMPLE"
        );

        let mut reordered = program.clone();
        let payload_completion = reordered
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CompleteDmaBufferForCpu { request }
                        if *request == CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0]
                )
            })
            .unwrap();
        reordered
            .bootstrap
            .handler
            .operations
            .swap(payload_completion, index);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-SAMPLE"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::CaptureEntropySample { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.byte_count = 31;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-SAMPLE"
        );
    }

    #[test]
    fn requires_the_exact_trusted_entropy_sample_quality() {
        // TOPAL-SYSTEMS-ENTROPY-QUALITY-001,
        // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-QUALITY-001.
        let mut malformed = CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG;
        malformed.quality_parts_per_1024 = 1023;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::AssessEntropySampleQuality { .. }
                )
            })
            .unwrap();

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::CreateEntropyPool { .. }
                    | CompilerSystemsOperation::MixEntropyPoolSample { .. }
                    | CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
                    | CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                    | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                    | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                    | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                    | CompilerSystemsOperation::BindReusableUserRandom { .. }
                    | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                    | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY
                && *effect != SYSTEMS_ENTROPY_POOL_CREATE
                && *effect != SYSTEMS_ENTROPY_POOL_MIX_SAMPLE
                && *effect != SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION
                && *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
                && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
                && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
                && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
        });
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-QUALITY"
        );

        let mut reordered = program.clone();
        let capture = reordered
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CaptureEntropySample { .. }
                )
            })
            .unwrap();
        reordered.bootstrap.handler.operations.swap(capture, index);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-QUALITY"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::AssessEntropySampleQuality { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.quality_parts_per_1024 = 512;
            request.credited_entropy_bits = 128;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-QUALITY"
        );
    }

    #[test]
    fn requires_the_exact_empty_entropy_pool() {
        // TOPAL-SYSTEMS-ENTROPY-POOL-001,
        // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-POOL-001.
        let mut malformed = CompilerEntropyPoolRequest::INITIAL_Q35;
        malformed.initial_credited_entropy_bits = 1;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CreateEntropyPool { .. }
                )
            })
            .unwrap();

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::MixEntropyPoolSample { .. }
                    | CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
                    | CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                    | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                    | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                    | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                    | CompilerSystemsOperation::BindReusableUserRandom { .. }
                    | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                    | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_ENTROPY_POOL_CREATE
                && *effect != SYSTEMS_ENTROPY_POOL_MIX_SAMPLE
                && *effect != SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION
                && *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
                && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
                && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
                && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
        });
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-POOL"
        );

        let mut reordered = program.clone();
        let assessment = reordered
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::AssessEntropySampleQuality { .. }
                )
            })
            .unwrap();
        reordered
            .bootstrap
            .handler
            .operations
            .swap(assessment, index);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-POOL"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::CreateEntropyPool { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.credit_capacity_bits = 128;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-POOL"
        );
    }

    #[test]
    fn requires_the_exact_first_entropy_pool_mix() {
        // TOPAL-SYSTEMS-ENTROPY-MIX-001,
        // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-MIX-001.
        let mut malformed = CompilerEntropyPoolMixRequest::INITIAL_Q35;
        malformed.byte_count = 31;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::MixEntropyPoolSample { .. }
                )
            })
            .unwrap();

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
                    | CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                    | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                    | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                    | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                    | CompilerSystemsOperation::BindReusableUserRandom { .. }
                    | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                    | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_ENTROPY_POOL_MIX_SAMPLE
                && *effect != SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION
                && *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
                && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
                && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
                && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
        });
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-MIX"
        );

        let mut reordered = program.clone();
        let pool = reordered
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CreateEntropyPool { .. }
                )
            })
            .unwrap();
        reordered.bootstrap.handler.operations.swap(pool, index);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-MIX"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::MixEntropyPoolSample { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.conditioner_identity = 2;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-MIX"
        );
    }

    #[test]
    fn requires_the_exact_first_entropy_credit_transfer() {
        // TOPAL-SYSTEMS-ENTROPY-CREDIT-001,
        // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-CREDIT-001.
        let mut malformed = CompilerEntropyPoolCreditRequest::INITIAL_Q35;
        malformed.resulting_credited_entropy_bits = 255;
        assert!(!malformed.is_well_formed());

        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
                )
            })
            .unwrap();

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                    | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                    | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                    | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                    | CompilerSystemsOperation::BindReusableUserRandom { .. }
                    | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                    | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION
                && *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
                && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
                && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
                && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
        });
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-CREDIT"
        );

        let mut reordered = program.clone();
        let mixing = reordered
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::MixEntropyPoolSample { .. }
                )
            })
            .unwrap();
        reordered.bootstrap.handler.operations.swap(mixing, index);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-CREDIT"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::CreditEntropyPoolContribution { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.conditioner_origin_identity = 2;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-CREDIT"
        );
    }

    #[test]
    fn requires_the_exact_initial_entropy_generator() {
        // TOPAL-SYSTEMS-ENTROPY-GENERATOR-001,
        // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-GENERATOR-001.
        let mut malformed = CompilerEntropyGeneratorRequest::INITIAL_Q35;
        malformed.key_byte_count = 31;
        assert!(!malformed.is_well_formed());
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                )
            })
            .unwrap();
        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                    | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                    | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                    | CompilerSystemsOperation::BindReusableUserRandom { .. }
                    | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                    | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
                && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
                && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
                && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
        });
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );
        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-GENERATOR"
        );
        let mut reordered = program.clone();
        let credit = reordered
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
                )
            })
            .unwrap();
        reordered.bootstrap.handler.operations.swap(credit, index);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-GENERATOR"
        );
        let mut substituted = program;
        if let CompilerSystemsOperation::InitializeEntropyGenerator { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.algorithm_identity = 2;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-GENERATOR"
        );
    }

    #[test]
    fn requires_the_exact_first_entropy_generator_extraction() {
        // TOPAL-SYSTEMS-ENTROPY-EXTRACT-001,
        // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-EXTRACT-001.
        let mut malformed = CompilerEntropyGeneratorExtractRequest::INITIAL_Q35;
        malformed.block_byte_count = 63;
        assert!(!malformed.is_well_formed());
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let index = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                )
            })
            .unwrap();
        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(index);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                    | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                    | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                    | CompilerSystemsOperation::BindReusableUserRandom { .. }
                    | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                    | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
                && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
                && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
        });
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );
        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            index,
            duplicated.bootstrap.handler.operations[index].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-EXTRACT"
        );
        let mut reordered = program.clone();
        let initialization = reordered
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                )
            })
            .unwrap();
        reordered
            .bootstrap
            .handler
            .operations
            .swap(initialization, index);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-EXTRACT"
        );
        let mut substituted = program;
        if let CompilerSystemsOperation::ExtractEntropyGenerator { request } =
            &mut substituted.bootstrap.handler.operations[index]
        {
            request.counter = 1;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-EXTRACT"
        );
    }

    #[test]
    fn requires_affine_initial_auxv_random_delivery_before_user_entry() {
        // TOPAL-SYSTEMS-ENTROPY-DELIVER-001,
        // TOPAL-SYSTEMS-USER-AUXV-RANDOM-001.
        let mut malformed = CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM;
        malformed.delivered_byte_count = 15;
        assert!(!malformed.is_well_formed());
        let program = analyze_systems_for_compiler(
            &memory_source(),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let delivery = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                        | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                )
            })
            .unwrap();
        let entry = program
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, CompilerSystemsOperation::EnterUserContext))
            .unwrap();
        assert!(delivery < entry);

        let mut missing = program.clone();
        missing.bootstrap.handler.operations.remove(delivery);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                CompilerSystemsOperation::ExtractEntropyGenerator { request }
                    if *request == CompilerEntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM
            ) && !matches!(
                operation,
                CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                    | CompilerSystemsOperation::BindReusableUserRandom { .. }
                    | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                    | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
            )
        });
        missing.bootstrap.handler.effects.retain(|effect| {
            *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
                && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
                && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
        });
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicated = program.clone();
        duplicated.bootstrap.handler.operations.insert(
            delivery,
            duplicated.bootstrap.handler.operations[delivery].clone(),
        );
        assert_eq!(
            validate_systems_program(&duplicated).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-DELIVER"
        );

        let mut reordered = program.clone();
        reordered.bootstrap.handler.operations.swap(delivery, entry);
        assert_eq!(
            validate_systems_program(&reordered).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut substituted = program;
        if let CompilerSystemsOperation::DeliverInitialAuxvRandom { request } =
            &mut substituted.bootstrap.handler.operations[delivery]
        {
            request.auxiliary_tag = 24;
        } else {
            unreachable!();
        }
        assert_eq!(
            validate_systems_program(&substituted).unwrap_err().code,
            "E-SYSTEMS-ENTROPY-DELIVER"
        );
    }

    #[test]
    fn rejects_unbounded_or_non_affine_bootstrap_region_forms() {
        // TOPAL-SYSTEMS-AUTHORITY-001, TOPAL-SYSTEMS-STORAGE-001.
        for (source, code, expected) in [
            (
                memory_source().replace(
                    "region byte store (offset-bytes is 0, value is 90)",
                    "region byte store (offset-bytes is 64, value is 90)",
                ),
                "E-SYSTEMS-STORAGE",
                "outside the static region size",
            ),
            (
                memory_source().replace(
                    "region byte store (offset-bytes is 0, value is 90)",
                    "region byte store (offset-bytes is 0, value is 256)",
                ),
                "E-SYSTEMS-STORAGE",
                "from 0 through 255",
            ),
            (
                memory_source().replacen(
                    "restored bootstrap release region",
                    "restored console write \"not released\"",
                    1,
                ),
                "E-SYSTEMS-STORAGE",
                "release requires",
            ),
            (
                memory_source().replace(
                    "restored fatal \"toolchain gate allocation failed\"",
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
    fn requires_the_sealed_affine_atomic_word_lifecycle() {
        // TOPAL-SYSTEMS-ATOMIC-001, TOPAL-SYSTEMS-ORDER-001,
        // TOPAL-SYSTEMS-QUALIFY-001.
        for (source, expected) in [
            (
                memory_source().replace("domain is cpu-shared", "domain is device"),
                "requires `cpu-shared`",
            ),
            (
                memory_source().replace("offset-bytes is 8", "offset-bytes is 9"),
                "requires aligned offset 8",
            ),
            (
                memory_source().replace(
                    "success-order is acquire-release",
                    "success-order is release",
                ),
                "initial compare/exchange requires",
            ),
            (
                memory_source().replace("failure-order is acquire", "failure-order is release"),
                "initial compare/exchange requires",
            ),
            (
                memory_source().replacen("region is atomic end", "wrong-region is atomic end", 1),
                "restore the original region binding",
            ),
            (
                memory_source().replace("Observed actual then", "Exchanged actual then"),
                "`Exchanged`, `Observed` order",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-ATOMIC");
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
            "restored console write \"TOPAL_KERNEL_BOOT\"",
            "saved is translated",
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

        let unknown = SOURCE.replace("restored debug break", "restored machine instruction");
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
                    "restored console write \"TOPAL_KERNEL_MEMORY_DESCRIBED\"",
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
                SOURCE.replacen("frame-count is 1", "frame-count is 4", 1),
                "E-SYSTEMS-FRAMES",
                "only one, two, three, five, 34, or exactly 503 frames",
            ),
            (
                SOURCE.replace("frame-count is 503", "frame-count is 502"),
                "E-SYSTEMS-FRAMES",
                "only one, two, three, five, 34, or exactly 503 frames",
            ),
            (
                SOURCE.replace("frame-count is 503", "frame-count is 504"),
                "E-SYSTEMS-FRAMES",
                "only one, two, three, five, 34, or exactly 503 frames",
            ),
            (
                SOURCE.replacen("frame-count is 1", "frame-count is 3", 1),
                "E-SYSTEMS-FRAMES",
                "initial kernel-mapping probe requires exactly one frame",
            ),
            (
                SOURCE.replace("alignment-frames is 1", "alignment-frames is 2"),
                "E-SYSTEMS-FRAMES",
                "unit alignment",
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
    fn requires_exact_static_elf_mapping_resource_ownership() {
        // TOPAL-SYSTEMS-STATIC-ELF-MAPPING-RESOURCES-001.
        let metadata_request = "frame-count is 1,\n                                                                        alignment-frames is 1\n                                                                      )\n                                                                        Ok static-elf-leaf-table-frame";
        let metadata_request_two = "frame-count is 2,\n                                                                        alignment-frames is 1\n                                                                      )\n                                                                        Ok static-elf-leaf-table-frame";
        for (source, code, expected) in [
            (
                SOURCE.replace(metadata_request, metadata_request_two),
                "E-SYSTEMS-TRANSLATION-EDIT",
                "exactly one frame",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_OK",
                    "TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_BAD",
                ),
                "E-SYSTEMS-TRANSLATION-EDIT",
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "unmapped frames release returned-static-elf-metadata",
                    "unmapped frames release other-static-elf-metadata",
                ),
                "E-SYSTEMS-FRAMES",
                "affine extent",
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
    fn requires_exact_static_elf_initial_leaf_construction() {
        // TOPAL-SYSTEMS-STATIC-ELF-LEAVES-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "layout is bounded-static-elf-initial-rw-nx",
                    "layout is other-static-elf-layout",
                ),
                "exact payload, metadata, and initial RW/NX layout",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_OK",
                    "TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replacen("resource is metadata", "resource is payload", 1),
                "exact construction and role",
            ),
            (
                SOURCE.replace(
                    "owner is detached-static-elf-construction,\n                                                                                resource is payload",
                    "owner is other-static-elf-construction,\n                                                                                resource is payload",
                ),
                "exact construction and role",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-TRANSLATION-EDIT");
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_complete_static_elf_payload_zeroing() {
        // TOPAL-SYSTEMS-STATIC-ELF-ZERO-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is static-elf-construction,\n                                                                                extent is complete",
                    "owner is other-static-elf-construction,\n                                                                                extent is complete",
                ),
                "exact live construction and complete extent",
            ),
            (
                SOURCE.replace("extent is complete", "extent is partial"),
                "exact live construction and complete extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replacen(
                    "owner is zeroed-static-elf-construction",
                    "owner is static-elf-construction",
                    1,
                ),
                "exact zeroed construction",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-TRANSLATION-EDIT");
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_payload_copy() {
        // TOPAL-SYSTEMS-STATIC-ELF-COPY-001.
        for (source, expected) in [
            (
                SOURCE.replacen(
                    "owner is zeroed-static-elf-construction",
                    "owner is static-elf-construction",
                    1,
                ),
                "exact zeroed construction",
            ),
            (
                SOURCE.replace(
                    "source is retained-static-user-elf",
                    "source is other-resource",
                ),
                "retained static ELF",
            ),
            (
                SOURCE.replace(
                    "plan is bounded-static-elf-four-loads",
                    "plan is other-plan",
                ),
                "four-load plan",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replacen(
                    "owner is detached-static-elf-construction,\n                                                                                resource is metadata",
                    "owner is populated-static-elf-construction,\n                                                                                resource is metadata",
                    1,
                ),
                "exact construction and role",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-TRANSLATION-EDIT");
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_final_permissions() {
        // TOPAL-SYSTEMS-STATIC-ELF-PERMISSIONS-001.
        for (source, expected) in [
            (
                SOURCE.replacen(
                    "owner is populated-static-elf-construction",
                    "owner is zeroed-static-elf-construction",
                    1,
                ),
                "exact populated construction",
            ),
            (
                SOURCE.replace("plan is bounded-static-elf-wx-relro", "plan is other-plan"),
                "W^X/RELRO plan",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_OK",
                    "TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replacen(
                    "owner is detached-static-elf-construction,\n                                                                                resource is metadata",
                    "owner is populated-static-elf-construction,\n                                                                                resource is metadata",
                    1,
                ),
                "exact construction and role",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-TRANSLATION-EDIT");
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_hierarchy_extent() {
        // TOPAL-SYSTEMS-STATIC-ELF-HIERARCHY-EXTENT-001.
        for (source, code, expected) in [
            (
                SOURCE.replacen(
                    "owner is sealed-static-elf-construction,\n                                                                                frame-count is 5",
                    "owner is populated-static-elf-construction,\n                                                                                frame-count is 5",
                    1,
                ),
                "E-SYSTEMS-TRANSLATION-EDIT",
                "exact sealed construction",
            ),
            (
                SOURCE.replacen("frame-count is 5", "frame-count is 34", 1),
                "E-SYSTEMS-FRAMES",
                "frame-allocation slice",
            ),
            (
                SOURCE.replacen(
                    "owner is sealed-static-elf-construction,\n                                                                                frame-count is 5,\n                                                                                alignment-frames is 1",
                    "owner is sealed-static-elf-construction,\n                                                                                frame-count is 5,\n                                                                                alignment-frames is 2",
                    1,
                ),
                "E-SYSTEMS-TRANSLATION-EDIT",
                "five unit-aligned frames",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK",
                    "TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_BAD",
                ),
                "E-SYSTEMS-TRANSLATION-EDIT",
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "unmapped console write \"TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK\"\n                                                                              attached-static-elf-construction is unmapped static image construction attach address space",
                    "unmapped frames release static-elf-hierarchy-frames\n                                                                              attached-static-elf-construction is unmapped static image construction attach address space",
                ),
                "E-SYSTEMS-OPERATION",
                "not admitted",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(
                error.code, code,
                "{expected}: {}",
                error.message
            );
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_address_space_attachment() {
        // TOPAL-SYSTEMS-STATIC-ELF-ADDRESS-SPACE-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "layout is bounded-static-elf-inactive-address-space",
                    "layout is other-static-elf-address-space",
                ),
                "inactive layout",
            ),
            (
                SOURCE.replace(
                    "hierarchy is static-elf-hierarchy-frames",
                    "hierarchy is other-static-elf-hierarchy",
                ),
                "hierarchy extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is detached-static-elf-construction,\n                                                                                resource is hierarchy",
                    "owner is attached-static-elf-construction,\n                                                                                resource is hierarchy",
                ),
                "exact construction and role",
            ),
            (
                SOURCE.replace(
                    "unmapped frames release returned-static-elf-hierarchy",
                    "unmapped frames release static-elf-hierarchy-frames",
                ),
                "affine extent",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_address_space_activation_and_restoration() {
        // TOPAL-SYSTEMS-STATIC-ELF-ACTIVATION-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is attached-static-elf-construction,\n                                                                                policy is bounded-static-elf-kernel-continuation",
                    "owner is sealed-static-elf-construction,\n                                                                                policy is bounded-static-elf-kernel-continuation",
                ),
                "exact owner and continuation policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-static-elf-kernel-continuation",
                    "policy is unbounded-static-elf-kernel-continuation",
                ),
                "exact owner and continuation policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is active-static-elf-construction,\n                                                                                policy is bounded-bootstrap-kernel-continuation",
                    "owner is attached-static-elf-construction,\n                                                                                policy is bounded-bootstrap-kernel-continuation",
                ),
                "exact owner and continuation policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-bootstrap-kernel-continuation",
                    "policy is unbounded-bootstrap-kernel-continuation",
                ),
                "exact owner and continuation policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_entry_authorization_and_revocation() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTRY-AUTHORITY-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is restored-static-elf-construction,\n                                                                                policy is validated-static-elf-entry",
                    "owner is active-static-elf-construction,\n                                                                                policy is validated-static-elf-entry",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "policy is validated-static-elf-entry",
                    "policy is unchecked-static-elf-entry",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is stack-slot-restored-static-elf-construction,\n                                                                                policy is bounded-entry-qualification-rollback",
                    "owner is restored-static-elf-construction,\n                                                                                policy is bounded-entry-qualification-rollback",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-entry-qualification-rollback",
                    "policy is unbounded-entry-qualification-rollback",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is entry-restored-static-elf-construction\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK\"",
                    "owner is restored-static-elf-construction\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK\"",
                ),
                "exact restored construction",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_slot_authorization_and_revocation() {
        // TOPAL-SYSTEMS-STATIC-ELF-STACK-SLOT-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is entry-authorized-static-elf-construction,\n                                                                                policy is bounded-linux-initial-stack-slot",
                    "owner is restored-static-elf-construction,\n                                                                                policy is bounded-linux-initial-stack-slot",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-linux-initial-stack-slot",
                    "policy is unguarded-linux-initial-stack-slot",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is zeroed-stack-backing-restored-construction,\n                                                                                policy is bounded-stack-slot-qualification-rollback",
                    "owner is entry-authorized-static-elf-construction,\n                                                                                policy is bounded-stack-slot-qualification-rollback",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-slot-qualification-rollback",
                    "policy is unbounded-stack-slot-qualification-rollback",
                ),
                "exact owner and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is stack-slot-restored-static-elf-construction,\n                                                                                policy is bounded-entry-qualification-rollback",
                    "owner is entry-authorized-static-elf-construction,\n                                                                                policy is bounded-entry-qualification-rollback",
                ),
                "exact owner and policy",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_backing_reservation_and_release() {
        // TOPAL-SYSTEMS-STATIC-ELF-STACK-BACKING-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is stack-slot-authorized-static-elf-construction,\n                                                                                frame-count is 1,",
                    "owner is entry-authorized-static-elf-construction,\n                                                                                frame-count is 1,",
                ),
                "exact slot-authorized construction",
            ),
            (
                SOURCE.replace(
                    "static-elf-stack-backing is unmapped static image construction reserve stack backing (\n                                                                                owner is stack-slot-authorized-static-elf-construction,\n                                                                                frame-count is 1,",
                    "static-elf-stack-backing is unmapped static image construction reserve stack backing (\n                                                                                owner is stack-slot-authorized-static-elf-construction,\n                                                                                frame-count is 2,",
                ),
                "one unit-aligned frame",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "unmapped frames release static-elf-stack-backing",
                    "unmapped frames release static-elf-hierarchy-frames",
                ),
                "live allocator and affine extent",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_backing_zeroing() {
        // TOPAL-SYSTEMS-STATIC-ELF-STACK-ZERO-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is stack-slot-authorized-static-elf-construction,\n                                                                                backing is static-elf-stack-backing,",
                    "owner is entry-authorized-static-elf-construction,\n                                                                                backing is static-elf-stack-backing,",
                ),
                "exact owner, backing, and complete extent",
            ),
            (
                SOURCE.replace(
                    "backing is static-elf-stack-backing,\n                                                                                extent is complete",
                    "backing is static-elf-hierarchy-frames,\n                                                                                extent is complete",
                ),
                "exact owner, backing, and complete extent",
            ),
            (
                SOURCE.replace(
                    "extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK\"",
                    "extent is partial\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK\"",
                ),
                "exact owner, backing, and complete extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is zeroed-stack-backing-restored-construction,\n                                                                                policy is bounded-stack-slot-qualification-rollback",
                    "owner is stack-slot-authorized-static-elf-construction,\n                                                                                policy is bounded-stack-slot-qualification-rollback",
                ),
                "exact owner and policy",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_leaf_publication_and_revocation() {
        // TOPAL-SYSTEMS-STATIC-ELF-STACK-LEAF-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is zeroed-stack-backing-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                access is writable-non-executable,",
                    "owner is stack-slot-authorized-static-elf-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                access is writable-non-executable,",
                ),
                "exact owner, backing, RW/NX access, and complete extent",
            ),
            (
                SOURCE.replace(
                    "backing is static-elf-stack-backing,\n                                                                                access is writable-non-executable,",
                    "backing is static-elf-hierarchy-frames,\n                                                                                access is writable-non-executable,",
                ),
                "exact owner, backing, RW/NX access, and complete extent",
            ),
            (
                SOURCE.replace(
                    "access is writable-non-executable,\n                                                                                extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK\"",
                    "access is writable-executable,\n                                                                                extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK\"",
                ),
                "exact owner, backing, RW/NX access, and complete extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is published-stack-leaf-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-leaf-qualification-rollback",
                    "owner is zeroed-stack-backing-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-leaf-qualification-rollback",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-leaf-qualification-rollback",
                    "policy is retained-stack-leaf",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_entry_initialization_and_clearing() {
        // TOPAL-SYSTEMS-STATIC-ELF-STACK-ENTRY-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is published-stack-leaf-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                profile is bounded-static-elf-busybox-echo,",
                    "owner is zeroed-stack-backing-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                profile is bounded-static-elf-busybox-echo,",
                ),
                "exact owner, backing, sealed profile, and complete extent",
            ),
            (
                SOURCE.replace(
                    "profile is bounded-static-elf-busybox-echo,\n                                                                                extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK\"",
                    "profile is unbounded-static-elf,\n                                                                                extent is complete\n                                                                              )\n                                                                              unmapped console write \"TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK\"",
                ),
                "exact owner, backing, sealed profile, and complete extent",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is initialized-stack-entry-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-entry-qualification-rollback",
                    "owner is published-stack-leaf-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-entry-qualification-rollback",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-entry-qualification-rollback",
                    "policy is retained-stack-entry",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_pagesz_addition_and_removal() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-PAGESZ-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is initialized-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is provider-base-page,",
                    "owner is published-stack-leaf-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is provider-base-page,",
                ),
                "exact owner, backing, provider page value, and policy",
            ),
            (
                SOURCE.replace("value is provider-base-page", "value is source-page"),
                "exact owner, backing, provider page value, and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_OK",
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is pagesz-stack-entry-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-pagesz-qualification-rollback",
                    "owner is initialized-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-pagesz-qualification-rollback",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-pagesz-qualification-rollback",
                    "policy is retained-stack-pagesz",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_entry_fact_addition_and_removal() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-ENTRY-001.
        for (source, expected) in [
            (
                SOURCE.replace(
                    "owner is pagesz-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is authorized-static-elf-entry,",
                    "owner is initialized-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is authorized-static-elf-entry,",
                ),
                "exact owner, backing, authorized entry value, and policy",
            ),
            (
                SOURCE.replace(
                    "value is authorized-static-elf-entry",
                    "value is source-entry-address",
                ),
                "exact owner, backing, authorized entry value, and policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_OK",
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_BAD",
                ),
                "exact marker",
            ),
            (
                SOURCE.replace(
                    "owner is entry-auxv-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-entry-fact-qualification-rollback",
                    "owner is pagesz-stack-entry-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-entry-fact-qualification-rollback",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "policy is bounded-stack-entry-fact-qualification-rollback",
                    "policy is retained-stack-entry-fact",
                ),
                "exact owner, backing, and rollback policy",
            ),
            (
                SOURCE.replace(
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_OK",
                    "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_BAD",
                ),
                "exact marker",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(error.message.contains(expected), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_program_header_group() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-PHDR-001.
        for source in [
            SOURCE.replace(
                "value is resident-static-elf-program-headers",
                "value is source-program-headers",
            ),
            SOURCE.replace(
                "owner is program-header-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-program-headers-qualification-rollback",
                "owner is entry-auxv-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-program-headers-qualification-rollback",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("program-header") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_executable_name_fact() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-EXECFN-001.
        for source in [
            SOURCE.replace(
                "owner is program-header-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-executable-name,",
                "owner is entry-auxv-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-executable-name,",
            ),
            SOURCE.replace(
                "value is initial-executable-name",
                "value is source-executable-name",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_BAD",
            ),
            SOURCE.replace(
                "owner is executable-name-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-executable-name-qualification-rollback",
                "owner is program-header-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-executable-name-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-executable-name-qualification-rollback",
                "retained-stack-executable-name",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("executable-name")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_credential_group() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-CREDENTIALS-001.
        for source in [
            SOURCE.replace(
                "owner is executable-name-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-process-credentials,",
                "owner is program-header-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-process-credentials,",
            ),
            SOURCE.replace(
                "value is initial-process-credentials",
                "value is source-process-credentials",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_BAD",
            ),
            SOURCE.replace(
                "owner is credential-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-credentials-qualification-rollback",
                "owner is executable-name-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-credentials-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-credentials-qualification-rollback",
                "retained-stack-credentials",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("credential") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_secure_mode_fact() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-SECURE-001.
        for source in [
            SOURCE.replace(
                "owner is credential-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-execution-security,",
                "owner is executable-name-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-execution-security,",
            ),
            SOURCE.replace(
                "value is initial-execution-security",
                "value is source-execution-security",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_BAD",
            ),
            SOURCE.replace(
                "owner is secure-mode-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-secure-mode-qualification-rollback",
                "owner is credential-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-secure-mode-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-secure-mode-qualification-rollback",
                "retained-stack-secure-mode",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("secure-mode") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_clock_tick_rate_fact() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-CLKTCK-001.
        for source in [
            SOURCE.replace(
                "owner is secure-mode-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is linux-user-hz,",
                "owner is credential-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is linux-user-hz,",
            ),
            SOURCE.replace("value is linux-user-hz", "value is hardware-timer-rate"),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_BAD",
            ),
            SOURCE.replace(
                "owner is clock-tick-rate-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-clock-tick-rate-qualification-rollback",
                "owner is secure-mode-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-clock-tick-rate-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-clock-tick-rate-qualification-rollback",
                "retained-stack-clock-tick-rate",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("clock-tick-rate")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_platform_fact() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-PLATFORM-001.
        for source in [
            SOURCE.replace(
                "owner is clock-tick-rate-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is native-userspace-platform,",
                "owner is secure-mode-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is native-userspace-platform,",
            ),
            SOURCE.replace("value is native-userspace-platform", "value is source-platform-string"),
            SOURCE.replace("TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_OK", "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_BAD"),
            SOURCE.replace(
                "owner is platform-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-platform-qualification-rollback",
                "owner is clock-tick-rate-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-platform-qualification-rollback",
            ),
            SOURCE.replace("bounded-stack-platform-qualification-rollback", "retained-stack-platform"),
            SOURCE.replace("TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_OK", "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_BAD"),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("platform") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_hardware_capabilities() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-HWCAP-001.
        for source in [
            SOURCE.replace(
                "owner is platform-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is native-userspace-hardware-capabilities,",
                "owner is clock-tick-rate-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is native-userspace-hardware-capabilities,",
            ),
            SOURCE.replace(
                "value is native-userspace-hardware-capabilities",
                "value is source-hardware-capability-mask",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_BAD",
            ),
            SOURCE.replace(
                "owner is hardware-capabilities-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-hardware-capabilities-qualification-rollback",
                "owner is platform-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-hardware-capabilities-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-hardware-capabilities-qualification-rollback",
                "retained-stack-hardware-capabilities",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("hardware-capability")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_load_metadata() {
        // TOPAL-SYSTEMS-STATIC-ELF-AUXV-LOAD-METADATA-001.
        for source in [
            SOURCE.replace(
                "owner is hardware-capabilities-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is static-executable-load-metadata,",
                "owner is platform-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is static-executable-load-metadata,",
            ),
            SOURCE.replace(
                "value is static-executable-load-metadata",
                "value is source-loader-base",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_BAD",
            ),
            SOURCE.replace(
                "owner is load-metadata-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-load-metadata-qualification-rollback",
                "owner is hardware-capabilities-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-load-metadata-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-load-metadata-qualification-rollback",
                "retained-stack-load-metadata",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("load-metadata")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_stack_random_slot_authority() {
        // TOPAL-SYSTEMS-STATIC-ELF-RANDOM-SLOT-001.
        for source in [
            SOURCE.replace(
                "owner is load-metadata-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-auxv-random-object,",
                "owner is hardware-capabilities-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is initial-auxv-random-object,",
            ),
            SOURCE.replace("value is initial-auxv-random-object", "value is source-random-buffer"),
            SOURCE.replace("extent-bytes is 16", "extent-bytes is 8"),
            SOURCE.replace(
                "bounded-linux-auxv-random-before-null",
                "source-selected-random-placement",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_OK",
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_BAD",
            ),
            SOURCE.replace(
                "owner is random-slot-authorized-stack-restored-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-random-slot-qualification-rollback",
                "owner is load-metadata-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                policy is bounded-stack-random-slot-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-stack-random-slot-qualification-rollback",
                "retained-stack-random-slot",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("random-slot")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_prepared_process_retention() {
        // TOPAL-SYSTEMS-STATIC-ELF-PREPARED-RETENTION-001.
        for source in [
            SOURCE.replace(
                "owner is random-slot-authorized-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is bounded-static-elf-initial-process,",
                "owner is load-metadata-stack-construction,\n                                                                                backing is static-elf-stack-backing,\n                                                                                value is bounded-static-elf-initial-process,",
            ),
            SOURCE.replace("value is bounded-static-elf-initial-process", "value is unsealed-process"),
            SOURCE.replace("bounded-static-elf-pre-entropy-retention", "unbounded-process-retention"),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_BAD",
            ),
            SOURCE.replace(
                "owner is retained-static-process-after-entropy-rollback,\n                                                                                policy is bounded-static-elf-retention-qualification-rollback",
                "owner is random-slot-authorized-stack-construction,\n                                                                                policy is bounded-static-elf-retention-qualification-rollback",
            ),
            SOURCE.replace(
                "bounded-static-elf-retention-qualification-rollback",
                "discard-retained-process",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("prepared-process")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_retained_disjoint_work() {
        // TOPAL-SYSTEMS-STATIC-ELF-RETAINED-DISJOINT-WORK-001.
        for source in [
            SOURCE.replace(
                "owner is retained-static-process,\n                                                                                frame-count is 1,",
                "owner is random-slot-authorized-stack-construction,\n                                                                                frame-count is 1,",
            ),
            SOURCE.replace(
                "owner is retained-static-process,\n                                                                                frame-count is 1,\n                                                                                alignment-frames is 1,",
                "owner is retained-static-process,\n                                                                                frame-count is 2,\n                                                                                alignment-frames is 1,",
            ),
            SOURCE.replace(
                "bounded-static-elf-retained-disjoint-work",
                "overlapping-retained-work",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_OK",
                "TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("retained disjoint")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_handoff() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-HANDOFF-001.
        for source in [
            SOURCE.replace(
                "owner is retained-static-process-after-disjoint-work,\n                                                                                process is bounded-static-elf-initial-process,",
                "owner is retained-static-process,\n                                                                                process is bounded-static-elf-initial-process,",
            ),
            SOURCE.replace("process is bounded-static-elf-initial-process", "process is unrelated-process"),
            SOURCE.replace("bounded-static-elf-initial-random-delivery", "loader-owned-random-delivery"),
            SOURCE.replace("TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_OK", "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_BAD"),
            SOURCE.replace("bounded-static-elf-entropy-handoff-qualification-rollback", "discard-entropy-pending-process"),
            SOURCE.replace("TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_OK", "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_BAD"),
        ] {
            let error = analyze_systems_for_compiler(&source, &CompilerSystemsTargetSelection::initial_x86_64_qemu()).unwrap_err();
            assert!(error.message.contains("entropy handoff") || error.message.contains("exact marker"), "{}", error.message);
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_acquisition_authority() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-ACQUISITION-AUTHORITY-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-pending-static-process,",
                "owner is retained-static-process,",
            ),
            SOURCE.replace(
                "output is initial-entropy-output",
                "output is unrelated-output",
            ),
            SOURCE.replace(
                "generation is generation-two",
                "generation is generation-one",
            ),
            SOURCE.replace(
                "bounded-static-elf-initial-entropy-acquisition",
                "loader-owned-entropy-acquisition",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_BAD",
            ),
            SOURCE.replace(
                "bounded-static-elf-entropy-acquisition-qualification-rollback",
                "discard-entropy-acquisition",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy acquisition")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_device_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-DEVICE-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "device is initial-virtio-rng-device",
                "device is unrelated-device",
            ),
            SOURCE.replace(
                "bounded-static-elf-entropy-device-coexistence",
                "synthetic-entropy-device",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-device") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_transport_discovery() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-TRANSPORT-DISCOVERY-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-device,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "transport is initial-virtio-rng-pci-transport",
                "transport is unrelated-transport",
            ),
            SOURCE.replace(
                "bounded-static-elf-entropy-transport-discovery",
                "synthetic-entropy-transport",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-transport")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_transport_negotiation() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-TRANSPORT-NEGOTIATION-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-transport,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "transport is initial-virtio-rng-pci-transport,\n                                                                                policy is bounded-static-elf-entropy-transport-negotiation",
                "transport is unrelated-transport,\n                                                                                policy is bounded-static-elf-entropy-transport-negotiation",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-transport-negotiation",
                "policy is synthetic-entropy-negotiation",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-transport negotiation")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_queue_discovery() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-QUEUE-DISCOVERY-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-negotiation,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "transport is initial-virtio-rng-pci-transport,\n                                                                                queue is initial-virtio-rng-queue-zero,",
                "transport is unrelated-transport,\n                                                                                queue is initial-virtio-rng-queue-zero,",
            ),
            SOURCE.replace(
                "queue is initial-virtio-rng-queue-zero,",
                "queue is unrelated-queue,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-queue-discovery",
                "policy is synthetic-entropy-queue",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-queue")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_bus_master_qualification() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-BUS-MASTER-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-queue,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "device is initial-virtio-rng-device,\n                                                                                policy is bounded-static-elf-entropy-bus-master",
                "device is unrelated-device,\n                                                                                policy is bounded-static-elf-entropy-bus-master",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-bus-master",
                "policy is synthetic-entropy-bus-master",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy bus-master")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_dma_buffer_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-DMA-BUFFER-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-bus-master,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "device is initial-virtio-rng-device,\n                                                                                domain is initial-dma-domain,",
                "device is unrelated-device,\n                                                                                domain is initial-dma-domain,",
            ),
            SOURCE.replace("domain is initial-dma-domain,", "domain is unrelated-domain,"),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-dma-buffer-coexistence",
                "policy is synthetic-entropy-dma-buffers",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy DMA-buffer")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_split_ring_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-SPLIT-RING-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-dma-buffers,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "device is initial-virtio-rng-device,\n                                                                                domain is initial-dma-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,",
                "device is unrelated-device,\n                                                                                domain is initial-dma-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,",
            ),
            SOURCE.replace(
                "domain is initial-dma-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,\n                                                                                policy is bounded-static-elf-entropy-split-ring-coexistence",
                "domain is unrelated-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,\n                                                                                policy is bounded-static-elf-entropy-split-ring-coexistence",
            ),
            SOURCE.replace(
                "domain is initial-dma-domain,\n                                                                                queue is initial-virtio-rng-queue-zero,\n                                                                                policy is bounded-static-elf-entropy-split-ring-coexistence",
                "domain is initial-dma-domain,\n                                                                                queue is unrelated-queue,\n                                                                                policy is bounded-static-elf-entropy-split-ring-coexistence",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-split-ring-coexistence",
                "policy is synthetic-entropy-split-ring",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy split-ring")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_dma_preparation_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-DMA-PREPARATION-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-split-ring,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-dma-preparation-coexistence",
                "policy is synthetic-entropy-dma-preparation",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy DMA-preparation")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_queue_address_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-QUEUE-ADDRESS-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-dma-preparation,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "transport is initial-virtio-rng-pci-transport,\n                                                                                policy is bounded-static-elf-entropy-queue-address-coexistence",
                "transport is unrelated-transport,\n                                                                                policy is bounded-static-elf-entropy-queue-address-coexistence",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-queue-address-coexistence",
                "policy is synthetic-entropy-queue-addresses",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy queue-address")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_queue_activation_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-QUEUE-ACTIVATION-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-queue-addresses,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-empty-queue-activation-coexistence",
                "policy is synthetic-entropy-queue-activation",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy queue-activation")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_request_completion_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-REQUEST-COMPLETION-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-queue-activation,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-request-completion-coexistence",
                "policy is synthetic-entropy-request-completion",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy request-completion")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_sample_capture_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-SAMPLE-CAPTURE-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-request-completion,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "sample is initial-virtio-rng-sample,",
                "sample is synthetic-entropy-sample,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-sample-capture-coexistence",
                "policy is synthetic-entropy-sample-capture",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-sample capture")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_sample_quality_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-SAMPLE-QUALITY-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-sample-capture,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "evidence is initial-q35-virtio-rng-trust-evidence,",
                "evidence is synthetic-trust-evidence,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-sample-quality-coexistence",
                "policy is synthetic-entropy-sample-quality",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-sample quality")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_pool_lifecycle_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-POOL-LIFECYCLE-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-sample-quality,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "pool is initial-entropy-pool,",
                "pool is synthetic-entropy-pool,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-pool-lifecycle-coexistence",
                "policy is synthetic-entropy-pool-lifecycle",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-pool lifecycle")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_entropy_generator_lifecycle_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-GENERATOR-LIFECYCLE-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-pool-lifecycle,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "generator is initial-entropy-generator,",
                "generator is synthetic-entropy-generator,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-entropy-generator-lifecycle-coexistence",
                "policy is synthetic-entropy-generator-lifecycle",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("entropy-generator lifecycle")
                    || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_initial_random_delivery_coexistence() {
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-RANDOM-DELIVERY-COEXISTENCE-001.
        for source in [
            SOURCE.replace(
                "owner is entropy-acquiring-static-process-after-generator-lifecycle,",
                "owner is entropy-pending-static-process,",
            ),
            SOURCE.replace(
                "delivery is initial-auxv-random-delivery,",
                "delivery is synthetic-random-delivery,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-initial-random-delivery-coexistence",
                "policy is synthetic-initial-random-delivery",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("initial-random") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn requires_exact_static_elf_process_entry() {
        // TOPAL-SYSTEMS-STATIC-ELF-PROCESS-ENTRY-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-BRK-QUERY-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-BRK-GROWTH-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-X64-FS-BASE-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-CLEAR-TID-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-EMPTY-ROBUST-LIST-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-RSEQ-FALLBACK-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-STACK-LIMIT-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-READLINKAT-FALLBACK-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-GETRANDOM-001,
        // TOPAL-SYSTEMS-STATIC-ELF-REUSABLE-GETRANDOM-001,
        // TOPAL-SYSTEMS-STATIC-ELF-POST-RANDOM-BRK-QUERY-001,
        // TOPAL-SYSTEMS-STATIC-ELF-POST-RANDOM-BRK-GROWTH-001,
        // TOPAL-SYSTEMS-STATIC-ELF-POST-RANDOM-BRK-PAGE-ALIGNMENT-001,
        // TOPAL-SYSTEMS-STATIC-ELF-RELRO-MPROTECT-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-PROCESS-NAME-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-GETUID-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-CONFIG-STAT-MISS-001,
        // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-CREDENTIAL-TAIL-001.
        for source in [
            SOURCE.replace("abi is linux-x86-64-process,", "abi is synthetic-process,"),
            SOURCE.replace(
                "startup is initial-brk-query-growth-x64-fs-base-clear-tid-empty-robust-list-rseq-fallback-stack-limit-readlinkat-fallback-getrandom-repeated-brk-query-growth-page-alignment-relro-mprotect-process-name-getuid-config-stat-miss-credential-tail-applet-newline-write-shell-getpid,",
                "startup is synthetic-startup,",
            ),
            SOURCE.replace(
                "terminal is exit-group-zero,",
                "terminal is exit-group-one,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-shell-getpid",
                "policy is synthetic-process-entry",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_GROWTH_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_GROWTH_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_X64_FS_BASE_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_X64_FS_BASE_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CLEAR_TID_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CLEAR_TID_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_EMPTY_ROBUST_LIST_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_EMPTY_ROBUST_LIST_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_RSEQ_FALLBACK_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_RSEQ_FALLBACK_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_STACK_LIMIT_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_STACK_LIMIT_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_READLINKAT_FALLBACK_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_READLINKAT_FALLBACK_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETRANDOM_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETRANDOM_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_REUSABLE_GETRANDOM_OK",
                "TOPAL_KERNEL_STATIC_ELF_REUSABLE_GETRANDOM_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_QUERY_OK",
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_QUERY_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_GROWTH_OK",
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_GROWTH_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_PAGE_ALIGNMENT_OK",
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_PAGE_ALIGNMENT_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_RELRO_MPROTECT_OK",
                "TOPAL_KERNEL_STATIC_ELF_RELRO_MPROTECT_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_PROCESS_NAME_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_PROCESS_NAME_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETUID_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETUID_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CONFIG_STAT_PROFILE_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CONFIG_STAT_PROFILE_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CREDENTIAL_TAIL_PROFILE_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CREDENTIAL_TAIL_PROFILE_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_PROCESS_EXITED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PROCESS_EXITED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("process entry") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
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
            "  deadline-notification is lang systems external-interrupt-entry deadline-notification-handler,\n",
            "  deadline-notification is lang systems external-interrupt-entry deadline-notification-handler,\n  interrupt is lang systems interrupt-entry interrupt-handler,\n",
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

    include!("systems_mapping_tests.rs");
    include!("systems_translation_tests.rs");
    include!("systems_translation_edit_tests.rs");
    include!("systems_critical_tests.rs");
    include!("systems_interrupt_tests.rs");
    include!("systems_clock_tests.rs");
    include!("systems_deadline_tests.rs");
    include!("systems_context_tests.rs");
}
