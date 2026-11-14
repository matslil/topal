# Step 4 bounded static ELF payload zeroing

## Outcome

The executable x86-64 systems profile now refines the live inactive static-ELF
construction into an owner whose complete 503-frame payload is verified zero.
The provider first revalidates both resource intervals and every live initial
leaf, writes zero across all 2,060,288 payload bytes, reads back that complete
extent, and verifies the unchanged 503 RW/NX leaves and nine absent entries.

The checked source observes the refinement through
`TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_OK` and only then invokes the existing
complete revocation, metadata release, payload reclaim, and whole-payload
release sequence. The object remains inactive, ordinary source gains no byte
or address access, and this increment copies no ELF file bytes.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ZERO-001` adds no core-language element. The
language checker admits only the exact live construction, complete-extent
selector, zeroing witness, zeroed-owner revocation, and existing affine return
order. Negative cases reject another owner, partial extent, wrong witness, or
revocation through the consumed uninitialized identity. The semantic model
requires the live-to-zeroed refinement before revocation and rejects either
state at disposition.

Root inspection proves one typed zeroing dependency and checked failure edge.
Provider-object inspection proves the exact 257,536-qword write and readback
bounds, leaf validation before and after, one ordering fence, and no exposed
address. The checked root contains 149 fatal edges, a 332-event semantic trace,
3,360 typed relocations, and 2,966 UART calls. Artifact `/112`, provider and
provider-object `/104`, root `/103`, boot adapter `/68`, and gate `/114`
identify the executable contract.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 90
ordered observations with identical hashes. The new witness follows initial
leaf publication and precedes complete revocation, so it is reachable only
after the provider's complete physical zero/write-readback and unchanged-leaf
verification succeed. Recorded hashes are:

- linked kernel: `f2eb541f12360b823ac89d4942ea3d6512a786116dc8f2088da5f5be5be048b8`;
- boot image: `fb1ce61426d5a8fc8decf7d5dd269a0e891408cabd226353d3d77f5d78a25899`;
- boot resource: `3353a052b6e9a89540dbad2d2f26494bfb1b8701b3ee967cd4367f16054af008`;
- generated image: `994132429fc84cf9cdbe48042a666418ca4e562f9529b9b62ba6b9f807b627a7`;
- supplied image: `adf24b0776be2281f2b634fa3f1493087e06b489925ce3e8fbd890ddae91b022`;
- artifact provenance: `8a75adbf6104e38b5f8b2f0b167eae7bd81c3a31dcb4d8e1e12df1d4c7428b5f`;
- boot provenance: `dcae21e3c8c09019845c7ced65d0ec669008f10a5ff822df0b168f74ea162fb6`;
- serial output: `bde7294a4b0ba8dbc89ae3bcd350c8c5a279c951c4e6f4312e7c6d0391609bf6`;
- evidence: `2b9accd7357468fcacba121735ce84fecc24464fb5d2a3c00b8e70c73bbbafb6`.

## Risk and remaining boundary

Risk is high because a wrong bound could erase unrelated kernel or boot state,
while incomplete initialization could expose allocator residue as future
segment padding or BSS. Review covered interval revalidation, exact byte/qword
arithmetic, complete write and readback loops, inactive leaf preservation,
affine state ordering, structural tests, and duplicate physical execution.

The zeroed construction is still revoked before use. The next increment must
select a digest-bound static ELF input without importing or redistributing it,
then copy and verify the four approved file intervals into the zeroed payload.
Final per-page W^X and RELRO permissions, TLS, stack construction, active
address-space attachment, entry publication, and BusyBox startup remain later
work.
