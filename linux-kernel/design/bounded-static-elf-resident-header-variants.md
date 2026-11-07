# Bounded static ELF resident-header variants

This record resolves `TK-DEC-192` as a systems-provider and qualification
refinement of the approved supplied-static-ELF and resident-program-header
contracts. It changes no core-language rule and does not redistribute the
pinned executable.

## Boundary

The clean-room qualification fixture and Debian's pinned external BusyBox have
identical runtime ELF identity, entry, and complete 11-entry program-header
table. Their 64-byte ELF headers differ only in section-directory metadata:

- the clean-room form has `e_shoff`, `e_shentsize`, `e_shnum`, and
  `e_shstrndx` all zero; and
- the pinned external form has `e_shoff=0x1edea0`, `e_shentsize=64`,
  `e_shnum=23`, and `e_shstrndx=22`.

For the external form, the compiler shall prove that the complete section
directory ends exactly at the validated 2,024,544-byte input boundary. The
runtime provider shall accept exactly either complete coherent tuple, compare
every other ELF-header byte and every byte of the complete resident
program-header table, and fail closed on a mixed or third form. The section
directory itself is not loadable process state and shall not be parsed or
retained at runtime.

Qualification shall keep the deterministic clean-room gate and add an
explicit caller-supplied external-image gate. The latter shall require the
pinned package digest before compilation, bind the exact supplied bytes through
artifact and boot provenance, and record evidence without committing the
executable.

## Alternatives considered

Requiring zero section fields preserves the fixture but rejects the selected
real executable. Ignoring all section fields would admit unbounded or
incoherent metadata. Embedding the section table in the provider or repository
would add unused runtime state and cross the licensing boundary. Generating a
provider from arbitrary input bytes would broaden the sealed target before a
general loader contract exists.

## Cross-architecture pressure

The portable fact is that runtime execution depends on a validated load plan,
entry, and resident program-header table, while non-loadable link-editor
metadata is checked at the artifact boundary. ELF field offsets and the two
x86-64 tuples remain provider details. A future architecture may select a
different executable format while preserving the same separation.

This increment does not admit arbitrary ELF headers, program-header layouts,
section parsing at runtime, dynamic linking, PIE, ASLR, filesystem lookup,
redistributed BusyBox, a prompt, containers, or virtualization hosting.
