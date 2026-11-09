# Step 4: owned user-memory extents

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-USER-EXTENT-001`, `TOPAL-COMP-SYSTEMS-USER-EXTENT-001`.

The [design](design/owned-user-extents.md) begins the next prerequisite for
general virtual memory. The production kernel still has the preceding finite
MM geometry and legacy user-buffer validators. This increment does not yet
change that executable behavior or complete initial Step 4.

The first stage records complete range, translation-permission, actual table
and backing ownership, stable entry lifetime and no-touch empty-range laws.
It identifies shared and independent buffer validators requiring migration,
and separates historical startup witnesses from arbitrary syscall admission.
The [primary-source record](knowledge/x86_64/user-memory-research.json) verifies
four exact files in the pinned upstream archive without importing their code.

The [model and tests](tools/user-memory/README.md) exercise the new contract
independently of the existing emitter. Model observations cannot create native
ownership; hardware context/stability and complete syscall/caller/publication
proofs remain required. The four implementation stages are recorded in the
design and will be commits in the existing initial-Step-4 PR.

Risk is high because an incorrect permission, ownership or lifetime check can
admit access to kernel storage. This stage uses a single-agent authority and
full-diff review, a separate flat permission-lattice oracle and guarded model
read traces. It makes no independent reviewer or native security proof claim.


## Stage 1 qualification

The [49-input record](tools/user-memory/results/stage-1/qualification.json)
passes all five recorded commands: seventeen model tests, all four Rust coverage
tests, workspace formatting, the unchanged Rust file-size ceiling and whitespace.
The model cases include 8192 direction/path permission combinations and an
actual 400-page model extent. No production Rust, LLVM or Topal kernel source
changed in this stage.

[Independent readback](tools/user-memory/results/stage-1/independent-readback.json)
checks every snapshot and log, compares retained production source bytes to the
preceding commit, replays all seventeen tests using isolated archived model
bytes, and verifies 681 local Markdown targets. It is artifact/model evidence,
not an independent human review or native execution proof. Exact qualification
inputs remain frozen; report and traceability updates only record the outcome.

Peak use was 107311104 bytes, with zero limit/OOM/kill events and zero validation
swap under the unchanged wrapper and host reserve. An initial harness input
enumeration named a nonexistent frontend requirement file and stopped before
tests. Its [failure scope](tools/user-memory/results/stage-1-harness-path-failure/failure.json)
remain retained. Removing that path required no model or runtime correction.

Stage 1 is complete. The actual native walk, all-caller migration and committed
production qualification remain stages 2–4. The full original kernel objective
is still unfinished. See the [replay notes](tools/user-memory/results/stage-1/README.md).
