# Bounded static-process shell root-directory condition

Status: approved systems-feature design (`TK-DEC-224`).

## Purpose

The bounded shell issues a successful root metadata request, but its silent
`test -d /` does not make the application's use of that metadata externally
observable. This increment replaces that command with exact
`test -d / && echo root-directory\n`. It proves that the application consumes
the complete metadata value, takes the successful shell-control-flow edge,
and publishes the selected result without granting a general conditional
language, pathname service, or output stream.

## Decision

The bounded static-process entry may admit exactly five ordered command
publications through one checked process buffer: `echo topal\n`, `cd /\n`,
`test -d / && echo root-directory\n`, `pwd\n`, and `exit\n`. Each refill shall
replace a consumed publication, zero and validate its sealed tail, retain no
user pointer, and advance one exclusive private input-owner state. The clean
profile shall publish exact `/ # ` before every command; the pinned external
BusyBox profile shall require those optional prompts absent.

Both applications shall issue the already qualified exact-root metadata
request and receive the already qualified complete immutable root-directory
value. The provider shall retain only its private successful-observation fact.
The application shall consume the metadata to decide that root is a directory
and, only on that successful edge, issue one standard-output write containing
exact `root-directory\n`. The provider shall validate the live process,
continuation, hierarchy, stack, dispositions, exact predecessor, input owner,
metadata observation, output selector, and complete readable 15-byte candidate
before staging, comparing, and publishing that output. It
shall return 15, erase all staging, and record one private conditional-output
fact. No user pointer or output byte shall be retained.

The following qualified `pwd` shall require the conditional-output fact and
preserve its profile-specific behavior. Terminal handling shall require the
fifth publication live, erase the complete command buffer, consume its owner,
and clear the directory generation, metadata observation, conditional-output,
and all command-time facts.

## X86-64 binding

The x86-64 realization retains native syscall 262 for the root metadata
request and binds the selected output to native syscall 1 on standard output.
The exact third input result is 33 and the conditional output result is 15;
the other input results remain 11, 5, 4, and 5. Provider-private phases and
facts distinguish metadata publication, application return, conditional
output publication, later `pwd`, final input, and teardown. The affine stack
remains `[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Successful metadata consumption, conditional application control flow,
checked output transfer, observation ordering, staging erasure, and terminal
cleanup are portable Linux-personality policy. X86-64, AArch64, and RISC-V may
select different syscall encodings, native metadata layouts, calling
conventions, mapping layouts, and transfer implementations. No native
selector, register, address, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Treating the metadata request alone as proof that the application evaluated
the result was rejected because it observes only the ABI boundary. Publishing
provider-owned confirmation text was rejected because it bypasses the shell's
control flow. Returning a partial metadata value, forcing a write after a
failed test, accepting arbitrary output, adding a general shell evaluator, or
inheriting host-root state was rejected as incorrect or broader later work.

## Deliberate bounds

This increment proves one externally observable successful conditional over
the already sealed root object. It does not add failed conditions, arbitrary
expressions, other paths, directory enumeration, file contents, mutable
filesystem objects, process creation, dynamic applications, containers, or
virtualization.
