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

The pinned source locations that control this record are:

| Boundary | Linux 7.2.9 source |
| --- | --- |
| native syscall numbers | `arch/x86/entry/syscalls/syscall_64.tbl` |
| register argument mapping | `arch/x86/include/asm/syscall_wrapper.h` |
| entry/return frame | `arch/x86/entry/entry_64.S`, `arch/x86/entry/syscall_64.c` |
| saved register layout | `arch/x86/include/asm/ptrace.h`, `arch/x86/include/uapi/asm/ptrace-abi.h` |
| ELF and auxiliary vectors | `arch/x86/include/asm/elf.h`, `arch/x86/include/uapi/asm/auxvec.h`, `include/uapi/linux/auxvec.h` |
| signal/ucontext layouts | `arch/x86/include/uapi/asm/sigcontext.h`, `arch/x86/include/uapi/asm/ucontext.h`, `arch/x86/kernel/signal_64.c` |
| ptrace register sets | `arch/x86/kernel/ptrace.c`, `arch/x86/kernel/fpu/regset.c` |
| vDSO exports | `arch/x86/entry/vdso/vdso64/vdso64.lds.S` |

## LK-X64-SYSCALL-001 — Native syscall register contract

For native 64-bit `SYSCALL`, the number is in `rax`; arguments one through six
are in `rdi`, `rsi`, `rdx`, `r10`, `r8`, and `r9`. The result is returned in
`rax`. Linux uses negated errno values for failures; raw values from `-4095`
through `-1` are the error range (`include/linux/err.h`). `rcx` and `r11` are
consumed by the architectural entry/return mechanism, and userspace must treat
them as clobbered. This boundary is not the System V function-call register
sequence for the fourth argument.

The syscall table contains 385 native `common` or `64` rows. Rows without an
entry implementation remain indexed as reserved/unimplemented numbers rather
than disappearing. The [generated syscall inventory](../../inventory/7.2.9/x86_64/syscalls.json)
is the number/name/source index; each row still needs argument, state,
interruption, restart, privilege, configuration, and subordinate-protocol
records before qualification.

Entry constructs a complete kernel-owned context before ordinary Topal handler
code runs. Return may use a fast path only when the restored instruction
pointer, flags, segments, and tracing state satisfy its architectural
preconditions; otherwise the fully general interrupt return path is required.
This distinction belongs to verified backend lowering, not source-visible
assembly or a handler-selected instruction.

## LK-X64-PROCESS-001 — ELF, initial process state, and TLS

The first profile accepts native little-endian `ELFCLASS64` objects with
`EM_X86_64`. Loading must validate program headers and ranges, map `PT_LOAD`
segments with Linux permissions and zero-fill behavior, honor `PT_INTERP` for
dynamic programs, implement PIE placement, and construct the initial stack
containing argument pointers, environment pointers, and the terminated
auxiliary-vector sequence. The selected page-size and randomization policies
are properties of the qualified machine/configuration and must be observed
differentially.

The auxiliary vector connects execution to credentials, page size, program
headers, entry address, random data, platform information, and the vDSO through
`AT_SYSINFO_EHDR`. Thread-local state is established and changed through the
native clone and `arch_prctl` contracts; `%fs`-base semantics must be preserved
without exposing a physical register as a general Topal language primitive.

## LK-X64-SIGNAL-001 — Signal frame and restart boundary

Signal delivery changes a running user context. It selects the normal or
alternate signal stack, validates and writes the native frame and extended
processor state, applies mask/disposition rules, enters the handler with the
native calling convention, and provides the restoration path through
`rt_sigreturn`. Restoration treats every user-supplied field as untrusted and
may require the general return path. Signal interruption also interacts with
syscall restart codes, remaining timeout values, partial completion, ptrace,
seccomp, and cancellation behavior.

The public frame is jointly defined by installed UAPI layouts and the pinned
signal implementation. It is not equivalent to exposing an internal kernel
trap-frame type. Extended state is feature- and format-qualified, so its
validation and size cannot be frozen from one development host.

## LK-X64-DEBUG-001 — Ptrace, regsets, and core state

Debuggers and core-dump readers observe register sets rather than Topal task
objects. Native general registers, floating-point/extended state, TLS bases,
debug registers, signal state, and process memory are exposed through the
Linux ptrace and `NT_*` regset contracts with their exact permission, stop,
short-buffer, and feature rules. A future internal context representation may
differ, but adapters must round-trip every writable field that Linux permits
and reject invalid restored states as Linux does.

## LK-X64-VDSO-001 — vDSO/vvar publication

Linux 7.2.9 declares 15 native x86-64 vDSO exports in the version script; some
are configuration-conditional. The [vDSO inventory](../../inventory/7.2.9/x86_64/vdso-symbols.json)
retains those conditions. Compatibility requires a valid mapped ELF image,
symbol versioning, auxiliary-vector discovery, associated vvar data and update
protocol, fallback syscall behavior, and per-symbol semantics. Publishing names
alone would not be compatible.

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
