# Step 4 correction: completed deadline notification admission

A queued deadline notification could enter the source handler again after its
event had completed. The failure appeared when the preempted worker resumed
its suspended interrupt frame: the second completion was correctly rejected,
but the native adapter had already admitted an invalid second typed entry.
The normal VM corpus and a repeated-run investigation exposed this path.

The adapter now filters a completed event's raw notification before source
dispatch, acknowledges it and restores the interrupted registers/frame. It
does not complete or preempt again. Accepted completion also masks the I/O
APIC route before disabling/clearing the HPET source and issuing local EOI.
A notification already queued locally still takes the filter path.

Systems decision `TK-DEC-248` formalizes this correction of
`TOPAL-SYSTEMS-DEADLINE-EVENT-001`; it adds no source operation or core-language
meaning. Artifact `/223`, root `/212`, provider `/213`, provider object `/205`,
boot `/123` and gate `/223` identify the correction. This remains Step 4 work;
distribution replacement, Docker/Podman and VM hosting are unfinished.

## Admission and lifetime

The generated x86 entry saves the complete existing register image before
reading its privately bound event state. Completed state takes a local EOI and
the normal interrupt-return mechanism without dispatching source. Other states
continue through the existing checked completion path; this does not accept
an early observation, duplicate source completion or invalid context state.
The filter changes no saved register, hardware frame or scheduler owner.

The source handler still has one typed completion and one preemption transfer.
The ordinary context-transfer, completion queue and terminal reclamation paths
remain responsible for their original affine owners. Target route/vector/MMIO
details stay in the private generated entry/provider. Another architecture
must use its own notification/controller admission mechanism.

## Controlled failure and success

`labs/qemu/x86_64/deadline_gate.py` builds the corrected artifact and pauses
at the exact suspended-handler continuation before register restoration and
interrupt return. Symbolic disassembly locates that point. It verifies the
event is completed and retains the entire 160-byte saved register/hardware
frame and current native registers.

The debugger temporarily changes the existing local-notification helper's
vector immediate from `0xf1` to `0xf2`, invokes that helper to queue the target
notification, then restores its complete original bytes and the scratch flag.
This is declared test setup; no kernel source interface exposes the vector,
helper, register or numeric address. Direct debugger MMIO writes and a native
64-bit store were insufficient injection controls, so they are not accepted
as evidence. The actual 32-bit controller helper supplies the hardware event.
The test proves setup preserved the saved frame/registers and helper code.

On the corrected artifact a second hardware breakpoint observes the completed
notification filter itself. Its saved frame has the same digest as the original
interrupted image. After acknowledgement/return, the worker resumes and retires,
the completion worker resumes/retires, all original resources are reclaimed,
QEMU stays alive and serial becomes quiescent. The typed preemption and worker
resumption/retirement markers each occur once.

The same controlled injection rejects the unfiltered predecessor snapshot.
Its retained digest identifies the actual tested artifact, which was built
while preparing the process-observation increment; no publication or feature
revision alone substitutes for that digest. It reaches terminal-worker
retirement and then fails before preempted-worker resumption, matching the
investigated failure. The record retains this expected rejection separately
from successful compatibility evidence.

## Validation and review

The 26 normal physical cases and the injected passing case use linked kernel
SHA-256 `cff275b4bfb26481e0f9ea85c4e17de86f1104f5e9c7f6e2c0297415062af921`.
One separate predecessor run is an expected failure. Evidence in
`labs/qemu/x86_64/results/topal-deadline-entry-filter-gate.json` binds the harness,
artifacts, temporary vector setup/restoration, frame equality and raw serial
observations. The normal matrix retains three clean variants, eighteen external
application cases and five RTC cases.

The full bounded Cargo workspace, including 196 compiler tests, twelve lab
contract tests, formatting, whitespace, document links and the unchanged
234-rule inventory all pass. Artifact tests account for
the additional private event-state reference and interrupt-return edge.

Risk is high because native interrupt acknowledgement, frame preservation and
affine context transfer interact. Review checks admission before source entry,
exact state binding, route/source/EOI order, all saved registers and frame
restoration, no second scheduler transition, explicit test instrumentation and
complete rollback. The controlled predecessor rejection and corrected frame
comparison supplement implementation self-review; human PR review and merge
remain required. Broader timer cancellation, periodic/SMP delivery, userspace
timers and the remaining initial-plan acceptance levels remain open.
