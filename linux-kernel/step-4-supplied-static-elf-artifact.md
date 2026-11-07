# Step 4 supplied static ELF artifact

## Outcome

Qualified x86-64 publication now accepts one caller-owned static executable
with `--static-user-image PATH`. The compiler reads the complete input before
publication, derives the bounded ELF load and page plans, and additionally
requires the exact approved 2,024,544-byte, eleven-program-header, 503-page
shape. It then atomically publishes the exact bytes as `static-user.elf` and
binds that canonical name and SHA-256 digest in artifact provenance without
recording the host path.

The input is independent of `--initial-user-image`. Qualification supplies
both: the existing distinct 12 KiB image remains the sole `init` boot resource,
while the new static file remains an inactive artifact for the next copy
increment. No BusyBox binary is stored or redistributed by this repository.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ARTIFACT-001` adds no core-language element. The
validator checks the ELF identity, exact eleven raw program-header rows, entry,
four R/RX/R/RW load tuples, TLS, non-executable stack, GNU property and RELRO
facts, complete file extent, 503 adjacent page records, and the inherited W^X
page plan. Payload bytes outside checked metadata remain caller authority.

Unit tests accept an independently changed payload byte and reject file-size,
header-count, entry, load, TLS, and RELRO mutations. Artifact tests prove the
optional sixth file is exact, its provenance has exactly one canonical digest
entry, omission preserves the five-file output set, and repeated ordinary
publication remains deterministic. CLI qualification proves the two image
options coexist, the input path is not retained, and the boot packager ignores
the not-yet-runnable static artifact.

Artifact schema `/113` and gate schema `/115` identify this publication
contract. Provider `/104`, provider object `/104`, root object `/103`, boot
adapter `/68`, the 149 fatal edges, 332-event semantic trace, 3,360 typed
relocations, 2,966 UART calls, and 90 runtime observations remain unchanged.

## Physical evidence

The gate constructs a deterministic clean-room image with the exact approved
metadata and one independently chosen payload byte. It does not claim BusyBox
behavior. Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs published
the same 2,024,544 bytes and provenance digest, then reached all 90 existing
ordered observations with identical hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `f2eb541f12360b823ac89d4942ea3d6512a786116dc8f2088da5f5be5be048b8`;
- boot image: `fb1ce61426d5a8fc8decf7d5dd269a0e891408cabd226353d3d77f5d78a25899`;
- boot resource: `3353a052b6e9a89540dbad2d2f26494bfb1b8701b3ee967cd4367f16054af008`;
- artifact provenance: `061f79391cab6d1715257567c3d1f16a19b45a80b2378b85258911ed0169c951`;
- boot provenance: `dcae21e3c8c09019845c7ced65d0ec669008f10a5ff822df0b168f74ea162fb6`;
- serial output: `bde7294a4b0ba8dbc89ae3bcd350c8c5a279c951c4e6f4312e7c6d0391609bf6`;
- evidence: `34fb8c17067a459c6267094e082ff10654465ec49e4037e0280d715dae0aa468`.

## Risk and remaining boundary

Risk is medium: incorrect acceptance could bind an image whose later fixed
construction plan disagrees, while publication or provenance mistakes could
substitute unreviewed bytes. Review therefore covered raw and derived plan
equality, payload neutrality, prepublication failure, exact-byte output,
canonical-name digest binding, path exclusion, output closure, and duplicate
physical execution.

The static file is not yet a boot resource and its bytes are not copied into
the zeroed inactive construction. The next increment must transport the exact
digest-bound bytes to the kernel, then copy and verify the four approved file
intervals before any final permission, address-space attachment, entry, or
execution work.
