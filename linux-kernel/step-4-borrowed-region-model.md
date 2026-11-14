# Step 4: borrowed-region model foundation

`TK-DEC-269`; implementation stage: portable ownership/access model.
Ordinary effectful source functions, native adapters and physical execution of
this boundary remain unqualified. This report does not add an executed kernel
capability or close Step 4.

The [design](design/borrowed-kernel-regions.md) moves the next compiler milestone
towards ordinary Topal subsystems over existing machine elements. The first
stage establishes the ownership and effect-identity protocol before source/ABI
integration. It introduces no new core allocator, mutable-reference meaning,
raw-address authority or native Linux handler.

## Implemented model

`systems_region_library.rs` borrows the existing `BootstrapStorageState` and
live `BootstrapRegion`. Load/store reaches those actual modeled owner bytes;
the invocation does not copy them into a separate store. Rust's exclusive
borrow keeps the owner inaccessible while the invocation exists. Runtime
generation/frame/view checks reject stale opaque view tokens, including copies
retained after completion. Qualified code labels are reference-model evidence
only, not native executable authority.

The binding model validates a closed acyclic call graph and complete maximum
call depth. Entry suspends caller access, admits all arguments and live-view
capacity, and publishes callee-local views. Return revokes the callee views and
restores the caller's views. All aliases, narrowed rights, windows, frames and
later invocations report the same owner resource identity for effect conflicts.
Bounds and rights never widen. The model admits 64 frames and 256 live views;
nonwrapping counters also reject before publication.

Completion requires the root frame. Cancellation/drop revokes every frame/view,
including nested calls, and preserves earlier successful stores. Revocation is
not transaction rollback or erasure of the owner's payload. Model metadata
revocation is not physical stack/runtime/code-binding erasure evidence.

## Validation and review

Validation passes: all 168 semantic-model tests, including 16 new region-borrow
cases; `cargo check --workspace --all-targets -j1`; and `cargo fmt --all -- --check`.
All Cargo/format commands ran through `scripts/run_bounded.py` with a 16 MiB Rust
stack and serial test execution. Rejection tests compare the complete debug state, including
underlying storage contents, transition observations, counters, view registry
and active frames. Positive cases read the same bytes through aliases and the
resumed owner. The tests cover nested calls, maximum depth/view capacity,
contained/end-empty windows, rights narrowing, value/range overflow, all-argument
prevalidation, stale code/owner/generation/frame/view evidence, identity
exhaustion, completion and cancellation/drop. These are model conformance tests,
not source/interpreter/compiler/native conformance claims.

The first full workspace test attempt failed during Rust compilation: the
systemd journal records a cgroup OOM kill at its automatically selected
903 MiB limit (903.7 MiB recorded peak). The limit was not raised or bypassed.
The subsequent bounded semantic suite and all-target workspace check passed.
The full workspace test suite is therefore not claimed as passing for this
stage. No native QEMU cohort was regenerated for this model-only change.
Logs: `/tmp/topal-tk269-semantics.log`,
`/tmp/topal-tk269-workspace-check.log` and `/tmp/topal-tk269-fmt-check.log`.

Risk is high at authority, alias effects, nonescape and partial publication.
Review checked the design against core resource/effect/layout meaning and the
approved kernel architecture, then checked constructor closure/depth, complete
argument admission, every byte-access precondition, nonwrapping identity
publication, frame retirement, owner restoration and evidence limits. The
implementation was reviewed in these roles by the implementing agent; no
independent human or agent code-review result is claimed.

## Remaining compiler milestone

1. Add checked systems views to ordinary function/module instantiation with
   explicit nonescape, effect and execution-context evidence.
2. Lower ordinary helper calls using qualified opaque region transport and
   provider byte access, with complete call/runtime bounds.
3. Bind frozen source/modules, type/code, owner, generation and access evidence
   into atomic artifact publication; reject tampering and undercharged proofs.
4. Execute real source helper frames on owned bytes under QEMU, inspect caller
   preservation and complete invocation reclamation, and retain the full primary
   kernel/application regression cohort.

The most recent physically qualified kernel remains the
[positional-transfer kernel](step-4-positional-file-transfers.md). Its historical
certificate/digest/cohort are not reissued as evidence for these new rules.
Distribution replacement, general processes/signals/dynamic ELF, persistent
VFS/storage, networking/isolation, Docker/Podman and VM hosting remain required
by the [acceptance ledger](step-4-status.md).
