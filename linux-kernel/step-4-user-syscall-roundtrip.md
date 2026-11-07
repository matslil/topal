# Step 4 initial user syscall-round-trip increment

## Outcome

The checked kernel root now prepares one affine PID-one user probe, transfers
from the restored kernel continuation into a user execution context, handles
Linux x86-64 `getpid` and `exit_group`, and consumes the terminal result after
the exact kernel continuation is restored. The success marker is emitted only
after the physical user-to-kernel-to-user-to-kernel sequence has completed.

The architecture-independent model owns process, probe, syscall-entry,
continuation, result, and termination identities. Source sees the semantic
syscall number and six arguments but no saved registers, selectors, page-table
entries, target return instruction, or stack pointer. Negative tests reject
wrong syscall order or values, altered identities, reuse, missing completion,
and entry/disposition mismatch.

## Initial x86-64 lowering

Provider revision `topal.provider.x86_64-qemu-pc-q35/16` adds three sealed
lowerings: probe preparation, user-context entry, and initial syscall
dispatch. Provider-object revision
`topal.provider-object.x86_64-qemu-pc-q35/16` emits their machine support
without a source assembly escape hatch.

The replacement translation reserves five contiguous page-table pages. Its
fifth page maps a fixed qualification-only user code page and NX user stack;
the frame-allocation floor advances past that page. Preparation copies the
generated probe while the code page is writable, removes write access,
invalidates the translation, enables `SYSCALL`, installs the sealed
`STAR`/`LSTAR`/`FMASK` and kernel-GS state, and then marks the context prepared.
The boot adapter adds user data and 64-bit code descriptors. Entry uses the
provider-owned privilege frame, and syscall entry switches to the retained
kernel stack before dispatch. `getpid` returns one through `SYSRETQ`;
`exit_group(0)` terminates the probe and restores the saved kernel continuation.

Root-object revision `topal.systems-root-object.x86_64/16` has 836 typed
relocations. Artifact revision
`topal.systems-artifact.x86_64-qemu-pc-q35/16` records 49 provider operations,
49 inspected placements, and a 128-transition semantic trace.

## Evidence

- semantic tests cover the complete affine user-execution protocol, exact
  syscall order and arguments, return value, terminal status, identities, and
  invalid transitions;
- source tests cover the typed user-syscall entry, exact handler disposition,
  preparation/entry/completion lifecycle, success marker, and negative forms;
- compiler tests cover provider planning, source-owned completion erasure,
  user page-table rights, user GDT descriptors, generated probe, syscall MSRs,
  GS/stack switching, `IRETQ`, `SYSRETQ`, typed relocations, semantic trace,
  and deterministic publication; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced byte-identical
  evidence: linked kernel
  `9c1e72fa5e27899a6bba610519c5da7beefae175b1eeff7578d83fb0e9a1e30b`,
  boot image
  `899efd47be595d90432d6802987ed6bb166ee705dcb5c42a75027e9d9f3a4f81`,
  artifact provenance
  `7e2f9cf5633e2ae5c2f4e7f893ff6e39355627eae8661533ad2e250f3eb3331a`,
  boot provenance
  `612bbb1711e23e79daaddd22318b1f91cdcc4d1ce13038f2091fced433234352`,
  and serial observation
  `98b50fbe8a1cae52e077436aff1c0b70132c0b0d642fb93b11c07e199693d05a`.

Gate schema `topal-kernel-toolchain-gate-qemu/18` records 22 ordered markers.
`TOPAL_KERNEL_USER_SYSCALL_OK` is reachable only after `getpid` returned PID
one, the probe issued `exit_group(0)`, and the provider restored the kernel
continuation. QEMU remains live and serial output is quiescent after the fatal
disposition.

## Validation and review

This is a high-risk privilege-boundary change. Review therefore covered the
approved architecture-independent contract, checked-source and reference-model
agreement, generated instruction and relocation structure, page rights and
reserved-frame separation, two independent physical QEMU executions, and the
complete repository test surface. Validation passed with:

```console
scripts/run_bounded.py -- cargo fmt --all -- --check
scripts/run_bounded.py -- cargo check --workspace --all-targets
scripts/run_bounded.py -- cargo test --workspace -q -j1
scripts/run_bounded.py -- python3 \
  linux-kernel/labs/qemu/x86_64/topal_gate.py --write-evidence
python3 -m unittest linux-kernel/labs/qemu/x86_64/test_topal_gate.py -v
git diff --check
```

## Remaining boundary

This is a qualification-only, single-process, single-thread, single-address-
space probe. It does not add ELF loading, arbitrary user memory, user-copy,
fault recovery, a general syscall table, file descriptors, signals, process
creation, scheduling policy, dynamic address spaces, SMP migration, or a
Linux userspace distribution. Those remain separate dependency-ordered
increments toward the complete pinned Linux interface.
