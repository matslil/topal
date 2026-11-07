# Bounded static BusyBox shell exit-command profile

Status: approved systems-feature design (`TK-DEC-204`).

## Purpose

The digest-pinned BusyBox shell already reaches one bounded read from a sealed
empty input source. This increment supplies the single command `exit\n` from a
sealed kernel-owned source, proves that the shell consumes it, and admits the
shell's immediate successful process exit. It is the first qualification that
executes shell command parsing without claiming an interactive or general
command-input service.

## Portable contract

The live initial process may replace the existing end-of-input result with one
complete publication of the five bytes `exit\n`. The transition requires the
same exact predecessor phase, executable continuation, active checked address
space, retained initial stack, and live child and ignored-quit dispositions as
the empty-input profile. It additionally requires a fresh sealed input owner
whose complete immutable content is exactly the approved command.

The provider validates the requested extent before access, copies all five
bytes into the checked writable destination, consumes the input owner, returns
five, and retains neither the user pointer nor a readable input copy. The next
accepted syscall is successful process exit. Terminal processing validates the
consumed input state and clears the process-visible bytes together with all
other process-owned storage before rollback. Failure preserves the predecessor
phase, input owner, destination, and every process resource.

The command is a policy value at the portable boundary. Native syscall numbers,
registers, virtual addresses, and instruction sequences remain provider
bindings. AArch64 and RISC-V can implement the same sealed-input transition
with their own syscall selector and calling convention; neither architecture
requires a command-stream primitive in Topal source.

## X86-64 binding

The qualified launch binds the transition to
`read(0, 0x5f8f60, 1023)`. It copies `65 78 69 74 0a`, returns five, and
advances the private startup phase. The digest-pinned BusyBox 1.37.0 shell then
issues `exit_group(0)` without another syscall. The clean fixture performs the
same read, verifies and erases the five published bytes, and exits with status
zero.

## Exclusions

This adds no interactive terminal, mutable or refillable input stream,
blocking read, partial read, input multiplexing, arbitrary shell command,
command output, external program execution, pathname lookup, filesystem,
process creation, job control, pipeline, container, or virtualization support.
