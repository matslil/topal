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

## Parent termination with a blocked child

The subsequent cancellation component exercises both exit operations at late
shell startup and immediately after replacement entry. Each ordinary userspace
program leaves a child blocked reading an empty pipe, retaining a 101-page heap
and an unlinked anonymous file with a dirty resident page. The parent retains
two writer aliases and its independent 35-page heap when it exits.

[readback_cancel.py](readback_cancel.py) binds the actual wait/FIFO/descriptor,
dirty heap and file bytes to complete destructor readback and detached active
MM cleanup. Its twelve controls per case alter saved records only.
[archive_cancel.py](archive_cancel.py) retains all four cases, exact build and
debugger inputs, unchanged predecessor comparisons, bounds and an identified
complete child-exec regression on the byte-identical kernel. Its optional
`--validation` argument selects fresh validation logs independently of a retained
native regression. [replay_cancel.py](replay_cancel.py) repeats these checks
from frozen selected inputs without executing a VM.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- cargo test -p topal-compiler --lib process_mm_parent_exit_cancels_blocked_child_and_all_resources -- --ignored --nocapture
```

Native boundary snapshots and whole-storage readback do not prove continuous
exclusion, each release/invalidation instruction, invalid internal-state fatal
retirement, every wait kind or general Linux orphan/init semantics. The complete
application corpus and public kernel activation remain separate gates.

The [writer archive](results/stage-3-writer-cancellation/report.json) reverses
endpoint ownership and parks on a full-ring write. It checks all 4,096 nontrivial
FIFO bytes before cancellation and the complete zeroed shared store afterward.
Four additional saved-data controls per case give 64 writer controls. Archive
schema v2 records the userspace writer/read choice and separate validation logs;
the replay command still accepts the preceding v1 reader archive and runs its
original frozen reader. No historical evidence is rewritten.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- cargo test -p topal-compiler --lib process_mm_parent_exit_cancels_blocked_writer_and_dirty_fifo -- --ignored --nocapture
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/replay_cancel.py linux-kernel/tools/process-memory/results/stage-3-writer-cancellation linux-kernel/.cache/x86_64/process-memory-writer-replay-new
```

## Actual physical refusal

The [allocation archive](results/stage-3-allocation-refusal/report.json) uses
ordinary fork and break requests in a 52-MiB guest with the unchanged kernel.
The complete original firmware map/reservations independently reconstruct all
allocator frames. A capacity miss and a fragmented-span miss preserve actual
owners, bytes, mappings and issuance; a separate userspace return binds the
fork's ENOMEM result and complete descriptor/PID-policy storage. One-page and
empty-heap retries then succeed. Native observations remain distinct from the
21 altered saved-data controls and the ten small catalog/firmware tests.

[archive_allocation.py](archive_allocation.py) binds exact inputs and identifies
retained child-exec/reader/writer native evidence on the byte-identical kernel.
[replay_allocation.py](replay_allocation.py) repeats all record checks from the
frozen selected inputs. It neither executes the kernel nor supplies a continuous
exclusion, all-caller, every-failure, fatal-retirement or public-activation proof.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- cargo test -p topal-compiler --lib process_mm_allocation_refusal_preserves_owner_and_pid_history -- --ignored --nocapture
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/replay_allocation.py linux-kernel/tools/process-memory/results/stage-3-allocation-refusal linux-kernel/.cache/x86_64/process-memory-allocation-replay-new
```

## Successful maximum copy

The [maximum-geometry archive](results/stage-3-maximum-geometry/report.json)
records successful ordinary fork and fork-like clone of complete 519-page heaps
in the normal 640-MiB guest. The child independently empties its heap and grows
one page; four ordinary yields retain the parent's 519/518-page pattern and
the child's zero/marked contents. Child exit, wait-output retry and parent
termination complete the observed lifetime. The native kernel remains identical
to the preceding allocation and cancellation cohorts.

[readback_geometry.py](readback_geometry.py) checks complete mapped copies,
owners, pins, table/bank transitions and retirement, with 22 altered saved-data
controls per case. [archive_geometry.py](archive_geometry.py) binds both builds,
actual guest configurations, sources, commands, bounds and identified retained
same-kernel regressions. [replay_geometry.py](replay_geometry.py) repeats the
checks with frozen selected inputs; it does not execute a VM or establish
continuous exclusion, unobserved invalidation order or public activation.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- cargo test -p topal-compiler --lib process_mm_maximum_fork_and_clone_copy_survive_empty_and_one_page_handoff -- --ignored --nocapture
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/replay_geometry.py linux-kernel/tools/process-memory/results/stage-3-maximum-geometry linux-kernel/.cache/x86_64/process-memory-geometry-replay-new
```

## Runnable and zombie termination

The [relationship archive](results/stage-3-relationship-termination/report.json)
records eight ordinary parent terminations: runnable child or unreaped zombie,
both exit operations, and late/early replacement startup. Read-only observation
captures the child's actual yield/exit request with dirty heap/file resources.
The runnable child retains them until cancellation; the zombie's resources have
already retired while its wait status remains. Complete parent/shared cleanup
then passes without duplicate child retirement.

[readback_relationship.py](readback_relationship.py) checks those observations
and 22 altered saved-data controls per case.
[archive_relationship.py](archive_relationship.py) binds all eight builds and
verifies the preceding maximum-copy archive before rechecking its twelve
identified same-kernel native runs. It retains their observations and summaries,
not complete historical rebuilds. [replay_relationship.py](replay_relationship.py)
repeats all new and retained comparisons with frozen selected inputs, without a
VM. These checks do not qualify general Linux orphan/init semantics, continuous
exclusion, every release/invalidation instruction or public kernel activation.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- cargo test -p topal-compiler --lib process_mm_parent_exit_retires_ready_and_zombie_relationships -- --ignored --nocapture
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/replay_relationship.py linux-kernel/tools/process-memory/results/stage-3-relationship-termination linux-kernel/.cache/x86_64/process-memory-relationship-replay-new
```

## Ordinary application interfaces on the process-MM selection

The [ordinary-interface archive](results/stage-3-interface-regressions/report.json)
records the existing transfer, exec-refusal,
signal, poll, pipe, storage, metadata, serial and stat programs on the
experimental process-MM kernel. It retains their original source and native
interface assertions. A separate observer namespace reuses the process-MM
boundary model and captures complete active/inactive/prepared banks around
each selected MM operation, plus the four ordinary source invocations.

The observer enables the common return breakpoint only for a pending MM
operation. Complete records remain in memory; atomic compact JSON checkpoints
occur every 64 events, on MM observer failure and at final completion. Four
host-only transport/checkpoint tests distinguish these observer mechanics from
native memory evidence. The guest retains its finite 600-second deadline,
normal two-CPU/640-MiB TCG configuration and existing host boundaries.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- cargo test -p topal-compiler --lib process_mm_preserves_source_built_application_interfaces -- --ignored --nocapture
```

For an explicit retry, set `TOPAL_TEST_PROCESS_MM_INTERFACES` to a comma-separated
selection such as `metadata,serial,stat`. Unknown, empty or repeated names fail;
without the variable all nine cohorts run. Archive acceptance still requires
successful records from all nine modes, regardless of how the runs were grouped.

[readback_interfaces.py](readback_interfaces.py) joins the ordinary interface
records to complete MM ownership/heap histories and rechecks selected interface
results. Its pipe checker independently reconstructs application FIFO byte order
through wrapped and full-ring transfers. Its metadata checker reconstructs the
whole filesystem/table from every recorded delta and captured clock/payload.
The ordinary ABI witness's children exit directly; cooperative handoff evidence
comes from the retained process cohorts, not this new matrix.

[archive_interfaces.py](archive_interfaces.py) binds sources, generated artifacts,
guest commands, exact observer programs, LLVM verification, bounds, predecessor
comparisons and sampled resources. It checks the preceding relationship archive
and retains twenty identified native histories on the byte-identical kernel.
[replay_interfaces.py](replay_interfaces.py) repeats saved-record checks from
frozen selected inputs without a VM or full historical rebuild. These checks
do not independently replay every native observer assertion or complete the
remaining caller/loan/fatal, continuous lifetime, application-corpus and public
activation gates.

## Unchanged application corpus on the process-MM selection

The ignored `process_mm_preserves_pinned_busybox_workloads` test selects the
existing pinned BusyBox application on the experimental process-MM kernel.
The image must retain SHA-256
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.
Provide an absolute path because Cargo executes the test from its crate directory.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 TOPAL_PINNED_STATIC_USER_IMAGE=/absolute/path/to/pinned/busybox scripts/run_bounded.py -- cargo test -p topal-compiler --lib process_mm_preserves_pinned_busybox_workloads -- --ignored --nocapture
```

The default runs all 51 existing command/output/status contracts in separate
normal two-CPU/640-MiB TCG guests. `TOPAL_TEST_PROCESS_MM_WORKLOADS` permits an
explicit comma-separated diagnostic selection; unknown, empty or repeated
names fail. A diagnostic subset is not complete corpus qualification.

[The application observer](../../labs/qemu/x86_64/process_mm_application_inspect.py)
retains the predecessor application checks for their original operations 0..7.
Separate namespaces retain the existing owned-extent callback/copy/staging
observers and the process-MM boundary model. Selected MM returns additionally
check the original complete caller/private/stack/facts/output/recovery retirement
windows. Final observation records whole frame-catalog and registry digests,
the remaining root tokens and actual CR3. No guest state is supplied or changed
by those read-only observations. The shared console checker preserves the
existing prelude, output, exit-status, newline and successful-exec contracts.
Saved-console tests use all 51 retained application consoles and the clean ABI
console, plus altered output/status/exec/prelude controls; those are host tests.

[readback_applications.py](readback_applications.py) joins the legacy and MM
records, rechecks every MM transition and binds each retired window's address,
size and zero digest. Recorded load spans and reservations bind to the linked
ELF and actual boot-resource length. It reconstructs the complete final frame catalog and
registry from the original firmware frames and the remaining pinned bootstrap
allocation, then compares their native digests. Console and altered-record
checks remain distinct from native execution and from continuous lifetime or
invalidation/release-order proofs.

[archive_applications.py](archive_applications.py) requires all 51 successful
workloads, exact pinned bytes and boot resource, the bound observer programs,
unchanged predecessor contributions, LLVM/bounds checks, fresh validation and
the actual bounded-run/source-stability record. It verifies the preceding
interface archive before rechecking its nine interface and twenty process
histories on the byte-identical kernel. Those histories are previous native
executions; they are not newly run. Earlier invocation and preliminary records
can be retained with their separate scope.
[replay_applications.py](replay_applications.py) extracts frozen selected inputs,
repeats the new and retained record comparisons, and runs the frozen Python
model suites without executing a VM or rebuilding the kernel. Retained record
identities also match the stored preceding report and its kernel identity.
The intermediate `records.json` covers saved-record comparisons only; the final
`replayed.json` is written after both frozen model suites also pass. A failed
suite leaves no overall replay success report.
Public activation
and the original complete Linux/distribution/container/VM-host objective remain
separate acceptance work.
