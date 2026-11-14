// TOPAL-SYSTEMS-BOOT-PARTICIPANT-SOURCE-001. These states validate the checked
// success continuation; they do not model actual native participants arriving.
#[derive(Default, Eq, PartialEq)]
enum BootParticipantRetentionState {
    #[default]
    Unselected,
    NeedsRetention,
    NeedsObservation,
    Retained,
}

impl BootstrapAuthorityState {
    fn observe_boot_participants(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        let error = |message| SystemsModelError::new("E-SYSTEMS-BOOT-PARTICIPANTS", message);
        match (&self.boot_participants, operation) {
            (
                BootParticipantRetentionState::NeedsRetention,
                SystemsOperation::RetainBootParticipantsForKernelLifetime,
            ) => {
                self.boot_participants = BootParticipantRetentionState::NeedsObservation;
                return Ok(true);
            }
            (BootParticipantRetentionState::NeedsRetention, _) => {
                return Err(error(
                    "arrival resource requires its exact lifetime consumer before other work",
                ));
            }
            (
                BootParticipantRetentionState::NeedsObservation,
                SystemsOperation::ConsoleWrite { text },
            ) if text == "TOPAL_BOOT_PARTICIPANTS_ARRIVED" => {
                self.boot_participants = BootParticipantRetentionState::Retained;
                return Ok(true);
            }
            (BootParticipantRetentionState::NeedsObservation, _) => {
                return Err(error(
                    "retained participant domain requires its arrival observation",
                ));
            }
            (_, SystemsOperation::InitializeBootParticipants { entry, .. }) => {
                if !self.boot_participant_window
                    || self.boot_participants != BootParticipantRetentionState::Unselected
                    || !self.allocator_created
                    || !self.image_tables_retained
                    || !self.image_retention_observed
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.translation != BootstrapTranslationState::BootstrapActive
                    || *entry != BootParticipantEntry::InitialProviderSelected
                {
                    return Err(error(
                        "initialization requires the fresh retained boot binding and unborrowed allocator",
                    ));
                }
                self.boot_participant_window = false;
                self.boot_participants = BootParticipantRetentionState::NeedsRetention;
                return Ok(true);
            }
            (_, SystemsOperation::RetainBootParticipantsForKernelLifetime) => {
                return Err(error(
                    "participant retention requires its sole live arrival resource",
                ));
            }
            _ => {}
        }
        // Any ordinary work closes initial admission, including work which
        // later returns the allocator to an otherwise matching ownership state.
        self.boot_participant_window = false;
        Ok(false)
    }

    fn complete_boot_participants(&self) -> Result<(), SystemsModelError> {
        if matches!(
            self.boot_participants,
            BootParticipantRetentionState::NeedsRetention
                | BootParticipantRetentionState::NeedsObservation
        ) {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-BOOT-PARTICIPANTS-LIVE",
                "arrival resource was not consumed into the observed retained boot domain",
            ));
        }
        Ok(())
    }
}
