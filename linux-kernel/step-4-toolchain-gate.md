# Step 4 pinned-QEMU toolchain gate

## Outcome

This report tracks the current executable Step 4 gate. The checked Topal
kernel source now builds through the ordinary `topalc` target publisher, boots
as the replacement `bzImage` on the pinned `pc-q35-10.2`/`qemu64-v1` QEMU
profile, validates the Linux E820 handoff and reserved bootstrap floor, selects
and accesses an E820 frame through an opaque kernel mapping, constructs and
activates a replacement bootstrap-equivalent translation, commits and removes
one opaque 4 KiB mapping through that active space, writes through the
polling 16550 provider, enters and restores one affine local-interrupt critical
scope, enters and resumes from the generated vector-3 handler,
stores and reloads sentinels in kernel-owned memory, and reaches the
nonreturning fatal provider.

The observed serial byte sequence is exactly:

```text
TOPAL_KERNEL_FRAME_ALLOCATEDTOPAL_KERNEL_FRAME_MAPPEDTOPAL_KERNEL_TRANSLATION_ACTIVETOPAL_KERNEL_TRANSLATION_EDITEDTOPAL_KERNEL_INTERRUPTS_MASKEDTOPAL_KERNEL_MEMORY_DESCRIBEDTOPAL_KERNEL_BOOTTOPAL_KERNEL_FAULT_RESUMEDTOPAL_KERNEL_MEMORY_OK
```

After all nine ordered markers, QEMU remained running and the serial stream
stayed unchanged for the settling interval. Combined with structural inspection
of the bounded E820 validators and selectors, replacement page-table
zeroing/population, CR3 activation, private leaf construction, commit-time
parent publication, exact local invalidation, fail-to-fatal branches, reserved image
floor, flags capture, local interrupt disable, conditional exact restoration,
fatal provider's interrupt-disable/halt loop, and the root's
separate register-indirect and bootstrap-pool store/load/compare sequences,
this is physical evidence that the checked bootstrap refined its handoff,
activated and edited a replacement translation, accessed both an owned frame
through the committed private mapping and its generated `.bss` pool, and completed its
expected path without returning to a host runtime.

## Reproduction and evidence

The QEMU harness invokes `topalc` with the exact systems target, CPU, and board
selection. The packaging-only `topal-kernel-toolchain-gate-builder` then
accepts that independently published kernel/provenance directory and feeds it
to the Linux boot adapter. Both stages write only below caller-selected
destinations and refuse partial or replacement publication. The harness builds
into ignored temporary state and attaches only the pinned firmware and ISA
serial device; no disk, network device, initramfs, or host service can account
for the markers.

The committed
[evidence record](labs/qemu/x86_64/results/topal-toolchain-gate.json) binds:

- the checked Topal source and Step 3 machine manifest;
- QEMU 10.2.2 and the SeaBIOS image digest;
- the provider, linked-artifact, and boot-adapter schema identities;
- the ordinary `topalc` target-publication identity;
- the linked ELF, artifact provenance, boot image, and boot provenance
  digests; and
- the exact serial bytes, ordered markers, post-marker QEMU liveness, and
  post-fatal serial quiescence.

Re-running the harness regenerates all intermediate artifacts rather than
trusting committed binaries. Static tests independently verify the QEMU command
shape and every committed evidence/input relationship that can be checked
without launching the emulator. Two consecutive reproductions produced the
same linked-artifact, boot-image, provenance, and serial-observation digests.

## Qualification boundary and risk

This completes the repository's toolchain phase gate for the initial admitted
systems slice: bootstrap entry, polling console write, synchronous debug-break
observation, context-preserving resume, checked bootstrap storage, one affine
physical-frame mapping, one replacement translation space, one active
map/unmap edit transaction, one affine local-maskable-interrupt critical scope,
and fatal
disposition. Both byte loads are retained
machine operations rather than constant-folded echoes of their sentinels. This
gate does not claim general address-space management, interrupt entry or
controller management, ACPI/PCI
discovery, SMP, time, virtio, a
userspace ABI, containers, or hosted virtualization.

The ordinary `topalc` target registry and output path publish the qualified
systems artifact without routing it through the Linux-process pipeline. This
QEMU evidence now consumes that exact publication. Linux boot-protocol
packaging remains a separate adapter.

Risk remains high because the evidence covers one closed privileged path and a
small observation window. Mitigations are the sealed source vocabulary,
affine handoff and region checking, conservative range and byte-content
reference models, deterministic typed provider, structural ELF and boot-image
inspection, exact machine identity, no guest devices beyond the serial port,
content-addressed evidence, and continued fail-closed ordinary target selection.
