# Monotonic-clock observation contract

## Decision

The initial kernel time element is one provider-created monotonic clock borrowed
through an admitted processor context. `now` returns an immutable
`Instant InitialMonotonicClock` and records the exact clock identity,
observation identity, and observed instant. Successive observations from that
clock never decrease.

This resolves `TK-DEC-025` as a systems specialization of
`TOPAL-TIME-CLOCK-001`, `TOPAL-TIME-TRACE-001`,
`TOPAL-SYSTEMS-OBSERVATION-001`, and `TOPAL-SYSTEMS-MACHINE-001`. It does not
expose a counter register, address, width, frequency, calibration mechanism,
wrap state, instruction, or device representation.

## Source observations

```topal
first : Instant InitialMonotonicClock is resumed monotonic clock now
second : Instant InitialMonotonicClock is resumed monotonic clock now
resumed console write "TOPAL_KERNEL_TIME_OK"
```

The processor context borrows one qualified `Clock C`; observing it does not
consume the context or clock. Each successful `now` creates one immutable
`Instant C`. Instants retain their exact clock identity, so another clock's
instant cannot be compared, subtracted, or substituted. A clock observation is
an external effect even when source discards the returned instant.

For clock `c`, observation identity `q`, and provider value `t`, the trace
records `Observe(c,q,Instant(c,t))`. Identities increase in source observation
order. The provider must reject a value below its preceding accepted value;
neither the compiler nor source may clamp, replace, predict, merge, duplicate,
or synthesize it. Equal consecutive instants are permitted because monotonic
means nondecreasing rather than strictly increasing.

The clock defines an exact typed tick unit and resolution. Arithmetic and
conversion retain clock and unit identity and must diagnose overflow or
unsupported precision. The initial executable slice observes instants but does
not expose their representation, perform conversion, or claim nanosecond
precision.

The first source slice requires exactly two observations after the completed
local-notification wait and before the time success marker. It rejects an
unknown clock, wrong context, wrong classifier, reordered marker, missing or
extra observation, decreasing model input, and ordinary completion without
the required observations.

## Provider obligations

A provider constructs the clock from qualified target and board evidence. It
owns discovery or fixed-board selection, enablement, access width and ordering,
period or frequency validation, wrap extension, regression detection, and any
state needed to preserve monotonicity. That state is not a source-visible
location or authority.

The provider records:

- the clock and implementation identities;
- the tick unit, resolution, counter width, and wrap policy;
- the board evidence selecting the source;
- the contexts and processors from which it may be observed; and
- whether suspend, migration, frequency change, or processor transfer can
  invalidate the qualification.

An unavailable, malformed, stopped, unsupported, regressing, or otherwise
unqualified clock fails closed. Falling back to a different counter changes
provider identity and requires its own evidence. A monotonic clock makes no
wall-clock, UTC, boot-time, scheduling, wakeup, latency, rate-accuracy, or
real-time guarantee.

## Architecture pressure test

| Target family | Provider-private implementation choices | Common observation |
| --- | --- | --- |
| x86-64 | qualified HPET, invariant TSC, or platform counter; capability/frequency validation; MMIO or instruction access; wrap extension | an immutable same-clock instant and nondecreasing trace |
| AArch64 | selected physical or virtual generic counter, `CNTFRQ` validation, access policy, and migration/suspend qualification | the same clock identity, observation ordering, and monotonic guarantee |
| RISC-V | qualified `time` source through an admitted CSR or execution-environment interface and recorded timebase frequency | the same clock identity, observation ordering, and monotonic guarantee |

The common operation is clock observation, not a spelling for HPET, TSC, an
ARM system register, or a RISC-V CSR. A provider unable to establish stable
identity, scale, access, and monotonicity must reject the profile.

## Initial x86-64 slice

The initial `pc-q35-10.2` provider uses the board's HPET as a private clock
source. It maps the provider-required MMIO page in both the initial and
replacement translations, validates the 64-bit counter capability and the
pinned QEMU period, enables the main counter without enabling timer interrupts,
and reads the 64-bit main counter twice. Provider-private bootstrap storage
retains initialization, preceding raw value, and wrap state. The initial
single-processor profile rejects regression and does not claim SMP-safe clock
state.

Structural inspection requires two clock-observation calls, the HPET mapping,
capability and period validation, enablement, two 64-bit counter reads,
provider-private monotonic state, and fail-to-fatal control flow. Pinned QEMU
evidence emits `TOPAL_KERNEL_TIME_OK` only after both accepted observations.

Deadline construction, timer programming and interrupts, periodic ticks,
sleeps, timeout races, wall-clock synchronization, suspend continuity,
migration continuity, SMP observation, userspace time ABIs, vDSO publication,
and executable AArch64 or RISC-V providers remain fail closed.

This contract was approved in the project discussion before implementation.
