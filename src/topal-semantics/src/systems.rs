//! Architecture-independent reference model for the initial systems profile.

use std::fmt;

pub const INITIAL_SYSTEMS_TARGET: &str = "x86_64-unknown-none";
pub const INITIAL_SYSTEMS_BOARD: &str = "topal-qemu-pc-q35-10.2";
pub const INITIAL_SYSTEMS_PROFILE: &str = "topal.systems.x86_64-qemu-pc-q35-10.2/1";
pub const SYSTEMS_CONSOLE_WRITE: &str = "topal.systems.device.console.write/1";
pub const SYSTEMS_DEBUG_BREAK: &str = "topal.systems.machine.debug-break/1";
pub const SYSTEMS_RESUME_DEBUG_BREAK: &str = "topal.systems.disposition.resume-debug-break/1";
pub const SYSTEMS_FATAL: &str = "topal.systems.disposition.fatal/1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsTargetSelection {
    pub target: String,
    pub board: String,
    pub profile: String,
}

impl SystemsTargetSelection {
    #[must_use]
    pub fn initial_x86_64_qemu() -> Self {
        Self {
            target: INITIAL_SYSTEMS_TARGET.into(),
            board: INITIAL_SYSTEMS_BOARD.into(),
            profile: INITIAL_SYSTEMS_PROFILE.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsEntryKind {
    Bootstrap,
    SynchronousExceptionDebugBreak,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsContextKind {
    Bootstrap,
    DebugBreak,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemsOperation {
    ConsoleWrite { text: String },
    DebugBreak,
}

impl SystemsOperation {
    #[must_use]
    pub const fn semantic_identity(&self) -> &'static str {
        match self {
            Self::ConsoleWrite { .. } => SYSTEMS_CONSOLE_WRITE,
            Self::DebugBreak => SYSTEMS_DEBUG_BREAK,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemsDisposition {
    Resume,
    Fatal { message: String },
}

impl SystemsDisposition {
    #[must_use]
    pub const fn semantic_identity(&self) -> &'static str {
        match self {
            Self::Resume => SYSTEMS_RESUME_DEBUG_BREAK,
            Self::Fatal { .. } => SYSTEMS_FATAL,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsHandler {
    pub name: String,
    pub context: SystemsContextKind,
    pub operations: Vec<SystemsOperation>,
    pub disposition: SystemsDisposition,
    pub effects: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsEntry {
    pub kind: SystemsEntryKind,
    pub handler: SystemsHandler,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsProgram {
    pub target: SystemsTargetSelection,
    pub bootstrap: SystemsEntry,
    pub debug_break: SystemsEntry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemsTransition {
    EnterBootstrap,
    ConsoleWrite { text: String },
    ObserveDebugBreak,
    EnterDebugBreak,
    ResumeDebugBreak,
    Fatal { message: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsModelError {
    pub code: &'static str,
    pub message: String,
}

impl SystemsModelError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for SystemsModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SystemsModelError {}

impl SystemsTransition {
    #[must_use]
    pub const fn semantic_identity(&self) -> &'static str {
        match self {
            Self::EnterBootstrap => "topal.systems.entry.bootstrap/1",
            Self::ConsoleWrite { .. } => SYSTEMS_CONSOLE_WRITE,
            Self::ObserveDebugBreak => SYSTEMS_DEBUG_BREAK,
            Self::EnterDebugBreak => "topal.systems.entry.synchronous.debug-break/1",
            Self::ResumeDebugBreak => SYSTEMS_RESUME_DEBUG_BREAK,
            Self::Fatal { .. } => SYSTEMS_FATAL,
        }
    }
}

/// Validate that a program belongs to the sealed initial systems profile.
///
/// # Errors
///
/// Returns a stable systems-model diagnostic when the target identity, entry
/// contexts, admitted operations, dispositions, or derived effects do not
/// match the initial profile.
pub fn validate_systems_program(program: &SystemsProgram) -> Result<(), SystemsModelError> {
    if program.target != SystemsTargetSelection::initial_x86_64_qemu() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-TARGET",
            "the initial systems model requires its exact target, board, and profile identity",
        ));
    }
    validate_entry(
        &program.bootstrap,
        SystemsEntryKind::Bootstrap,
        SystemsContextKind::Bootstrap,
    )?;
    validate_entry(
        &program.debug_break,
        SystemsEntryKind::SynchronousExceptionDebugBreak,
        SystemsContextKind::DebugBreak,
    )?;
    if program.bootstrap.handler.name == program.debug_break.handler.name {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-ENTRY-SET",
            "bootstrap and debug-break entries require distinct handlers",
        ));
    }
    Ok(())
}

fn validate_entry(
    entry: &SystemsEntry,
    required_kind: SystemsEntryKind,
    required_context: SystemsContextKind,
) -> Result<(), SystemsModelError> {
    if entry.kind != required_kind || entry.handler.context != required_context {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-HANDLER",
            "systems entry kind and affine context do not match",
        ));
    }
    if entry.handler.name.is_empty() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-HANDLER",
            "a systems entry requires a statically named handler",
        ));
    }
    if required_context == SystemsContextKind::DebugBreak
        && entry
            .handler
            .operations
            .contains(&SystemsOperation::DebugBreak)
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-OPERATION",
            "debug-break is not admitted by a live debug-break context",
        ));
    }
    match (required_context, &entry.handler.disposition) {
        (SystemsContextKind::Bootstrap, SystemsDisposition::Fatal { .. })
        | (
            SystemsContextKind::DebugBreak,
            SystemsDisposition::Resume | SystemsDisposition::Fatal { .. },
        ) => {}
        (SystemsContextKind::Bootstrap, SystemsDisposition::Resume) => {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DISPOSITION",
                "resume is admitted only by a live debug-break context",
            ));
        }
    }

    let mut expected_effects = entry
        .handler
        .operations
        .iter()
        .map(|operation| operation.semantic_identity().to_owned())
        .collect::<Vec<_>>();
    expected_effects.push(entry.handler.disposition.semantic_identity().to_owned());
    expected_effects.sort();
    expected_effects.dedup();
    if entry.handler.effects != expected_effects {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-EFFECTS",
            "systems handler effects must equal its sorted, deduplicated semantic identities",
        ));
    }
    Ok(())
}

/// Execute the deterministic abstract transition model for one checked root.
///
/// # Errors
///
/// Returns a stable systems-model diagnostic when the program is outside the
/// sealed initial profile.
pub fn model_systems_transitions(
    program: &SystemsProgram,
) -> Result<Vec<SystemsTransition>, SystemsModelError> {
    validate_systems_program(program)?;
    let mut transitions = vec![SystemsTransition::EnterBootstrap];
    for operation in &program.bootstrap.handler.operations {
        match operation {
            SystemsOperation::ConsoleWrite { text } => {
                transitions.push(SystemsTransition::ConsoleWrite { text: text.clone() });
            }
            SystemsOperation::DebugBreak => {
                transitions.push(SystemsTransition::ObserveDebugBreak);
                transitions.push(SystemsTransition::EnterDebugBreak);
                for operation in &program.debug_break.handler.operations {
                    if let SystemsOperation::ConsoleWrite { text } = operation {
                        transitions.push(SystemsTransition::ConsoleWrite { text: text.clone() });
                    }
                }
                match &program.debug_break.handler.disposition {
                    SystemsDisposition::Resume => {
                        transitions.push(SystemsTransition::ResumeDebugBreak);
                    }
                    SystemsDisposition::Fatal { message } => {
                        transitions.push(SystemsTransition::Fatal {
                            message: message.clone(),
                        });
                        return Ok(transitions);
                    }
                }
            }
        }
    }
    if let SystemsDisposition::Fatal { message } = &program.bootstrap.handler.disposition {
        transitions.push(SystemsTransition::Fatal {
            message: message.clone(),
        });
    }
    Ok(transitions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn program(debug_disposition: SystemsDisposition) -> SystemsProgram {
        let target = SystemsTargetSelection::initial_x86_64_qemu();
        SystemsProgram {
            target,
            bootstrap: SystemsEntry {
                kind: SystemsEntryKind::Bootstrap,
                handler: SystemsHandler {
                    name: "boot".into(),
                    context: SystemsContextKind::Bootstrap,
                    operations: vec![
                        SystemsOperation::ConsoleWrite {
                            text: "boot".into(),
                        },
                        SystemsOperation::DebugBreak,
                    ],
                    disposition: SystemsDisposition::Fatal {
                        message: "done".into(),
                    },
                    effects: vec![
                        SYSTEMS_CONSOLE_WRITE.into(),
                        SYSTEMS_FATAL.into(),
                        SYSTEMS_DEBUG_BREAK.into(),
                    ],
                },
            },
            debug_break: SystemsEntry {
                kind: SystemsEntryKind::SynchronousExceptionDebugBreak,
                handler: SystemsHandler {
                    name: "debug".into(),
                    context: SystemsContextKind::DebugBreak,
                    operations: Vec::new(),
                    effects: vec![debug_disposition.semantic_identity().into()],
                    disposition: debug_disposition,
                },
            },
        }
    }

    #[test]
    fn resumed_debug_break_returns_to_bootstrap_before_fatal() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-OBSERVATION-001.
        assert_eq!(
            model_systems_transitions(&program(SystemsDisposition::Resume)).unwrap(),
            [
                SystemsTransition::EnterBootstrap,
                SystemsTransition::ConsoleWrite {
                    text: "boot".into(),
                },
                SystemsTransition::ObserveDebugBreak,
                SystemsTransition::EnterDebugBreak,
                SystemsTransition::ResumeDebugBreak,
                SystemsTransition::Fatal {
                    message: "done".into(),
                },
            ]
        );
    }

    #[test]
    fn fatal_debug_break_cannot_resume_bootstrap() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-DISPOSITION-001.
        let transitions = model_systems_transitions(&program(SystemsDisposition::Fatal {
            message: "fault".into(),
        }))
        .unwrap();
        assert_eq!(
            transitions.last(),
            Some(&SystemsTransition::Fatal {
                message: "fault".into(),
            })
        );
        assert!(!transitions.contains(&SystemsTransition::Fatal {
            message: "done".into(),
        }));
    }

    #[test]
    fn rejects_invalid_context_operations_and_effect_claims() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-AUTHORITY-001.
        let mut invalid_context = program(SystemsDisposition::Resume);
        invalid_context
            .debug_break
            .handler
            .operations
            .push(SystemsOperation::DebugBreak);
        assert_eq!(
            model_systems_transitions(&invalid_context)
                .unwrap_err()
                .code,
            "E-SYSTEMS-OPERATION"
        );

        let mut invalid_effects = program(SystemsDisposition::Resume);
        invalid_effects.bootstrap.handler.effects.clear();
        assert_eq!(
            model_systems_transitions(&invalid_effects)
                .unwrap_err()
                .code,
            "E-SYSTEMS-EFFECTS"
        );
    }

    #[test]
    fn rejects_wrong_target_and_bootstrap_resume() {
        // TOPAL-SYSTEMS-QUALIFY-001, TOPAL-SYSTEMS-DISPOSITION-001.
        let mut wrong_target = program(SystemsDisposition::Resume);
        wrong_target.target.board = "another-board".into();
        assert_eq!(
            validate_systems_program(&wrong_target).unwrap_err().code,
            "E-SYSTEMS-TARGET"
        );

        let mut invalid_disposition = program(SystemsDisposition::Resume);
        invalid_disposition.bootstrap.handler.disposition = SystemsDisposition::Resume;
        invalid_disposition.bootstrap.handler.effects = vec![
            SYSTEMS_CONSOLE_WRITE.into(),
            SYSTEMS_DEBUG_BREAK.into(),
            SYSTEMS_RESUME_DEBUG_BREAK.into(),
        ];
        assert_eq!(
            validate_systems_program(&invalid_disposition)
                .unwrap_err()
                .code,
            "E-SYSTEMS-DISPOSITION"
        );
    }
}
