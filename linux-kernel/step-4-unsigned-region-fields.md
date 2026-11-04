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
whitespace checks pass. This is the first definition/model stage; complete
runtime and affected-machine qualification belongs to later stages.

## Remaining stages and evidence scope

Source member selection, native lowering, complete reviewed resource/runtime
certificates, independent C/GDB components, current-codec integration and fresh
affected VM qualification remain required. The new source operations are not
yet executable-qualified. The earlier
[TK278 checkpoint](step-4-source-ram-page-allocation.md) retains its own complete
Rust and machine records; it does not qualify this new runtime or a changed
codec. The selected kernel and existing byte primitives are unchanged at this
model stage.

Risk is high at full-span preflight, unsigned/endian conversion, aliases,
caller exclusion, fact ordering and complete costs. Self-review checks the
existing resource/effect/scoped-region contracts and independent byte/refusal
oracles. No independent reviewer is claimed. The
[initial acceptance ledger](step-4-status.md) remains in force.
