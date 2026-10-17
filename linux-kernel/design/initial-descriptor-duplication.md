# Initial standard-descriptor duplication

This record resolves `TK-DEC-118` as the first bounded realization of
`TK-KERNEL-OBJECT-001` and `TK-MAP-FD-001`. It refines the three initial live
bits into integer-slot mappings to fixed open-description capabilities without
introducing VFS allocation, paths, or a general object allocator.

## Decision

The initial process shall own exactly three descriptor slots, zero through
two. At creation, slot zero references one readable console-input open
description, slot one references one writable console-output description, and
slot two references a distinct writable console-error description. A closed
slot contains no reference. `read` and `write` shall authorize the referenced
description's access mode rather than the numeric slot: duplication preserves
access mode, and closing one slot does not invalidate another reference.

Linux-compatible `dup(old)` shall reject a negative, out-of-range, or closed
source with `-EBADF`. Otherwise it shall install another reference in the
lowest numbered closed slot and return that number. It shall return `-EMFILE`
when all three slots are live, without changing any mapping.

Linux-compatible `dup2(old, new)` shall first reject an invalid or closed
source or an out-of-range destination with `-EBADF`, without changing the
destination. When both numbers are equal and the source is live, it shall
return that number without changing state. Otherwise it shall atomically
replace any destination mapping with another reference to the source open
description and return the destination number. The bounded fixed table
requires no allocation after validation, so the qualified operation has no
later failure point.

`close` shall remove only the selected slot mapping. The fixed open
descriptions and every remaining mapping shall remain valid. Terminal process
teardown shall remove all mappings and clear the provider-private table before
the exact kernel continuation resumes. Portable source owns table bounds,
access policy, lowest-free selection, and Linux error numbers `EBADF=9` and
`EMFILE=24`; it observes no packed representation, native register, syscall
number, reference count, or object address.

The ring-3 qualification shall first prove `dup(1)` returns `-EMFILE` while
the table is full. After the existing successful `close(0)`, `dup(1)` shall
return zero. Closing descriptor one and then `dup2(0, 1)` shall restore a
second mapping, proving the duplicate survives source-slot close. `read(0, …)`
shall return `-EBADF` because the referenced output description is not
readable, without consuming input. After closing zero, `dup2(-1, 0)` shall
return `-EBADF` without replacement; `dup2(2, 0)` shall then return zero, and
closing zero shall leave the restored output and original error mappings live
for terminal verification.

## Architecture pressure test

Slot lookup, lowest-free selection, replacement order, open-description
identity, access mode, independent mapping lifetime, and Linux errors are
portable personality meaning. X86-64 selects native syscalls 32 and 33;
AArch64 and RISC-V use `dup3` syscall 24 to express non-identity replacement
and require a separate portable no-flags adapter. Providers may use packed
indices, pointers, or capabilities internally, but portable source cannot
observe those choices or equate descriptor integers with object identities.

## Alternatives rejected

Copying only the existing live bit was rejected because a duplicate would
lose whether it names input, output, or error. Mutating the source slot into
the destination was rejected because Linux duplication creates another
mapping. Closing the open description whenever either integer closes was
rejected because aliases must survive independently. Implementing `dup2` as
close followed by fallible duplication was rejected because an invalid source
must leave the destination unchanged and replacement must not expose an
intermediate closed state. Adding general reference-counted VFS allocation was
rejected because the three fixed descriptions and bounded table need neither
path lookup nor dynamic reclamation.

## Deliberate bounds

This increment supports one process, one thread, three slots, three fixed
console descriptions, `dup`, `dup2`, access-mode lookup, independent close,
lowest-free allocation, atomic replacement, and terminal clearing. It does not
provide `dup3`, `fcntl`, close-on-exec, slots above two, dynamically created
open descriptions, path lookup, general reference counts, pipes, sockets,
polling, fork or exec inheritance, concurrent lookup/close, signals,
cancellation, or device-specific final release.
