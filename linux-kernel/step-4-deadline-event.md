# Step 4 one-shot deadline-event increment

## Outcome

The checked kernel root now constructs one `Deadline InitialMonotonicClock`
exactly `1[ms]` after its second accepted clock observation, arms it, and
consumes one `ArmedDeadline InitialMonotonicClock` in a completion-aware wait.
The generated typed external entry records delivery from that same clock,
rejects an observation earlier than the scheduled instant, completes the event,
and resumes the interrupted continuation. `TOPAL_KERNEL_DEADLINE_OK` is
unreachable until that lifecycle succeeds.

The architecture-independent model distinguishes construction, arming,
delivery, completion, and resumed observation. It admits late delivery without
rewriting the scheduled instant, makes an already-expired deadline immediately
deliverable, and rejects clock-identity mismatch, early delivery, duplicate or
missing completion, and affine-token reuse. The source checker admits only the
approved single-event sequence and typed handler.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/11` adds four generated
operations and one external entry for the closed Q35 implementation. Deadline
construction adds exactly 100,000 ticks to the retained HPET observation after
the provider has validated the machine's 10 ns period. Arming requires the
64-bit main counter and timer-0 route-2 capability, masks both legacy PICs,
programs timer 0 as a one-shot comparator, routes GSI 2 through the I/O APIC at
private vector `0xf2`, and enables its interrupt. An already-expired deadline
uses a local-APIC self-notification through the same typed entry instead of
starting a new interval.

The completion operation reads the same HPET counter, rejects delivery before
the scheduled value, retains the observed value, disables timer-0 delivery,
clears HPET status, acknowledges the local APIC, and publishes completion to
the waiting continuation. The boot adapter installs the private vector gate;
the source cannot name the comparator, route, vector, registers, frame, wait,
acknowledgement, or interrupt-return instruction.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/11` records 41
provider-plan identities, 39 linked semantic placements, and a 67-transition
trace. Root object revision `topal.systems-root-object.x86_64/11` has 403 typed
relocations: 330 console writes, 14 opaque storage references, 29 fail-to-fatal
edges, and three interrupt returns. Provider-object revision
`topal.provider-object.x86_64-qemu-pc-q35/11` contains the HPET, I/O APIC,
local-APIC, and legacy-PIC machine sequences. Boot adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/5` installs the generated
deadline entry at vector `0xf2`.

## Evidence

- architecture-neutral unit tests cover future, late, and already-expired
  delivery plus clock mismatch, early delivery, overflow, and lifecycle misuse;
- source tests cover the exact accepted handler/root sequence and reject wrong
  clocks, classifiers, contexts, duration, order, count, completion, and marker;
- compiler tests cover all 41 provider mappings, exact 1 ms tick construction,
  HPET capability and period checks, route capability, comparator programming,
  legacy-PIC masking, I/O APIC routing, immediate delivery, typed entry,
  completion acknowledgement, linked placement, and semantic trace; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs with `hpet=on` produced
  identical linked kernel
  `f8c41d9d7e36a3c8da5de9a8332cef904eefe1e8453806fd8f593146201973e5`,
  boot image
  `68f7145ad0bae08c66441c6269ace24f2db49d3079f6251f878427146d1b13fd`,
  artifact provenance
  `337a026cbc4b497cc5ccc8f1112c3f49ca2c76f839ba7690c0e0ae733e368208`,
  boot provenance
  `f6b808c3960008d810fd49a8975b17c08d179869e96d2c3ef71237dbef3d1d0e`,
  and serial observation
  `b10bcbce012b06cb70bb3f940392644c424e7d6c8d5905d5987ee0b0b0e4a9fc`.

Gate schema `topal-kernel-toolchain-gate-qemu/12` records the exact thirteen
markers, generated artifact identities and digests, post-marker QEMU liveness,
and serial quiescence after fatal disposition. This is high-risk privileged
timer and interrupt-controller work; review therefore combines
authority-ordered design checks, architecture-independent negative tests,
exact machine-code and ELF inspection, generated IDT inspection, two
independent QEMU runs, and full workspace validation.

## Remaining boundary

This increment does not add cancellation, rearming, periodic release,
scheduler integration, sleep or timeout composition, multiple outstanding
events, bounded latency or rate accuracy, SMP delivery, suspend or migration
continuity, userspace timer ABIs, or executable AArch64/RISC-V providers. The
HPET, PIC, I/O APIC, local APIC, comparator, route, vector, and acknowledgement
remain provider details rather than portable Topal source elements.
