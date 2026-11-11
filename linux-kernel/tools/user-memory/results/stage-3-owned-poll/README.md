# Stage 3 owned descriptor readiness

This systems-only refinement of `TK-DEC-283` and
`TOPAL-SYSTEMS-USER-EXTENT-001` makes bounded readiness depend on the live
description and requested event for either validated ELF-header form. The clean
form may retain direct reads or poll first. The external form keeps its terminal
probes. Repeated polling does not advance file position; input `revents` is
ignored, successful polling publishes `POLLIN`, and a successful header read
clears pending readiness. Refused transfers preserve readiness and position.
Descriptor ownership, one record, descriptor three, `POLLIN`, infinite timeout
and the existing command lifecycle remain required. This is not general poll.

The [formatted-producer native run](poll/native-run.json) passes four poll/read
callbacks: 32 polls, twenty header reads and twelve heap calls, plus 56 existing
shared-runtime calls across four images. The 120 observed source requests are
this cohort's checks, not a count of every syscall in the enclosing application.
Its [C program](poll-cases.c) exercises:

- Unmapped, supervisor, overflowing and partially mapped eight-byte inputs.
- Eight polls with complete readable records in read-only code storage and
  refused writable result fields, both before and after readiness is pending.
- Successful records across directory/page boundaries beyond page 34, a result
  field itself crossing a page, and a complete record ending at an unmapped page.
- Arbitrary incoming `revents`, repeated polls before consumption, refused
  header-read outputs with readiness retained, and successful one-byte reads
  into the final mapped byte. The source copies returned bytes into its caller's
  original candidate and restores its original heap contents.

[Native observations](poll/guest/observations.json) compare all selected
registration, header, stat-staging, file-state and command bytes before/after
every callback syscall. They allow only pending readiness on poll success and
the ordered byte-position transition on read success. They also check original
registers, credentials, stack shape, heap restoration and terminal ownership
retirement. The debugger observes; it does not assign guest state.

[Independent readback](independent-readback.json) parses all 8520 linked text
bytes in the read-only executable load, the syscall adapter, resolved source
bridge and four generated header-call sites. It independently reconstructs
each `pollfd` and readiness/position transition, verifies the actual read-only
records and recomputes the conservative 1648-of-2048-byte source stack bound.
Compared with the preceding signal checkpoint, fourteen component/predecessor
objects remain byte-identical. Only the provider object, linked kernel and
boot image change; within the provider, 162 function bodies are identical and
only the process-entry function changes.

The [combined source regression](regression/native-run.json) passes its existing
796 observed syscalls. [Separate regression readback](regression-readback.json)
repeats complete stat/directory reconstruction, command/queue transitions,
source-code inclusion and caller-stack checks against the new kernel.
The [unchanged BusyBox run](external/native-run.json) separately passes the
external-header path, exact shell output and first-image terminal retirement.
[Rebuild/resource inspection](rebuild-equivalence.json) independently verifies
the exact external SHA-256 and its unchanged bytes in the boot resource. The
pinned external digest gate remains in force; no third-party executable is
archived. These are selected regressions, not a full application corpus.

The [initial post-check](../stage-3-owned-poll-format-failure/README.md) remains
failed. Explicit rustfmt found an unformatted included-file function signature
that workspace formatting skipped. The correction wraps that signature across
three lines and changes no other byte in the retained pre-format source.
The three earlier native runs used that pre-format producer. A fresh focused
native run uses the formatted producer, and all seventeen objects/images match
each earlier build exactly. The earlier regressions therefore establish behavior
of the exact matching executable; they are not represented as runs from the
formatted source input. [All six final post-checks](post-validation.json) pass,
including two build-only tests, default-fixture preservation, both formatting
checks, locked workspace/all-target compilation, source size and whitespace.
A fresh build preserves the preceding source-transfer fixture image exactly.

The final native poll test takes 115.61 seconds; the combined regression takes
286.24 seconds, both within the unchanged 300-second guest deadline. BusyBox
takes 29.89 seconds within its unchanged 180-second deadline. Peak bounded use
across these retained successful commands is 1156730880 bytes, including the
earlier compilation. Every recorded limit/OOM/kill count and validation swap
value is zero. Host protections, VM filesystem/network boundaries and resource
policy remain unchanged. No AI safeguard denial occurred.

Risk remains high around kernel memory boundaries. Single-agent review checks
the design hierarchy, complete input/result admission, owner/lifetime gates,
readiness/position/error order, caller liveness and predecessor equality. Separate
native observations and byte/state readback supplement that review; no independent
human review or complete native security proof is claimed. [Source identity](source-identity.json)
freezes nineteen selected executable/test inputs and five authority inputs.
Documentation/evidence follow execution. This is not a full producer census,
clean-checkout run or committed production reconstruction. [The manifest](manifest.json)
indexes retained source, logs, observations and artifact digests without ELF
payloads. Reproduce with the [explicit command](../../../../labs/qemu/x86_64/README.md#source-built-header-poll-cases).

The targeted header-poll buffer gap is now covered through ordinary source
calls, with a separate unchanged external-profile regression. Complete caller/
user-access inventory, native ownership/mode/stability qualification, canonical
production binding and cold reconstruction remain required. General descriptor,
event, count and timeout polling also remains part of the full kernel goal.
Stage 3, initial Step 4 and the original kernel objective remain unfinished.
