# x86-64 platform and board-support map

## LK-X64-BOARD-001 — Firmware-driven board description

The initial board is a version-pinned QEMU x86-64 machine, to be selected in
the VM phase. Its board contract shall list every CPU, memory region, firmware
table, interrupt controller, timer, PCI host bridge, storage controller, network
device, console, entropy device, firmware interface, and power/reset mechanism.

Unlike the later AArch64 design input, the initial x86 machine is expected to
use ACPI, PCI discovery, and standard x86 firmware conventions rather than
Device Tree as its principal description. Board support therefore separates:

- machine/firmware entry and memory discovery;
- ACPI table validation and typed interpretation;
- PCI hierarchy and configuration-space enumeration;
- architectural interrupt and timer discovery; and
- drivers bound to discovered resources.

The Topal architecture model describes qualified hardware facts used for code
generation and validation. Firmware tables are runtime input from an external
producer. Neither source is allowed silently to override the other: mismatch
must reject, degrade through an explicit supported path, or produce a precise
diagnostic according to the owning contract.

Linux 7.2.9 source families used to construct the eventual manifest include
`arch/x86/boot/`, `arch/x86/kernel/setup.c`, `arch/x86/kernel/acpi/`,
`drivers/acpi/`, `drivers/firmware/efi/`, `drivers/pci/`, `drivers/virtio/`,
`drivers/tty/serial/`, `drivers/clocksource/`, and `drivers/char/hw_random/`.
These locations explain Linux implementation responsibilities; the external
machine and firmware specifications remain authoritative for hardware meaning.

## LK-X64-BOARD-002 — Qualified board manifest

Step 3 must emit a machine-readable manifest alongside its QEMU launch recipe.
At minimum it pins:

| Manifest area | Required facts |
| --- | --- |
| execution | QEMU version, accelerator, machine name/version, CPU model/features, vCPU count |
| boot | firmware artifact/digest, boot protocol, kernel image, command line, initramfs |
| memory | size, firmware map, reserved ranges, NUMA/topology if enabled |
| interrupts/time | local and I/O APICs, routing, MSI/MSI-X, clocks, timers |
| buses | ACPI table set/digests, PCI topology, configuration mechanism |
| console | early and normal console device, address/transport, terminal settings |
| storage/network | exact device models, transports, identifiers, queue/features, topology |
| entropy/power | entropy device policy, reset, shutdown, suspend support |
| persistence | image format/digest, writable-overlay policy, generated-state locations |

The manifest is the board-support input for both Linux-reference and Topal
kernel boots. A launch command without this data is not reproducible because
QEMU defaults and unversioned CPU models can change observable hardware.

## LK-X64-DISCOVERY-001 — Discovery and authority order

The boot path establishes trusted transport and memory bounds before parsing
firmware input. Firmware-table checksums, lengths, pointers, table references,
and arithmetic are validated before typed interpretation. ACPI supplies
platform topology and control descriptions; PCI enumeration supplies live
discoverable functions and resources; CPUID supplies processor capabilities;
SMBIOS supplies descriptive identity and narrowly reviewed quirks; the command
line supplies policy. Conflicts are resolved by a named owning contract, not by
last-writer-wins properties.

Board initialization is dependency ordered: early console and memory precede
general allocation; interrupt controllers and timers precede scheduling and
device interrupts; PCI host setup precedes virtio-pci discovery; block support
precedes the selected root filesystem. Those dependencies become explicit
construction inputs in the Topal design.

## LK-X64-VIRTIO-001 — Proposed initial virtual-device set

The minimal useful QEMU profile should prefer standardized, inspectable devices:

- one serial or virtio console for early and userspace interaction;
- virtio block for the root disk;
- virtio network for application and container tests;
- virtio random after an explicit entropy design; and
- PCI transport with MSI/MSI-X once interrupt setup permits it.

This is a proposal, not yet the qualified board. The VM pull request will pin
QEMU, machine version, firmware, device models, feature bits, queue sizes,
storage format, network topology, and launch command before implementation uses
the profile.

## Board-description relationship

ACPI, Device Tree, PCI configuration space, SMBIOS/DMI, command-line input, and
architecture models answer different questions:

| Input | Role |
| --- | --- |
| Topal architecture model | qualified legality and implementation facts |
| ACPI | runtime platform topology/configuration and power/control methods |
| PCI configuration | runtime discoverable bus/device resources |
| Device Tree | runtime declarative hardware topology on selected platforms |
| SMBIOS/DMI | descriptive machine/product information and selected quirks |
| command line/initramfs | boot policy and supplied initial userspace |

Board support composes these inputs by provenance and authority. It does not
flatten them into one untyped property map.

Device Tree source/DTB behavior and its relationship to firmware nodes are
mapped separately in [the Device Tree record](../common/device-tree.md).
