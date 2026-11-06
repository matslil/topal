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

## Current file-position integration and owning caller

The third stage replaces the current
[ordinary codec](kernel/support/word-bytes.t) with one explicit eight-byte
little-endian store. Its public signature is unchanged. The ordinary
[file-position policy](kernel/policy/file-position.t) still computes the full
Nat sum and admits the signed file-offset range. Native code still completes
admission/transfer before lending the actual shared description field, and
requires the source Boolean and complete temporary cleanup before resuming the
owner. No native arithmetic or codec duplicates the ordinary source body.

The complete file-position allocation certificate is now **464 bytes**, down
from 60152: 320 source-evaluation bytes, 112 bridge bytes and a 32-byte header.
It retains one view and proves a 65-bit maximum numeric intermediate. The native
source graph has 21 functions and a 592-byte complete source-stack bound; the
owning caller totals **712 bytes**, including 88 live bytes and 32 alignment
bytes. The 65536-byte arena and 4096-byte retired-stack reserve remain unchanged
and are completely erased. A standalone observed allocation peak of 176 bytes
does not replace the conservative certificate.

All eight current-codec/compiler-API tests pass. Independent C calls retain the
168-case signed and full-u64 source controls and twelve ABI refusals; GDB sees
the actual addition/codec values, Boolean returns, control revocation and caller
state. Changed source fails the independent byte oracle. Exact source/runtime,
immutable data, code, costs and owning storage bind through native publication
and boot, including missing/wrong unsigned digests, orphan evidence and a
combined schema downgrade plus digest removal. Runtime presence is also coupled
to actual retained helper code, so coordinated metadata edits cannot disguise
unsigned code as a byte-only object.
Correcting the outer ELF hash cannot legitimize changed code or data.

The freshly executed
[file-position VM record](labs/qemu/x86_64/results/topal-file-position-region-gate.json)
contains thirteen observations: primary and lifetime runs, inactive-child
cancellation, five pre-entry refusals, two frozen callable contexts, wrong-codec
and false-action cases, and source-absent execution. Primary/lifetime check
360/411 syscall returns and 46 actual field calls each; cancellation checks 389
returns and 46 calls. The selected primary/lifetime kernel SHA-256 is
`2f2da9b4f308cac3e4bf5456408a72b70d68b17050d12129ff821cb9bb75cf69`.

GDB observes each actual unsigned primitive's view, full Nat facts, static
eight-byte/little-endian format, real owner-field address, Boolean and complete
64-byte description before/after. It independently validates that only the
position field changes. Existing source/helper/caller, frame, generation,
shared-description/syscall, post-return erasure and terminal reclamation checks
remain. The private LLVM result is `i1`; its undefined upper RAX bits are not a
Boolean value. The public adapter's complete u64 result is checked separately.
The debugger's changed-codec oracle stops before owner resumption. The native
controller halts on a subsequent false result after successful writes, preserving
those writes and erasing the complete temporary storage.
Controlled cases prove the actual stopped guest before disposal.

The earlier action and unscoped-frame controls execute the exact
[frozen bytewise fixture](labs/qemu/x86_64/fixtures/word-bytes-bytewise.t).
Their source schemas and omitted unsigned metadata retain their earlier
contracts; the record binds their exact module/fixture digest. The current
instance executes the current codec and unsigned schema `/4`. Both its frame
ABI/family and the native invocation binding `/3` remain unchanged.

Artifact/root/provider/object/boot/gate revisions are now
252/238/239/231/152/252. The first VM attempt failed debugger setup because the
hyphenated private symbol needed an address lookup; the corrected run passes
all thirteen cases. The focused evidence/publication test, source ceiling,
bounded formatting and strict whitespace checks pass. Other machine cohorts
and complete workspace qualification still require the fourth stage.

## Remaining stage and evidence scope

Complete workspace qualification and the remaining 145 affected VM observations
remain required. The current thirteen observations qualify only their own
changed inputs and dispositions. The earlier
[TK278 checkpoint](step-4-source-ram-page-allocation.md) retains its own complete
Rust and machine records; it does not qualify this new runtime or a changed
codec. Stages one/two preserved the selected kernel; the third changes the
selected codec and requires fresh execution throughout its affected cohort.

Risk is high at full-span preflight, unsigned/endian conversion, aliases,
caller exclusion, fact ordering and complete costs. Self-review checks the
existing resource/effect/scoped-region contracts and independent byte/refusal
oracles. No independent reviewer is claimed. The
[initial acceptance ledger](step-4-status.md) remains in force.
