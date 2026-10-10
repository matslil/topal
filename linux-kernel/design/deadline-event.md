# One-shot deadline-event contract

This record resolves `TK-DEC-026` for the first executable systems-profile
deadline event. It extends the provider-created monotonic clock with one typed,
one-shot external event while keeping timer registers, interrupt routing, and
entry mechanics outside portable source.

## Portable source contract

`Deadline C` contains one absolute `Instant C`. Constructing it from an instant
and an exact positive `Duration` is fallible on range overflow and never reads
the clock. The deadline retains the exact identity `C`; another clock cannot
arm, deliver, compare, or satisfy it.

Arming borrows the admitted processor context, consumes the deadline, and
produces one affine `ArmedDeadline C`. Waiting consumes the armed event and
temporarily makes that processor context unavailable. It returns a refined
context only after the matching event has entered its declared handler, the
handler has consumed its completion obligation, and the interrupted
continuation has resumed.

Delivery records both the scheduled deadline instant and a newly observed
instant from the same clock. The observed instant must not precede the
scheduled instant. A late observation is valid and retained rather than
rewritten. Arming a deadline which has already expired produces the same event
immediately; it does not construct or restart a relative interval.

The initial source shape is deliberately closed:

```topal
deadline-notification-handler is fn (
  context : DeadlineInterruptContext InitialMonotonicClock
) -> DeadlineInterruptDisposition InitialMonotonicClock
  completed is context deadline notification complete
  completed resume

deadline : Deadline InitialMonotonicClock is second deadline after 1[ms]
armed : ArmedDeadline InitialMonotonicClock is
  resumed deadline notification arm deadline
resumed is resumed deadline notification wait armed
resumed console write "TOPAL_KERNEL_DEADLINE_OK"
```

The artifact binds exactly one `deadline-notification` external-interrupt entry
to that handler. Entry receives one affine context for the scheduled deadline,
event identity, and observed delivery. `deadline notification complete`
consumes the context and discharges the provider's acknowledgement obligation;
only then is `resume` legal. Source cannot inspect or select a vector,
controller, route, comparator, machine frame, counter encoding, acknowledgement,
wait instruction, or return instruction.

## Provider boundary

The portable abstraction is: deliver one event no earlier than an absolute
deadline on the same monotonic clock. It is intentionally above timer-device
programming and below scheduler policy. A provider proves that its clock and
delivery source share the required identity and ordering.

The first x86-64 Q35 provider uses one HPET comparator in one-shot mode, routes
its event through the I/O APIC, and enters a generated private interrupt entry.
It uses the same HPET main counter as `InitialMonotonicClock`, so no source- or
compiler-visible calibration converts between clocks. Comparator selection,
legacy or explicit routing, vector allocation, masking, acknowledgement, and
controller state remain provider evidence.

AArch64 can implement the contract with the generic timer compare facility and
a GIC per-processor interrupt. RISC-V can use `stimecmp` or the qualified SBI
timer facility and the supervisor timer interrupt. Those targets have
different programming, routing, masking, and acknowledgement rules, but none
requires portable source to name a register or instruction.

## Initial boundary

This increment admits exactly one deadline derived from the second qualified
clock observation, the exact duration `1[ms]`, one arm/wait lifecycle, and one
typed handler. It does not admit cancellation, rearming, periodic release,
scheduler integration, sleep or timeout composition, multiple outstanding
events, bounded latency, rate accuracy, SMP delivery, suspend/migration
guarantees, or a userspace timer ABI. Those require later design decisions and
qualification rather than accidental extension of this interface.

