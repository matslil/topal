# Shared namespace transactions for owned regular inodes

`TK-DEC-281`; systems-only refinement under standing chat authorization.

## Purpose and authority

Add rename and hard links to the existing owned RAM filesystem, enabling ordinary
`mv` and `ln` through shared filesystem services. Keep the
[Step 2 architecture](topal-kernel-architecture.md),
[named files](named-volatile-files.md),
[shared inode modes](shared-inode-mode-actions.md),
[borrowed regions](borrowed-kernel-regions.md),
[instance isolation](instance-scoped-region-actions.md) and
[core resource meaning](../../docs/resources.md).
This advances `TK-KERNEL-OBJECT-001`, `TK-KERNEL-STATE-001` and
`TK-KERNEL-UAPI-001` without changing the core language.

The selected filesystem retains four regular inodes, four directory-entry
slots, eight resident pages and 255-byte raw components. Entry capacity and
inode capacity are independent resource budgets: four names may refer to one
inode. Anonymous or open-unlinked inodes consume inode capacity without consuming
entry capacity. This is the next implementation increment, not a reduction of
the full distribution, containers, virtualization or retained-UAPI goal.
Directory creation/moves, symlinks, whiteout device nodes, other mounts,
credentials beyond the prepared root and general VFS remain required work.

## Names and inode lifetime

The directory owns entries containing copied component bytes and stable inode
identity. An inode owns contents, metadata, temporary-link eligibility and page
associations; an open description owns rights, flags and its shared position.
Neither the numeric inode identity nor a copied name is an authority. Lookup
must match a live object generation in the actual filesystem owner.

Replace the one-name-per-object representation with independently allocated
entry slots. Derive link count from all live entries naming an inode, or verify
an explicitly stored count against that table before admission. Every existing
lookup, create, unlink, stat, getdents, chmod, close, exec/fork restoration and
terminal consumer must use the same table. Reclaim an inode only when it has
neither directory entries nor open descriptions. Reusing an entry slot must
not reuse or resurrect an inode identity. Erase retired names and all released
object/page state; terminal cancellation erases all entries as well as objects.

Cookies enumerate dot/dot-dot, then the four entry slots, ending at six. They
are independent of object slots and inode identities. A fresh directory read
reports every current hard link with the same inode and exact raw name. Keep
the existing qualified no-snapshot-across-mutation contract, 280-byte maximum
native record, complete-copy-before-cursor publication and erased staging.
The preceding native evidence retains its earlier object-slot-based layout;
the new table needs a new exact native evidence version before qualification.

## Semantic transactions

All recoverable owner, name, reference, flags, lookup, capacity and clock checks
precede mutation. Names compare as raw bytes; no command identity or host lookup
selects behavior. One exclusive filesystem operation publishes each transaction;
future SMP requires an explicit admitted synchronization domain.

Hard link resolves an existing regular inode and a missing destination name.
It consumes one clean entry slot, adds one link, clears initial temporary-link
eligibility, sets inode ctime and parent mtime/ctime, and preserves contents,
size, allocation, permissions, atime/mtime, rights and description positions.
An existing destination is EEXIST. Entry exhaustion is ENOSPC even if inode
slots remain; it is not an invented maximum-link-count error. A path-selected
source requires a current entry. An empty-path descriptor may select an inode
with existing links, or a never-linked eligible anonymous temporary inode.
An O_TMPFILE|O_EXCL inode is ineligible. After the initial successful link and
subsequent removal of every link, a retained descriptor cannot resurrect it:
return ENOENT. Same-label foreign pools and stale descriptors cannot link it.

Rename with flags zero moves a source entry to an absent name using its current
slot, even when all entry slots are full. If a destination exists, atomically
replace that entry and remove the source entry; the victim loses exactly one
link and remains alive through other names or open descriptions. Update source
and surviving victim ctime and parent mtime/ctime; preserve data, size, mode,
atime/mtime, allocation and all positions/rights. A destination naming the same
inode is a complete successful no-op, preserving both names, all metadata and
invocation history. This applies to identical pathnames and distinct hard links.

RENAME_NOREPLACE returns EEXIST for every existing destination, including the
same spelling or inode, before the same-inode no-op. RENAME_EXCHANGE requires
both entries and exchanges their inode associations while preserving names,
link counts, contents, rights and positions; both inode ctimes and parent times
change. Exchange of two names for the same inode is a complete no-op. Missing
source or missing exchange destination is ENOENT. Invalid or incompatible native
flags are EINVAL. RENAME_WHITEOUT remains explicitly unqualified until device
nodes are implemented; do not disguise it as ordinary rename success or claim
that a fallback implements it.

## Linux adapters and research obligations

X86-64 uses rename 82, link 86, renameat 264, linkat 265 and renameat2 316.
Extend the current source dispatch in a new version; preserve the exact first
five vocabularies, tags and named adapters. Root declaration selects a distinct
`namespace-metadata` instance using the existing two-Nat/Boolean frame ABI.

Adapt fd and flags through declared native 32-bit types. Rename's admitted
flags are zero, NOREPLACE and EXCHANGE; invalid bits and EXCHANGE combined with
NOREPLACE/WHITEOUT have Linux's invalid-argument priority. Linkat admits
AT_SYMLINK_FOLLOW and AT_EMPTY_PATH; no symlink traversal is implemented by
merely accepting FOLLOW in a namespace without symlinks. Empty old pathname
requires EMPTY_PATH and the existing prepared-root authority. A null pointer
still has copied-string EFAULT semantics. Absolute paths ignore their dirfd;
relative old and new paths resolve against their separate checked directories.
Both path copies and intermediate/final component classifications must retain
their actual native error order, including dot/root/trailing-slash cases.

Research and then execute pinned Linux controls for cross-mount EXDEV, target
parent write denial, existing destination, old-target lookup, same-inode cases,
invalid flags and bad paths. Rename checks parent mounts before final target
disposition; hard-link creation has its own lookup/write ordering. Do not reuse
chmod or unlink ordering without evidence. Immutable boot objects/directories
cannot gain mutation through aliases or dirfds. Unimplemented object kinds and
flags remain separately recorded acceptance work.

## Ordinary source and native placement

Use actual persistent metadata, not a candidate map followed by native mutation.
Keep the existing 36864-byte filesystem and 311296-byte process backing. Reserve
four 16-byte entry records at filesystem offsets 64..127, holding inode identity
and name length; retain the four 256-byte names at offset 1536. Object offset 56
becomes a checked link count, offset 64 carries explicit temporary-link intent,
and the former per-object name-length word at 72 becomes reserved zero.
Object and metadata/page/payload placements stay disjoint and unchanged.

The exclusive controller lends the actual first metadata page, not file payload
pages. It places independently checked operation inputs and the copied new
component in the existing 512-byte directory staging at offset 3072 before
entry; these remain immutable input by the selected source contract. The two
ordinary Nat frame facts are admitted operation and realtime seconds. Source
reads checked semantic entry/object indexes and bytes from that staged input,
validates the full fixed geometry and all preconditions before stores, and
performs the real entry/name/link-count/ctime/parent-time changes with ordinary
checked byte codecs. It constructs no address or authority. All other metadata
and allocation-map bytes must remain exact. Source code must not implement
lookup by recognizing caller command text.

Staged-input immutability is a contract of the exact bound source: the metadata
view retains its existing read/write authority, and qualified source performs
no stores to input staging. It is not a new read-only-view facility or transport
ABI. Component/source-access evidence must establish that no-write property;
the independent controller also checks the complete unchanged input bytes.

The controller independently checks the complete expected persistent delta,
true completion and untouched immutable input, then erases staging, input frame,
arena, control and retired stack before owner resumption. Entry cleanup and
last-reference page reclamation occur under the same exclusive operation.
False, malformed state or cleanup failure after admitted writes is fatal after
reclamation, without rollback or owner resumption. Histories do not rewind on
rename, link, object reuse, exec or child restoration. Keep separate instance
code/data/hooks/storage; overlapping filesystem loans conflict with existing
position/page/inode actions. No-op and pre-admission failures issue no action.
The existing 65536/4096 source temporary/stack limits remain; prove complete
source and owning-controller costs before native binding.

This describes required source integration; the first stage is a direct
portable ownership model, not opaque source execution or physical cleanup.
AArch64/RISC-V use their own syscall numbers, argument conventions and native
encodings. Ownership, transactions, byte comparison, generation and cleanup
laws do not depend on x86 registers or raw addresses.

## Risk and staged delivery

Risk is high: name/inode identity, replacement lifetime, complete failure
atomicity, source grants, native error order and reclamation interact. Review
uses authority order, full-state model oracles, pinned source and Linux controls,
independent C/guarded-byte tests and physical GDB/QMP observations. Self-review
does not claim independent human review. No delegation is required.

Use four staged commits in the existing initial Step 4 PR:

1. Systems design/rules/traceability, pinned primary research and separate-name
   ownership model: alias/link counts, replacement/exchange/no-op, independent
   capacity, exact metadata/data preservation, foreign/stale refusal and cleanup.
2. Ordinary namespace source and actual-owner loan model; exact code/data/input/
   cost bindings and independent guarded native/GDB components, including
   false-after-store, changed association and complete temporary erasure.
3. All five Linux adapters, current versioned entry-table consumers, actual
   source-controlled mutation, pinned Linux comparisons, physical lifecycle/
   refusal controls and unmodified BusyBox mv/ln workloads. Preserve all frozen
   dispatch controls and distinguish unsupported device/directory operations.
4. Complete affected committed Rust/lab/native/VM qualification, lossless evidence,
   accurate remaining-scope records, all-open-PR conflict audit and publication.
