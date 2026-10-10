# Step 4 kernel-context-transfer increment

## Outcome

The checked kernel root now consumes one 16 KiB bootstrap-reclaimable region
into a typed `SuspendedKernelContext InitialProcessor`, transfers from the
running bootstrap continuation to its `kernel-thread` entry, terminally retires
that worker to the exact suspended caller, and reclaims the completed worker
stack before the bootstrap context reaches its fatal disposition.

The architecture-independent model owns the continuation, stack, processor,
address-space identity, and entry identity as one affine value. Creation,
selection, retirement, caller resumption, and reclamation are distinct
transitions. Retirement is admitted only at a transfer-safe point with no live
obligations. The source checker rejects wrong processor classifiers, stack
shape, entry identity, transfer token, retirement caller, resumed marker, and
completed token.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/12` adds four generated
operations. Creation validates the exact stack extent and disabled-interrupt
state, constructs a provider-private initial callee-saved register image, and
retains the typed worker entry. Transfer saves the bootstrap callee-saved
continuation and stack pointer before selecting the worker stack. Retirement
marks the worker completed, restores the exact saved caller stack and
callee-saved registers, and returns through the suspended provider call.
Reclamation consumes both saved stack pointers and closes the state machine.

The 16 KiB physical stack begins at offset 4096 in the bounded bootstrap pool.
That provider placement preserves the source request's 16-byte alignment while
keeping it disjoint from the clock, deadline, interrupt, and context state in
the low page. The source cannot name stack pointers, registers, register-save
order, continuation addresses, or provider state offsets.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/12` records 45
provider-plan identities, 44 linked semantic placements, and a 76-transition
trace. Root object revision `topal.systems-root-object.x86_64/12` has 472 typed
relocations: 386 console writes, 19 opaque storage references, 32 fail-to-fatal
edges, three interrupt returns, one reference to each context primitive, and
one reference to the generated worker entry. Provider-object revision
`topal.provider-object.x86_64-qemu-pc-q35/12` contains the inspected save,
stack-select, restore, and terminal-reclaim sequences. Boot adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/5` is unchanged because the
resumed thread is an internal linked root rather than a hardware entry vector.

## Evidence

- architecture-neutral tests cover create/transfer/retire/reclaim and reject
  the wrong stack contract and retirement with live obligations;
- source tests cover the exact typed handler and round trip, including negative
  processor, entry, affine-token, marker, and reclamation cases;
- compiler tests cover all 45 provider mappings, exact generated state-machine
  instructions, stack construction, callee-saved save/restore, exact caller
  selection, linked worker placement, relocation ledger, and semantic trace;
- the complete compiler library suite passes 188 tests; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced the linked kernel
  `045492103102adcff4b959c2a37629a970b05eaa76553fa780d786c1fff2538a`,
  boot image
  `a039d0cdbb5f039f0b1ec7a97943da4d875cf8a63133efbe5a6b7c1a09570763`,
  artifact provenance
  `664ca6453ba6db4d6b27237cb904c00efc0a398caeee9be7145cf7fb26244324`,
  boot provenance
  `7445852b3a9e53f07b388ddf9459d3b4c9c4abd9d095dc201319b17281f74b62`,
  and serial observation
  `dca1772aa9205e566d1e6e84b9ce113ffefc4a7acdea0f19663256d9582501cd`.

Gate schema `topal-kernel-toolchain-gate-qemu/13` records all fifteen ordered
markers. `TOPAL_KERNEL_CONTEXT_ENTERED` proves execution reached the generated
worker on its constructed stack; `TOPAL_KERNEL_CONTEXT_RESUMED` is emitted only
after terminal retirement restored the suspended bootstrap caller. QEMU then
remains running with quiescent serial output in the fatal halt.

This is high-risk continuation and stack-ownership work. Review combines the
authority-ordered design check, cross-architecture semantic pressure test,
affine negative tests, exact machine-code and ELF inspection, two physical
QEMU runs, and full workspace validation.

## Remaining boundary

This increment does not add preemption, scheduler policy, run queues, blocking
tasks, migration, SMP context ownership, user/kernel transitions, multiple
address spaces, extended floating-point/vector state, debug state, guard
pages, stack growth, cancellation, or executable AArch64/RISC-V providers.
Those remain later reviewed increments; they are not implicit in this minimal
same-processor, same-address-space round trip.
