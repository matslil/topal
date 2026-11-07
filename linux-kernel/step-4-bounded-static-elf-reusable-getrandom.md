# Step 4 bounded static ELF reusable `getrandom`

The static process now owns a reusable affine random service. Initial-random
delivery moves the reserved 16-byte first output and generation-two ChaCha20
key into provider-private process storage before the provider-global entropy
workspace and every device, DMA, transport, pool, and sample resource are
destroyed. The first request remains byte-for-byte compatible with the prior
increment.

For later successful requests, the syscall path validates Linux flags, a
length from one through 16, and the complete overflow-free writable static
extent before changing secret state. It switches from the user stack to the
suspended privileged continuation stack, performs the qualified ChaCha20
fast-key-erasure extraction, restores every native return register, copies and
verifies the requested prefix, and wipes the complete temporary output. Zero,
`-EINVAL`, and `-EFAULT` paths do not advance the service.

Rule `TOPAL-SYSTEMS-STATIC-ELF-REUSABLE-GETRANDOM-001` and negative and
structural tests enforce exclusive key ownership, validation before evolution,
the exact 421-byte clean-room startup sequence, a second eight-byte
nonblocking request, opaque observation without a value oracle, complete
per-call wipe, and terminal destruction. Terminal completion erases and
verifies every private key, output, reference, counter, and stack scratch byte.
It normalizes hardware Accessed/Dirty bits across the accepted writable static
mapping before the existing reversible qualification chain.

Artifact `/167`, provider `/157`, provider object `/149`, root `/156`, and gate
`/169` identify the contract. The root has 414 operations, 475 semantic events,
9,087 relocations, and 6,463 console-writer relocations. The systems profile
contains 159 rules.

Two independent pinned-QEMU executions reached all 165 observations with
identical evidence SHA-256
`bfa71148c9fea49b8630ebed6e653c19e48e2ba6d4421cdd6639e988a5328938`.
Key digests are linked kernel
`bc134ad16846a64cf06517df6f8efc2d4e1f283f19f40e6ba696372dd5cb408b`,
boot image `62ee15f14a73d837c1e50394ca03597393637f759aa12eb45287d0dd5721a376`,
boot resource `c9e20b6a758c5c24394f6ae10f3fc2f5dad64950b16bb383846bb71092a3f88b`,
static image `86a9c84edab439c6e444066e8ef1070faded0215dd2babcee38f75683bccea39`,
artifact provenance
`4eb396ddf418cd50abb61f9ca3e1cb5b9a301356ea07a83cab51bf0f61e9d29b`,
boot provenance
`5e74d1d333b2ce7515418de543a56b8684ea7b90552b99a98ad0017396b5ede0`,
source `b57582cbaf30dab1f41f4e9e479fc4ba534b05b2a6496b29b6953a381e12b748`,
and serial output
`21d79ae36c91d8669a46eed996e71bb3a2977ae140287f0f22289b3d0b869336`.

Risk is high because secret generator ownership now spans user execution and a
native syscall performs cryptographic work after a privileged stack switch.
Review covers affine ownership, first-versus-later output selection, ABI state
preservation, validation-before-evolution, key replacement before publication,
complete wipe, terminal destruction, page-table status normalization, full
workspace validation, and duplicate VM execution. This does not add requests
larger than 16 bytes, blocking, signals, concurrency, inheritance, reseeding,
random device files, vDSO delivery, redistributed BusyBox, or a prompt.
