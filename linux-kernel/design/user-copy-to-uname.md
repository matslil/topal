# Fault-contained `copy_to_user` and `uname` contract

This record resolves `TK-DEC-033` for the first pointer-bearing Linux syscall.
It refines the already approved `TOPAL-SYSTEMS-RECOVERY-001` boundary without
changing the Topal core language. The vocabulary, affine checks, model, and
lowering exist only when the systems feature and qualified kernel target are
selected.

## Semantic boundary

A `UserTransferScope P A T B I` is opaque affine authority derived from one
live `UserSyscallContext P A T B I`. It retains the exact processor, user
address space, thread, ABI personality, and syscall-entry identity. Opening the
scope temporarily suspends ordinary disposition of the syscall entry. Every
transfer consumes one scope and returns its uniquely refined successor; return
to user execution or termination requires reunion with that successor.

A user candidate remains an untrusted ABI machine word. It is never a Topal
reference and cannot be loaded or stored by ordinary source. `copy_to_user`
combines that candidate, a checked maximum extent, source-owned kernel bytes,
and an explicit progress policy. Its closed result is either:

- `Complete`, which proves that the complete extent was stored; or
- `Fault(copied-prefix-bytes)`, which reports only the prefix that the selected
  Linux contract permits the caller to observe as progress.

Both outcomes return the refined transfer scope. Integer overflow, a range
outside the target user family, wrong address-space or entry identity,
duplicate scope use, disposition with a live scope, and direct candidate
access fail before lowering or fail closed. A failed all-or-error syscall may
leave an unspecified prefix in user memory; the abstraction does not promise
transactional rollback that Linux does not provide.

The backend owns target access-enable state and binds only the generated copy
sites to generated recovery continuations. Source cannot observe or supply an
instruction address, exception-table entry, page-table entry, fault frame, or
target access-control bit. Unexpected kernel faults and faults outside the
scope remain fatal.

## First executable profile

The initial x86-64 profile extends the sealed probe with native Linux syscall
63 (`uname`) between `getpid` and `exit_group`. Source owns one deterministic
Linux `new_utsname` value with six 65-byte NUL-padded fields:

| field | initial value |
| --- | --- |
| `sysname` | `Linux` |
| `nodename` | `topal` |
| `release` | `7.2.9-topal` |
| `version` | `#1 Topal` |
| `machine` | `x86_64` |
| `domainname` | `(none)` |

The first call supplies a valid writable user buffer, receives result zero,
and verifies representative bytes from the encoded structure. The second call
starts in the same writable page but crosses into a deliberately unmapped
guard page. The generated copy site faults, the generated page-fault recovery
entry returns control to the transfer, and source maps incomplete transfer to
Linux `-EFAULT` (`-14`). The probe verifies that result before terminating
with `exit_group(0)`.

The recovery entry accepts only the active generated copy operation and its
exact expected page-fault class. It consumes the machine fault frame privately,
records prefix progress, and resumes the registered recovery continuation.
Any page fault without that active scope is fatal.

## Cross-architecture boundary

X86-64 may use ordinary supervisor stores with target-qualified SMAP state and
an exception-table-like generated recovery binding. AArch64 may require PAN or
UAO handling and an exception return; RISC-V may require SUM-related state and
supervisor-trap recovery. Those mechanisms are provider evidence. All targets
expose the same affine scope, untrusted candidate, bounded copy, prefix result,
and closed recovery/fatal distinction.

The common contract does not require faulting one byte at a time, a particular
copy instruction, or one rollback behavior. A provider may prevalidate facts
but must still meet the specified result and qualification obligations for
faults that can arise after validation.

## Rejected alternatives

- Treating a user integer as a reference would confer unchecked authority and
  make architecture access controls invisible.
- A catch-all source exception handler could convert kernel corruption into a
  user error and expose target frames or continuation addresses.
- A private test syscall would qualify neither the pinned Linux number nor its
  pointer-bearing ABI.
- Adding `write` in the same increment would prematurely couple user transfer
  to descriptor, console, partial-I/O, blocking, and interruption semantics.
- Returning only success or fault would lose Linux interfaces whose partial
  progress is observable.

## Deliberate exclusions

This increment qualifies kernel-to-user transfer only. It does not add
`copy_from_user`, arbitrary layouts or extents, dynamic mappings, demand
paging, concurrent mapping changes, multiple processes or threads, signals,
restart, preemption in user mode, SMAP enablement policy, a general UTS
namespace, ELF loading, file descriptors, or a complete syscall table. The
fixed mappings, guard page, UTS value, and probe remain qualification evidence.
