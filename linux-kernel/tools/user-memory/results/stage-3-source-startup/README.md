# Stage 3 source-built startup transfers

This checkpoint exercises `TOPAL-SYSTEMS-USER-EXTENT-001` through ordinary
application arguments. Two C objects and normal backend ABI adapters are linked
before a fresh first-party witness is constructed. No finished application is
patched, and pinned third-party applications are unchanged.

The [native run](native-run.json) passes four initial/replacement image sequences,
twenty startup callbacks and 200 startup syscalls, plus four shared-runtime
callbacks and 56 syscalls. [Actual observations](guest/observations.json) cover:

- `prlimit64` output crossing a directory, and random/name output crossing a
  page beyond the predecessor's 34-page heap limit;
- unmapped, wrapping, read-only, supervisor and partially mapped output
  candidates, with no mutation of the valid prefix on refusal;
- random zero-count behavior and invalid flag/count error precedence;
- complete configuration pathname admission across a page, refused input
  candidates, and `ENOENT` without accessing an invalid unused stat destination;
- preserved caller registers, actual initial/exec credential vectors, original
  heap extent and contents, and terminal command/thread/heap retirement.

The first image uses the sealed configuration fallback and advances its startup
phase. After exec, an active namespace owner handles the same lookup before that
fallback and leaves its phase unchanged. The observer checks the selected route,
the phase, and every syscall result separately. It reads guest state without
assigning registers, memory, mappings or ownership facts.

[Independent readback](independent-readback.json) parses ELF loads/sections and
checks all 2765 linked code bytes in the final executable read-only load, both
syscall adapters, the startup bridge and its resolved C call, and five generated
call sites with disjoint startup scratch. Static C frame sizes plus adapters,
returns and the 1024-byte scratch reserve total a conservative 1832 bytes within
the unchanged 2048-byte fixture bound. This is user stack accounting, separate
from the kernel caller reserve.

The same readback compares seventeen objects/images exactly against the preceding
source-fixture checkpoint, including the full kernel ELF and boot image. All
7606 serial bytes also match. The [post-check record](post-validation.json) covers
the two build-only tests, including exact default-generator byte preservation,
formatting, locked workspace/all-target checking, source size and whitespace.
No full-workspace test execution or clean-checkout reconstruction is claimed.

Selected native peak memory was 348504064 bytes, with no limit/OOM/kill events
or validation swap under the existing wrapper and host reserve.

[Earlier failures](earlier-failures) remain failed evidence. The first helper
overlapped still-live startup-vector storage; actual source-call stack addresses
and compiler stores explain the later credential invariant halt. The corrected
fixture reserves disjoint scratch and checks credential preservation. Two later
observer failures assumed the initial credential address and sealed lookup phase
after exec. Correcting those assumptions follows the existing exec/namespace
implementation; kernel checks and executable kernel bytes were unchanged.
These are test/observer failures, not reported AI-safety denials or resource
exhaustion. Their exact source hashes/scripts and nonzero statuses are retained.

[Source identity](source-identity.json) records eight selected executable/test
inputs, not a complete producer census. Documentation and evidence were recorded
after execution. Executable artifacts remain ignored; only their
[identities](artifacts.json), first-party source and logs are retained. The local
Git attribute preserves original log whitespace, with exact bytes indexed by
the manifest.

Reproduce through the [explicit native test](../../../../labs/qemu/x86_64/README.md#source-built-user-buffer-cases).
The archived runners/readback describe the selected experiment; the artifact
comparisons depend on retained local baseline artifacts and are not a cold
reconstruction facility. Other direct stat/open/directory/signal/command boundary
cases, complete ownership/mode/stability qualification, canonical production
binding and cold reconstruction remain required. Stage 3, initial Step 4 and the
full original kernel goal remain unfinished. See [scope](scope.json).
