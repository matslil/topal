# Step 4: ordinary reusable RAM-page allocation

`TK-DEC-278`; follows the [page-allocation design](design/source-ram-page-allocation.md)
and `TOPAL-SYSTEMS-REGION-PAGE-OWNER-001`,
`TOPAL-SYSTEMS-REGION-PAGE-ALLOCATE-001` and
`TOPAL-SYSTEMS-REGION-PAGE-RELEASE-001`.

## Portable ownership model

The portable model lends the filesystem owner's real eight-byte allocation map
through the existing scoped ordinary-region protocol. Normal file allocation,
truncate, truncate-on-open, final-unlinked destruction and terminal reclamation
use that same map. Each occupied bit agrees with a live object/logical-page
association; each free slot has clean metadata and a zero payload. Map bytes,
slot numbers and model labels do not grant native memory authority.

The pool remains exclusively borrowed while an action is live. Allocation
completion inspects actual helper-written bytes and accepts exactly one free
bit becoming occupied. It validates payload cleanliness before publishing the
admitted association. Release admission validates identity, slot, context and
generation before erasing a payload; after a valid one-bit release and complete
reclamation it clears the association. No source execution or Boolean result
is predicted by the model.

Page-map loans have their own nonwrapping history and owner identity, distinct
from position loans even when graph labels, pool identity and numeric generations
match. They retain the same conservative filesystem effect domain. False,
malformed delta, incomplete cleanup or abandonment poisons the owner; successful
stores and admitted payload erasure are not rolled back. Ordinary use remains
blocked until terminal teardown. Model cleanup flags are explicit obligations,
not physical erasure evidence.

The [model](../src/topal-semantics/src/systems_ram_page_actions.rs) and
[tests](../src/topal-semantics/src/systems_ram_page_action_tests.rs) cover
all 256 canonical occupancy patterns with first/last free-slot effects, every
release slot, clean reuse, aliases and persistent history. They reject malformed
maps/associations, invalid ranges, foreign tokens, stale/cross-domain views,
exhausted generations, incomplete five-part reclamation, wrong/multiple deltas
and false-after-store completion. Existing normal VFS/positional/named-file
models retain their behavior and check shared-map consistency. Testing exposed
one missed bitmap clear in truncate-on-open; that path now updates and erases
the same resource as ordinary truncation/destruction.

All 181 semantics unit tests and 621 language unit tests pass, including eight
new page-action tests. Workspace all-target checks and formatting pass. The
existing source-size ceiling passes; model and test files remain below 2000
lines. The systems specification now has 300 unique rule IDs. Further stage
checks and traceability are recorded with their actual results as work proceeds.

## Ordinary source and native component qualification

The [entry](kernel/memory/ram-page-pool.t) and
[ordinary helpers](kernel/support/page-slots.t) now compile through the existing
instance interface with the declared two facts and actual Boolean result. They
use ordinary checked byte access and an acyclic function graph. A first-fit
allocator changes one free map byte; release validates its slot and clears one
occupied byte. Extent, unknown-operation and malformed full-width fact refusals
preserve source-owned bytes. This adds no core syntax or allocator meaning.

The complete source-object certificate proves 488 temporary-allocation bytes,
1152 source-stack bytes, 49 native functions and one view node, within the
existing 65536/4096-byte limits and unchanged frame ABI/family. These are source
bounds; the live kernel caller's complete combined proof remains required.

The [workspace integration](tools/toolchain-gate-builder/tests/ram_page_pool.rs)
builds the actual first-fit source and a separately generated last-fit module
snapshot. An [independent C oracle](labs/qemu/x86_64/ram_page_pool_client.c) executes
every occupancy mask and release slot plus malformed full-width operation/slot
facts whose low bits can resemble valid requests, and wrong extents. Both
variants pass 4646 calls and 32 native ABI pre-entry refusals in total. Guarded
read-only input frames prevent writes and overreads; the caller erases/readbacks
its complete input staging and arena. Source-object context/count/instance,
undercharged bounds, present-null fields and changed-helper replay account for
eleven additional metadata refusals.

[GDB](labs/qemu/x86_64/ram_page_pool_debug.py) observes 110 sampled calls across
the two variants: actual ordinary parameters, imported helper specializations,
opaque view identity, map bytes, Boolean returns, immutable frame values,
callee GPR/SIMD preservation and RSP. Sampled source/helper depth reaches 936
bytes and the observed allocation peak is 440 bytes, within the complete static
bounds; those observations do not replace the certificates. Breakpoints on the
actual owner-byte store primitive observe exactly one store per successful
sampled action and none for any sampled false result (38 stores in total).

The [component record](labs/qemu/x86_64/results/topal-source-ram-page-pool-component.json)
binds source, helper, test, C and GDB identities and both source objects. It
explicitly qualifies source-object execution only. Live kernel ownership,
kernel dispatch and complete retired-stack/control/history reclamation are not
proved by this host component test. Existing root/position objects retain their
separate owning callers and contracts.

## Live-kernel implementation and pending qualification

The [x86-64 controller](../src/topal-compiler/src/systems_ram_page_region.rs)
and `ram-page-pool-library` artifact selection are now
implemented and selected in the primary root. The controller lends the actual
eight-byte map at filesystem offset 24, with independent arena/control/history.
It preserves all fourteen incoming non-result GPRs. Its 184-byte live frame and
64-byte complete call/alignment allowance combine with the 1152-byte source
bound to give a 1400-byte invocation certificate. Enclosing native lifetime
frames are separately included in the filesystem's 328-byte native helper bound:
five enclosing return addresses and the lifetime helper's five register saves.
The 65536/4096-byte source resource limits and frame ABI/family remain unchanged.

Missing-page writes count capacity without choosing a slot before the fallible
clock. After copying the complete admitted input prefix, they call ordinary
source and derive the actual slot from its exact whole-map delta. Clean
association/payload checks precede publication. Resident writes issue no page
callback. Full release validates owner/object/map/context/resources/generation
before zeroing/readback, then clears the real map through source and erases the
association after complete cleanup. Partial truncation retains the bit.
The complete source input/result frame is erased before requiring true or
validating the map delta. Independent owner preparation facts survive only as
controller scalars; views, arena, control and the retired source stack do not.

Boot packaging checks all three independent source namespaces and their actual
relocated code, immutable data, disjoint storage, module/input/output identities,
resource certificates and complete native templates. File-position and page
invocations share the verification algorithm with explicit caller templates;
their runtime resources and owning bindings remain separate. Source absence
retains the native allocator and old filesystem metadata omissions.

The first primary smoke observation passed 360 syscall checks and 42 real page
actions (21 allocations and 21 releases). A subsequent exploratory suite
collected successful primary/lifetime/last-fit observations, three admitted
source-failure dispositions and ten allocation/release pre-entry refusals. Its
aggregate command failed because raw serial bytes were not JSON serializable;
the reporting error is corrected. An earlier lifetime observation also exposed
a debugger assumption that a resumed parent fork repeats the syscall instruction;
the observer now handles the saved continuation explicitly.

The final [RAM-page suite](labs/qemu/x86_64/results/topal-ram-page-region-gate.json)
now passes all 21 VM dispositions on the selected primary revision: primary,
lifetime and last-fit execution; cancellation with a live inactive child; six
admitted bad-source allocation/release cases; ten pre-entry refusals; and a
native-only compatibility run. The primary executes 360 syscall checks and 42
page actions; lifetime executes 411 and 50. The cancellation run checks complete
reclamation after 46 actions. GDB observes the actual four-frame ordinary source,
adapter, native controller and service call chain, saved return PC, all fourteen
GPRs, source facts/result, exact map/association/payload changes, and physical
arena/control/retired-stack/input-frame erasure. Controlled failures prove the
stopped guest before disposal. The publication contract test passes.

The [public-API binding integration](tools/toolchain-gate-builder/tests/ram_page_binding.rs)
also passes exact code/storage/data/bounds tampering, rebound outer-kernel-digest
code mutation, present-null, orphan snapshot and missing source refusals. Native
source absence publishes the old allocator/profile omissions, contains no page
instance symbols and passes boot adaptation.

Primary kernel SHA-256 is
`efebdfa319455a1374ce1996ce69c5ec78bf8509930c837a23ce71b0d7c7374f`.
Artifact/root/provider/object/boot/gate revisions are 251/237/238/230/151/251.
This qualifies the bounded RAM-page owner; it does not yet qualify the complete
affected preceding VFS/BusyBox/source cohort. That fresh cohort and complete
workspace qualification remain the final stage.

Two bounded compiler unit-test builds were killed by their cgroup at 723/816 MiB;
a later build was stopped after prolonged reclaim pressure at its 601 MiB cap.
The host reserve and zero-swap policy were retained. Sharing the verifier, moving
the binding test to a small public-API integration target and reducing host
allocator arenas allowed compilation and binding qualification to complete.
Compiler-library checking and the strict source ceiling pass. Remaining workspace
and affected-cohort results will be recorded when actually complete.
The [initial acceptance ledger](step-4-status.md) remains in force. General
allocation, distribution replacement, containers and VM hosting remain unfinished.

Risk is high at bitmap/association consistency, preparation versus publication,
source failure, zeroing, stale ownership, cross-domain views, bounds and shared
process lifetime. Current self-review covers approved systems/resource/effect
intent, three exact rules, actual model borrows and independent reference laws.
Complete workspace and affected-cohort qualification remain outstanding;
no independent reviewer result is claimed.
