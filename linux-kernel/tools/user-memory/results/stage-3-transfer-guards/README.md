# Stage 3 selected transfer-result and fallback-read audit

`TOPAL-SYSTEMS-USER-EXTENT-001`, `TK-DEC-283`.

The [artifact audit](transfer-guard-audit.json) verifies all 91 recorded helper
calls, actual helper bindings and return-word erasure before requiring an
immediate carry-refusal branch. The nine-byte erasure is a flag-preserving MOV;
no intervening instruction changes the admission result. The branches reach
14 decoded refusal entries. This applies to the exact selected provider,
not every kernel configuration or every direct user access.

Three local fallback slices additionally check the actual candidate setup,
admitted length, success/failure branch, read widths/indices, content-mismatch
targets and decoded direct entries. The named-condition copy admits 14 bytes
and reads indices 0 through 13. Signal reset admits 32 bytes and reads four
eight-byte fields. The missing-path lookup admits nine bytes and reads one
eight-byte word and its terminator. Seven static read instructions are checked;
this is not native execution of those slices or a transcript of their loads.
The [routing notes](../../caller-routing.md) distinguish their lifecycle guards.

The [fresh native test](native-run.json) passes the unchanged pinned BusyBox
first-image exit with actual signal-table transfer and terminal retirement
observations. Its [coverage](guest/observations.json) reaches 38 of 91 sites.
[Separate readback](readback.json) verifies all 17 compared kernel objects/images
against both the preceding serial cohort and earlier external-image cohort.
The 2024544-byte BusyBox image retains SHA-256
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.
All 6999 console bytes match the earlier external cohort exactly. Combined with
the four preceding matching-kernel cohorts, the union remains 88 of 91; no
additional fallback reachability is claimed.

Five [offline controls](readback.json) alter copies of compiler-owned object
bytes and rebind the function digest. The checker rejects changed carry
condition, admitted length, loop bound, action-field offset and pathname
terminator offset. These copies are never executed, loaded into a VM or used
to change the pinned third-party image. Their decoder listing stays unchanged;
the controls establish byte-shape rejection, not a separate decoder proof.
An earlier checker incorrectly expected a seven-byte encoding of the common
`-EFAULT` result. Actual bytes use ten-byte MOVABS. The retained
[checker replay](earlier-checker-replay.json) and [log](earlier-checker-replay.log)
document this corrected offline assumption; they are a replay of the initial
checker, not the original attempt, a failed guest test or an AI safety denial.

[Six post-checks](post-validation.json) pass: two fixture-build tests, workspace
formatting, explicit included-file formatting, locked workspace/all-target
checking, the unchanged Rust file-size ceiling and whitespace. Native command
elapsed time is 68.04 seconds, including compilation; peak cgroup memory is
898260992 bytes. Post-check peak is 511676416 bytes. Both retain zero swap and
zero limit/OOM/kill events under the unchanged wrapper and host reserve.

The [source record](source-identity.json) freezes 62 selected inputs, including
the applicable authority documents and actual fallback emitters. It is not the
complete producer. The [manifest](manifest.json) covers retained evidence. To
replay the exact static object audit into a new directory:

```sh
rtk proxy python3 linux-kernel/tools/user-memory/results/stage-3-transfer-guards/replay-audit.py linux-kernel/.cache/transfer-guard-replay
```

This replays archived first-party object/disassembly and checker bytes without
guest execution or the external BusyBox binary. [Replay evidence](replay.json)
requires the generated report to equal the archived report exactly.

Risk remains high for the underlying memory boundary. This increment changes
test selection, observation and artifact auditing; compared kernel bytes are
unchanged. Review is single-agent self-review against higher-level contracts,
separate ELF/byte reading, offline controls and native observations. It is not
independent human review. The linear disassembly contains 57 unknown rows and
six decoded indirect transfers, including embedded-data decoding limitations;
it cannot establish a complete reachable instruction graph or access inventory.
Native fallback execution, global direct/shared-access dominance, helper and
mapping lifetime/mode/ownership qualification, canonical production activation
and exact committed cold reconstruction remain required. Stage 3, initial
Step 4 and the full distribution/container/VM-host goal remain unfinished.
