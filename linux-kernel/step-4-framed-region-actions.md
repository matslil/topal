# Step 4: framed inputs for ordinary region actions

`TK-DEC-275`; implements the [framed-input design](design/framed-region-actions.md)
and `TOPAL-SYSTEMS-REGION-FRAME-SOURCE-001`/`TOPAL-SYSTEMS-REGION-FRAME-ABI-001`.
Step 4 remains in progress. Distribution replacement, Docker/Podman and VM
hosting remain unqualified.

## Behavior and boundaries

The new systems qualification selects an ordinary published function with one
initial `KernelByteRegion`, exactly the caller-declared 0–16 Nat facts, and a
Boolean result. Existing explicit operand packaging, declared-type selection,
effects, direct calls, arbitrary-precision arithmetic and nonescape rules apply.
There is no change to core function syntax or arithmetic meaning.

Its private x86-64 entry takes control, expected owner, next generation, an
immutable native-u64 word frame and exact word count. Thus larger source
signatures do not require additional native argument registers or a distinct
entry ABI per arity. Zero facts require a null empty frame and no word access.
Count and structural nonempty-frame guards precede scalar loads/source entry.
Source receives ordinary Nat values and its sealed region, never the frame
pointer or a native address.

The qualified caller must prove that the complete frame is mapped, immutable
until return, aligned and disjoint from owner/control, temporary arena and
retired stack. Structural bridge checks do not make arbitrary unqualified C
pointers safe. Input words convey facts rather than ownership. Controller
exclusion, fresh nonwrapping generation, owner suspension, view revocation and
complete cleanup retain their existing contracts.

The first kernel consumer remains [file-position source](kernel/policy/file-position.t).
The native caller lends its actual shared position field and captures effective
base/count in the first sixteen bytes of its existing owned 32-byte live frame.
It supplies the frame address and count two, replacing the old two native
scalar arguments. The ordinary source signature and byte-codec algorithm are
unchanged. Full staging, temporary arena/control and retired-stack erasure
precede owner resumption; an admitted false result remains invariant fatal.
Append/alias/fork/exec, positional noncalls and errno/empty/EOF ordering retain
their existing Linux behavior.

## Representation and bounds

Framed source-object and native-ABI schemas start at version 1; the linked
file-position binding uses version 2 with an explicit count. Symbols use the
flat `topal.framed-region.` namespace, outside Unit ownership. The legacy Unit
and two-scalar action schemas, entry ABIs, namespace and omitted optional fields
remain intact. Packaging rejects mixed profiles and incompatible count, family,
signature, code/data, native caller or resource evidence.
Readers also reject explicit JSON nulls for framed-only fields instead of
silently normalizing them to absence in a legacy context. Canonical old records
still omit those keys; current framed records, including count zero, retain
their exact values through serialization and deserialization.

The reviewed adapter-family digest is
`c33b5beb0f1869341bf24493e867f5d3101a578a12962b525c2613c1e09e3bbe`.
It binds every admitted arity's guards, scalar construction, exact single source
call and argument order, and return sequence;
each object also binds its selected rendering. The complete arena bound charges
64 root-view bytes, at most 24 bytes per input, the 32-byte header and complete
source evaluation. The 65536-byte arena, 4096-byte combined stack, 256-view,
64-function and 256-bit proof limits are unchanged.

Actual file-position compilation proves 1856 source-stack bytes plus 88 live
caller bytes and 32 alignment/count-save bytes: 1976 combined bytes. Its complete
allocation bound remains 60152 bytes. The two-word adapter digest is
`70e7230605470670149bd4c9a3832ca5ff02d7b38b4509cc5fee3ad82c4269d4`.
These are conservative static bounds; physical observations remain separate.

## Validation

Current artifact/root/provider/object/boot/gate revisions are
249/235/236/228/149/249. Complete bounded workspace validation passes, including
621 language, 253 compiler unit, 261 CLI and 173 semantics tests, eight existing
action integrations, two Unit-data integrations, the new framed native/GDB
integration and all remaining suites. All workspace targets, workspace
formatting and explicit included-source formatting pass. Physical regression
and final evidence checks pass. The fresh primary kernel SHA-256 is
`f68708d7fe75e4c5effc50c19e2f4fa8a8d8aa3404d6bc5b07421ee2af8985a8`.

All 134 QEMU observations pass, including 40 unmodified BusyBox workloads,
seven root-region cases and twelve persistent-action cases with the separately
selected legacy transport. The general positive cohort contains 70 observations
sharing the primary kernel and 21 independently identified positive variants.
Controlled states and expected refusals retain their separate oracles. All 40
Python lab contracts pass, as do checks of 294 unique normative rules, relative
links, exact code/data/manifest correspondence, retained ABIs/resource bounds
and whitespace. A fresh build and physical execution after strict-reader
hardening reproduce the exact primary kernel, preserving the completed native
cohort. The final explicitly aligned native C/GDB fixture also passes all five
arities, 48 vectors and sixteen entry refusals per arity.

A 694-MiB bounded rebuild was killed by its cgroup memory cap. Investigation
identified rustc memory pressure; subsequent host builds keep one Cargo job
and use 1024 smaller code-generation units, without changing memory limits, swap prohibition
or the host reserve. The final full Rust run passes through the same wrapper.

The [public native integration](tools/toolchain-gate-builder/tests/region_frames.rs)
builds real source objects for 0, 1, 2, 3 and 16 facts. For each count the
[independent C caller](labs/qemu/x86_64/region_frame_client.c) executes 48 boundary
and generated vectors. Nonempty frames end exactly at an inaccessible guard
page and remain read-only during invocation. A separate unsigned-128-bit
weighted checksum covers full-u64 values and order, with owner canaries, native
Boolean results, generation/revocation and complete caller-staging/arena cleanup.
Sixteen refusal cases per count check entry count, owner/generation/state,
canonical empty/nonempty pointers, alignment, overflow and control/owner overlap
before source allocation or owner mutation.

The [GDB oracle](labs/qemu/x86_64/region_frame_debug.py) independently compares
all actual ordinary Nat parameters with the captured frame on every successful
call, checks their arena/read-only extents and the real view, and observes
stack/GPR/SIMD preservation and revoked control at the saved return address.
Standalone caller evidence does not qualify kernel retired-stack reclamation.

The [linked integration](tools/toolchain-gate-builder/tests/region_actions.rs)
rejects altered actual instructions/constants/caller code under a corrected
outer checksum and fifteen metadata variants, including wrong count, family,
ABI and invocation profile. Legacy native tests remain; Unit/action data readers
also reject framed metadata. Frontend tests check exact counts/types, maximum
arity, visibility/features and unchanged syntactic operand limits. Code-only
zero-input graphs remain valid without immutable data.

The [physical record](labs/qemu/x86_64/results/topal-file-position-region-gate.json)
observes actual owned frame words and ordinary source parameters, shared field
writes, unchanged staging until source return, preservation, complete cleanup,
noncalls and active/inactive finalization. Its separate legacy run explicitly
selects and verifies the frozen source-object ABI; the harness rejects accidental
selection of the framed profile. Controlled failures require completed debugger
disconnect and QMP confirmation of the stopped guest before teardown.

The FIFO failure-injection run exposed the same result-file/teardown race in
the older shared child observer. Its controlled child, pair and FIFO cases now
wait for successful GDB exit after disconnect, exercise a deliberate 0.2-second
exit delay and independently confirm the QMP debug/stopped state before teardown.
The records bind the shared observer and QMP helper. This repairs the harness;
the kernel's instructions, LLVM and canonical metadata representation remain
unchanged by that repair and the strict-null reader checks.

Risk is high at declared selection, frame lifetime/aliasing, full-width transport,
complete resource charging, cleanup and private-format compatibility. Self-review
checks the authoritative ordinary/core and systems intent, generated family and
caller code, independent native/physical observations, strict packaging refusals
and legacy execution. Human PR review remains required; no independent reviewer
result is claimed. The [status ledger](step-4-status.md) retains the remaining
initial-plan work.
