# Natural results from owned metadata traversal

This continues TK-DEC-282 Stage 3 under the approved systems-only direction.
Allocator source needs to return a selected position or other natural fact from
its actual metadata. The existing Boolean root interface expresses a checked
owner postcondition and must retain that meaning.

Add a distinct source-only Nat-result selection and an owned native constructor
for region/three-Nat inputs, with the first Nat as the countdown. Existing shared
tail lowering/runtime already represent natural results; the new constructor
must retain their complete representation rather than turn a result into a word
or a Boolean. Its schema must identify the new qualification. Neither constructor
silently accepts the other's result profile. Source gains no new type, syntax,
address, pointer, instruction or authority constructor.

Admission retains the actual nonempty exclusive read-write region, immutable
word-bounded facts, persistent nonwrapping history, disjoint private/output
domains and existing 65536/4096 reserves. Normal completion copies the entire
natural result into the existing 56-byte output, then retires the invocation.
Failure preserves performed metadata writes/issuance and reaches trusted fatal
cleanup. Native result extraction must first validate the canonical sign, limb
count and complete value; it cannot discard high limbs to obtain an index.

A natural index remains a fact. A future allocator caller must additionally
bind actual metadata ownership, request/result meaning, normalized frame ranges,
reservations/provenance, whole-extent identity and zero-before-republication.
Ordinary source chooses free spans; the provider checks authority and performs
physical transitions. It must distinguish normal exhaustion with unchanged
allocator state from a protocol failure after metadata writes. General dynamic
requests and multiple live extents remain required.

This component is not a new root source-library interface or artifact grant.
The current typed root remains Boolean with its existing 64-byte owner and
true-result gate. Linking multiple traversal components later requires distinct
closed code identities and independently owned domains; a symbol collision or
shared arena is not permission to combine them. Kernel owning-stack, interrupt,
recovery, complete retirement and boot binding require their own implementation
and evidence before publication. No AArch64/RISC-V execution is implied.

The initial ordinary query fixture finds a zero byte and returns the countdown
value at that position. For a validated interval the caller derives the position
as the original requested end minus that value. Zero means no match; the
all-ones word identifies an invalid interval, outside an admitted query's result
domain. Carrying this word-bounded fact preserves the existing reset invariant
while the cursor retains its separate affine potential. The byte query is a
mechanism fixture; aligned contiguous allocation and provenance remain required.
