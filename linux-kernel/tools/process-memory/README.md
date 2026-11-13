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

## Process publication and retirement records

`TOPAL-SYSTEMS-PROCESS-MM-001` / `TOPAL-SYSTEMS-USER-EXTENT-001`.
[publication.py](publication.py) checks actual owner/pin routes for resize,
exec, equal-size fork/handoff/child-exit and empty-heap publication. Retirement
requires an earlier completed demap, successful unpin, complete physical erasure
before release and consumed owner storage. Its model evidence does not prove
continuous access exclusion or other-CPU invalidation.

The optional native observer is enabled with
`TOPAL_TEST_PROCESS_MM_PUBLICATION=1` for the seven existing ignored
process-memory tests: owner handoff, child exec, allocation refusal, maximum
geometry, blocked-reader cancellation, blocked-writer cancellation and
ready/zombie relationships. Together these create 21 VMs, retaining each
cohort's normal RAM, CPU and timeout configuration. The default observer and
native kernel behavior retain their existing selection.

[readback_publication.py](readback_publication.py) requires the complete bound
observer, actual publisher/caller bytes, all hierarchy pages and MM boundaries,
every raw CPU-log header and every observed retirement. It also runs the
original cohort reader. Terminal source cases retain their marker through
exec/exit; returning cases must clear it. Neither kind may retain a pending MM
request or refusal. Completion tests distinguish these states without weakening
the original terminal cleanup assertions. Twelve altered-record controls test
the new publication/retirement checks separately from native execution.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/readback_publication.py linux-kernel/.cache/x86_64/BUILD
```

[archive_publication.py](archive_publication.py) freezes the completed
seven-test matrix. It requires 21 distinct builds, exact source/command/artifact
bindings, unchanged predecessor contributions, LLVM/bounds checks and fresh
validation. It preserves raw CPU logs and the earlier failed/pilot attempts
with their original input identities and separate scope. The previous 51
application, nine interface and twenty process histories are rechecked using
their original kernel and boot-resource facts; those are historical native
executions. [replay_publication.py](replay_publication.py) restores frozen inputs
and repeats the complete source-bound readers and model suites. Its final
`replayed.json` is written only after both the record checks and the 203/59
model suites pass. It does not run a VM or rebuild a kernel.

Historical matrix build directories also retain later host readbacks and LLVM
verification. The original matrix command/error logs establish what passed or
failed in that invocation; a later successful readback does not rewrite its
failure or imply another native execution. Historical source manifests identify
their captured or explicitly reconstructed input bytes separately.

The complete seven-test/**21-VM** matrix passes, with 279 publications,
26,191 invalidation states, 166 retired extents and 252 new
saved-data controls. The [archive](results/stage-3-process-publication/report.json)
binds its sources/artifacts and separately scoped historical evidence. Frozen
replay passes the new and retained records plus the 203/59 model suites; its
[companion receipt](results/stage-3-process-publication-replay.json) binds the archive manifest. This qualifies the local observer
component. All remaining caller/loan/writer, continuous lifetime,
AP/DMA/asynchronous, fatal and public activation gates remain acceptance
obligations.

## Source-built process policy failures

The first completed-delta stage ran the unchanged ordinary process cohort
and two diagnostic Topal policies with the unmodified fork/resize C fixture.
The current [run_policy_failure.py](run_policy_failure.py) adds the two partial
source-status cases described below. [policy_failure.py](policy_failure.py) checks retained effects/owners,
poison and declared invocation retirement. The read-only observer preserves the
ordinary MM and publication assertions. [readback_policy_failure.py](readback_policy_failure.py)
binds the complete observer, source/controller/application/kernel/VM inputs,
whole registry, source deltas, mappings, raw CPU logs and physical retirement.
The original archived reader used 12 allocation and 15 release controls,
which altered saved records only; current readers add source-result/call checks.

The release failure remains incomplete after publication, unpin and erasure;
neither its retained token nor the comparison snapshot may be treated as a
successful release or rollback. The direct-MM gateway retires 4,000 caller bytes.
The separate outer adapter and other fatal/lifetime paths remain acceptance work.
Completed local publication readback is shared with the ordinary cohort, whose
full completion and terminal-root checks remain mandatory.

```sh
rtk proxy mkdir -p linux-kernel/.cache/x86_64/VALIDATION-TMP
rtk proxy env TMPDIR=/absolute/repository/linux-kernel/.cache/x86_64/VALIDATION-TMP RUST_TEST_THREADS=2 RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/run_policy_failure.py linux-kernel/.cache/x86_64/NEW-MATRIX
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/archive_policy_failure.py collect linux-kernel/.cache/x86_64/NEW-MATRIX linux-kernel/.cache/x86_64/NEW-ARCHIVE
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/archive_policy_failure.py replay linux-kernel/.cache/x86_64/NEW-ARCHIVE linux-kernel/.cache/x86_64/NEW-REPLAY
```

The [three-VM archive](results/stage-3-process-policy-failure/report.json)
preserves the source-frozen matrix, fresh validation logs, exact diagnostic
policies, native/boot artifacts and CPU records. Collection verifies all seven
LLVM modules per build. Frozen replay repeats the three saved-record readers and
their controls; it runs no VM, rebuild or model suite. The separate
[review receipt](results/stage-3-process-policy-failure-review.json) binds actual
bounded resources, final descriptive statuses, unchanged surrounding objects and
linked terminal instructions. Its scope does not establish canonical allocator
qualification, every fatal/outer-caller/lifetime path or public activation.

The original three native/readback cohorts passed, but the broader compiler
suite encountered explicit temporary-filesystem quota errors. Its failed batch
and all logs remain archived. Separate fresh qualification uses disk-backed
task scratch and two test threads under the unchanged bounded memory policy.
Collection can bind that separately successful `--validation` directory to the
exact original matrix bytes. An explicit `--original-collector` is required for
the sole post-native collector repair: original/new collector bytes and hashes
are preserved. No compiler, policy, application, observer or reader source
change is permitted through that mechanism. Future fresh runs need neither
recovery option when their complete matrix succeeds.

### Partial source-status diagnostics and current five-case driver

The current driver runs one complete ordinary cohort plus completed allocation/
release delta failures and partial allocation/release status failures. Each new
partial producer performs 13 stores before returning invalid protocol; its
progress header retains twelve completed advances. The reader binds the actual
boxed source result, decoder and fatal call sites, full retained effects and
held identities through nonreturning cleanup. Current diagnostic reader counts
are 15/18/17/20 controls, respectively. These saved-data controls execute no VM.

The [five-VM archive](results/stage-3-process-partial-source/report.json) and
[review receipt](results/stage-3-process-partial-source-review.json) retain fresh
native executions, all seven verified LLVM modules per build and frozen-reader
replay. Fourteen surrounding objects remain byte-identical. The boundary object
differs only in its source-derived history-field displacement (1,272 to 1,360);
the receipt binds the generated LLVM field, exact operand, linked instruction
and equality of every other byte. The earlier three-VM archive remains unchanged.

`run_policy_failure.py` defaults to full validation. The optional
`--validation-profile observer` is restricted to this private-test/observation
increment from commit `9c84c21a305f502a29aaa0cce974ba85fc879c59`. It verifies the
earlier successful full compiler-suite receipt and unchanged compiler hashes,
refuses other tracked compiler/dependency/design changes, runs six fresh checks
(including workspace/all targets, both model suites and the default ABI fixture),
and requires all five native cases with byte-identical historical kernels.
The report explicitly records that the earlier 383-test suite was not rerun.
`test_policy_failure_profile.py` supplies four model-level reuse/refusal controls.
Use the default full profile for a compiler/runtime/dependency/design change.
Neither profile qualifies every fatal path, outer caller, continuous lifetime,
other-CPU/DMA/asynchronous access or public kernel activation.
