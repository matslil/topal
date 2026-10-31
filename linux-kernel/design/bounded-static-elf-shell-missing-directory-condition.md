# Bounded static-process shell missing-directory condition

Status: approved systems-feature design (`TK-DEC-225`).

## Purpose

The bounded shell now exposes successful metadata-driven control flow for the
sealed root, but it does not demonstrate Linux-compatible negative pathname
lookup or the application's failure edge. This increment replaces that
condition with exact `test -d /missing || echo missing\n`. It proves that the
application consumes `-ENOENT`, takes the failed-test shell-control-flow edge,
and publishes the selected result without granting a general pathname service,
conditional language, or output stream.

## Decision

The bounded static-process entry may admit exactly five ordered command
publications through one checked process buffer: `echo topal\n`, `cd /\n`,
`test -d /missing || echo missing\n`, `pwd\n`, and `exit\n`. Each refill shall
replace a consumed publication, zero and validate its sealed tail, retain no
user pointer, and advance one exclusive private input-owner state. The clean
profile shall publish exact `/ # ` before every command; the pinned external
BusyBox profile shall require those optional prompts absent.

Both applications shall issue one metadata request for exact absolute pathname
`/missing\0`, relative to `AT_FDCWD`, with flags zero and a complete checked
writable native metadata candidate. The immutable sealed root contains no
entry named `missing`, so the provider shall return exact `-ENOENT`, leave the
entire candidate unchanged, retain no lookup pointer or bytes, and record only
one private missing-path observation. The application shall consume that
result to decide the directory test is false and, only on that failure edge,
issue one standard-output write containing exact `missing\n`.

The provider shall validate the live process, continuation, hierarchy, stack,
dispositions, exact predecessor, input owner, missing-path observation, output
selector, and complete readable eight-byte candidate before staging,
comparing, and publishing the output. It shall return eight, erase all
staging, and record one private failed-condition-output fact. No user pointer
or output byte shall be retained. The following qualified `pwd` shall require
that fact and preserve its profile-specific behavior. Terminal handling shall
require the fifth publication live, erase the complete command buffer, consume
its owner, and clear the directory generation, missing-path observation,
failed-condition output, and all command-time facts.

## X86-64 binding

The x86-64 realization binds the failed metadata request to native syscall 262
with `AT_FDCWD=-100`, exact `/missing\0`, a 144-byte writable `stat` candidate,
flags zero, and result `-ENOENT`. The exact third input result is 33 and the
selected output result is eight; the other input results remain 11, 5, 4, and
5. The output binds to native syscall 1 on standard output. Provider-private
facts distinguish lookup failure, application return, failed-condition output,
later `pwd`, final input, and teardown. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Negative pathname lookup, unchanged failed-call output memory, application
control flow, checked output transfer, observation ordering, staging erasure,
and terminal cleanup are portable Linux-personality policy. X86-64, AArch64,
and RISC-V may select different syscall encodings, native metadata layouts,
calling conventions, mapping layouts, errno encodings, and transfer
implementations. No native selector, register, address, or instruction becomes
Topal source vocabulary.

## Alternatives rejected

Returning `-ENOSYS` was rejected because it does not express pathname absence.
Reusing the earlier `/dev/tty` or mailbox miss without a call was rejected
because it loses application-visible ordering and candidate validation.
Publishing provider-owned confirmation, mutating the failed output candidate,
forcing a write after successful lookup, accepting arbitrary paths or output,
adding a general shell evaluator, or consulting host root was rejected as
incorrect or broader later work.

## Deliberate bounds

This increment proves one externally observable failed condition over one
absent child of the already sealed root. It does not add arbitrary conditions,
relative paths, traversal, successful non-root objects, directory enumeration,
file contents, mutable filesystem objects, process creation, dynamic
applications, containers, or virtualization.
