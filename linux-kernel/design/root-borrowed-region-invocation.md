# Kernel-root invocation of ordinary region functions

`TK-DEC-271`; systems-only refinement under standing authorization.

Bind the [checked region source object](source-kernel-region-functions.md) to
an existing live bootstrap-region owner and execute it in the replacement
kernel. The result must include real ordinary helper frames and complete
temporary reclamation before owner resumption. A standalone C caller or another
memory simulator cannot qualify this boundary.

## Selected source interface

The systems-root artifact may declare one `region-library` using the existing
`lang systems source-library` form, with a contained source path, published
export, interface `borrowed-byte-region-v1`, `stack-limit-bytes is 4096` and
`runtime-limit-bytes is 65536`. It has no retained source-state reserve. The
export remains the ordinary `(KernelByteRegion) -> Unit` entry, with the exact
frozen source/module closure and declared-type selection from TK270.

Within a live plain bootstrap-region scope, `region library apply` lends the
region to that selected entry and returns its owner to the surrounding scope
after cleanup. It grants no source address, constructor, function pointer or
new owning resource. Source chooses the library implementation through the
artifact declaration, rather than native Linux-specific policy selection.
The initial root profile admits one such synchronous invocation in the existing
64-byte bootstrap-region witness, before an atomic location is derived. No
plain-region or atomic-owner access may overlap the loan. A missing declaration,
invocation outside that scope or duplicate invocation diagnoses before linking.

The existing source block may initialize a byte, apply the ordinary library,
and test a byte through the resumed owner before its existing continuation and
release. A mismatch takes the existing fatal branch. This is a dynamic byte
guard; source export/type/effect checks do not prove application-specific output.
Internal ordinary helper calls retain the TK269/TK270 borrowing, effect and
nonescape laws.

## Ownership and temporary storage

The artifact supplies a distinct owned 65536-byte temporary arena, private
invocation control and generation history. The root caller binds the source
code, its actual live bootstrap-region extent and owner identity, current
generation, admitted rights and exclusively lent temporary storage. Numeric
owner labels, linked addresses and checksums remain evidence, not grants of
region authority. Before entry, validate all bindings, disjoint extents,
alignment, overflow, inactive/zero temporary state and complete bounds.

The invocation is synchronous in a qualified bootstrap context with interrupts
disabled and no suspension, callbacks or nested native invocation. Plain
bootstrap ownership is suspended while the source graph runs. Temporary hooks
must use this root-owned invocation context, rather than borrowing an unrelated
prepared-process or GS context. Dedicated private region allocation/fatal hooks
may implement the existing explicit temporary-allocation contract. Their names,
code, stack obligations and complete cost certificate must be versioned and
bound; changing a private symbol does not grant ambient storage to source.

The source object clears its live views and control bindings on return. Before
the root resumes ordinary owner access, erase and read back the entire temporary
arena, retired helper-stack reserve and all source/owner/code/address bindings,
and clear transient pointer-bearing registers. Preserve the root's live frames,
required registers and the owner's payload. Successful source stores persist.
A nonwrapping scalar generation history may remain in a separately owned
history location; it grants no access and must not retain a pointer or code
binding. Terminal cleanup retires that history with its owner. A fatal invariant
failure never resumes the suspended owner.

The native ordinary-region placement must exclude provider-private event and
context-control bytes. Its complete physical footprint, including reserved
prefix/alignment padding, must fit the declared pool before publication. Later
atomic access must derive its address from that actual region base and its
source-relative offset; a fixed low-pool offset cannot stand in for the owner.

## Model and publication evidence

The resource model records the loan, source effect boundary, revocation and
owner resumption. An opaque source invocation makes its owner's byte contents
unknown to static byte prediction. Subsequent bounds/lifetime checks remain
mandatory, while a byte equality guard over those unknown contents is recorded
as a runtime obligation. The model must not silently leave the old byte values
known or claim it executed the helper. Explicit stores can restore knowledge of
the bytes they overwrite. Actual source and native byte behavior are qualified
separately; model-only labels cannot qualify linked code.

Publication checks the complete source/object certificate, declared root slot,
invocation ownership/order, temporary placement, qualified hook implementation,
linked code/data extents and relocations. Reject missing/extra sources, changed
modules/code, mismatched interfaces or costs and undercharged arena/stack/view
bindings before atomic publication. Inspection and packaging preserve the exact
root/provider/toolchain/board identities. An object certificate alone does not
qualify a different linked executable.

## Qualification and architecture review

The primary kernel invokes the real imported byte helper against its actual
bootstrap owner, observes the resulting byte through that same owner after
cleanup, then continues the existing kernel/application regression workload.
Pinned QEMU debugger observations cover the root/ordinary helper caller chain,
parameters, exact owner addresses/windows, preserved native state, and complete
arena/retired-stack/control reclamation before the resumed byte guard. Include
independent changed-helper and corrupted-binding/data controls, absence/frozen
interface controls and the full preceding primary/BusyBox cohort. No kernel may
be reported qualified merely because it boots or an object replay succeeds.

The source and lifecycle laws also apply to AArch64 and RISC-V. Root context,
stack lending, register preservation, address derivation and temporary hook
placement need separate architecture-qualified implementations. Neither GS nor
the x86 register set is source meaning. Only x86-64 is implemented now.

Alternatives considered: retaining the standalone C adapter proves no kernel
ownership; reusing a prepared-process numeric context would conflate independent
owners; interpreting bytes as a native policy would leave source subsystems
unqualified. A dedicated root loan over the same ordinary checked source graph
preserves the approved architecture and provides the next physical evidence.

Risk is high at privilege, resource provenance, model knowledge, hidden pointer
lifetime, ABI/stack bounds and atomic publication. Review follows authoritative
resource/effect/layout intent, every ownership transition and byte-access guard,
complete object/link/temporary closure and independent physical oracles. The
implementing agent performs these roles; human PR review remains required.
