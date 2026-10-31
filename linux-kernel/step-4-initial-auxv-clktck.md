# Step 4 bounded initial `AT_CLKTCK` fact

## Outcome

The initial process now receives Linux `AT_CLKTCK` with `USER_HZ` value 100
after `AT_SECURE` and before `AT_NULL`. The fact is explicitly distinct from
the Q35 HPET period and from any scheduler timer frequency.

The bounded stack block grows from 240 to 256 bytes and moves down by 16 bytes.
The executable-name string remains at its original address, preserving exact
`argv[0]`/`AT_EXECFN` alias identity. Constructed code validates the new pair,
the moved terminator, and exact enlarged-block consumption before continuing.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-AUXV-CLKTCK-001` requires exact value 100, placement
after `AT_SECURE` and before the terminator, fixed aliasing, and bounded
consumption. Structural tests inspect every initialized word, the runtime
`AT_ENTRY` patch displacement, constructed comparisons, and 256-byte stack
adjustment.

Artifact schema `/71`, provider `/68`, provider object `/68`, artifact root
`/69`, boot adapter `/31`, and gate `/73` identify the changed contract. The
boot-resource schema remains `/1`. Focused compiler, language, semantics, and
gate-fixture tests pass, as does the complete workspace suite.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. The evidence records:

- linked kernel: `4151f4bb687b3a5c945663fc1557848d9d1d017890274987896c0b559e50184f`;
- boot image: `cd2578ddbe5f487c294ee6e8cb380f0ca10b02659bee24df24c874b8e8871961`;
- boot resource: `9c0a4c01155e7d047c0ad74b29e702242e4a9764134ad4181343cd458fce801f`;
- generated default image: `8341674a4190bdf939b35fee7edf9d3a7b8f7f95c2325fc28641616a9b397369`;
- supplied image: `8c9f5bd10ce5704bb1008d82d2235c7bcfdfec9a3c672678409f0397762edd73`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `59756eda2c4700c530d831bdeef619bae2369b9b9cf5f7f9b62fc9ac5ee1a047`;
- boot provenance: `819d92993feb720b205fa5fa11ae357c265eb975cb0151dbec2ec67753659534`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `e3a3354a1e6e67688fbe5d3e066231601a0363f35dc4ef9982f08d0cc0a88975`.

## Risk and review

Risk is medium because this shifts every initial-stack word while preserving
the mapped page and external aliases. Review covered tag/value semantics,
ordering, unique termination, runtime entry patching, fixed string identity,
alignment and consumption, structural inspection, full workspace validation,
and two physical runs.

## Remaining boundary

There is no process accounting, CPU-time clock, interval timer, scheduler tick,
`times`, `AT_RANDOM`, platform/hardware-capability fact, or general auxiliary-
vector builder.
