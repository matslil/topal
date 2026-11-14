# Bounded static ELF initial `readlinkat` fallback

## Decision

The bounded static process may extend its stack-limit startup profile with one
optional `/proc/self/exe` discovery attempt. The pinned executable requests
Linux `readlinkat(AT_FDCWD, 0x005bfdda, output, 4096)` immediately after reading
its initial stack limit.

The syscall remains outside the implemented static-process dispatch set and
therefore returns the established Linux-compatible `-ENOSYS` result. The
fallback selects only the syscall number and live startup state; it does not
inspect or validate arguments, read the path, access the output candidate,
retain state, or disturb any prior process state. The same context then
continues through its implemented terminal operation.

Source names an optional executable-name discovery probe, not a syscall
number, directory descriptor, path address, output pointer, buffer length, or
errno encoding. The portable element is the recoverable unavailable-operation
result at the Linux ABI boundary; each provider owns native lowering.

This systems-only refinement does not add procfs, VFS lookup, symbolic links,
path resolution, executable-name publication, BusyBox redistribution, or a
prompt.
