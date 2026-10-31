# Step 4: owned runtime for ordinary numeric kernel policy

`TK-DEC-253` moves relative-sleep deadline arithmetic into ordinary Topal
[`Nat`/`Int` functions](kernel/policy/relative-sleep-deadline.t). The kernel
executes the shared arbitrary-precision algorithms using exclusively borrowed
temporary storage. Core-language meaning and hosted arithmetic are unchanged.
The [design](design/owned-numeric-kernel-runtime.md) and rules
`TOPAL-SYSTEMS-NUMERIC-RUNTIME-001` and
`TOPAL-SYSTEMS-SLEEP-DEADLINE-LIBRARY-001` define this qualified profile.

This advances Step 4's freestanding-library requirement. General allocation,
process creation, distribution boot, containers and VM hosting remain unfinished;
the [remaining acceptance work](step-4-status.md) applies.

## Source and publication

The root declares a contained `time-library` snapshot, public export, semantic
interface, 4096-byte stack reserve and 65536-byte temporary arena. The shared
frontend checks the export at its declared types, without specializing it to the
selection client's literal inputs. Ordinary helper calls, Boolean decisions,
numeric arithmetic and Euclidean quotient/remainder retain their semantics.
Unsupported forms, effects, foreign calls, imports, cycles and missing/private
or mismatched inputs fail before atomic publication. A declared library cannot
select a native arithmetic fallback.

The existing value algorithms are separated mechanically from the hosted Linux
allocation/output/exit hooks. Hosted programs retain those hooks. This kernel
library instead binds only the qualified allocation and fatal services. LLVM
removes unreachable runtime functions; inspection rejects unexpected exports,
undefined dependencies, writable allocated globals, TLS and unsupported code.
Source functions receive ordinary values, never the arena or an owner pointer.
Private source symbols use a distinct namespace to keep separate library debug
and code bindings unambiguous. The detached debug file omits LLVM's optional
accelerator index; ordinary source, line and frame information remains available.

Native interval copying/validation, clock sampling/ownership and waiting surround
the source calculation. The selected source computes conversion saturation,
saturating addition and upward rounding; native code only validates the returned
scalar. The initial interface uses the qualified ten-nanosecond clock view.
Other architectures must normalize their observations or qualify explicit scale
facts, together with their own machine and ownership evidence.

## Resource proof and lifetime

The allocation certificate binds the exact shared runtime digest. Conservative
analysis covers the complete admitted input ranges, all decision branches,
helper calls, repeated expression evaluations and Euclidean iteration costs.
The largest admitted intermediate bound is 93 bits. The proof reserves
**41440 bytes** for source/runtime evaluation, **72 bytes** for three native
input objects and **32 bytes** for arena bookkeeping: **41544 bytes**, within
65536. A runtime algorithm change invalidates the certificate and requires
review; measurements alone cannot establish this bound.

Actual LLVM frame metadata and function-section call relocations form an acyclic
native/source/runtime graph. Return addresses, an inspected eight-byte native
leaf cost and the 32-byte alignment reserve yield a maximum of **808 bytes**,
within 4096. Compilation prohibits red zone, implicit floating point and tail
calls. Missing, cyclic, excessive or ambiguous frame/dependency evidence fails
publication.

The root owns supervisor-only arena storage and lends its identity with the
immutable code binding to the prepared process. Each invocation checks inactive,
zeroed storage, owner and masked interrupts; activates one lending interval;
constructs ordinary numeric representations; and copies out a checked scalar.
Allocation validates owner/activity, alignment, extent and count. Invalid state
fails closed. Before waiting, the provider erases and reads back all 65536 arena
bytes and the entire 4096-byte retired helper-stack reserve, then clears volatile
pointer registers. Bindings survive exec and are cleared/read back at terminal
return. Concurrent/reentrant lending remains outside this profile.

## Executed qualification

The primary kernel SHA-256 is:

```text
f5273afc44d0efdb93f3fe1e63a3f02783b4e165fd0477a6c7d010f12a7a198e
```

Artifact/root/provider/object/boot/gate revisions are
`229 / 218 / 219 / 211 / 129 / 229`.

The [numeric debugger harness](labs/qemu/x86_64/numeric_gate.py) and
[retained record](labs/qemu/x86_64/results/topal-owned-numeric-runtime-gate.json)
observe **24** actual calls before and after exec. Independent integer arithmetic
checks each result. GDB observes ordinary source/helper and shared Euclidean
runtime frames, active owner bookkeeping, bounded allocation, aligned stack,
masked interrupts, preserved callee-saved and all sixteen XMM registers, untouched
unused arena storage, complete arena/retired-stack erasure and cleared pointer
registers. Terminal observation confirms erased bindings and arena storage.

A separate source variant returns zero for ordinary deadlines. Root/provider
object digests remain unchanged and the numeric object changes. The independent
debugger oracle observes zero where a positive target is required and explicitly
rejects execution before completion. Timeout alone is insufficient. This record
claims neither normal completion nor reclamation for the counterexample.

Native object execution independently checks **1300** valid boundary/generated
input combinations and three invalid native representations. Its measured peak
is 11440 bytes, below the conservative proof; this measurement is supplementary.
Negative compiler/model tests cover mismatched/private/missing exports, source
initializers, signature mismatch, cycles, unproven divisors, unsupported arithmetic,
operand/resource limits, wrong ownership, reentry, overflow, exhaustion and dirty
release. Publication and boot-packager tests exercise both source snapshots.

The same primary kernel passes **40 physical QEMU runs**: three clean environment
modes, 26 pinned unmodified BusyBox workloads, five controlled RTC cases, three
controlled relative-time cases, completed-deadline notification injection, the
finite-library debugger run and the numeric-runtime debugger run. Controlled
hardware advancement remains explicit test input. The reordered-Enum kernel is
one additional positive run. Wrong finite/numeric policies and the historical
deadline predecessor remain separate expected rejections.

The bounded full-workspace suite passes; the final focused compiler suite passes
206 tests. All seventeen lab contracts pass. Formatting, whitespace, relative
links and 244 unique normative systems-rule identities are checked.

## Review and limits

Risk is high at the arbitrary-precision representation, allocation proof,
privileged code/storage ownership, ABI and erasure boundaries. Self-review traced
the authority chain, declared-type selection, unchanged shared algorithms,
per-operation allocation counts, repeated evaluation, frame/call closure,
machine mappings, native leaf assumptions, argument alignment, result validation,
exec retention and terminal release. Independent host arithmetic, controlled
hardware cases and the wrong-source counterexample test observable behavior.

This qualifies one pure numeric interface and bounded exclusive invocation.
It does not qualify arbitrary containers/imports, persistent allocation,
multiple user processes, signal delivery, writable VFS or distribution/container/
virtualization acceptance. Substantial kernel policy remains in native adapters.
