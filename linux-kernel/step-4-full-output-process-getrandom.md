# Step 4 full-output process `getrandom`

## Outcome

The Topal kernel now accepts Linux x86-64 `getrandom` requests through the
complete 32-byte output produced by its qualified ChaCha20 transition. Both the
initial reservation and reusable process service name maximum delivery extent
32. The reservation selects the full output with an empty discard partition;
the service still consumes and wipes exactly one output for every successful
call and retains no remainder.

The ring-3 probe preserves zero length, unknown flags, incompatible flags, a
complete-range fault, and two successful 16-byte calls. It then requests 32
bytes, requires an exact return of 32, and reads all four qwords without
constraining secret-derived values. The added call uses a fresh extraction,
replaces the persistent key before publication, copies and verifies all output
bytes, wipes and verifies the complete output, and advances delivery state only
after success.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-GETRANDOM-FULL-OUTPUT-001` refines the existing
reusable-service and entropy-reservation rules without adding a new core-
language element. Language, semantic, provider-planning, artifact, and
structural tests cover the exact 32-byte reservation and binding, validation
before extraction, seven native calls, complete four-qword observation, output
erasure, delivery accounting, and unchanged terminal teardown.

The checked root contains 144 fatal edges, an exact 308-event semantic trace,
2,873 relocations, and 2,489 UART calls. The sealed provider plan contains 84
operations and retains the 160-byte entropy-storage layout. Artifact schema
`/98`, provider and provider-object schemas `/90`, artifact-root schema `/96`,
boot-adapter schema `/58`, and gate schema `/100` identify this contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. `TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK` is reachable only after
the prior two successful calls, a fresh generator transition, an exact 32-byte
copy, observation of all four result qwords, and complete output erasure.

Two independent pinned-QEMU runs reached all 76 ordered observations. Hashes
for the recorded run are:

- linked kernel: `56c6cb8085bd433fb9bcbeb9d394be8c05560dc75d8a1619841867e7e048b8af`;
- boot image: `e2f6944cd68e14c560858bd4d66403a12ef9bfaeb91f767d28e8550c6df6eaad`;
- boot resource: `3b52b43cf4a115310dd7957429aeb4837f7e22ddbbb65f3da1fa716d106ebc17`;
- generated image: `eee93c16e06ac08930822e9eda6c09fa82d43995d7be5592dc6db6405e400298`;
- supplied image: `82afbc1dca4d529714fd1c28cf1e1f8beadb942babf684132687d130236b8b72`;
- artifact provenance: `d833c126017a259a1172d5905b7c3568950130ae4802736f3ed8d9a7314aadde`;
- boot provenance: `5f247c1ab0b0220710c05a16b28050c6dd94fe05fd70903a2f32b9a124c9b0a3`;
- serial output: `2784d7abc666068a47875af3833fa9341792b97312b26518e85e179f9ede06ec`;
- evidence: `decd4636d21eb95f8d73a49d8a69fe983102df0d1bbd88f9bc129e541ecccadd`.

## Risk and remaining boundary

Risk is high because the refinement expands secret output copied through a
recoverable user boundary and changes both reservation and live-service
limits. Review covered exact partition arithmetic, validation before state
advance, one-output ownership, key replacement, complete copy verification,
full-object wiping, delivery count publication, structural inspection, and
duplicate physical execution.

The qualified profile still limits each call to one 32-byte output and one
initial process. Multi-output requests, partial completion, blocking before
CRNG readiness, signal interruption, `/dev/random`, `/dev/urandom`, vDSO
`getrandom`, reseeding, per-CPU generators, batching, fork/exec inheritance,
and concurrent callers remain unavailable.
