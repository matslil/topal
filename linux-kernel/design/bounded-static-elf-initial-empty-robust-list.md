# Bounded static ELF initial empty robust list

## Decision

The bounded static process may extend its clear-child-TID registration with one
exact empty robust-list registration. The pinned executable requests Linux
`set_robust_list(0x005f76e0, 24)`. The complete head lies in the live anonymous
break reserve and initially contains a self-linked list pointer, futex offset
`-32`, and null pending pointer.

The provider retains the candidate without dereference during registration and
returns zero. On terminal exit, while the process address space remains active,
it copies and validates the complete empty head, clears private registration,
then performs the proven clear-child-TID, FS-base, and heap teardown. No robust
entry is walked and no futex is modified or woken.

Source names an initial-thread empty-robust-list profile, not a syscall,
address, layout, task field, or futex mechanism. The portable element is a
checked optional robust-list registration tied to one live thread; native ABI
layout stays provider-owned.

This systems-only refinement does not add nonempty lists, robust futex recovery,
multiple threads, cloning, futex wait/wake, signals, BusyBox redistribution, a
prompt, or general task lifecycle compatibility.
