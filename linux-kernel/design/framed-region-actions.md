# Framed scalar inputs for ordinary region actions

`TK-DEC-275`; systems-only extension under standing authorization. Generalize
the private transport of ordinary action inputs without adding source addresses,
frame constructors, new core values or another native ABI for every arity.
The [persistent action contract](persistent-region-actions.md) and existing
Unit/two-scalar action profiles retain their meanings and entry ABIs.

## Source selection and portability

Add a distinct `borrowed-byte-region-framed-action-v1` qualification context.
A caller selects a published ordinary function with exactly one initial
`KernelByteRegion`, followed by 0–16 `Nat` parameters, returning `Boolean`.
The caller supplies the expected scalar count; declaration checking must match
it exactly. Do not infer an overload, insert ignored arguments or reinterpret
parameter names as native algorithms. Existing selection, visibility, packaged
parameter, effects, nonescape, closed direct-call and termination rules apply.
The count limit belongs to this systems qualification, not core function types.
Additional fields use the existing packaged-operand syntax; the core limit of
two syntactic operands remains unchanged.

Each transported fact is an unsigned 64-bit value constructed as an ordinary
Nat. Source arithmetic remains arbitrary precision. The private ABI receives
control identity, expected owner, next epoch, a caller-owned native-u64 word
frame and its word count. Word order follows declared parameter order; byte
order and machine calling convention belong to the target ABI. Source receives
ordinary values and its sealed view, never the frame pointer or native words.
Zero inputs use a null empty frame without any word access.

The first implementation is x86-64 SysV. AArch64 and RISC-V retain the same
source/value/lifetime obligations but require independent target layout,
calling, instruction, stack and physical evidence. A bounded frame avoids
coupling the source interface to one architecture's argument-register count.
It introduces no source assembly or Linux-specific language operation.

## Input ownership, entry and completion

The qualified native caller owns mapped, aligned, immutable-during-call storage
for exactly the declared words. It evaluates/captures each fact once and retains
the frame until return. Its complete proof establishes containment, nonwrapping
extent and disjointness from the lent owner, control, temporary arena and
retired source-stack reserve. The frame carries values, not authority; a raw
pointer cannot substitute for that caller proof.

The bridge validates the exact count and structural frame conditions before
word loads/source dispatch. Nonempty frames require a nonnull aligned pointer,
bounded nonwrapping extent and disjoint owner/control extents. The owning
caller's mapped-frame and complete arena/stack-disjointness obligations remain
mandatory; this private ABI does not make arbitrary unqualified C pointers safe.
All ordinary Nat inputs are prepared before the source call. Invalid transport
or ownership is an invariant failure, not an application errno, source false
result or fallback implementation.

The controller's complete domain remains excluded while source executes;
interrupt masking alone is not SMP exclusion. No source view/value/frame pointer
may escape, suspend, reenter or acquire another memory domain. Source Boolean
false retains its ordinary meaning. The selected file-position consumer still
requires true for an admitted transfer; false after private preparation remains
invariant fatal without owner resumption or promised rollback.

Return revokes every view and clears complete temporary arena/control/retired
stack state before resuming ownership. The native caller clears its complete
temporary word staging and restores non-result registers. Input-frame storage
is separate from source temporaries and cannot be reclaimed while the bridge
still reads it. Generations, last-owner terminal cleanup and persistent owner
bytes retain the existing action laws.

## Bounds, representation and publication

Retain the 65536-byte temporary arena, 4096-byte combined stack reserve,
256-view, 64-source-function and 256-bit numeric proof limits. The root view
costs 64 bytes; every supplied u64 fact costs at most 24 bytes in ordinary
numeric conversion. Charge `64 + 24*N` bridge bytes plus the existing 32-byte
arena header and the complete source evaluation. Include actual adapter frames,
all source/runtime calls, return addresses, native alignment and live caller
staging in the stack proof. Measurements remain separate evidence.

Bind the complete generated adapter family for all admitted counts and its
selected exact rendering in reviewed allocation evidence. Inspect actual target
objects and LLVM frame/call evidence. A code-only zero-input graph with no
immutable data remains valid when actual inspection proves that absence.

Use a new source-object schema and private native ABI for framed actions, with
an explicit scalar count. Legacy Unit/action metadata must not accept framed
fields or silently become the new context. Use a flat private namespace outside
the existing `topal.region.` hierarchy, avoiding accidental Unit ownership of
another nested profile. Preserve exact ordinary source/debug identities and
the existing Unit/action object/entry representations.

Publication and boot bind the complete source/module, scalar signature, adapter,
code/data, caller-frame, controller/storage and resource evidence. Incompatible
schemas/counts, foreign namespaces, altered actual bytes, hidden inputs and
incomplete callers reject. The existing per-object policy-input bindings remain
mandatory. No declaration or standalone object supplies a live callable grant.

## First physical integration and qualification

Allow the existing file-position declaration to select the new framed context.
That caller still requires exactly two Nat facts: effective transfer base and
actual count. Reuse its existing caller-owned scalar staging as a two-word frame;
the source signature, arithmetic/codec algorithm, Linux behavior and successful
owner mutation remain unchanged. Retain explicit legacy-interface qualification.
The application selects an expected signature; arbitrary arity is not a way to
change a syscall's semantic contract.

Qualify standalone native source calls with zero, one, two, three and sixteen
facts, full-u64 values, independent arithmetic/byte/canary oracles, actual source
frames and complete cleanup. Refuse excessive or mismatched counts/types,
malformed frame extents/placement, wrong owner/epoch, stale entry metadata and
inconsistent object/caller/resource evidence. Keep valid source/module variants
independent of native policy. The VM must observe actual frame words, ordinary
Nat arguments, real shared field writes, native caller preservation, full
cleanup, error/empty/positional noncalls and active/inactive finalization.

Requalify the complete current kernel/BusyBox cohort and old source interfaces.
This advances ordinary kernel-call infrastructure; general persistent
allocation, broader subsystems and the remaining initial-plan acceptance work
are still required.

Risk is high at declared selection, input lifetime/aliasing, transport, complete
bounds, cleanup and private-format compatibility. Review the existing core
function/Nat/effect meanings and systems ownership/call obligations, alternative
transport choices, actual generated code/frames and independent native/physical
evidence. Human PR review remains required in the existing Step 4 PR.
