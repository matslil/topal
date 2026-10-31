# Step 4 bounded multipage initial executable

## Outcome

The Topal kernel now constructs and executes its sealed initial x86-64 image
from one affine, physically contiguous two-frame code extent. The executable
segment spans `0x603000..0x605000`; its ELF headers and deterministic padding
occupy the first page, while entry at `0x604100` and the successful terminal
path execute from the second. The distinct one-page RW/NX data role remains at
`0x605000`.

Both code leaves are initially RW/NX. The provider copies and verifies the
complete runtime-derived segment, removes write authority from both leaves,
verifies the complete read-only extent, and only then grants execute authority
to both. Publication, permission changes, execution promotion, revocation, and
teardown invalidate both virtual pages. Unmap returns the original two-frame
extent as one object; no partial ownership or W+X state is admitted.

## Validation boundary

Rule `TOPAL-SYSTEMS-MULTIPAGE-EXECUTABLE-001` adds no core-language element.
The approved systems profile, requirements, specification, language checker,
semantic model, checked root, artifact validator, and x86-64 provider all
require the exact two-frame code role. Structural tests cover adjacent leaf
publication, two-page invalidation and revocation, runtime ELF bounds, the
second-page entry, deterministic padding, whole-extent release, and rejection
of wrong frame counts or entry placement.

The checked root contains 144 fatal edges, a 320-event semantic trace, 3,185
typed relocations, and 2,801 UART calls. The provider plan retains 84
operations and 160 entropy bytes. Artifact `/108`, provider/provider-object
`/100`, root `/99`, boot adapter `/68`, and gate `/110` identify this contract.
The boot resource grows to 12,528 bytes and carries the exact 12,288-byte
supplied ELF.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 86
ordered observations with identical hashes. The new
`TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK` witness appears only after entry,
all executable work, terminal `exit_group(0)`, and exact kernel return from the
second page. Recorded hashes are:

- linked kernel: `a6a1e9273efb8ac2ae915c644600419d21f5f5c173ec87ee57a9c38f73626501`;
- boot image: `6b39d46dda6c981126e9b171faa2192aa059c1ceeca7e739b814adda196184ee`;
- boot resource: `3353a052b6e9a89540dbad2d2f26494bfb1b8701b3ee967cd4367f16054af008`;
- generated image: `994132429fc84cf9cdbe48042a666418ca4e562f9529b9b62ba6b9f807b627a7`;
- supplied image: `adf24b0776be2281f2b634fa3f1493087e06b489925ce3e8fbd890ddae91b022`;
- artifact provenance: `3ccb97bf81131cc722db89bc31ffc42d9f56574364790fda6529543d6bac44d9`;
- boot provenance: `c13d61c618ca4e5be925d03f363186d5bf266bea763fd2560fb4f38a0825488a`;
- serial output: `ac6f1f7eef48dccb7f9214e330bfe7739bbf2aa989a6728360d54711bf16f463`;
- evidence: `926ce7f193c6168b30e28121c3fb2a56d2434aaab846777556da58c65e2abf8d`.

## Risk and remaining boundary

Risk is high because this increment changes executable-page ownership,
permission transitions, translation invalidation, and teardown. Review covered
the authority-ordered design and rule, exact frame-count propagation, all
provider state transitions, complete ELF-copy bounds, entry containment, W^X,
whole-extent return, focused language/semantic/compiler suites, structural
instruction checks, and duplicate physical execution.

This remains a deliberately sealed two-page image, not a general ELF loader.
It does not admit arbitrary segment counts, sparse mappings, demand paging,
PIE, relocations, `PT_INTERP`, dynamic linking, TLS, filesystem-backed
execution, or the multi-megabyte static BusyBox image.
