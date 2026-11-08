# Native traversal completion

This continues `TK-DEC-282` Stage 2 under the approved systems-only direction.
Ordinary scalar/Unit results keep their existing meaning. A private provider
result frame is an implementation detail, not a source type or public ABI.

After terminal checkpoint retirement, a distinct completion role runs on the
same 4096-byte stack used sequentially by source and checkpoint. The caller
supplies independently owned private/stack/history and result-frame facts. The
x86-64 result frame has 48 canonical value bytes followed by its eight-byte kind;
the caller owns these 56 bytes before entry. It is separate from the invocation
temporary reserve and must be counted in the still-live owning caller's frame.
Neither geometric disjointness nor this component creates mapped ownership.

Completion validates exact context/role/history, terminal result, root scope,
completed-step bounds and complete retirement of helpers, both banks, work and
provenance. It requires an empty output frame, copies and reads back the complete
actual result, and returns only a private success status. Its result pointer is
compared with the target-pinned pointer before any output access.

The target then restores the caller stack, erases and reads back the entire
private stack and temporary extent, revokes the active cell, and only then clears
the persistent history's active flag. Frame/loan/generation history is retained.
All caller registers and volatile-state rules remain those of the existing
boundary. No source reference, context, bank, result or completion spill escapes.

The fixed completion recovery header pins both history and the independently
owned output frame. A running-body abort dispatches through that exact role;
returned-body cleanup retains both pins in saved registers through header
erasure. Admitted failure poisons actual history, erases the output even after a
partial copy, attempts reclamation of every temporary domain, and enters bound
nonreturning fatal handling. Earlier owner stores/identity issuance persist.
Pre-entry geometry/busy refusal preserves all domains; an admitted kernel caller
must never turn an unexpected refusal into successful owner resumption.

Independent clients must check complete scalar results, immutable caller inputs,
all owner bytes/history, physical private/output/stack cleanup, caller registers,
role backtraces, CFI and linked stack/import/instruction evidence. Tests may escape
a fatal observer only after actual cleanup. Kernel admission, interrupt/context
binding, total owning-caller stack/storage accounting and allocator/metadata
integration remain separate obligations before executable publication. AArch64
and RISC-V providers must realize the same value/lifetime/failure contract with
their own target layouts and calling conventions.
