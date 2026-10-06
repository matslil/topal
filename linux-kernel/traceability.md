# Linux-kernel project traceability

This file connects the approved project direction to its first artifacts and
future evidence. It does not duplicate the repository-wide language
traceability ledger.

| Project intent | Research/design identity | Current artifact | Required downstream evidence |
| --- | --- | --- | --- |
| implement only x86-64 initially | `LK-SCOPE-001` | [baseline](baseline.md), [approved boot-adapter contract](design/x86-boot-adapter.md), [generated adapter](step-4-x86-boot-adapter.md), [pinned-QEMU gate](step-4-toolchain-gate.md) | initial entry, exception, and kernel-owned-memory execution pass; native UAPI implementation remains |
| support the complete latest interface | `LK-SCOPE-002`, `LK-SCOPE-003` | [baseline](baseline.md), [UAPI map](knowledge/common/linux-uapi.md), [source inventory](inventory/coverage.md) | semantic dispositions and differential tests |
| use Topal elements instead of source assembly | `TK-ELEMENT-001`, `TK-ELEMENT-ENTRY-001`, `TK-ELEMENT-MACHINE-001` | [approved systems profile](../docs/systems-profile.md), [formal rules](../spec/systems-profile.md), [checked kernel root](kernel/arch/x86_64/toolchain-gate.t), [generated provider object](step-4-x86-provider-object.md), [linked artifact](step-4-linked-artifact.md), [generated boot adapter](step-4-x86-boot-adapter.md), [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | initial entry roots, machine primitives, and adapter transitions are typed, generated, inspected, and executed; general compiler publication remains |
| model hardware/concurrent observations | `TK-ELEMENT-OBSERVATION-001` | `TOPAL-SYSTEMS-OBSERVATION-001`, [Step 4 semantic report](step-4-semantic-foundation.md), [linked artifact](step-4-linked-artifact.md), [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | debug-break transition and context-preserving resume pass under pinned QEMU; other observations remain |
| use bounded kernel-owned bootstrap memory | `TK-ELEMENT-STORAGE-001`, `TK-DEC-008`, `TK-DEC-016` | `TOPAL-SYSTEMS-STORAGE-001`, [bounded-storage report](step-4-bootstrap-storage.md), [bootstrap-region contract](design/bootstrap-region.md), [checked kernel root](kernel/arch/x86_64/toolchain-gate.t), [linked artifact](step-4-linked-artifact.md), [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | allocation/access checking, byte-content model, x86 lowering, structural load/store inspection, and QEMU memory evidence pass |
| keep new elements portable in meaning | `TK-ELEMENT-*` | [cross-architecture pressure test](design/architecture-pressure-test.md), `TOPAL-SYSTEMS-QUALIFY-001`, [x86-64 provider plan](step-4-x86-provider-plan.md), [generated provider object](step-4-x86-provider-object.md), [linked artifact](step-4-linked-artifact.md), [pinned-QEMU gate](step-4-toolchain-gate.md) | common semantics remain separate from typed x86 objects and artifact publication; the initial x86 provider path has physical evidence |
| fit Linux responsibilities to Topal | `TK-KERNEL-*`, `TK-MAP-*` | [kernel architecture](design/topal-kernel-architecture.md), [mapping ledger](design/linux-to-topal-mapping.md) | module designs, implementations, and conformance evidence |
| map Linux driver frameworks | `LK-DRIVER-*` | [driver map](knowledge/common/driver-frameworks.md) | Topal protocol mappings and subsystem conformance ledgers |
| support DTS/DTB | `LK-DT-*` | [Device Tree map](knowledge/common/device-tree.md), [binding inventory](inventory/7.2.9/x86_64/devicetree-bindings.json) | parser/binding tests and selected platform boot |
| describe board support | `LK-X64-BOARD-001`, `LK-X64-BOARD-002`, `LK-X64-DISCOVERY-001` | [platform map](knowledge/x86_64/platform-and-board.md), [machine manifest](labs/qemu/x86_64/manifest.json) | [two-kernel runtime discovery evidence](labs/qemu/x86_64/results/verification.json) |
| run standard applications | `LK-SCOPE-005` | acceptance ladder | pinned rootfs, libc/application corpus, result records |
| run Docker and Podman | `LK-UAPI-CONTAINER-001` | [UAPI map](knowledge/common/linux-uapi.md) | Step 3 rootful smoke [evidence](labs/qemu/x86_64/results/verification.json); full rootful/rootless profiles remain |
| run VMs on the Topal kernel | `LK-UAPI-KVM-001` | [UAPI map](knowledge/common/linux-uapi.md) | QEMU TCG tests, KVM API tests, optional nested profile |

## Step 1 exit criteria

- Linux release and architecture scope are immutable and reproducible.
- “All public interfaces” has an explicit inclusion rule and status model.
- The native syscall table, installed UAPI headers, documented ABI entries,
  direct ioctl definitions, vDSO exports, and Device Tree bindings have
  reproducible counts, identities, locations, and aggregate digests.
- UAPI, kernel, driver, Device Tree, ABI, board, container, and virtualization
  research have stable identities, reviewed boundaries, and primary source
  families.
- The no-source-assembly direction has semantic design criteria and an open
  decision register rather than premature syntax.
- No protected Topal meaning or Linux-derived code has been introduced.
- Remaining semantic enumeration and runtime qualification are explicitly
  carried into later steps rather than hidden behind source-file counts.

## Step 2 proposal exit criteria

- The internal kernel architecture preserves Linux external contracts without
  mechanically translating Linux internal C design.
- Every Linux subsystem family from Step 1 has a stable Topal responsibility
  mapping and a repeatable future-baseline update procedure.
- Existing Topal foundations and mandatory kernel gaps are distinguished.
- Proposed new elements are limited to machine-irreducible semantics; ordinary
  kernel policies and subsystems remain libraries/modules.
- x86-64, AArch64, and RISC-V pressure-test the abstraction while only x86-64
  receives implementation scope.
- Protected Topal meaning remains unchanged until explicit human approval and
  authority-ordered propagation.
