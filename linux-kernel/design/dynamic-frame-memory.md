# Runtime memory requests and frame ownership

This extends TK-DEC-282 Stage 3 under the approved systems-only direction.
The [full boot Root binding](frame-policy-kernel-binding.md) supplies real
allocation and release for existing IR lifetimes. The next binding must obtain
backing from actual userspace requests and retire it on resize, exec and exit.
A fixed provider heap, or an allocation performed only by a boot probe, cannot
establish this connection.

## Privileged invocation

The memory operation and its allocator calls share one owning gateway. Its
complete compiled call graph, source-owner terminal callback and physical
retirement must fit the existing 4096-byte caller reserve. Nesting another
4096-byte erasure beneath a live memory-operation frame would require a separate
proof; this binding instead owns the combined dispatch/controller path once.
The source evaluation reserve remains 65536 bytes and its private stack remains
4096 bytes.

Syscall admission binds the actual captured process context, current address
space, fresh nonwrapping entry identity, ABI request, trusted kernel continuation
stack and compiler-generated return site. It checks hardware privilege and
critical state before lending the invocation. Stack addresses alone cannot
establish a syscall lease. The target masks the direction flag on syscall entry
while retaining the original user flags for return. The invocation identity and
lease remain live through every allocator call; source cannot access or create
either. Exec reset and completed-process cleanup have distinct checked callers
and context states. Invalid or busy entry cannot claim retirement authority.

For the current x86 implementation the captured kernel continuation uses the
existing bootstrap stack. The binding additionally requires its exact saved
stack value, correct GS context, CR3 and return site. This does not make the
bootstrap lease sufficient for arbitrary syscall stacks. AArch64 and RISC-V
must supply equivalent typed context, address-space, critical and entry-lifetime
facts using their own entry mechanisms; GS, MSRs and x86 return sites are private
realizations.

## Heap replacement

`brk` uses the actual requested break and current process state. The target
derives the necessary whole-page count with checked arithmetic. Query and
within-page changes need no new physical extent. Growth or shrink obtains a
replacement extent from the ordinary frame policy, resolves its private facts,
copies the retained prefix and keeps new pages zero. The current and candidate
tokens are private affine values for one replacement operation, not an
application allocation quota. Registry capacity still derives from physical
memory.

Before mutation the owner validates its current handle and every affected
translation leaf. It pins the replacement before publishing translations,
invalidates all affected TLB entries, and only then unpins and consumes the old
whole extent. Ownership transfer consumes the candidate token once. Ordinary
unavailability preserves the old break, mappings, bytes and live handle;
admission/protocol failures cannot masquerade as that result. A failure after
stores preserves the performed effects, poisons the memory/allocator owner,
retires invocation temporaries and enters nonreturning fatal handling. It does
not restore a comparison snapshot.

Exec reset and process completion revoke the complete live heap, invalidate its
translations and release its backing. They must replace the old fixed-length
erase path. Argument/environment staging required by exec remains owned until
the new image has consumed it.

The first target integration retains the existing qualified user page-table
geometry, with the heap between the image and stack. Its page-table limit is an
explicit temporary architecture boundary, not a general userspace quota or
completion of the original memory goal. General virtual-address placement,
hierarchy growth, independent `mmap` regions, protection/unmap/split semantics,
multiple process address spaces and physical-access expansion remain required.
Memory requests must not be restricted to the old startup phase or fixed page
counts. Any retained startup witness serves only the predecessor ABI tests.

Qualification must execute actual user requests with different sizes, preserve
data across replacement, observe zero new pages and stale translation removal,
check reuse and failures, and prove whole exec/exit retirement. It must retain
exact source/native/context/load inputs. Test-built experiments remain separate
from canonical publication and from real distribution, container and VM-host
acceptance.
