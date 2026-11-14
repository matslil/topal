# Step 4: complete linked finite/numeric policy inputs

`TK-DEC-274`; implements the [object-owned input design](design/policy-linked-inputs.md)
and `TOPAL-SYSTEMS-POLICY-LINKED-INPUTS-001`. Step 4 remains in progress;
distribution replacement, Docker/Podman and VM hosting remain unqualified.

## Confirmed gap and correction

The preceding [region-input qualification](step-4-region-immutable-data.md)
binds Unit/action inputs and explicitly leaves other profiles for separate
audits. The finite/numeric implementation checks source/object provenance,
function presence/placement and resource evidence but omits a complete binding
of actual relocated instructions and immutable data.

Independent changes to the earlier artifact's ordinary deadline literal 10
(changed to 11) and a numeric helper's first instruction byte both pass its
boot packager when only the outer kernel digest is corrected. Source snapshots,
symbols and inner bindings remain unchanged. The instruction-mutated image has
SHA-256 `e28652ec02a2902d19f13ed218cc5682101c8831108a5f2de0a0fb5353aadbc4`.
Neither altered image is claimed to execute successfully. The literal is
independently outside the already bound Unit/action extents.

The new publisher binds the complete allocated code/read-only contribution of
each compiler-owned finite/numeric policy object. Boot packaging repeats the
inspection and rejects changed actual bytes under unchanged inner bindings.
This covers the finite, time, image, dispatch and process-identity slots.
Finite immutable inputs include an actual relocated jump table; a numeric-only
literal list would not establish this complete closure.

## Object and linked evidence

Each complete ordered section manifest records its original name, private
slot-qualified marker identity, exact bytes/alignment, executable role and
unlinked byte digest. Preparation rejects writable/TLS and unsupported allocated
sections. A verified object-copy transformation retains all payload bytes and
relocation semantics, supplies private begin/end markers and prevents immutable
section merging from fragmenting those extents. The pinned LLVM 22.1.6 requires
adding hidden markers and then localizing them in a separate pass to preserve valid local/
global ELF symbol ordering. Only the verified final object is linked/published.

The linked binding records the transformed object digest, complete manifest
digest, actual relocated bytes and exact marker-delimited addresses/extents.
Inspection requires local marker identity, complete namespace closure, correct
roles/alignment and disjoint extents within the supervisor envelope. Slot absence
requires marker absence. Boot also ties each object identity to exactly one
matching published object output. Missing, duplicate, reordered, wrong-slot,
wrong-object and changed-byte records reject; atomic refusal leaves no new
destination directory.

Private finite/numeric metadata uses schema 3/9 and linked-input schema 1.
Source signatures, native entry interfaces, stack/allocation certificates,
retained-state lifetimes and complete cleanup keep their existing meanings.
Unit object schema 3, root binding schema 2 and action schema/entry ABI 1 remain
separate and mandatory. No new source primitive, core-language meaning, Linux
interface or host-security change is introduced. This is a consistency/provenance
binding, not authentication against replacement of all source and inner evidence.

## Validation

Current artifact/root/provider/object/boot/gate revisions are
248/234/235/227/148/248. The fresh primary kernel SHA-256 is
`9e1560c2bdef7791f4f6fc1101fa872d8c059070a6d89b2da1ade2bce5c92206`. Full workspace, all-target and formatting checks pass;
the complete physical regression and evidence checks also pass.

The primary policy inputs retain these complete section contributions:

| Slot | Executable sections | Read-only data sections |
| --- | ---: | ---: |
| finite file-open | 1 | 1 |
| deadline | 41 | 2 |
| image header | 68 | 2 |
| syscall dispatch | 191 | 3 |
| structured process identity | 19 | 2 |

These are 330 complete object-owned sections, including 320 executable sections
and ten data sections; a finite executable section can contain several functions.
They are separate from the existing Unit/action region bindings.

The [public integration](tools/toolchain-gate-builder/tests/policy_inputs.rs)
uses an independent ELF walker to alter actual code and read-only bytes for
every selected slot, including the finite table. It independently locates the
actual Nat 10, corrects only the outer digest and requires the changed-policy-
binding refusal. Marker names, binding, endpoint/duplicate and section-rights
mutations exercise actual ELF inputs. Eighteen metadata cases per slot include
complete manifest digests, source-section identity, old schemas, object/output
coupling, missing/extra records and cross-slot names. Actual changed-byte
publication refusal must preserve atomicity.

The compiler object test accepts a valid code-only input and rejects writable,
TLS and preexisting reserved marker inputs. The Python evidence contract checks
all five complete manifests, relocated bindings, manifest/object correspondence,
section roles/alignments, nonoverlap and actual relocated finite data.

Fresh execution also exposed an older controlled-identity harness race: result
publication triggered VM teardown before GDB completed its final command. An
independent run with a deliberate 0.2-second debugger delay reproduces the broken
pipe. Scalar/retained controls now disconnect without resuming the guest, wait
for successful debugger exit and use QMP to confirm `running: false` and
`status: debug` before teardown. Every such control exercises that delay. The
shared QMP helper is bound by digest into scalar, retained, structured and
persistent-action records; affected physical cases are requalified. These are
test-harness changes, with the already tested kernel implementation unchanged.

Fresh bounded validation passes the complete Rust workspace, including 620
language, 252 compiler unit, 261 CLI and 173 semantics tests, eight action
integrations, two Unit immutable-data integrations and the new policy-input
integration, plus the remaining suites. All workspace targets, workspace
formatting and explicit included-source formatting pass with the existing
resource limits and host reserve.

The completed QEMU cohort passes all 133 observations, including 40 unmodified
BusyBox workloads, seven root-region and eleven persistent-action cases. The
general positive cohort contains 70 observations sharing the primary kernel
and 21 independently identified positive variants. Other cases retain their
separate execution, controlled-condition and expected-refusal oracles.
All 39 Python lab contracts pass against the fully refreshed records, including
complete policy manifests and successful debugger completion with actual stopped
guests. Checks confirm 292 unique normative rules, relative links, exact
code/data and manifest correspondence, retained ABIs/resource bounds and
whitespace. The failed harness run is superseded by fresh successful affected
cases and the remainder of the unchanged kernel cohort.

Risk is high at object/link closure, relocation preservation, section placement,
private-format compatibility and existing privileged caller/resource boundaries.
Self-review follows the authoritative immutable-code, complete source/runtime,
ABI and resource intent, the independent byte counterexamples, complete
transformation/marker checks, strict boot refusals and physical execution/cleanup.
Human PR review remains required; no independent reviewer result is claimed.
The [status ledger](step-4-status.md) retains all remaining initial-plan work.
