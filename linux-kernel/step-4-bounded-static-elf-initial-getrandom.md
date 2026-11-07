# Step 4 bounded static ELF initial `getrandom`

The bounded static process now consumes a private affine reservation derived
from bytes 16 through 31 of the already-qualified initial entropy output. The
first half still supplies `AT_RANDOM`; all provider output storage, the
generator, and every other entropy resource are destroyed before process
entry. After its `readlinkat` fallback, the process performs the observed
`getrandom(0x005f5e68, 8, GRND_NONBLOCK)` request. Native syscall 318 validates
the exact request and writable static destination, publishes and verifies the
opaque eight-byte prefix, wipes all 16 reserved bytes, marks the reservation
consumed, and returns eight.

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-GETRANDOM-001` and negative and
structural tests enforce provenance, the exact 373-byte clean-room startup
sequence, single-use consumption, complete wipe, marker ordering, and later
terminal dispatch. Terminal completion requires consumed zero state and clears
the service marker before the qualification inverse. Because the write makes
the destination leaf Accessed and Dirty, restoration normalizes those hardware
status bits alongside the already-qualified entry and stack leaves.

Artifact `/166`, provider `/156`, provider object `/148`, root `/155`, and gate
`/168` identify the contract. The root has 413 operations, 474 semantic events,
9,041 relocations, and 6,418 console-writer relocations. The systems profile
contains 158 rules.

Two independent pinned-QEMU executions reached all 164 observations with
identical evidence SHA-256
`2979252394c7737622d33beb0e1ef5abf149b198bc60189d6efbe334a8c5ca27`.
Key digests are linked kernel
`b4d3810875e76605d65951dc6f5c0dc341d874662bc115d0b70c2fd1710db03a`,
boot image `9e64962f649a09f39ce3a708c15973722f5ac8864621fa31b06da8e0beb72c75`,
boot resource `f20c0d42f0ef6014d6438dfba1167302156b55a43855916b29d4289570fd62b4`,
static image `1e13d3288ae001015baa0223c656cb94af78d2050ae4af7f61e0b812fdcd1505`,
artifact provenance
`28aecb47b3a16933cf1b81bfdf6a37ebdb6e4cc97fe4a3a27aa71eff258e1253`,
boot provenance
`99b1241ccc2cec91f6f23a4e666a75518761616c085bd3e74f10d6716ceafa47`,
source `17795e109c6707dc77aefb64a7983a439b5a75ec746ebbae38adda4571e80638`,
and serial output
`69062ce0c403a7f1b9d09b878da3228e9cd08b86eea2107c0fb9d419c6d370d4`.

Risk is high because this increment carries secret-derived state across the
entropy teardown boundary, adds a writable static-image syscall path, and must
restore both software ownership and hardware page-table status. Review covers
affine provenance, exact ABI selection, destination validation, opaque
publication without a value oracle, complete single-use destruction, terminal
state, page-table normalization, full workspace validation, and duplicate VM
execution. This does not retain the generator, admit a second nonempty request,
implement blocking or reseeding, redistribute BusyBox, or provide a prompt.
