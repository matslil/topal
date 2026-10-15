# Bounded monotonic `clock_gettime`

This record resolves `TK-DEC-083` as a systems-profile refinement. It exposes
the already qualified monotonic observation source through one Linux userspace
clock ABI without exposing counters, MMIO addresses, or conversion policy to
Topal source.

## Decision

The sole initial process admits `clock_gettime(CLOCK_MONOTONIC, result)` while
its return continuation is executable and `result` names a complete writable
native 16-byte timespec in an owned user mapping. The provider samples its
qualified monotonic clock once, converts the observation into normalized
nonnegative seconds and nanoseconds, publishes the complete pair under fault
recovery, retains no user pointer, and returns zero.

Successive completed observations in one boot must be nondecreasing. Unsupported
clock identities return `-EINVAL`; invalid output returns `-EFAULT` without
partial committed output being reported as success. The qualification program
performs two calls, validates normalization, and validates lexicographic
monotonicity.

## Alternatives and cross-architecture pressure

A constant timestamp would satisfy neither observation nor progress-sensitive
applications. Exposing HPET ticks would make x86 frequency and counter details
part of the ABI. Adding realtime now would require a wall-clock epoch and update
policy not yet designed.

The portable contract is a normalized, nondecreasing monotonic observation and
complete user publication. X86-64 may use its qualified HPET source; AArch64
and RISC-V may use their architectural counters with provider-specific scale
and rollover handling. This increment excludes realtime, coarse/raw clocks,
vDSO, timer creation, sleeping, namespaces, adjustment, and SMP migration.
