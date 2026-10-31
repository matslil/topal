# Step 4 runtime-derived data and BSS extent

## Outcome

Initialized-data construction, explicit BSS zeroing, and post-protection
verification now derive their bounds from the external ELF data program header
at runtime. The provider no longer assumes one initialized qword and one BSS
qword. The generated constructed entry also checks the complete validated BSS
extent before continuing.

Portable source observes only the ordered
`TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK` witness. ELF field offsets, byte
counts, loops, addresses, and instructions remain provider details. This is a
systems-feature refinement and changes no core-language rule.

## Model and x86-64 lowering

The checked model requires the runtime-data witness exactly once after the
runtime executable-extent witness and before the existing BSS, data, transfer,
syscall, retention, or teardown observations. Negative tests reject a missing
or renamed witness, while the ordered continuation rejects early, duplicate,
or late placement.

Provider revision `/47` excludes data `p_filesz` and `p_memsz` from the
immediate metadata oracle. It loads both from the external ELF and requires
`8 <= p_filesz <= p_memsz <= 4096`, with both extents qword-complete.
Population copies exactly `p_filesz` bytes from the immutable resource and
explicitly zeros exactly `p_memsz - p_filesz` bytes. After RO/NX publication,
verification compares the initialized extent to the same resource and checks
the complete memory-only tail for zero. Zero, truncated, inverted, misaligned,
over-page, mismatched, or nonzero-tail inputs fail closed.

Artifact and root revisions `/48` record 62 placements, a 195-transition
trace, and 1,714 typed relocations, including 1,528 UART writes. The initial
ELF reserves one complete file page for bounded data variation and is 8,192
bytes; the deterministic `newc` resource is 8,432 bytes. Boot resource schema
`/1` is unchanged, while the boot adapter advances to `/9` for the larger
resource envelope.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 47 ordered markers. Both copied the complete initialized data
extent, explicitly zeroed the complete BSS remainder, verified both after
write-authority revocation, completed the constructed-user BSS observation and
every prior process-entry and Linux syscall check, remained in the live fatal
halt, and left serial output quiescent. Gate schema `/50` records:

- linked kernel: `e354a00957904c2841b5dd7332637fb600ac07e2d392d7b0646fc95133df1318`;
- boot image: `2d59381bc9ebc51daaf9a3dd7806d978fe00a83e93fff6c9bc0e0027291fa636`;
- boot resource: `23f4afe061e2e0dcee19586b16c5643d07afea550fafd88e40c5adf57b40b96b`;
- initial user image: `6db776051032ae81441a7f9c959a9705dc8fa44cbc9b3a1c3dcdbe6432b85bb9`;
- artifact provenance: `ce81440b434eba138d622fee6247fca76a1f4021b430e6dc6ae6d1ab1b48dda6`;
- boot provenance: `4a7240f39315a237a97316f4b4a784bb76a873c119d380a0f2edaede150514e1`;
- serial: `f6b21833b6933373d742439a13fdf9981794efb3d00853fb6ef50f4c4a0124a0`;
- evidence record: `673b8b24df546fb5588380d3e4d2c1e16b1a18acf27696c4304c31e3c98f2384`.

## Risk and review

Risk is high because resource-controlled loop bounds govern data population,
zero initialization, verification, and user observation. Review covered field
oracle removal, relational and capacity checks, qword completeness, exact loop
termination, same-resource provenance, explicit zeroing, RO/NX publication,
fail-closed mismatch handling, preservation of provider state and backing
identity across runtime loops, negative model tests, structural instruction
tests, full workspace validation, and two independent physical runs.

## Remaining boundary

Placement, entry, resource size, immutability, one-page capacity, and the
generated qualification payload remain sealed. The profile does not yet admit
writable data, arbitrary segments, relocations, interpreters, dynamic linking,
TLS, PIE, ASLR, demand paging, path lookup, or a general process loader.
