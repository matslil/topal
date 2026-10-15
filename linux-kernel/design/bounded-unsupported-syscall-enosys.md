# Bounded unsupported-syscall `ENOSYS` fallback

This record resolves `TK-DEC-085` as a systems-profile refinement. It gives the
sole initial process Linux-compatible failure for syscall numbers outside its
implemented dispatch set, so userspace can perform ordinary capability probes
without being terminated.

## Decision

While the initial process return continuation remains executable, every syscall
number not selected by an implemented dispatch arm returns `-ENOSYS` through
the ordinary Linux syscall return path. The fallback does not inspect unused
arguments, mutate kernel or process state, copy user memory, or claim that the
requested interface exists.

The qualification program invokes one deliberately unsupported number, checks
the exact `-ENOSYS` result, and then continues through an implemented syscall.
This proves both recoverable failure and continued process usability. Each later
implemented syscall must replace the fallback with its own validated dispatch
arm and contract.

Returning success or a fabricated result was rejected because it would create
false compatibility. Terminating the process was rejected because Linux
applications and libraries use failed syscalls for feature discovery. A
provider-level catalogue of unsupported calls was rejected because syscall
selection is kernel policy, not machine-provider semantics.

The portable meaning is an unimplemented operation result at the Linux ABI
boundary. X86-64 uses its native negative errno return convention; AArch64 and
RISC-V providers would use their own ABI lowering for the same kernel policy.
This increment does not add a generic Topal error mechanism, syscall
introspection, audit events, seccomp, tracing, or any new implemented syscall.
