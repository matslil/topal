# Initial boot processor inventory

`TK-DEC-290` is a systems-only prerequisite for all-root/all-CPU protection,
adopted under the standing systems authorization. It adds firmware discovery
to the private inactive-table boot binding without changing source syntax or
core semantics. It does not start, reset, park or otherwise change a processor.

External debugger threads are observations, not facts the kernel can use to
establish its processor domain. The native provider must discover the complete
initial processor inventory advertised by its supported boot firmware and bind
the actual bootstrap processor. Discovery is needed before a later protocol
can establish participation and exclusion for every relevant processor.

The initial x86 implementation follows the qualified legacy ACPI discovery
path: EBDA first, then the BIOS search window, on sixteen-byte boundaries. It
validates RSDP checksums and supported revision, selects XSDT when advertised
or RSDT for the original format, validates the complete root table, and follows
its bounded entry list to exactly one complete valid MADT. Other referenced
table headers are retained to bind the discovery route; this is not AML or
general ACPI-service support.

SDT reads require wholly retained firmware ranges within the initial physical
window, excluding allocator-eligible RAM and conflicting firmware claims.
Limits are sixty-four root entries, 4,096 MADT bytes, sixteen relevant processor
records and 8,192 retained bytes. Every relevant retained byte is compared
before discard. Unsupported geometry or firmware record kinds fail closed.
Decoding uses the retained snapshot after validating its length and checksums;
separately copied headers must agree. This establishes internal snapshot
consistency, not atomic firmware capture or continuous firmware exclusion.
These are explicit initial provider limits, not portable language constants.

Local APIC and Local x2APIC processor records retain type, firmware UID, native
controller identity and enabled/online-capable flags. Unusable records are
ignored as required by ACPI. Relevant UIDs/controller identities are unique;
the actual BSP must match exactly one enabled participant. The initial xAPIC
provider checks its existing mapping's UC cache index before reading the local
controller identity and compares that identity with CPUID's initial identity.
Known non-processor MADT records are walked with exact supported lengths;
other record types are refused rather than treated as evidence of absence.

The private journal and byte copy are observations and retained validation
state. Source receives no table, pointer, register or processor-control API.
Independent replay reconstructs discovery and participant records from actual
captured firmware bytes, with corrected-checksum/rehash counterexamples kept
separate from native execution. Existing ordinary workloads and predecessor
profiles remain independent regression obligations.

Arm providers may obtain processor identities from ACPI GICC or Device Tree CPU
nodes; RISC-V providers may use firmware hart descriptions. Their discovery,
enablement and participation protocols require their own qualification. This
design deliberately separates firmware inventory from a live execution-domain
resource. Runtime AML discovery, hotplug, AP startup/parking, asynchronous entry,
all-CPU lifetime and protected activation remain unfinished obligations.

Primary format source: [ACPI 6.6, sections 5.2.5–5.2.8 and 5.2.12](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html),
including RSDP/root discovery, MADT and Local APIC/x2APIC structures. The current
serialized process pair reuses one process hierarchy while swapping USER
leaves/continuations; it does not clone a whole kernel hierarchy.
