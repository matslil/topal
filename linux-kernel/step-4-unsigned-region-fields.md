# Step 4: checked unsigned region fields

`TK-DEC-279`; follows the [systems design](design/unsigned-region-fields.md),
`TOPAL-SYSTEMS-REGION-UNSIGNED-001` and
`TOPAL-SYSTEMS-REGION-UNSIGNED-QUALIFY-001`.

## Definition and portable model

The model adds explicit one-to-eight-byte little/big-endian encodings over the
existing actual borrowed owner. Width includes odd byte counts; it describes
data, independently of architecture word size. A validated format carries no
address, view or memory authority. Arbitrary-precision offset/value facts reject
before conversion when outside the complete field range. A successful store
preserves surrounding bytes, and load returns the exact unsigned Nat.

The [implementation](../src/topal-semantics/src/systems_region_unsigned.rs)
uses the same owner, view, window, frame, generation and effect identity as
existing byte access. Whole-field admission precedes all payload operations.
Readonly/bounds/value refusals preserve the complete model invocation, including
payload, observations and lifetime counters. Existing controller completion
guards retain responsibility for application postconditions, actual Boolean
acceptance and temporary reclamation; a valid field encoding cannot discharge
those obligations.

All 190 semantics unit tests pass, including nine
[unsigned-field tests](../src/topal-semantics/src/systems_region_unsigned_tests.rs).
Independent standard integer encodings check every width/order and every
contained offset in a sixteen-byte owner, every bit position, unsigned maxima,
distinctive patterns and all one-byte values. Complete-state refusal controls
include narrow/empty windows, readonly aliases, overflow and facts at `2^256`
and `2^512`.
Successful model operations also record exactly the selected payload-byte
observations: stores contain no hidden payload read, and loads contain no write
or access outside their field.
Nested helpers preserve shared effects, suspension, revocation and generation
checks. Actual file-position loans update both description aliases; independent
page pools with equal human-readable labels retain distinct authority. Tests
also prove no rollback after cancellation or failed application completion.

The reference model uses the workspace's already locked `num-bigint` package
for mathematical facts; it adds no freestanding Rust runtime to the kernel.
It is not source execution, native authority or physical erasure evidence.
No new core value, Memory, resource, effect or destruction meaning is introduced.

The workspace all-target check passes, along with all 622 language unit tests
and four specification/requirement coverage checks. The systems ledger has 302
unique rules. Bounded formatting, the unchanged 2000-line Rust ceiling and strict
whitespace checks pass. These checks qualified the first definition/model
stage. The following component evidence extends that checkpoint; actual owning
callers and affected machines require later qualification.

## Typed source and independent native components

The second stage adds the qualified source members, static format validation,
owner-relative Read/Write inference through ordinary helpers and native lowering.
Runtime fact expressions execute in lexical order before assignment to native
argument roles. Five new language tests cover every format, rejected facts,
arbitrary Nat inputs, false empty-effect claims and runtime selection from
lowered but statically unselected branches. The latter keeps runtime input
selection separate from an allocation path's reachability.

The separate [unsigned runtime](../src/topal-compiler/src/runtime/kernel_region_unsigned.ll)
retains the existing view/control guard and full Nat decoder. Its digest is
`d4671003ce1b40ec4b6e732815267d193ab1851980c8405ffdc852d69784a5c4`.
Complete interval, rights and value checks precede payload access. Every byte
loop starts at zero and terminates at the checked width in `[1,8]`; shifts stay
in `[0,56]`, including width eight. Span/store allocate nothing. A load allocates
at most 24 bytes for the full unsigned integer and 16 for its Optional; refusal
uses the existing 16-byte None. Shared guard/conversion/value runtime inputs
retain their independent digest gates. These reviewed algorithms justify the
40-byte load and zero-byte store primitive costs, rather than measured peaks.

The source certificate includes argument evaluation, input bridge, header,
views and the complete native call graph. The independent dispatch fixture
needs 656 temporary bytes and 1136 source-stack bytes for all sixteen formats,
within the unchanged 65536/4096 limits. Those fixture bounds do not certify the
later real file-position caller.

Unsigned source objects use Unit `/4`, action `/2`, unscoped frame `/3` and
instance frame `/4` schemas. Their allocation records must contain the exact
`unsigned_runtime_sha256`; byte-only objects retain their earlier schemas and
omit that field. Native identity checks reject missing, orphan or wrong runtime
evidence and profile/schema mismatches; deserialization rejects present null.
Complete reproduction binds source/modules, LLVM, retained code, objects, data
and costs. Mutating any input or undercharging the allocation/stack refuses.
The existing native entry ABI and frame adapter family remain unchanged.

Five new compiler tests execute an independent
[C oracle](labs/qemu/x86_64/region_unsigned_client.c),
[ordinary source fixture](labs/qemu/x86_64/region_unsigned_probe.t) and
[debugger oracle](labs/qemu/x86_64/region_unsigned_debug.py):

- All sixteen width/order combinations execute 92622 calls. An independent
  division-based encoding oracle checks every offset in a 23-byte owner, every
  bit, maxima and distinctive patterns, with complete-page canaries and guard
  pages on both sides. Loads also execute through read-only root views over
  physically read-only payload pages.
- Valid controls and frames accompany inaccessible payload pages for empty,
  narrow, readonly and out-of-range refusals. Both 65-bit and 256-bit actual
  source computations must refuse before payload access. Input frames occupy
  a read-only page between inaccessible pages.
- For each byte order, GDB observes 30 actual source calls and all generated
  helper specializations, full numeric values, actual Boolean returns,
  callee-preserved GPRs, all sixteen XMM registers, restored RSP, revoked control
  and unchanged input frames. Hardware access watches distinguish exactly eight
  payload stores from eight loads; stores contain no hidden payload reads.
  The caller then physically erases its complete arena and input frame, with
  30 independent debugger readbacks per order.
- Earlier byte-only/profile controls remain; an effectful argument-order test
  executes the existing independent caller oracle. Separate Unit, action,
  unscoped and instance contexts bind exactly their selected runtime and
  immutable data, including an unselected unsigned branch.

Initial component attempts rejected invalid fixture syntax. The debugger's
first attempt observed only one helper specialization; it now registers every
generated specialization. These failed validations were corrected in the
qualification fixtures without changing core syntax or the runtime algorithms.

The complete compiler unit suite passes all 259 tests with no failures or ignored
tests, including the five new component/certificate tests and existing byte-only
source, native helper, profile, binding and kernel publication controls. All 627
language unit tests and four coverage/traceability checks also pass. Bounded
workspace formatting, the unchanged 2000-line Rust ceiling, strict whitespace
and all 1004 local links in the changed reports/traceability files pass. The
existing cgroup wrapper retains its normal host reserve and zero swap; no host
security or memory policy was changed.

## Remaining stages and evidence scope

Current-codec integration, actual owning caller/boot binding and fresh affected
VM qualification remain required. Source/native component evidence grants no
new live kernel loan. The earlier
[TK278 checkpoint](step-4-source-ram-page-allocation.md) retains its own complete
Rust and machine records; it does not qualify this new runtime or a changed
codec. The selected kernel and existing byte primitives remain unchanged through
this component stage.

Risk is high at full-span preflight, unsigned/endian conversion, aliases,
caller exclusion, fact ordering and complete costs. Self-review checks the
existing resource/effect/scoped-region contracts and independent byte/refusal
oracles. No independent reviewer is claimed. The
[initial acceptance ledger](step-4-status.md) remains in force.
