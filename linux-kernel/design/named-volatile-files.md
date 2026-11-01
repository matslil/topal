# Named files in the owned RAM filesystem

`TK-DEC-267`; systems-only refinement adopted under standing authorization.

## Purpose and boundary

Extend the [owned sparse filesystem](owned-volatile-files.md) with caller-created
regular names in `/tmp`, independent reopening, truncating opens, unlink and
coherent pathname/directory observations. This enables ordinary shell redirection,
append, reads, listing and removal through the same file objects and descriptors.
The boot executable and its view stay immutable. This does not add persistent
storage, new directories, hard links, rename, symlinks or general mounts.
The [Step 2 architecture](topal-kernel-architecture.md) and existing distribution,
container and virtualization acceptance work retain their scope.

Select the existing ordinary `file-open-decision-v1` policy in the primary root.
Use its semantic facts and named plans; native code still revalidates owned
preconditions before publication. Preserve the frozen boot-open interface as an
explicit compatibility selection. Extend the selected syscall source interface
to `static-linux-dispatch-v3`, adding `Unlink` and `UnlinkAt` (x86-64 selectors
87 and 263) to v2's exact 57 operations. V1 and v2 retain their exact vocabularies,
exports and checked adaptation. All three occupy one exclusive dispatch slot.
Unsupported operations/argument profiles must remain reported as unqualified;
this increment does not claim directory removal through AT_REMOVEDIR.
No core language meaning or source assembly changes.

## Names, objects and descriptions

The filesystem keeps its four-object/eight-page budget. A linked name owns one
object reference, independent of its zero or more open descriptions. Each initial
name has exactly one link; hard linking is outside this profile. Names contain
one through 255 raw bytes excluding NUL and slash. Dot and dot-dot keep pathname
meaning and cannot become regular names. Exact bytes determine equality; no host
lookup, Unicode normalization or command identity selects an object.

Independent open calls allocate distinct descriptions at position zero and share
the same object and sparse contents. Dup/fork aliases retain their existing
shared description/position. Closing a description cannot free an object still
linked or held by another independently opened description. Unlink erases the
name and sets link count zero, while open descriptions retain the original
contents, metadata, identity and positions. Recreating that spelling issues a
fresh object identity. Only an unlinked object without live descriptions releases
its pages and slot. Reclamation erases the name, association, object, page
metadata and payload. Closed linked files remain under the namespace owner's
lifetime. Terminal cancellation first drops both processes' descriptions, then
reclaims remaining namespace-owned objects and verifies the whole store clean.
Private process restoration must not roll back names or shared contents.

Keep filename storage, object identity, object slot, page owner, description
identity, descriptor slot and directory cookie distinct. Mode/umask applies only
to new objects; reopen does not reapply mode. Close-on-exec is per descriptor,
access/append flags and position per description, contents/links/times per object.
The model must use object identity for page ownership even when independently
opened description identities differ. A numeric inode or cookie grants no
filesystem authority.

## Lookup and open publication

Preserve native flag admission, complete copied name acquisition, lowest-free
slot selection and applicable pathname-error ordering. Parent traversal and the
final component are separate results. A missing prefix is not a creatable final
name. A regular component rejects further traversal or a directory requirement.
Absolute names ignore dirfd; relative names require cwd or a live directory
description. Copied bytes and pending parent/name facts remain private and are
erased before returning to userspace.

Normal open admits create/exclusive/truncate/append in the selected new policy
profile. O_CREAT|O_DIRECTORY remains invalid before copying. O_EXCL without
O_CREAT has no create effect. Final normal-component O_CREAT plus trailing slash
follows pinned Linux's lookup EISDIR disposition before final open planning;
root/dot/dot-dot retain their separate handling. Existing exclusive creation
returns EEXIST before filesystem write denial. Existing read-only creation may
open without mutation. An absent target in the immutable view cannot be created.

A CreateDescription plan requires a missing final regular name in owned `/tmp`,
then checks clean object/name/description capacity and obtains its timestamp
before installing any association. A truncate plan requires an existing owned
mutable regular target and requested truncation. Reserve the description and
obtain the clock before truncating; publication afterward has no recoverable
failure. O_RDONLY|O_TRUNC truncates using the owner's filesystem write authority
but returns a read-only description. It must not silently grant later write
access. Existing descriptions retain their positions while observing truncation.
All capacity/copy/clock/lookup failures preserve prior names, object data, size,
positions, allocation and descriptor installations.

## Unlink and directory observation

Unlink/unlinkat with the qualified non-directory flags resolves the parent and
last-component class before mutation. Pinned Linux checks parent mount write
access before final child lookup for unlink, unlike ordinary open disposition.
Do not replace that ordering with a full-target lookup: even a missing final
name in the immutable parent view can return EROFS. Dot/dot-dot/root and normal
trailing-slash cases retain their pinned errors. Invalid native unlinkat flags
are checked before copying; unqualified AT_REMOVEDIR must not pretend that rmdir
was implemented. Qualified failures are recoverable without mutation.

A successful unlink samples the clock before publication, erases the exact name,
updates object change time and parent modification/change times and either
retains or destroys the object according to its independent live descriptions.
Readonly views cannot gain mutation through an alias or relative dirfd.

Pathname stat, fstat and directory entries must agree on live identity, regular
mode, links, logical size, allocated blocks and timestamps. `/tmp` link count
remains two for regular names. Creation/unlink update its modification/change
time; opening an anonymous file does not create a directory event. Retain the
selected realtime-seconds/noatime policy. Open truncation updates object
modification/change time even when the previous size was zero.

Directory records include linked names only, never anonymous or unlinked objects.
Use exact bounded raw names, aligned native records and zero padding. Cookies
are filesystem-local opaque continuation values; aliases share the directory
description position and independent directory opens start from the beginning.
Document their selected slot-order, deletion/rewind and EOF policy. A largest
255-byte name needs a 280-byte native record, so it cannot use the existing
256-byte transfer staging. Provide separate bounded directory staging, erase it
on every return/error and admit the complete returned output prefix before
publishing cursor progress. No-fit and output fault must preserve the cursor;
partial successful batches may leave later records for the next call.

## Placement, portability and qualification

Use versioned filesystem evidence for exact name/directory-staging placements,
capacities, bounds and disjointness. The proposed native layout uses four
256-byte names at filesystem-relative 1536 and separate 512-byte directory
staging at 3072, within the existing metadata page. Existing payload begins at
4096; the whole filesystem remains 36864 bytes and process backing 311296 bytes.
Keep pending path state within erased private staging, disjoint from umask and
signal actions. Prove every fixed scan and complete native helper stack path;
never increase a memory/stack limit merely to pass validation.

Portable laws govern ownership, names, shared content, failure atomicity and
reclamation. X86-64 supplies flags, syscall numbers, metadata/dirent encoding,
page representation and calling conventions. AArch64 and RISC-V need their own
ABI adapters without inheriting register identities as language meaning.
The selected single-CPU syscall service serializes namespace transactions;
a later concurrent filesystem needs an explicit synchronization domain.

Risk is high: shared storage, namespace lifetime, user copies and object reuse
interact. Review requires independent model/native/pinned-Linux/physical oracles,
all source dispatch versions, malformed layout/version rejection and ordinary
source-policy execution before ownership publication. Qualify native names and
long records, exact contents/metadata, create/exclusive/truncate, private umask,
append, independent positions versus aliases, unlink-open and recreation,
closed-linked capacity, object/page/descriptor exhaustion, rejected copies,
relative/cwd/dot/trailing errors, exec/fork and both active/inactive cancellation.
Run unmodified BusyBox shell redirection, reads/counts/listing/removal from its
ordinary prompt. Complete reclamation and a deliberately wrong-data/association
case must be observed independently. Until this evidence exists, named mutation
remains planned rather than an executed compatibility claim.

References: pinned Linux 7.2.9 `fs/namei.c` (`lookup_fast_for_open`, `do_open`,
`filename_unlinkat`, `vfs_unlink`), `fs/open.c` (`build_open_flags`, truncation),
`fs/readdir.c` and `mm/shmem.c`, the [source plan](source-file-open-plans.md),
[file descriptions](reusable-static-file-descriptions.md) and
[namespace](reusable-readonly-namespace.md).
