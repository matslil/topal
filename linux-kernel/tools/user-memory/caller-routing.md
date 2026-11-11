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

The [ordinary serial cohort](results/stage-3-source-serial/README.md) reaches the
remaining serial descriptor site through real stdin reads. Its 64 source calls
observe actual UART data consumption, refusal before consumption, copy windows,
table preservation and bounded 512-to-256 admission. Eight host messages travel
through the existing VM serial connection after ordinary application markers.
The four matching-kernel cohorts now reach 88 of 91 sites in union. Three bounded
fallback inputs remain unobserved; their dispatch/ownership classification and
the complete direct-access/caller obligations still require explicit evidence.

The [selected result-gate audit](results/stage-3-transfer-guards/README.md)
checks immediate carry refusal after return-word erasure at all 91 calls and
the exact local bounds of the three remaining fallback read slices:

| Fallback | Selected emitted call offset / admitted bytes | Source lifecycle and local read boundary |
| --- | --- | --- |
| Named-root condition output | 88236 / 14 | [Output emitter](../../../src/topal-compiler/src/systems_process_entry_output.rs) requires context byte 418 to equal four, plus the preceding command/condition state. Its one-byte loop reads indices 0–13 before private content checks and serial publication. |
| Exec signal reset | 94708 / 32 | [Completion emitter](../../../src/topal-compiler/src/systems_process_entry_completion.rs) selects the reset with context byte 418 equal to six. It reads eight-byte fields at offsets 0, 8, 16 and 24 after input admission. The earlier active-table routing reasoning remains separate from native execution or full reachability proof. |
| Missing root pathname | 101802 / 9 | [Completion emitter](../../../src/topal-compiler/src/systems_process_entry_completion.rs) requires context byte 418 equal to four and pathname-query state. It reads eight bytes and the terminator at offset eight. An absent lookup has no metadata output admission or transfer. |

The two phase-four paths precede the final command read's phase-six/table
activation. Active-table interception therefore does not justify dismissing
all three fallback paths as unreachable. The fresh unchanged BusyBox exit
cohort reaches 38 sites already in the preceding union; the union remains 88
of 91. No historical check is removed. The audit's direct-entry checks concern
decoded branches into these local slices only; embedded data, unknown decoding,
indirect/exception entry, complete access inventory and stable observation
still need their own evidence.

The [linked process graph](results/stage-3-process-accesses/README.md) now lists
31016 instructions and their potential addressed/implicit-stack operations.
Exact LLVM byte checks and GNU re-decoding agree on every instruction boundary
and all 36938 direct-control edges. Two real branch destinations require targeted
decoding after the linear listing loses alignment. All 88 preceding native
first-reach sites occur in this graph. Exec signal reset and missing-path calls
are in its 90-call set; the named-condition call is outside it. The 91-call
emission census and all three fallback obligations remain intact. The graph's
listed roots, call-return assumptions and 18 external/six indirect boundaries
still require complete entry/callee contracts. Its 14584 potential-access
instructions are a review inventory, not user-pointer classification or proof
of admission dominance, transfer direction or stable mapping lifetime.

The [callback binding and ordinary-execution record](results/stage-3-callback-bindings/README.md)
now follows all five source-controller entries from actual bootstrap relocations
and argument pushes through private context registration to the six indirect
calls. All 357 allocated sections match their source objects after 978 relocation
applications. The final ordinary storage cohort observes all six sites and 1168
returns, checks the actual registered targets, supervisor execution and ABI/
captured-argument/registry/full-process-table preservation, and compares all
38541 live section bytes on first use of each controller. The separate unchanged
BusyBox exit observes 77 dispatch calls. Hardware supervisor write permission
remains present; this is not hardware read-only code qualification. Recorded
source/runtime call certificates, complete callee effects, alternate entries,
pointer authority, admission dominance and stable lifetime retain their own
obligations. No metadata association grants new kernel authority or closes a
fallback gate.
