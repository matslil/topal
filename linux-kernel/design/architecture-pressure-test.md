# Cross-architecture pressure test for kernel elements

This review uses x86-64, AArch64, and RISC-V to test the abstraction level of
the proposed systems profile. Only x86-64 receives implementation and
qualification in the current project scope. The comparison is semantic and
does not claim one uniform hardware mechanism.

## Method

An element is common only when all three families can implement the same
observable contract with target-qualified evidence. Different instruction
counts or entry frames are acceptable. Different preconditions, strengths, or
failure meanings require either an explicit parameter/profile or a
target-specific operation.

Architecture models describe availability and lowering facts. Runtime
capabilities identify the actual CPU, address space, interrupt domain, or
device on which an operation is authorized. Neither substitutes for the other.

## Entry and execution state

| Concern | x86-64 | AArch64 | RISC-V | Design result |
| --- | --- | --- | --- | --- |
| privilege | rings, descriptor/system state | exception levels | M/S/U privilege modes | opaque privilege capability; no portable numeric level |
| synchronous entry | IDT/system-call mechanisms with architecture frames | vector table and exception syndrome | trap vector and cause/status CSRs | special-entry kind plus target frame decoder |
| asynchronous entry | IDT/APIC delivery, NMI distinction | IRQ/FIQ/SError routes | interrupt causes through selected controller/firmware | event class and controller source remain distinct |
| entry stack | privilege/IST/task-state rules | selected stack pointer per level/configuration | software/firmware convention over trap state | backend-owned stack transition evidence |
| user return | fast syscall return or general interrupt return | exception return | supervisor return | typed disposition; backend proves legal return path |
| per-CPU base | segment-base facilities commonly used | thread-pointer/system-register choices | thread-pointer/CSR convention | semantic per-CPU location provider |
| extended state | XSAVE-family feature-dependent state | FP/SIMD/SVE/SME feature-dependent state | FP/vector extension state | opaque feature-qualified context component |
| suspended kernel context | stack position plus ABI/feature-qualified callee-saved state | `SP`, `x19`–`x30`, and qualified system state | `sp`, `ra`, `s0`–`s11`, and qualified system state | affine continuation owning a stack and provider-declared state policy |

Consequences:

- a portable handler never receives a C-like register structure;
- syscall and interrupt entries are distinct even when an architecture shares
  some physical vector machinery;
- target return validation is mandatory; and
- lazy/eager extended-state policy belongs to the scheduler provider, not the
  semantic user context.

The initial context-transfer slice keeps one processor and address space and
excludes extended state. This is a qualification boundary, not permission for
portable source to assume the x86-64 callee-saved set. AArch64 and RISC-V can
preserve the same suspend/resume ownership contract with different private
frames and state sets.

The cooperative handoff extension does not change that target boundary.
X86-64, AArch64, and RISC-V providers each preserve their qualified
call-boundary state while the portable operation exchanges only opaque running
and suspended identities. A provider-owned run queue is deliberately excluded:
runnable selection remains architecture-independent Topal policy.

## Translation and user access

| Concern | x86-64 | AArch64 | RISC-V | Design result |
| --- | --- | --- | --- | --- |
| translation root | control-register page-table root and tags | translation-table bases/control and ASIDs | `satp` modes and ASIDs | activate validated address space |
| page formats | multi-level x86 formats/features | granule/level/configuration variants | Sv39/Sv48/etc. schemes | target-qualified page-table layout provider |
| invalidation | address/context/global operations and shootdown | scoped TLBI plus completion ordering | `SFENCE.VMA` scopes and shootdown | semantic range/address-space/scope invalidation |
| kernel/user isolation | page permissions, SMEP/SMAP-related controls | permission and PAN-related controls | U/S permissions and SUM-related control | user-transfer capability and access scope |
| instruction visibility | coherent cases plus serialization rules | cache maintenance may be required | fence/cache-block facilities vary | explicit instruction-publication contract |

A common `flush TLB` intrinsic is rejected: targets differ in scope,
completion, broadcast, and required ordering. The common operation instead
names the mapping change, affected address-space/range, processors, and point
at which reuse becomes legal.

## Atomicity, ordering, and critical state

| Concern | x86-64 | AArch64 | RISC-V | Design result |
| --- | --- | --- | --- | --- |
| memory strength | comparatively strong normal-memory model | weakly ordered with acquire/release and barriers | RVWMO or qualified stronger profile | language atomic order cannot assume x86 strength |
| atomic implementation | locked RMW and compare/exchange families | LSE or exclusive-monitor loops | A extension or LR/SC | one semantic atomic operation, provider-selected implementation |
| interrupt mask | flags/controller state with NMI outside mask | DAIF and controller domains | status bits plus controller domains | affine scoped token naming exact domain |
| wait/wake | halt/monitor and interrupt mechanisms | WFI/WFE/event mechanisms | WFI and interrupt mechanisms | wait only under stated wake/progress contract |
| CPU/device ordering | architecture-specific fence and I/O rules | explicit memory/device types and barriers | fence predecessor/successor domains | separate CPU, MMIO, DMA, and maintenance relations |

The common atomic orders in `TK-ELEMENT-ATOMIC-001` describe language
relations, not opcodes. Progress classification is separate: an LL/SC loop may
meet a different progress bound from a single architectural RMW under
contention or interruption.

## Devices, DMA, and addressed I/O

| Concern | x86-64 | AArch64 | RISC-V | Design result |
| --- | --- | --- | --- | --- |
| device access | MMIO plus port I/O | primarily MMIO | primarily MMIO | MMIO common; port I/O target-specific |
| cache coherence | common PC profiles often coherent | platform/device dependent | platform/device dependent | DMA protocol always states coherence and maintenance |
| interrupt controller | local/I/O APIC and MSI family | GIC family commonly used | PLIC/APLIC/IMSIC or platform choices | controller provider plus common routed interrupt capability |
| firmware description | ACPI/PCI commonly primary | Device Tree and/or ACPI | Device Tree and/or firmware interfaces | provenance-retaining firmware-node adapters |
| firmware calls | UEFI/ACPI/platform mechanisms | PSCI/SMCCC and platform firmware | SBI and platform firmware | target/platform-specific service protocols |

Assuming coherent DMA because the first QEMU x86-64 machine supplies it would
make the common model incorrect. Every DMA transition therefore retains cache
ownership and maintenance requirements even when the selected provider proves
that no instruction is necessary.

## Boot and artifact pressure

The kernel payload, final boot image, initial CPU mode, firmware handoff, and
secondary-CPU startup are platform contracts rather than properties of the
Topal language:

- x86-64 may use the Linux boot protocol or UEFI and starts secondary CPUs
  through a platform-specific sequence;
- AArch64 commonly uses a firmware/boot protocol plus PSCI for CPU management;
  and
- RISC-V commonly uses firmware/SBI and a defined hart handoff.

The common artifact profile therefore supports special roots, placement
requirements, relocation policy, and qualified packaging adapters. It does not
standardize one entry symbol, firmware table, load address, or CPU-start
mechanism.

## Operations deliberately kept target-specific

- x86 port I/O, descriptor-table details, CPUID leaves, MSRs, and VMX/SVM;
- AArch64 named system-register and exception-level facilities without a
  shared semantic counterpart;
- RISC-V CSRs, SBI calls, and extension-specific controls without a shared
  semantic counterpart;
- cache operations whose line/visibility contract cannot be expressed by a
  qualified common maintenance operation; and
- virtualization controls until a separately reviewed common VM provider
  interface is demonstrated.

Target-specific does not mean unchecked. These operations still require sealed
capabilities, typed operands/results, context rules, effects, and provider
evidence. It means the language does not give an x86 concept a misleading
portable name.

## Rejected abstraction levels

| Rejected design | Reason |
| --- | --- |
| inline assembly or instruction-template element | exposes registers/clobbers, defeats semantic checking, and couples source to one backend |
| generic read/write control-register intrinsic | erases register-specific legality and state transitions |
| universal pointer over physical, virtual, MMIO, and DMA addresses | turns numeric coincidence into alias and authority |
| one `barrier` operation | cannot state which agents, observations, direction, scope, or completion are ordered |
| one interrupt-disabled Boolean | omits nesting, previous state, CPU identity, domains, and non-maskable sources |
| C11 atomics copied as an isolated library | atomics must integrate with Topal effects, invariants, external events, contexts, and race rejection |
| architecture model values visible to ordinary branching | would make optimization/target choice change semantic program behavior |
| use ordinary Topal task handlers as hardware entry | permits the wrong blocking, cancellation, stack, and scheduling assumptions |

## Qualification consequence

A new common element is accepted only after:

1. its semantics can be stated without an architecture name;
2. each comparison architecture has a plausible provider or a precisely
   stated unsupported condition;
3. stronger target behavior cannot leak into the portable guarantee;
4. zero-instruction and multi-instruction lowerings are observationally equal;
5. negative tests reject wrong context, authority, lifetime, scope, and target;
   and
6. x86-64 artifact and QEMU evidence proves the first implementation.
