# Step 4 pinned-QEMU boot and exception gate

## Outcome

This is the seventh implementation increment of Step 4. The checked Topal
kernel source now builds through a dedicated lab-only publisher, boots as the
replacement `bzImage` on the pinned `pc-q35-10.2`/`qemu64-v1` QEMU profile,
writes through the polling 16550 provider, enters and resumes from the generated
vector-3 handler, and reaches the nonreturning fatal provider.

The observed serial byte sequence is exactly:

```text
TOPAL_KERNEL_BOOTTOPAL_KERNEL_FAULT_RESUMED
```

After both ordered markers, QEMU remained running and the serial stream stayed
unchanged for the settling interval. Combined with structural inspection of
the fatal provider's interrupt-disable/halt loop, this is the physical evidence
that the checked bootstrap completed its expected path without returning to a
host runtime.

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

This completes the physical boot and exception portion of the repository's
toolchain phase gate for the initial admitted systems slice: bootstrap entry,
polling console write, synchronous debug-break observation,
context-preserving resume, and fatal disposition. The phase gate's
kernel-owned-memory criterion remains open: the artifact reserves the checked
pool, but the Topal root does not yet allocate, write, read, and release a
region. It also does not claim memory management, general interrupts, ACPI/PCI
discovery, SMP, time, virtio, a userspace ABI, containers, or hosted
virtualization.

The ordinary `topalc` target registry remains model-only. The physical evidence
qualifies this dedicated lab boot path, but the ordinary native compiler still
has no systems-artifact command and must not route this target through its
Linux-process pipeline. Its diagnostic now names the remaining memory-use and
publication gaps instead of claiming that provider, artifact, or QEMU evidence
is absent.

Closing the memory criterion requires `TK-DEC-016`. The approved storage model
defines monotonic allocation, affine release, bounds, alignment, and semantic
placement, but the authoritative language design does not yet define the
source spelling for allocation or any checked byte-access operation on the
opaque `BootstrapRegion`. Those semantics cannot be invented in the backend.

Risk remains high because the evidence covers one closed privileged path and a
small observation window. Mitigations are the sealed source vocabulary,
deterministic typed provider, structural ELF and boot-image inspection, exact
machine identity, no guest devices beyond the serial port, content-addressed
evidence, and continued fail-closed ordinary target selection.
