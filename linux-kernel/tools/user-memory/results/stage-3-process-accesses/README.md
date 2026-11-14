# Stage 3 linked process instruction and potential-access inventory

`TOPAL-SYSTEMS-USER-EXTENT-001`, `TK-DEC-283`.

The [linked graph summary](graph-summary.json) records a direct-control
overapproximation of the selected process function. Roots are its actual ELF
entry, the RIP-relative address programmed into LSTAR and five exported internal
symbols. Both outcomes of conditional branches are included. Calls include an
explicit return assumption; their external/indirect effects are separate review
obligations. All direct branch displacements and instruction bytes are checked
against the exact linked kernel. This is not a complete all-kernel entry graph.

The graph contains 31016 instructions spanning 221656 bytes, with 36938 edges,
18 external transfers, six indirect calls and 267 return/exit/halt terminals.
No reached instruction is unknown or overlaps another reached instruction.
The 10007 unvisited function bytes are not all classified as data or proven
unreachable. Linear decoding had missed two valid branch destinations at offsets
162020 and 185049; targeted decoding repairs their instruction boundaries.
The [GNU verification](gnu/gnu-verification.json) independently decodes every
graph node, agrees on all instruction lengths and branch/return edges, and
reconstructs the same closure from the listed roots. This does not validate
missing roots or callee contracts.

Potential-access inventory lists 13994 instructions with explicit addressed
operands and 590 implicit-stack instructions. Explicit operands comprise 8640
GS-based, 1003 stack-base and 4351 general-address instructions; 557 string
operations are included in these counts. Intel bracket syntax also captures
absolute GS operands that a parentheses-only text scan would miss. LEA is
excluded from memory operations. These are syntactic categories, not claims
that every operand is user memory, has a particular transfer direction or is
safe. Address-bearing operations such as INVLPG still require semantic review.
The complete node/edge/operand ledger is compressed in
[the exact graph artifact](artifacts/process-access-inventory.json.gz).

Ninety of the 91 emitted extent calls occur in this graph. The named-condition
call at offset 88236 is outside it. The graph includes the two other unobserved
fallback calls, exec signal reset at 94708 and missing pathname at 101802.
An [exploratory all-emitted-calls assumption](exploration.json) failed for 88236;
the final inventory reports the distinction and does not discard the call or
remove its acceptance obligations. Outside this direct graph is not a complete
whole-program unreachability proof. The [indirect-load ledger](indirect-context-loads.json)
checks each call's actual GS-slot load, null guard and aligned-call slice.
Slots correspond to policy, numeric, header, dispatch and identity source
controllers; registration, owner binding and callee effects remain separate.

The [fresh native command](native-run.json) passes the unchanged pinned BusyBox
first-image exit with signal transfer and terminal retirement observations.
The [native binding check](process-access-coverage.json) requires all 38 observed
helper calls to belong to this graph. [Separate readback](readback.json) additionally
places all 88 sites in the preceding matching-kernel native union in this graph.
These are helper first-reach checks, not a native instruction or access trace.
All 17 compared kernel objects/images, the unchanged 2024544-byte pinned BusyBox,
the boot resource and all 6999 console bytes match the preceding checkpoint.
The pinned digest remains
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.

The native command takes 69.57 seconds including compilation; its selected test
takes 64.78 seconds. Peak cgroup memory is 940883968 bytes. All
[six post-checks](post-validation.json) pass, including two fixture-build tests,
workspace/all-target checking, formatting, file size and whitespace. Post-check
peak is 509779968 bytes. Both retain the normal bounded wrapper, zero swap and
zero limit/OOM/kill events. GNU readback also runs through the wrapper and
only reads host artifacts. Its retained warning concerns ignored `.debug_names`
metadata; all instruction-length and control-edge checks pass.

The [source record](source-identity.json) freezes 70 selected inputs; it is not
the complete producer. [Artifact identities](artifact-identity.json) and
[manifest](manifest.json) bind the linked/object/decoder/observer evidence. The
[offline replay](replay.json) restores archived source and first-party artifacts,
recomputes the exact LLVM graph, reruns GNU re-decoding and checks the recorded
observation binding. Supply an LLVM objdump executable as the second argument:

```sh
rtk proxy python3 linux-kernel/tools/user-memory/results/stage-3-process-accesses/replay-graph.py linux-kernel/.cache/process-graph-replay /path/to/llvm-objdump
```

Replay uses GNU GDB but no VM or third-party application execution. It neither
rebuilds the kernel nor constitutes production cold reconstruction.

Risk remains high for the underlying memory boundary. This increment changes
test integration and artifact inspection; compared kernel bytes are unchanged.
Review is single-agent authority/source review, two decoder implementations,
exact artifact comparisons and native observations, not independent human
review. Complete callee/exception/root closure, user-pointer origin and direction,
admission dominance, check-to-copy lifetime, native ownership/mode/stale-state
qualification and production activation/reconstruction remain required. All
three fallback coverage/reachability obligations remain open. Stage 3, initial
Step 4 and the original MM/distribution/driver/container/VM-host goal are unfinished.
