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

At the portable-model stage, all 181 semantics unit tests and 621 language unit
tests passed, including eight new page-action tests. Workspace all-target checks
and formatting passed at that stage. The
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

## Live-kernel implementation

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
This qualifies the bounded RAM-page owner. The complete affected preceding
VFS/BusyBox/source cohort has also been freshly executed: 137 observations,
including forty unmodified BusyBox workloads. Combined with the 21 new RAM-page
cases, this increment has 158 fresh VM observations. All 42 Python lab contracts
pass against current source/tool identities, pinned Linux controls, actual
native artifacts and controlled dispositions. No TK277 unchanged-artifact
exception or preceding machine observations substitute for these executions.

Two bounded compiler unit-test builds were killed by their cgroup at 723/816 MiB;
a later build was stopped after prolonged reclaim pressure at its 601 MiB cap.
The host reserve and zero-swap policy were retained. Sharing the verifier, moving
the binding test to a small public-API integration target and reducing host
allocator arenas allowed compilation and binding qualification to complete.
Compiler-library checking, current source-component/binding tests, workspace and
included-source formatting, strict source size and whitespace checks pass.
The later all-target check was killed by its 486 MiB cgroup. A retry with one
allocator arena and aggressive trimming was stopped after prolonged reclaim
pressure at 365 MiB. These are failed/incomplete validations. The full current
Rust workspace suite and all-target check were still required at that point.
The host's 1569 MiB reserve and zero-swap policy remained in force; more available
RAM or a separate qualified runner was needed.
An additional retry used the compiler's actual jemalloc configuration with one
arena, immediate dirty/muzzy-page purging, no retained mappings and no thread
cache. The compiler explicitly accepted every setting. This changed no cgroup
or host policy, but the all-target check was still killed at 420 MiB. A separate
language-model test build was killed at 388 MiB. The journal confirms both
out-of-memory dispositions. All 181 current semantics tests were separately
rebuilt and passed within a 380 MiB budget; this does not qualify the remaining
workspace targets.

A [separate-runner driver](tools/rust-qualification/README.md) is now prepared
for the outstanding Rust checks, retaining the unchanged bounded wrapper and
recording exact commit/tool/command/log identities. Its environment preflight
passes against Rust 1.97.0 and LLVM 22.1.6, and Python syntax and workflow structure
checks pass. The adjacent GitHub Actions template is inactive: activation needs
one workflow entry point outside `linux-kernel/`, at GitHub's required location.
The file-location exception was requested while the host lacked available RAM.
The later local qualification below completes the Rust checks; no hosted run
or workflow activation was needed.

## Additional Rust package qualification

Splitting validation by existing Cargo target completed the following current
implementation tests without changing the bounded-wrapper policy:

| Scope | Passed tests | Result |
| --- | ---: | --- |
| Source, syntax, C ABI/bindgen, GEIR, semantics, serialization and transfer packages | 323 | All targets pass; the 181 semantics tests are included in this count. |
| Best-practices, build, debugger, documentation, interpreter, linter and LSP packages | 779 | All targets pass, including 280 debugger and 404 interpreter CLI tests. |
| Kernel toolchain gate builder | 16 | Seven unchanged targets pass; the corrected region-action target passes all eight of its tests. |
| Language specification/requirement coverage integration | 4 | All tests pass after correcting the mechanically checked rule count. |

That partial stage qualified 1122 distinct Rust tests, before complete workspace
qualification.
The first gate-package command failed one legacy test: the current root's new
page declaration triggered its missing-snapshot guard before the intended
file-position guard. That structural control now explicitly selects the retained
native page allocator and verifies the absence of page-source calls. It still
requires the exact missing action-snapshot error and no destination publication.
The separate page-binding target retains its page-snapshot refusal controls.
All eight affected region-action tests then pass; unchanged targets were not
repeated. No production code, generated kernel or machine-observation binding
changed.

The coverage integration initially found two failures from the stale systems
rule count, 297 instead of 300, in `se/core-language-coverage.md`. Updating only
that count restores all four coverage/traceability checks without changing owner,
phase or completion status. Bounded workspace formatting and strict whitespace
checks also pass. An earlier formatting attempt refused insufficient available
memory and is not counted as a pass.

The compiler CLI integration build was killed at its 274 MiB cap; the separate
workspace doc-test build was killed at 297 MiB. Both are journal-confirmed cgroup
out-of-memory failures. Compiler/language unit targets, compiler CLI integration,
workspace doc tests and the complete all-target check remained unqualified then.

## Complete Rust qualification after releasing host memory

After the user freed host RAM, the unchanged bounded runner admitted an 1884 MiB
check budget and a 1797 MiB test budget, retaining the 1569 MiB host reserve and
zero cgroup swap. The [first complete attempt](tools/rust-qualification/results/tk278/before-provider-control-fix/qualification.json)
passed all-target checking and the doc-test command, with 2260 tests passing and
one structural provider test failing. This was an actual assertion failure,
distinct from the earlier resource failures.

The old provider control expected 51 relocations and two ordinary-source imports.
Selecting the new page controller adds relocations and its separately bound
source entry. The [corrected control](../src/topal-compiler/src/systems_provider_object_tests.rs)
now checks both profiles: the native-page variant retains 51 relocations and
two source imports; the source-page variant has 80 relocations and three source
imports. The latter also has a forward descriptor-table symbol record. Before
accepting it, the test proves its sole matching strong data definition in the
same object's process BSS, with exact extent and alignment. This records a local
forward reference rather than an additional external dependency. Deterministic
objects, exact symbol sets, ELF identity and nonallocated unwind data remain
checked in both variants. The focused control passes. No production code or
generated kernel changed.

The [fresh final record](tools/rust-qualification/results/tk278/final/qualification.json)
binds clean revision `ee5555f06b4e14f9b1d58629e85f22ab4ca30ba9`, its tree,
Rust 1.97.0/LLVM 22.1.6, the actual driver/wrapper and all command/log digests.
All three commands return zero: locked workspace all-target checking, locked
workspace all-target tests and the separate workspace doc-test command.
All **2261 tests across 41 targets pass**, with zero failed or ignored tests;
these include 254 compiler unit tests, 261 compiler CLI tests, 622 language unit
tests, 181 semantics tests and all 16 gate-builder tests. The doc-test command
currently discovers no executable doc tests. Every recorded log digest is
independently checked against its decompressed archived log. Both the earlier
failed attempt and the final successful attempt retain their complete logs.

Bounded workspace formatting and the unchanged 2000-line Rust source ceiling
also pass. The final changes concern a test control and qualification records;
the 158 fresh machine observations and 42 lab contracts above retain their
qualified production identities. This completes TK278's staged qualification.
The workflow template stays inactive, and its location exception is unnecessary
for this completed local validation.

The [initial acceptance ledger](step-4-status.md) remains in force. General
allocation, distribution replacement, containers and VM hosting remain unfinished.

Risk is high at bitmap/association consistency, preparation versus publication,
source failure, zeroing, stale ownership, cross-domain views, bounds and shared
process lifetime. Current self-review covers approved systems/resource/effect
intent, three exact rules, actual model borrows and independent reference laws.
Complete Rust workspace and all-target qualification now pass. No independent
reviewer result is claimed.
