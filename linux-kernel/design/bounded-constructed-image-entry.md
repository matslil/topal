# Bounded constructed-image entry

This record resolves `TK-DEC-048` as a systems-profile refinement. It makes
the first user instruction fetch come from the finalized code-role mapping,
then transfers through a sealed provider payload to the existing qualified
Linux syscall and fault probe. It changes no core-language rule and does not
define a general executable-image format.

## Decision

The first mapping generation is populated by the distinct
`user mapping install entry trampoline` operation. The operation consumes the
exact live code-role mapping and produces an entry-installed refinement while
the mapping remains RW/NX. After read-only protection,
`user mapping verify entry trampoline` consumes that exact protected mapping
and produces a verified refinement. Existing executable promotion then seals
the mapping RO/X.

User-context preparation borrows both finalized image roles as before, but
the provider must select the code-role mapping as the initial instruction-fetch
mapping. Its opaque, verified payload performs exactly one control transfer to
the unchanged provider-owned qualification probe. A successful exact kernel
return publishes `TOPAL_KERNEL_USER_IMAGE_ENTRY_OK` before the existing
residency witness and ordered teardown.

```text
map code RW/NX -> install sealed entry payload -> protect RO/NX
  -> verify exact payload -> promote RO/X
  -> finalize data RO/NX -> image-ready witness
  -> enter at code-role mapping -> transfer to qualified probe
  -> exact kernel return -> image-entry witness -> residency witness
  -> data-first teardown -> address-space completion
```

The source contract identifies neither bytes nor virtual addresses. The
provider owns payload encoding, placement, cache or instruction-stream
synchronization, and the private qualification-probe destination. Installation
and verification must agree on the exact payload for the selected provider;
verification failure is fatal, and no writable-plus-executable state is
admitted.

## Entry and failure contract

The code-role mapping must be the initial user program-counter mapping. The
entry payload may transfer only to the sealed qualification probe and may not
read or write the data-role mapping. The existing probe retains its syscall,
fault-containment, `exit_group(0)`, and exact kernel-continuation contract.
Consequently, observing the post-return image-entry witness proves that an
instruction fetch began in the constructed code mapping and completed the
already-qualified probe sequence.

Installing the payload in another mapping generation, byte substitution,
verification before read-only protection, executable promotion before exact
verification, entry at the provider probe instead of the code mapping,
fall-through outside the payload, mutation after verification, an early or
missing image-entry witness, and recoverable completion with live authority
are invalid. Fatal-only completion remains in force after permission
publication begins.

## Cross-architecture pressure

The portable abstraction is an opaque verified entry payload plus the
obligation that initial instruction fetch begins in its finalized code-role
mapping. It is deliberately not an assembly escape hatch: source cannot name
instructions, registers, encodings, addresses, relocation kinds, or branch
ranges.

An x86-64 provider may encode a direct or register-indirect transfer and relies
on coherent instruction fetch after the RW/NX-to-RO/X transition. An AArch64
provider may use a literal or multi-instruction address materialization and
must perform the architecture-required data-cache clean, instruction-cache
invalidate, and barriers before publication. A RISC-V provider may use a
PC-relative or register-indirect sequence and must perform the required
instruction-stream synchronization. These mechanisms are provider evidence,
not source semantics.

This increment does not expose arbitrary entry selection, execute the data
mapping, transfer mapping ownership to a process object, define relocations or
ELF loading, admit arbitrary segment collections or aliases, support demand
paging, or define a general process model.
