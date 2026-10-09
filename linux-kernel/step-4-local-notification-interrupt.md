# Step 4 local-notification interrupt increment

## Outcome

The checked kernel root now consumes its restored bootstrap context into one
local-notification send/wait session. A provider-created typed external-
interrupt entry consumes its completion obligation, resumes the interrupted
wait continuation, and returns a refined bootstrap context with the exact prior
local maskable-interrupt state restored. The success marker
`TOPAL_KERNEL_INTERRUPT_OK` is unreachable until that entire lifecycle
completes.

The architecture-independent model assigns event identity 1 to the initial
source and records send, wait begin, observation, typed entry, completion,
resumption, and wait completion in order. It rejects a second send, wait
without pending authority, an unmatched event, completion outside the handler,
resume before completion, duplicate completion, use of the consumed context,
and ordinary completion with a live event.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/9` adds four closed
lowerings: local-APIC self-notification, completion-aware wait, local-APIC EOI,
and interrupt resumption. The boot adapter installs private vector `0xf1` as an
interrupt gate. Both its initial and generated replacement translations retain
one provider-only 2 MiB mapping covering the local-APIC MMIO page; portable
source observes neither that mapping nor the vector or controller addresses.

Send verifies the sealed initial disabled interrupt state and xAPIC mode,
enables the local controller, and issues one fixed edge-triggered self-
notification. Wait uses the architecture-defined interrupt shadow around its
enable-and-halt sequence, restores the prior disabled state immediately after
each wake, and continues until the private completion byte proves that the
selected event—not an unrelated wake—completed. The generated entry preserves
the qualified register set, writes EOI, publishes completion, restores the
interrupted state, and uses the existing generated interrupt return.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/9` records 35
provider-plan identities, 33 linked semantic placements, and a 55-transition
trace. Root object revision `topal.systems-root-object.x86_64/9` has 340 typed
relocations, including one call each to send, wait, and completion, eight
opaque storage references, and two interrupt-return edges. Boot adapter
revision `topal.boot-adapter.linux-x86-protocol-2.15-q35/3` records both entry
addresses and the extra local-APIC mapping.

## Evidence

- architecture-neutral unit tests cover both prior mask states and reject
  duplicate, unmatched, and out-of-order transitions;
- systems source tests cover the accepted entry and lifecycle and reject a
  missing entry, wrong handler type, invalid completion, pre-completion resume,
  wrong send/wait authority, and premature success marker;
- compiler tests cover all 35 provider mappings, exact controller and wait
  bytes, the root-relative replacement PDPT slot, the APIC leaf, both IDT
  gates, complete typed dependencies, linked placements, and semantic trace;
  and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced identical linked
  kernel `56b1c06c6f7a0528db25f87b508cd944f0f942ab6dcf0ee26def60be3f4d1a26`,
  boot image `c742e0e3332668044460f0c4fbea6db43777ff816211f47df97373c91bc5901d`,
  artifact provenance
  `5d7ca60f442b44aa97ce077693c2d2ca03bc997fe0f86a64c98198fbdf5ff94a`,
  boot provenance
  `56554f74b6c875dbe65e3e60dc241289fe391d490b60cf3ee33de3f32195eb6e`,
  and serial observation
  `ff915bb9a88c136f4f969832fb8324f55a75ee7d6394440f7b2fd47fe978e410`.

The committed evidence records the exact eleven-marker order, post-marker QEMU
liveness, and serial quiescence after fatal disposition. During qualification,
QEMU's exception trace also exposed and localized an initially incorrect
replacement-PDPT store before evidence was accepted; the structural regression
test now fixes that root-relative slot explicitly.

## Remaining boundary

This increment does not add timer semantics, shared device interrupts,
controller discovery, routing or affinity policy, multiple pending events,
nested external interrupts, SMP delivery, scheduler disposition, userspace
interruption, interrupt-thread handoff, or executable AArch64/RISC-V providers.
The local notification is an interrupt-control observation, not a memory fence
or a claim about data shared with concurrent producers.
