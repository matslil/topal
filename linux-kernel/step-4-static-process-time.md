# Step 4 increment: prepared-process time and relative sleep

Prepared static processes can query monotonic time and resolution and complete
relative nanosleep/clock_nanosleep requests across exec. Unmodified BusyBox
sleep runs use 0.001 and 0.002 second inputs. The process borrows the kernel's
existing clock identity and history rather than constructing another timeline.

Decision `TK-DEC-251`, `TOPAL-SYSTEMS-STATIC-PROCESS-MONOTONIC-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-RELATIVE-SLEEP-001` define the increment. Artifact
`/227`, root `/216`, provider `/217`, provider object `/209`, boot `/127` and
gate `/227` identify it. Core-language meaning is unchanged.

## Authority and native behavior

The checked root passes a private borrow of its fresh clock observation state
as the twelfth provider argument, retaining stack alignment with a separate pad.
Native entry checks the binding's alignment, supervisor identity range and fresh
state. Private context slot 944 retains it across exec. The adapter reuses the
qualified monotonic provider's counter/scale/wrap/regression implementation.
Terminal return clears the binding and preserves legitimate clock history for
later ordinary kernel clock observations and deadline preemption.

Clock IDs use native integer width. CLOCK_MONOTONIC gettime publishes normalized
seconds/nanoseconds after complete writable-extent validation. Getres publishes
zero seconds/ten nanoseconds, permits null output and creates no clock sample.
Unsupported clock IDs and invalid outputs retain EINVAL/EFAULT respectively.
Staging is erased and read back, and the ordinary return restores registers and
clears saved entry fields.

Relative nanosleep, and clock_nanosleep with CLOCK_REALTIME/CLOCK_MONOTONIC and
relative flags zero, copy a complete native interval. Negative fields or
nanoseconds at least one billion return EINVAL; incomplete readable input
returns EFAULT. Input errors neither observe the clock nor mutate resources.
On normal completion the remaining-time candidate is untouched, even when it
is invalid. Absolute and interruptible sleeps remain separately qualified.

The native ktime boundary follows the pinned source: seconds at least
9223372036 saturate to signed 64-bit nanosecond maximum; smaller validated
intervals convert exactly. Relative addition saturates at the same maximum.
The deadline rounds upward to the ten-nanosecond source resolution and never
wraps to an earlier value. The sole-thread provider observes its clock until
that deadline is reached. It preserves process resources and grants no new
runnable peer, scheduler fairness or latency guarantee. Saturation does not
promise waiting beyond the baseline's representable deadline.

## Executed qualification

Independent Rust/Python user-image encoders check clock normalization and order,
exact/null resolution, integer widths, valid outputs ending at a mapping
boundary, rejected buffers and canaries, invalid intervals, readable immutable
input, exact input boundaries, zero/nonzero waits and unchanged remaining-time
fields. Elapsed observations bracket zero, one-nanosecond, 1000001-nanosecond
and two-millisecond intervals. The checks execute before first exec and after
each of the maximum fixture's three replacements. Scratch preserves upper
stack payloads and is restored/cleared before the later resource witnesses.

`time_gate.py` separately executes three unmodified BusyBox inputs and observes
native arithmetic at the wait boundary. Conversion saturation uses seconds
9223372036; addition saturation uses seconds 9223372035 after a declared forward
HPET test input; resolution rounding uses one nanosecond and checks a target one
tick after the sampled instant. Independent integer arithmetic checks the
observed converted duration and target. QTest explicitly advances saturated
waits to their targets; this is accelerated hardware test input, not evidence
that centuries elapsed in real time. The counter is disabled for each setup
write and its exact prior configuration is restored and verified. Initial clock
binding/history, actual counter readback and unchanged CPU registers are checked.
Normal duration cases use the unmodified hardware timeline.

The matrix has three clean variants, twenty-six external cases and five RTC
cases, plus the three controlled timing cases and one completed-deadline
notification case. All 38 successful physical runs use linked kernel SHA-256
`6c34c690ca1a9d614bdc3727fb3badf332248bfe4816e58ad86c70c238ab7c04`.
The deadline predecessor's expected rejection remains separate. Full bounded
Cargo workspace tests, fifteen lab contract tests, formatting, whitespace,
local links and the 240-rule inventory pass. Evidence binds actual input/output,
status, replacements, full rollback, live QEMU and final serial quiescence.

## Risk review and remaining work

Risk is high because clock authority, observation history, arithmetic and
waiting interact. Self-review checks owner provenance, borrow lifetime, native
widths, complete user transfers, exact baseline saturation, upward rounding,
shared wrap/regression handling, no early completion, remainder behavior,
register restoration and terminal erasure/owner return. Independent encoders,
controlled arithmetic observations and unmodified utilities supplement review;
human PR review and merge remain required.

The [design](design/static-process-time.md) keeps counter mechanisms and ABI
encodings target-owned. Other clocks, absolute/interruptible waits, signal
delivery, runnable-peer scheduling, general ELF selection, dynamic linking,
fork/wait, writable storage, networking, distribution boot, Docker/Podman and
VM hosting remain required initial-plan work. This increment does not complete
Step 4 or imply broader time/scheduling qualification.
