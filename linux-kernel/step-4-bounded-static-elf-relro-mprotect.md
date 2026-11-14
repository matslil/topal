# Step 4 bounded static ELF RELRO `mprotect`

The bounded static process now supports the exact
`mprotect(0x005e5000, 0x8000, PROT_READ)` request issued by the pinned x86-64
BusyBox after its post-random break alignment. The native dispatcher verifies
the live aligned-break phase, executable continuation, active hierarchy,
checked GNU RELRO interval, all eight RELRO leaves and their original affine
frames, every retained heap leaf, the disjoint stack leaf, and reusable-random
service. It seals and invalidates the exact target interval before returning
zero and advancing the private startup phase.

The pinned-external qualification exposed that the executable self-relocates
inside GNU RELRO before this request. `TK-DEC-193` therefore refines this
earlier increment: the loader now publishes the eight pages RW/NX for entry,
and this handler replaces them with RO/NX leaves, preserves exact frame
identity, publishes and invalidates all eight target pages, and reads them back
before success. W^X holds throughout and RELRO is immutable after the observed
request.
Rule `TOPAL-SYSTEMS-STATIC-ELF-RELRO-MPROTECT-001`, negative translation tests,
provider-structure tests, and the refined clean-room fixture enforce the
request, entry and sealed role and frame identity, hardware-status-bit
normalization, exact translation mutation and invalidation, terminal phase,
and complete existing rollback. Marker
`TOPAL_KERNEL_STATIC_ELF_RELRO_MPROTECT_OK` binds runtime evidence to the path.

Artifact `/171`, provider `/161`, provider object `/153`, root `/160`, boot
adapter `/71`, and gate `/173` identify the contract. The root has 418
operations, 479 semantic events, 9,282 relocations, and 6,658 console-writer
relocations. The systems profile contains 163 rules.

Two independent pinned-QEMU executions reached all 169 observations with
identical evidence SHA-256
`e2d98e017e6a965b7df51c1975421b099fbcd18570c78b869de7c9b1843c53b3`.
Key digests are linked kernel
`9bcf8467564670ff8b3b07727d69e19f0f5d5a3eba80ff07f6074a2215ebcb7a`,
boot image `0f774c377b82ea90e9e7d84f2f1cff6e9b6c84f7044a34bdd60e032ddda71e14`,
boot resource `43d28ebbad972d8de783bea716aebe6801250c9b0712c527e105b9ea957ff623`,
static image `d15d11bb10fd6cfeb985afda9a1cf2f3529382132d2289799cfa59407ffa5cf2`,
artifact provenance
`d41691f035a1782f9718ad0701041531d1df5572ca13b0ad8071f64e133d48a1`,
boot provenance
`ad6980037ce6ce52acc5a44238619f61e8c960abcba7353ba73c5d45976e0fc5`,
source `ca9912a0b2c77dbd89430848df2cd9dc7d0e905ebd487bcb9575fcad0275c91a`,
and serial output
`19aefe0099c32ea2a1e57d37cdc52b10d35eadd334e58316d158db8dbab6f604`.

Risk is high: the logical change is narrow, but it adds another native syscall
return path whose `rcx` and `r11` registers carry architectural `SYSRETQ`
state and whose checks traverse two page-table ownership domains. Review covers
exact native arguments, checked-role derivation, affine frame identity,
accessed/dirty-bit normalization, complete heap and stack validation, bounded
translation publication and invalidation, user return-address and RFLAGS
preservation, terminal rollback, generic-user regression coverage, full
workspace validation, and duplicate VM execution.

Development qualification found and corrected two provider-representation
mistakes before the original evidence was recorded. The first compared a live
hierarchy entry without normalizing target-maintained accessed state. The second
confused the anonymous-frame base with the separately owned high leaf-table
address. GDB-backed pinned-QEMU traces located both fail-closed branches. The
refined path additionally stages and seals exactly eight leaves, normalizes only
accessed/dirty status, derives the high table from the checked hierarchy root,
uses the anonymous base solely for expected frame identity, and preserves `r11`
throughout. The artifact revisions and hashes above remain the historical
pre-refinement evidence; the pinned-external increment records current evidence.

This increment does not add other `mprotect` ranges or permissions, writable
RELRO after the checked transition, executable publication, splitting,
protection keys, VMA collections, dynamic linking, signals, concurrency,
filesystem access, redistributed BusyBox, containers, or a prompt.
