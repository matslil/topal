//! Checked source model for the initial freestanding systems profile.
//!
//! This module deliberately stops before target lowering. It establishes the
//! sealed source vocabulary and abstract transitions which a later qualified
//! provider may consume without reinterpreting syntax.

use std::collections::BTreeMap;
pub use topal_semantics::SYSTEMS_REGION_LIBRARY_APPLY;

#[path = "systems_library.rs"]
mod source_library;

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
    SystemsSourceLibraryInterface as CompilerSystemsSourceLibraryInterface,
    SystemsSourceLibraryRequirement as CompilerSystemsSourceLibraryRequirement,
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
    // Root selection and invocation are one structural source contract. Keep
    // this check here as well as in semantic validation: callers of the shared
    // frontend must not receive an unbound operation for later linking.
    let region_calls = bootstrap
        .operations
        .iter()
        .filter(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::ApplyBootstrapRegionLibrary
            )
        })
        .count();
    if region_calls != usize::from(root_entries.region_library.is_some()) {
        return Err(source_diagnostic(
            &source,
            "E-SYSTEMS-REGION-ROOT",
            statement_span(root),
            "region-library requires exactly one bootstrap invocation",
        ));
    }
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
        kernel_library: root_entries.kernel_library,
        region_library: root_entries.region_library,
        file_position_library: root_entries.file_position_library,
        time_library: root_entries.time_library,
        image_library: root_entries.image_library,
        dispatch_library: root_entries.dispatch_library,
        process_identity_library: root_entries.process_identity_library,
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
    kernel_library: Option<CompilerSystemsSourceLibraryRequirement>,
    region_library: Option<CompilerSystemsSourceLibraryRequirement>,
    file_position_library: Option<CompilerSystemsSourceLibraryRequirement>,
    time_library: Option<CompilerSystemsSourceLibraryRequirement>,
    image_library: Option<CompilerSystemsSourceLibraryRequirement>,
    dispatch_library: Option<CompilerSystemsSourceLibraryRequirement>,
    process_identity_library: Option<CompilerSystemsSourceLibraryRequirement>,
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
    let mut kernel_library = None;
    let mut region_library = None;
    let mut file_position_library = None;
    let mut time_library = None;
    let mut image_library = None;
    let mut dispatch_library = None;
    let mut process_identity_library = None;
    let mut entries = BTreeMap::new();
    for field in fields {
        let label = artifact_field_label(source, field)?;
        let label_text = source.slice(label);
        if label_text == "file-position-library" {
            let requirement = source_library::parse_requirement(source, field)?;
            if !matches!(
                requirement.interface,
                CompilerSystemsSourceLibraryInterface::BorrowedByteRegionActionV1
                    | CompilerSystemsSourceLibraryInterface::BorrowedByteRegionFramedActionV1
                    | CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
            ) || file_position_library.replace(requirement).is_some()
            {
                return Err(source_diagnostic(
                    source,
                    "E-SYSTEMS-REGION-ACTION",
                    label,
                    "file-position-library requires one qualified region action entry",
                ));
            }
            continue;
        } else if label_text == "region-library" {
            let requirement = source_library::parse_requirement(source, field)?;
            if !matches!(
                requirement.interface,
                CompilerSystemsSourceLibraryInterface::BorrowedByteRegionV1
                    | CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
            ) || region_library.replace(requirement).is_some()
            {
                return Err(source_diagnostic(
                    source,
                    "E-SYSTEMS-REGION-ROOT",
                    label,
                    "region-library requires one qualified Unit or instance-action entry",
                ));
            }
            continue;
        } else if label_text == "process-identity-library" {
            let requirement = source_library::parse_requirement(source, field)?;
            if !matches!(
                requirement.interface,
                CompilerSystemsSourceLibraryInterface::ProcessIdentityTransitionV1
                    | CompilerSystemsSourceLibraryInterface::RetainedProcessIdentityV1
                    | CompilerSystemsSourceLibraryInterface::StructuredProcessIdentityV1
            ) {
                return Err(source_diagnostic(
                    source,
                    "E-SYSTEMS-LIBRARY",
                    label,
                    "process-identity-library requires the process identity transition interface",
                ));
            }
            if process_identity_library.replace(requirement).is_some() {
                return Err(duplicate_artifact_field(source, label, label_text));
            }
        } else if label_text == "dispatch-library" {
            let requirement = source_library::parse_requirement(source, field)?;
            if !matches!(
                requirement.interface,
                CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV1
                    | CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV2
                    | CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV3
                    | CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV4
            ) {
                return Err(source_diagnostic(
                    source,
                    "E-SYSTEMS-LIBRARY",
                    label,
                    "dispatch-library requires the static Linux operation interface",
                ));
            }
            if dispatch_library.replace(requirement).is_some() {
                return Err(duplicate_artifact_field(source, label, label_text));
            }
        } else if label_text == "image-library" {
            let requirement = source_library::parse_requirement(source, field)?;
            if requirement.interface != CompilerSystemsSourceLibraryInterface::StaticElf64HeaderV1 {
                return Err(source_diagnostic(
                    source,
                    "E-SYSTEMS-LIBRARY",
                    label,
                    "image-library requires the static ELF64 header interface",
                ));
            }
            if image_library.replace(requirement).is_some() {
                return Err(duplicate_artifact_field(source, label, label_text));
            }
        } else if label_text == "time-library" {
            let requirement = source_library::parse_requirement(source, field)?;
            if requirement.interface
                != CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1
            {
                return Err(source_diagnostic(
                    source,
                    "E-SYSTEMS-LIBRARY",
                    label,
                    "time-library requires the relative sleep interface",
                ));
            }
            if time_library.replace(requirement).is_some() {
                return Err(duplicate_artifact_field(source, label, label_text));
            }
        } else if label_text == "kernel-library" {
            let requirement = source_library::parse_requirement(source, field)?;
            if !matches!(
                requirement.interface,
                CompilerSystemsSourceLibraryInterface::BootOpenDecisionV1
                    | CompilerSystemsSourceLibraryInterface::FileOpenDecisionV1
            ) {
                return Err(source_diagnostic(
                    source,
                    "E-SYSTEMS-LIBRARY",
                    label,
                    "kernel-library requires a qualified finite open interface",
                ));
            }
            if kernel_library.replace(requirement).is_some() {
                return Err(duplicate_artifact_field(source, label, label_text));
            }
        } else if label_text == "bootstrap-storage" {
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
        kernel_library,
        region_library,
        file_position_library,
        time_library,
        image_library,
        dispatch_library,
        process_identity_library,
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

include!("systems_source_bootstrap.rs");
include!("systems_source_user_memory.rs");
include!("systems_source_memory_devices.rs");
include!("systems_source_device_values.rs");

#[cfg(test)]
mod tests {
    include!("systems_source_tests_root_and_devices.rs");
    include!("systems_source_tests_device_lifecycles.rs");
    include!("systems_source_tests_memory_and_image.rs");
    include!("systems_source_tests_image_completion.rs");
}
