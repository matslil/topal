impl RootEncoder {
    fn construct_deadline(&mut self, duration_nanoseconds: u64) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.monotonic_observations != 2
            || self.deadline != RootDeadlineState::Fresh
            || duration_nanoseconds != 1_000_000
        {
            return Err(CompileError::Tool(
                "x86 deadline construction requires the second clock observation and exact duration 1[ms]".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::ConstructDeadline)?;
        self.deadline = RootDeadlineState::Constructed;
        Ok(())
    }

    fn arm_deadline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.deadline != RootDeadlineState::Constructed
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 deadline arm requires one constructed deadline and restored processor context"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::ArmDeadline)?;
        self.deadline = RootDeadlineState::Armed;
        Ok(())
    }

    fn wait_deadline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.deadline != RootDeadlineState::Armed
        {
            return Err(CompileError::Tool(
                "x86 deadline wait requires one matching armed event".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::WaitDeadline)?;
        self.deadline = RootDeadlineState::Completed;
        Ok(())
    }

    fn complete_deadline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::DeadlineHandler {
            return Err(CompileError::Tool(
                "x86 deadline completion is admitted only in its external entry".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::CompleteDeadline)
    }

    fn create_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.deadline != RootDeadlineState::Constructed
        {
            return Err(CompileError::Tool(
                "x86 kernel-context creation requires one constructed deadline and its sealed stack region"
                    .into(),
            ));
        }
        let stack_offset = self.bootstrap_region_offset.ok_or_else(|| {
            CompileError::Tool("kernel context requires its live stack owner".into())
        })?;
        let stack_end = stack_offset
            .checked_add(16384)
            .ok_or_else(|| CompileError::Tool("kernel stack extent overflow".into()))?;
        if self.bootstrap_region_bytes != Some(16384)
            || stack_offset < 4096
            || stack_offset % 16 != 0
            || stack_end > self.bootstrap_capacity
            || self
                .context_stack_offsets
                .iter()
                .flatten()
                .any(|other| stack_offset < *other + 16384 && *other < stack_end)
        {
            return Err(CompileError::Tool(
                "kernel context stack is not a contained disjoint ordinary region".into(),
            ));
        }
        let (index, state_offset, entry) = match self.kernel_context {
            RootKernelContextState::Fresh if self.context_layout_matches([false, false, false]) => {
                (0, 80, ProviderSymbol::KernelThreadEntry)
            }
            RootKernelContextState::CooperativeCreated
                if self.context_layout_matches([true, false, false]) =>
            {
                (1, 112, ProviderSymbol::CompletionThreadEntry)
            }
            RootKernelContextState::CompletionCreated
                if self.context_layout_matches([true, true, false]) =>
            {
                (2, 144, ProviderSymbol::TerminalThreadEntry)
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 profile creates exactly three contexts from its sealed disjoint stacks"
                        .into(),
                ));
            }
        };
        self.load_bootstrap_storage_address(state_offset)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]); // lea rsi, stack
        self.rip_relative_storage(stack_offset, 0)?;
        self.bytes
            .extend_from_slice(&[0xba, 0x00, 0x40, 0x00, 0x00]); // 16 KiB
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x0d]); // lea rcx, worker entry
        self.rip_relative_symbol(entry, 0)?;
        self.call_checked_bool(ProviderSymbol::CreateKernelContext)?;
        self.context_stack_offsets[index] = self.bootstrap_region_offset.take();
        self.bootstrap_region_bytes = None;
        self.kernel_context = match index {
            0 => RootKernelContextState::CooperativeCreated,
            1 => RootKernelContextState::CompletionCreated,
            2 => RootKernelContextState::AllSuspended,
            _ => unreachable!("closed context index"),
        };
        Ok(())
    }

    fn context_layout_matches(&self, live: [bool; 3]) -> bool {
        self.context_stack_offsets
            .iter()
            .zip(live)
            .all(|(offset, expected)| offset.is_some() == expected)
    }

    fn create_kernel_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.completion != RootCompletionState::Fresh
            || self.kernel_context != RootKernelContextState::Fresh
        {
            return Err(CompileError::Tool(
                "x86 completion creation requires one fresh dispatcher state".into(),
            ));
        }
        self.completion = RootCompletionState::Created;
        Ok(())
    }

    fn transfer_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role == RootRole::CooperativeKernelThreadHandler {
            self.load_bootstrap_storage_address(80)?;
            self.call_checked_bool(ProviderSymbol::TransferKernelContext)?;
            return Ok(());
        }
        if self.role != RootRole::BootstrapWithLocalNotification {
            return Err(CompileError::Tool(
                "x86 cooperative transfer requires the dispatcher or cooperative worker".into(),
            ));
        }
        let (state_offset, next_state) = match self.kernel_context {
            RootKernelContextState::CooperativeDequeued
                if self.context_layout_matches([true, true, true]) =>
            {
                (80, RootKernelContextState::CooperativeYielded)
            }
            RootKernelContextState::TerminalDequeued
                if self.context_layout_matches([true, true, true]) =>
            {
                (144, RootKernelContextState::TerminalCompleted)
            }
            RootKernelContextState::CooperativeDequeuedAgain
                if self.context_layout_matches([true, true, false]) =>
            {
                (80, RootKernelContextState::CooperativeCompleted)
            }
            RootKernelContextState::CompletionDequeuedAgain
                if self.context_layout_matches([false, true, false]) =>
            {
                (112, RootKernelContextState::CompletionCompleted)
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 kernel-context transfers must follow terminal, preemptible, then awakened-completion order"
                        .into(),
                ));
            }
        };
        self.load_bootstrap_storage_address(state_offset)?;
        self.call_checked_bool(ProviderSymbol::TransferKernelContext)?;
        self.kernel_context = next_state;
        Ok(())
    }

    fn transfer_kernel_context_until_deadline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::CooperativeDequeued
            || !self.context_layout_matches([true, true, true])
            || self.deadline != RootDeadlineState::Constructed
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 deadline-bound transfer requires the first selected worker, constructed deadline, and restored processor context".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::ArmDeadline)?;
        self.deadline = RootDeadlineState::Armed;
        self.load_bootstrap_storage_address(80)?;
        self.call_checked_bool(ProviderSymbol::TransferKernelContext)?;
        self.deadline = RootDeadlineState::Completed;
        self.kernel_context = RootKernelContextState::CooperativeYielded;
        Ok(())
    }

    fn await_deadline_preemption(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::CooperativeKernelThreadHandler {
            return Err(CompileError::Tool(
                "x86 deadline-preemption await is admitted only in its bound worker".into(),
            ));
        }
        self.load_bootstrap_storage_address(80)?;
        self.call_checked_bool(ProviderSymbol::AwaitDeadlinePreemption)
    }

    fn transfer_kernel_context_until_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::CompletionDequeued
            || self.completion != RootCompletionState::Created
            || !self.context_layout_matches([true, true, true])
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 completion-bound transfer requires the fresh selected block-once worker"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(112)?;
        self.call_checked_bool(ProviderSymbol::TransferKernelContext)?;
        self.kernel_context = RootKernelContextState::CompletionBlocked;
        self.completion = RootCompletionState::Blocked;
        Ok(())
    }

    fn await_kernel_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::CompletionKernelThreadHandler {
            return Err(CompileError::Tool(
                "x86 completion await is admitted only in its bound block-once worker".into(),
            ));
        }
        self.load_bootstrap_storage_address(112)?;
        self.call_checked_bool(ProviderSymbol::TransferKernelContext)
    }

    fn signal_blocked_kernel_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::TerminalReclaimed
            || self.completion != RootCompletionState::Blocked
        {
            return Err(CompileError::Tool(
                "x86 completion signal requires the exact blocked worker after terminal reclamation"
                    .into(),
            ));
        }
        self.kernel_context = RootKernelContextState::CompletionSignaled;
        self.completion = RootCompletionState::Signaled;
        Ok(())
    }

    fn consume_kernel_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::CompletionReclaimed
            || self.completion != RootCompletionState::Signaled
        {
            return Err(CompileError::Tool(
                "x86 completion consumption requires its awakened worker reclamation".into(),
            ));
        }
        self.completion = RootCompletionState::Consumed;
        Ok(())
    }

    fn take_preempted_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::CooperativeYielded
            || self.deadline != RootDeadlineState::Completed
        {
            return Err(CompileError::Tool(
                "x86 preempted-context take requires the completed bound deadline transfer".into(),
            ));
        }
        self.kernel_context = RootKernelContextState::PreemptedTaken;
        Ok(())
    }

    fn preempt_current_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::DeadlineHandler {
            return Err(CompileError::Tool(
                "x86 deadline preemption is admitted only in its external entry".into(),
            ));
        }
        self.load_bootstrap_storage_address(80)?;
        self.call_checked_bool(ProviderSymbol::PreemptCurrentKernelContext)
    }

    fn create_runnable_queue(&mut self, capacity: usize) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::AllSuspended
            || capacity != 3
        {
            return Err(CompileError::Tool(
                "x86 root runnable queue requires three suspended contexts and capacity three"
                    .into(),
            ));
        }
        self.kernel_context = RootKernelContextState::QueueCreated;
        Ok(())
    }

    fn enqueue_runnable_context(&mut self) -> Result<(), CompileError> {
        self.kernel_context = match self.kernel_context {
            RootKernelContextState::QueueCreated => RootKernelContextState::CooperativeEnqueued,
            RootKernelContextState::CooperativeEnqueued => {
                RootKernelContextState::CompletionEnqueued
            }
            RootKernelContextState::CompletionEnqueued => RootKernelContextState::AllEnqueued,
            RootKernelContextState::PreemptedTaken => RootKernelContextState::HandoffEnqueued,
            RootKernelContextState::CompletionSignaled => {
                RootKernelContextState::CompletionEnqueuedAgain
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 root runnable enqueue violates the sealed three-worker FIFO ownership"
                        .into(),
                ));
            }
        };
        Ok(())
    }

    fn dequeue_runnable_context(&mut self) -> Result<(), CompileError> {
        self.kernel_context = match self.kernel_context {
            RootKernelContextState::AllEnqueued => RootKernelContextState::CooperativeDequeued,
            RootKernelContextState::HandoffEnqueued => RootKernelContextState::CompletionDequeued,
            RootKernelContextState::CompletionBlocked => RootKernelContextState::TerminalDequeued,
            RootKernelContextState::CompletionEnqueuedAgain => {
                RootKernelContextState::CooperativeDequeuedAgain
            }
            RootKernelContextState::CooperativeReclaimed => {
                RootKernelContextState::CompletionDequeuedAgain
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 root runnable dequeue violates the sealed three-worker FIFO ownership"
                        .into(),
                ));
            }
        };
        Ok(())
    }

    fn consume_empty_runnable_queue(&mut self) -> Result<(), CompileError> {
        if self.kernel_context != RootKernelContextState::CompletionReclaimed
            || self.completion != RootCompletionState::Consumed
        {
            return Err(CompileError::Tool(
                "x86 root runnable queue must be empty after all context and completion reclamations".into(),
            ));
        }
        self.kernel_context = RootKernelContextState::QueueConsumed;
        Ok(())
    }

    fn retire_kernel_context(&mut self) -> Result<(), CompileError> {
        let state_offset = match self.role {
            RootRole::CooperativeKernelThreadHandler => 80,
            RootRole::CompletionKernelThreadHandler => 112,
            RootRole::TerminalKernelThreadHandler => 144,
            _ => {
                return Err(CompileError::Tool(
                    "x86 kernel-context retirement is admitted only in a resumed entry".into(),
                ));
            }
        };
        self.load_bootstrap_storage_address(state_offset)?;
        self.jump(ProviderSymbol::RetireKernelContext);
        Ok(())
    }

    fn reclaim_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification {
            return Err(CompileError::Tool(
                "x86 kernel-context reclaim requires one completed transfer".into(),
            ));
        }
        let (index, state_offset, next_state) = match self.kernel_context {
            RootKernelContextState::TerminalCompleted
                if self.context_stack_offsets[2].is_some() =>
            {
                (2, 144, RootKernelContextState::TerminalReclaimed)
            }
            RootKernelContextState::CooperativeCompleted
                if self.context_stack_offsets[0].is_some() =>
            {
                (0, 80, RootKernelContextState::CooperativeReclaimed)
            }
            RootKernelContextState::CompletionCompleted
                if self.context_stack_offsets[1].is_some() =>
            {
                (1, 112, RootKernelContextState::CompletionReclaimed)
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 kernel-context reclaim requires the next completed FIFO context".into(),
                ));
            }
        };
        self.load_bootstrap_storage_address(state_offset)?;
        self.call_checked_bool(ProviderSymbol::ReclaimKernelContext)?;
        self.context_stack_offsets[index] = None;
        self.kernel_context = next_state;
        Ok(())
    }
}
