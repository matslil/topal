# Step 4 bounded static ELF initial stack-limit query

The bounded static process now performs exact
`prlimit64(0, RLIMIT_STACK, null, 0x005ffe60)` after its optional `rseq` probe.
The provider validates the live startup state and query form, publishes the
native 8 MiB soft limit and infinite hard limit, retains no candidate or
mutable resource state, and returns zero. The 291-byte clean-room payload
validates both words, erases the complete scratch pair below the ABI stack
block, restores its entry stack pointer, and continues through `exit_group(0)`.

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-STACK-LIMIT-001` and negative and
structural tests enforce exact arguments, writable extent, result pair,
scratch erasure, marker ordering, and later terminal dispatch. Artifact `/164`,
provider `/154`, provider object `/146`, root `/153`, and gate `/166` identify
the contract. The root has 411 operations, 472 semantic events, 8,942
relocations, and 6,320 console-writer relocations. The systems profile contains
156 rules.

Two independent pinned-QEMU executions reached all 162 observations with
identical evidence SHA-256
`b24187a489afb95c5900564338e533503dec73d251b097781352c025f82bece4`.
Key digests are linked kernel
`dab3bba5ba5852fc15e309157409bd4cd2e14f85c00b1b593a60692b983a18a9`,
boot image `47263e4554ddee6eefe3fd0ba8b8fa368a00d0cadd0dfd6e97e9a9a81c42cae9`,
boot resource `ee3362d9b3a4d7630819b5f3060ef782cd7d8c9c887437af799137e440f0c052`,
static image `2aef7c94fb3d808911b9bd961e2f618a27de83cc8620d2ab944df69ea155cfe2`,
artifact provenance `ed57678778df894d94b64fc4f9b6c866cf8fd8fa1561fe229fb705e2466acf78`,
boot provenance `2834b8474d5473c3e8f4ccdcd2f4b0a0952ac610d8aeb9bfbdcd49e2c388df77`,
source `202450e103b868eb730162c35d72bbf042d2bbde6240264c33f3736a1ff1625d`,
and serial output `537feaa0cb6d952dce0861621d57115375f61127e376fa0e4b84fccd2f08b653`.

Qualification initially placed scratch at `0x005fff90`; later affine rollback
correctly rejected the resulting overwrite of the constructed auxiliary-vector
block. Moving the complete pair to the proven-zero gap below `AT_RANDOM` and
erasing it before stack restoration resolved the failure without broadening the
contract.

Risk is high because this path writes into the prepared Linux stack page.
Review covers candidate placement, non-overlap with the ABI block and random
object, complete publication and erasure, exact stack restoration, immutable
values, full workspace validation, and duplicate VM execution. This does not
add resource mutation, enforcement, accounting, inheritance, another resource,
BusyBox redistribution, or a prompt. The next observed startup dependency is
the optional `readlinkat` probe, whose `ENOSYS` fallback is known to be
acceptable.
