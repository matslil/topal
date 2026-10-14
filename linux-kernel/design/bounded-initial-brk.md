# Bounded initial-process program break

This record resolves `TK-DEC-073` as a systems-profile refinement. It adds one
Linux-compatible `brk(2)` state to the initial process without introducing a
general virtual-memory-area service or changing core-language meaning.

## Baseline behavior

Linux x86-64 assigns syscall number 12 to `brk`. Linux 7.2.9 retains a current
and initial break, treats the requested address as a byte-granular value,
returns the requested break on success, and returns the prior break when the
request is below the minimum or cannot be realized. A zero request therefore
acts as a query because it is below the process minimum.

## Decision

The initial process owns one fixed RW/NX data page already constructed from the
validated external ELF. The provider derives the initial break as the checked
data virtual base plus runtime `p_memsz`; no independently embedded data extent
may select it. Before user entry, the provider explicitly zeroes every byte
from `p_filesz` through the end of that page, covering both ELF BSS and the
future bounded heap reserve.

The personality accepts `brk(request)` only while the user context is live and
the return instruction lies in the finalized executable mapping. It keeps the
current break in private process state. A zero request returns the current
break. A request from the initial break through the data-page end, inclusive,
atomically becomes and returns the new break, permitting both growth and
shrink. A request outside that interval leaves state unchanged and returns the
prior break. Teardown clears the private break before releasing the data role.

The qualification program queries the initial break, grows it by 16 bytes,
writes and reads the newly admitted first byte, shrinks to the initial break,
and verifies that an out-of-range request returns the unchanged break. An
ordered source witness remains reachable only after terminal user return.

## Isolation and failure contract

Runtime ELF arithmetic, page-end arithmetic, and every request comparison are
overflow checked or fail closed. The complete page tail is initialized before
user entry so growth cannot expose allocator residue. No user pointer is
retained. Unsupported requests use the Linux syscall result convention rather
than a negative errno.

This first slice does not make sub-page access protection stronger than Linux
page mappings: bytes above the logical break share the mapped final page. The
break constrains allocation state and return behavior, while the page remains
RW/NX under its existing role.

## Alternatives considered

Returning one fixed constant would qualify the syscall number but not mutable
process state. Allocating and mapping pages on demand would require a VMA
collection, concurrent edit protocol, accounting, rollback, and fault policy.
Using a compile-time data size would reintroduce a second authority after the
loader already adopted runtime ELF metadata. The selected bounded page-tail
state is the smallest coherent bridge toward libc allocation.

## Cross-architecture pressure

The portable contract is a private byte-granular break bounded by an owned,
zero-initialized writable user reserve. It does not prescribe syscall numbers,
argument/result registers, page-table formats, TLB operations, virtual
addresses, or where process state is stored. An architecture provider derives
the reserve from its checked image and address-space representation.

This increment does not admit additional pages, `mmap`, `mprotect`, `munmap`,
ASLR, overcommit, resource limits, locked memory, page faults, copy-on-write,
multiple processes, concurrent threads, or a general userspace allocator.
