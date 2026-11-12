# Portable process memory model

This is the stage-1 reference foundation for `TK-DEC-284` and
`TOPAL-SYSTEMS-PROCESS-MM-001`. It executes no privileged instructions, guest,
compiler or application binary. Its Python objects and frame numbers do not
construct native memory authority.

[model.py](model.py) models independent process/MM identities, image generations,
heap bytes, whole extent issuance, active/inactive retention, raw break results,
creation, cooperative handoff, exec, wait/exit and outer cancellation. All live
extents are logically pinned. Its transient events represent allocation,
zero-before-grant, pin, mapping publication, complete abstract invalidation,
unpin, erasure and release. The first-fit allocator is a reference implementation,
not a requirement on ordinary Topal policy or native placement.

[trace_check.py](trace_check.py) separately reconstructs held/pinned frame sets
and visible/cached translations from those events. It rejects missing,
reordered, foreign, aliased, incompletely erased or duplicate effects. Cached
translations are an abstract conservative state, not an observation of hardware
TLBs. A zeroing event contains zero bytes; it proves a reference transition,
not a physical write.

[test_model.py](test_model.py) has seventeen tests. Its application oracle tracks
only process-local byte arrays, breaks and generations; it shares no physical
placement, token, pin or event algorithm. Thirty-two deterministic seeds each
exercise 100 operations. Separate fixtures cover 0/1/33/34/35/67/519 pages,
partial-break page slack, different page sizes, exhaustion/fragmentation,
abandoned prepared extents, both processes' exec, wait eligibility/output retry,
recreation and exact cancellation. Nineteen wrong saved histories and seven
inconsistent saved states must fail. These controls alter model data only.

Run from the repository root, retaining the normal resource wrapper:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/qualify.py --output linux-kernel/.cache/x86_64/process-memory-check-new
```

Use a new output directory. The qualifier records complete unit-test output,
selected source hashes, local link/stable-ID checks and the actual cgroup's
limits/events. It refuses an unbounded or swap-enabled environment. The committed
[stage-1 result](results/stage-1/report.json) records one run, not proof that
later source revisions or the kernel are qualified.

Trace: [language intent](../../../docs/systems-profile.md#process-address-space-ownership),
[requirement](../../../se/requirements.md#topal-req-systems-process-mm-001--independent-process-memory-ownership),
[formal rule](../../../spec/systems-profile.md#topal-systems-process-mm-001--process-address-space-ownership),
[compiler obligation](../../../src/topal-compiler/se-requirements.md#topal-comp-systems-process-mm-001--process-memory-owner-binding),
[design](../../design/process-memory-ownership.md),
[stage report](../../step-4-process-memory-ownership.md).

The model omits image/stack contents, register/TLS/SIMD/descriptors, actual table
walks and permissions, faulting copies, native instruction/entry/stack binding,
physical erasure, identity-width exhaustion, partial-mutation fatal cleanup and
AP/DMA/asynchronous behavior. It models only the selected nonshared heap and
normal/prepublication-unavailable transitions. It provides no evidence for those
omitted obligations or for distribution, containers, virtualization or general
scheduling/MM. Native integration and public source/load/boot reconstruction
must preserve the existing qualified child and pipe workloads before activation.

## Native preparation component

The subsequent [stage-2 preparation archive](results/stage-2-preparation/report.json)
records 32 actual prepared-heap lifetimes inside ordinary captured `brk` calls.
[preparation_native.py](preparation_native.py) checks saved live frame/entry/table
and byte observations. [readback_preparation.py](readback_preparation.py) also
binds exact generated LLVM deltas, preserved contributions and fourteen saved
data controls. It retains an earlier overbroad intermediate-ELF comparison failure;
the actual application image is byte-identical, while three nonloaded intermediate
debug sections differ. The [stage report](../../step-4-process-memory-ownership.md)
states the precise native scope and unresolved activation/fatal/lifetime gates.

Replay the archived records with selected frozen inputs, under the same wrapper:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/replay_preparation.py linux-kernel/tools/process-memory/results/stage-2-preparation linux-kernel/.cache/x86_64/process-memory-preparation-replay-new
```

Use a new output directory. This checks archive identities and repeats the
record/control comparisons from frozen Python; it does not build or execute a
kernel. Native preparation/cancellation does not qualify independent child MM
activation, fork/exec/switch integration or general Linux compatibility.
The test-only `brk` hook is a component witness, including deliberately allocating
on queries; it is not a publishable `brk` policy. Actual process callers must
replace it while preserving the production allocation-free query contract.

## Independent process/MM component

The [first integrated archive](results/stage-3-process-integration/report.json)
records eight real creations/exits and eight handoffs, including four new
source-built 67/101-page independent-heap scenarios and the full ABI witness's
four existing 34-page child/clone cases. Three parent execs and terminal cleanup
also execute. [process_native.py](process_native.py) checks actual saved banks,
pins, entry/mode observations, complete table snapshots and consumed/zeroed
backing. [readback_process.py](readback_process.py) adds whole-case/resize checks
and twenty saved-data negative controls; these controls never modify the guest.

Replay its frozen records without building or executing a kernel:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/replay_process.py linux-kernel/tools/process-memory/results/stage-3-process-integration linux-kernel/.cache/x86_64/process-memory-integration-replay-new
```

Use a fresh output directory. [archive_process.py](archive_process.py) retains
selected sources, native records, commands, bounds, regression logs, prior
failed attempts and exact predecessor artifact comparisons. Frozen replay
checks archive/source identities and repeats the saved-record comparisons; it
does not independently establish the recorded load or any unobserved native
lifetime. Child exec, inactive-child cancellation, empty/maximum/resource/fatal
cases, continuous exclusion, complete publication observations and public
production qualification remain blockers listed in the stage report.

## Child exec component

The subsequent [child-exec archive](results/stage-3-child-exec/report.json)
records four actual child replacements with retained parents, zero/fresh heaps,
FS/registration reset and repeated handoff/restoration. The source program uses
ordinary syscall arguments and selects its own userspace startup from argv;
no test string selects kernel behavior. [readback_exec.py](readback_exec.py)
extends the boundary checks with exact replacement-entry/reset binding and ten
additional saved-data controls. The earlier twenty controls also run.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/replay_process.py linux-kernel/tools/process-memory/results/stage-3-child-exec linux-kernel/.cache/x86_64/process-memory-child-exec-replay-new
```

Use a fresh output directory. Frozen replay supplies no native load, continuous
mapping lifetime, general executable-loader or production authority. The
[stage report](../../step-4-process-memory-ownership.md#child-exec-and-replacement-startup)
retains those blockers, the initial userspace scratch-placement failure and the
separate scope of earlier successful observations.
