# Device Tree support and board-description map

## LK-DT-001 — Source, binary, and runtime boundary

Device Tree Source (DTS) is an authoring format. `dtc` and related build tools
resolve includes, labels, and source syntax into a flattened Device Tree blob
(DTB). The kernel consumes the flattened tree; supporting DTS therefore means
interoperating with the standard DTS-to-DTB toolchain and correctly consuming
its DTB result. It does not require a DTS parser in the kernel.

The Linux 7.2.9 source boundary is:

| Concern | Pinned source location |
| --- | --- |
| compiler and FDT utilities | `scripts/dtc/` |
| kernel OF interfaces | `include/linux/of*.h` |
| flattened-tree parsing and unflattening | `drivers/of/fdt.c`, `drivers/of/base.c` |
| address/resource translation | `drivers/of/address.c` |
| interrupt translation | `drivers/of/irq.c` |
| platform-device population | `drivers/of/platform.c` |
| overlay/change-set behavior | `drivers/of/overlay.c`, `drivers/of/dynamic.c` |
| schema and legacy bindings | `Documentation/devicetree/bindings/` |

The generated [binding inventory](../../inventory/7.2.9/x86_64/devicetree-bindings.json)
indexes all 6,276 YAML and legacy text binding files in that source boundary.
These bindings are cross-architecture input; their existence does not imply
that all described hardware is present on the qualified x86-64 VM.

## LK-DT-002 — Typed tree and reference model

The wire input is an untrusted, bounded byte sequence with a header, reserve
map, structure block, and string block. Parsing must validate offsets, sizes,
alignment, token nesting, property bounds, string termination, and arithmetic
before creating typed nodes. A valid encoding can still violate a binding or
refer to an unsupported provider.

The semantic model retains:

- node identity, hierarchy, name, unit address, and enabled status;
- byte-valued properties plus binding-directed typed interpretations;
- `compatible` lists in preference order;
- phandles and references with provider-defined argument-cell counts;
- inherited `#address-cells` and `#size-cells`, `reg`, and recursive `ranges`
  or `dma-ranges` translation;
- interrupt parents, domains, specifier cells, and mapping;
- clocks, resets, power domains, IOMMUs, GPIOs, pin control, and other
  provider/consumer references; and
- `/chosen`, memory nodes, `/reserved-memory`, aliases, CPU topology, and the
  other platform-level conventions selected by an architecture profile.

Raw properties do not become ambient driver authority. A binding-specific
decoder converts a validated node into resources and dependency references;
the relevant bus, interrupt, memory, DMA, or provider service then grants
narrow capabilities to the bound driver.

## LK-DT-003 — Binding, population, and lifecycle

Device creation is a policy operation over a validated tree, not a side effect
of parsing bytes. Bus population walks only supported nodes, preserves parent
and dependency relationships, matches compatible identifiers, and defers
consumers whose providers are not ready. Disabled, malformed, unknown, or
unbound nodes retain Linux-compatible visibility and diagnostics without
inventing a partial device.

The initial read-only boot-tree profile requires:

1. standard DTB validation and immutable typed-tree construction;
2. address, DMA, and interrupt translation;
3. binding-directed resource decoding and compatible matching;
4. platform/bus population with dependency-aware deferred binding; and
5. `/sys/firmware/devicetree` exposure when the profile boots from a tree.

Runtime overlays and dynamic changes are part of the complete Linux baseline,
but form a separate conditional profile. They require transactional change
sets, reference and notifier ordering, device removal, rollback, and explicit
rules for outstanding capabilities. Merely accepting an overlay blob is not
conformance.

## LK-DT-004 — Architecture relationship

Device Tree, ACPI, discoverable buses, and the Topal architecture model have
different provenance and authority. The shared firmware-node view may let a
driver request equivalent properties from ACPI or Device Tree, but must not
erase format-specific validity or lifecycle rules. On the first x86-64 QEMU
profile, ACPI and PCI are expected to describe the board. DTB boot remains a
required interoperability path and is the principal cross-architecture design
pressure from AArch64 and RISC-V.

The architecture model proves which operations and representations a backend
may provide. The runtime tree describes one machine instance. A mismatch is
handled by a stated reject, degradation, or diagnostic rule; neither input
silently overwrites the other.

