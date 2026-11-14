# Stage 3: conditional fixed pipe output audit

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`, `TOPAL-SYSTEMS-USER-EXTENT-001`.

The [new reader](../../audit_pipe_fixed_outputs.py) checks two fixed output
paths in the existing experimental owned kernel: the eight-byte descriptor
pair and its eight-byte readback, and the four-byte readable-count result.
Every reviewed instruction has an exact byte and boundary contract. The
candidate/count setup, return-word erasure, carry gate, incoming local edges
and global success-edge cuts are checked against the actual linked graph.
The intervening 45-instruction lookup has a bounded four-record search, no
stores or nested calls, and preserves R8/R9 on normal return. Its private reads,
borrowed eight-byte return word, entry arguments and fatal/exception behavior
still require their own ownership and effect evidence.

[GNU readback](gnu/readback.json) independently checks 135 instructions in
three windows, all three user-memory operands and reverse admission cuts.
Its restricted operand review agrees on the explicit register/store effects.
[Twelve offline first-party ELF counterexamples](controls/controls.json)
rebind outer hashes and are rejected by the positive-backed reader. They cover
candidate/count changes, preparation and lookup writes to R8, output/readback
base changes, carry polarity, source-field changes and lookup bounds. Changed
ELF files are inspected only; none is executed, booted or loaded into a VM.
Eight new graph/operand models pass within the 60-test model/reader suite.

The [ordinary source-pipe VM regression](native/run.json) passes with the new
reader integrated into the existing selected builder. The one Rust case takes
231.03 seconds; its recorded command takes 237.66 seconds including compilation.
The three changed source files remain identical throughout. Cgroup swap is zero,
peak memory is 1965445120 bytes, and all recorded memory-event counters are zero.
The kernel, provider, census, graph and source-origin map are byte-identical to
the preceding checkpoint. Existing syscall/state observations are retained in
[the native observation identities](native/observation-identity.json). This is
a fresh regression, **not** a new per-access permission/ownership observer.

The [selected source record](source-identity.json) has 137 inputs: three local
snapshots and 134 inherited snapshots. Five unchanged artifact snapshots are
inherited by [identity](artifact-identity.json), avoiding duplicate ELF images.
The [frozen replay](replay.json) restores these inputs, reproduces the exact
[audit report](audit.json), and runs all eight focused models without executing
an ELF file, compiler, decoder, debugger or VM. From the repository root:

```sh
scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-fixed-pipe-audit/replay-fixed-pipe-audit.py linux-kernel/.cache/x86_64/fixed-pipe-replay-new
```

The output directory must not already exist. This is selected-input replay,
not a complete clean producer or canonical production reconstruction.

[Review and limitations](review.json) retain the two validation-script failures:
an initial control offset error and a GNU zero-displacement spelling mismatch.
Neither was a kernel failure or AI safety notice. No model/provider safety
interruption occurred in this increment; the historical trigger remains unknown.

Risk remains high if these conditional results are mistaken for complete
memory safety. Review is single-agent authority/source review plus separate
GNU and reverse-graph checks, offline counterexamples, models and VM regression.
No authoritative language/system design or executable kernel behavior changes.
The published caller/access index remains **79/91 sites and 306 instructions**;
these two new conditional reports are not added to it yet. The matching-kernel
native helper union remains **88/91**. Fresh per-access observation, actual
table/output/stack ownership and nonaliasing, continuous lifetime, complete
helper/entry/return/fatal/exception contracts, all twelve sites without published
intervals, fallback obligations and canonical production qualification remain
open. Stage 3, initial Step 4 and the original kernel objective are unfinished.
