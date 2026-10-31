# Step 4 bounded monotonic `clock_gettime`

## Outcome

The initial process now implements Linux-compatible
`clock_gettime(CLOCK_MONOTONIC)` for complete writable native timespec
destinations. The x86-64 provider validates and samples the qualified 64-bit
Q35 HPET with its 10 ns period, normalizes the observation to seconds and
nanoseconds, stages the pair privately, and publishes all 16 bytes under the
existing page-fault recovery protocol.

The constructed application performs two calls, validates both nanosecond
fields are below one billion, and proves the second pair is lexicographically
nondecreasing. No counter, MMIO address, frequency, register, or core-language
mechanism is exposed to source.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-CLOCK-MONOTONIC-001` requires one qualified sample per
call, normalized nonnegative output, complete fault-contained publication, no
retained candidate, and nondecreasing successful observations. Structural
tests prove syscall 228 dispatch, HPET capability and period validation,
integer normalization, a private 16-byte staging copy, two constructed calls,
and balanced user storage.

Artifact schema `/67`, provider `/64`, provider object `/64`, artifact root
`/65`, boot adapter `/27`, and gate `/69` identify the changed contract. The
boot-resource schema remains `/1`. Focused language, semantics, compiler, and
gate-fixture tests pass, as does the complete workspace suite.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. The evidence records:

- linked kernel: `21732e5cfd57afabc8e7c1e122a23825632c5392c9bf2e32d3b9d40c1edf95b1`;
- boot image: `6afef1e2ff8f10cf0341c4024a63e34c396377ddf7531d084087820465e91076`;
- boot resource: `f75404c64285d0d994d7775104eddb59e4155e6c65039fbb7b0a3adf64e77202`;
- generated default image: `9b856a6fd108633c6a3127a7d2066377d70219665c2a891671042f2ff96d99f3`;
- supplied image: `cee3a907dd43c4bdfe3f5bac2dba0df1318a1fb1ab43838779deaa47fc6bf9d2`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `8fa0e3a8789eaec309290817bcbeb0f8df422d9cce5591a753dff473ac4f47fa`;
- boot provenance: `c0f3c077d1b1f1e7c5a5a82dbc17c72b84c35ab6c53b1db6ca8a141af4d5ccd8`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `f41e921081efb714b6e04be77844459cd0115f207b0afbd57bb7c1315b576f1a`.

## Risk and review

Risk is high because this exposes hardware-derived time and writes a native
user structure. Review covered source qualification, capability and period
validation, single sampling, exact integer conversion, normalization,
fault-contained complete copy, stack balance, repeated monotonicity,
structural inspection, full workspace validation, and two physical runs.

## Remaining boundary

There is no realtime, coarse or raw clock, vDSO, clock resolution query, timer
creation, timerfd, sleep integration, epoch/update policy, adjustment, clock
namespace, SMP migration, or general userspace time service.
