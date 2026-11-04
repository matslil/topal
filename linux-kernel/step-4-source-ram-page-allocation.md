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

## Remaining live-kernel work

The live artifact/caller selection, actual x86-64 filesystem invocation and VM
qualification remain pending. The new systems artifact member is not yet
executable; it will be enabled with the actual controller binding. The model
and source-component stages do not add a kernel capability, change the preceding
qualified native artifact, or qualify general allocation.
The [initial acceptance ledger](step-4-status.md) remains in force.

Source selection will use the existing two-fact Boolean instance/frame ABI under
`ram-page-pool`, with separate control/arena/history and complete source/caller
bounds. The actual write path must preserve all recoverable admission and clock
ordering before source mutation, validate its exact map delta and publish the
selected actual backing slot. Release must prove zeroing, complete cleanup and
association retirement. Changed native code requires fresh affected machine
observations, including independent alternate-placement and refusal controls.

Risk is high at bitmap/association consistency, preparation versus publication,
source failure, zeroing, stale ownership, cross-domain views, bounds and shared
process lifetime. Current self-review covers approved systems/resource/effect
intent, three exact rules, actual model borrows and independent reference laws.
Live-kernel and complete physical reclamation evidence remain future obligations; no
independent reviewer result is claimed.
