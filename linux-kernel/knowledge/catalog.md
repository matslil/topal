# Knowledge catalog

The catalog is the coverage index for the initial research. `Mapped` means that
the design area and authoritative source families are identified. It does not
mean the complete generated interface inventory or an implementation exists.

| Stable ID | Subject | Scope | Status | Record |
| --- | --- | --- | --- | --- |
| `LK-BASELINE-001` | pinned upstream | Linux 7.2.9 | reviewed | [baseline](../baseline.md) |
| `LK-SCOPE-001` | implementation architecture | x86-64 native | reviewed | [baseline](../baseline.md) |
| `LK-SCOPE-002` | public-interface completeness | x86-64 native | reviewed | [baseline](../baseline.md) |
| `LK-UAPI-001` | UAPI inventory boundary | common/x86-64 | mapped | [UAPI](common/linux-uapi.md) |
| `LK-UAPI-SYSCALL-001` | native syscall ABI | x86-64 | mapped | [UAPI](common/linux-uapi.md) |
| `LK-UAPI-PROCESS-001` | process/ELF/signal ABI | x86-64 | mapped | [UAPI](common/linux-uapi.md) |
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
| `LK-DRIVER-CORE-001` | device/driver/bus/class model | common | mapped | [drivers](common/driver-frameworks.md) |
| `LK-DRIVER-LIFECYCLE-001` | discovery, binding, removal, PM | common | mapped | [drivers](common/driver-frameworks.md) |
| `LK-DRIVER-RESOURCE-001` | IRQ/MMIO/DMA/clock/resource services | common | mapped | [drivers](common/driver-frameworks.md) |
| `LK-X64-ABI-001` | native processor ABI | x86-64 | mapped | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-BOOT-001` | boot and kernel entry | x86-64 | mapped | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-TRAP-001` | exception/interrupt/syscall entry | x86-64 | mapped | [x86-64 ABI](x86_64/abi-and-entry.md) |
| `LK-X64-BOARD-001` | firmware and board discovery | x86-64 QEMU | mapped | [platform](x86_64/platform-and-board.md) |
| `LK-X64-VIRTIO-001` | initial virtual devices | x86-64 QEMU | proposed | [platform](x86_64/platform-and-board.md) |
| `TK-ELEMENT-001` | kernel-element design rules | cross-architecture | proposed | [Topal elements](../design/topal-kernel-primitives.md) |

The next knowledge PR must add generated coverage counts and exact source paths
for every `LK-UAPI-*` family before any family can advance from `mapped` to
`inventoried`.

