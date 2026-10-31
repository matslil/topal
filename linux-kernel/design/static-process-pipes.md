# Owned anonymous pipes and condition-bound process waits

Status: systems-only direction adopted under standing chat authorization,
`TK-DEC-256`; implemented and physically qualified in the
[Step 4 report](../step-4-static-process-pipes.md).

## Purpose and scope

The [resumable pair](resumable-static-process-pair.md) can cooperate through
shared file offsets, but has no byte-stream IPC. Standard shell command
substitution needs anonymous pipes, inherited endpoints, EOF and child wait.
This refinement adds shared FIFO objects and condition-bound read/write waits
to the existing two-process/static-image profile. It does not qualify arbitrary
process tables, multiple-child pipelines, poll/epoll, splice, named FIFOs,
packet/notification pipes, filesystem mutation or general signal delivery.

A host-Linux control using the digest-pinned BusyBox observes `pipe2` with
flags zero, fork-like `clone`, parent reads through EOF, and child wait for
`value=$(/bin/busybox echo pipe-child); echo "$value"`. The child redirects
and closes inherited endpoints, execs the unmodified utility and writes bytes
through ordinary stdout. These are required interface relationships, not a
command sequence or chosen output to encode in provider dispatch.

The selected implementation reserves four independently owned anonymous FIFO
objects, each with a 4096-byte ring, under the prepared outer owner. This
capacity matches the native atomic-write boundary and is a declared resource
limit, not an application-dependent choice. An eager generic VFS/allocator is
the later general implementation. Host pipes, substituted output and coupling
FIFO lifetime to either process snapshot are invalid alternatives.

## Endpoints, publication and reclamation

Native `pipe` and `pipe2` allocate two lowest free descriptor slots and distinct
read/write descriptions referring to one fresh FIFO identity. Native flags use
their declared integer width. The first profile admits ordinary stream pipes
with optional O_CLOEXEC and O_NONBLOCK. Unknown bits are invalid; valid Linux
packet/notification shapes remain explicitly unqualified. CLOEXEC belongs to
each slot, while nonblocking status belongs to the shared endpoint description.
Duplication/fork inherit those identities; exec filtering and close release
only the corresponding references.

Creation checks the complete native pair output, reserves every slot,
description and FIFO resource before publication, and publishes two exact
32-bit descriptors. Failure returns no partial pair or authority and leaves
caller bytes/resource counts unchanged. Resource errors follow declared native
admission order; a user candidate cannot justify leaking a reserved endpoint.

FIFO storage and identity/history belong to the shared outer owner, outside
private process snapshots. Closing the last reference to an endpoint updates
peer presence and wait readiness. Closing both endpoints erases and reads back
all payload and object metadata before reuse. Generation identities cannot be
reused while any endpoint/wait retains them. Parent termination releases inactive
child endpoints before active descriptors and verifies complete FIFO teardown.

Pipe descriptions report their native access/status flags through the existing
description service. Pipe metadata uses FIFO type/permissions, coherent anonymous
device/inode identity, zero regular-file size and the qualified block/atomic
write size. Timestamp fields must derive from the retained clock view rather
than fabricated calendar constants. Seek admission first resolves the descriptor
and native unsigned mode: a live nonseekable endpoint returns ESPIPE for modes
zero through four, but a larger mode returns EINVAL before object dispatch.
Unused offset values grant no authority or position. Unsupported pipe control operations must
remain explicit; they cannot be treated as regular-file or terminal success.

## Byte transfer and waits

Read and write resolve the exact live endpoint and its direction before user
transfer. Numerical range admission and complete live permissions for bytes
actually transferred precede payload/cursor mutation. User bytes are data;
neither their content nor application identity selects FIFO behavior. Consumed
ring bytes are erased, shared cursors/counts remain coherent through wrap, and
an empty buffer with no writer returns EOF. A read need not fill its request.

Writes up to 4096 bytes are atomic: they transfer completely or wait/return
EAGAIN without partial publication. Larger writes may return an actual positive
partial transfer bounded by available space. Nonblocking empty/full operations
return EAGAIN as required. Zero-length and fault behavior follows native
admission rather than inventing peer activity or consuming data.

A blocking empty read with a writer, or an insufficient-space write with a
reader, atomically records its exact FIFO generation and readiness condition,
parks a native retry continuation and selects an eligible owned counterpart.
Wake eligibility follows published bytes/space or last-peer closure. It cannot
come from a matching descriptor integer, an unrelated FIFO, a stale generation
or an ordinary yield. The retry re-resolves authority and user permissions in
the restored process generation before committing bytes. No checked user pointer,
partial transfer or live kernel continuation is borrowed across suspension.

The parent may wait for child exit or for pipe readiness; the child may wait
for pipe readiness. A process waiting for child exit cannot be woken by pipe
traffic. When no process can make progress in this closed profile, no fabricated
success/EOF/error is permitted; the owner retains its blocked obligations.
Child exit releases its endpoints, publishes peer-closure readiness, and retains
the normal child-status relationship. Restored retries cannot return before
their condition or a real error holds.

The first broken-write error qualification requires SIGPIPE to be ignored,
then returns EPIPE with no payload mutation, including when that ignored signal
is masked. Nonignored blocked/default/user-handler signal cases remain
unqualified until pending-signal/delivery lifetimes are implemented;
the provider must not silently return EPIPE while omitting required signal state.
This limit does not affect an ordinary pipe whose reader remains live.

## Qualification and portability

The model and independent actual user witness must cover creation/error rollback,
native widths/flags, endpoint direction, aliases/shared flags, fork and exec
lifetimes, ring wrap, atomic short writes, larger partial writes, nonblocking
empty/full, EOF after last writer, ignored-SIGPIPE EPIPE, resource exhaustion,
generation reuse and complete teardown. Physical execution must force both read
and write blocking, prove the exact counterpart progressed, retain private/native/
TLS/extended state, and demonstrate readiness and once-only retry publication.
An unrelated condition or controlled stale/wrong FIFO identity must be rejected
independently of ordinary completion markers or timeout.

Unmodified BusyBox command substitution must pass through the shared services.
A changed payload and a payload exceeding ring capacity must establish that the
provider transports application-selected bytes and makes cooperative progress.
Existing application/library/clock/child evidence must remain valid on one kernel.

Risk is high across shared endpoint counts, atomic byte publication, private
snapshot exclusion, wait registration/readiness, exec/exit, failure rollback
and reclamation. Review follows each resource through creation, aliasing,
suspension, peer closure, resumed transfer and destruction. Architecture-neutral
FIFO/wait semantics apply to AArch64 and RISC-V; native descriptor-pair layouts,
argument widths, retry/entry rules and memory-ordering evidence remain per target.
The four-object/4096-byte allocation and syscall numbers are the first x86-64
binding. Other providers choose capacity appropriate to their native page
granularity, report that actual capacity and preserve the separate Linux atomic
write bound. Neither storage-page size nor retry-instruction width becomes
portable source meaning.

The refinement changes systems features only. Native realization remains behind
qualified machine elements. General ordinary Topal subsystem modules, broader
signal/IO/process semantics, distribution replacement, Docker/Podman and VM-host
qualification remain in [Step 4 status](../step-4-status.md).
