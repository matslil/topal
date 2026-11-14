# Bounded static-process shell refill input

This record resolves `TK-DEC-220` by replacing the clean shell's single
combined command publication with two ordered affine refills. The shell
receives `echo topal\n`, publishes its output and a second `/ # ` prompt, then
receives `exit\n` through the same process buffer before successful teardown.
The pinned external BusyBox profile follows the same two-publication ownership
lifecycle but requests its refill directly because its prompt is not part of
the qualified serial contract.

## Decision

The bounded static-process entry may admit exactly two command-input
publications through one checked process buffer. The first shall publish exact
`echo topal\n`; on the clean profile it shall follow the qualified initial
prompt and its output shall be followed by exactly one second qualified
root-prompt write. Both profiles shall then request one refill that replaces
the consumed first command with exact `exit\n`. Each
publication shall require the live process, sealed executable continuation,
active checked address-space hierarchy, retained executable pathname and
stack, live child and ignored-quit dispositions, exact predecessor state,
complete prior interactive observations, and exclusive input ownership.

The provider shall validate the complete buffer and requested bound before
publication, publish only the exact command prefix, zero the unused sealed
tail, read back the complete published state, retain no user pointer, and
advance one private input-owner state. The second prompt shall use the same
checked-copy, private-staging, output, result, and erasure contract as the
initial prompt and publish a distinct completion fact. A duplicate, omitted,
reordered, wrong-buffer, wrong-bound, stale-owner, partial, combined, or
different command shall fail closed. Successful terminal handling shall
require the second refill live, validate exact `exit\n` plus zero tail, erase
the complete command buffer, consume the input owner, and clear the second-
prompt fact with all predecessor observations. The pinned external echo-
command profile shall require the same exact second refill and terminal
consumption with its optional prompt observations absent.

## X86-64 binding

The x86-64 realization binds both publications to native syscall 0 with
`fd=0`, the same checked anonymous-buffer address, requested count 1023, and
results 11 then 5. The intervening clean prompt binds to native syscall 1 with
`fd=2`, count and result four, and exact `/ # ` bytes. Fresh provider-private
states distinguish first publication, optional clean prompt, and second
publication. The clean path requires the second-prompt fact; the external path
requires that fact and all other optional clean observations absent before
admitting the same refill. The clean fixture also proves its first bytes
unchanged before replacement. The external application may parse or mutate
its process-owned input after the first publication, so its refill relies on
the affine consumed-owner state and overwrites the complete sealed prefix and
tail rather than treating prior user bytes as provider state.

## Architecture pressure test

Affine refill ownership, complete checked publication, replacement rather than
append aliasing, exact results, prompt ordering, zero tail, and terminal
consumption are portable Linux-personality policy. X86-64, AArch64, and RISC-V
may select different syscall encodings, calling conventions, user-copy
mechanisms, console transports, and private layouts. No selector, register,
address, UART port, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Keeping both commands in one read was rejected because it does not demonstrate
refill. Appending the second command without consuming the first owner was
rejected because it aliases stream state. Returning arbitrary host input was
rejected because it breaks reproducibility and grants unbounded authority. A
general stream, parser, terminal editor, or command loop was rejected as later
scope.

## Deliberate bounds

This increment proves two exact refills and a repeated prompt. It does not add
arbitrary input, partial reads, blocking or wakeup policy, general commands,
dynamic applications, filesystems, processes, containers, or virtualization.
