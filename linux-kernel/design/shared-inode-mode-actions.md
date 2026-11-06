# Shared inode modes through ordinary metadata actions

`TK-DEC-280`; systems-only work under standing chat authorization.

## Purpose and authority

Add the chmod family to the existing owned RAM filesystem and anonymous pipes.
Standard utilities need mutable permissions independently of their file data
and independently opened descriptions. Use the newly qualified
[unsigned fields](unsigned-region-fields.md) for a real multi-field ordinary
Topal operation. Keep the [initial architecture](topal-kernel-architecture.md),
[owned files](owned-volatile-files.md), [named lifetime](named-volatile-files.md),
[borrowed regions](borrowed-kernel-regions.md),
[instances](instance-scoped-region-actions.md),
[resources](../../docs/resources.md) and [effects](../../docs/effects.md).
This implements `TK-KERNEL-OBJECT-001`, `TK-KERNEL-STATE-001` and
`TK-KERNEL-UAPI-001`; it changes no core language meaning.

The qualified target remains x86-64 and pinned Linux 7.2.9. This increment covers
the prepared root credential and existing bounded object kinds. General
credentials/capabilities, ACLs, symlinks, mounts, persistent devices and directory
creation remain acceptance work. Recording permission bits does not qualify
unimplemented non-root access control or set-ID execution. Existing open
descriptions retain their granted access rights after chmod.

## Linux personality boundary

The pinned `fs/open.c` definitions select x86-64 syscall numbers 90 (`chmod`),
91 (`fchmod`), 268 (`fchmodat`) and 452 (`fchmodat2`). The last takes flags;
the original `fchmodat` has three arguments and always uses zero flags. Adapt
the native mode as `umode_t`, then retain its low `07777` permissions. High
argument bits and supplied file-type bits cannot replace the actual inode type.
This is a Linux boundary conversion, not a truncating conversion of core Nat.

For `fchmodat2`, reject flags outside `AT_SYMLINK_NOFOLLOW | AT_EMPTY_PATH`
before copying a name or validating a descriptor. Adapt fd and flags through
their declared 32-bit native types before those checks. Copy/resolve the path before
acquiring mutation authority. Absolute paths ignore dirfd; relative paths use
the existing cwd/live directory service. An empty string requires
`AT_EMPTY_PATH` and resolves the referenced inode, including a regular file or
pipe; `AT_FDCWD` selects cwd. An unreadable or null path is EFAULT even with
`AT_EMPTY_PATH`: this syscall's pinned `getname_uflags` copies a string and does
not use the special nullable pathname acquisition used by some other syscalls.
No symlink exists in the current namespace, so the qualified nofollow flag has
no different traversal effect; this does not implement symlinks.

Invalid fd yields EBADF. Descriptor access mode is not a chmod permission check:
a live read-only description may change mode with the prepared owning credential.
Path lookup failures precede target mount-write denial for chmod, unlike the
existing unlink-parent ordering. Existing immutable boot-root objects retain
their genuine read-only filesystem disposition; they must not report successful
mutation. Mutable targets are all existing RAM regular objects, `/tmp` and
anonymous pipe inodes. Do not turn unimplemented target kinds or credential
profiles into success. Native error order must be compared with the pinned
reference kernel before claiming syscall qualification.

Select `static-linux-dispatch-v5` for the new operations: v4's exact 61
operations plus `ChangeMode` (90), `ChangeModeByFile` (91), `ChangeModeAt`
(268) and `ChangeModeAtFlags` (452), plus Unsupported. Earlier four vocabularies,
source tags and named native adapters remain frozen. V5 requires an explicit
`inode-mode-library` root declaration using the existing qualified instance
action interface; its exact caller-derived instance is `inode-metadata`.
Selecting a source object alone does not supply native inode authority.

Success preserves file type, identity, links, contents, size, page associations,
description positions/rights/flags, directory entries and atime/mtime. It sets
exactly the requested permission bits and ctime under the existing
realtime-seconds policy. Umask applies only to creation and does not filter a
chmod request. A request equal to the old mode still samples the mutation clock
and updates ctime; equal clock seconds need not produce different stored bytes.
The prepared credential retains set-user-ID, set-group-ID and sticky bits;
non-root bit stripping and authorization remain explicitly unqualified.

## Shared state and ordinary source

Represent the personality's inode metadata as four explicit little-endian
eight-byte fields: full mode, atime seconds, mtime seconds and ctime seconds.
The encoding is a private schema, not an architecture word access or a source
address. Type and permissions are semantic state at higher interfaces. A
copied metadata snapshot grants no inode access.

One inode, rather than one description, owns this block. All independent opens,
dup/fork aliases and both pipe endpoints observe it. Unlink retains the old
inode under open descriptions; recreating its spelling creates an independent
inode and metadata block. Last-reference reuse begins with creation metadata,
without retaining an earlier object's permissions. `/tmp` has its own block;
entry creation/unlink update its mtime/ctime without resetting its mode.
Pipe creation time remains separate from its mutable atime/mtime/ctime.

Use a distinct `inode-metadata` instance of the existing two-Nat/Boolean framed
region action context. Facts are already adapted permissions in `[0,4095]`
and admitted realtime seconds fitting u64. After proving a complete 32-byte
window and all data bounds, source reads the existing mode, preserves its
type, and uses ordinary arithmetic plus checked unsigned stores for the new
mode at offset zero and ctime at offset 24. The other words remain unchanged.
The controller must lend the real persistent block; a copied candidate followed
by native publication would not implement this operation.

Independent source data tests cover narrow views, excessive/large Nat facts,
malformed type encodings and read-only views. These reject without owner
mutation. Successful earlier stores are not rolled back by later failure.
After admitted entry, false, malformed state, abandonment or failed cleanup
must stop without resuming the owner. This is not a new recoverable Linux error.

The native controller owns lookup, prepared-credential/mount checks, clock
acquisition, exclusive access and source admission. It validates the complete
post-state after true and complete reclamation, including preserved type,
requested permissions/ctime and unchanged atime/mtime. Guards may check the
effect but must not implement the source mutation. One instance has disjoint
hooks, code/data, arena, frame, control and persistent history from existing
root, position and page actions. Bind exact actual filesystem/pipe owners and
live identities before lending; equal numeric identities or labels grant
nothing. Parent/child restoration and object destruction must not rewind
invocation generations. Source false after either store remains terminal after
complete temporary cleanup, without a rollback claim.

The portable model first uses the same owned metadata bytes as its normal file,
directory and pipe observations and transfer/lifetime laws. It models the
admitted root operation directly; this is neither execution of an opaque source
body nor a physical erasure or complete syscall-error-order proof. Actual loans
and opaque completion must subsequently be modeled and independently executed
before machine qualification. Preserve that distinction in each staged report.

## Native placement and portability

Keep existing storage extents and snapshot boundaries. Regular-file mode and
three timestamps already form the 32 bytes at object offset 24. Use free
filesystem-header bytes 32 through 63 for `/tmp` metadata, leaving the identity
counter at zero and page map at 24 intact; retire the prior two timestamp words
at offsets 8 and 16 to reserved zero rather than duplicate live authority.
In each 128-byte pipe record use offsets 64 through 95 for mode/timestamps;
move its separate creation-time observation to previously unused offset 96.
These placements require new exact evidence versions and updated native
creation/stat/transfer/clock/cleanup consumers. Reject stale or overlapping
metadata. Never copy an old layout version into a new qualification record.

No new integer, pointer, intrinsic, effect or assembly facility is needed.
AArch64/RISC-V adapters supply their syscall numbers and argument conventions;
source field widths/order, metadata ownership and lifetime do not change with
CPU register width. The Linux personality's type/permission encoding is explicit,
not a requirement on a future native Topal filesystem representation. The
current exclusive single-CPU service serializes the operation; SMP needs its
own admitted synchronization protocol. Keep the existing 65536-byte temporary
and 4096-byte source-stack limits and prove the complete costs before binding.

## Risk, alternatives and qualification

Risk is high because shared metadata, native argument ordering, ownership,
source loans and reuse interact. Review in authority order, self-review the
complete diff and use independent byte, Linux, native and debugger oracles.
No delegation is required for the model stage; later independent executable
oracles are mandatory before qualifying a native operation.

Changing each description independently breaks inode sharing. A native-only
mutation would bypass the ordinary source implementation objective. Copying a
candidate block would exercise no actual persistent-field loan. Enlarging
existing storage or temporary limits is unnecessary. Selecting only writable
descriptions would impose an access restriction absent from Linux fchmod.

Use four staged commits in the existing initial Step 4 PR:

1. This systems design, exact state/adapter/action rules, traceability and
   portable shared-state model, with exhaustive permission/type preservation
   and independent alias/name/pipe/reuse/refusal cases.
2. Ordinary source, complete source/object/cost binding, actual-owner loan
   model and independent guarded native/GDB components, including false-after-
   store and complete cleanup. No new core-language semantics.
3. All four syscall adapters, exact source-controlled actual inode mutation,
   new evidence versions, pinned Linux comparisons, physical VM cases and
   unmodified BusyBox chmod/stat/shell workloads. Retain old dispatch controls.
4. Complete affected Rust/lab/native/VM qualification and accurate scope records,
   lossless evidence, review and publication. Initial distribution/container/
   VM-host acceptance remains until its own tests pass.

Primary research inputs are the locally cached, manifest-pinned Linux 7.2.9
`fs/open.c`, `fs/namei.c`, `fs/attr.c`, `include/uapi/linux/stat.h`,
`include/linux/stat.h`, `include/linux/types.h` and
`arch/x86/entry/syscalls/syscall_64.tbl`, with their complete file hashes in
the [research record](../knowledge/x86_64/inode-mode-research.json).
