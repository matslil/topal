# Stage 3 conditional scalar transfer admission

`TOPAL-SYSTEMS-USER-EXTENT-001`, `TK-DEC-283`.

The [artifact check](fixed-transfer-audit.json) connects 27 scalar user-buffer
instructions to nine actual extent success edges in the selected linked process
function. It checks exact candidate/count materialization and helper binding,
the immediate carry-refusal branch, preserved candidate aliases, transfer
direction and every accessed byte. Removing each success edge from the listed
direct-control graph makes its complete local transfer interval unreachable.
The original graph must first be reachable from its declared roots: absence
alone cannot establish dominance. All graph instruction bytes are checked
against the linked ELF, direct edges are recomputed, and actual LSTAR/exported
roots are checked before the reachability calculation.

| Transfer region | Admission | Scalar instructions |
| --- | --- | ---: |
| Echo input | Read 6 bytes | 2 |
| Exec signal reset | Read 32 bytes | 4 |
| SIGCHLD installation | Read 32 bytes | 4 |
| Process name output/readback | Write 16 bytes | 4 |
| Missing pathname | Read 9 bytes | 2 |
| Mail pathname | Read 21 bytes | 4 |
| Configuration pathname | Read 18 bytes | 3 |
| Stack limit output | Write 16 bytes | 2 |
| Descriptor limit output | Write 16 bytes | 2 |

The local raw-byte decoder accepts a small scalar subset and refuses indexed,
RIP-relative, segmented, address-size-overridden, string and read/modify/write
operations. Loads invalidate any overwritten pointer alias. The stack-limit
adapter restores its volatile R10 from preserved R8. The descriptor-limit
paths converge after selecting R8, so that case proves the current R8 candidate
rather than incorrectly attributing both paths to the preceding RSI move.
Writable admission permits readback under this x86 permission contract.
All selected accesses retain their emission-source locations.

This is **conditional local evidence**, not general memory safety. It relies
on the shared helper's specified preservation and admission contract, correct
decoder semantics outside these intervals, complete entry/callee/exception
contracts and stable owned mappings. The graph still has call-return
assumptions and external/indirect boundaries. The check makes no assertion
about accesses after each listed interval, other accesses in those syscalls,
error/effect ordering, mapping lifetime or native execution of every instruction.
Exec-reset and missing-path fallbacks remain natively unobserved; the third
named-condition fallback remains outside this direct graph. No historical
security check is removed or canonical production selection activated.

[Separate readback](readback.json) re-decodes every selected local instruction
with GNU objdump, checks all 21 read/six write operands and their byte bounds,
and walks backward from each access through unique predecessors to its success
edge. GNU and LLVM agree on every local instruction boundary and raw byte.
The [initial reader failure](readback-initial/failure.json) preserves GNU's
wrapped display and the resulting parser mismatch. Explicit `--insn-width=16`
fixes that reader without changing native code. This is not a kernel failure
or an AI safeguard notice.

[Five offline controls](controls.json) reject a missing edge, an invented edge,
a missing root, inconsistent instruction bytes and an omitted access origin.
They update the outer graph digest where necessary so rejection reaches the
structural checks. Only diagnostic JSON changes; first-party ELF inputs remain
unchanged and no corrupted artifact is executed. Five additional unit cases
exercise refusal bypass, an extra root, cycles/call-return overapproximation,
raw operand widths/base clobber and unsupported memory forms. These controls
are not authentication against replacing an entire producer.

The check is integrated before boot in the existing selected application and
source poll/pipe/storage/serial harness. A [fresh unchanged pinned BusyBox
exit](native-run.json) passes signal/terminal checks, 38 extent first reaches
and 77 completed dispatch callbacks. All 17 compared kernel objects/images,
the 2024544-byte pinned application, boot resource, 6999 console bytes and the
original census/graph/source-join metadata match the preceding checkpoint.
The native command takes 74.98 seconds including 4.68 seconds compilation
(selected test 70.26 seconds), peak 893915136 bytes. [Seven post-checks](post-validation.json)
pass: all 22 model/reader tests, both source-fixture build tests, workspace and
explicit included-file formatting, locked workspace/all-target checking, the
unchanged Rust size ceiling and whitespace. Post-check peak is 565604352 bytes.
Both retain zero swap and zero limit/OOM/kill events under the ordinary bounded
wrapper. The native source freeze is unchanged during execution.

The [116-input record](source-identity.json) explicitly inherits 113 unchanged
snapshots from the [preceding source-index archive](../stage-3-process-origins/README.md)
and stores three changed/new inputs locally. It is selected-source evidence,
not the complete producer. [Artifact identities](artifact-identity.json) likewise
reference the exact preceding bytes after fresh equality checks. The
[manifest](manifest.json) lists local files and inherited references separately.
[Python-only replay](replay.json) restores those exact sources/artifacts,
rechecks the complete source join and reproduces the scalar report exactly:

```sh
rtk python3 linux-kernel/tools/user-memory/results/stage-3-fixed-transfers/replay-fixed-transfers.py linux-kernel/.cache/fixed-transfer-replay
```

Keep both adjacent evidence directories when copying this package. Replay does
not rebuild the kernel or run LLVM, GNU, GDB or a VM.

Risk is high at the underlying memory boundary. This change adds test tooling
and integration with unchanged native bytes. Review is single-agent full-diff
and authority review, separate GNU/backward readback, offline controls and
ordinary VM observations, not independent human security review. All other
semantic access regions, helper soundness, native owner/mode/stale-state and
stable-lifetime qualification, complete root/callee/exception contracts, three
fallback obligations, canonical binding and committed production reconstruction
remain open. Stage 3, initial Step 4 and the original kernel goal are incomplete.
