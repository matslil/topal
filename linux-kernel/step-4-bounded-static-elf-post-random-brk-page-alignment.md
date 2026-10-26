# Step 4 bounded static ELF post-random `brk` page alignment

The bounded static process now supports the exact `brk(0x00619000)` request
issued by the pinned x86-64 BusyBox after its cross-boundary growth to
`0x00618dc0`. The native dispatcher verifies the live 34-page anonymous
extent, every retained heap leaf, the disjoint stack leaf, sealed executable
continuation, and reusable-random service. It then updates only the
byte-granular current break and returns `0x00619000`; no storage allocation or
translation edit occurs. The clean-room payload writes and reads the final
newly admitted byte `0x00618fff` before terminal exit.

Rule `TOPAL-SYSTEMS-STATIC-ELF-POST-RANDOM-BRK-PAGE-ALIGNMENT-001`, negative
translation tests, provider-structure tests, and the exact 514-byte clean-room
fixture enforce the old and new breaks, mapping preservation, final-byte
access, `SYSRETQ` register preservation, terminal state, and complete existing
rollback. Marker
`TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_PAGE_ALIGNMENT_OK` binds runtime
evidence to the new path.

Artifact `/170`, provider `/160`, provider object `/152`, root `/159`, boot
adapter `/70`, and gate `/172` identify the contract. The root has 417
operations, 478 semantic events, 9,241 relocations, and 6,617 console-writer
relocations. The systems profile contains 162 rules.

Two independent pinned-QEMU executions reached all 168 observations with
identical evidence SHA-256
`310bfdddc76f47e6d484e5a40a1a033d26b3fcfdda1d68ea503ca1ee5ae84700`.
Key digests are linked kernel
`89af83473b7549e1b55e83626ded2b93bd4004250937652d6204ba1af1effffc`,
boot image `618c87b43c6e590cc7deb5e05b0f2602a90b65e5afa3d16b65379c67045bc82b`,
boot resource `f9dc27c3356ea81790df6691dc1416a5edcc2513f34f95523965545fdc12abd0`,
static image `df7e6f8e8aa710c7267c45a4e74a074b340f47537fb98a0238ad8be87be3a478`,
artifact provenance
`a8b2e08a109472eac3c6ee1292a3a13796a8940c2af9c5f4f5e0aefc6369e6e9`,
boot provenance
`ee03c314163e0acd0fad3b129d1c8dc89e23702bffebf1d5525360171ee1f2f8`,
source `0d951043a2aa651eac1b11caaa3897772b53e11dd0528620d97b854389a2feec`,
and serial output
`2d300d4192a940f60067dbc8a47667a02c0d7a3d5d631ba1797faba735281718`.

Risk is high: the logical change is narrow, but it extends a native syscall
return path whose `rcx` and `r11` registers carry architectural `SYSRETQ`
state. Review covers exact request and lifecycle state, full retained-leaf
validation without mutation, stack/heap disjointness, reusable-service
preservation, user return-address and RFLAGS preservation, final-byte access,
terminal rollback, generic-user regression coverage, full workspace
validation, and duplicate VM execution.

Development qualification found and corrected two independent issues before
evidence was recorded: an unrelated stack zero-region count was initially
mistaken for the fixture byte length, and the first leaf-validation loop used
`r11` as scratch even though `SYSRETQ` consumes it as user RFLAGS. Targeted
GDB-backed QEMU runs located both fail-closed boundaries; structural tests now
lock the distinct stack count, terminal state, and `r11` preservation. The two
final evidence runs completed identically.

This increment does not add arbitrary `brk` policy, shrinking, allocation,
translation edits, `mmap`, `mprotect`, signals, concurrency, filesystem access,
redistributed BusyBox, containers, or a prompt.
