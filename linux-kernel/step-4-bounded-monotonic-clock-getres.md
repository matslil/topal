# Step 4 bounded monotonic `clock_getres`

## Outcome

The initial process now implements Linux-compatible
`clock_getres(CLOCK_MONOTONIC)` for a null result pointer or a complete writable
native timespec destination. The x86-64 provider reports the qualified Q35
HPET's exact 10 ns resolution as `{ tv_sec: 0, tv_nsec: 10 }`, stages the pair
privately, and publishes all 16 bytes under the existing page-fault recovery
protocol.

The constructed application verifies both supported forms: it checks the exact
timespec written to user memory and checks that the null-pointer query succeeds.
No HPET register, MMIO address, period encoding, or core-language mechanism is
exposed to source.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-CLOCK-RESOLUTION-001` requires the qualified monotonic
clock, exact nonnegative resolution, null-pointer success, complete
fault-contained publication, and no retained candidate. Structural tests prove
syscall 229 dispatch, the staged 10 ns value, two constructed calls, and balanced
user storage.

Artifact schema `/68`, provider `/65`, provider object `/65`, artifact root
`/66`, boot adapter `/28`, and gate `/70` identify the changed contract. The
boot-resource schema remains `/1`. Focused language, semantics, compiler, and
gate-fixture tests pass, as does the complete workspace suite.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. The evidence records:

- linked kernel: `891861185b12ae6326886ea692eaa34d0216df31c0df76b2b7e2eb5038e5cc98`;
- boot image: `b691942b074c1ddc5508af2bdea9546e7491aa7898775abca68d8d8128909d09`;
- boot resource: `826a4f9d7eb60c483b052f70c9c023ca60ffb367589fd21683323a550c2f6dc0`;
- generated default image: `6acfd029f99971653d473eddd04c2207e36bbb64a50c6b269adcdb93343c6b64`;
- supplied image: `0fee991e6bab1d27c42be08065a6ecff9a0dff4950eed8cec464a813757c45ef`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `7e2b24e364a189bbf445056f64f4dc661e1359a1315ba78981f3838b42620baa`;
- boot provenance: `3ef50e7ba6044ec89f4c062756696307502fb5c42c628371ad4cce6993c4c431`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `3d9e67958a3ac0cfe4d4d5e3649dc00d6cc814dc0aec9620d4d08f239f367946`.

## Risk and review

Risk is high because this publishes a native user structure and completes a
hardware-derived clock interface. Review covered the pinned HPET period, exact
unit conversion, null-pointer behavior, complete-range validation,
fault-contained copy, stack balance, structural inspection, full workspace
validation, and two physical runs.

## Remaining boundary

There is no realtime, coarse or raw clock, vDSO, timer creation, timerfd, sleep
integration, epoch/update policy, adjustment, clock namespace, SMP migration,
or general userspace time service.
