# Immutable inputs in region source bindings

`TK-DEC-273`; implementation hardening of the existing exact linked code/data
obligation in `TOPAL-SYSTEMS-REGION-ROOT-BINDING-001`. The standing systems-only
authorization applies. Ordinary values, function signatures, effects, resource
authority and Linux interfaces retain their meanings.

## Confirmed gap

The [root invocation design](root-borrowed-region-invocation.md) requires exact
linked code/data binding. Its first implementation binds relocated source and
runtime instructions, native templates and owned writable storage, but omits
the actual immutable literal bytes in the linked executable. An unlinked object
digest does not close that gap after linking.

An independent disassembly identifies the root entry's immutable Nat literal
for destination byte 1. Changing only its limb from 1 to 2, then correcting the
outer kernel checksum, leaves source snapshots, instructions, symbols and all
inner bindings unchanged. The earlier boot packager accepted that artifact.
Pinned QEMU/GDB then observes the ordinary helper copying to byte 2 and the
resumed owner's byte-1 guard refusing. This is a confirmed qualification gap,
with a functional consequence, rather than a proposed change to source meaning.

The [persistent action profile](persistent-region-actions.md) already binds its
named immutable data independently of instruction hashes. Bring Unit region
objects and their root integration to that same standard. Other source-library
profiles need separate inspection; this increment makes no blanket claim about
their immutable inputs.

## Representation and inspection

Reuse one region-data inspection implementation with an explicit Unit/action
profile discriminator. Retain compiler-owned immutable LLVM globals as named
local definitions in each object's private namespace. Give Unit literal globals
their own qualified names. Preserve source/debug strings and ordinary source
function names; change no value bytes or writable state. The Unit namespace
must exclude the nested action namespace when inspecting a combined image.

Object inspection records exact local symbol identity, nonzero extent,
alignment and byte digest. Require allocated read-only, nonexecutable storage;
reject writable/TLS/foreign data, unsupported relocations, ambiguous names,
unplaced/truncated extents and nonzero allocated data outside the complete
binding. A zero-data Unit graph is valid when inspection proves it has no
immutable input; it must not acquire a dummy semantic constant just to satisfy
the metadata format.

Linked inspection requires every expected symbol exactly once, with matching
kind/local visibility, extent, alignment, read-only placement and bytes. Refuse
missing/extra qualified data and incompatible records. Immutable merging is
permitted when every independently bound symbol still names exactly its
expected bytes; address disjointness is required for mutable owners, not for
equivalent immutable constants.

Keep the published `(KernelByteRegion) -> Unit` entry, native entry ABI, root
declaration, hooks and allocation/stack/view laws unchanged. Advance the private
Unit source-object schema to version 3 and the root invocation binding schema
to version 2 so old incomplete metadata cannot masquerade as the new complete
binding. Preserve the action object's current schema and its existing public
inspection entry point; both profiles share the hardened implementation.
Advance current artifact/provider/boot qualification identities as usual.

Root publication records relocated immutable data alongside source instructions
and exact native/storage/cost records. Boot packaging repeats the complete
inspection before accepting the payload. A changed literal with only a corrected
outer checksum must reject. Empty, reordered, duplicate, foreign-profile,
wrong-size/alignment/visibility/placement and changed-data records must diagnose.
No fallback to the old incomplete binding is permitted in a newly published
artifact.

## Alternatives and compatibility

Hashing instructions alone is insufficient, as the counterexample demonstrates.
Hashing the entire final read-only segment would conflate unrelated libraries,
linker-owned material and placement; it would not establish each source object's
closed immutable input set. Recompiling every source during boot packaging
would add an unnecessary toolchain dependency to that boundary. Named local
data with complete object coverage and exact linked comparison extends the
already qualified action mechanism and retains object-level provenance.

This is a systems artifact-format refinement. It introduces neither a source
constructor nor native addresses, new core Memory/Layout or integer semantics,
extra privilege, a Linux ABI change or a new application interface. AArch64 and
RISC-V retain the same immutable-input obligation, with separately qualified
object format, relocation, placement and ABI evidence.

## Qualification and review

Reproduce and retain the corrected-outer-checksum counterexample as a rejection
test for the fixed publisher/boot boundary. Test Unit objects with actual numeric
literals and valid zero-data bodies, exact source replay, Unit/action coexistence,
readonly-data mutations independent of text and metadata/namespace failures.
Keep independent execution oracles: changing a correctly rebuilt ordinary source
helper remains a semantic variant, whereas changing linked data beneath an
unchanged binding must fail packaging.

Requalify the actual root loan, existing persistent field actions, caller and
temporary/history cleanup, complete resource proofs, all source-policy controls
and the full current VM/BusyBox cohort. Update historical report scope where its
first binding did not cover this linked input, and connect current evidence to
the existing root/source requirements plus the precise immutable-data rule.

Risk is high at namespace closure, immutable aliasing, linked provenance and
format compatibility. Review higher-level immutable-value and systems artifact
intent, object-to-linked coverage, old entry ABI preservation, refusal paths,
all declared bounds and independent source/physical evidence. Human PR review
remains required; this increment stays within the existing Step 4 PR.
