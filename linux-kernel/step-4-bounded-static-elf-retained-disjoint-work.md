# Step 4 bounded static ELF retained-process disjoint work

## Outcome

The qualified x86-64 path now performs one resource-bearing operation while
the complete static ELF process remains behind its opaque affine owner. The
existing sealed allocator selects one fresh aligned frame with the whole
retained process as its exclusion interval, releases only that frame, performs
a complete retained-process readback, publishes
`TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_OK`, and then restores the exact
random-slot-authorized construction and nested stack backing.

The checked source sees only the retained owner, a one-frame request, and a
sealed policy. The compiler preserves the private stack-backing identity in
aligned root storage while the provider uses the physical-base register for
the fresh extent. No address, allocator cursor, process resource, or target
representation is exposed.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-RETAINED-DISJOINT-WORK-001` adds no core-language
element. Negative tests reject substituted owner, extent, policy, and marker.
The affine semantic model requires the retained owner and closed construction
allocator channel. Structural tests prove complete retained-state readback,
future random-object zeros, absence of retained stores/fences/invalidation,
one additional frame allocation, exact release, and later restoration.

Artifact `/138`, provider and provider-object `/128`, root object `/127`, gate
`/140`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 358 operations; the provider plan contains 129; the
root contains 192 fatal edges, a 419-event semantic trace, 5,359 typed
relocations, 4,879 UART calls, 91 required text symbols, and 132 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
132 observations in order within the supported 60-second allowance and
produced identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `f29cfda764231713cde4eec344bd07e3c80d8dbb4fd1f76a22dfa20a6c94b51c`;
- boot image: `b9316296b4010723a358e042f79a34a70bbe26d75f5ebc31edce054b2f3ae2b0`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `a62725c7aa48f42df654efa5ff83fbd82410eda64ed0ee10a10a3e39dbd800bc`;
- boot provenance: `4836e91295c0f8d7f78485bfe6acc9e457b8e40622bbca509873329d1a9b0fab`;
- serial: `d239f43fa21dc9cca245d2c8daed96d24ac6bab1a3d5d1adf2da806272740a04`;
- evidence: `bfad6a0cfbceb7413e4141e541ff9c94d1dd14dccf5f424bb9e6cc04f40ef134`.

## Risk and remaining boundary

Risk is high because a reused or overlapping frame would silently corrupt the
retained process, while losing the private stack-backing identity would make
restoration unsound. Review covered complete unsigned interval exclusion,
aligned compiler storage, existing allocator-state advancement, exact fresh
extent release, complete process revalidation, exact restoration, structural
machine code, negative cases, and duplicate successful physical execution.

This increment performs only one immediate qualification allocation. It does
not yet carry the process across DMA or device work, consume entropy, publish
`AT_RANDOM`, initialize TLS, construct a user context, enter ring 3, execute
BusyBox, or add a syscall.
