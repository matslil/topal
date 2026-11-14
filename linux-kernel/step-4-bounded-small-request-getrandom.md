# Step 4 bounded small-request `getrandom`

## Outcome

The Topal kernel now accepts complete Linux x86-64 `getrandom` requests from
one through 256 bytes. A request is validated as one complete user range before
the entropy service changes state. It is then divided into at most eight
32-byte outputs; every output after the first is produced by a fresh ChaCha20
transition that replaces the persistent key before publication.

The provider copies and verifies each bounded chunk, wipes and verifies the
entire 32-byte output object, and only then advances to the next output. It
returns the exact requested byte count and increments the process-service
delivery count once after the complete request succeeds. The ring-3 probe adds
one 256-byte request after the existing rejected, 16-byte, repeated, and
32-byte cases, requires an exact return of 256, and reads its first and last
qwords without constraining secret-derived values.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-USER-SMALL-REQUEST-001` and
`TOPAL-SYSTEMS-USER-GETRANDOM-SMALL-001` refine the approved systems profile;
they do not add or change a core-language element. Language, semantic,
provider-planning, artifact, and structural tests cover the exact eight-output
and 256-byte bounds, whole-range validation before extraction, eight native
calls, first-and-last-qword observation, per-output key replacement and
erasure, one delivery-count update, and unchanged terminal teardown.

The checked root contains 144 fatal edges, an exact 310-event semantic trace,
2,917 relocations, and 2,533 UART calls. The sealed provider plan contains 84
operations and retains the 160-byte entropy-storage layout. Artifact schema
`/99`, provider and provider-object schemas `/91`, artifact-root schema `/97`,
boot-adapter schema `/59`, and gate schema `/101` identify this contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. `TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK` is reachable only
after the complete 256-byte range has been accepted, eight outputs have been
copied and erased in sequence, the exact result has been returned, and both
ends of the result are readable by ring 3.

Two independent pinned-QEMU runs reached all 77 ordered observations. Hashes
for the recorded run are:

- linked kernel: `758faf8417a68b57b597fff9f13659838d18394ea9a2ca37c115be2c002e5de2`;
- boot image: `cf4d80059eed479b4671fbf695064506bbc57435bdaa96b05861a80e311bafa3`;
- boot resource: `394ebeb353d0b364aa3d846bfd1a06bb6a80b6020987db016cb1195516cd95f6`;
- generated image: `591777d6f6e8a266558618444001e772b56e45e1b696ce25c81844ea093ad7b4`;
- supplied image: `9983be791e5e2c84b4cde90c1b5149e66724d5ed59ee18dfc22bb59272bf526d`;
- artifact provenance: `9f567f94948049e6ff579a1d4a3677daed6f2885cba4414293a8761e163a689a`;
- boot provenance: `e601073a06ea8d88606f66529acefac8265e88ecf430196e7d157484114fe1cd`;
- serial output: `47b0d482681634466d54f5332e8cb315b5c1815c194f1926aa16098bfb75e22e`;
- evidence: `a722d7f6e7e8e364b2e4f0f26252aa8738e26e6606481cc62637a5f312c17ff5`.

## Risk and remaining boundary

Risk is high because one syscall now drives several secret-producing state
transitions across a recoverable user boundary. Review covered overflow-free
whole-range validation, bounded chunk arithmetic, generator/key replacement,
complete per-output wiping, failure ordering, exact return length, single-call
delivery accounting, structural inspection, and duplicate physical execution.

The qualified profile remains limited to one initial process, complete requests
of at most 256 bytes, and nonblocking ready-state execution. Partial completion,
blocking before CRNG readiness, signal interruption, larger requests,
`/dev/random`, `/dev/urandom`, vDSO `getrandom`, reseeding, per-CPU generators,
batching, fork/exec inheritance, and concurrent callers remain unavailable.
