# Step 4 generated Linux x86 boot adapter

## Outcome

This is the sixth implementation increment of Step 4. The compiler can now
validate the linked freestanding x86-64 artifact and its digest provenance,
materialize its load segments, generate the approved Linux x86 boot-protocol
2.15 transition, and atomically publish a deterministic `bzImage` with
canonical adapter provenance.

The generated adapter contains a relocatable 16-bit setup entry followed by
compiler-owned 32-bit and 64-bit transition code. It enables A20, installs the
generated GDT, enables PAE and long mode, establishes the approved one-GiB
identity map and bootstrap stack, installs the generated IDT, retains the
boot-parameter address in `RSI`, and transfers directly to the linked kernel
entry. IDT vector 3 names the linked debug-break handler. These encodings and
register obligations remain private typed compiler operations; Topal source
does not gain instruction, register, descriptor, page-table, or source-assembly
syntax.

The published directory contains exactly:

- `bzImage`, the Linux x86 protocol 2.15 image; and
- `boot-provenance.json`, the protocol, target, board, machine CPU, provider,
  layout, entry-address, protected-size, linked-kernel-digest, and image-digest
  record.

Both outputs are staged and become visible through one directory rename. An
existing destination is rejected rather than overwritten.

## Inspection boundary

Packaging accepts only the sealed `x86_64-unknown-none`,
`topal-qemu-pc-q35-10.2`, `qemu64-v1` artifact identity. The supplied kernel
bytes must match its recorded digest and parse as a static executable
little-endian ELF64 x86-64 file. Every load segment must begin at or above 2
MiB, advertise at least 4 KiB alignment, satisfy ELF address/file-offset
congruence, remain inside the initial identity map, and not overlap another
segment. The generated bootstrap and debug-break symbols must occupy executable
load segments, and the ELF entry must select the generated bootstrap symbol.

The adapter materializes segment contents at package time and zero-fills
uninitialized memory tails, so the privileged bootstrap contains no ELF
parser. Fixed transition structures are bounds-checked against the protected
payload and the package is capped at 64 MiB. Header sizes use checked narrowing,
and the image digest is computed only after the complete byte sequence exists.

## Validation and risk

Focused compiler tests build the checked Topal toolchain-gate root through the
existing LLVM 22 artifact publisher, generate two images at different paths,
inspect the boot header, PML4 and IDT gate, compare both publication sets byte
for byte, and decode and verify the canonical provenance and both digests. The
tests also exercise the single two-file publication rename.

Risk remains high because the adapter executes before the typed kernel entry
and a malformed transition can fail without diagnostics. Structural checks
close the admitted input and layout. The subsequent
[pinned-QEMU toolchain gate](step-4-toolchain-gate.md) establishes physical
execution for this exact lab-only publication path.
