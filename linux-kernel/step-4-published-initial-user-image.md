# Step 4 published initial user image

## Outcome

The initial user ELF is now a canonical compiler output. The systems artifact
publisher emits `initial-user.elf` atomically with the linked kernel and binds
its filename and digest in artifact provenance. The boot packager consumes
those exact bytes, verifies their digest before publication, and binds the same
image in boot provenance.

This closes the former duplicate-generation path without changing the initial
process ABI or any core-language rule. It also creates a reviewable file and
provenance boundary for later replacement by a separately supplied compatible
application image.

## Publication and packaging checks

Artifact schema `/50` contains five canonical files: linked kernel, debug
companion, link map, `initial-user.elf`, and provenance. Publication uses one
temporary directory and fails before rename if any output cannot be generated,
hashed, or encoded. Its provenance has four bound output digests, including the
initial image.

Boot adapter revision `/11` no longer regenerates an image. It receives the
published file, verifies its digest against artifact provenance, and only then
constructs the deterministic `init` newc member. A missing, unbound, or changed
image is rejected before the boot destination is created. The builder's
negative test changes one published image byte and proves rejection leaves no
destination behind.

Provider `/48`, provider object `/48`, artifact root `/49`, and boot-resource
schema `/1` retain their prior executable behavior. Gate schema `/52` records
the new artifact and boot-adapter revisions while preserving the 48-marker
runtime observation.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 48 ordered markers. In both runs the compiler-published initial
image crossed the checked packaging boundary unchanged, the provider validated
and loaded it from the resulting boot resource, every process-entry and Linux
syscall check completed, and the kernel remained in the live quiescent fatal
halt. The evidence records:

- linked kernel: `e69faabd8288482d2911d090efb5f11e0e56211232b822978c8e016db5f52f85`;
- boot image: `15ea7c7a9b75657ecd0decceb2e5c078fa4e92b02b93f970ffb50c58b30f9fc6`;
- boot resource: `2b8238ed1178facff10cd6ddc5af26e0d852682a3a3de32382cab82a587eb5b5`;
- initial user image: `b230a105144d38f41a71cb7e15d47535561067ac96b3a66bb44f4ef6000b4683`;
- artifact provenance: `5edbc0c543c53b7b414faaa045b0f3d5425ad83568d5e6b80b936a07c3a3dda5`;
- boot provenance: `9e9d1f650e0b6bf23eac0be9b5c38b46e472e302cba15c17aeb57fbb77005874`;
- serial: `766fbc7afcb2f58f2a6d84e1e1cd4dd0d048ecd288c11869e70a3753ddf12317`;
- evidence record: `bf1dd4fdaa390bb685fdb022691a0df4df52144a89c72040cad2b74b5f59e147`.

## Risk and review

Risk is medium-high because this changes the authority boundary for executable
bytes without changing their qualified contents. Review covered atomic output
sets, filename and digest binding, deterministic publication, exact-byte
packaging, tamper rejection, destination cleanup, schema propagation, targeted
compiler and builder tests, the full workspace suite, and two independent
physical runs.

## Remaining boundary

The compiler still generates the fixed qualification image. This increment
does not yet accept an independently supplied application, admit writable or
arbitrary ELF segments, add path lookup or a filesystem loader, or provide a
general process lifecycle. Those remain subsequent userspace-gate work.
