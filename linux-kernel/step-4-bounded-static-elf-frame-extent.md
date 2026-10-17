# Step 4 bounded static ELF frame extent

## Outcome

The executable x86-64 systems profile now admits the exact `(503, 1)` physical-
frame request derived by the static-ELF page plan. The checked source allocates
it only after returning the earlier three-frame probe, observes it through
`TOPAL_KERNEL_STATIC_ELF_FRAME_EXTENT_OK`, and releases the complete extent
before entering the interrupt-critical continuation.

The extent remains one opaque affine owner. The x86-64 provider converts the
count to 2,060,288 bytes, selects the complete page-aligned interval inside one
E820 RAM run, excludes the boot resource and retained reservations, and keeps
the end below the qualified first-GiB bootstrap mapping. No byte access, split,
or mapping is granted.

## Validation boundary

Rules `TOPAL-SYSTEMS-FRAMES-001` and
`TOPAL-SYSTEMS-STATIC-ELF-FRAMES-001` add no core-language element. The language
checker admits only counts one, two, three, and exactly 503 at unit alignment;
502, 504, other intermediate counts, and non-unit alignment fail before
lowering. The semantic model retains indivisible ownership and whole release.
The root and provider objects contain the exact 503 immediate, complete-range
E820 comparison, reservation exclusion, and matching allocate/release calls.

The checked root now contains 145 fatal edges, a 323-event semantic trace,
3,226 typed relocations, and 2,840 UART calls. Artifact `/109`, provider and
provider-object `/101`, root `/100`, boot adapter `/68`, and gate `/111`
identify the executable contract.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 87
ordered observations with identical hashes. The new witness precedes the
critical-scope and boot-memory observations, so it is reachable only after the
complete 503-frame selector succeeds and before source consumes whole release.
Recorded hashes are:

- linked kernel: `dd7bea953d294f16b4c50a22c5199c5c0f4abb18aefee9f3ab7d12bf7309b9d5`;
- boot image: `19882d672e7236c6772f987681016db588831a1dd8e285c9795b96d1646db4d1`;
- boot resource: `3353a052b6e9a89540dbad2d2f26494bfb1b8701b3ee967cd4367f16054af008`;
- generated image: `994132429fc84cf9cdbe48042a666418ca4e562f9529b9b62ba6b9f807b627a7`;
- supplied image: `adf24b0776be2281f2b634fa3f1493087e06b489925ce3e8fbd890ddae91b022`;
- artifact provenance: `98a8c202c01cb8aeb3c7a9167dde8f273aa9c6348729b625ff2f1018e4326cb8`;
- boot provenance: `9076a57edc7b1c53215dd691f715f08715fa4a4d41a558efcf7e31e2199e5eab`;
- serial output: `71f9d7d6f2219dd302215a80ddc1b2ba8c2a05715bcb6a0d41d518c1a0d1636e`;
- evidence: `df9bab764c38a3f95494730c4a5a640812d37f4b7399c9053809725fed2222fe`.

## Risk and remaining boundary

Risk is high because the provider now selects and owns a multi-megabyte
physical interval. Review covered exact source admission, model ownership,
checked byte conversion, E820 whole-range containment, reservation exclusion,
the first-GiB ceiling, fail-closed unsupported counts, exact release, object
structure, and duplicate physical execution.

The extent is immediately released without mapping. The next increment must
transfer it into a bounded static-image construction owner and publish the 503
page-table leaves without exposing physical or virtual integers to Topal
source. Copy, zero, final permissions, TLS, stack, entry, and BusyBox startup
remain later work.
