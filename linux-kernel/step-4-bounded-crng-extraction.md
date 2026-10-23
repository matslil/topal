# Step 4 bounded CRNG extraction

## Outcome

The Topal kernel now consumes ready entropy-generator identity one at generation
one through `topal.systems.entropy.generator.extract/1` and returns ready
generation two plus opaque output identity one. The checked transition computes
one standard 20-round ChaCha20 block with zero counter and nonce, replaces the
persistent key with the first 32 bytes before publishing the second 32 bytes,
and emits `TOPAL_KERNEL_ENTROPY_OUTPUT_READY` without delivering a byte to an
interface.

The x86-64 provider verifies the complete successor key and output, wipes and
verifies all 128 bytes of temporary input and working state, and fails closed by
wiping persistent key and output storage on any partial failure. Source cannot
observe either storage address, any secret byte, the word representation, or a
target instruction.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-EXTRACT-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-EXTRACT-001` require exactly one extraction
after initialization. Negative tests reject omission, duplication, reordering,
substituted generator, algorithm, generation, output, extent, split, counter,
nonce, readiness state, or malformed arithmetic. The prior generation and
output object are affine and cannot be reused.

The checked root contains 140 fatal edges, an exact 300-event semantic trace,
2,707 relocations, and 2,340 UART calls. The sealed provider plan contains 81
operations. Artifact schema `/94`, provider and provider-object schemas `/86`,
artifact-root schema `/92`, boot-adapter schema `/54`, and gate schema `/96`
identify the contract. Structural inspection proves 80 occurrences of each
ChaCha20 rotation and the ordered key/output publications; an independently
executed zero-key known vector proves the complete generated block.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The new marker follows successful sample capture, conditioning,
credit, generator initialization, successor-key replacement, opaque-output
publication, and temporary-state destruction.

Two successful independent pinned-QEMU runs reached all 72 ordered
observations. Hashes for the recorded run are:

- linked kernel: `4f3dffdeba4ff9e3a7b6c0460f9e1beb73c9b6241f2ee20dd417a511f1a0341e`;
- boot image: `7acd2418f084d8630b7f0d3c8b999b5ca26fa3705dc3e5fb5dcbf9b0f511c419`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `87f6151b236077ba6af8f98e4ac5d5bb74b87ab61d27e0433d69bc85af45a324`;
- boot provenance: `1e97333b024932911b07227b58b2a51a6f77e708978b97ace0a76434e9de7c56`;
- serial output: `315c2398dd1089dcf7070160384c4987e3190aa8dcea474c49805c33ce861a8b`;
- evidence: `2546f68a8d16c392807ee969dd7523b85a3d50b6247aef10a5941da2acd1d9a8`.

## Risk and remaining boundary

Risk is high because this increment implements cryptographic expansion, live
key replacement, and the first random-output object. Review covered the exact
ChaCha20 construction and little-endian encoding, affine generation advance,
key-before-output order, complete publication checks, temporary and failure
wipes, malformed and reordered rejection, structural instruction inspection,
an independent executable known vector, and duplicate physical execution.

`AT_RANDOM`, `getrandom`, `/dev/random`, kernel-consumer delivery, multi-block
generation, reseeding, per-CPU state, batching, queue recycling and
renotification, asynchronous interrupt routing, reset, and recovery remain
unavailable.
