# Bounded external static-process context

## Decision

The x86-64 systems provider may execute the one digest-pinned external static
BusyBox image through the existing affine static-process boundary. The
qualification selects the exact `busybox true` argument vector and the already
approved bounded startup syscall sequence. External bytes remain an explicit
gate input and are neither committed nor redistributed.

The portable operation remains `enter process`: source observes a sealed Linux
process identity, one terminal result, and ownership transfer. Native register
names, SIMD controls, initial-stack addresses, syscall numbers, resident-header
variants, and translation status bits stay provider facts. Other architectures
may implement the same boundary with their native exception return, vector
context, calling convention, and translation maintenance.

## X86-64 entry context

The provider enters at the fully assembled initial stack pointer `0x007ffe90`,
not the earlier intermediate argument-vector position. Before CPL3 entry it:

1. verifies `CR0.EM` and `CR0.TS` are clear;
2. retains exact prior `CR4` and enables OSFXSR plus OSXMMEXCPT;
3. initializes x87, loads MXCSR `0x1f80`, and zeros XMM0 through XMM15; and
4. records whether the already validated resident header is the clean-room or
   pinned external form.

On terminal return it scrubs the same SIMD state, restores exact prior `CR4`,
and clears every private context slot. This is a target systems feature, not a
core-language assembly escape.

## Register and rollback integrity

Each native syscall saves the complete Linux x86-64 boundary register set.
Only `rax` carries the result; `rcx` and `r11` supply native `SYSRETQ`, and every
other general register regains its boundary value. The encompassing provider
call independently retains and restores System V callee-saved registers so an
arbitrary conforming executable cannot corrupt the kernel continuation.

BusyBox may use the zero prefix below the canonical initial-stack block and may
set hardware Accessed or Dirty bits on any admitted page. Before exposing the
completed owner, the provider zeros and verifies all 466 prefix qwords,
normalizes A/D bits across all 503 image leaves and the disjoint stack leaf,
clears thread and syscall state, and revalidates the exact sealed mappings. The
header-variant tag gates the two exact stack-local startup outputs; it does not
broaden pointer acceptance.

## Rejected alternatives

- Raw source assembly or register access would make portable process entry
  architecture-specific and weaken the typed provider boundary.
- Reusing the intermediate stack pointer would present auxiliary-vector words
  as `argc` and `argv` to a normal C runtime.
- Returning handler scratch registers or allowing user code to clobber provider
  callee-saved registers would violate the Linux or System V ABI.
- Preserving application stack residue or hardware A/D changes would make the
  qualification inverse depend on executable internals and break exact affine
  rollback.
- Accepting another external digest, argument vector, or startup observation
  would turn this qualification into an unreviewed general loader claim.

## Scope

This design executes one pinned BusyBox `true` path. It does not add a prompt,
filesystem, dynamic linker, signals, multiple processes, arbitrary ELF support,
containers, virtualization, or general application compatibility.
