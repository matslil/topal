# Stage 3 additional scalar admission intervals

`TOPAL-SYSTEMS-USER-EXTENT-001`, `TK-DEC-283`.

The [summary](summary.json) adds conditional local checks for 33 helper-call
regions and 51 user-access instructions. These cover working-directory results
and paths, bounded opens/stat paths, signal records, prompts/newline output,
poll input/result fields, header-byte publication/readback, time/TLS/child
results, and individual exec/environment/namespace byte or vector probes.
No executable kernel or application behavior changes.

The reader verifies exact candidate/count setup and helper binding, immediate
carry refusal, actual preserved register aliases, raw memory direction/width,
complete candidate-relative byte bounds and every incoming local edge. Removing
the success edge makes each selected access unreachable from the existing
verified direct-control roots. A repeated byte/vector probe qualifies only
that invocation's admitted access; private staging limits and full loop/progress
contracts are separate obligations.

The poll result uses the exact target-word relationship `candidate = RDI + 6`.
Its two-byte store at `[RDI + 6]` therefore has offset zero within the admitted
two bytes, rather than a six-byte offset inside them. The selected interval
ends after that store. The later complete-record readback needs its separate
input-admission/liveness reasoning and is not covered by this output interval.

The header-byte preparation has four payload branches and a captured-pointer
reload. Its complete preparation DAG cannot overwrite the captured pointer;
the reload must use the same full-width GS field established before admission.
The selected byte store/readback then uses that preserved candidate. GS offsets
establish no ownership. The TLS-result preparation accounts for the exact
FS-base MSR read's RAX/RDX effects without granting a general MSR or exception
contract. The wait-status preparation's additional qword read at `[R15 + 8216]`
is explicitly exposed as an unqualified ownership/access obligation; its
register name does not prove private storage.

The shared decoder now handles the selected byte loads/stores/comparisons and
byte zero-extension. A byte destination such as R8B destroys the full R8 pointer
alias. Legacy high-byte destinations are refused. The unneeded combination of
operand-size override and REX.W is also refused, rather than using the wrong
immediate length. Preparation decoding adds exact GS-byte and AL-comparison
effects. The preceding nine-scalar, fifteen-REP and sixteen-staged reports remain
**byte-identical** for the selected kernel.

[Independent readback](readback.json) decodes all 279 selected instruction
boundaries in 33 windows with GNU. It checks 38 read/13 write operands, all
51 complete byte bounds and 51 backward success-edge cuts. Original GNU
commands/stdout/stderr are losslessly bundled with per-entry identities.
[Eight offline controls](controls.json) corrupt copied first-party provider and
linked artifacts, rebind outer census/graph/join identities, and reach semantic
rejections for shifted-candidate/final-byte bounds, pathname end, byte candidate,
header reload, signal end, clear-result store width and carry polarity. Changed
artifacts are never compiled, booted, loaded into a VM or used to alter guest
state. These controls do not authenticate source provenance.

The fresh [pinned BusyBox test](native-run.json) passes in 107.31 seconds.
Its entire native observation object, all seventeen compared kernel
objects/images, application/boot resources, original census/graph/source
metadata and 6999 console bytes match the preceding run. It reaches fourteen
of these 33 helper sites before invocation; **this is not new native scalar
transfer observation or proof that all 33 regions execute in that run**. Earlier
native syscall/refusal records retain their own scopes. No new native region
observer or additional application fixture is introduced here.

[Seven post-checks](post-validation.json) pass: all 45 model/reader/observer
tests, both source-fixture build tests, workspace and explicit included-file
formatting, locked workspace/all-target checking, the unchanged Rust file-size
ceiling and whitespace. The six new controls cover byte aliases, shifted bounds
and setup, register-only effects, GS-byte reload distinctions and ambiguous
prefix/immediate lengths. Native peak use is 421466112 bytes and post-check
peak is 55910400; both have zero validation swap and zero limit/OOM/kill events.
The native selected source hashes stay unchanged during execution. VM deadlines,
reserves, host protections and guest filesystem/network boundaries are unchanged.

An initial reader construction lacked the header's existing AL comparisons and
was corrected before source freeze. Review then added the prefix rejection and
a regression control, followed by a fresh native run and all final checks.
Neither development correction was an AI safeguard notice. No new safeguard
interruption is reported or inferred from repository behavior.

The [joined index](caller-access-index.json) now identifies 73 of 91 helper-call
sites with selected local intervals and 300 distinct user-access instructions.
The remaining eighteen sites are listed explicitly. This does not qualify whole
callers: other accesses may remain before or after every selected interval.
The matching-kernel native census union remains 88/91 and all three fallback
obligations remain open.

The [125-input record](source-identity.json) stores five changed/new inputs and
inherits 120 unchanged snapshots. [Artifacts](artifact-identity.json) are
inherited after equality checks; the [manifest](manifest.json) separates local
files and inherited Git-equal references. [Python-only replay](replay.json)
restores the selected source/artifact bytes, rechecks the complete emission join
and reproduces the exact additional-scalar report:

```sh
rtk python3 linux-kernel/tools/user-memory/results/stage-3-additional-scalars/replay-additional-scalars.py linux-kernel/.cache/additional-scalar-replay
```

Replay does not rebuild or run a kernel, GNU/LLVM, GDB or a VM. Risk remains high
at the memory boundary. Review is single-agent source/authority/full-diff review,
separate GNU/reverse-cut readback, malformed-artifact controls and unchanged
native regression; no independent human security review is claimed. Complete
helper/entry/callee/exception contracts, mapping/counterpart/table/backing owners,
continuous lifetime, other accesses and syscall effects, native mode/stale-state
qualification, canonical binding and exact committed production reconstruction
remain required. Stage 3, initial Step 4 and the full kernel/distribution/device/
driver/container/VM-host objective remain incomplete.
