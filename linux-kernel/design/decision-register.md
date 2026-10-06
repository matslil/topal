# Kernel design decision register

This register prevents implementation experiments from silently choosing new
Topal or kernel semantics.

| ID | Decision | Why it matters | Status |
| --- | --- | --- | --- |
| `TK-DEC-001` | exact QEMU x86 machine, firmware, and boot path | fixes board, image, and discovery contracts | resolved for initial x86-64 profile; [Step 3 manifest](../labs/qemu/x86_64/manifest.json) |
| `TK-DEC-002` | kernel artifact format and linker/publication model | current compiler publishes Linux process ELFs | resolved for the initial profile; [Linux boot-adapter contract](x86-boot-adapter.md), linked publisher, structural adapter, and [pinned-QEMU gate](../step-4-toolchain-gate.md) implemented |
| `TK-DEC-003` | typed special-entry functions and context frames | replaces handwritten entry assembly | approved; `TOPAL-SYSTEMS-ENTRY-*` |
| `TK-DEC-004` | privileged operation and architecture-package boundary | determines which new elements are portable or target-specific | approved; `TOPAL-SYSTEMS-MACHINE-001` |
| `TK-DEC-005` | physical/virtual/user/device/DMA address types | required for page tables, user access, MMIO, and DMA safety | approved; `TOPAL-SYSTEMS-ADDRESS-001`, `TOPAL-SYSTEMS-MAPPING-001` |
| `TK-DEC-006` | interrupt, preemption, and external nondeterminism model | current portable task semantics are insufficiently specific for kernel entry | approved; entry/observation/critical/context rules |
| `TK-DEC-007` | synchronization and memory-ordering source model | compiler synthesis alone may not express kernel/device algorithms | approved; atomic/order/critical rules |
| `TK-DEC-008` | kernel allocation, stack, and fatal-failure model | removes dependency on host syscalls and process termination | approved; storage/artifact/disposition rules |
| `TK-DEC-009` | safe user-memory transfer and recoverable faults | every pointer-bearing UAPI depends on it | approved; `TOPAL-SYSTEMS-RECOVERY-001` |
| `TK-DEC-010` | permitted non-Topal bootstrap or generated support | user requested Topal elements instead of source assembly | resolved: backend-generated typed support only; no source assembly |
| `TK-DEC-011` | kernel code license and third-party provenance policy | Linux and this repository use different licensing terms | open before implementation/import |
| `TK-DEC-012` | reference kernel configuration and conditional UAPI policy | required to reproduce absence and capability behavior | resolved for initial profile; [Linux 7.2.9 configuration](../labs/qemu/x86_64/reference-kernel.config) |
| `TK-DEC-013` | root filesystems and dynamic libc qualification corpus | application tests do not define the interface alone | resolved for initial corpus; [Step 3 report](../step-3-report.md) |
| `TK-DEC-014` | rootful/rootless Docker and Podman profiles | container requirements and security boundaries differ | open; container phase |
| `TK-DEC-015` | KVM and nested-virtualization acceptance profiles | guest, QEMU TCG host, KVM host, and nested host are distinct | open; virtualization phase |
| `TK-DEC-016` | bootstrap-region source vocabulary and checked byte access | the approved storage model defines allocation ownership but not source spelling or how a region proves physical read/write use | resolved for the initial slice; [bootstrap-region contract](bootstrap-region.md), implementation and QEMU evidence pending |

Resolving a row requires a written proposal, alternatives, cross-architecture
review where applicable, and the repository-mandated approval for protected
design meaning. A code change cannot mark a design row resolved by itself.
