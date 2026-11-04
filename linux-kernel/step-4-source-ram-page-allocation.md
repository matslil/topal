# Step 4: ordinary reusable RAM-page allocation

`TK-DEC-278`; follows the [page-allocation design](design/source-ram-page-allocation.md)
and `TOPAL-SYSTEMS-REGION-PAGE-OWNER-001`,
`TOPAL-SYSTEMS-REGION-PAGE-ALLOCATE-001` and
`TOPAL-SYSTEMS-REGION-PAGE-RELEASE-001`.

## Current stage: portable ownership model

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

## Remaining source and native work

The ordinary allocator source, artifact caller selection, actual x86-64
invocation and native/VM qualification remain pending. The new systems artifact
member is not yet executable. This model stage does not add a kernel capability,
change the preceding qualified native artifact, or qualify general allocation.
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
Compiled-source and physical evidence remain separate future obligations; no
independent reviewer result is claimed.
