# Linux-kernel project traceability

This file connects the approved project direction to its first artifacts and
future evidence. It does not duplicate the repository-wide language
traceability ledger.

| Project intent | Research/design identity | Current artifact | Required downstream evidence |
| --- | --- | --- | --- |
| implement only x86-64 initially | `LK-SCOPE-001` | [baseline](baseline.md) | target profile, boot test, native UAPI inventory |
| support the complete latest interface | `LK-SCOPE-002`, `LK-SCOPE-003` | [baseline](baseline.md), [UAPI map](knowledge/common/linux-uapi.md) | generated 7.2.9 inventory, dispositions, differential tests |
| use Topal elements instead of source assembly | `TK-ELEMENT-001` | [primitive criteria](design/topal-kernel-primitives.md) | approved language design, spec rules, compiler tests, entry artifact inspection |
| keep new elements portable in meaning | `TK-ELEMENT-001` | cross-architecture pressure table | x86-64/AArch64/RISC-V design review |
| map Linux driver frameworks | `LK-DRIVER-*` | [driver map](knowledge/common/driver-frameworks.md) | subsystem records and Topal protocol mappings |
| describe board support | `LK-X64-BOARD-001` | [platform map](knowledge/x86_64/platform-and-board.md) | pinned QEMU machine manifest and runtime discovery tests |
| run standard applications | `LK-SCOPE-005` | acceptance ladder | pinned rootfs, libc/application corpus, result records |
| run Docker and Podman | `LK-UAPI-CONTAINER-001` | [UAPI map](knowledge/common/linux-uapi.md) | separate rootful/rootless profiles and OCI tests |
| run VMs on the Topal kernel | `LK-UAPI-KVM-001` | [UAPI map](knowledge/common/linux-uapi.md) | QEMU TCG tests, KVM API tests, optional nested profile |

## Foundation exit criteria

- Linux release and architecture scope are immutable and reproducible.
- “All public interfaces” has an explicit inclusion rule and status model.
- UAPI, kernel, driver, ABI, board, container, and virtualization research have
  stable identities and primary source families.
- The no-source-assembly direction has semantic design criteria and an open
  decision register rather than premature syntax.
- No protected Topal meaning or Linux-derived code has been introduced.

