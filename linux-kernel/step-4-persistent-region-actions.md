# Step 4: ordinary source actions on persistent file positions

`TK-DEC-272`; implements the [persistent action design](design/persistent-region-actions.md)
and `TOPAL-SYSTEMS-REGION-ACTION-{SOURCE,OWNER,RETURN,POSITION}-001`.
Step 4 remains in progress. Distribution replacement, Docker/Podman and VM
hosting remain unqualified.

## Executed boundary

The [primary root](kernel/arch/x86_64/toolchain-gate.t) selects
[ordinary file-position source](kernel/policy/file-position.t) through
`file-position-library`. Its generic callable context is
`borrowed-byte-region-action-v1`, with ordinary signature
`(KernelByteRegion, Nat, Nat) -> Boolean`. The source computes the admitted
effective base plus actual transferred count, checks the signed file-offset
limit and calls an imported [eight-byte codec](kernel/support/word-bytes.t).
That helper uses ordinary arbitrary-precision arithmetic and checked byte
stores to update the actual shared description field. The transport accepts
both complete unsigned 64-bit scalar inputs; addition can require 65 bits.
No core numeric, effect, Memory, Layout, resource or AllocationRegion meaning
changes.

Selected immutable-file reads and volatile regular-file reads/writes invoke
that source after the admitted nonempty transfer. Their native path performs
no duplicate position addition/store or fallback. File-size growth reloads
the source-written field. Append supplies EOF as the effective base even when
the old description position differs. Positional transfers, zero/EOF/error
paths issue no action. Seek, directory positions, copying, clipping, append
placement and recoverable admission/error ordering retain their existing
native implementations. This is a first source subsystem operation, with
substantial remaining native file policy.

The controller owns both description banks, including aliases and inactive
parent/child references. Its grant suspends all controller access throughout
the call. The qualified x86-64 services execute serially on the bootstrap
processor with interrupts masked, without suspension, callbacks or another
processor accessing that controller. QEMU still has two virtual processors;
local interrupt masking alone is not an SMP exclusion proof.

The native bridge validates the actual live owner, description bank/stride,
kind/reference, kernel stack, clean private storage and fresh nonwrapping
generation. It lends exactly the existing eight-byte position field. Adjacent
kind, reference-count and control bytes remain outside the source capability.
Every call owns a distinct 65536-byte arena, 64-byte control and scalar
generation history; none aliases the process numeric context or still-live
root region invocation. The containing controller retains its effect identity
through aliases and contained views.

## Completion and executable binding

The actual Boolean result survives complete view revocation, arena/control
erasure and readback, erasure/readback of the entire 4096-byte retired stack
reserve and 32-byte input/result staging, volatile-register clearing and native
callee/RSP restoration. Successful position stores survive. Only scalar
generation history persists between calls. Terminal cleanup retires it after
both active and inactive references and controller storage are gone.

In-place payload/position preparation stays private until completion and
reclamation. A valid admitted transfer requires true. False afterward is an
invariant failure which never resumes the owner, adds no Linux errno and
promises no rollback. The model separately checks independent expected bytes
and explicit cleanup obligations. It does not pretend that a source graph,
detached buffer or model flags provide physical execution/erasure evidence.

The action source object has its own private schema, native Boolean/scalar
ABI, export, leaf allocation hook, nonreturning abort and complete namespace.
It coexists with the prior Unit entry contract. Declared-entry selection uses
the shared ordinary frontend without constructing a source capability or
fake executable client. Imported modules retain ordinary visibility, effects,
nonescape and closed-call checking.

The complete proof charges 60008 source-evaluation bytes, 112 bridge bytes and
32 header bytes: 60152 against the 65536-byte arena. The bridge includes one
64-byte view and two ordinary Nat constructions. The source stack proof is
1856 bytes; the caller adds 88 live-frame bytes and 32 alignment bytes, for
1976 against the 4096-byte reserve. The 32-byte caller allowance includes the
immutable reader's eight-byte live count/alignment push and 24-byte inner
adjustment; the two volatile call sites need only 16 inner bytes. The physical
inspector checks those actual call-site residues and charges on every invocation.
The graph instantiates 26 functions,
requires one view and retains the existing 256-bit proof budget with an actual
65-bit maximum. Eight conservative integer-allocation charges cover the
existing finite Absolute path; positive Euclidean quotient intervals preserve
the shrinking word-extraction ranges. No runtime arithmetic algorithm changes.

Publication freezes source/modules, ordinary types/effects, complete bounds,
actual source/runtime instructions, native templates/relocations, storage,
the complete process-service caller code and actual controller owner. Action
constants retain local named immutable ELF symbols and independently bound
byte digests; code hashes alone would leave literal data unbound. Missing or
extra code/data, incompatible placement/costs and changed immutable bytes
reject. Boot packaging repeats the complete linked inspection. A corrected
outer image checksum does not legitimize changed source data or code. The
structural counterexample also aliases a separate root allocation into the
action arena through ELF symbol metadata and corrects the outer digest; the
boot inspector rejects the overlapping ownership.

## Qualification

The primary kernel SHA-256 is
`ccb41e7735d4962b43d2e0213608518252402b067ca9e07e8b50188681a79827`.
Artifact/root/provider/object/boot/gate revisions are 246/232/233/225/146/246.


The [physical harness](labs/qemu/x86_64/file_region_gate.py),
[inspector](labs/qemu/x86_64/file_region_inspect.py) and
[record](labs/qemu/x86_64/results/topal-file-position-region-gate.json)
observe actual controller fields, both Nat inputs, the real Boolean result,
ordinary codec/entry/source bridge/native bridge/service-caller frames and
the independently read caller return PC. Each invocation checks the complete
64-byte description, temporary storage, saved registers/RSP and generation.
Paired independent userspace syscall observations check every live position
and require exactly one action for a successful selected ordinary transfer,
with no call or position change on positional/zero/EOF/error paths.

The primary and lifetime fixtures each observe 46 real source invocations,
with 360 and 411 independently checked user syscalls respectively. Three
append invocations per run supply a base different from the old position.
These fixtures exercise immutable and sparse volatile files, shared aliases, append with a different old position, independent opens,
fork/exec and eventual retirement. An inactive-child control suspends a live
child, terminates its parent, and checks complete controller/history erasure.
Pre-entry controls supply an invalid owner, field stride, enabled interrupts,
dirty arena or exhausted generation. They enter no source, never resume the
owner and leave their captured storage unchanged; they do not claim successful
cleanup. A wrong ordinary first-byte codec is caught by an independent field
oracle before owner resumption. A separate action stores the correct bytes
then returns false; its actual invariant-fatal path reclaims all temporary
storage without undoing stores. Source absence exercises the retained native
compatibility path.

Controlled observations wait for successful debugger disconnect and query QMP
for an actually stopped target before QEMU teardown. The inactive-terminal
case can emit ordinary final root markers; its harness keeps observing until
the debugger and stopped-target checks finish. Saved-return-address
breakpoints qualify native leaf returns inside internal same-symbol service
subroutines, where GDB finish-scope inference is ambiguous.

The [public compiler integration tests](tools/toolchain-gate-builder/tests/region_actions.rs)
add independent hosted native evidence: 168 signed-domain calls, another 168
full-u64 calls, extent/rights/canary/overflow checks, 12 pre-entry refusals per
run, ordinary helper debugger frames, actual Boolean transport, Unit/action
coexistence, exact certificate replay, source/body/data controls, explicit native
call relocations, source-snapshot rejection and kernel/boot binding mutations.
The hosted maximum observed arena use is 29664 bytes; that observation does
not replace the complete 60152-byte proof.

The complete bounded serial Cargo workspace passes, including 620 language
unit tests, 251 compiler unit tests, 261 compiler CLI tests, 173 semantic-model
tests and all eight public action integration tests. The workspace all-target
check, workspace formatting and explicit included-file formatting checks pass.
The fresh full cohort passes 133 VM observations: the preceding 122, including
all 40 unmodified pinned BusyBox workloads, plus eleven focused action cases.
All 37 Python lab contract tests, aggregate code/data/cleanup/position evidence,
290 unique normative rules, relative links and whitespace checks pass. Historical
reports retain their earlier kernel identities; current records name the kernel
above. The final focused rerun additionally checks every actual caller alignment
and live count-save charge without changing the kernel.

Earlier validation builds were killed at automatically selected 718–751 MiB
cgroup budgets; the journal identified rustc. Requested wrapper limits and host
reserve were unchanged. The final passing workspace uses serial compilation/
tests, disabled host test debug/incremental state and two allocator arenas within
its automatically selected 1166 MiB budget. Generated Topal DWARF remains enabled
and physically checked. An initial provider unit assertion retained the prior
24 relocations/sole Unit import; the current provider has 51 relocations and
exactly the Unit/action source imports. The corrected full suite passes.

Risk is high at shared lifetime/exclusion, private commit ordering, scalar ABI,
complete costs, erasure and linked provenance. The implementing agent reviewed
authoritative resource/effect and systems intent, independent model laws,
native admission/call/return/teardown paths and physical negative oracles.
Human PR review remains required; no independent reviewer result is claimed.
The [status ledger](step-4-status.md) retains the initial-plan acceptance work.
