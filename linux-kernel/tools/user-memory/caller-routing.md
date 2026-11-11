# Owned caller routing notes

This is a source-review record for `TOPAL-SYSTEMS-USER-EXTENT-001`, not a
complete caller inventory or proof that every direct user access is qualified.
The [signal checkpoint](results/stage-3-source-signals/README.md) records native
observations separately from the following control-flow reasoning.

| Previously named area | Current owned selection | Evidence boundary |
| --- | --- | --- |
| Initial process-name and stack-limit output | Both adapters choose complete owned output admission. Their calls to the old `emit_stack_output_extent` helper occur only in the predecessor `else` branches. | [Adapter source](../../../src/topal-compiler/src/systems_process_entry_completion.rs) and [startup native checkpoint](results/stage-3-source-startup/README.md). This does not qualify every initial-vector construction or publication obligation. |
| Signal reset after the final shell command | The final read initializes the process table before publishing command phase six. Initialization imports/consumes prelude signal state and publishes the active table. On subsequent syscall entry, active-table action/mask dispatch precedes the old prelude signal selector. | [Final-read source](../../../src/topal-compiler/src/systems_process_entry_read_exec.rs), [initialization](../../../src/topal-compiler/src/systems_file_descriptions.rs), [entry order](../../../src/topal-compiler/src/systems_process_entry_start.rs) and [signal dispatch](../../../src/topal-compiler/src/systems_process_signals.rs). The new cohort requires that actual table to be active and tests its service; it does not force the old fallback route. A complete reachability/producer inventory remains required. |
| Header readiness polling | The owned adapter admits both validated header forms, repeated polls and arbitrary input `revents`, retaining the open immutable description, complete copy checks and ordered read consumption. The external form retains its terminal probes; clean direct reads remain valid. | [Header-read adapter](../../../src/topal-compiler/src/systems_process_entry_read_exec.rs) and [new native poll/regression record](results/stage-3-owned-poll/README.md). Source-issued buffer cases now reach the common poll path, and unchanged pinned BusyBox separately exercises the external profile. General descriptor/event/count/timeout polling remains required. |

The process transfer staging extent is 256 bytes beginning at table offset
1408. The process umask begins at offset 1664, before the signal action array
at 1792. Signal preparation and return use the shared staging wipe; testing
that wipe must preserve the adjacent umask and all other process state. The
signal observer compares all 8192 table bytes against the operation's permitted
action/mask update, and independently requires the entire 256-byte staging
extent to be zero. It does not erase or ignore adjacent ownership.

The old fallback labels remain emitted. Source reasoning about the normal
dispatch order is not native execution of those labels, a claim that arbitrary
corrupt private state is unreachable, or permission to delete their checks.
Every remaining ownership, paging-mode, stable-observation, direct/shared-caller
and production reconstruction obligation remains in force. The source identities
used for these notes are retained with the selected signal and readiness evidence.

The [selected emitted-call census](results/stage-3-call-census/README.md) now
checks all 91 calls to the two owned helpers in the selected provider against
ELF bytes and LLVM disassembly. The enclosing ordinary process/poll cohort
reaches 77 sites and leaves 14 unobserved: three bounded fallback inputs,
positional and serial descriptor reads, named-directory publication, four
process-pipe calls and four volatile-file calls. Exact offsets, immediate
emitter locations and preceding native labels are retained. These are first-hit
observations before invocation, not admission/return qualification or a complete
direct-access inventory. They identify the next active-descriptor source cases
and fallback classification needed without weakening any acceptance gate.

The subsequent [ordinary FIFO cohort](results/stage-3-source-pipes/README.md)
reaches all four previously unobserved process-owned pipe sites. Its 144 source
calls verify complete selected table/FIFO transitions, buffer refusals, actual
byte transport and exact endpoint/ring retirement. The two matching-kernel
cohorts reach 81 of 91 sites in union; ten remain unobserved in that union.
This does not replace the complete direct-access and caller qualification gates.

The [ordinary storage cohort](results/stage-3-source-storage/README.md) adds the
four volatile-file helpers, immutable-file pread and named-directory publication.
Its 208 source-issued calls compare selected table/filesystem transitions,
failure atomicity, output, independent positions and complete anonymous-file
reclamation. Exact matching-kernel/census checks establish an 87-of-91 union
with the preceding poll and pipe cohorts. Serial descriptor read and three
bounded fallback inputs remain unobserved in that union. The complete direct-
access, length/error/lifetime and production gates remain separate and required.
