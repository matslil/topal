# Step 4 initial x86-64 FS-base control

## Outcome

The initial x86-64 thread now implements bounded Linux-compatible
`arch_prctl(ARCH_SET_FS)` and `arch_prctl(ARCH_GET_FS)`. A nonzero base must
name an already-owned readable user mapping; zero disables it. Successful set
operations retain private thread state and install `IA32_FS_BASE`. Get
operations publish the exact retained value only to a complete writable
eight-byte user extent under fault recovery.

The constructed application selects its RW/NX data base, gets and compares it
through a stack word, clears FS to zero, and gets and compares zero before
terminal exit. Data-role teardown clears both private and physical FS state.
No core-language rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-X64-FS-BASE-001` requires live-context and executable-
return validation, zero or mapped canonical base selection, exact private and
physical installation, fault-contained `GET_FS`, Linux-compatible failure
results, and teardown clearing. Structural tests prove syscall and operation
selectors, all four application calls and result checks, private state
load/store, `WRMSR`, checked output, and both teardown clears.

Artifact schema `/59`, provider `/56`, provider object `/56`, artifact root
`/57`, boot adapter `/19`, and gate `/61` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive successful pinned `pc-q35-10.2`/`qemu64-v1` executions were
byte-identical and reached all 51 ordered observations. One preceding emulator
attempt exited with status zero before producing serial output; the gate
reported failure and wrote no evidence. The two subsequent complete runs
matched exactly. The evidence records:

- linked kernel: `d36da3ec9ad562a6ebefe1c07e27a315d565deba940036a0c926c3f3decce91f`;
- boot image: `d1e377119a78f40d639ca7341f4be05293fb9716fc547ab1304aad37dacf93b6`;
- boot resource: `cdf2dbc533e6fe7c1a6e2e0d897d77e04ba8ce7be852fb69b35f67950decb9a1`;
- generated default image: `ce7beaecff3fd03b7928bef1c5d15ff4e1ce81835fcacf01d8d736395d5976a8`;
- supplied image: `9883b5bafd64347d974abaa3193e869800910ccf783995fa094c0ac8ad0d3e5d`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `52215b7052d7d417605dccfdb893c37f8cc52bd62e4d5d3be9903cf8cc19b98c`;
- boot provenance: `69eac6201b2726461cd788ff1739853911c56838d13d7204865f31f124687333`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `9fba5e75abfeebf30ec3ee3c0cfc1f68c7a1e51483d224c8df4441c0dfaf371e`.

## Risk and review

Risk is high because this changes architecture-visible thread state across a
privilege boundary. Review covered syscall and operation numbers, bounded
canonical address policy, zero disablement, saved-register restoration,
`SWAPGS` independence, private-state identity, fault recovery, exact error
results, physical MSR installation and clearing, structural inspection, full
workspace validation, and repeated physical runs.

## Remaining boundary

This is one initial x86-64 thread. It has no GS controls, CPUID controls,
dynamic TLS allocation, multiple threads, cloning, scheduler restoration,
signals, ptrace, or general architecture-control service. Other architectures
retain their native thread-pointer mechanisms rather than this x86 protocol.
