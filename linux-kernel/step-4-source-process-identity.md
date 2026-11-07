# Step 4: source process-identity transitions

`TK-DEC-262` implements the [identity transition design](design/source-process-identity-transition.md),
`TOPAL-SYSTEMS-NUMERIC-RESULT-RECORD-001` and
`TOPAL-SYSTEMS-PROCESS-IDENTITY-LIBRARY-001`. Ordinary Topal source now proposes
the existing process-ID allocation and next-cursor transition. The owning native
process resource applies both validated scalar results after invocation cleanup.
This moves one retained-state policy into source. Core-language meaning and the
existing bounded process model remain unchanged. [Step 4](step-4-status.md)
remains unfinished.

The primary kernel SHA-256 is:

```text
5a1f2342fac6b76c56250de9ad3f746cb93bf52a66295b0e7ceeaeca3dbceec6
```

Artifact/root/provider/object/boot/gate revisions are
`238 / 224 / 225 / 217 / 138 / 238`. Numeric metadata, allocation certificate
and owned numeric machine profile advance to version six. They record canonical
result-field slots and the ordinary per-value construction-order suffix. Existing
finite-library and module-snapshot formats retain their versions.

## Source and owning resource

The [source policy](kernel/policy/process-identity.t) exports ordinary declared
type `Record (next : Nat) -> Record (allocated : Nat, next : Nat)`. Its imported
[numeric helpers](kernel/support/process-identity.t) implement the existing law:
zero initializes allocated ID two, another legal cursor allocates itself, and
the successor increments or wraps from 2147483647 to two. Native input guards
admit zero or 2..2147483647 before allocation; both extracted results must lie
in 2..2147483647. These are numeric facts and confer no process authority.

The current process resource retains ownership of its cursor, empty/live/reaped
child slot, generation, memory, continuations and registration state. Existing
argument, capacity, lifetime and abandoned-snapshot checks precede invocation.
Temporary flags/TID staging precedes the copied cursor. Child identity and cursor
publication follow complete result validation and temporary cleanup. The initial
parent keeps PID one. Reuse requires the existing reaped/empty slot; this does not
provide scalable PID allocation or a general PID namespace.

The bridge constructs the ordinary one-field input Record using the shared
numeric runtime, calls source, validates result presentation indexes and both
ordinary numeric representations, and extracts fields by semantic name. The
ordinary backend keeps canonical field slots and a separate per-value order
suffix. Its exact internal result is `{ ptr, ptr, i32, i32 }`; LLVM lowers that
24-byte private result through a caller-owned stack slot. The adapter preserves
this existing representation and returns only two validated native words through
a separately qualified private C convention, RAX/RDX on this target. No source
Record, numeric allocation or pointer leaves the exclusive invocation.

Independent native execution caught an initially incomplete private prototype
that omitted the ordinary order suffix. Using the complete existing Record
representation fixed it. A separate physical witness decodes the actual returned
stack Record, including its order suffix, before observing the two native words.
Reversed declared and constructed fields change the observed presentation indexes
from `[0,1]` to `[1,0]` while preserving named results and canonical slots.

Identity, dispatch, header and deadline policies operate independently or share
one arena through exclusive sequential invocation. Identity uses active tag four
and a distinct source/runtime namespace and code binding. All live callee-saved
and SIMD state survives. Both result words survive full arena erasure/readback,
exact retired-stack erasure/readback and volatile-pointer clearing. The binding
fits within the existing process-context snapshot, survives child/exec lifetime,
and is erased/read back with all selected bindings on terminal return. Declared
source failure never selects native cursor arithmetic. Roots without the new
declaration retain their previous bounded path.

## Resource and protocol evidence

The complete allocation bound is **312 bytes** within **65536**: **256** source
evaluation bytes, **24** copied-fact bridge bytes and **32** owner-header bytes.
The maximum bounded operand width is **32 bits**. This is a qualification bound,
not a change to arbitrary-precision numeric meaning. The complete actual stack
bound is **536 bytes** within **4096**, including aggregate input/output transport,
actual source/runtime frames, returns, native saves (**64**) and alignment (**32**).
Observed arena high-water is **80 bytes** and the retained imported-source
snapshot is **280 bytes** below the native caller stack. Measurement supplements
the complete proofs.

The independent C client executes **66538 transitions per field order**, **133076**
in total: every legal cursor below 65536, three maximum/wrap boundaries and 1000
deterministic dispersed cursors. It checks both returned words independently.
Six executions reject invalid private inputs, with abort checking that no
allocation occurred first. Three further executions reject typed source results
that violate the output protocol: zero, 2147483648 and a multiword value whose
low word would appear legal if truncated. Exact source signatures, malformed
types/shapes, source isolation, canonical slots/order suffix, missing/cross-policy
bindings, tampered provenance and undercharged resources also receive tests.
The lifetime model exercises all fifteen nonempty subsets of four numeric
policies, including wrong-owner/policy, overlap rejection and complete teardown.

The [identity harness](labs/qemu/x86_64/identity_gate.py) and
[physical inspector](labs/qemu/x86_64/identity_inspect.py) retain
[five VM observations](labs/qemu/x86_64/results/topal-source-process-identity-gate.json):

- The primary kernel completes nine identity transitions and subsequent child
  returns. Each call observes the actual source Record/returned fields, native
  two-word adaptation, unchanged full context/table/clock before commit, complete
  temporary cleanup, subsequent owner cursor/identity writes and the live child
  PID. The primary sequence allocates IDs two through ten.
- A reordered declaration/construction kernel completes the same nine transitions
  with reversed actual order indexes and unchanged native root/provider objects.
- An identity-only numeric configuration completes the same nine transitions,
  with deadline/header/dispatch imports absent and complete terminal reclamation.
- A controlled copied cursor changes only the C-boundary input to 2147483647.
  Source returns `(2147483647,2)`; the witness stops after cleanup before commit
  and proves the persistent cursor is still zero. This is controlled input,
  not billions of executed child lifetimes.
- A changed imported successor helper returns a legal but wrong `(2,2)` instead
  of `(2,3)`. The independent oracle rejects it after cleanup and before owner
  publication. Root policy and native root/provider objects remain unchanged;
  the imported snapshot digest changes. This is expected rejection, not a
  successful userspace run.

The three successful configurations cover **27 cleaned and committed transitions**;
the two stopped observations add two independently checked precommit boundaries.
Source/helper/adapter traces retain actual debug frames without claiming unwind
qualification for the enclosing native syscall loop.

## Complete regression and review

The refreshed cohort contains **59 successful primary physical runs on one kernel**,
**seven positive variant-kernel runs**, **two controlled precommit observations**
and **ten expected-rejection runs**, totaling **78 retained VM observations**.
The primary cohort includes the unchanged static BusyBox command corpus, empty/
null environments, deadline/header/dispatch/numeric/source-library policies,
children, cooperative pair/termination and pipe/substitution checks. Existing
dispatch evidence continues to inspect every selected native operation before
handler entry; identity cleanup occurs later inside a selected creation handler.

Validation records the bounded serial workspace, formatting and whitespace
checks, **231 compiler / 120 semantic / 607 language tests**, **27 toolchain plus
four reference-VM contracts**, **259 unique normative systems rules**, same-kernel
cohort checks and qualification-link checks. The host wrapper retains its memory,
serialization and no-swap safeguards.

Risk is high at Record representation, aggregate return adaptation, retained-state
publication, temporary lifetimes, shared bindings and native continuation safety.
Single-agent implementation/review follows authoritative Record/resource intent,
the conservative evaluator, actual ABI/object/frame data, independent transition
arithmetic and full before/after state observations. This does not replace human
code review. Source and tests are independently authored; no third-party kernel
implementation material is added.

For a future Linux baseline, compare ID-domain/reuse and creation/commit requirements
with the source law and owner boundary. Changes to value policy belong in source;
ownership, synchronization, lifecycle and external errors require separate resource
and conformance updates. Other architectures must qualify their native adapters,
aggregate/scalar conventions and complete bounds while retaining this value law.

General persistent Topal allocation and effectful retained subsystems, scalable
processes/scheduling, signal delivery, general/dynamic ELF, writable storage,
networking, namespaces/cgroups, distribution replacement, Docker/Podman and VM-host
qualification remain required by the [initial acceptance](step-4-status.md).
