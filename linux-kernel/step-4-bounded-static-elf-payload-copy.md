# Step 4 bounded static ELF payload copy

## Outcome

The qualified x86-64 path now consumes the zeroed 503-frame static-image
construction and populates it from the retained `static-user.elf` boot-resource
member. Checked source names only the opaque zeroed owner, retained source, and
sealed `bounded-static-elf-four-loads` plan. Success yields one affine populated
owner and publishes `TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_OK` before the
existing complete revocation and return protocol.

The provider revalidates the payload and metadata extents, all 503 live RW/NX
leaves, nine absent entries, complete initrd handoff, canonical ELF identity,
entry, program-header offset, count, and all eleven approved program headers.
It copies exactly the four approved file intervals, compares every copied byte
with the immutable source, scans every intervening gap and the complete BSS and
tail interval for zero, and rechecks the unchanged leaves before success. The
construction remains inactive and no address, byte count, loop, or archive
representation is exposed to Topal source.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-COPY-001` adds no core-language element. Source
negative tests reject the wrong owner, source, plan, witness, or populated
identity. The semantic model rejects copy before zeroing, revocation before
copy, duplicate or out-of-order ownership, and every live construction at
disposition. Provider-object inspection proves exactly four `rep movsb` copy
loops, four `repe cmpsb` comparisons, four preserved-zero scans, both leaf
checks, the fixed archive offset, and complete approved metadata comparison.

Artifact `/115`, provider and provider-object `/106`, root object `/104`, boot
adapter `/69`, boot-resource `/2`, and gate `/117` identify this executable
contract. The checked root contains 150 fatal edges, a 334-event semantic
trace, 3,403 typed relocations, 3,007 UART calls, and 91 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions reached the
copy witness in order, completed all 91 observations, remained in the terminal
fatal halt, and produced identical artifact and serial hashes. Recorded hashes
are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `00eb76cf4c4396e0388c50646b285d7b2670caa4cc6e077fe91fb62603d2d90d`;
- boot image: `5364fa225e7e5a1f01eeb8af5b8f972ae9a076da46c71a273cf903d344f04012`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `7c5386cfa3964c5ab87a53d7ed143f76ec95cc904ef4e367186df398a6f4ea0e`;
- boot provenance: `a7ec700d43bf6fac46e2f8a8384aaf322c5da098ebd95b0ceec7254d2d288915`;
- serial output: `31ed0f6a2f479a26406e253eb546def108d65ee7f442674d6709263624b2dae5`;
- evidence: `0df0a4a9241618428ecb563b87f08d4045390901bcd2611796257fb1049de5dd`.

## Risk and remaining boundary

Risk is high because an interval or ownership error could construct executable
content from the wrong source, retain allocator residue, or return frames while
provider access remains live. Review covered source/destination arithmetic,
complete copied and zero complements, compile/runtime metadata agreement,
stack cleanup on every failure path, affine state transitions, structural
machine-code inspection, and duplicate physical execution.

This increment deliberately revokes and returns the verified payload. It does
not retain populated ownership, apply final RO/X, RO/NX, RW/NX, or RELRO
permissions, initialize TLS or a process stack, attach an address space,
publish the static entry point, or execute the image. The next increment must
retain a populated construction long enough to apply and verify final W^X
roles before any active mapping or entry authority exists.
