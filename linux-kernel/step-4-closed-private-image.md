# Closed compilation and reconstruction for the private process kernel

The private process-MM kernel builder now uses the canonical frame compiler's
static relocation, local symbol binding, source global dead-code elimination,
normalized source identity, native options and instruction inspection. Previously
its separate compilation path retained live GOT entries and an unreachable
task-runtime global in writable source storage, preventing the approved complete
image reconstruction. The GOT/readonly-source checks remain strict.

This realizes existing `TOPAL-SYSTEMS-TAIL-BINDING-001`,
`TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001` and part of
`TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001`, with corresponding compiler requirements
and `TK-DEC-284`/`TK-DEC-285`. Core language semantics and source syntax are
unchanged. The approved static-link design already requires local binding and
source retention before qualification; it excludes GOT/TLS relocations.

## Construction and ownership of compiler inputs

The fresh private producer reconstructs all 23 root/provider/source/runtime/owner/
controller/target/dispatch/gateway/boundary/extent and policy-library objects.
Every allocated contribution receives compiler-generated endpoints. Existing
relocation reconstruction checks symbols, exact bytes, alignment, permissions,
disjoint placement and loaded projection; complete allocation coverage and the
actual image reservation remain required. The validated private image has
862 contributions, a checked ELF header and only the checked empty-GOT page tail.
It has no live GOT slots or writable ordinary-source storage.

Original producer objects stay intact for the existing independent audit readers.
Marked copies are written into a separate `linked-inputs` directory and supplied
to the linker. The builder records all 23 original hashes before preparation and
asserts them again after linking; the driver independently rehashes them. Baseline
published library objects and the pinned third-party application are unchanged.
Four static counterexamples reject altered executable/immutable/zero contribution
sizes and an omitted policy library; they execute no malformed guest code.

Complete image reconstruction produces the descriptive image-layout component
from this actual private image. It still provides only image subdivisions and
their additional leaf storage, not complete root backing or live protection.
`private-image-binding.json` explicitly records false production qualification
and false protected-context status. Extra extent-role preparation and private
inspection remain test-only. The public source-bearing API retains its complete
canonical reconstruction gate, and predecessor private profiles keep their
compilation paths.

## Validation

The [bounded matrix](tools/user-memory/run_closed_private_image.py) passes all
ten commands: five canonical compiler layout cases, the unchanged default ABI
fixture, workspace/all-target checking, formatting, the Rust file ceiling,
271 user-memory and 83 process-memory models, the new private build-only case,
ordinary child exec, and all 51 unchanged pinned BusyBox workloads. Independent
saved-record readers pass on the fresh child case and every application run.
The child case executes four generations and sixteen ordinary failed execs.

All 766 watched inputs remain unchanged throughout the
7464.73-second matrix. The final cgroup peak/limit are
1,801,719,808 / 3,379,101,696 bytes, with
zero swap and recorded memory events. The
[result summary](tools/user-memory/results/stage-3-closed-private-image.json)
retains exact commands, artifact layouts, native/readback scope, counterexamples
and resource observations. Complete local logs and source hashes remain under
the matrix output directory; this summary is not a closed source archive or
frozen replay receipt.

The descriptive reports were updated after validation. Among watched inputs,
only `step-4-status.md` changed; the summary records its before/after hashes.
Compiler, model, driver and kernel implementation inputs match the validated
snapshots.

The build-only, child-exec and application builds all produce kernel
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.
Each reconstructs 23 objects and 862 allocated contributions, preserves all
23 original producer objects, and rejects the four static altered-input cases.
Its actual image layout needs three additional leaf tables, totaling 12,288
bytes. This is additional image storage, not a complete root allocation.
The unchanged pinned application digest is
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.

Raw root, provider, gateway, boundary, extent-callers and extent-bridge objects
match the preceding calendar kernel in the preliminary comparison. The complete
kernel changes because source retention/local binding and linked layout change;
the preceding native results keep their original image identity and scope.

The initial private reconstruction correctly refused `source:.data`; reusing
the entire canonical compilation path removes the unreachable runtime object
before inspection. The first full matrix later stopped before BusyBox guest
execution when linker markers changed the audited provider object's identity.
The correction preserves raw objects and marks separate link copies; the original
identity assertion is retained. Preserve both failed runs and distinguish them
from guest/resource/AI-safeguard failures. No safeguard denial occurred here.

The selected validation runs ordinary source-built syscall programs and unchanged
pinned BusyBox. It does not rerun the full compiler suite's legacy host-RWX or
debugger-injected-state tests; that suite remains incomplete. Resource bounds,
host filesystem/network/security boundaries and no-swap execution remain intact.

## Review and next native integration

Risk is high for native compiler/link changes interacting with ownership, stack,
caller and relocation assumptions. Review is single-agent authority and complete
diff review, fresh source/object/link checks, independent original-object hash
checks, static counterexamples, ordinary native execution and saved-record
readback. No independent human security review or continuous lifetime proof is
claimed.

This stage implements private image construction and reconstruction. It does
not change page-table builders, owner geometry checks, hardware permissions,
the bootstrap-equivalent contract or canonical publication. Hardware read-only/
execute-disable protection, all-entry/caller/alias/writer/CPU/lifetime proof and
full boot production reconstruction remain open.

Next, bind complete provider-selected topology and backing to both the implicitly
allocated bootstrap bank and the separately constructed static-process bank,
including every restorable root. Preserve exact issuer/generation/count/
alignment/pin/bounds/exclusion/user-table and lifetime checks; image leaf bytes
alone cannot authorize a five-to-eight-page replacement. Establish effective
controls and publication/maintenance before activation, retain write protection
and actual-root retirement evidence. The full original Linux UAPI, DTS,
distribution, Docker/Podman and VM-host acceptance remain unfinished.
