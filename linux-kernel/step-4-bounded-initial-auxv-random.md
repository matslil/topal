# Step 4 bounded initial `AT_RANDOM` delivery

## Outcome

The Topal kernel now consumes opaque entropy output identity one through
`topal.systems.entropy.output.deliver-initial-auxv-random/1` after the first
conditioned CRNG extraction and before initial user entry. The checked
transition copies exactly bytes 0 through 15 into a distinct 16-byte-aligned
initial-stack object, publishes Linux x86-64 auxiliary tag 25 with its native
user pointer, discards bytes 16 through 31, wipes and verifies the complete
provider output, and refines the prepared address space to a single-use
entry-ready state.

The ring-3 probe validates the exact tag and pointer, reads both complete random
qwords without comparing secret data with a fixed value, validates the platform
and executable strings plus `AT_NULL`, and consumes the exact 336-byte initial
stack block. The provider keeps the delivery state separate from the existing
anonymous-frame pointer, so later anonymous `mmap` operations retain their
aligned frame authority.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-DELIVER-001` and
`TOPAL-SYSTEMS-USER-AUXV-RANDOM-001` require one well-formed delivery between
CRNG extraction and user entry. Negative tests reject a malformed extent,
omission, duplication, reordering, a substituted auxiliary tag, reuse of the
affine output, or entry without successful delivery. Provider-object inspection
proves both selected qword copies, complete output wiping, independent delivery
state, and the user-entry readiness check.

The checked root contains 141 fatal edges, an exact 302-event semantic trace,
2,749 relocations, and 2,378 UART calls. The sealed provider plan contains 82
operations. Artifact schema `/95`, provider and provider-object schemas `/87`,
artifact-root schema `/93`, boot-adapter schema `/55`, and gate schema `/97`
identify the contract. The boot adapter also masks both legacy PICs before
handoff, preventing a pending IRQ0 from bypassing the admitted interrupt model.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The new marker occurs after queue completion, entropy conditioning,
credit transfer, CRNG initialization and extraction, and verified delivery, but
before the first ring-3 syscall.

Two successful independent pinned-QEMU runs reached all 73 ordered
observations. Hashes for the recorded run are:

- linked kernel: `b6c5f12c9318b4c009410c86a149d43ce8cf9fcf17be47726a9b239a9a706f65`;
- boot image: `9987c089e76334e6daea74713f8c36493bf4504636497638ff73fbf8399817a4`;
- boot resource: `0f9c0874a54dc7017b8550ff4cb51f2f07fed9456e4082726eed593888a919b7`;
- generated image: `1adb890492d6d6c99c90a6ae694634fbcda6848a66abc54345007787762dd5fc`;
- supplied image: `6d96b573116746ab854355163347bdae3641994faf320f5b5c82d0d86517d953`;
- artifact provenance: `049e9344b7f9840fb1ed11fdb24527c897698951bca8e52319f8f47b65847ef2`;
- boot provenance: `add328da76f65a5e70172d945fcd1b4bc480d27a188185ea5c3695dc6fab9ca2`;
- serial output: `85536560a755f986bfd007b8b330ccd3e23947af4464d20933ed3e6d98d3eb26`;
- evidence: `84fb27a1b016629b96c94fcfb53e8e7d7ffa41c2adcca20257be7249ac0a0d8c`.

## Risk and remaining boundary

Risk is high because this increment exposes secret-derived bytes across the
kernel/user boundary and changes the initial process stack. Review covered
affine output consumption, exact selection and discard extents, copy-before-wipe
ordering, complete success and failure wipes, non-overlapping provider state,
the native auxiliary-vector layout, value-independent ring-3 reads, malformed
and reordered rejection, boot interrupt containment, and duplicate physical
execution.

`getrandom`, `/dev/random`, `/dev/urandom`, additional kernel consumers,
multi-block generation, reseeding, per-CPU state, batching, queue recycling and
renotification, asynchronous interrupt routing, reset, and recovery remain
unavailable.
