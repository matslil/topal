# Bounded static ELF initial clear-child-TID word

## Decision

The bounded static-process startup profile may extend the proven x86-64 thread-
pointer install with one exact initial-thread exit-word registration. The pinned
executable requests Linux `set_tid_address(0x005f7710)`. The complete four-byte
candidate lies in the live artifact-owned anonymous break reserve. The provider
retains it without dereference and returns sealed TID one.

On terminal `exit_group(0)`, the provider writes zero to the retained word while
the process address space remains active, verifies the result, clears the
private candidate, and only then restores kernel state and revokes the heap.
The clean-room payload initializes the word to one and verifies the immediate
TID result; structural and terminal-state evidence prove the exit clear.

## Portable boundary

Source names one initial-thread exit-word profile, not a syscall, address,
register, task structure, or futex operation. The portable element is an
optional writable user word tied affinely to one live thread. Providers map
their native ABI to that lifecycle; no architecture-specific representation
enters source.

## Rejected alternatives and scope

Returning one without retaining and clearing the word would violate Linux exit
semantics. Adding futex wakeup, cloning, a task table, or robust-list handling
would introduce policy for capabilities that cannot yet be exercised. This
systems-only increment admits one exact registration and terminal clear. It
does not add another candidate, a futex waiter or wake, robust lists, multiple
threads, cloning, signals, BusyBox redistribution, a prompt, or general task
lifecycle compatibility.
