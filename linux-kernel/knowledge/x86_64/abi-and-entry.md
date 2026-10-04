# Native x86-64 ABI and kernel-entry map

## LK-X64-ABI-001 — Native processor and Linux userspace ABI

The native profile combines several separately versioned contracts:

- AMD64 instruction-set and system architecture;
- System V AMD64 processor ABI for ELF, data layout, calling conventions, TLS,
  unwind information, and process conventions;
- Linux x86-64 syscall entry/return convention and syscall table;
- Linux-native signal, ptrace, register-set, vDSO, and auxiliary-vector layouts;
  and
- Linux UAPI structures whose native layout depends on C type sizes or
  architecture headers.

Kernel-internal Topal calls need not use the System V ABI. Every userspace,
firmware, boot, debugger, or tool boundary must use its selected external ABI
through an explicit adapter. The primary processor-ABI source is the
[x86-64 psABI](https://gitlab.com/x86-psABIs/x86-64-ABI); Linux-specific facts
come from the pinned source and installed UAPI.

## LK-X64-BOOT-001 — Boot entry and image contract

The VM phase shall choose one exact boot path and image format. Candidate paths
include the Linux x86 boot protocol and a defined UEFI/PE entry. The kernel
artifact publisher must construct the required image rather than relying on an
ELF executable intended for a Linux process.

The selected contract records initial CPU mode, paging state, descriptor
tables, interrupt state, register inputs, memory map, command line, initramfs,
firmware tables, load address/alignment, relocations, secondary-CPU startup,
and ownership of temporary boot storage. The Linux
[x86 boot protocol](https://docs.kernel.org/arch/x86/boot.html) is the baseline
reference even if the project initially chooses a narrower supported entry.

## LK-X64-TRAP-001 — Exceptions, interrupts, syscalls, and return

The x86-64 implementation requires backend-owned entry lowering for events
that do not obey an ordinary function ABI. The contract includes:

- IDT and gate state, vector and error-code rules, privilege transitions, and
  interrupt-stack-table selection;
- complete saved/restored architectural context and debug/unwind representation;
- masking, nesting, preemption, non-maskable events, machine checks, and fault
  recursion;
- syscall entry through the qualified mechanism, user/kernel stack transition,
  argument capture, speculation/security state, restart, signal delivery, and
  safe return; and
- context-switch state including extended processor state selected by the
  qualified CPU feature profile.

Topal source shall not spell entry assembly, clobbers, or fixed registers. A
typed entry declaration must describe the semantic event and verified context;
the qualified backend owns the physical prologue, epilogue, and return
instruction.

## Privileged facility inventory

The design must account for, without directly exposing instruction spelling:

- CPU and feature discovery;
- descriptor tables and task/interrupt stack state;
- control, model-specific, debug, and extended-state registers;
- page-table roots, translation invalidation, protection keys, and cache
  maintenance;
- local APIC, I/O APIC, interrupt routing, timers, and interprocessor interrupts;
- interrupt enable/mask state, halt/wait, and ordering operations;
- port I/O as an x86-specific addressed-I/O facility; and
- VMX/SVM only in the later virtualization profile.

Each facility receives a target-qualified semantic operation with explicit
authority and state transition. Operations with no portable counterpart remain
x86-specific rather than receiving misleading generic names.

