# Step 4 bounded static ELF mapping resources

## Outcome

The executable x86-64 systems profile now keeps the exact 503-frame static-ELF
payload extent live while allocating one separate unit-aligned metadata frame.
The checked source observes both affine owners through
`TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_OK`, releases the metadata frame,
then releases the complete payload before entering the interrupt-critical
continuation.

The provider receives the complete payload interval as opaque exclusion
evidence. It validates aligned exclusion bounds, advances a conflicting
candidate past the complete payload, and still requires the metadata frame to
fit one normalized E820 RAM run, exclude the boot resource and retained
reservations, and remain below the first-GiB bootstrap ceiling. Neither base is
source-observable and no byte access or leaf publication is granted.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-MAPPING-RESOURCES-001` adds no core-language
element. The language checker admits only the exact nested `(503, 1)` then
`(1, 1)` requests, the two ordered witnesses, and metadata-then-payload whole
release. Negative cases reject a two-frame metadata request, a wrong witness,
and reversed affine identities. The semantic model has a distinct two-resource
state which returns to payload-only ownership on the first release. Root and
provider object inspection proves complete-exclusion construction, retained
payload preservation, aligned bound validation, overlap comparison, candidate
advance, and balanced callee-saved-register restoration.

The checked root contains 146 fatal edges, a 326-event semantic trace, 3,272
typed relocations, and 2,884 UART calls. Artifact `/110`, provider and provider-
object `/102`, root `/101`, boot adapter `/68`, and gate `/112` identify the
executable contract.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 88
ordered observations with identical hashes. The new witness follows the live
503-frame witness and precedes both releases and the interrupt-critical scope,
so it is reachable only after the provider finds a disjoint metadata frame
while preserving the complete payload owner. Recorded hashes are:

- linked kernel: `914b1e2b81bb858f4ce9bd362e21e5d4cdb270cf45d064e9f0a54e5375864d49`;
- boot image: `aee9ae342b010ee484db856c5a41852f07cb717e6077a44c5fefb022b1e944ac`;
- boot resource: `3353a052b6e9a89540dbad2d2f26494bfb1b8701b3ee967cd4367f16054af008`;
- generated image: `994132429fc84cf9cdbe48042a666418ca4e562f9529b9b62ba6b9f807b627a7`;
- supplied image: `adf24b0776be2281f2b634fa3f1493087e06b489925ce3e8fbd890ddae91b022`;
- artifact provenance: `d0b4fb7ed41e9ec86245104d66abd58e955dca86b6e472e8a226e17fb4dbecd1`;
- boot provenance: `3d69e99df76b45f81b1b6e70c1f5fcf9db1b867f82ba88feb31f8f4b30abb3ad`;
- serial output: `263e7d60ce471481ed0a6e8bf50b73dfac299321b7be97e01096dcf279dfb8aa`;
- evidence: `e04ccf9da57ea363e518a1371562c44f4d8dbb48038da0f9c9793e4218437b25`.

## Risk and remaining boundary

Risk is high because two physical owners are now simultaneously live and the
provider exclusion ABI changed from an assumed size to explicit interval
bounds. Review covered fail-closed interval validation, overflow and alignment,
whole-range E820 containment, reservation exclusion, register preservation,
exact nested source ownership, unwind order, object structure, and duplicate
physical execution.

Both extents are still immediately released. The next increment must transfer
them into an opaque static-image construction owner and install the 503 initial
RW/NX leaves without exposing target page-table representation. Deterministic
zero/copy, final W^X and RELRO permissions, TLS, stack, entry, and BusyBox
startup remain later work.
