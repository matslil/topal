# Bounded monotonic `clock_getres`

This record resolves `TK-DEC-084` as a systems-profile refinement. It exposes
the resolution of the already qualified monotonic userspace clock without
exposing its counter or frequency representation to Topal source.

## Decision

The sole initial process admits `clock_getres(CLOCK_MONOTONIC, result)` while
its return continuation is executable. A null result is a successful query
without output. Otherwise the candidate must name a complete writable native
16-byte timespec in an owned mapping; the provider publishes exactly zero
seconds and ten nanoseconds under fault recovery and returns zero.

The resolution must agree with the qualified source and the conversion used by
`clock_gettime`. Unsupported clock identities return `-EINVAL`; invalid output
returns `-EFAULT`. The qualification program checks both the exact pair and the
null-output form.

The portable contract is an exact normalized resolution and optional complete
publication. X86-64 derives ten nanoseconds from its Q35 HPET contract;
AArch64 and RISC-V providers may publish different qualified resolutions. This
increment excludes other clocks, vDSO, timers, adjustment, and realtime policy.
