# Bounded static-process shell `MAILPATH` metadata miss

This record resolves `TK-DEC-210` as the exact startup metadata lookup that
follows the bounded shell's non-terminal warning. It advances the clean shell
to its banner boundary without inventing mailbox contents, a filesystem, or a
provider-side substitute for the application's lookup.

## Decision

The bounded static-process entry may admit exactly one metadata lookup after
the complete three-segment terminal warning and before banner output or sealed
command input. The user process shall issue the immutable absolute pathname
`/var/spool/mail/root` from the current-working-directory selector, supply a
complete writable native metadata candidate in its retained initial stack, and
select no lookup flags. The provider shall require the live process, sealed
executable continuation, active checked address-space hierarchy, retained
initial stack, live child and ignored-quit dispositions, exact shell-input
predecessor phase, and every prior interactive observation.

The provider shall report exact absence, shall not modify the candidate, shall
not allocate a descriptor or retain the pathname or candidate, and shall leave
process phase and descriptor state unchanged. One provider-private fact shall
publish only after every argument and the complete candidate extent have been
validated and shall admit the lookup at most once. Later startup work, sealed
input, and terminal rollback shall require that fact. The clean fixture shall
prefill, issue, verify, and erase the complete candidate. The pinned external
echo-command profile shall omit this optional interactive observation.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 262,
`AT_FDCWD=-100`, pathname address `0x004a6920`, a checked 144-byte native
`stat` candidate in the retained initial stack, flags zero, and result
`-ENOENT`. The private fact occupies the next byte of the already qualified
observation qword after the console, `/dev/tty`, standard-terminal, and warning
facts. The clean observation ends with that byte equal to one; the external
echo state remains the all-zero qword.

## Architecture pressure test

The absolute pathname, root-identity derivation, absence result, no-output
effect, complete-candidate validation, no-retention effect, and observation
lifecycle are portable Linux-personality policy. X86-64, AArch64, and RISC-V
may select different syscall encodings, native metadata layouts, calling
conventions, and address-space validators. No selector, structure layout,
register, pointer, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Skipping the lookup was rejected because it would bypass observable shell
startup behavior. Reporting success or fabricating mailbox metadata was
rejected because the bounded environment has no mailbox object. Reusing the
earlier configuration-file miss without a new call was rejected because it
would lose pathname, ordering, and candidate evidence. Adding a generic VFS or
mailbox service was rejected because this transition needs only one immutable
absence fact. Retaining a user pointer or native layout in portable source was
rejected because either would weaken isolation or portability.

## Deliberate bounds

This increment admits the exact clean-fixture `MAILPATH` lookup miss and
preserves the external echo-command profile. It does not add generic pathname
lookup, mailbox contents, filesystems, the shell banner, profile lookup, a
prompt, mutable input, processes, containers, or virtualization.
