# Step 4 initial stack resource-limit queries

## Outcome

The initial process now implements bounded Linux-compatible `getrlimit` and
query-only `prlimit64` for `RLIMIT_STACK`. Both interfaces publish one sealed
native pair: an 8 MiB soft limit and `RLIM_INFINITY` hard limit. The
constructed application checks the complete pair after both calls.

The provider accepts `prlimit64` only for PID zero or the sole process PID one,
requires a null new-limit pointer, and admits no other resource selector. Both
interfaces validate a complete writable 16-byte destination and copy under
fault recovery. No core-language rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-RLIMIT-STACK-001` requires live-context and
executable-return validation, native Linux x86-64 call shapes, exact selector
and process identity, immutable private values, complete destination checking,
fault-contained publication, and deterministic rejection. Structural tests
prove both application calls and full-pair checks, exact dispatch and argument
validation, the shared 16-byte copy loop, both constants, and teardown
rejection.

Artifact schema `/62`, provider `/59`, provider object `/59`, artifact root
`/60`, boot adapter `/22`, and gate `/64` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. Both stack-limit queries complete
before terminal processing. The evidence records:

- linked kernel: `26fda120191b43484954b6b28221c55ce9cd9c3e4f013a9ee30d3ddb08faa458`;
- boot image: `435482f9fb6042631c98f8b29aa9ca5064a3d0b669c8873f28de8a226f7f5037`;
- boot resource: `2899a96863e0e78708268d756f21a3820d5cf6569134a1d996d52114023b1680`;
- generated default image: `e43c080c5872212b6edf489c75a1a3ff0b91fdbcc4fc1469901045ad70e6435c`;
- supplied image: `7ceb56b7cc22ce9e2033b4e4f4bd131e14b793bd794b887f8727f818ef670658`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `3669148fbaa0ed89d050dbe7790b87bc68d87356e7ed9c8f858ed97b3e553860`;
- boot provenance: `72a4a2a7fb05bf4145b90284bc2befeac134c142fb01ca02246de7563efe5622`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `c816fe73d55b53746c4365886153f17e319f88222af6271e6f308fb77ce7a9dd`.

## Risk and review

Risk is medium because the change adds two native ABI paths that publish a
multiword result through a user pointer. Review covered exact call shapes,
process and resource selection, immutable value identity, complete range
validation, shared fault containment, rejection ordering, structural
inspection, full workspace validation, and two independent physical runs.

## Remaining boundary

No other resource is admitted. There is no limit mutation or enforcement,
inheritance, accounting, multiple processes, cgroups, capabilities, or a
general resource-limit service.
