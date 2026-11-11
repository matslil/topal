# Stage 3 source-built shell and command transfers

This selected checkpoint exercises `TOPAL-SYSTEMS-USER-EXTENT-001` through
ordinary application arguments and compiled C code. The [native run](native-run.json)
passes 38 shell callbacks (299 syscalls), five command callbacks (65 syscalls),
and the preceding 176 file, 200 startup and 56 shared-runtime syscalls. The full
combined run therefore observes 796 syscalls across four images.

The [actual observations](guest/observations.json) cover:

- Twenty scalar shell output operations with refused complete input and
  unchanged serial output, plus successful input across page boundaries.
- Five signal actions: read-only child/quit installs and default INT/QUIT/TERM
  queries. Invalid input/output leaves protected state unchanged.
- Uname output, three cwd outputs, root chdir and four terminal misses.
  Pathname input crosses a page; cwd output ends immediately before an unmapped
  page despite a larger requested capacity. Terminal misses use an invalid
  unused argument and leave the original candidate unchanged.
- Four header-byte reads into the last mapped byte before an unmapped page.
  Refused output preserves position, staging, phase and other protected controls.
- Five command callbacks with empty reads, five refused output candidates,
  a one-byte read, an empty/refused retry after that prefix and final delivery.
  They exercise both sealed cursor retention and serial receive/queue retention.

Command collection is part of the source program. It copies the returned
prefix and suffix into the original application candidate and returns their
aggregate length to the enclosing witness. The observer records each actual
syscall result separately; it does not present the final suffix result as one
full-command syscall.

[Independent readback](independent-readback.json) reconstructs every expected
command byte and the complete 512-byte private input state at each transition.
For serial input it separately constructs received bytes, length, mismatch
classification, unread suffix and zeroed consumed bytes. For sealed input it
checks the exact cursor changes and unchanged remainder. It also retains the
independent complete stat/directory record reconstruction from the preceding
checkpoint.

Separate ELF parsing checks all 7200 linked source bytes in the executable,
read-only load, both syscall adapters, the resolved source bridge and all 68
generated call sites with disjoint scratch. It checks the actual read-only
signal data against complete expected actions. Native observations verify
original registers, actual credential vectors, permitted output changes only,
unrelated heap bytes and whole-process command/thread/heap retirement.

The C frame costs are 496 bytes for runtime, 168 for startup, 400 for file,
168 for shell and 104 for command functions, including conservative return
words. Only startup delegates, to one of file/shell/command; those alternatives
cannot call each other or the runtime entry. The maximum complete path plus
existing scratch/adapters/alignment/returns remains 1720 of 2048 bytes. The
kernel caller reserve and host resource policy are unchanged.

Seventeen exact comparisons preserve the preceding kernel and boot image.
All 7606 serial bytes match. The [six post-check commands](post-validation.json)
pass two build-only tests, including default-fixture byte preservation,
formatting, locked workspace/all-target compilation, source size and whitespace.
The native run takes 284.64 seconds within the unchanged 300-second guest
deadline. Native peak memory is 353239040 bytes with no limit/OOM/kill events
or validation swap. No full-workspace test execution is claimed.

An initial source-emitter delimiter error was corrected before this frozen
native run; the subsequent default-byte test and all final checks pass. A
readback attempted while the VM was still live found no terminal observation
file; it was repeated only after that same process completed. Neither event
is treated as an AI safety denial or as passing native evidence.

[Source identity](source-identity.json) freezes twelve selected executable/test
inputs. This is not a complete producer census or clean-checkout reconstruction.
Documentation and evidence were recorded after execution. [Artifact identities](artifacts.json),
first-party source and original logs are retained; executable payloads remain
ignored. The local Git attribute preserves log whitespace indexed by the
manifest. Reproduce through the [explicit native test](../../../../labs/qemu/x86_64/README.md#source-built-user-buffer-cases).

This clean-header witness does not reach header-poll negative cases.
Specialized signal/reset and vector-output cases, complete caller coverage,
full native ownership/mode/stability qualification, canonical production
binding and cold reconstruction remain required. Stage 3, initial Step 4 and
the full original kernel goal remain unfinished. See [scope](scope.json).
