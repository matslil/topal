# Step 4 bounded initial `AT_PLATFORM` fact

## Outcome

The initial process now receives Linux `AT_PLATFORM` after `AT_CLKTCK` and
before `AT_NULL`. Its value points to a distinct in-block NUL-terminated
`x86_64` string. The fixed executable-name object remains separate, with exact
`argv[0]`/`AT_EXECFN` alias identity.

The bounded stack grows from 256 to 288 bytes and moves down by 32 bytes while
both strings retain explicit bounds. Constructed code validates the native tag,
exact platform pointer and bytes, moved terminator, executable name, and exact
block consumption.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-AUXV-PLATFORM-001` requires a complete distinct native
platform string, ordered termination, alias separation, and bounded
consumption. Structural tests inspect every initialized word and string, the
runtime `AT_ENTRY` patch, pointer comparison, and 288-byte adjustment.

Artifact schema `/72`, provider `/69`, provider object `/69`, artifact root
`/70`, boot adapter `/32`, and gate `/74` identify the changed contract. The
boot-resource schema remains `/1`. Focused suites, gate fixtures, formatting,
and the complete workspace suite pass.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. Hashes are:

- linked kernel: `c344a08df82619252c664bc985b6be4ca682ab39f446d927bcca5b9bf3215af5`;
- boot image: `f3fec8887e082c2e028aabcbe55d63430cc2bc2a0add61cb0b92963c3d332f07`;
- boot resource: `8115508c4eb289a67bb9d1661d8647c6afa7a786bc099d3fcbf2499da79a34a3`;
- generated image: `d918cd5817623f963f61f04cea932037d57fe28787c1294c03b8ceed9a22ffc2`;
- supplied image: `49623d754a3780427065adec7bccc56b61c51d27f2a2e17ca83c366f2eaa0221`;
- artifact provenance: `3fbe9f2012f0fbb41004800e3cd1a112724e171b7a7c7432fd82e06371956b12`;
- boot provenance: `e36f6be4f6f47a296e30f39c375b97845e228d49c0722cff98c4753c31002d41`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence: `ca75ff35ef4b6eefc4826cfa2a3f7742abd619ae54e7190c1bb6eb8562bc030f`.

## Risk and remaining boundary

Risk is medium because this adds a pointer-bearing entry and shifts the bounded
stack. Review covered pointer range, termination, distinct identity, alias
preservation, alignment, structural inspection, full regression, and physical
execution. Hardware-capability masks, personalities, compatibility ABIs,
`AT_RANDOM`, and a general auxiliary-vector builder remain absent.
