# Local-notification interrupt contract

## Decision

The initial external-interrupt source element is one affine local-notification
session. Sending consumes the current bootstrap context into a pending session.
Waiting on that session admits exactly one provider-created external-interrupt
entry, requires completion of that event, resumes the interrupted continuation,
and returns a refined bootstrap context with the exact prior local maskable-
interrupt state restored.

This resolves `TK-DEC-024` as a specialization of
`TOPAL-SYSTEMS-ENTRY-001`, `TOPAL-SYSTEMS-DISPOSITION-001`,
`TOPAL-SYSTEMS-OBSERVATION-001`, and `TOPAL-SYSTEMS-MACHINE-001`. It does not
expose an interrupt vector, controller identity, register, frame, instruction,
mask bit, or machine return mechanism.

## Source lifecycle

```topal
local-notification-handler is fn (
  context : LocalNotificationInterruptContext
) -> LocalNotificationInterruptDisposition
  completed is context local notification complete
  completed resume

pending is restored local notification send
resumed is pending local notification wait
resumed console write "TOPAL_KERNEL_INTERRUPT_OK"

lang systems artifact (
  ...,
  local-notification is lang systems external-interrupt-entry
    local-notification-handler
)
```

`local notification send` consumes the current processor context and creates
one pending event with a source-local monotonically increasing event identity.
The same context name is unavailable while the pending session is live.
`local notification wait` consumes the session and enters a provider wait
protocol which admits only the matching event as successful completion.
Unrelated admitted entries may resume the wait, but cannot satisfy it.

The provider creates one affine `LocalNotificationInterruptContext` only when
the pending event enters its external-interrupt handler. The handler must
consume that context through `local notification complete`; the resulting
completed context admits `resume` and no ordinary continuation. Completion
discharges the source's acknowledgement or pending-state obligation. It can
lower to no instruction only when target evidence proves that no explicit
acknowledgement is required.

Resumption returns to the exact interrupted wait continuation. The wait then
returns a refined bootstrap context whose local maskable-interrupt state equals
the state immediately before `send`. A provider may temporarily enable the
selected interrupt domain inside the sealed wait transition; source cannot
observe or retain that state. A fatal disposition may consume all outstanding
authority, but an ordinary exit cannot leave a pending session, entered event,
or completion obligation live.

The first source slice admits exactly one send/wait lifecycle and exactly one
local-notification handler. It rejects a missing or duplicate entry, handler
reuse, wait without send, a second send while pending, ordinary operations
through the consumed context, completion outside the handler, resume before
completion, duplicate completion, and completion or disposition with live
notification authority.

## Observation and ordering

Sending records `Send(s,q)`, where `s` is the qualified local-notification
source and `q` is its next event identity. Delivery records `Observe(s,q)`,
then entry, completion, resumption, and wait completion in that order. The
provider cannot invent, merge, duplicate, or satisfy the wait with another
event. The initial single-processor gate has one permitted event order and
makes no fairness, latency, or real-time guarantee.

The protocol is an interrupt-control and observation relation. It is not a CPU
memory fence, lock, timer, scheduler handoff, or device/DMA completion. Data
shared with a future concurrent producer still requires its own atomic and
visibility contract.

## Architecture pressure test

| Target family | Provider-private implementation choices | Common observation |
| --- | --- | --- |
| x86-64 | local-APIC self-notification, private IDT vector, controller completion, an interrupt-safe enable-and-wait sequence, and interrupt return | one sent event is observed by its typed handler, completed, and resumes the waiting continuation with prior mask state restored |
| AArch64 | GIC software-generated interrupt, private interrupt ID, acknowledge/EOI sequence, and qualified wait | the same event identity, completion obligation, resumption, and restoration |
| RISC-V | supervisor software interrupt or qualified IPI facility, private cause/control state, pending-bit completion, and qualified wait | the same event identity, completion obligation, resumption, and restoration |

The common operation is a local notification, not a spelling for APIC, GIC, or
RISC-V interrupt-controller facilities. A provider unable to prove matching
delivery, completion, and restoration must reject the profile.

## Initial x86-64 slice

The initial provider installs one private interrupt gate, maps only the
provider-required local-APIC page in addition to the existing bootstrap-
equivalent normal-memory coverage, and sends one fixed-delivery self-
notification. The provider owns its vector, local-APIC configuration, pending
state, completion write, interrupt frame, saved registers, wait sequence, and
return instruction. Generated entry code saves and restores the qualified
register set around the checked handler.

Structural inspection requires distinct send, wait, entry, completion, and
resume placements; one private external-interrupt gate; controller send and
completion operations; the wait sequence; and the interrupt return. Pinned
QEMU evidence emits `TOPAL_KERNEL_INTERRUPT_OK` only after the handler has
completed and the interrupted bootstrap continuation has resumed.

Timers, shared device interrupts, routing and affinity policy, nested external
interrupts, multiple pending events, SMP delivery, scheduler disposition,
userspace interruption, interrupt-thread handoff, and executable AArch64 or
RISC-V providers remain fail closed.

This contract was approved in the project discussion before implementation.
