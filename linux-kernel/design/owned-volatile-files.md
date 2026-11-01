# Owned sparse volatile regular files

`TK-DEC-265`; systems-only extension adopted under standing authorization.
The [implementation report](../step-4-owned-volatile-files.md) records actual
sparse bytes, signed interval/error ordering, shared process lifetime, complete
owned storage evidence, Linux controls and physical reclamation qualification.

## Purpose and initial boundary

The immutable boot namespace cannot support application-created data. Introduce
an owned RAM filesystem service, initially reached through `O_TMPFILE` in `/tmp`.
This gives ordinary Linux callers writable regular files without recognizing
command text or adding a separate data path for each application. The object
service is also the intended backing for later linked-file namespace mutation.
The existing executable archive remains immutable.

This increment qualifies anonymous regular files, sparse contents, shared open
description positions, read/write, seek, truncate, coherent metadata and full
destruction. Named creation, links, rename/unlink, writable mappings, storage
devices, durability and general mount operations remain later acceptance work.
`O_TMPFILE` success alone is not a distribution or container qualification.
This is a Topal RAM filesystem with declared allocation, timestamp and transfer
policies; it does not claim to implement every observable property of Linux's
`tmpfs` driver. Linux controls establish the retained generic UAPI/value laws.

The initial selected filesystem has four simultaneously live regular-object
slots and eight 4096-byte resident pages shared by those objects. Capacity is
an explicit filesystem resource budget, not a per-file maximum length. Logical
size and position may extend through the native signed file-offset domain.
Holes consume no data pages. Exhaustion must not silently shorten file length,
wrap offsets, substitute data or borrow a host allocation.

## Portable ownership and sparse storage

Separate the filesystem owner, stable object identity, object slot, resident
page, open description, descriptor slot and native inode/offset representations.
Numbers and page indexes confer no authority. An object owns its logical size,
mode/credentials, timestamps and page associations. Each resident page belongs
to exactly one live object generation and one logical page index. Unallocated
pages read as zero. Free metadata and payload are completely clean.

A reusable physical slot must not resurrect a previous object identity or data.
The filesystem owns monotonic identity issuance and rejects exhaustion before
publication. Anonymous objects have link count zero. Duplication and fork retain
the existing shared open description and position; they do not clone contents.
Last-description destruction releases every page and erases the entire object.
Exec retains eligible descriptions and closes close-on-exec slots normally.
Active/inactive process restoration must not roll back shared object contents,
size, page accounting, timestamps or description position. Terminal cancellation
releases both processes before reclaiming the complete shared filesystem store.

The selected single-CPU syscall service serializes these transitions. A later
concurrent implementation must replace that assumption with an explicit object
and allocation synchronization domain; ordinary immutable Topal values do not
gain shared mutation or native-pointer meaning.

## Namespace and creation

Publish `/tmp` as a directory backed by the selected RAM filesystem. Its
identity, metadata, root enumeration, relative lookup, parent traversal and cwd
behavior must agree with existing namespace services. Root/bin/archive inode
identities retain their meanings. The directory grants access only through
normal checked lookup and the prepared root credential; knowing its inode does
not create a writable-file capability. The initial directory mode is 01777.

`open` and `openat` share creation. `O_TMPFILE` includes native `O_DIRECTORY`,
requires writable access, resolves a live directory and allocates a fresh object
and open description. Its checked native mode is filtered by the retained
process umask (initially 0022); native file-type bits come from the service.
The ordinary `umask` operation returns the previous mask and retains only its
native 0777 permission bits. Fork copies the mask into private process state;
exec preserves it. It does not change modes of existing objects or mutate the
other process's mask.
`O_CLOEXEC` remains descriptor state. `O_EXCL` records the Linux no-link intent
even though linking is outside this increment. Ordinary status flags and access
rights remain description state. Read-only and write-only descriptors must not
gain the other direction through the backing implementation.

Validate the selected native flag/mode contract before copying the name. Copy a
complete name before descriptor allocation; allocation precedes component walk,
as in the existing open contract. Keep distinct missing, non-directory,
read-only-filesystem, invalid-flag, bad-name, descriptor-exhaustion and
filesystem-capacity failures. Wrong flags, failed user copies or failed lookup
must create neither an object nor a description. Object allocation and descriptor
installation form one exclusive publication; every failure erases private work.
No anonymous file appears in `getdents64` or becomes reachable by a synthesized
pathname. Existing normal opens of the new directory remain directory opens.

The initial adapter explicitly qualifies native access modes zero/one/two and
the declared compatible open modifiers; other native protocols require their
own qualification. Contradictory `O_CREAT|O_DIRECTORY`, malformed `O_TMPFILE`
and read-only `O_TMPFILE` follow the pinned baseline's error/validation order.
No unsupported creation or linking request may manufacture success.

Add `static-linux-dispatch-v2` for the enlarged closed operation vocabulary:
the 55 existing names plus `TruncateFile` (x86-64 selector 77) and
`FileCreationMask` (95), and `Unsupported`. Preserve v1's exact source file,
55-name contract and private export as an independently supported compatibility
profile. Each version supplies its own complete named tag adaptation and native
result validation; exactly one may occupy the dispatch slot. The versions share
one exclusive invocation domain and existing stack/arena reserves. The primary
root selects a separate v2 source file. Neither version may silently reinterpret
the other's Enum or redistribute an unsupported captured selector in native code.

## Transfer, seeking and truncation

Resolve descriptor liveness and access rights before touching user bytes.
Numerical user-range admission precedes zero/EOF handling; actual nonempty
transfers check live mapping permissions. Reuse bounded owned transfer staging,
never retain a user pointer and preserve native return/register semantics.
After numerical user-range admission, validate the complete requested count and
the current description-position/count interval in the native signed domain.
An invalid count or overflowing interval returns `EINVAL` before clipping or
EOF disposition. This also refines existing borrowed immutable-file reads.
For append, that initial check uses the current description position; selecting
actual EOF happens afterward, as in the pinned `rw_verify_area`/write-iterator
ordering. A valid seek alone does not make every later count admissible.
The first transfer qualification covers complete admitted prefixes; it does not
establish Linux partial progress across a fault inside one such prefix. Report
that limitation explicitly rather than generalizing whole-prefix tests.

Reads select the prefix bounded by request, remaining logical length, the
256-byte transfer quantum and the current logical page boundary. A hole supplies
actual zero bytes. Publication advances the shared position by exactly the
returned prefix; failed output admission preserves contents, size and position.
Zero/EOF reads allocate nothing. Reads cannot reveal stale freed page bytes.

Writes copy a complete admitted prefix before publishing it. Select at most the
request, transfer quantum, page boundary and representable file-offset remainder.
Append selects current logical EOF at the serialized write operation, rather
than at open or seek. An existing resident page remains usable when the pool is
otherwise full. A missing page consumes one clean page or returns `ENOSPC` with
contents, size, position and accounting unchanged. Partial positive progress is
permitted across successive syscalls; a later page failure cannot revoke bytes
already returned. Zero writes allocate nothing and do not move position or grow
the file. After native interval admission, an append selected at maximum EOF
returns `EFBIG`; a normal overflowing position/count already fails `EINVAL`. Finite
filesystem page exhaustion is not an artificial maximum file size.

Successful write publication copies the selected bytes, preserves unrelated
bytes, extends size only as required and advances the shared position to the
actual end. Gaps remain zero. Failure cannot consume a page reservation or
publish a partially installed object/page association. Source/library evaluation,
if selected, must finish and reclaim temporaries before storage publication.

`lseek` set/current/end uses checked signed arithmetic and leaves position
unchanged on rejection. Seek beyond EOF does not allocate or extend size.
`SEEK_DATA`/`SEEK_HOLE` use the resident-page map: a resident page is data and
an absent page is a hole, clipped at EOF. EOF is the final implicit hole;
offset at/beyond EOF returns `ENXIO`. Negative offsets and invalid selectors
follow native error rules. This reports allocation extents, not byte nonzeroness.

`ftruncate` requires a writable regular description and a nonnegative admitted
length. Growth creates holes without allocating pages or changing position.
Shrink releases pages wholly beyond the new EOF and zeroes the truncated suffix
of a surviving page before exposing the new size. Regrowth must return zeros
for removed bytes. Truncation through an alias is immediately visible through
every other alias and inherited descriptor. Equal-size truncation preserves
data and allocation. Unqualified flags or descriptor types do not grant truncate
authority. No signal delivery or finite file-size rlimit is introduced here;
the selected file-size limit remains unlimited within the native signed domain.

## Observation, realization and portability

`fstat` reports stable device/inode, regular mode/credentials, zero link count,
logical size, page-based allocated block count, qualified block size and retained
timestamps. Reads/writes/truncation use the selected filesystem timestamp policy
and clock resolution. The initial mount selects `noatime`; creation initializes
all three times from the retained realtime-seconds service, and successful
nonzero writes and size-changing truncations update modification/change times
at that declared resolution. Directory/path and descriptor observations must agree;
temporary files have no directory entry. Metadata copying uses complete checked
native output and erased private staging. Actual page budget and inode capacity
must be documented and bound to the selected artifact; no host filesystem or
userspace service supplies data.

The native provider realizes byte access, allocation ownership, checked copies
and publication/erasure over explicit supervisor storage. These are trusted
systems mechanisms, not new core language effects. Portable semantic models
must independently describe size, holes, page ownership, shared positions and
failure atomicity. Native representations and Linux argument adaptation remain
separate from those laws. Further migration of personality policy into ordinary
Topal modules remains required by Step 2; this storage refinement does not
qualify arbitrary source mutation or general persistent allocation.

X86-64 supplies native flag values, offset/metadata encoding, page size and
mapping/copy realization. AArch64 and RISC-V must bind their own ABI and storage
layout without inheriting x86 registers, encodings or raw addresses as portable
meaning. Page capacity, object capacity and transfer quantum are selected
resource bounds, rather than architecture-independent language constants.

## Qualification and review

Host-backed files were rejected because they would replace kernel storage with
an external service. Per-descriptor data copies cannot express shared contents.
A dense fixed-length buffer would incorrectly turn physical capacity into a
maximum logical file size. Sparse owned pages establish reusable volatile
storage before adding persistent devices and linked namespace transactions.

Risk is high: user copies, sparse allocation, aliasing, truncation and reclamation
cross process lifetimes. Review must check identity issuance, complete failure
atomicity, live-slot/description accounting, unique page ownership, clean free
storage, shared history during restoration and last-reference release.

Required independent cases include temporary-file flags/modes/name errors,
lowest-slot allocation, capacity/exhaustion/reuse, read/write rights, exact bytes,
zero/EOF transfers, page boundaries, sparse large offsets, append, seek data/hole,
shrink/regrow zeroing, metadata and allocated blocks, bad/overflowing/guarded user
buffers, aliases, fork/exec inheritance and close-on-exec, active/inactive
termination and complete store/temporary erasure. Compare native observations
with a pinned Linux control using the same configured filesystem budget.
Physical tests must execute ordinary caller-issued syscalls; host-prepared output,
fabricated file bytes and command-specific adapters cannot qualify the service.
Existing namespace, source policies, command corpus, process-pair and FIFO tests
remain regressions. Until native and physical evidence exists, the capability
must remain listed as planned rather than executed.

Primary reference: the pinned Linux 7.2.9 source in `fs/open.c`
(`build_open_flags`, `do_sys_openat2`, truncation), `fs/namei.c` (temporary-file
creation), `fs/read_write.c` (range/rights, transfer and offset limits), and
`mm/shmem.c` (sparse pages, size, allocation accounting, seek and truncation).
These sources establish external behavior; their internal structures are not
the Topal object model. See the [architecture](topal-kernel-architecture.md),
[file descriptions](reusable-static-file-descriptions.md),
[namespace](reusable-readonly-namespace.md) and [baseline](../baseline.md).
