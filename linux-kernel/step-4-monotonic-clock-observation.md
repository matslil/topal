# Step 4 monotonic-clock observation increment

## Outcome

The checked kernel root now borrows one provider-created monotonic clock twice
after completing its local-notification wait. Each `now` produces an immutable
`Instant InitialMonotonicClock`; the architecture-independent model records
ordered observation identities 1 and 2 and rejects a decreasing accepted
value. `TOPAL_KERNEL_TIME_OK` is unreachable until both observations succeed.

The source checker requires the exact clock classifier, context, placement,
two-read count, and success marker. It rejects an unknown or mismatched clock,
wrong context or result classifier, reordered or missing observations, an
extra observation, and ordinary completion before the closed observation
sequence.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/10` adds the closed
`topal.provider.x86_64.time.hpet-monotonic-now/1` lowering. It selects the
pinned Q35 HPET privately, requires its 64-bit main-counter capability and
10 ns QEMU period, enables only the main counter, and reads it without
programming a timer or enabling timer interrupts.

Provider-private bootstrap storage retains initialization, the preceding raw
counter, and a wrap epoch. A raw decrease is accepted only across a bounded
64-bit wrap boundary; any other regression and epoch overflow fail closed.
The returned instant is a 128-bit epoch/counter pair, while source observes
only its clock-typed identity. The initial and replacement translations each
contain a cache-disabled 2 MiB leaf covering the HPET MMIO window. The gate
pins `hpet=on` explicitly on the versioned QEMU machine.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/10` records 36
provider-plan identities, 34 linked semantic placements, and a 58-transition
trace. Root object revision `topal.systems-root-object.x86_64/10` has 366 typed
relocations, including exactly two clock-provider calls, ten opaque storage
references, and fail-to-fatal edges after both calls. Boot adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/4` installs the HPET leaf in
the initial translation; the provider retains the corresponding leaf in the
replacement hierarchy.

## Evidence

- architecture-neutral unit tests accept equal and increasing observations
  and reject regression;
- source tests cover the exact accepted sequence and the clock, classifier,
  context, count, ordering, and marker failures;
- compiler tests cover all 36 provider mappings, the HPET capability and
  period checks, main-counter enable/read path, private monotonic state,
  wrap-versus-regression boundary, both MMIO leaves, two typed root calls,
  linked placement, and the ordered semantic trace; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs with `hpet=on` produced
  identical linked kernel
  `0e5dfa536ecf3764ca0689f1e2191cf2618fc306093432b9af5a444d981412a0`,
  boot image
  `a16e560d92352e2d0bb9e3ab1548b8002e664398ff0f453676aea23100e9b9c3`,
  artifact provenance
  `90033a7ddb1735d918a93ca4569f0cd342342bb213a9c5cb5dae8b979c2ee5b7`,
  boot provenance
  `96058f386337ab0364abfca288d4c5426164afb5bcd6bc1e328bd2f860bd7c63`,
  and serial observation
  `bde753d5a7bf953914c334e0df1f82039d0fac5b527e26f058c6a2c1e9a1beaf`.

Gate schema `topal-kernel-toolchain-gate-qemu/11` records the exact twelve
markers, generated artifact identities and digests, post-marker QEMU liveness,
and serial quiescence after fatal disposition. This is high-risk privileged
MMIO and translation work; review therefore combines authority-ordered design
checks, architecture-independent negative tests, exact machine-code and ELF
inspection, initial/replacement page-table inspection, two independent QEMU
runs, and full workspace validation.

## Remaining boundary

This increment does not add timer delivery, deadline construction, periodic
ticks, sleeps, timeout races, wall-clock or UTC synchronization, suspend or
migration continuity, SMP-safe observation, userspace clock ABIs or vDSO
publication, or executable AArch64/RISC-V providers. The HPET address,
registers, width, period, scale, enablement, and wrap state remain provider
details rather than portable Topal source elements.
