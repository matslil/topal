# Step 1 — Linux design and interface map

## Outcome

The Linux 7.2.9 native x86-64 baseline now has a reproducible source inventory
and a reviewed design map for userspace ABI, kernel subsystems, driver
frameworks, board support, and Device Tree. This establishes what later design
and implementation work must preserve without treating Linux internal C
structure as the required Topal architecture.

Step 1 is a map and coverage-control mechanism. It does not claim that 385
syscall rows or 1,025 exported headers are themselves a complete semantic
specification. The complete-interface commitment remains open entry by entry
until the dispositions and evidence defined by `LK-SCOPE-004` exist.

## Reproducible evidence

- Baseline: Linux 7.2.9 archive with SHA-256
  `b4c5dfbe51a364a6c7f03869200f88c8e1f77403539005f14b7fc6bc91b8d8ba`.
- Architecture: native LP64 x86-64; i386 and x32 userspace ABIs excluded.
- Generator: `tools/inventory_linux.py`, with deterministic and negative unit
  coverage in `tools/test_inventory_linux.py`.
- Generated breadth: [coverage report](inventory/coverage.md) and immutable
  aggregate hashes in the inventory summary.
- Reviewed records: [knowledge catalog](knowledge/catalog.md).

## Design conclusions carried into Step 2

1. External compatibility is defined by protocols and observations, not by
   Linux function names, structures, or subsystem directory boundaries.
2. Native syscall entry, ELF loading, signals, vDSO, ptrace/regsets, and user
   memory transfer form one architecture ABI and must be designed together.
3. Driver support needs separate device, driver, bus, class, firmware-node,
   resource, dependency, and published-endpoint identities with explicit
   probe/removal and in-flight-operation protocols.
4. Board support composes provenance-specific inputs. ACPI, PCI configuration,
   Device Tree, SMBIOS, boot parameters, and the Topal architecture model are
   not interchangeable property maps.
5. DTS is compiled outside the kernel. Runtime support consumes validated DTB,
   applies bindings, translates references/resources, and populates devices.
6. Containers and VM hosting are high-level compatibility profiles over many
   UAPIs. Booting a distribution, running one container, or running QEMU with
   software emulation does not qualify every dependent interface.
7. Irreducible machine transitions require typed Topal elements whose meaning
   is pressure-tested against x86-64, AArch64, and RISC-V. Exact semantics are
   still protected design decisions, not outcomes of this research step.

## Deferred by design

- the reference kernel configuration and exact QEMU machine (Step 3);
- runtime probing and application/container/VM-host corpora (Steps 3–4);
- approval and formalization of new Topal language elements (Step 2);
- implementation dispositions and conformance evidence (Step 4 onward); and
- AArch64 implementation, which the revised project scope excludes while
  retaining AArch64 as abstraction-design evidence.

## Risk and review assessment

The change is research and deterministic tooling, with no Topal semantic or
runtime implementation change. The main risks are false completeness and
version drift. They are controlled by pinned source identity, exact generated
coverage boundaries, explicit extractor limits, stable record identities, and
the rule that extraction never promotes a record to semantically reviewed or
qualified status.

