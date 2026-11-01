# Source-defined regular-file open plans

`TK-DEC-266`; systems-only refinement adopted under standing authorization.

## Purpose and boundary

Move the reusable open disposition into ordinary Topal before introducing linked
files. The selected `file-open-decision-v1` interface occupies the existing unique
`kernel-library` slot. Preserve the original `boot-open-decision-v1` source,
vocabulary and private ABI as a compatibility profile. Core-language meaning,
source assembly and resource authority are unchanged.

The new source function accepts eight Boolean facts, in order: write access
requested, resolved target is a directory, directory required, selected view
is mutable, target exists, creation requested, exclusive creation requested,
and truncation requested. These are semantic facts derived by the owner after
native flag admission, complete copied-name acquisition, free descriptor
selection and the applicable lookup. They contain no flags, pointers, inode,
errno or native representations. Write access excludes truncation, which has its
own fact: Linux permits an otherwise read-only description to request truncation.

The result is nominal `FileOpenDecision`, with semantic names `OpenDescription`,
`NotDirectory`, `DirectoryWrite`, `ReadOnlyView`, `Missing`, `AlreadyExists`,
`CreateDescription` and `TruncateDescription`. Source order is private. For an
existing target, exclusive creation first yields AlreadyExists; directory
requirements reject regular files; creation, write access or truncation of a
directory yields DirectoryWrite; mutation of an immutable regular view yields
ReadOnlyView; truncation of a mutable regular target yields TruncateDescription;
otherwise OpenDescription. For an absent target, no create yields Missing;
creation in an immutable view yields ReadOnlyView; a directory requirement yields
NotDirectory; otherwise CreateDescription. The function is total over all 256
fact vectors, including combinations the native caller never emits.

Lookup owns prefix failures and trailing-component rules, and validates native
O_CREAT|O_DIRECTORY before the source call. The plan does not replace pathname
walking. In particular Linux's O_CREAT with trailing slash is rejected by
lookup before final open disposition. Future linked-file creation must preserve
these rules rather than treating every lookup failure as an absent final name.

## Ownership and native binding

The source returns a plan, never a resource grant. Native code validates the
selected action against the owned view, resolved object and requested operation
before reserving or publishing storage. At this stage existing immutable and
directory opens consume the plan; O_TMPFILE uses an absent regular target in
the resolved directory's view and consumes CreateDescription. Its native
adapter still verifies directory identity, write access and anonymous allocation
conditions. New plans cannot make the immutable executable writable or turn a
missing path into a file. Named creation/reopening/unlinking remains subsequent
work, not an executed capability of this increment.

A wrong result is fail-closed before owned mutation. Errors are adapted to Linux
only at the native boundary. The finite library's existing pure Boolean/Enum
profile, ordinary frontend/backend, closed acyclic helpers, GPR-only code and
1024-byte complete additional stack bound apply. A generated adapter unpacks the
facts and maps checked nominal tags by name. Each interface uses its own private
export. Artifact metadata, provider selection and linked-code validation must
agree exactly and reject mismatched or simultaneous exports. No unchecked code
pointer, writable library global, hosted runtime or separate interpreter is added.
The selected code binding survives exec/fork and is erased on terminal return.

X86-64 owns flags, calling convention, native error numbers, register saves and
stack alignment. AArch64/RISC-V can reuse the facts and plans with their own
native adapters and complete stack evidence. Filesystem budgets are not language
constants. This extends the existing finite library, not arbitrary effectful
source mutation.

## Qualification and review

Risk is high: a source choice precedes native resource publication. Review covers
complete finite input/result adaptation, current and frozen interfaces, stack
closure, binding/version confusion, native resource revalidation, no mutation on
rejection, and binding destruction. Test all 256 vectors with independent native
clients and reversed Enum order; invalid masks must reject. Execute actual
ordinary helper calls in QEMU for immutable/directory and temporary creation,
check ABI/frame evidence and reclamation, and reject a deliberately wrong create
plan before filesystem publication. Existing command, process, sparse storage
and frozen policy qualifications remain regressions. Source source/profile
changes must not be described as qualifying named files or complete Linux.

References: pinned Linux 7.2.9 `fs/namei.c` (`lookup_fast_for_open`, `do_open`,
`vfs_tmpfile`) and `fs/open.c` (`build_open_flags`), the
[finite policy boundary](freestanding-kernel-policy-library.md),
[owned sparse files](owned-volatile-files.md) and
[approved architecture](topal-kernel-architecture.md).
