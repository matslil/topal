# Bounded static ELF initial `rseq` fallback

## Decision

The bounded static process may extend its empty robust-list startup profile with
one exact optional `rseq` registration attempt. The pinned executable requests
Linux `rseq(0x005f7300, 33, 0, 0x53053053)` immediately after robust-list
registration.

The syscall remains outside the implemented dispatch set and therefore returns
the established Linux-compatible `-ENOSYS` result. The fallback does not inspect
or validate any argument, access the candidate, retain state, or disturb the
registered exit word, robust list, FS base, or anonymous break reserve. The
same process then continues through its implemented terminal operation.

Source names an initial optional-feature-probe profile, not a syscall number,
address, structure size, signature, register assignment, or errno encoding.
The portable element is a recoverable unavailable-operation result at the
Linux ABI boundary; each architecture provider owns its native lowering.

This systems-only refinement does not implement restartable sequences, register
a userspace area, expose CPU identity, add scheduling or signal behavior,
redistribute BusyBox, or provide a prompt.
