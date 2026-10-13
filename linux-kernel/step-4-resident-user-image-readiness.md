# Step 4 resident user-image readiness increment

## Outcome

The bounded resident image now publishes an exact physical readiness witness
while both finalized mappings are still live. Source emits
`TOPAL_KERNEL_USER_IMAGE_READY` after generation-two immutable-data sealing and
before data-first teardown. This is a systems-feature qualification only; it
changes no core-language rule and grants no new mapping or access authority.

## Model and x86-64 artifact

The checked source requires the marker at the exact construction boundary.
The architecture-independent model additionally rejects missing, duplicate,
early, or late readiness: the marker is accepted only while the sealed RO/X
code mapping and sealed RO/NX data mapping coexist and source owns no returned
construction extent.

The privileged x86-64 provider and provider object remain at revision `/29`
because their mapping and console contracts are unchanged. Artifact revision
`topal.systems-artifact.x86_64-qemu-pc-q35/30` records 61 provider operations,
62 placements, and a 177-transition trace with the readiness observation
between immutable-data sealing and its unmap. Root revision
`topal.systems-root-object.x86_64/30` has 1,090 typed relocations, including
904 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical.
Gate schema `/32` records all 29 ordered markers, live fatal halt, and serial
quiescence.

- linked kernel: `d8c0b4660c09d260867e38d309ca76793518b8a2c10d09fc71c0ddcdd7c26069`;
- boot image: `08e464ef0958487c3e9151ebf17b44c234ac04567e1d05e97b68ebaa8faef518`;
- artifact provenance: `e35df002085a89f05a02c6f0762d0eda9cf3c02a5c766ca3094516e3b16e4987`;
- boot provenance: `be4ed1bdf0b96969f1dde1bce732cba4e7546725e29e3ec1e340198418eb22b8`;
- serial: `7247f2a67ada6794c9a72cc5e36aed31ccfb6c54855c0baf614e894ae8720de4`;
- evidence record: `265df3010d22aa04b0893788e7170fae4eea398361d7046d8a631ce50f7b7a34`.

## Remaining boundary

The witness proves finalized construction residency, not readiness for user
entry. Both mappings are still revoked before the existing sealed user probe.
Retaining an image into execution, invoking loaded code, arbitrary segment
collections, aliases, relocation, ELF parsing, and demand paging remain
excluded.
