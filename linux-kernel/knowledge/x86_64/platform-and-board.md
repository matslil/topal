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

