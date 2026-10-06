# Step 4 linked freestanding x86-64 artifact

## Outcome

This is the fifth implementation increment of Step 4. The compiler can now
lower the checked bootstrap and debug-break handlers into a generated root
object, link that object with the sealed x86-64 provider, structurally inspect
the result, and publish one deterministic artifact directory.

The generated root preserves source operation order. Console writes call the
provider-owned polling 16550 primitive byte by byte, debug-break operations
call the provider-owned vector-3 primitive, and dispositions transfer only to
the interrupt-return or nonreturning fatal provider. The synchronous-exception
root saves and restores all general-purpose registers around its checked body.
These choices remain private typed backend operations; Topal source names no
instruction, register, symbol, relocation, or section.

The published directory contains exactly:

- `kernel.elf`, a static executable ELF64 x86-64 payload with the generated
  bootstrap root as its ELF entry;
- `kernel.debug`, the matching symbol/debug companion;
- `kernel.map`, the deterministic link map; and
- `provenance.json`, the canonical target, provider, toolchain, semantic-trace,
  input/output digest, entry-set, and linked-placement record.

All four outputs are staged and become visible through one directory rename.
An existing destination is rejected instead of being overwritten.

## Inspection boundary

Before publication, the compiler proves that the linked payload is an x86-64
executable with the generated entry, exception root, machine primitives, and
aligned checked bootstrap storage. It rejects undefined symbols, retained
relocations, dynamic-loader sections, PLT/dynamic symbol state, and implicit
unwind/runtime sections. It also checks the debug companion and that the map
retains every semantic input section and required symbol. Provenance records
the actual linked address, size, section, and alignment of seven semantic
roots/facilities.

This payload is deliberately not a boot image. It has no Linux boot-protocol
header or setup code, does not establish the initial stack or IDT, and has not
run under QEMU. The separately qualified boot adapter must establish those
machine handoff obligations and install the generated debug-break entry. The
systems target therefore remains model-only in compiler discovery and has no
ordinary CLI publication path yet.

## Validation and risk

Focused tests inspect the root object's exact four provider dependencies and
relocations, exercise the LLVM 22 link/debug tools, parse every ELF output,
verify the atomic four-file publication set and canonical provenance, and
prove byte-for-byte repeatability across two different destination paths. The
full compiler library and core-language traceability suites cover integration
with all preceding systems increments: all 186 compiler library tests and all
four coverage tests pass. The changed compiler path passes strict Clippy after
allowing only the crate's four unchanged baseline findings, and formatting
checks are clean.

Risk remains high because entry and exception code executes with machine
privilege and a publication mistake could make an incomplete payload appear
qualified. The backend is closed over checked operations, external references
are limited to the sealed provider, structural checks precede the single
publication rename, and no target or boot qualification changes in this
increment.
