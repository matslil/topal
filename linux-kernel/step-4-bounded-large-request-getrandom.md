# Step 4 bounded large-request `getrandom`

## Outcome

The Topal kernel now accepts complete Linux x86-64 `getrandom` requests from
one through 512 bytes in the qualified signal-free process profile. The service
validates the complete writable range before changing generator state and then
partitions the request across at most sixteen 32-byte outputs.

Every fresh output replaces the persistent ChaCha20 key before publication.
The provider copies and verifies each bounded chunk, wipes and verifies the
complete output object, and advances only after that erasure succeeds. It
returns the exact request length and records one interface delivery for the
whole syscall. The ring-3 probe preserves the prior eight calls, adds one
512-byte request, and reads its first and last qwords without constraining
secret-derived values.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-USER-LARGE-REQUEST-001` and
`TOPAL-SYSTEMS-USER-GETRANDOM-LARGE-001` refine the approved systems profile;
they add no core-language element. Language, semantic, provider-planning,
artifact, and structural tests cover the exact sixteen-output and 512-byte
bounds, whole-range validation before extraction, nine native calls,
first-and-last-qword observation, per-output key replacement and erasure, one
delivery-count update, and unchanged terminal teardown.

The checked root contains 144 fatal edges, an exact 312-event semantic trace,
2,961 relocations, and 2,577 UART calls. The sealed provider plan contains 84
operations and retains the 160-byte entropy-storage layout. Artifact schema
`/100`, provider and provider-object schemas `/92`, artifact-root schema `/98`,
boot-adapter schema `/60`, and gate schema `/102` identify this contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. `TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK` is reachable only
after the complete 512-byte range has been accepted, sixteen outputs have been
copied and erased in sequence, the exact result has been returned, and both
ends of the result are readable by ring 3.

Two independent pinned-QEMU runs reached all 78 ordered observations. Hashes
for the recorded run are:

- linked kernel: `cae8c4eb53dc1e16bd0b1aa5bebc48bb95d3f7101928972ea9150b8753f7cd46`;
- boot image: `c0740454e696de92ef574a36604e33c2a108930dbe5b252567c22f5ef1fd0b3a`;
- boot resource: `62beb84b6c40b73f99661f8ae990450729b75217ad0734a12974d924aa79911a`;
- generated image: `42eec303740e9efb4d66472c243d92202a762f10381c06c844ce34444a7861c2`;
- supplied image: `0ab0a459cb3c3ac9a9b734bd7e505e3e7e89b324433c1f4af6e7d785a4003fd1`;
- artifact provenance: `a1146caf596ac557d2ef4287ec590cce40e92cf933034e52565cc6d69912eacf`;
- boot provenance: `5542aa324c56038353f754b62a67eb49f0c2fcc06ed6b80d49a2646a260f44b4`;
- serial output: `f79455b4ab5a595cd1b94cee0d95f730f635b77e34579d85a755d3eb7e9d48fb`;
- evidence: `81d2ccbf014ea0cb3d49fe4c4fb2faf7a609607572790e8b8ea74eea44969067`.

## Risk and remaining boundary

Risk is high because one syscall drives sixteen secret-producing state
transitions across a recoverable user boundary. Review covered overflow-free
whole-range validation, bounded chunk arithmetic, generator/key replacement,
complete per-output wiping, failure ordering, exact return length, single-call
delivery accounting, structural inspection, and duplicate physical execution.
The 512-byte bound was selected as the next physically qualified step above the
small-request threshold; a 4,096-byte exploratory bound exceeded the gate's
bounded execution budget and was not admitted.

The profile remains limited to one initial process, complete requests through
512 bytes, and nonblocking ready-state execution. Larger requests, partial
progress, blocking before CRNG readiness, signal interruption, `/dev/random`,
`/dev/urandom`, vDSO `getrandom`, reseeding, per-CPU generators, batching,
fork/exec inheritance, and concurrent callers remain unavailable.
