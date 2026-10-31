# Step 4 reusable process `getrandom` service

## Outcome

The Topal kernel now refines the live generation-three ChaCha20 generator and
the reserved second output into
`topal.systems.entropy.user-random.bind-reusable/1`. The resulting affine
service belongs to the initial process for its complete lifetime. Its first
successful Linux x86-64 `getrandom` call consumes the already extracted output;
each later successful call performs a fresh ChaCha20 extraction, replaces the
persistent key before publishing output, and erases the complete 32-byte
provider output after copying at most 16 bytes.

Zero-length calls and `-EINVAL` or `-EFAULT` paths validate without advancing
the generator or delivery count. The initial ring-3 probe exercises the four
existing non-consuming cases followed by two successful 16-byte calls. It
reads both returned qwords after each success without constraining
secret-derived values. `exit_group(0)` requires both successes, then wipes and
verifies the output, generator key, and 32-byte service object before clearing
the process-private reference and continuing ordinary address-space teardown.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-USER-SERVICE-001` and
`TOPAL-SYSTEMS-USER-GETRANDOM-REPEATED-001` require exact service binding,
first-output reuse, fresh extraction for later successes, non-consuming
validation failures, output erasure, and process-exit teardown. The language,
semantic, provider-planning, artifact, and structural tests cover the exact
request, operation/effect identity, ordering, affine state transitions,
provider symbols, 160-byte entropy-storage layout, local extraction call,
delivery counter, complete erasure, and six ring-3 calls.

The checked root contains 144 fatal edges, an exact 307-event semantic trace,
2,831 relocations, and 2,447 UART calls. The sealed provider plan contains 84
operations. Artifact schema `/97`, provider and provider-object schemas `/89`,
artifact-root schema `/95`, boot-adapter schema `/57`, and gate schema `/99`
identify this contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. `TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK` is reachable only after
the first output is consumed, a fresh generator block is produced for the
second success, both selected prefixes are copied, both complete output objects
are erased, and process teardown verifies and erases the remaining secret and
service state.

Two independent pinned-QEMU runs reached all 75 ordered observations. Hashes
for the recorded run are:

- linked kernel: `ca93eda67691165b7fe586450b46957cd9cd714fc707a3251f0e43a14fa6aa39`;
- boot image: `bee11b332d7ae35330399247d714033ebe1375bdfd2d66b82eb99b6c5fb97c43`;
- boot resource: `62142c86eb53b1e3bba05d8459a48bfc0ed9790666e63a5ad70100da7ede13dd`;
- generated image: `78a0d7fc799aa08886ea47fa81f9aa3710be70228c83cf553a963b1ef74622cd`;
- supplied image: `eee808594bacab14716dcc7faa9ceac478d736728eba43289d1f71399eec8bad`;
- artifact provenance: `4f28115432996dd021ef51a2a5b9fc80e0a8040547a82a02b98f10098ec4cbe9`;
- boot provenance: `5552f98f3d2ec402bd1915ee28635e69bc0132194735c91e101cfc272e9791f6`;
- serial output: `8df8ae8c8366547b9dc037d5d7c57e5c7e7eba761f23713c4995f83701723830`;
- evidence: `dbb2ebafea5c6503a36f00ff0d7a7e8eb3593e504b6a374eab71f46547697f3a`.

## Risk and remaining boundary

Risk is high because this increment retains and evolves secret generator state
across user entries and adds erasure obligations to terminal teardown. Review
covered affine binding, first-versus-later extraction order, key replacement,
argument validation before generator advance, overflow and guard checks,
copy-before-wipe order, full-object erasure, counter publication, structural
inspection, and duplicate physical execution.

The qualified profile still limits each call to 16 bytes and one initial
process. Blocking before CRNG readiness, signal interruption, `/dev/random`,
`/dev/urandom`, vDSO `getrandom`, multi-block requests, reseeding, per-CPU
generators, batching, fork/exec inheritance, and concurrent callers remain
unavailable.
