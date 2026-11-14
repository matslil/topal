# Stage 3 source-built signal transfers

This selected checkpoint exercises `TOPAL-SYSTEMS-USER-EXTENT-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-SIGNAL-{ACTION,MASK}-001` through a compiled C
application and ordinary `rt_sigaction`/`rt_sigprocmask` arguments. It requests
registrations and queries, without signal delivery. The [native run](native-run.json)
passes four source invocations: 92 action calls, 92 mask calls and twelve heap
calls. These 196 observed requests are separate from the preceding source
transfer and exec-vector cohorts. The enclosing application also completes
its existing replacement and terminal checks.

The [source program](signal-cases.c) exercises:

- Complete action/mask input and output beyond page 34, crossing pages and
  directory boundaries. Successful 32-byte and eight-byte results also end
  exactly at the next unmapped page.
- Unmapped, supervisor, overflowing, read-only-output and partial-mapping
  candidates, with preserved admitted prefixes on refused output.
- Sigset-size and input-copy-before-selector error order, forbidden action
  installations, ignored query selectors, flag normalization and exclusion of
  signals nine/nineteen from action masks and blocked sets.
- Read-only code-resident input, action reset/restore, block/unblock/set,
  and overlapping input/old-value output. The returned old value must precede
  the committed new value.
- Eight valid updates whose old-value copies fail with `EFAULT`. These updates
  remain committed as the approved signal contracts require. The cohort does
  not incorrectly require all state to remain unchanged after such an error.

[Native observations](guest/observations.json) compare the complete 8192-byte
process table with exactly the permitted action/mask update on every signal
call, including failures. They preserve all other fields, including the umask,
thread registrations, shell/file controls and private command bytes. The full
256-byte transfer staging area must be zero before and after every request.
The source restores the original action/mask and old heap contents before
return. Callee registers, stack shape, complete table restoration and terminal
heap/token/command/thread retirement remain observed. The debugger only reads
guest state and sets ordinary observation breakpoints.

[Separate readback](independent-readback.json) uses sets of signal numbers for
block/unblock/set and a declared flag-bit set, independently of the native
emitter's bitwise implementation. It checks every observed state transition
and old-value result, all 36 code-resident inputs, eight successful overlap
transfers and eight output-fault commits. It parses all 10144 linked text bytes
inside a read-only executable load and verifies the syscall adapter and normal
runtime call. Its conservative 1496-byte source stack bound includes all C
frames/returns and existing adapter/scratch allowances, within the unchanged
2048-byte fixture bound. Other linked C entries are unused by this cohort.

Complete-table comparisons happen in the retained native observer. Readback
does not reconstruct all 8192 bytes from an archived full-table transcript;
the selected action/mask states and data-transfer records are retained. This
is single-agent self-review with separate native observations and a set-based
oracle, not independent human review or a complete security proof.

Seventeen exact comparisons preserve the preceding kernel objects, linked
kernel and boot image. All 7606 serial bytes match, and a fresh build preserves
the preceding source-transfer fixture image exactly. The [six post-check commands](post-validation.json)
pass two build-only tests, default-fixture preservation, formatting, locked
workspace/all-target compilation, source size and whitespace. No full-workspace
test execution or fresh external BusyBox qualification is claimed.

The native test finishes in 106.19 seconds under the unchanged 300-second guest
deadline. Bounded peak use is 359383040 bytes, with zero limit/OOM/kill events
and validation swap. Host protections and VM boundaries remain unchanged.

The [earlier failed run](../stage-3-source-signal-observer-failure/README.md)
retains its nonzero result and selected input snapshots. Its observer wrongly
included the adjacent process umask in a staging-zero assertion. The corrected
run checks the actual 256-byte staging extent while comparing all table bytes,
including that umask. A [post-run verifier correction](readback-correction.json)
also separates successful overlap transfers from rejected equal-pointer values.
Neither failure is treated as an AI safeguard decision or passing evidence.
Kernel bytes and security requirements did not change.

[Source identity](source-identity.json) freezes fifteen selected executable/test
inputs and seven unchanged routing sources. Documentation and evidence were
recorded after execution. This is not a complete producer census, clean-checkout
run or cold production reconstruction. [Artifact digests](artifacts.json) retain
identity without executable payloads; [the manifest](manifest.json) indexes
the retained files. Logs preserve their original whitespace. Reproduce through
the [explicit native command](../../../../labs/qemu/x86_64/README.md#source-built-signal-transfer-cases).

[Routing notes](../../caller-routing.md) distinguish the active process signal
service from old prelude fallback labels and identify predecessor-only stack
output checks. They do not remove a native acceptance obligation or claim
complete reachability analysis. Header-poll negative cases, a complete direct/
shared caller and user-access inventory, native ownership/mode/stability
qualification, canonical production binding and cold reconstruction remain
required. Stage 3, initial Step 4 and the full kernel objective are unfinished.
