# Portable process memory model

This is the stage-1 reference foundation for `TK-DEC-284` and
`TOPAL-SYSTEMS-PROCESS-MM-001`. It executes no privileged instructions, guest,
compiler or application binary. Its Python objects and frame numbers do not
construct native memory authority.

[model.py](model.py) models independent process/MM identities, image generations,
heap bytes, whole extent issuance, active/inactive retention, raw break results,
creation, cooperative handoff, exec, wait/exit and outer cancellation. All live
extents are logically pinned. Its transient events represent allocation,
zero-before-grant, pin, mapping publication, complete abstract invalidation,
unpin, erasure and release. The first-fit allocator is a reference implementation,
not a requirement on ordinary Topal policy or native placement.

[trace_check.py](trace_check.py) separately reconstructs held/pinned frame sets
and visible/cached translations from those events. It rejects missing,
reordered, foreign, aliased, incompletely erased or duplicate effects. Cached
translations are an abstract conservative state, not an observation of hardware
TLBs. A zeroing event contains zero bytes; it proves a reference transition,
not a physical write.

[test_model.py](test_model.py) has seventeen tests. Its application oracle tracks
only process-local byte arrays, breaks and generations; it shares no physical
placement, token, pin or event algorithm. Thirty-two deterministic seeds each
exercise 100 operations. Separate fixtures cover 0/1/33/34/35/67/519 pages,
partial-break page slack, different page sizes, exhaustion/fragmentation,
abandoned prepared extents, both processes' exec, wait eligibility/output retry,
recreation and exact cancellation. Nineteen wrong saved histories and seven
inconsistent saved states must fail. These controls alter model data only.

Run from the repository root, retaining the normal resource wrapper:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/qualify.py --output linux-kernel/.cache/x86_64/process-memory-check-new
```

Use a new output directory. The qualifier records complete unit-test output,
selected source hashes, local link/stable-ID checks and the actual cgroup's
limits/events. It refuses an unbounded or swap-enabled environment. The committed
[stage-1 result](results/stage-1/report.json) records one run, not proof that
later source revisions or the kernel are qualified.

Trace: [language intent](../../../docs/systems-profile.md#process-address-space-ownership),
[requirement](../../../se/requirements.md#topal-req-systems-process-mm-001--independent-process-memory-ownership),
[formal rule](../../../spec/systems-profile.md#topal-systems-process-mm-001--process-address-space-ownership),
[compiler obligation](../../../src/topal-compiler/se-requirements.md#topal-comp-systems-process-mm-001--process-memory-owner-binding),
[design](../../design/process-memory-ownership.md),
[stage report](../../step-4-process-memory-ownership.md).

The model omits image/stack contents, register/TLS/SIMD/descriptors, actual table
walks and permissions, faulting copies, native instruction/entry/stack binding,
physical erasure, identity-width exhaustion, partial-mutation fatal cleanup and
AP/DMA/asynchronous behavior. It models only the selected nonshared heap and
normal/prepublication-unavailable transitions. It provides no evidence for those
omitted obligations or for distribution, containers, virtualization or general
scheduling/MM. Native integration and public source/load/boot reconstruction
must preserve the existing qualified child and pipe workloads before activation.
