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

The controlling Linux source families are `drivers/base/{core,bus,dd,class,
platform,property,devres}.c`, `include/linux/device*.h`, and the relevant bus
and functional-subsystem directories. Embedding `struct device` is a Linux C
composition technique, not an external requirement.

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

## LK-DRIVER-DEPENDENCY-001 — Firmware nodes, links, and deferred probe

Linux firmware-node interfaces let common drivers consume properties and
references originating from ACPI or Device Tree. Device links express
supplier/consumer ordering. Deferred probe permits discovery before a required
provider is ready. The Topal design should retain the shared query where the
meaning is genuinely common, while carrying provenance and binding-specific
validity in the returned typed resource.

A missing provider, a provider not yet ready, an optional resource, and an
invalid reference are distinct outcomes. Dependency cycles and permanent
deferral require diagnostics. Publishing a user endpoint before every required
supplier and rollback action is installed would expose a half-probed device.

## LK-DRIVER-MANAGEMENT-001 — Managed acquisition and teardown

Linux devres attaches cleanup actions to device lifetime and probe failure.
Topal lexical/resource lifetimes can make many acquisitions explicit, but
device removal is asynchronous with respect to open descriptors, interrupts,
DMA, queued work, mappings, and userspace. The protocol therefore needs both
ownership and revocation:

1. prevent new operations and unpublish discoverability;
2. mask new device observations and stop queues;
3. cancel or drain work, interrupts, and DMA under stated ordering;
4. invalidate or preserve outstanding userspace objects according to their
   subsystem contract; and
5. release dependencies and storage exactly once.

Probe rollback uses the same acquisitions in reverse dependency order. It must
be correct at every failure point rather than relying on process exit.

## LK-DRIVER-IRQ-DMA-001 — Interrupt and transfer services

`kernel/irq/`, `kernel/dma/`, `drivers/iommu/`, and architecture providers
jointly implement services used by drivers. The Topal mapping separates:

- interrupt source identity, domain translation, vector allocation, routing,
  affinity, masking, acknowledgement/completion, sharing, and handler context;
- physical storage ownership from CPU virtual mappings, DMA addresses, and
  IOMMU translations;
- coherent versus streaming mappings, device DMA masks, direction, cache
  ownership, scatter/gather, pinning, bounce behavior, and completion; and
- CPU-memory ordering from MMIO, DMA, and device-protocol ordering.

These cannot be represented by an integer IRQ and a generic pointer without
losing authority, address-space identity, or lifecycle guarantees.

## LK-DRIVER-PCI-VIRTIO-001 — Initial bus stack

PCI (`drivers/pci/`) owns configuration-space enumeration, BAR/resource
assignment, capability walking, DMA constraints, interrupt modes, error/reset,
power, hotplug, and driver binding. Virtio (`drivers/virtio/` plus transport
and functional drivers) negotiates feature bits and queue layout over a
transport. Virtio is not itself a discovery bus that replaces PCI in the
proposed VM profile.

The Topal split is a PCI function capability, a virtio transport session, one
negotiated device protocol, typed virtqueues, and functional publication. No
feature-dependent queue or configuration access is legal before negotiation;
reset/removal invalidates the session and every derived queue capability.

## LK-DRIVER-FUNCTIONAL-001 — Userspace-facing frameworks

Functional subsystems translate device completion into stable UAPI. Initial
implementation pressure comes from:

- TTY/serial: termios state, line disciplines, controlling terminals, job
  control, hangup, polling, and console paths;
- block: request queues, partitions, flush/FUA/discard, page-cache/filesystem
  interaction, device ioctls, and removal/error completion; and
- network: net-device lifecycle, queues/NAPI, packet ownership, addressing,
  namespaces, routing/filtering integration, socket readiness, and Netlink.

The framework boundary is therefore not only a driver callback interface. It
is where resource lifetimes and concurrency become externally visible Linux
protocols.

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
[Device Tree usage model](https://docs.kernel.org/devicetree/usage-model.html)
and [the project Device Tree record](device-tree.md).
