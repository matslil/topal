# Knowledge catalog

The catalog is the coverage index for the initial research. `mapped` identifies
the design boundary and source families; `source-indexed` adds reproducible
breadth over a stated source set; `reviewed` means the human-readable contract
in the linked record has been examined. None means implemented or qualified.

| Stable ID | Subject | Scope | Status | Record |
| --- | --- | --- | --- | --- |
| `LK-BASELINE-001` | pinned upstream | Linux 7.2.9 | reviewed | [baseline](../baseline.md) |
| `LK-SCOPE-001` | implementation architecture | x86-64 native | reviewed | [baseline](../baseline.md) |
| `LK-SCOPE-002` | public-interface completeness | x86-64 native | reviewed | [baseline](../baseline.md) |
| `LK-UAPI-001` | UAPI inventory boundary | common/x86-64 | source-indexed | [UAPI](common/linux-uapi.md), [coverage](../inventory/coverage.md) |
| `LK-UAPI-SYSCALL-001` | native syscall protocol ledger | x86-64 | source-indexed | [UAPI](common/linux-uapi.md), [inventory](../inventory/7.2.9/x86_64/syscalls.json) |
| `LK-UAPI-PROCESS-001` | process/ELF/signal ABI family | x86-64 | mapped | [UAPI](common/linux-uapi.md) |
| `LK-UAPI-FD-001` | file descriptor and VFS ABI | common/x86-64 | mapped | [UAPI](common/linux-uapi.md) |
| `LK-UAPI-NET-001` | socket and Netlink ABI | common/x86-64 | mapped | [UAPI](common/linux-uapi.md) |
| `LK-UAPI-DEVICE-001` | ioctl and device ABI | common/x86-64 | mapped | [UAPI](common/linux-uapi.md) |
| `LK-UAPI-PSEUDOFS-001` | proc/sys/dev/cgroup surfaces | common/x86-64 | mapped | [UAPI](common/linux-uapi.md) |
| `LK-UAPI-CONTAINER-001` | container prerequisites | common/x86-64 | mapped | [UAPI](common/linux-uapi.md) |
| `LK-UAPI-KVM-001` | KVM host API | x86-64 | mapped | [UAPI](common/linux-uapi.md) |
| `LK-CORE-001` | kernel subsystem decomposition | common | mapped | [kernel design](common/kernel-design.md) |
| `LK-CORE-BOOT-001` | staged initialization | x86-64 | mapped | [kernel design](common/kernel-design.md) |
| `LK-CORE-MM-001` | memory management | common/x86-64 | mapped | [kernel design](common/kernel-design.md) |
| `LK-CORE-SCHED-001` | execution and scheduling | common/x86-64 | mapped | [kernel design](common/kernel-design.md) |
| `LK-CORE-VFS-001` | VFS and filesystem model | common | mapped | [kernel design](common/kernel-design.md) |
| `LK-DRIVER-CORE-001` | device/driver/bus/class model | common | reviewed | [drivers](common/driver-frameworks.md) |
| `LK-DRIVER-LIFECYCLE-001` | discovery, binding, removal, PM | common | reviewed | [drivers](common/driver-frameworks.md) |
| `LK-DRIVER-RESOURCE-001` | IRQ/MMIO/DMA/clock/resource services | common | reviewed | [drivers](common/driver-frameworks.md) |
| `LK-DRIVER-DEPENDENCY-001` | firmware nodes, links, deferred probe | common | reviewed | [drivers](common/driver-frameworks.md) |
| `LK-DRIVER-MANAGEMENT-001` | managed acquisition and teardown | common | reviewed | [drivers](common/driver-frameworks.md) |
| `LK-DRIVER-IRQ-DMA-001` | interrupt and DMA services | common | reviewed | [drivers](common/driver-frameworks.md) |
| `LK-DRIVER-PCI-VIRTIO-001` | PCI and virtio transport stack | common/x86-64 | reviewed | [drivers](common/driver-frameworks.md) |
| `LK-DRIVER-FUNCTIONAL-001` | TTY, block, and network frameworks | common | reviewed | [drivers](common/driver-frameworks.md) |
| `LK-DT-001` | DTS/DTB/runtime boundary | cross-architecture | source-indexed | [Device Tree](common/device-tree.md), [bindings](../inventory/7.2.9/x86_64/devicetree-bindings.json) |
| `LK-DT-002` | typed tree and reference model | cross-architecture | reviewed | [Device Tree](common/device-tree.md) |
| `LK-DT-003` | binding, population, lifecycle | cross-architecture | reviewed | [Device Tree](common/device-tree.md) |
| `LK-DT-004` | architecture relationship | cross-architecture | reviewed | [Device Tree](common/device-tree.md) |
| `LK-X64-ABI-001` | native processor ABI | x86-64 | reviewed | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-SYSCALL-001` | syscall register/error contract | x86-64 | reviewed | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-PROCESS-001` | ELF/process/TLS contract | x86-64 | reviewed | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-SIGNAL-001` | native signal/restart contract | x86-64 | reviewed | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-DEBUG-001` | ptrace/regset/core contract | x86-64 | reviewed | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-VDSO-001` | vDSO/vvar publication | x86-64 | source-indexed | [x86-64 ABI](x86_64/abi-and-entry.md), [inventory](../inventory/7.2.9/x86_64/vdso-symbols.json) |
| `LK-X64-BOOT-001` | boot and kernel entry | x86-64 | mapped | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-TRAP-001` | exception/interrupt/syscall entry | x86-64 | mapped | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-BOARD-001` | firmware and board discovery | x86-64 QEMU | reviewed | [platform](x86_64/platform-and-board.md) |
| `LK-X64-BOARD-002` | qualified-board manifest schema | x86-64 QEMU | reviewed | [platform](x86_64/platform-and-board.md) |
| `LK-X64-DISCOVERY-001` | discovery and authority order | x86-64 | reviewed | [platform](x86_64/platform-and-board.md) |
| `LK-X64-VIRTIO-001` | initial virtual devices | x86-64 QEMU | proposed | [platform](x86_64/platform-and-board.md) |
| `TK-ELEMENT-001` | kernel-element design rules | cross-architecture | proposed | [Topal elements](../design/topal-kernel-primitives.md) |

The broad UAPI boundary is source-indexed, but most semantic families remain
`mapped`. They advance only when their command/layout/state ledgers reconcile
the indexed source with the pinned reference VM and reviewed behavior.
