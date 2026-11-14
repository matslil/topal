# Kernel-image protection

Status: systems-only design adopted under the standing chat authorization
(`TK-DEC-285`). This contract and its portable models do not implement or
qualify native protection, change existing profiles, or authorize production
activation. Core language semantics remain unchanged.

## Problem and selected approach

The [calendar placement stage](../tools/user-memory/results/stage-3-calendar-placement.md)
separates data from ELF instruction sections, but actual captured supervisor
translations remain writable with execute-disable clear. The final bootstrap
root is also outside the last captured hierarchy. Section names, numeric
addresses and a successful user-buffer check cannot establish kernel-image
protection.

Introduce an explicit systems refinement which combines the qualified
artifact's semantic placement plan with actual translation/backing ownership.
It supplies readable/executable code, readable non-executable immutable data,
and readable/writable non-executable mutable data. All kernel-image routes
remain supervisor-only. Relocation storage may become immutable only after its
declared final initialization. Device and translation-storage roles retain
their own access and memory-kind contracts; neither is ordinary immutable
program data.

The provider selects mapping granules and owned translation storage from the
actual image and target. It may retain a large leaf only where every covered
role has compatible rights. Mixed leaves require finer mapping or a different
validated layout. Source does not select PTE bits, table levels, physical
addresses, ELF names or a universal table count. Plan geometry is descriptive;
only a qualified owner/context can authorize construction or publication.

The existing `bootstrap-equivalent` template retains its exact coverage/access
contract. Protection is a separately selected refinement. The current static
process-image builder is a second producer and must satisfy the same selected
kernel-image contract; protecting only the bootstrap builder is insufficient.

Alternatives considered:

| Approach | Consequence |
| --- | --- |
| ELF flags alone | Actual mappings demonstrate that this does not enforce protection |
| Align every role to a large block | Can avoid some tables, but wastes reserved memory and ties placement to one target's block geometry |
| Small leaves throughout the physical window | Simplifies permission subdivision but reserves storage for unrelated coverage; it does not solve aliases, ownership or publication |
| Semantic roles with provider-selected subdivision | Selected: storage follows the actual qualified layout, while rights and ownership remain portable |

For the current x86 image, header, BSS, immutable data, text, relocation storage
and mutable data have distinct roles. Large regions 21 and 22 mix roles; the
header in region 1 also needs subdivision if neighboring mappings are retained
with different rights. These observations guide tests, not native address or
table-count constants. Live boot stacks, transition code, handlers, MMIO and
all non-image mappings must be classified separately before native selection.

## Resources and state transitions

A kernel-image plan is bound to the actual qualified image, provider, layout,
address-space domain and supported interpretation mode. Its placement facts
come from validated publication and the boot/image owner. An ELF supplied by a
user process, a corrected outer checksum, a source integer or copied metadata
cannot construct that authority. The plan need not embed its own final image
digest in itself; the qualified publisher/boot binding must avoid that circular
definition and supply exact final artifact identity externally.

Preparation borrows the image and allocation authorities, derives the complete
translation-storage requirement, and obtains disjoint, pinned backing through
the existing fallible allocation ownership protocol. A numeric requirement
permits sizing, not access. Every backing owner must match the exact plan and
actual topology. Count, alignment, issuer, generation, pinning, bounds,
non-aliasing and lifetime checks remain mandatory. A provider may not erase the
current exact five-page checks and accept an arbitrary larger range instead.

The protected-image context is granted only after all of the following:

1. Every active, inactive, prepared and restorable kernel root is identified
   through actual ownership, including the bootstrap return root.
2. Every kernel-image mapping and physical alias in those roots has the required
   role, rights and memory kind. A writable alias to immutable/code backing or
   user-accessible alias to kernel backing prevents success. Unrelated user
   mappings retain their own checked owners and rights.
   Declared primary image views provide their required rights; additional aliases
   may narrow them, for example a non-executable read-only view of code. An alias
   does not gain execution permission merely because the backing contains code.
3. The mode makes those protections effective. Existing x86 supervisor write
   protection stays enabled; execute-disable support/control must precede use
   of execute-disable entries. No temporary protection disabling is allowed.
4. The provider completes publication, translation/cache maintenance and
   synchronization for every relevant context and CPU. Stale translations are
   not accepted as protected merely because table bytes changed.
5. Current code, return targets, stacks, interrupt/exception/fatal paths and
   live data remain valid throughout the transition. Untrusted execution and
   unqualified competing writers cannot cross an incomplete publication.

The protected context owns the image-wide obligation across subsequent root
creation, activation, exec, handoff, cancellation, restoration and retirement.
New roots must derive the same protection binding before they become eligible
for activation or restoration. No usable unsealed fallback root survives the
refinement. Retiring a root does not release shared backing until every owner,
reference, pin and architecture maintenance obligation is discharged.

Failure before publication preserves the existing context's declared state and
reclaims temporary resources according to their owners. Failure after a possible
hardware-visible partial effect has no ordinary continuation: retain/poison
affected authority and enter the qualified fatal path. Neither case may return
a protected-image witness. Existing profile-specific fatal rules remain.

## Architecture pressure

| Provider | Facts the native realization must bind |
| --- | --- |
| x86-64 | Actual paging/control state, effective ancestor/leaf rights, supervisor write protection, execute-disable interpretation, large-leaf subdivision, aliases and qualified invalidation/publication |
| AArch64 | Selected supported granule and levels, privilege/execute controls, memory attributes, table ownership and qualified translation/cache maintenance across relevant contexts |
| RISC-V | Selected translation scheme/levels, valid leaf rights, supervisor access controls, local and remote translation synchronization, and instruction visibility where relevant |

Arm's documented 4/16/64 KiB granules and differing block geometry prevent a
portable four-KiB or five-table rule. RISC-V's Sv39/Sv48/Sv57 levels and local
scope of translation fences require explicit provider and participating-CPU
obligations. These differences motivate the selected abstraction; they do not
qualify executable Arm or RISC-V providers.

Primary sources: [Intel SDM, current Volume 3A, paging](https://cdrdv2.intel.com/v1/dl/getContent/671190)
(revision 093, Chapter 5), [Arm AArch64 memory management guide 1.4](https://documentation-service.arm.com/static/670e4dc89fbc7343d3e4cee1?token=)
(translation granules and maintenance), and [RISC-V supervisor specification](https://docs.riscv.org/reference/isa/v20260120/priv/supervisor.html)
(address translation, Sv39/Sv48/Sv57 and synchronization).

## Provider-selected static hierarchy request

`TK-DEC-286` is a systems-only interface increment adopted under the standing
chat authorization. Static construction accepts `backing is provider-selected`
beside its exact sealed owner, keeping a purpose-specific reservation in checked
IR and provider provenance. The old explicit count/alignment request remains
compatible; mixing the forms is invalid. This removes target geometry from new
source requests without erasing the resource's purpose into a generic allocation.

The current x86 bootstrap-equivalent provider resolves the request to its exact
five unit-aligned frames, using the existing actual allocator and complete
attachment/restoration/reclaim/release path. Its exact owner, exclusion and
user-table checks remain unchanged. No larger bank or new protection mode is
enabled. The later protected provider must instead bind its complete requirement
to the actual qualified image and all owned roots; missing qualification cannot
fall back to the legacy shape. The source interface is portable, while each
provider's native realization remains independently qualified. Core semantics
are unchanged.

Alternatives: synthesizing a generic five-frame allocation during parsing loses
the purpose needed for future image-derived resolution; accepting a source
count such as eight exposes target geometry and supplies no owner binding.
Keeping the request opaque and resolving it at the provider preserves both
portability and the existing security obligations.

## Security review and qualification gates

Before replacing any current check or enabling native selection, review the
complete obligation set: actual root/table/image identities, all producers and
writers, all aliases, user-backing exclusion, range/overflow/entry formats,
mode stability, CR0.WP preservation, publication/TLB/cache completion, competing
CPU/exception/NMI/DMA activity, caller state, failure effects and exact
retirement. A missing native observation remains a gap; model equality cannot
fill it. Host protections and bounded VM/resource limits remain unchanged.

The first model stage may prove role partitioning, granule conflicts, necessary
subdivision and rejection of incompatible aliases on supplied descriptions.
It creates no root or owner and is not a native page-table generator. Subsequent
stages must bind actual compiler/loader facts, obtain the exact backing,
implement all construction/validation paths together, exercise ordinary guest
interfaces and read actual controls/mappings through final restoration. Preserve
pinned third-party binaries and use source-built guest programs where needed.

Full source/native/boot reconstruction, all-entry/caller/lifetime obligations
and original Linux UAPI, distribution, container and VM-host acceptance remain
independent requirements. A protected-image witness does not establish them.
