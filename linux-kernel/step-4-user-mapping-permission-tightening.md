# Step 4 user-mapping permission-tightening increment

## Outcome

Both bounded user-mapping generations now transition from RW/NX to RO/NX
before their verification load. The transition consumes the exact stored
mapping refinement, revokes writable authority, completes target translation
invalidation, and returns a protected affine refinement.

This is a systems-feature refinement only. It changes no core-language
grammar, type rule, pointer conversion, source-visible page-table operation,
or instruction name. Rights representation, page-table layout, target
ordering, and invalidation remain provider-private.

## Checked source and semantic model

The checked root requires `map`, `store`, `protect read only`, `load`, and
`unmap` for each generation. Missing, duplicated, reordered, stale, or
cross-generation refinements fail closed. The architecture-independent model
adds a protected ownership state between stored and verified; a load can no
longer consume writable authority.

The generic user protocol preserves the mapping identity while consuming
`Stored` into `Protected`. Both generations retain fresh opaque mapping
identities and return the exact original one-frame extent only after verified
unmap completion.

## Initial x86-64 lowering

Provider, provider-object, root-object, and artifact revisions advance to
`/26`. The generated protection function accepts only provider states 5 and 9,
clears the x86 writable PTE bit, orders the update with `mfence`, invalidates
the sealed guard address with `invlpg`, and publishes protected states 12 and
13. The verification load accepts only those protected states and preserves
the existing success and failure cleanup behavior.

The generated root calls the typed protection provider twice and admits load
only from its protected state. Structural inspection requires the writable-bit
clear, ordering fence, invalidation, both generation states, both call sites,
and typed fatal paths.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/26` records 59
planned provider operations, 60 inspected placements, and a 172-transition
semantic trace. Root-object revision `topal.systems-root-object.x86_64/26` has
1,052 typed relocations. Boot-adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/6` is unchanged.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` builds and executions were
byte-identical. Gate schema `topal-kernel-toolchain-gate-qemu/28` records 28
ordered markers. `TOPAL_KERNEL_USER_MAPPING_OK` is reachable only after both
RW-to-RO transitions, protected loads, unmaps, and exact final frame return.
Later intentional guard-crossing user-copy faults also prove the mapping was
not left accessible. The evidence binds:

- linked kernel:
  `2a4c368c07d16f80dd28bcfc79e93350a7f5dbf7fd31b67b4521861983f7bdcc`;
- boot image:
  `cd35c5c0803897f1964cb44da94c48142cf25ded8f03f33349e947de5262c485`;
- artifact provenance:
  `486b9613c2ebca50cb82c2f7a39f8ba7c0efa6a653812f93d22c16dc29748d50`;
- boot provenance:
  `420c5fd560486f0d61b60f00c7333b5497c1c5e3af4815963b70d95ae01f7473`;
- serial observation:
  `968b9aaafe358dde292b500a8d4b997bcf274d212e77a1618638003932f3b0dc`;
  and
- complete evidence record:
  `87cc3969c50130124b007d83e868f59dc63b0fb24667c3d639068657c98fbb02`.

QEMU remained live in the final fatal disposition and its serial output stayed
quiescent after all markers.

## Validation and review

This is high-risk permission, translation-completion, and affine-ownership
work. Review covers authority-order conformance, systems-only scope, writable
authority revocation, stale-refinement rejection, both generation states,
provider instruction structure, typed relocation closure, negative tests, two
physical executions, and the complete repository test surface. Final
validation uses:

```console
scripts/run_bounded.py -- cargo fmt --all -- --check
scripts/run_bounded.py -- cargo check --workspace --all-targets
scripts/run_bounded.py -- cargo test --workspace -q -j1
scripts/run_bounded.py -- python3 \
  linux-kernel/labs/qemu/x86_64/topal_gate.py --write-evidence
scripts/run_bounded.py -- python3 -m unittest \
  linux-kernel/labs/qemu/x86_64/test_topal_gate.py -v
git -c core.whitespace=trailing-space,space-before-tab diff --check
```

## Remaining boundary

The profile still admits one sealed guard location, a fixed byte, two
sequential generations, and no more than one live mapping. It does not admit
arbitrary rights, executable mappings, permission relaxation, simultaneous
mappings, aliases, replacement, demand paging, copy-on-write, or an ELF
loader. Those remain later systems contracts.
