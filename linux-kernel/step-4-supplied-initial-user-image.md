# Step 4 supplied initial user image

## Outcome

The qualified systems compiler now accepts one caller-selected application ELF
with `--initial-user-image PATH`. It validates the complete bounded x86-64
envelope before publication, emits the supplied bytes as canonical
`initial-user.elf`, and preserves the artifact-to-boot provenance chain. Native
application compilation rejects the systems-only option.

The physical gate constructs a supplied fixture by changing an initialized-data
byte that the generated default leaves zero. Its distinct digest survives
compiler publication, boot packaging, resource validation, segment copying,
post-protection comparison, and completed user execution. No core-language
rule changes.

## Validation boundary

Artifact schema `/51` requires an exact 8,192-byte ELF64 envelope with the
qualified x86-64 executable identity, three-entry program-header layout, fixed
segment placements, and immutable metadata. It relationally validates:

- an equal executable file/memory extent of 264 through 4,096 bytes, aligned to
  eight bytes;
- an entry at or above the payload floor and strictly below the executable
  file-extent end; and
- an initialized-data extent of 8 through 4,096 aligned bytes plus an aligned
  memory extent no smaller than the file extent and no larger than one page.

Executable payload and initialized-data contents remain caller authority.
Wrong file size, immutable metadata, executable extent, entry, or data/BSS
relation is rejected before a publication directory exists. The host input
path is excluded from provenance, so identical bytes remain reproducible
regardless of their origin.

Provider `/48`, provider object `/48`, artifact root `/49`, boot adapter `/11`,
and boot-resource schema `/1` are unchanged. Gate schema `/53` records both the
generated-default and supplied-image digests and identifies the selected mode.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 48 ordered markers. The supplied image differs from the default,
was published and packaged exactly, passed independent runtime ELF validation,
completed every process-entry and Linux syscall check, and left the kernel in
the live quiescent fatal halt. The evidence records:

- linked kernel: `e69faabd8288482d2911d090efb5f11e0e56211232b822978c8e016db5f52f85`;
- boot image: `15ea7c7a9b75657ecd0decceb2e5c078fa4e92b02b93f970ffb50c58b30f9fc6`;
- boot resource: `4465aaf06e3f0460d8886e8485e5897004789c95f83fa7a16cdfe40e0f4abdb2`;
- generated default image: `b230a105144d38f41a71cb7e15d47535561067ac96b3a66bb44f4ef6000b4683`;
- supplied image: `2fb0f879640d27f273b76d2f48f7df7200921a99b0d21dc53b1386ea4e64804f`;
- artifact provenance: `ed7da917824f17fbf6c0bdbc85621f8903e9c5976b9bd046f68708b6ab586a2e`;
- boot provenance: `5606dba1a7779b57d6da76c2c318e371bdee4adc7a7fe472b59f7d81e3da2374`;
- serial: `766fbc7afcb2f58f2a6d84e1e1cd4dd0d048ecd288c11869e70a3753ddf12317`;
- evidence record: `f2c64090f5068c0640ed37ac922aed004b7f8b81cd752c83bb42fb7fe8518904`.

## Risk and review

Risk is high because caller bytes now become executable authority. Review
covered systems-only CLI scoping, prepublication failure, every variable and
immutable ELF field class, content/path provenance separation, exact-byte
publication and packaging, a distinct successful payload, malformed-input
tests, full workspace validation, and two independent physical runs.

## Remaining boundary

This is a replaceable bounded application, not a general Linux process loader.
The layout, addresses, segment count, permissions, initial stack, and syscall
surface remain sealed. Dynamic linking, writable data, TLS, PIE, relocations,
path lookup, filesystems, process creation, and ordinary Linux applications
remain subsequent userspace-gate work.
