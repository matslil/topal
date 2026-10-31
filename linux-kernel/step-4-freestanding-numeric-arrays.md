# Step 4: bounded ordinary numeric Arrays

`TK-DEC-259` implements the [numeric Array design](design/freestanding-numeric-arrays.md)
and `TOPAL-SYSTEMS-NUMERIC-ARRAYS-001`. The imported clock module now constructs
an ordinary two-entry integer Array and passes it to its duration helper.
Checked Optional reads supply the actual seconds/nanoseconds used by the policy.
The native scalar interface and Linux-visible behavior are unchanged.
Core-language meaning is unchanged. [Step 4](step-4-status.md) remains unfinished.

The primary kernel SHA-256 is:

```text
1ad6065ca12d63795453f6f4e2a293ced64beebd09a2b8955b57ffd28a4afeb8
```

Artifact/root/provider/object/boot/gate revisions are
`235 / 221 / 222 / 214 / 135 / 235`. Numeric metadata, the allocation certificate
and the owned numeric machine profile advance to version three, binding the
additional reviewed collection runtime/layout bytes. Finite-library metadata
and exact source-module snapshot formats retain their existing version.

## Semantic and ownership boundaries

The subset admits finite `List Int` construction, fixed `Array (N, Int)`
collection, constant-index checked lookup, count/emptiness observations and
`Optional Int` construction/decisions. Extents are exact and limited to 256
entries. The evaluator retains each entry's numeric range independently;
equal-length branch joins combine corresponding entries. Optional joins retain
both possible absence and all possible payload ranges. Decisions bind payloads
only on present paths and charge the maximum reachable branch. Ordinary frontend
checking and complete emitted-object closure/stack inspection remain required.

List construction charges every head/tail evaluation and its 16-byte node.
Collection charges the source once plus the 16-byte Array header. Every lookup
charges its receiver once plus a 16-byte Optional wrapper, including absent
results. Count includes conversion allocation. Unsupported elements, dynamic
indexes, transformations, other containers, effects, recursion and excessive
length/work remain profile errors. Isolated module clients construct precise
collection arguments without caller-private declarations.

Ordinary shared lowering and collection algorithms are reused. Dead-code
elimination retains only qualified Array/count/Optional helpers; object
inspection rejects other collection symbols and hosted dependencies. The
certificate binds these exact runtime digests:

| Input | SHA-256 |
| --- | --- |
| Shared numeric/Optional runtime | `a1c809e3b73bcd168457ca394a0dd4f1613abb68086dc0b97f1e7c168d415de6` |
| Shared fundamental collections | `a740ddaf6b69935187c18cdc010597f575940fa1e59a3f799c362a9fd1d176d3` |
| Shared integer List layout | `73ccaed62678c8170247f02ab6b46632919ba768765cb53c0ca804696cd66561` |

No allocated value escapes one exclusive invocation. Source receives no native
pointer, context, layout, allocator or resource authority. The bridge extracts
only its checked scalar result; all allocated nodes, headers, wrappers and
numeric objects plus retired stack storage are erased. The existing immutable
shared zero constant remains immutable code-owned data and needs no invocation
allocation or erasure.

The complete actual stack bound remains **1304 bytes** within **4096**. The
conservative allocation bound is **41624 bytes** within **65536**: **41520**
source/runtime bytes, **72** bridge bytes and **32** owner-header bytes. The
80-byte increase covers two nodes, one Array header and two Optional results.
The operand bound remains 93 bits. Neither reserve increases.

## Mapping and cross-architecture intent

The clock's Linux representation copy/validation and root-owned hardware view
retain their previous boundary. Ordinary source groups facts into Records,
constructs the immutable interval sequence, performs checked reads and computes
duration, saturation and rounding. Private container storage is not a Linux ABI
or an external byte layout. Exact finite sequences are a foundation for source
parsing of future owned snapshots; this increment does not yet grant that access.
AArch64 and RISC-V retain the collection/evaluation laws while requiring their
own private representation, runtime, calling convention and complete frame proof.

## Executed qualification

The [numeric harness](labs/qemu/x86_64/numeric_gate.py) observes an actual imported
`duration-from-parts` Array parameter, checks its header and both List nodes in
the active owned arena, decodes arbitrary-precision entries independently and
compares them with the native input facts. The observed entries are `[0, 1]`;
the header is at arena offset 112 and the nodes at 96 and 80. One payload is the
immutable zero constant, verified against actual immutable ELF section bytes;
the other payload lies in the invocation allocation. Complete arena/retired-stack
erasure and terminal binding release still pass. Backtraces retain real source
helper paths; scalar root arguments shown in older frames can be stale after
register reuse and are not used as value evidence.

The separate reordered-Record kernel has SHA-256
`7c1bb7cce4aea8817a04dfd2e5712f878a1119a161950d1530e4771d2567adb8`.
It retains the same module snapshot/native root/provider objects, opposite
Record-order metadata, equal Array entries, independent arithmetic and erasure.

Five new compiler tests cover per-entry bounds and evaluation costs,
inline/bound source costs, empty/out-of-bounds/count behavior, Array/Optional
joins, malformed/unqualified source and the 257-entry rejection. The native
Array fixture executes **2048** independent present/absent/empty/joined cases.
The existing native deadline fixture passes **1300** independent 128-bit
arithmetic cases and all invalid native-input fatal guards. The oversized
Record-call stack rejection and module-isolation/provenance tests remain.

Validation includes 220 compiler library tests, 116 semantic-model tests,
24 toolchain contracts plus four reference-VM contracts, the full bounded serial
Cargo workspace, formatting/whitespace and 254 unique normative systems rules.
The physical matrix retains **56 successful primary runs on the same kernel**,
two separate positive Enum/Record-order kernels and seven separate expected
rejections: **65 retained VM runs**. The primary corpus includes 34 unmodified
external workloads, three clean environments, clocks/time, source/runtime
helpers, child/pair/FIFO state, large command substitution and complete teardown.
The independent changed-root/changed-module policy oracles, corrupted child/pair/
FIFO rejection and deadline predecessor rejection remain separate evidence.

Risk is high at per-entry bounds, finite traversal, Optional branch accounting,
runtime closure and ownership. Review follows authoritative collection semantics,
the abstract evaluator, shared LLVM/runtime algorithms, actual native objects,
independent C arithmetic/collection fixtures, source-frame observations and
terminal reclamation. This is single-agent implementation/review with independent
executable oracles; it is not an independent human code review.

General byte-buffer access, persistent subsystem state/effects, scalable
processes, dynamic ELF, writable storage, networking, namespaces/cgroups,
distribution replacement, Docker/Podman and VM hosting remain required by the
[initial acceptance](step-4-status.md).
