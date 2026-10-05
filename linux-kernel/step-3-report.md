# Step 3 report: x86-64 Linux reference VM

## Outcome

Step 3 is complete for the agreed x86-64 scope. The committed lab recipes
construct a pinned QEMU VM, provision a small Debian 13.7 userspace, and verify
it first with Debian's standard cloud kernel and then with the exact Linux
7.2.9 compatibility baseline. Both boots reach a standard serial login and SSH
shell. The normal shell, a Docker container, and a Podman container all pass
the same smoke marker.

The implementation is in [the x86-64 QEMU lab](labs/qemu/x86_64/README.md).
Downloaded images, Linux source, SSH keys, writable disks, logs, and build
products remain outside version control.

## Resolved VM decisions

### `TK-DEC-001` — board, firmware, and boot path

The qualified board is QEMU 10.2.2 `pc-q35-10.2` under TCG with the versioned
`qemu64-v1` CPU, two vCPUs, 640 MiB RAM, and a hash-pinned SeaBIOS 1.17.0
image. It exposes an ISA 8250 serial console and modern non-transitional
virtio-pci block, network, and random devices at fixed PCI addresses. The
cloud-init seed is a read-only fourth PCI function retained in both qualified
boots so ACPI and PCI discovery do not drift between controls.

The control uses SeaBIOS disk boot. The reference uses QEMU's x86 direct Linux
boot with a `bzImage`; SeaBIOS remains the firmware. This is also the Step 4
replacement seam. The complete values, feature choices, persistence policy,
and network topology are machine-readable in
[the manifest](labs/qemu/x86_64/manifest.json).

### `TK-DEC-012` — reference kernel configuration

The full [Linux 7.2.9 configuration](labs/qemu/x86_64/reference-kernel.config)
is committed and hashed. It excludes the i386 and x32 userspace ABIs, enables
the exact storage/console/firmware devices, and includes the namespaces,
cgroup v2, seccomp, BPF, overlayfs, bridge/veth, Netfilter/nftables, procfs,
sysfs, devtmpfs, and security facilities needed by the initial application and
container corpus. Devices absent from the qualified board are not fabricated.

The build metadata and toolchain identities are pinned, and the resulting
`bzImage` must hash to
`c005235648520b35e104e3d8025abf48f6f523697a54f491b9cf692db53fae0c`.
The binary is a reproducible external build result and is not committed.

### `TK-DEC-013` — root filesystem and initial corpus

The root is Debian 13.7 `genericcloud` build `20261001-2618`, with its SHA-512
fixed in the manifest. Additional packages come from the Debian archive and
security snapshots at `20261002T000000Z`. The observed corpus pins glibc
2.41-12+deb13u4, BusyBox 1.37.0, Docker 26.1.5, and Podman 5.4.2.

The corpus is an initial executable witness, not the definition of Linux
compatibility. Complete interface status remains governed by the Linux 7.2.9
inventory and dispositions.

## Verification result

The committed [verification evidence](labs/qemu/x86_64/results/verification.json)
records:

| Boot | Kernel | Shell | Docker | Podman |
| --- | --- | --- | --- | --- |
| distribution control | `6.12.111+deb13-cloud-amd64` | pass | pass | pass |
| pinned reference | `7.2.9` | pass | pass | pass |

Both runs observed identical PCI functions and ACPI table hashes and selected
the TSC clocksource. The evidence also captures memory/resource discovery,
interrupts, consoles, block topology, namespaces, cgroup filesystems, package
versions, and kernel command lines. The container tests are rootful and use a
locally imported static BusyBox filesystem with networking disabled, keeping
registry behavior out of the kernel result.

## Limits and next step

This PR does not claim complete Linux UAPI, rootless-container, KVM, nested-VM,
or OCI-runtime qualification. Those remain distinct gates and open decisions.
It also does not implement Topal kernel code.

Step 4 can now begin against a stable target: produce the smallest bootable
Topal x86-64 artifact for this exact QEMU board, replace the reference
`bzImage`, and advance through the acceptance ladder without changing the
userspace control.

## Risk and review

Risk is medium: the change is test infrastructure and configuration rather
than language semantics, but a drifting VM could invalidate later comparison.
The mitigation is fail-closed input, firmware, configuration, and artifact
hashing; versioned QEMU machine and CPU models; an immutable package snapshot;
explicit device options; and differential observations from two kernels on
the same board. The complete generated configuration and evidence were
self-reviewed against `LK-X64-BOARD-002`, `LK-SCOPE-004`, and the Step 3
request.
