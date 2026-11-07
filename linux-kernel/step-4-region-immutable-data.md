# Step 4: complete immutable inputs for region source

`TK-DEC-273`; implements the [immutable-input design](design/region-immutable-data-bindings.md)
and `TOPAL-SYSTEMS-REGION-READONLY-DATA-001`, completing the existing exact
linked code/data obligation of `TOPAL-SYSTEMS-REGION-ROOT-BINDING-001`.
Step 4 remains in progress; distribution replacement, containers and VM hosting
remain unqualified.

## Confirmed defect and correction

The earlier [root-owner qualification](step-4-root-region-invocation.md) binds
ordinary source/runtime instructions, native templates and writable ownership.
It does not independently bind the actual linked immutable literal inputs.
An object digest and instruction hashes do not establish that later linked
constant bytes still match their source object.

An independent disassembly of the earlier kernel identifies the root entry's
destination Nat literal at `0x2003d0`. Changing its limb from 1 to 2, while
preserving source snapshots, instructions, symbols and inner bindings and
correcting only the outer kernel checksum, is accepted by the earlier boot
packager. Pinned QEMU/GDB observes the ordinary helper copying to byte 2 and
the resumed owner's byte-1 guard refusing. The changed image's SHA-256 is
`bfca7fef1777799efeb61755b67f8bd33d1dec342d9e1377ca746f2343056ce5`.
This demonstrates an actual execution consequence in the isolated VM, with
unchanged source and instruction bindings. It does not claim a successful
normal kernel qualification for that deliberately altered image.

The new publisher and boot inspector independently bind those immutable
inputs and reject that class of changed literal before boot. The
[persistent action profile](step-4-persistent-region-actions.md) already has
this protection; Unit and action source objects now share one implementation.
This increment makes no blanket immutable-data qualification claim about the
other finite/numeric source-library profiles, which need their own audits.

## Preserved contracts and complete data coverage

The Unit object uses private source-object schema version 3 and root invocation
binding version 2. The published `(KernelByteRegion) -> Unit` signature, native
entry ABI version 1, root declaration, allocation/fatal hooks, ordinary source
functions, effects, ownership and resource budgets retain their meanings.
The separate action object retains its schema and scalar/Boolean ABI. No core
value, Memory, Layout, allocation or Linux interface changes.

Compiler-owned immutable LLVM globals retain local names instead of temporary
assembler labels. Unit literals use the Unit namespace; source/debug strings
remain unchanged. Actual ELF data records carry each definition's exact local
identity, nonzero extent, alignment and byte digest. Complete object inspection
rejects nonzero allocated immutable bytes outside that binding, unsupported
data relocations, foreign/ambiguous identities and incompatible placement.

The linked inspector compares the actual read-only, nonexecutable, non-TLS
definitions and bytes, with exact extent/alignment/local visibility. Both
profiles compare their complete private data-name sets. Unit inspection
explicitly excludes the nested action namespace in a combined executable.
Missing/extra, duplicate, reordered or changed records reject. Immutable
definitions may share addresses when every independently bound extent still
contains its expected bytes; mutable owners retain their disjointness rules.

A Unit body which consumes no immutable input has a valid empty data binding.
The compiler proves that absence from the actual source object and linked
namespace; it introduces no dummy source constant. Reentrant ownership,
generation, arena, stack, view revocation and physical erasure contracts remain
separate and unchanged.

## Validation

The current primary kernel SHA-256 is
`782152760ffaa32e060cc987d77090dd67004cef5bc00f0ab83102aba447f3d2`.
Artifact/root/provider/object/boot/gate revisions are 247/233/234/226/147/247.
The primary binds 3 Unit immutable definitions alongside its 17 source/runtime functions.
The root still proves 136 arena bytes and 496 source-stack bytes plus 32 caller
bytes (528 combined). The persistent action still proves 60152 arena bytes
and 1976 combined stack bytes; its entry ABI, 54 functions and 22 data bindings
remain separate.

The [public integration tests](tools/toolchain-gate-builder/tests/region_data.rs)
qualify constant-bearing and valid zero-data Unit graphs, unchanged entry ABI/
signature/hooks, complete object replay, metadata mutation refusal and coexistence
with the action profile. The independent ELF walker changes only the actual
destination literal, corrects the outer image digest, and requires the exact
changed-immutable-bytes diagnostic. Atomic boot publication must leave no
destination on that refusal. A separate actual-section flag mutation checks
that read-only TLS is rejected. Twelve metadata cases cover missing source/
linked records, changed digests, size/alignment, the foreign action namespace,
order/duplicates, old incomplete schemas and wrong linked addresses.

Fresh bounded validation passes the full Rust workspace (including 620 language,
251 compiler unit, 261 CLI and 173 semantics tests, eight action integrations and
both new immutable-data integrations), all workspace targets and formatting.
Explicit formatting checks also cover included compiler sources.

The complete QEMU cohort passes 133 observations, including all seven root
region cases, eleven persistent-action cases and 40 unmodified BusyBox
workloads. The [root evidence](labs/qemu/x86_64/results/topal-root-region-gate.json)
records the complete Unit data binding alongside the independent execution
oracles. The [action evidence](labs/qemu/x86_64/results/topal-file-position-region-gate.json)
retains actual persistent-field updates, caller preservation and complete
temporary/controller/history cleanup. Within the general positive cohort,
70 observations share the primary kernel and 21 positive variants retain
distinct identities. Additional cases retain their separate execution,
controlled-condition and expected-refusal oracles.

All 37 Python lab contracts pass against the fully refreshed records. Checks
also confirm 291 unique normative systems rules, linked code/data correspondence,
preserved entry ABIs and resource bounds, relative links and whitespace.

Risk is high at source/object/linked closure, namespace ownership, immutable
aliasing and private-format compatibility. The implementing agent reviews the
authoritative immutable-value and exact linked-input intent, object coverage,
scope separation, actual literal counterexample, old ABI preservation, complete
resource proofs and independent physical execution/cleanup. Human PR review
remains required; no independent reviewer result is claimed.
The [status ledger](step-4-status.md) retains the initial-plan acceptance work.
