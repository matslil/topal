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
observations, and deadline-driven preemption and later restoration of one
kernel context, followed by one qualification-only ring-3 syscall sequence
covering `getpid`, `uname`, zero-duration `nanosleep`, and `exit_group` before
the nonreturning fatal path. It does not boot a general userspace or attach its
reference root disk.

From the repository root:

```console
scripts/run_bounded.py -- python3 \
  linux-kernel/labs/qemu/x86_64/topal_gate.py --write-evidence
python3 -m unittest \
  linux-kernel/labs/qemu/x86_64/test_topal_gate.py -v
```

The first command first publishes the generated qualification image, derives a
structurally compatible fixture with distinct initialized data, and then
publishes the checked Topal source through the ordinary `topalc` systems target
using `--initial-user-image`. It passes that artifact and the exact published
user image to the packaging-only
`topal-kernel-toolchain-gate-builder`, and writes the Linux boot image into
ignored temporary state. The packager verifies the image against artifact
provenance instead of regenerating it. The harness then boots through QEMU's
direct Linux interface
and waits for the 51 ordered serial markers. The generated real-mode adapter
first collects the bounded SeaBIOS E820 continuation into the Linux zeropage;
the first marker is reachable only after the generated description validator
proves an allocatable page above the closed bootstrap reservation floor, the
provider's initial one-frame selector succeeds, and the root establishes and
releases its affine extent. The later `TOPAL_KERNEL_FRAME_EXTENT_OK` marker is
reachable only after the same provider proves that a complete static
three-frame extent lies in one E820 RAM run and the root releases it as a
whole. The final context markers prove that the deadline interrupts
the worker in its sealed region, the handler returns to the exact dispatcher,
source runs the terminal and completion workers in the checked order, and
redispatch resumes then retires both suspended workers. The final user marker
is reachable only after a ring-3 probe receives PID one from Linux x86-64
`getpid`, completes and faults both output-direction `uname` and
input-direction zero-duration `nanosleep` transfers, issues `exit_group(0)`,
and restores the exact kernel continuation. The last marker additionally
requires the allocator-backed user code, stack, and leaf-table extent to have
its mappings revoked, its three relevant virtual locations invalidated, its
opaque address-space state reclaimed, its exact affine ownership returned, and
the extent released by source. Construction, user-context binding, revocation,
and reclaim are separate checked operations in the artifact trace. The
`TOPAL_KERNEL_FRAME_DISJOINT_OK` marker additionally requires the prepared
address space and one source-owned frame to coexist without overlap, followed
by release of that frame before user-context binding. The preceding
`TOPAL_KERNEL_USER_MAPPING_OK` marker requires the provider to map that frame
RW/NX at the sealed guard location, return distinct affine map, stored,
read-only protected, and verified refinements through two fresh sequential
generations and clear writable authority before verification. For the first
generation, the provider installs and verifies an exact opaque entry payload;
the second retains the fixed-byte data qualification. It then promotes the
first verified RO/NX mapping to role-specific RO/X code with target
invalidation and retains it while a separately allocated, disjoint frame is
mapped at the sealed data location. After content verification, the second
mapping is promoted through a distinct writable-data role from RO/NX to RW/NX
with target invalidation; execute authority remains absent. The intervening
`TOPAL_KERNEL_USER_IMAGE_READY` marker is emitted while both role owners still
coexist. The following `TOPAL_KERNEL_BOOT_RESOURCE_OK` marker proves that the
provider validated the complete deterministic CPIO resource supplied through
QEMU `-initrd` and kept its physical extent outside frame allocation. Both
mappings then remain resident while the existing sealed probe
begins at the constructed code mapping with an aligned sealed initial stack.
The constructed entry validates and consumes exact `argc`, `argv`, empty
environment, provider-consistent `AT_PAGESZ=4096`, `AT_ENTRY=0x604000` bound
to its own first instruction, `AT_EXECFN` aliased to `argv[0]`, terminating
zero-valued `AT_UID`/`AT_EUID`/`AT_GID`/`AT_EGID` root credentials,
resident `AT_PHDR`/`AT_PHENT`/`AT_PHNUM` facts, terminating auxiliary-vector,
and the shared string contents before
directly observing the initialized sentinel, proving the BSS byte starts at
zero, writing `0xa5` through the writable mapping, reading it back, and then
transferring to the existing syscall and fault-recovery probe. A mismatch
takes the payload's fail-closed trap instead. After its existing compatibility checks,
the probe transfers to a sealed continuation in the constructed mapping. That
continuation reserves a bounded buffer in the existing RW/NX user stack,
invokes Linux x86-64 `uname`, requires a complete transfer, validates the
returned `Linux` identity, and restores the stack. After output disposal it
stages a zero-duration native timespec in the same
stack, invokes `nanosleep` with no remainder candidate, requires complete
input transfer and a successful return, and restores the stack again. It then
constructs the 26-byte `TOPAL_KERNEL_USER_WRITE_OK` payload in another bounded
stack region and invokes Linux `write(1, candidate, 26)`. Dispatch admits a
caller-selected one-to-256-byte extent only when it is completely contained in
a known readable user mapping. It copies the complete extent to private kernel
staging before output, publishes the staged prefix through the checked Q35
serial provider, and returns the argument-derived byte count; the provider has
no embedded copy of the application payload and requires no baked call site.
The user validates that result and restores its stack, so the payload is
observable before the source-owned post-return witnesses. The harness connects
a pseudo-terminal to the same emulated serial
device and injects `q` after the boot-resource witness. The continuation next
invokes `read(0, candidate, 1)` into a bounded RW/NX stack region. Dispatch
polls and privately stages that byte, copies it to the exact user candidate
under fault recovery, and returns one; user mode validates the count and byte
before restoring its stack. `TOPAL_KERNEL_USER_READ_OK` is the first source-
owned post-return witness and proves that path reached terminal completion.
The continuation then invokes `getpid`, requires PID one, takes the
same trap on mismatch, and issues
terminal `exit_group(0)` without returning to the provider probe. After the
exact kernel return,
`TOPAL_KERNEL_USER_IMAGE_ENTRY_OK` proves that entry path completed,
`TOPAL_KERNEL_USER_IMAGE_STACK_OK` proves initial-stack validation succeeded,
`TOPAL_KERNEL_USER_IMAGE_AUXV_OK` proves the provider-sealed base-page-size
entry was validated before `AT_NULL`,
`TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK` proves the sealed executable-entry
fact matched the selected first instruction,
`TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK` proves executable-name pointer identity
and shared in-block string validation,
`TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK` proves the coherent sealed root
credential fact group,
`TOPAL_KERNEL_USER_IMAGE_ELF_OK` proves the resident ELF64 header, three-entry
program-header table, selected entry, and initial PHDR facts agree,
`TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK` proves that the executable mapping was
populated from the validated external `init` member before W^X finalization,
`TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK` proves that the finalized mapping
was compared completely back to that same external payload without a second
provider-embedded executable identity,
`TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK` proves executable copy and
post-protection comparison used the relationally validated runtime ELF extent
rather than a build-time unrolled byte count,
`TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK` proves the initial instruction
pointer and `AT_ENTRY` were both taken from one ELF entry value validated
inside that executable extent rather than a provider-embedded runtime address,
`TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK` proves initialized-data copying,
explicit BSS zeroing, and post-protection verification used the relationally
validated runtime ELF file and memory extents rather than fixed qword counts,
`TOPAL_KERNEL_USER_IMAGE_BSS_OK` proves initialized data was copied from the
same external ELF resource and the complete memory-only tail remained zero
after RO/NX publication,
`TOPAL_KERNEL_USER_IMAGE_DATA_OK` proves the data observation succeeded, and
`TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK` proves the initialized user program
successfully changed and reread its BSS through the verified RW/NX data role,
and
`TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK` proves the constructed output transfer,
`TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK` proves the constructed input transfer,
`TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK` proves the bounded syscall succeeded, while
`TOPAL_KERNEL_USER_IMAGE_RETAINED` proves both role owners still
coexist before data-first and code-second revocation return their exact frames.
The private construction placements remain distinct from the probe's code,
stack, transfer candidates, and deliberate `0x602000` fault guard. The harness
proves QEMU is still running and the serial stream is quiescent in the fatal
halt before terminating it. The committed
[toolchain-gate evidence](results/topal-toolchain-gate.json) binds the source,
manifest, firmware, generated artifacts, external boot resource, provider,
QEMU identity, and observed serial bytes by digest.
