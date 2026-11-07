# Recoverable unknown syscalls in static processes

Status: implemented and physically qualified under standing systems-only chat
authorization (`TK-DEC-250`); see the [report](../step-4-static-syscall-fallback.md).

The prepared static process uses the same kernel dispatch policy as the initial
process: a selector with no implemented arm returns native Linux ENOSYS through
the retained syscall return continuation. It does not interpret unused arguments,
copy user bytes, mutate resources, consume startup phase or terminate execution.
This applies before and after each static image replacement.

Implemented selectors retain their validated contracts. Unqualified argument
profiles in those arms remain outside their qualification; this change does not
invent results for them or advertise unimplemented interfaces as supported.
Adding an implemented arm replaces fallback selection for its number. Native
selector width and return encodings remain target-owned; source pointers,
assembly, syscall reflection and core-language semantics are unchanged.

Qualification shall invoke several absent native selectors, including negative,
large, upper-bit and unselected ABI-tag cases, with invalid unused pointer
values. User-memory canaries and non-result register observations shall survive;
later implemented identity/file/signal/namespace queries, exec transitions and
terminal reclamation shall still pass. Independent Rust/Python encoders shall
bind the physically executed clean witness. Existing standard applications and
interrupt regressions shall run on the combined kernel.

Risk is high because dispatch mistakes can select an implementation incorrectly
or strand a live user context. Review follows complete implemented-arm
precedence, native selector normalization, exact negative errno, ordinary return
and saved-register clearing, unchanged state, exec lifetime and full rollback.
AArch64 and RISC-V must use their own native selector and continuation adapters.

Primary evidence is pinned Linux `kernel/sys_ni.c` (`sys_ni_syscall`) and the
native x86 syscall dispatch in `arch/x86/entry/syscall_64.c`. See the
[initial-process policy](bounded-unsupported-syscall-enosys.md) and
[baseline](../baseline.md). Broader Linux compatibility remains open.
