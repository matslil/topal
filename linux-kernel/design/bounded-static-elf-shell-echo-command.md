# Bounded static BusyBox shell echo-command profile

Status: approved systems-feature design (`TK-DEC-205`).

## Purpose

The digest-pinned BusyBox shell already parses one sealed `exit` command. This
increment supplies the complete command sequence `echo topal\nexit\n`, proves
that the shell executes its built-in `echo`, publishes the exact `topal\n`
result through standard output, and then accepts its successful exit. It adds
observable shell-command execution without claiming an interactive prompt,
general input, or external-program service.

## Portable contract

The live initial process may replace the existing five-byte command with one
complete publication of the sixteen bytes `echo topal\nexit\n`. The transition
requires the same predecessor phase, active checked address space, retained
initial stack, live signal dispositions, and fresh sealed input owner as the
exit-command profile.

The provider validates the entire read request before access, copies all
sixteen bytes into the checked writable destination, consumes the input owner,
and returns sixteen. The only admitted nonterminal successor is a write of the
six bytes `topal\n` to standard output from a completely checked readable
application extent. The provider stages and validates all output bytes before
publishing them, returns six, advances the process phase, and retains neither
the user pointer nor a readable output copy. The next admitted syscall is
`exit_group(0)`.

Terminal processing requires the completed output phase, validates the
consumed input owner, erases and proves erasure of all sixteen process-visible
command bytes, clears the retained dispositions, and completes the existing
rollback. Failure before publication preserves the predecessor phase, input
owner, destination, and process resources; failure after successful command
publication cannot retroactively withdraw bytes already observed at standard
output.

The command and output are semantic policy values at the portable boundary.
Native syscall numbers, registers, virtual addresses, serial transport, and
instruction sequences remain provider bindings. AArch64 and RISC-V can
implement the same checked read/write/exit transitions with their own calling
conventions, so this profile adds no architecture-specific Topal operation.

## X86-64 binding

The qualified launch binds the input transition to
`read(0, 0x5f8f60, 1023)`, copies the sixteen command bytes, and returns
sixteen. The pinned BusyBox 1.37.0 shell then issues the exact standard-output
write of `topal\n` from its writable data area before `exit_group(0)`. The clean
fixture performs the same read, verifies the command, issues and verifies the
same output write, erases the command bytes, proves the cleared extent, and
exits successfully.

## Exclusions

This adds no interactive terminal or prompt, refillable input stream, partial
I/O, arbitrary command or output, pathname lookup, executable loading,
filesystem, process creation, job control, pipeline, container, or
virtualization support.
