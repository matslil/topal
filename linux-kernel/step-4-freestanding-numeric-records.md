# Step 4: structured immutable numeric modules

`TK-DEC-258` implements the [numeric Record design](design/freestanding-numeric-records.md)
and `TOPAL-SYSTEMS-NUMERIC-RECORDS-001`. The relative-sleep entry now groups its
validated scalar facts into a nested ordinary Record; the imported clock module
owns the complete arithmetic policy, including immutable state reconstruction.
The native scalar interface and Linux-visible behavior are unchanged.
Core-language meaning is unchanged. [Step 4](step-4-status.md) remains unfinished.

The primary kernel SHA-256 is:

```text
5d98ad88f8282033610df1ab73874625ec1589ad408e4a9a2d5093ca809b7a7d
```

Artifact/root/provider/object/boot/gate revisions are
`234 / 221 / 222 / 214 / 134 / 234`. Numeric library metadata retains its
version-two shape, with machine profile `x86_64-owned-numeric-gpr-only/2` and
allocation certificate `topal.numeric-allocation/2`. The shared numeric runtime
digest is still `a1c809e3b73bcd168457ca394a0dd4f1613abb68086dc0b97f1e7c168d415de6`.

## Value, lifetime and proof boundaries

The profile admits recursively closed Records whose fields contain the existing
qualified Boolean, Nat, Int or Euclidean-pair values. Construction, static
selection, reconstruction, branch joins and helper parameter/result transport
reuse the ordinary shared frontend and lowering. Imported type declarations,
nominal payloads, effects, dynamic field selection, reflection and other
unqualified forms remain rejected. Source data grants no machine, allocator,
address or writable resource authority.

The bound evaluator retains a conservative numeric bound per semantic field.
It charges all constructed fields, the selected receiver, the reconstruction
base and each replacement according to their actual evaluation. Reconstructed
bounds preserve untouched fields and do not mutate the original environment.
Matching branch shapes join fieldwise. Its cache omits presentation order only
because admitted operations cannot observe that order; actual ordinary lowering
retains the order metadata. No aggregate allocator or new runtime algorithm is
introduced: Record components remain ordinary aggregate/SSA values, with numeric
subvalues in the existing exclusive invocation arena.

Isolated module-checking clients now construct exact recursive Record arguments
without caller-private declarations. Exact root/dependency snapshots and module
provenance retain the preceding qualification. Every aggregate and numeric
subvalue stays within one invocation; the native bridge validates/extracts only
the scalar result before full arena/retired-stack erasure.

Actual target objects and native call/frame metadata include aggregate argument
transport and spills. The complete stack bound is **1304 bytes** within the
unchanged **4096-byte** reserve. The complete conservative allocation bound is
still **41544 bytes** within **65536**, including a 41440-byte source/runtime
evaluation bound, 72 bridge bytes and the 32-byte owner header. The admitted
operand bound remains 93 bits. A separately compiled 256-field Record call is
rejected specifically because its actual native stack requirement exceeds the
reserve; its small arithmetic cost cannot substitute for the frame proof.

## Original-design mapping

| Responsibility | Current Topal realization |
| --- | --- |
| Validate native time representations | Existing Linux personality checks/copies native input before constructing semantic facts |
| Group interval and clock observations | Ordinary nested Records, independent of `timespec` bytes, HPET registers or a native C layout |
| Convert/saturate/round deadlines | Complete ordinary imported clock policy with named fields and immutable reconstruction |
| Carry helper state through calls | Shared checked aggregate lowering, target-private ABI, complete frame/allocation certificates |
| Retain and reclaim temporary values | Exclusive invocation ownership, scalar result extraction and checked complete erasure |

The source interface is a step toward structured ordinary subsystem modules.
Record order is not a Linux ABI, and a field value is not resource authority.
AArch64 and RISC-V require their own aggregate ABI, frame and storage evidence
while preserving the same source field/evaluation laws and normalized clock view.

## Executed evidence

The [numeric harness](labs/qemu/x86_64/numeric_gate.py) observes a real
`duration-nanoseconds` Record parameter in the imported module. Both decoded
arbitrary-precision fields equal the native input facts. Its 24-byte private
argument storage lies 736 bytes below the original caller stack, within the
proved reserve. The primary execution carries construction-order metadata
`[1, 0]` through nested source calls. A separate reordered-constructor kernel
carries `[0, 1]` while preserving named values, independent native arithmetic,
native state and complete storage/binding erasure. The native root/provider
objects and imported module snapshots remain unchanged in that variant.

That positive variant has SHA-256
`6eb49d6f5bf87139ffbbd49bbba547109d249be4d18bfd5856680f58b67c24f1`.
Both variants retain the same 1304-byte stack and 41544-byte allocation bounds.
The preceding wrong-root and changed-imported-helper deadline counterexamples
still reach the independent native arithmetic oracle before normal completion.
They retain separate rejection records and make no normal rollback claim.

The native host driver compares the complete source policy with independent
128-bit arithmetic over 300 boundary combinations and 1000 deterministic
additional inputs, including rounding, conversion/addition saturation and
extended epochs. Invalid native input cases fail closed. Its measured highwater
is 11440 bytes; this observation supplements the conservative proof.

New bound/admission tests check nested helper values, unchanged reconstruction
bases, equal inline/bound-base costs, evaluation of unused fields, fieldwise
branch ranges, order-insensitive bounds, malformed labels/shapes/classifiers,
unqualified payloads, unbounded arithmetic and actual aggregate stack overflow.
The lab contract binds real Record inputs, source frames/order metadata,
unchanged imported sources, native arithmetic and full erasure to the physical
executions.

Final qualification includes the bounded serial workspace, 215 compiler tests,
116 semantic-model tests, twenty-three toolchain plus four reference-VM contracts,
formatting, whitespace, relative links and 253 unique systems rules. The retained
primary matrix has 56 successful physical QEMU runs on one kernel. Reordered
Enum and Record kernels are two additional positives; seven separate expected
rejections remain distinct from successful runtime qualification.

## Review and remaining scope

Risk is high at recursive shape admission, label identity, exact evaluation
charges, cache abstraction, aggregate ABI/frame costs and lifetime of nested
numeric values. Review traced each source form through shared checking/lowering,
field bounds, actual object dependencies and frame evidence, isolated module
clients, unchanged native bridges and full erasure. Independent arithmetic,
actual typed argument bytes, order variants, resource failures and preceding
process/FIFO regression address different failure modes. No third-party source
or new dependency is introduced.

The profile still admits bounded pure numeric invocation work. Persistent
allocation/state, broader ordinary types/effects and general kernel subsystems
remain. General processes/scheduling/signals, executables/mappings, writable
VFS/devices, networking/isolation, distribution replacement, Docker/Podman and
VM hosting remain required. This report does not complete the initial plan.
