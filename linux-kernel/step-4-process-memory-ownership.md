# Step 4: process memory ownership

`TK-DEC-284`, `TOPAL-REQ-SYSTEMS-PROCESS-MM-001`,
`TOPAL-SYSTEMS-PROCESS-MM-001`, `TOPAL-COMP-SYSTEMS-PROCESS-MM-001`.

## Stage 1: design and portable reference foundation

The [design](design/process-memory-ownership.md) preserves the qualified child,
resumable-pair and pipe behavior while replacing the assumption of a fixed
34-page private heap with independent live frame ownership. Runtime MM already
updates the active heap pointer after allocation; its initial zero value is not
evidence that all child creation must be refused. The remaining integration gap
is the fixed snapshot/leaf-table exchange and single active MM state across
independent resize, handoff and exec.

The [reference model](tools/process-memory/README.md) separates process/PID,
address-space identity, image generation, entry epoch and extent issuance.
Parent/child heaps keep distinct backing while inactive; handoff remaps without
heap byte exchange. Child exit restores the parent before releasing the child.
Outer cancellation consumes both held heaps once. Failed creation consumes no
PID/MM identity. If an extent was already issued before a later preparation
miss, its issuance stays consumed while that extent is retired.

Seventeen bounded Python tests pass, including 3,200 generated application
transitions checked against a separate process-byte/break/generation oracle.
Explicit cases include zero, partial, 34-page and larger heaps through the
current 519-page geometry; 4/16/64-KiB page examples; independent writes/resize
and either process's exec; occupied/exhausted/fragmented/prepared-failure cases;
wait blocking/WNOHANG/output retry/recreation; stale entry/loan refusal and
inactive-child cancellation. A separate event checker rejects nineteen wrong
histories and state validation rejects seven inconsistent snapshots. All such
controls change reference data only.

The [qualification record](tools/process-memory/results/stage-1/report.json)
binds the selected exact inputs, full unit output, local links/stable IDs and
observed bounded-run resource controls. No Rust implementation, linked kernel,
provider, third-party binary, boot resource, canonical selection or host
protection changes in this stage. No VM or production regression is newly run.
Model identity/zeroing/invalidation facts are not native authority or physical
observations. The models omit actual image/stack/register/descriptor transfer,
table/permission checks, width exhaustion, partial-mutation fatal cleanup and
asynchronous/native lifetime proofs; those remain required.

Risk is high for isolation, ownership, translation and evidence errors. Review
uses single-agent authority/source/diff review, separate application-state and
event-history checks, plus model-only negative controls. No independent human
or native security review is claimed. Existing qualified native evidence remains
historical and is not extended by these results.

## Remaining stages and blockers

2. Bind actual active/inactive/prepared MM owners to allocator handles, captured
   process/context/entry, pinned translation/backing resources and every caller.
   Preserve exact resource/retirement costs and fatal partial-effect handling.
3. Migrate creation, all switches, exec/exit/wait/cancellation and overlapping
   leaf-table writers. Run source-built ordinary syscall witnesses for independent
   heaps/partial breaks/resize/exec/yields/wait retry and physical cancellation.
   Preserve existing child, pipe and unchanged pinned application workloads.
4. Complete affected production qualification and immutable source/native/load/
   boot reconstruction before activation. Preserve mode, concurrent-access and
   fault gaps as acceptance blockers, not successful model evidence.

The [owned-extent work](step-4-owned-user-extents.md) retains its own unfinished
caller, lifetime and production gates. This foundation does not complete that
work, initial Step 4 or the original distribution, Docker/Podman and VM-host goal.
