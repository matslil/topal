# Kernel design decision register

This register prevents implementation experiments from silently choosing new
Topal or kernel semantics.

| ID | Decision | Why it matters | Status |
| --- | --- | --- | --- |
| `TK-DEC-001` | exact QEMU x86 machine, firmware, and boot path | fixes board, image, and discovery contracts | open; VM phase |
| `TK-DEC-002` | kernel artifact format and linker/publication model | current compiler publishes Linux process ELFs | proposed semantic profile; boot container remains `TK-DEC-001` |
| `TK-DEC-003` | typed special-entry functions and context frames | replaces handwritten entry assembly | proposed in `TK-ELEMENT-ENTRY-001`; approval required |
| `TK-DEC-004` | privileged operation and architecture-package boundary | determines which new elements are portable or target-specific | proposed in `TK-ELEMENT-MACHINE-001`; approval required |
| `TK-DEC-005` | physical/virtual/user/device/DMA address types | required for page tables, user access, MMIO, and DMA safety | proposed in `TK-ELEMENT-ADDRESS-001`; approval required |
| `TK-DEC-006` | interrupt, preemption, and external nondeterminism model | current portable task semantics are insufficiently specific for kernel entry | proposed through entry/observation/critical/context elements; approval required |
| `TK-DEC-007` | synchronization and memory-ordering source model | compiler synthesis alone may not express kernel/device algorithms | proposed in atomic/fence elements; approval required |
| `TK-DEC-008` | kernel allocation, stack, and fatal-failure model | removes dependency on host syscalls and process termination | proposed in storage/artifact/fatal elements; approval required |
| `TK-DEC-009` | safe user-memory transfer and recoverable faults | every pointer-bearing UAPI depends on it | proposed in user-access/fault elements; approval required |
| `TK-DEC-010` | permitted non-Topal bootstrap or generated support | user requested Topal elements instead of source assembly | proposed: backend-generated machine support only; approval required |
| `TK-DEC-011` | kernel code license and third-party provenance policy | Linux and this repository use different licensing terms | open before implementation/import |
| `TK-DEC-012` | reference kernel configuration and conditional UAPI policy | required to reproduce absence and capability behavior | open; VM phase |
| `TK-DEC-013` | root filesystems and dynamic libc qualification corpus | application tests do not define the interface alone | open; VM phase |
| `TK-DEC-014` | rootful/rootless Docker and Podman profiles | container requirements and security boundaries differ | open; container phase |
| `TK-DEC-015` | KVM and nested-virtualization acceptance profiles | guest, QEMU TCG host, KVM host, and nested host are distinct | open; virtualization phase |

Resolving a row requires a written proposal, alternatives, cross-architecture
review where applicable, and the repository-mandated approval for protected
design meaning. A code change cannot mark a design row resolved by itself.
