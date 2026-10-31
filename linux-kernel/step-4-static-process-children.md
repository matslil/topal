# Step 4: independently owned static child processes

`TK-DEC-254` lets the unmodified pinned BusyBox shell launch an external child,
wait for its status and continue executing commands. For example,
`/bin/busybox echo child; echo parent` executes the first command in a real child
and resumes the original shell for the second. This advances Step 4; the
[remaining acceptance work](step-4-status.md) still applies.

The [design](design/serialized-static-process-children.md) and rules
`TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-WAIT-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-THREAD-REGISTRATION-001` define the qualified scope.
Core-language meaning is unchanged. Process/ABI mechanisms remain in the sealed
provider; ordinary Topal kernel libraries retain their separate qualification.

## Process and resource lifetimes

The initial schedule selects a finite child until exit while retaining the
parent's pending fork return in an owned inactive snapshot. Child data and stack
writes are private. Child exec replaces only its image; exit restores the parent
before any parent instruction runs, returns the created PID and retains one
waitable status. This serialized order qualifies neither general scheduling
fairness nor parent/child IPC progress.

The native personality admits `fork` and fork-like `clone` with SIGCHLD, zero
alternate stack and optional child-TID publication/clear flags. Unsupported
shapes return ENOSYS before publication; an occupied live/unreaped slot returns
EAGAIN. Child-TID publication uses a checked four-byte write and retains Linux's
best-effort behavior for an inaccessible value. The child observes its own PID/
TID and parent PID one; the initial parent retains PID one and parent zero.

Reusable thread registration retains opaque clear-TID and robust-head values,
checks the native robust-head length, returns the current TID, and resets child/
exec registrations appropriately. The native FS subset supports setting a base
below the qualified four-level user limit and checked eight-byte observation.
Operation selectors use their native integer width. These values grant no source
address authority. Private-mm exit drops registrations without inventing a
shared-mm write/wake; robust traversal and other architecture controls remain
separately qualified.

Descriptor slots/flags, cwd and signal state are private per process. Inherited
slots retain shared open descriptions and their positions/status. The store
accommodates **32** distinct descriptions across two sixteen-slot tables. This
removes the predecessor's single-process assumption that one free slot implies
a free description in the original sixteen-cell bank. Child-only cells must be
released and verified empty before reclaiming snapshot storage.

The expanded supervisor-only process backing is **253952 bytes** (62 pages),
including the original two-page descriptor region. Its child region holds owner/
relationship bookkeeping, a 1024-byte private context, the aligned legacy
extended-state record, private table state, both leaf-table snapshots, eighteen
mutable image pages, all 34 reserved heap pages and both stack pages. The final
page also holds the additional description bank. The linker/boot adapter verifies
the complete symbol identity, size, alignment, writable/nonexecutable role and
disjoint supervisor extent. Truncated, overflowing, unaligned and user-visible
bindings are rejected even with a matching updated kernel digest.

Child exit releases only child slots and preserves parked parent references.
Restoration copies and reads back complete backing intervals, restores page
permissions and invalidates translations, reinstates user registers, FS and
extended state, then erases and reads back the retired snapshot. Clock and entropy
service history remains shared. In particular, restoration preserves the evolved
generator key and delivery counter; it cannot reactivate the parent's old key.

`wait4` admits matching selectors, zero/WNOHANG options, optional status output
and null rusage. A complete checked status publication precedes once-only reaping.
Bad output retains the child for retry; no matching child or a repeated reap
returns ECHILD. Unsupported accounting is explicit and cannot fabricate rusage.
Parent termination erases an unreaped record and all private backing before the
outer owner returns.

## Executed evidence

The primary kernel SHA-256 is:

```text
8ea1d32424b68d609a7f297b40375310fe0582e3c679071f41af4442b4a44ef7
```

Artifact/root/provider/object/boot/gate revisions are
`230 / 219 / 220 / 212 / 130 / 230`.

Independent [Rust](../src/topal-compiler/src/systems_child_checks.rs) and
[Python](labs/qemu/x86_64/static_child_checks.py) ABI encoders exercise actual
user instructions and syscall results. Cases cover both creation returns,
ignored fork arguments, clone selector width, unsupported shapes, capacity,
child identity, private image/stack bytes, inherited shared offsets, independent
slot flags/cwd, changed child FS/XMM state, opaque registration values, invalid
TID publication, exact four-byte publication, status masking, all admitted wait
selectors, null status, failed wait/retry and repeated reap. Four consecutive
children demonstrate slot reuse. The pressure case retains sixteen distinct
parent descriptions, opens sixteen independent child descriptions, and exercises
alias replacement, shared positions and flags in the second bank.

The [debugger harness](labs/qemu/x86_64/children_gate.py) and
[retained record](labs/qemu/x86_64/results/topal-static-process-children-gate.json)
observe four clean child lifetimes and one unmodified BusyBox child exec. At each
parent return, every captured private backing byte, native register, FS base,
XMM/MXCSR and x87 control/status/tag observation matches the parent. Shared
description references remain live and snapshot storage is erased. The pressure
breakpoint observes all **32** distinct live descriptions and their reference
counts. The external exec changes the entropy key/history, and parent restoration
retains that exact evolved history. Terminal observation confirms complete
process-backing, registration and library-binding erasure.

A separate controlled counterexample changes one byte in the inactive parent
snapshot. The independent user witness reaches its exact failing comparison:
observed `0x12345679`, expected `0x12345678`, unequal flags and the intentional
failure loop. The normal gate explicitly rejects this test input before
completion. Timeout alone does not qualify it; its record claims neither normal
completion nor reclamation.

Five pinned BusyBox workloads add child output/parent continuation, nonzero child
status, repeated children, child-shell exec/private cwd and shared inherited file
offsets. The complete retained matrix contains **47 positive physical QEMU runs**
on one kernel: three clean environment modes, 31 external workloads, five RTC
cases, three controlled timing cases, completed-deadline injection, finite and
numeric library debugger runs, and two child debugger runs. A reordered-Enum
kernel is an additional positive run. Wrong finite/numeric policies, the
historical deadline predecessor and the corrupted snapshot remain separate
expected rejections. Hardware time advancement remains explicit test input.

Validation includes the bounded full-workspace suite, 207 compiler library tests,
111 semantic-model tests, nineteen lab contracts, formatting, whitespace, relative
links and 247 unique normative systems rules. The child model checks ownership,
capacity, private restoration, shared history, failed publication/retry, once-only
reaping and terminal release. The harness cleanup regression also proves that an
already closed PTY descriptor cannot close a new debugger-owned descriptor when
the operating system reuses its number.

## Review and remaining scope

Risk is high across private backing, shared reference counts, native/TLS/extended
state, exec/exit and wait publication. Self-review traced higher-level intent,
every snapshot interval and restore/erase boundary, the two-table description
capacity argument, native argument widths, thread lifetime, entropy history and
outer-owner return. Independent ABI encoders, actual unmodified application
execution, full-capacity observation, malformed linked bindings and a precise
restore counterexample supplement model/structural checks. No third-party
material is added; pinned qualification binaries remain outside version control.

One additional serialized process and one static mapping/executable profile are
qualified. General processes/threads and preemption, signal delivery, vfork/shared
VM, dynamic/general ELF, writable VFS/devices, networking/isolation, reference
distribution replacement, Docker/Podman and VM-host acceptance remain required.
AArch64/RISC-V need their own native argument, context, extended-state, mapping
and ownership evidence. This result does not establish full Linux compatibility.
