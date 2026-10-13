# Step 4 allocator-backed user-probe increment

## Outcome

The initial user execution now consumes one allocator-owned contiguous
three-frame extent instead of relying on hidden provider-owned physical pages.
The checked root allocates the extent, transfers it into preparation and user
execution, receives the same indivisible extent only after terminal completion
revokes its mappings, and then releases it. A new
`TOPAL_KERNEL_USER_BACKING_OK` marker is unreachable until that lifecycle has
completed.

This is a systems-feature refinement only. It adds no grammar, numeric address,
pointer conversion, source instruction, or core-language rule. Portable source
observes an affine backing extent and user-execution lifecycle; the provider
retains target page-table formats, virtual placement, publication ordering,
invalidation, and generated probe bytes.

## Checked source and semantic model

The source checker requires the exact three-frame result from the qualified
allocator call to be passed as `backing` to user-probe preparation. Successful
preparation consumes that extent. Terminal user completion must return the
exact extent, and the root must release it before publishing the success
marker. Negative tests reject a wrong backing identity, wrong release identity,
missing completion result, and unsupported request shape.

The executable semantic model carries the complete
`PhysicalFrameRequest(3, 1)` through prepared and completed user states. It
removes that request from source-owned physical memory while execution owns it,
returns it only on terminal completion, and requires the ordinary allocator
release transition afterward. The architecture-neutral extent remains opaque
and indivisible.

## Initial x86-64 lowering

Provider, provider-object, root-object, and artifact revisions advance to
`/20`. Preparation writes the generated probe through the supervisor identity
mapping, clears the extent's third frame, constructs sealed code and stack
leaves there, and publishes that leaf table into the active hierarchy. The
code frame is read-only and executable; the stack frame is writable and
non-executable; translation metadata is not user-accessible; and the next user
page remains an unmapped guard. The old fixed physical code and stack backing
is absent from the replacement translation-space builder.

After `exit_group(0)` restores the registered kernel continuation, the new
completion provider removes the user hierarchy entry, orders publication,
invalidates the code, stack, and guard virtual locations, clears its retained
backing identity, and returns the original physical base to the checked root.
The root reconstructs the exact opaque three-frame extent and releases it.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/20` records 52
provider operations, 52 inspected placements, and a 156-transition semantic
trace. Root-object revision `topal.systems-root-object.x86_64/20` has 955 typed
relocations, including the added allocator, completion, release, failure, and
marker paths. Boot-adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/6` is unchanged because the
handoff and entry contracts did not change.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` builds and executions were
byte-identical. Gate schema `topal-kernel-toolchain-gate-qemu/22` records 26
ordered markers. The final `TOPAL_KERNEL_USER_BACKING_OK` marker proves that the
allocator-backed user probe completed, its mappings were revoked, the exact
extent was returned, and source released it. The evidence binds:

- linked kernel:
  `dce1cbafcc3982d5b337a0435d305a402b444db856379d278b1f62276da53d6a`;
- boot image:
  `005e913f399209d565f7366842412e3fbb87638c8b992b3a24adae21c86d992e`;
- artifact provenance:
  `fc075866efef22bddadfa7992e105c2fdbf253eb0f6ce1540b66b49d38ed5914`;
- boot provenance:
  `75ae41b3a5b6d4f22f0113c85fce251cb77cbe6fa8e9e2c3d0f5218fc44d570a`;
- serial observation:
  `fdb1c01e032b40867664b8edec4f24612676e6539ab3d5167e2cf09e1a807525`;
  and
- complete evidence record:
  `e946d2b9d265480b1b21ce07f5fabf972b7fcb2e794ae29b7fbefb00695ebf2d`.

QEMU remained live in the final fatal disposition and its serial output stayed
quiescent after all markers.

## Validation and review

This is a high-risk physical-memory ownership, translation, and privilege
boundary change. Review covered authority-order conformance, systems-only
scope, exact affine ownership transfer, executable-versus-writable separation,
non-user translation metadata, guard preservation, complete revocation,
translation invalidation, structural provider inspection, negative tests, two
physical executions, and the complete repository test surface. Final
validation uses:

```console
scripts/run_bounded.py -- cargo fmt --all -- --check
scripts/run_bounded.py -- cargo check --workspace --all-targets
scripts/run_bounded.py -- cargo test --workspace -q -j1
scripts/run_bounded.py -- python3 \
  linux-kernel/labs/qemu/x86_64/topal_gate.py --write-evidence
python3 -m unittest linux-kernel/labs/qemu/x86_64/test_topal_gate.py -v
git diff --check
```

## Remaining boundary

This increment does not add a general user address-space builder, ELF loading,
multiple processes, dynamic user layouts, subextent ownership, simultaneous
source-live extents, reclaim under concurrency, copy-on-write, demand paging,
SMP translation shootdown, or general userspace. The next dependency-ordered
increment can introduce a source-owned opaque user address-space lifecycle
while retaining the qualified allocation and revocation boundary.
