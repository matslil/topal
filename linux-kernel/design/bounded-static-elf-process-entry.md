# Bounded static ELF process entry

## Decision

The randomized static-process owner may be consumed by one architecture-
independent `enter process` transition. The transition owns the complete
privileged boundary: it validates the retained process, prepares a provider-
private execution context, activates the process address space, enters the
sealed ELF entry with the sealed initial stack, accepts only the selected
terminal Linux ABI result, restores the kernel address space and continuation,
and produces one affine completed-process owner.

The first x86-64 qualification payload is a clean-room probe at the already
validated ELF entry. It performs only `exit_group(0)`. This proves a real
ring-3 `IRETQ` entry and `SYSCALL` return through the static address space; it
does not claim that the retained metadata fixture is BusyBox or that the full
Linux syscall surface exists.

## Portable boundary

Source names the process owner, Linux process ABI identity, terminal result,
and policy. It does not name a page-table root, privilege selector, register,
MSR, entry address, stack address, syscall number, or continuation address.
Those are provider facts derived from the sealed process and target profile.

The abstraction also fits AArch64 and other targets: a provider may lower the
same transition with exception-level return, its native syscall exception,
translation-base switching, and target context record. The portable contract
is enter/terminal-return ownership, not x86 instruction vocabulary.

## X86-64 lowering

The pinned provider shall:

1. revalidate the randomized process, native entry and initial-stack facts;
2. install a private syscall entry and context record without exposing either;
3. retain the qualified kernel CR3 and kernel continuation stack;
4. activate the retained static CR3 and enter CPL3 at `0x004a6580` with the
   sealed stack top;
5. accept only native `exit_group(0)` from that context;
6. restore the kernel CR3, GS state, stack and continuation before returning;
7. prove the process bytes, mappings, stack and delivered `AT_RANDOM` remain
   unchanged; and
8. produce one completed-process owner.

A qualification-only inverse clears the private context and restores the
randomized-process owner. It does not re-enter user mode. This permits the
existing exact initial-random and construction rollback to remain executable.

## Rejected alternatives

- General assembly or raw register access would expose the provider boundary
  and make the source architecture-specific.
- Reusing the earlier constructed-probe dispatcher would couple this process
  to unrelated backing, entropy-service and teardown state.
- Treating successful address-space activation as process entry would not
  prove privilege transition or syscall return.
- Running redistributed BusyBox bytes in the gate would violate the current
  clean-room fixture and provenance boundary.

## Scope

This is a systems-profile refinement only. It adds no core-language syntax or
unchecked escape. It does not initialize TLS, support a nonterminal syscall,
execute BusyBox, provide a prompt, or establish container compatibility.
