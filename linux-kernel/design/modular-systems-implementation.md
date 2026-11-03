# Modular systems implementation maintenance

`TK-DEC-277`; implementation maintenance under the existing reviewability rule.
This changes no language design, system requirement or normative observable
behavior. The [instance qualification](../step-4-instance-scoped-region-actions.md)
identified six owned Rust files already exceeding the repository's 2000-line
ceiling. The size audit also traverses ignored cached Linux dependencies.

## Scope and preserved contract

Decompose systems source checking, the architecture-independent reference model,
artifact publication, x86-64 provider generation and their tests into reviewable
files. Keep each owned Rust source at or below the existing ceiling. Prune
ignored dependency/build caches from the filesystem audit while retaining that
ceiling for owned source. Do not add exemptions, discard checks, compress code
or hide a large routine behind an unreviewable include or macro.

Preserve all 297 current normative systems rules, public Rust/source interfaces,
diagnostics, semantic transitions, caller/ownership/resource laws, source-module
closures, native instruction/data generation, object/boot formats and private
ABI families. This work introduces no core or systems language meaning and no
architecture implementation. Existing source qualifications and artifact
revisions remain valid only when their observable bytes and contracts remain
unchanged.

Prefer complete item/module extraction for declarations, helpers and test groups.
Preserve their lexical Rust namespace where public paths or private access make
that necessary. Split oversized routines into named, ordinary helper phases
with explicit shared encoder/checker state, retaining statement order, guards,
early exits, labels and relocation positions. Keep related Linux-specific test
fixtures under `linux-kernel`; preserve literal contents and formatting inputs.
Moved `include_str!` paths, macro expansion contexts, imports, type paths,
diagnostic spans and formatting coverage require explicit review.

Source checking and reference-model code remain architecture independent.
Native x86-64 instruction emission remains in its provider implementation;
AArch64/RISC-V are not represented by x86 register or layout assumptions in
source/model interfaces. This is source organization, not a new machine model.

| Existing owner | Extracted responsibilities | Preserved evidence |
| --- | --- | --- |
| `src/topal-language/src/systems.rs` | systems declarations, bootstrap/context decisions, device/user requests and source tests | accepted/rejected forms, exact diagnostics/spans, operations and transition trace |
| `src/topal-language/src/systems_translation_edit.rs` | translation/edit and static-image construction checks | affine ordering, rollback and exact construction operations |
| `src/topal-semantics/src/systems.rs` | public model types/constants, validation/protocol states, transition modeling and tests | public type paths, ownership/failure laws and exact model transitions |
| `src/topal-compiler/src/systems_artifact.rs` | provenance/publication, supplied image fixtures, root encoding and tests | canonical formats, atomic publication, exact linked root and resource evidence |
| `src/topal-compiler/src/systems_provider.rs` | sealed operation registry, provider planning and tests | stable operation identities, target/ownership checks and deterministic plan |
| `src/topal-compiler/src/systems_provider_object.rs` | primitive/controller emission, process-entry phases and instruction audits | exact bytes, labels, relocations, symbols/storage, stack and retained ABI contracts |

The process-entry emitter and context-decision checker exceed the ceiling even
as individual routines. They require explicit helper phases rather than merely
moving the same oversized item. The large instruction audit must retain all
checks through named audit phases; its assertion count is not a size budget.

## Commit stages

1. Record the affected source and preserved contract, with a concrete ownership
   and verification map before extraction.
2. Extract source-checking/reference-model items and tests, splitting oversized
   decision routines without changing their accepted/rejected programs.
3. Extract artifact/provider items and instruction audits, split the large
   process-entry emitter, and repair dependency-cache traversal in the size audit.
4. Qualify the final decomposition, update implementation traceability and
   publish it through the existing Step 4 PR.

Each stage has its own commit. No normative specification change is proposed.
The existing initial-plan work remains in the [status ledger](../step-4-status.md).

## Verification and evidence reuse

Run the complete bounded Rust workspace, all-target checks, formatting including
new included files, and the source-size audit. Retain every existing assertion
and conformance obligation; newly named helper phases do not replace functional
checks. Compare actual generated native code/data, metadata, ABI/resource facts
and source/adapter identities with the preceding qualified artifacts.

The preceding primary kernel is
`397e07bfa0c0528e73389ea781e112923b2637a5aa1ee587e4a795cc2cbc7bb0`.
Rebuild and execute it after decomposition. Prior VM evidence may be retained
only for unchanged artifact and harness identities; explicitly distinguish
retained observations from fresh runs. A changed generated artifact requires
investigation and qualification rather than an unexplained new checksum.
Never describe retained evidence as a complete fresh cohort run.

Risk is high at extraction boundaries, shared mutable checker/emitter state,
early-return order, namespace/type paths, literal templates, instruction/label/
relocation order and lost test coverage. Review those boundaries and the full
diff against approved source/model/provider contracts. Human PR review remains
required; no independent reviewer result is claimed. The standing systems-only
authorization and meaning-preserving maintenance exception apply; any necessary
new core meaning would still require a human decision.
