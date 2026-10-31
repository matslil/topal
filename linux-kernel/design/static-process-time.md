# Prepared-process monotonic time and relative sleep

Status: implemented and physically qualified under standing systems-only chat
authorization (`TK-DEC-251`); see the [report](../step-4-static-process-time.md).

The prepared static process borrows the bootstrap owner's existing initial
monotonic clock view. This is a temporary binding to one live machine capability,
not authority derived from a native clock number. Both ordinary kernel
observations and user ABI adapters update the same provider-owned wrap history.
The process retains that binding across exec and returns it at terminal cleanup;
only its private binding and copied interval/output scratch are erased.

The initial native observation profile implements CLOCK_MONOTONIC gettime and
getres with the existing qualified source, including normalized seconds and
nanoseconds and its exact resolution. The native adapter normalizes integer
widths, validates complete writable outputs before publication, permits null
getres output and retains no user pointer. Unsupported clock identities retain
the initial monotonic adapter's EINVAL result. Realtime/coarse/raw/vDSO and clock
adjustment remain separate qualification work.

Relative nanosleep and clock_nanosleep copy a complete native timespec before
waiting. The latter admits relative flags zero and CLOCK_REALTIME or
CLOCK_MONOTONIC: a relative duration is measured by the borrowed monotonic
source. Negative seconds/nanoseconds or nanoseconds at least one billion return
EINVAL; invalid complete input returns EFAULT. Remaining-time pointers are not
used on normal completion. Absolute sleeps and signal interruption are outside
this qualification.

Native timespec conversion saturates at signed 64-bit nanosecond maximum, as
in pinned Linux ktime. Relative addition to current monotonic time saturates
at that same maximum; it never wraps. The resulting deadline is rounded upward
to the clock resolution. Waiting observes the same extended clock history until
that deadline is reached. A saturated request makes no promise to wait beyond
the baseline's representable deadline. This first sole-thread provider may poll while waiting;
it does not claim user scheduling, fairness, latency bounds or interruptible
wait queues. Zero duration is a successful interval without fabricated time.
Resources, cwd, descriptors, signals, startup phase and the running image remain
unchanged across success and declared input errors.

Native qualification shall observe normalized/nondecreasing time before and
after exec, exact/null resolution, native integer widths, writable/readable
extent boundaries and canaries, invalid interval fields, zero and finite waits,
ceil-to-resolution behavior and retained resources. Independent Rust/Python
user witnesses shall compare elapsed observations with requested intervals.
Unmodified pinned BusyBox sleep runs shall use different nonzero durations.
The standard, RTC and completed-deadline regressions shall share one kernel.

Risk is high because authority borrowing, counter wrap, native arithmetic and
blocking interact. Review follows the root's owner/binding provenance, shared
wrap history, full input/output checks, native conversion/addition saturation, deadline rounding,
no early completion, normal-completion remainder semantics, register
restoration, exec retention and terminal erasure/owner return. Cross-architecture
meaning is one clock identity and a relative deadline; AArch64 and RISC-V own
their counter scale/wrap, wait mechanisms and native ABI encodings. Core-language
meaning is unchanged.

Primary evidence is pinned Linux `kernel/time/posix-timers.c`,
`kernel/time/posix-stubs.c`, `kernel/time/hrtimer.c` and
`kernel/time/time.c`, plus the existing
[monotonic observation](bounded-monotonic-clock-gettime.md),
[resolution](bounded-monotonic-clock-getres.md) and
[clock owner](monotonic-clock-observation.md) contracts. See the
[baseline](../baseline.md). General distribution/container/VM-host support
remains unfinished.
