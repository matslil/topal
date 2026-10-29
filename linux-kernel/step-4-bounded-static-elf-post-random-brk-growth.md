# Step 4 bounded static ELF post-random `brk` growth

The bounded static process now supports the exact program-break growth issued
by the pinned x86-64 BusyBox after its repeated `getrandom` startup work and
post-random `brk(0)` query. The native syscall dispatcher admits only growth
from `0x005f7d80` to `0x00618d80`, maps 34 zeroed anonymous pages, validates a
write and read at the final admitted byte `0x00618d7f`, and updates the current
break only after every leaf is present.

The growth crosses the 6 MiB leaf-table boundary. To keep target details out of
source while preserving a disjoint bounded stack, the x86-64 realization now
owns a five-frame hierarchy: its fifth frame is an initially empty high user
leaf table. The stack moved from the top of the low table to
`[0x007ff000,0x00800000)`, and the separately owned anonymous reserve remains
exactly 34 data frames. Terminal return removes all 34 heap leaves, clears and
returns every data frame, retains the high-table parent long enough to revoke
the stack leaf, and finally verifies and releases the complete hierarchy.

Rule `TOPAL-SYSTEMS-STATIC-ELF-POST-RANDOM-BRK-GROWTH-001`, negative
translation tests, provider-structure tests, and the exact 476-byte clean-room
fixture enforce syscall 12, the exact old and new break, lifecycle ordering,
zero backing, boundary-crossing publication, final-byte access, and fail-closed
handling of every other request. Marker
`TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_GROWTH_OK` binds runtime evidence to
the new path.

Artifact `/169`, provider `/159`, provider object `/151`, root `/158`, and gate
`/171` identify the contract. The root has 416 operations, 477 semantic events,
9,184 relocations, and 6,560 console-writer relocations. The systems profile
contains 161 rules.

Two independent pinned-QEMU executions reached all 167 observations with
identical evidence SHA-256
`b21d853ca1f0adf5c334913f98679a738ceb67d861f94dad30ab2dc425754d49`.
Key digests are linked kernel
`2923db77978fe4d6f2336d93664dc313b022d874e381749f0fd33fc0db4e666d`,
boot image `b3ce66efe07b9f7979a5560d1c683c96a0fd07faa09719b2d378ea4c38bd7ce6`,
boot resource `c65b85e8d89dbc5442bb0e80855ed9bdd33b01aad27a97a2b458731ea97c04cd`,
static image `24ba5c0c76d4ab935d63e50f40835fb25dee8385e3d9742743cd8eeaba5a02ca`,
artifact provenance
`5352be6c8e1c9d7f1a19da1e86d3d9496a7dc310c362d1d1f28aefa90bfb8ce0`,
boot provenance
`e14ac124eaf1e4b84a6e574967b1f341ac7450b3387314829905b59c7f42c453`,
source `77bc5586653b051c4f213686fc5368cb8ff810efe0c9c6904a2cabcb311cc749`,
and serial output
`89af5764bd7e0d38da3f63733e3ba547fdd721d5270693f466b35175d349703b`.

Risk is high: the change expands a live process heap across a page-table
boundary and revises the target-private static address-space layout. Review
covers hierarchy ownership, stack/heap disjointness, initial-versus-later heap
state, exact mapping order, last-byte access, generic-user regression coverage,
complete rollback, full workspace validation, and duplicate VM execution. One
intermediate VM run completed the static-process rollback but timed out at an
existing later scheduler boundary; the preceding run and both recorded
qualification runs completed all observations.

This increment does not add arbitrary `brk` policy, shrinking, `mmap`, signals,
concurrency, filesystem access, redistributed BusyBox, containers, or a prompt.
