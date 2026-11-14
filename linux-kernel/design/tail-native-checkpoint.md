# Native checkpoint return bodies

This continues `TK-DEC-282` Stage 2 under the approved systems-only direction.
The provider owns checkpoint validation, root replacement and retirement as
well as source hooks. Source still uses ordinary checked calls and values.

Source and controller bodies run sequentially on the same private 4096-byte
stack. Each returns through a target boundary that restores the owning caller,
erases and reads back the entire retired stack/header, revokes the active cell
and clears volatile registers. No second stack reserve is introduced. The
owning caller's still-live state remains outside erasure.

The new checkpoint target entry receives private/stack extents, the caller's
independently retained history and the source outcome. It preserves caller
state and pins the exact checkpoint-body role, context, history and recovery
stack before entry. It never infers the history owner from mutable context.
After a normal body return, a preserved register retains the history pointer
through header erasure/readback, so late target cleanup failure still marks the
actual history fatal. An abort while the body is live uses the armed header.
Source hooks require the source role; controller checks require the checkpoint
role. A busy/invalid pre-entry refusal preserves active state. An already
admitted kernel invocation must treat an unexpected refusal as fatal before
owner resumption; numeric geometry establishes no ownership.

After source return the body validates the outcome, root frame, retired helper
descriptors and whole terminal/successor state. A successor has every scalar
staged, no result, valid complete numeric/Boolean representations, exact admitted
natural descent and remaining per-step/invocation frame/loan budgets. The native
measure stays within the initial unsigned-word domain by descent; larger base
bounds make successor entry unreachable, and decrements above that domain cannot
be taken. Terminal paths preserve the actual result and require an empty
successor bank. All fallible checks precede retirement or identity publication.
When all allocating arms are unreachable, the physical runtime still reserves
one eight-byte work granule and its provenance byte. The generated context and
temporary cost include this granule; the zero source-allocation bound remains
distinct from the complete native storage bound.

Checkpoint erases complete source work and provenance, erases the old bank,
retires its root loan/frame, issues fresh persistent identities for a successor
and installs the other bank. Terminal paths erase both banks. Persistent owner
stores and issuance history are never rolled back. Any admitted failure uses
the independently pinned recovery record, marks the actual history fatal and
reclaims all invocation temporaries before bound nonreturning fatal handling.

Independent guarded clients must compare every successor/result and complete
owner/history delta, retained caller inputs, retired buffers, actual role
backtraces, caller registers and source/controller stack bounds. Physical
descriptor reuse still relies on closed source lifetime/no-escape proofs.
Normal completion, kernel admission/context, owner-specific invariants and exact
publication remain separate obligations. Target-specific AArch64/RISC-V return
providers must preserve the same value, lifetime and failure contract.
