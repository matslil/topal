# Initial static-process children and wait ownership

Status: systems-only direction adopted under standing authorization
(`TK-DEC-254`); implementation and qualification pending.

## Purpose and selected realization

The initial shell can replace itself but cannot launch a child and subsequently
continue. Step 4 requires process creation, shared open descriptions, independent
image/thread state, exit ownership and child wait. This refinement introduces
those relationships for the existing static executable/mapping profile, with
one additional process slot. It does not claim general scheduling, threads,
vfork/shared-address-space cloning, asynchronous signal delivery or distribution
boot.

The first scheduler chooses a newly created child until its terminal transition;
the parent's pending fork return remains parked. This is a qualified serialized
execution order, not vfork: child memory is private, child writes never modify
the parent, and exec does not alter the parent's image. The finite-child corpus
does not establish fairness or parent/child IPC progress. The future scheduler
must make independent runnable contexts available at blocking/preemption points.

An owned inactive snapshot represents the parked parent while the child's
private image occupies the active mapping window. The native provider copies
the complete mutable backing, permissions, user registers, TLS/extended state,
descriptor slots, signal registrations/mask and process-local observations.
It preserves immutable executable backing and shared open-description identities.
Clock and entropy-service history remain shared with their existing owners;
restoration must never reactivate a saved old random-generator key or counter.
At child exit it restores the parent before any parent instruction executes.
Numeric equality of a PID, pointer or frame does not create a snapshot owner.

Alternative implementations were considered. Returning a fabricated PID without
executing a child is invalid. Shared parent/child data would implement different
clone semantics. Eager independent physical hierarchies and a preemptive scheduler
remain the eventual general realization, but are not prerequisites for the
initial serialized order. Snapshot parking reuses already owned mapping/backing
resources with explicit additional private storage and complete reclamation.

## Lifetimes and native interfaces

The prepared process owner reserves a separate child/snapshot region alongside
its descriptor storage. The existing descriptor region retains its layout and
ownership. Construction checks the whole expanded supervisor-only extent,
alignment, disjointness and initial zero state. The region records one live
child relationship or one exited, unreaped child; a second creation fails with
resource exhaustion while that relationship occupies the slot.

The initial native personality qualifies `fork` and the fork-like `clone`
shape used by the pinned static libc: `SIGCHLD`, zero alternate stack, and
optional child TID publication/clear flags. Shared-VM/files/signal/thread,
TLS-selection and other clone modes require later qualification. Unknown
selectors retain ENOSYS. Unsupported shapes must fail before resource or output
publication. The selected child-TID publication is best effort, matching Linux: an
inaccessible value does not reject creation and grants no user access authority.
Published words are native 32-bit values in the child address-space generation;
parent-TID publication remains a separate unqualified shape.

Creation copies process-local state, increments each inherited open-description
reference and assigns a child PID distinct from the parent. Per-slot flags and
cwd/signal state are independent after creation; positions/status of inherited
open descriptions remain shared. The shared store accommodates thirty-two
distinct descriptions for two sixteen-slot tables. A free child slot cannot
justify treating the original sixteen-cell bank as sufficient: parked parent
references remain live. Additional child-only cells must be empty before private
snapshot storage is reclaimed. A child may use shared services and replace
its own static image repeatedly. Exec resets its thread/image registrations and
close-on-exec slots while retaining process identity and parent relationship.

Native thread registrations must become reusable after startup: `set_tid_address`
retains its opaque user value and returns the current TID; `set_robust_list`
validates the native record length and retains its opaque head value. Registration
does not dereference those values or grant storage authority. The initial child is the sole owner of its private address space, so exit drops
its clear-TID registration without a shared-mm write/wake, matching the baseline.
Shared-mm clear/wake and robust owner-death traversal require later qualification. TLS and complete extended state
must be restored with the parked parent. Child setup cannot overwrite the parent's
retained registration, image, user stack or native return frame.
The reusable native x86-64 TLS subset admits setting a canonical positive user
FS base and checked observation of that base. It does not dereference the base
or qualify other architecture-control selectors. Child exec resets it for new
startup; parent restoration reinstates its exact saved hardware and private value.

Child exit closes only the child's slots, releases its last description references
and image/thread state, records the low eight status bits, and restores the parent.
It does not release the outer root/process owner or publish the parent's terminal
marker. The parent then receives the created PID at its original fork continuation.
The exited child's relationship remains until wait consumes it or the parent
terminates. PID allocation/reuse follows that retained lifetime rather than a
caller-supplied integer. The child observes its own PID/TID and the real parent PID;
the initial parent retains PID one and its existing parent-zero view.

The initial `wait4` profile admits a matching child selector, zero or WNOHANG
options and a null rusage destination. It returns an existing exited child's PID
and optional native status word; a bad destination preserves the child for retry.
No matching relationship returns ECHILD, including a second successful reap.
Unsupported options/output profiles cannot consume the relationship. Wider
wait selectors, accounting, stopped/continued children and signal interruption
remain later work. A zero status pointer is valid. Successful publication precedes
destruction of the relationship. Parent termination drops any unreaped record.

Every snapshot byte, retained user pointer, allocation/publication witness and
private extended-state record is erased and read back once its lifetime ends.
The child slot and descriptor counts must agree before the outer owner returns.
A stale snapshot, nested activation, missing/changed backing identity, wrong PID
owner or failed restoration enters the invariant-failure path.

## Qualification and portability

Tests must execute both fork returns, private parent/child data and stacks, child
PID/parent observations, status masking, rejected creation and retry, capacity,
matching and repeated waits, invalid wait outputs, independent slot flags/cwd,
shared description offsets, child exec, preserved TLS/extended state and complete
snapshot/relationship/description teardown. The pinned unmodified shell must
launch external child commands, observe their statuses and continue itself.
Application command bytes and success output must never select provider behavior.

A host-Linux GDB control with the digest-pinned BusyBox observes fork-like
`clone` flags `0x1200011`, zero alternate stack, a child-TID pointer, then
`wait4(-1, status, 0, NULL)` and a later WNOHANG ECHILD query. This is evidence
of a required ABI shape, not an acceptance sequence to encode in the kernel.
Its host PIDs, stack addresses and paths do not enter implementation policy.

Risk is high: snapshots, memory isolation, shared-description lifetime, native
register/TLS/extended state, exit/exec and wait publication interact. Review follows
the authority chain and separately audits each owned backing interval, publication
point, reference-count update, restore operation and erasure. Independent user
witnesses and wrong-state tests must supplement model/structural checks.

The semantic process/child/wait ownership applies to x86-64, AArch64 and RISC-V.
Each provider needs its own native clone argument order, register/extended-state
save rules, user-transfer layouts, translation invalidation and physical-backing
proof. The snapshot representation and serialized schedule are implementation
choices, not new core-language tasks, shared references or source assembly.
See the [architecture](topal-kernel-architecture.md),
[file descriptions](reusable-static-file-descriptions.md),
[process observations](reusable-process-observations.md) and
[remaining acceptance work](../step-4-status.md).
