# Bounded static ELF initial x86-64 FS base

## Decision

The bounded static-process startup profile may extend its proven initial
program-break query and growth with one exact initial-thread-pointer install.
For the pinned x86-64 executable, the request is Linux
`arch_prctl(ARCH_SET_FS, 0x005f7440)`. The candidate lies inside the live
artifact-owned anonymous break reserve, so the provider may retain it as
private initial-thread state, install it for user return, and report success.

The clean-room qualification payload writes a known byte at the selected base,
performs the install, and reads the byte through x86-64 FS-relative addressing.
Terminal return restores the pre-entry physical FS base, clears the private
selected value, and then continues the already-proven heap-leaf revocation and
complete reserve erasure.

## Portable boundary

Source names an initial-thread-pointer startup profile, not `arch_prctl`, an
operation code, address, register, MSR, or instruction. The affine live process
owner and its readable mapping authority justify selection; the provider owns
ABI decoding, exact candidate derivation, physical installation, and return
convention.

The portable concept is a thread pointer attached to a live thread. X86-64
uses the FS base selected through its Linux ABI, AArch64 normally uses TPIDR_EL0,
RISC-V normally uses `tp`, and capability architectures may retain bounded
pointer authority rather than reconstructing it from an integer. This profile
therefore refines the existing target-qualified thread-pointer element without
making the x86 control protocol portable language vocabulary.

## X86-64 lowering

The pinned provider shall:

1. admit syscall 158 only after the exact query and growth have completed;
2. require operation `0x1002`, candidate `0x005f7440`, and an executable
   continuation inside the sealed text mapping;
3. prove the complete candidate byte lies in the live readable anonymous break
   mapping and retain the exact selection in private process context;
4. install the selection in `IA32_FS_BASE`, return zero through `SYSRETQ`, and
   permit a clean-room FS-relative byte read;
5. accept `exit_group(0)` only after successful installation;
6. restore the pre-entry FS base before heap teardown, clear the retained
   selection, and preserve the existing mapping/storage rollback; and
7. let the qualification inverse prove and clear install completion before
   restoring the randomized-process owner.

## Rejected alternatives

- Exposing `arch_prctl`, FS, an MSR, or a native selector to source would put
  architecture and Linux calling conventions at the wrong abstraction level.
- Accepting any address would exceed the sealed startup profile and weaken the
  existing readable-mapping authority check.
- Treating the TLS pointer as ELF payload would ignore that this executable
  selects storage in the newly admitted anonymous break mapping.
- Leaving the physical or private FS base live after terminal return would leak
  user-thread state into the kernel continuation and qualification inverse.

## Scope

This is a systems-profile refinement only and changes no core-language rule.
It installs one exact x86-64 initial-thread FS base after the first break growth.
It does not add `ARCH_GET_FS`, another `arch_prctl` operation, general TLS
allocation or initialization, multiple threads, context-switch restoration,
signals, cloning, another architecture implementation, redistributed BusyBox
bytes in qualification, a prompt, or general application compatibility.
