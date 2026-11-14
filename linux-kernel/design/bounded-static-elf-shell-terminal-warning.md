# Bounded static-process shell terminal warning

This record resolves `TK-DEC-209` as the exact non-terminal warning emitted
after the bounded shell's three standard-description probes. It advances the
clean shell through application-produced standard-error output without
inventing terminal support or embedding a provider-side substitute message.

## Decision

The bounded static-process entry may admit exactly three ordered standard-error
writes after the completed standard-terminal-probe count and before later
startup lookup, banner output, or sealed command input. The user process shall
supply the complete byte sequence `sh: can't access tty; job control turned
off\n` in segments of four, forty, and one byte. Every write shall require the
live process, sealed executable continuation, active checked address-space
hierarchy, retained initial stack, live child and ignored-quit dispositions,
exact shell-input predecessor phase, all prior interactive observations, the
next expected segment, descriptor two, and one complete readable candidate.

The provider shall copy and validate a complete segment before output becomes
observable, publish only those copied bytes to the existing console-error
transport, return the segment length, retain no user pointer or payload, and
leave descriptor state and process phase unchanged. One provider-private count
shall advance only after the complete segment is published and shall admit each
segment at most once. Later startup work, sealed input, and terminal rollback
shall require count three. The clean fixture shall stage, issue, verify, and
erase every segment. The pinned external echo-command profile shall omit all
optional interactive observations and output.

## X86-64 binding

The x86-64 realization binds the operations to native syscall 1, descriptor 2,
lengths 4, 40, and 1, checked candidates in the retained initial stack, exact
results equal to each length, and the existing polled serial transport. The
provider stages the four- and one-byte segments in registers and the full
forty-byte segment in five registers before validating and publishing it. The
private count occupies the next byte of the already qualified observation
qword after the console, `/dev/tty`, and standard-terminal facts. The complete
clean observation ends with warning count three; the external echo state
remains the all-zero qword.

## Architecture pressure test

The output channel, ordered segment contents, complete-copy-before-output rule,
results, no-retention effect, and observation lifecycle are portable Linux-
personality policy. X86-64, AArch64, and RISC-V may select different syscall
encodings, register staging, calling conventions, and physical console-error
adapters. No selector, descriptor encoding, register, pointer, or instruction
becomes Topal source vocabulary.

## Alternatives rejected

Emitting one provider-owned warning was rejected because it would bypass the
application's three observable writes and weaken user-copy evidence. Combining
the segments was rejected because the pinned executable performs distinct
calls with distinct results. Publishing before complete validation or rereading
unvalidated bytes was rejected because it would expose partial or substituted
output. Treating the warning as terminal configuration was rejected because it
reports terminal absence rather than changing it. Activating interactive
external arguments here was rejected because the following startup lookup and
banner dependencies remain separate qualification boundaries.

## Deliberate bounds

This increment admits the exact clean-fixture warning and preserves the
external echo-command profile. It does not add general standard-error output,
formatting, terminal attributes, warning variants, startup lookup, banner or
prompt output, mutable input, processes, filesystems, containers, or
virtualization.
