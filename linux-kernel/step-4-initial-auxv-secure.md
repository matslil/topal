# Step 4 bounded initial `AT_SECURE` fact

## Outcome

The initial process now receives Linux `AT_SECURE` with value zero after its
program-header facts and before `AT_NULL`. The value records that the sealed
PID-one image did not cross a privilege-changing execution boundary and agrees
with its coherent real/effective root credentials.

The bounded stack block grows from 224 to 240 bytes and moves down by 16 bytes.
The executable-name string remains at its original address, preserving exact
`argv[0]`/`AT_EXECFN` alias identity. Constructed code validates the new pair,
the moved terminator, the string, and exact enlarged-block consumption before
continuing.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-AUXV-SECURE-001` requires exact value zero, placement
before the unique terminator, credential coherence, preserved aliasing, and
bounded consumption. Structural tests inspect the initialized words, runtime
`AT_ENTRY` patch displacement, constructed comparisons, and 240-byte stack
adjustment.

Artifact schema `/70`, provider `/67`, provider object `/67`, artifact root
`/68`, boot adapter `/30`, and gate `/72` identify the changed contract. The
boot-resource schema remains `/1`. Focused compiler, language, semantics, and
gate-fixture tests pass, as does the complete workspace suite.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. The evidence records:

- linked kernel: `91ec553bd6528df71d3b2694db1b976d75b3316b4b2074d9743a49db6dd5820f`;
- boot image: `0bc8e2ccfbb9affbb5a6ede0e8c61ab0f390f27173c624bdc04953ed589f6ed9`;
- boot resource: `69cb3a5d649734b9437b3946d04d876f6c101928efb25cca13923f52b1044397`;
- generated default image: `51725a40ffa26403e695cae702f43d7a878f70249966f742a5f54f1b26eab455`;
- supplied image: `34ab1a4638407884fb265dc04e2ede223827879c37511b475297c5e6197fa50f`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `55873745100841dc4b9705b13613eb6abd055a933e5f0f20416aeea6a47c7d2e`;
- boot provenance: `3c3ffbb662f7a953104255cf4d99f77635065161390872d9b1c39dee83012077`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `4c0c166df90fd1a947aee0219db68b582e899ed7a583308c62bcf08f1a59d4db`.

## Risk and review

Risk is medium because this changes every initial-stack displacement while
retaining the same mapped page and external aliases. Review covered native tag
and value, ordering, terminator uniqueness, runtime entry patching, preserved
string address, block alignment and consumption, structural inspection, full
workspace validation, and two physical runs.

## Remaining boundary

There is no privilege transition, environment filtering, capabilities,
security module, `AT_RANDOM`, platform/hardware-capability fact, dynamic
interpreter metadata, or general auxiliary-vector builder.
