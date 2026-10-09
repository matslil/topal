# Step 4 atomic-word location increment

## Outcome

The checked kernel root now consumes its 64-byte ordinary bootstrap region
into one aligned unsigned 64-bit atomic location at byte offset 8. It
initializes the word to 41, performs compare/exchange to 42 with
`AcquireRelease` success and `Acquire` failure order, confirms 42 with an
`Acquire` load, consumes `atomic end`, and releases the returned ordinary
region. The success marker `TOPAL_KERNEL_ATOMIC_OK` is unreachable until those
operations complete.

The architecture-independent model owns the entire source region while the
atomic location is live, retains its identity and byte contents, records one
per-location modification order, distinguishes `Exchanged previous` from
`Observed actual`, and returns the same region on end. It rejects unsupported
domains or orders, misalignment, out-of-bounds placement, plain access or
release while atomic, missing or duplicate lifecycle transitions, incomplete
result coverage, and completion with a live location.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/8` adds four closed
lowerings: aligned word initialization, locked compare/exchange, acquire load,
and affine consumption. Three private provider symbols receive a generated
pointer to `topal_bootstrap_storage + 8`; no address, register, instruction, or
encoding is a Topal source value. Compare/exchange contains the exact
`lock cmpxchg qword ptr [rdi], rdx` sequence and returns only success to root
control flow. The qualified x86-64 normal-memory model needs no additional
instruction for the acquire load, which remains a distinct typed facility.

Generated root state rejects plain byte access and region release while the
atomic location is live. It emits independent fatal branches for provider
validation, failed comparison, and load mismatch, then performs the zero-code
ownership transition before the existing release. Artifact revision
`topal.systems-artifact.x86_64-qemu-pc-q35/8` records 31 provider-plan
identities, 28 linked semantic placements, and a 47-transition trace. Gate
schema `topal-kernel-toolchain-gate-qemu/9` binds those revisions and the
unchanged boot-adapter revision `/2`.

## Evidence

- architecture-neutral unit tests cover successful and failed
  compare/exchange, retained contents, modification order, invalid release
  order, bounds, alignment, plain/atomic exclusion, and live-location failure;
- systems source tests cover the accepted lifecycle and reject unsupported
  domains, offsets, success/failure orders, ownership return, and nonexhaustive
  result alternatives;
- compiler tests cover the complete 31-operation provider map, exact provider
  bytes, closed-object properties, three typed root dependencies, 306 root
  relocations, five storage references, 21 fatal branches, 28 placements, and
  the ordered 47-transition trace; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced identical linked
  kernel `0a5b8de49c689ba95ea41b0a2f46605f928f1befd497c51deac62c94abce6abd`,
  boot image `6b06682b4ce56b55f8a67faaeafb71d6e78b79b34f8cfcb7a414efacccc25c20`,
  artifact provenance
  `dd326e622e9f418df3e07884c8879ae20da44410d4099f5e492aa73d519ca900`,
  boot provenance
  `81910c26ffc71aa5947d96cf65577d2d784df1c4d41bf76636058209b81cf335`,
  and serial observation
  `9523ce15810cb26c17d1e3499b53babedf0392c1e92f1c61a8be95f6f2e5c5d5`.

The committed evidence records exact marker order, post-marker QEMU liveness,
and serial quiescence after fatal disposition. Structural inspection, rather
than the single-agent runtime path alone, proves retention of the locked
instruction and both compare/exchange outcomes.

## Remaining boundary

This increment makes no contention or progress guarantee. It does not add
multiple locations, other widths, atomic store or exchange, general
read-modify-write functions, sequential order, lock libraries, SMP startup,
device or DMA atomicity, or executable AArch64/RISC-V providers. AArch64
LSE/exclusive loops and RISC-V A/LR-SC remain architecture pressure tests of
the common result, ordering, ownership, and unobservable-retry contract.
