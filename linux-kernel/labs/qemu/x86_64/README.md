# Reproducible x86-64 Linux reference VM

This lab is the Step 3 control environment for the Topal kernel project. It
boots one pinned QEMU board in two ways:

1. the Debian cloud image's distribution kernel, to prove that the image and
   provisioning are independently usable; and
2. the project's exact Linux 7.2.9 reference `bzImage`, supplied with QEMU's
   direct Linux boot interface, to establish the replacement-kernel path.

Both boots use the same writable root disk, firmware, CPU model, memory size,
PCI topology, ACPI tables, serial console, and network configuration. The
[machine manifest](manifest.json) is authoritative for the lab profile. The
[recorded evidence](results/verification.json) contains the observed board,
kernel, userspace, Docker, and Podman results.

## Host prerequisites

The recorded run used QEMU 10.2.2, SeaBIOS 1.17.0, GCC 16.2.1, binutils
2.46.1, GNU Make 4.4.1, Python 3.14.7, `genisoimage` 1.1.11, OpenSSH, and the
repository's `scripts/run_bounded.py`. Exact identifying strings and firmware
hashes are in the manifest. The lab rejects a different QEMU version or
firmware digest. The expected reference configuration and `bzImage` hashes
also make a toolchain-induced build difference fail closed.

Generated state lives below `linux-kernel/.cache/qemu-x86_64` and
`linux-kernel/build/x86_64`; neither is versioned. The Debian and Linux
downloads are verified before every operation. The writable 12 GiB qcow2 is an
overlay, so the downloaded Debian image remains unchanged.

## Reproduce the lab

From the repository root:

```console
python3 linux-kernel/labs/qemu/x86_64/vm.py fetch
python3 linux-kernel/labs/qemu/x86_64/vm.py prepare --reset
scripts/run_bounded.py -- python3 linux-kernel/labs/qemu/x86_64/vm.py \
  configure-kernel
scripts/run_bounded.py -- python3 linux-kernel/labs/qemu/x86_64/vm.py \
  build-kernel
scripts/run_bounded.py -- python3 linux-kernel/labs/qemu/x86_64/vm.py \
  verify-all --write-evidence
```

The first provisioning boot downloads packages from the timestamped Debian
snapshot and can take several minutes under TCG. `verify-all` waits for that
operation, then runs the two boot profiles serially. To record the profiles in
separate invocations, use `verify --boot distro --write-evidence` followed by
`verify --boot reference --write-evidence`.

Regenerating the committed kernel configuration is an explicit review action:

```console
scripts/run_bounded.py -- python3 linux-kernel/labs/qemu/x86_64/vm.py \
  configure-kernel --write-config
```

Update the configuration and expected artifact hashes in the manifest in the
same reviewed change. A normal build does not modify the committed
configuration.

## Interactive prompts

Run either boot with a serial terminal:

```console
scripts/run_bounded.py -- python3 linux-kernel/labs/qemu/x86_64/vm.py \
  run --boot distro
scripts/run_bounded.py -- python3 linux-kernel/labs/qemu/x86_64/vm.py \
  run --boot reference
```

The disposable serial-console account is `topal` with password `topal`.
Password authentication over SSH is disabled. The only host-forwarded port is
localhost TCP 2222, and the generated SSH private key remains in ignored lab
state:

```console
ssh -i linux-kernel/.cache/qemu-x86_64/state/id_ed25519 \
  -p 2222 topal@127.0.0.1
```

## What the smoke test proves

The ordinary guest shell checks a dynamic glibc executable, files, processes,
`/proc`, `/sys`, device nodes, cgroup filesystems, and all namespace kinds.
Provisioning imports the installed static BusyBox into separate Docker and
Podman stores without relying on an external container registry. Each runtime
then starts a rootful, network-disabled container and exercises `/proc` from
inside it.

The evidence additionally records ACPI table hashes, PCI functions, the
firmware memory map, interrupt routing, clocksource, consoles, block topology,
kernel command line, package versions, and runtime versions. Matching ACPI and
PCI observations across the two boots guard against accidentally comparing
different virtual boards.

This is a Step 3 smoke profile, not container qualification. Rootless
operation, container networking, storage drivers beyond this local import,
security-policy combinations, and exhaustive OCI behavior remain under
`TK-DEC-014` and the later container gate.

## Replacement-kernel seam

The `reference` boot passes a Linux-protocol `bzImage` and command line to the
pinned SeaBIOS/QEMU board while retaining the same root disk and devices. Step
4 can replace that kernel path with the Topal-produced x86-64 boot artifact;
the userspace image, machine contract, and verification commands do not need
to change. This choice does not require a Topal source-assembly escape hatch:
the approved systems profile still requires the compiler backend to emit the
typed entry and machine support.

## Topal toolchain gate

Step 4 now exercises the initial Topal kernel image on the same pinned QEMU,
SeaBIOS, machine, CPU, memory, topology, UUID, RTC, and serial-console profile.
The focused gate intentionally attaches no root disk or network device because
this kernel increment tests boot-memory handoff validation, privileged entry,
console output, a resumable debug-break exception, checked kernel-owned-memory
use, one completed local-notification interrupt, two HPET-backed monotonic-clock
observations, one HPET comparator-backed one-shot deadline event, and the
nonreturning fatal path rather than userspace.

From the repository root:

```console
scripts/run_bounded.py -- python3 \
  linux-kernel/labs/qemu/x86_64/topal_gate.py --write-evidence
python3 -m unittest \
  linux-kernel/labs/qemu/x86_64/test_topal_gate.py -v
```

The first command publishes the checked Topal source through the ordinary
`topalc` systems target, passes that artifact to the packaging-only
`topal-kernel-toolchain-gate-builder`, and writes the Linux boot image into
ignored temporary state. It then boots through QEMU's direct Linux interface
and waits for the thirteen ordered serial markers. The generated real-mode adapter
first collects the bounded SeaBIOS E820 continuation into the Linux zeropage;
the first marker is reachable only after the generated description validator
proves an allocatable page above the closed bootstrap reservation floor, the
provider's one-frame selector succeeds, and the root establishes and releases
its affine extent. The harness
proves QEMU is still running and the serial stream is quiescent in the fatal
halt before terminating it. The committed
[toolchain-gate evidence](results/topal-toolchain-gate.json) binds the source,
manifest, firmware, generated artifacts, provider, QEMU identity, and observed
serial bytes by digest.
