# Step 4 pinned-QEMU toolchain gate

## Outcome

This is the seventh implementation increment of Step 4. The checked Topal
kernel source now builds through a dedicated lab-only publisher, boots as the
replacement `bzImage` on the pinned `pc-q35-10.2`/`qemu64-v1` QEMU profile,
writes through the polling 16550 provider, enters and resumes from the generated
vector-3 handler, stores and reloads a sentinel in kernel-owned memory, and
reaches the nonreturning fatal provider.

The observed serial byte sequence is exactly:

```text
TOPAL_KERNEL_BOOTTOPAL_KERNEL_FAULT_RESUMEDTOPAL_KERNEL_MEMORY_OK
```

After all three ordered markers, QEMU remained running and the serial stream
stayed unchanged for the settling interval. Combined with structural inspection
of the fatal provider's interrupt-disable/halt loop and the root's separate
store/load/compare sequence, this is the physical evidence that the checked
bootstrap used its generated `.bss` pool and completed its expected path
without returning to a host runtime.

## Reproduction and evidence

The `topal-kernel-toolchain-gate-builder` workspace tool accepts only the
committed kernel root, checks it against the initial systems target selection,
uses the sealed provider and linked-artifact publishers, then feeds that exact
kernel and provenance to the Linux boot adapter. It writes only below its
caller-selected destination and refuses to replace an existing destination.
The QEMU harness builds into ignored temporary state and attaches only the
pinned firmware and ISA serial device; no disk, network device, initramfs, or
host service can account for the markers.

The committed
[evidence record](labs/qemu/x86_64/results/topal-toolchain-gate.json) binds:

- the checked Topal source and Step 3 machine manifest;
- QEMU 10.2.2 and the SeaBIOS image digest;
- the provider, linked-artifact, and boot-adapter schema identities;
- the linked ELF, artifact provenance, boot image, and boot provenance
  digests; and
- the exact serial bytes, ordered markers, post-marker QEMU liveness, and
  post-fatal serial quiescence.

Re-running the harness regenerates all intermediate artifacts rather than
trusting committed binaries. Static tests independently verify the QEMU command
shape and every committed evidence/input relationship that can be checked
without launching the emulator.

## Qualification boundary and risk

This completes the repository's toolchain phase gate for the initial admitted
systems slice: bootstrap entry, polling console write, synchronous debug-break
observation, context-preserving resume, checked kernel-owned-memory use, and
fatal disposition. The byte load is a retained machine operation rather than a
constant-folded echo of the sentinel. This gate does not claim general memory
management, general interrupts, ACPI/PCI discovery, SMP, time, virtio, a
userspace ABI, containers, or hosted virtualization.

The ordinary `topalc` target registry remains model-only. The physical evidence
qualifies this dedicated lab boot path, but the ordinary native compiler still
has no systems-artifact command and must not route this target through its
Linux-process pipeline. Its diagnostic now names ordinary compiler publication
as the remaining integration gap instead of claiming that provider, artifact,
memory-use, or QEMU evidence is absent.

Risk remains high because the evidence covers one closed privileged path and a
small observation window. Mitigations are the sealed source vocabulary,
affine region checking, byte-content reference model, deterministic typed
provider, structural ELF and boot-image inspection, exact machine identity, no
guest devices beyond the serial port, content-addressed evidence, and continued
fail-closed ordinary target selection.
