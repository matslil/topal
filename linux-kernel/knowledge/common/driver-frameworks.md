# Linux driver-framework map

## LK-DRIVER-CORE-001 — Device, driver, bus, class, and firmware model

Linux driver core relates devices to buses, drivers, classes, firmware nodes,
power-management state, sysfs objects, and uevents. Bus-specific matching binds
a driver to a device and invokes lifecycle callbacks. Class membership supplies
a userspace-oriented view independent of physical topology.

The Topal mapping should use distinct identities for physical/logical devices,
driver implementations, bus membership, firmware descriptions, user-visible
class endpoints, and acquired resources. One object should not impersonate all
of these roles because Linux embeds a common C structure in subsystem-specific
structures. The Linux [driver-model overview](https://docs.kernel.org/driver-api/driver-model/overview.html)
and [device-driver description](https://docs.kernel.org/driver-api/driver-model/driver.html)
are primary orientation sources.

## LK-DRIVER-LIFECYCLE-001 — Discovery, matching, binding, and removal

The common lifecycle contains:

1. discover or describe a device through firmware, a bus, or explicit platform
   construction;
2. allocate an identity and enumerate address, interrupt, DMA, clock, reset,
   power, dependency, and configuration resources;
3. match a compatible driver;
4. probe while acquiring resources and allowing defer/failure;
5. publish functional and userspace-visible endpoints only after success;
6. suspend, resume, reset, quiesce, shut down, unbind, or remove under explicit
   dependency and in-flight-operation rules; and
7. release every acquired resource exactly once.

Topal resource lifetimes and protocols fit this structure better than implicit
cleanup lists, but hot removal, failed probes, asynchronous completions, and
external device disappearance require explicit cancellation and invalidation.

## LK-DRIVER-RESOURCE-001 — Shared driver services

The framework map must cover:

- MMIO and port-I/O resources, register layouts, regmap-like access, ordering,
  and device-specific side effects;
- interrupt domains, vectors, affinity, masking, threaded handlers, and shared
  interrupts;
- DMA masks, buffers, scatter/gather, cache ownership, IOMMU mappings, pinning,
  and completion;
- clocks, resets, regulators, pin control, GPIO, power domains, runtime power
  management, and device links;
- firmware loading, configuration data, error reporting, and hotplug events;
- device nodes, sysfs attributes, ioctl protocols, and subsystem publication;
  and
- concurrency between user operations, interrupts, work, reset, suspend, and
  removal.

Each service should grant a narrow capability with a lifetime and effect
identity. A driver should not receive ambient access to every address,
interrupt, DMA engine, or firmware property.

## Bus and functional framework families

The knowledge library will separately map at least:

| Family | Initial relevance |
| --- | --- |
| ACPI and firmware nodes | x86 board discovery and power/configuration |
| PCI/PCIe | QEMU device discovery, BARs, MSI/MSI-X, DMA |
| platform devices | non-discoverable and firmware-described devices |
| virtio over PCI | initial console/block/network/random devices |
| serial/TTY | boot console and interactive userspace |
| block and partitioning | root filesystem and container storage |
| network device/NAPI | sockets, container networking, QEMU host tests |
| input, graphics, sound, USB, SCSI, NVMe, HID, media, and others | complete UAPI target, later implementation increments |

Device Tree remains a required future data format and cross-architecture design
input, but it is not the primary firmware description for the initial x86 QEMU
board. DTS is compiled to a DTB outside the kernel; the kernel consumes the
validated flattened tree and bindings. See the Linux
[Device Tree usage model](https://docs.kernel.org/devicetree/usage-model.html).

