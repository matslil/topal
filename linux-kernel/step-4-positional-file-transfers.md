# Step 4: positional file transfers

`TK-DEC-268` implements the [positional transfer design](design/positional-file-transfers.md).
Pread64/pwrite64 now access the owned sparse files and immutable boot file using
invocation-owned offsets. Positive, short, zero, EOF and failed transfers retain
the shared description position. This extends the reusable storage service;
Step 4's distribution, container and VM-host acceptance remains unfinished.

## Realization and bounds

The primary root selects ordinary [static-linux-dispatch-v4](kernel/arch/x86_64/policy/static-linux-dispatch-v4.t): v3's exact 59
operations plus ReadAt/WriteAt (x86-64 selectors 17/18) and Unsupported. All three
older vocabularies/exports remain frozen and independently selectable. One
dispatch slot occupies the existing exclusive runtime domain. The complete
source stack/allocation bounds remain **632 / 56 bytes**, within the existing
4096-byte stack and 65536-byte runtime reserves. No core-language meaning changes.

The [descriptor adapter](../src/topal-compiler/src/systems_positional_transfer.rs)
rejects a negative signed offset before descriptor lookup. A live
console or either pipe endpoint returns ESPIPE before direction/count/copy checks;
invalid descriptors return EBADF. Seekable regular access checks precede numerical
user-range admission, which precedes the complete supplied signed interval check
and zero/EOF/clipping. Directory pread preserves that range/interval ordering
before EISDIR, while pwrite through read-only directory/immutable descriptions
returns EBADF. Offset/count validation uses the supplied position independently
of the stored position, including when that stored position is at INT64_MAX.

[Mutable read/write realization](../src/topal-compiler/src/systems_volatile_transfer.rs)
shares its existing copy, sparse-page, capacity,
timestamp and returned-prefix rules, parameterized by position ownership. Native
code loads positional facts from captured invocation arguments and emits no
description-position store in positional paths. It never implements seek/restore.
O_APPEND pwrite validates the supplied interval first, then places nonempty data
at object EOF without advancing the description. This preserves pinned Linux's
append behavior. Immutable reads use checked retained archive bytes and private
staging, without allocating pages or changing positions.

Filesystem profile version two remains unchanged: four objects, eight resident
pages, 256-byte transfer quantum, 36864-byte store and 311296-byte process backing.
The complete native helper bound stays **112 bytes**. No extra saved stack words
remain across the existing twelve-GPR/RTC clock path. Signed checks, zero/EOF and
failed copies retain existing ownership and cleanup obligations. Vector I/O,
RWF flags and general filesystem-driver equivalence remain unqualified.

Artifact/root/provider/object/boot/gate revisions are
`244 / 230 / 231 / 223 / 144 / 244`. The primary kernel qualified by the current
cohort has SHA-256:

```text
76faf15de41d3282112817dda54a483f5762ebc3222ce919f62a63445c34315b
```

## Independent qualification

The regenerated cohort passes **115 Topal VM observations**: **70** successful
primary-kernel runs, **21** successful variants, **eight** controlled observations
and **16** expected rejections. All **40** unmodified BusyBox workloads retain
one primary kernel identity. Pinned Linux controls remain separate. The complete
bounded serial workspace passes, including **244** compiler library, **261**
compiler CLI, **152** semantic-model and **610** language tests. All **33**
toolchain and **four** reference-VM Python contracts pass, as do formatting,
whitespace, relative links and **275** unique normative systems rules. The entire
physical driver and its Cargo/LLVM/GDB/QEMU children run inside the required
`scripts/run_bounded.py` scope; no validation bypasses the host resource limit.

The [portable model](../src/topal-semantics/src/systems_positional_files.rs)
uses shared read/write transition logic with an optional
invocation position. Independent tests check alias versus independent positions,
positions at INT64_MAX, supplied interval overflow before append, zero/EOF,
rejected numerical/prefix copies, page-boundary clipping, sparse holes at large
offsets, unlinked descriptions and final reclamation. All **16** focused file
model tests pass. The runtime model accepts exactly **79** legal nonempty policy
combinations and rejects multiple dispatch or identity owners.

Compiled independent C clients execute all 65536 low selectors, high-bit variants
of each known selector and five additional wide cases, for all four versions and
both nominal source orders: **525256** bridge calls. They retain the same measured
632-byte stack and 56-byte allocation bounds. Cross-version nominal vocabulary
and boot evidence checks reject mismatched or incomplete selections. These C
clients are qualification oracles and are never kernel-linked.

Six successful dispatch configurations observe **1108** actual source-to-native
boundaries each (**6648** total), including source reordering, dispatch-only and
all three frozen versions. Wrong source operation selection is independently
rejected before native handler entry. The prior source-open, retained-state,
clock, header, process, environment, command and pipe proofs are regenerated for
the current artifact identities; historical reports retain their earlier scope.

The [Linux control](labs/qemu/x86_64/positional_control.py) compiles an independent
raw-syscall caller and boots pinned Linux 7.2.9 in a disposable overlay. It asserts
**24** exact error dispositions, actual regular/immutable bytes, independent and
aliased positions, append placement, large sparse offsets and unlinked lifetime.
The updated control first passed its value assertions but hit the required host
cgroup during shutdown. Reducing QEMU's host translation cache to **32 MiB**,
recorded with the actual emulator command, preserves guest hardware/640 MiB RAM
and completes at a **775.5 MiB** scope peak. A transient SSH handshake timeout is
retried within the same overall 600-second readiness deadline. The wrapper limit
is never raised or bypassed.

The [physical caller](labs/qemu/x86_64/static_positional_file_checks.py) includes
the existing anonymous/named witnesses and issues real positional syscalls.
It passes **360** primary returns, including **38** positional calls. The separate
[inspector](labs/qemu/x86_64/positional_file_inspect.py) snapshots every live regular
description position and watches the target position during native service. It
compares return-time positions, real object/page associations, actual contents,
failed-store atomicity and complete backing erasure. An exploratory caller first
expected an OSABI-zero ELF prefix; inspecting the pinned input showed OSABI three.
The corrected caller explicitly binds that input prefix and passes immutable
positional reads without changing its shared position.

The exec/fork lifetime caller passes **411** returns and **48** positional calls,
including close-on-exec, retained independent descriptions, child writes/unlink,
unchanged aliased/independent positions and parent observations of shared contents.
It leaves one closed linked and one open unlinked object with two pages for
terminal reclamation. Controlled inactive-child cancellation passes **389**
returns and **43** positional calls before complete active/inactive teardown.
Deliberate payload corruption after pwrite is rejected at the caller's specific
independent byte comparison after **328** observed returns; it claims neither
normal completion nor terminal cleanup.

## Review and remaining acceptance

Risk is high because supplied positions, shared state, append, user copies,
immutable backing and reclamation interact. Review follows captured arguments,
complete range admission, effective append position, absence of positional
description stores, native helper bounds, shared page ownership and all cleanup
paths. Independent model, compiled source, pinned Linux, native caller and
debugger oracles check separate boundaries; no external or independent-agent
review is claimed.

General persistent allocation and ordinary effectful Topal subsystems,
storage/devices/mounts, general processes/signals/dynamic ELF, networking,
isolation, distribution replacement, Docker/Podman and VM hosting remain
initial-plan acceptance work. ARM64 implementation remains outside the approved
x86-64 focus. See the [Step 4 status](step-4-status.md) and
[traceability](traceability.md).
