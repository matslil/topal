# Step 4: retained ordinary source identity state

`TK-DEC-263` implements the [owned-version design](design/retained-source-identity-state.md),
`TOPAL-SYSTEMS-RETAINED-NUMERIC-STATE-001` and
`TOPAL-SYSTEMS-RETAINED-PROCESS-IDENTITY-001`. Successive process-identity calls
now borrow the actual previously published ordinary Topal Record and numeric
object. The selected owner preserves this value between calls and reclaims each
old version after its last borrow. The [scalar-only interface](step-4-source-process-identity.md)
remains independently supported. [Step 4](step-4-status.md) remains unfinished.

The primary kernel SHA-256 is:

```text
2ec272717125d715492847d0138fd201bd3f0a03fe8d09e6c25fb299c4ed02c0
```

Artifact/root/provider/object/boot/gate revisions are
`239 / 225 / 226 / 218 / 139 / 239`. Numeric metadata, allocation certificates
and the owned numeric machine profile use version seven. The retained-state
certificate uses `topal.retained-numeric-state/1`. This is systems qualification;
ordinary language semantics and the source identity algorithm are unchanged.

## Actual value ownership and publication

The root selects `retained-process-identity-v1`, with an explicit 4096-byte state
reserve, unchanged 4096-byte invocation stack and 65536-byte temporary arena.
The [source policy](kernel/policy/process-identity.t) and imported
[helpers](kernel/support/process-identity.t) still expose
`Record (next : Nat) -> Record (allocated : Nat, next : Nat)` without storage or
machine vocabulary. The owner retains a separate supervisor-owned 4096-byte
region. Its private 64-byte header identifies ownership, selected code, current
version and one pending preparation. Two 2016-byte banks hold alternating values.
Each complete ordinary graph occupies 40 bytes: a 16-byte Record with order
suffix and a 24-byte normalized numeric node. Unused bank storage stays zero.

The first call constructs ordinary zero through the shared runtime. Later calls
validate the published graph, its exact owned node address, order, numeric shape,
range, clean tail and agreement with the compatibility cursor, then load the
actual ordinary Record. They do not box a new input from the native cursor.
The private native adapter receives cursor, store, owner and selected-code facts;
source receives only its ordinary Record. Stored pointers never authorize reads
outside the selected bank. Malformed input or exhausted version history fails
before source execution or preparation.

Source computes the same allocation/successor law as before. The adapter checks
both named fields and ordinary result-order suffix, then copies the semantic
successor into a complete independently owned graph in the inactive bank. The
published header/version and old graph remain unchanged during this preparation.
The two scalar results survive complete temporary-arena and retired-stack
erasure/readback. Only then does the owning process resource publish the new
version with its PID/cursor construction state. It consumes preparation state
and erases/reads back the old bank before further source calls or user execution.
Interrupts remain masked and there is one exclusive writer; SMP publication is
not qualified by this increment.

The store holds shared identity history, surviving child exec, child/parent
continuation restoration and calls to other selected policies. Context snapshots
preserve its explicit binding rather than rolling back the graph or version.
Terminal return erases/reads back the complete 4096-byte region and clears its
binding together with all selected code/arena bindings. The previous scalar-only
identity version has no state reserve or retained-store symbol, and keeps its
copied-input contract. Both versions occupy the same semantic slot and cannot
be selected simultaneously.

## Bounds and independent checks

The retained graph bound is **40 bytes** within each **2016-byte** bank, with
maximum published numeric width **31 bits**. The full state reserve is **4096**.
Temporary allocation remains bounded by **312 bytes**: **256** source evaluation,
**24** conservative adaptation and **32** arena header; maximum intermediate
operand width remains **32 bits**. Observed high-water is **56 bytes**. The full
actual stack bound is **760 bytes** within **4096**, including aggregate transport,
source/shared-runtime frames, returns, native saves and alignment. Measurements
supplement the conservative bounds.

The certificate separately charges state-region word work: bridge validation (**512** words), native
prepared-graph validation (**261**), commit validation (**6**), release validation
(**2**), initial cleanliness (**512**) and owner/code binding (**2**). It also
charges input Record transport (**2**), graph projection (**5**), preparation
metadata (**3**), commit metadata (**6**), old-bank erasure/readback (**504**) and
terminal erasure/readback (**1024**). These fixed bounds describe the private
selected layout and protocol, not a portable source ABI or general allocator.

The [independent C client](labs/qemu/x86_64/retained_identity_client.c) executes
**66541 transitions per result-field order**, **133082** total: exhaustive low
cursors, initialization/maximum/wrap boundaries and deterministic dispersed
cursors. It checks old graph preservation, independent new graph ownership,
survival after temporary cleanup and version alternation. **68** malformed
private-input/store cases reject before allocation or state mutation. Three
invalid source successors reject without changing a previously published graph:
zero, 2147483648 and a multiword value with a misleading legal low word. The C
client is a hosted qualification tool and is never linked into the kernel.

The ownership model exercises repeated alternation, wrong owners, overlap,
unconsumed preparations, premature commit, stale/double publication, retirement,
other-policy exclusion, epoch overflow and termination at five protocol phases.
Rejected model operations preserve the complete prior state. Frontend tests
require the exact state reserve and unique semantic slot. Boot tests reject
twelve missing, cross-policy or tampered retention-certificate cases, in addition
to previous identity/Record provenance checks. Linked-object inspection requires the complete aligned Data symbol in an
allocated, writable BSS section without the executable ELF section flag. The
selected region remains supervisor-owned through the existing machine mapping.

The [retained harness](labs/qemu/x86_64/retained_gate.py) and
[GDB inspector](labs/qemu/x86_64/retained_inspect.py) retain
[five physical observations](labs/qemu/x86_64/results/topal-retained-source-identity-gate.json).
Primary, reordered-result and retained-only configurations each complete nine
creations. All **27** observe source arguments/results and publication/retirement;
**24** source arguments point into an already published numeric graph and three
use ordinary zero initialization. Old graph/header preservation is checked before
and after source evaluation and after temporary cleanup. Actual child return/PID
observations and terminal reclamation follow. Reordering changes result order
indexes while native root/provider objects and named results stay consistent.

A controlled initializer fact supplies 2147483647 to the first ordinary numeric
constructor. Source returns `(2147483647,2)`, and observation stops after cleanup
with the published store still uninitialized. This is controlled wrap evidence,
not normally reached persistent history. A changed imported helper proposes
`(2,2)` instead of `(2,3)`; the independent oracle rejects it after cleanup and
before publication. Header/dispatch inspectors additionally compare the entire
4096-byte store across their invocations, and terminal inspectors check its erased
binding. The scalar identity harness now explicitly runs its five observations
as a compatibility control, separately from the retained primary kernel.

## Regression, review and remaining work

The refreshed cohort contains **59 successful primary physical runs on one
kernel**, **ten positive variant-kernel runs**, **three controlled precommit
observations** and **eleven expected-rejection runs**, totaling **83 retained VM
observations**. The scalar full-policy run is now a compatibility variant; the
retained full-policy run occupies its primary-cohort position. Variant counts
describe executed runs, not ten distinct binaries. All previous command,
environment, numeric/module/header/dispatch, clock/deadline, child, cooperative
pair/cancellation and pipe/substitution regressions pass on the retained kernel.

Validation passes the complete bounded serial workspace, formatting and
whitespace checks, **234 compiler / 126 semantic-model / 608 language tests**,
**28 toolchain plus four reference-VM contracts**, **261 unique normative systems
rules**, exact harness/inspector bindings, the same-kernel cohort and relative
qualification links. An older structural assertion incorrectly forbade an R11
scratch instruction throughout the dispatcher; it now checks the complete
credential handlers whose preservation it establishes. The focused assertion
and full workspace rerun pass; this correction changes no kernel code or hash.
The host wrapper retains serialization, host memory reserve and no-swap limits.

Risk is high at persistent alias/lifetime correctness, complete ordinary layout,
private calling conventions, exclusive publication, shared process history and
erasure. Single-agent review checks authoritative immutable/resource intent,
generated graph access, actual object/frame data, complete conservative bounds
and independent native/model/physical observations. Human review remains required
through the existing Step 4 PR. No third-party kernel implementation is added.

For a future Linux baseline, compare creation/reuse and process-history rules
with the source value law and exclusive owner protocol. Value-policy changes
belong in the source module; ownership, concurrency, external errors and lifetime
changes require corresponding systems and conformance qualification. Other
architectures must qualify private layout, calling convention, owned placement
and publication ordering while preserving ordinary value meaning.

General persistent allocation and effectful source subsystems, scalable processes
and scheduling, signal delivery, general/dynamic ELF, writable storage, networking,
namespaces/cgroups, distribution replacement, Docker/Podman and VM hosting remain
required by the [initial acceptance](step-4-status.md).
