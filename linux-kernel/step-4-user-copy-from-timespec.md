# Step 4 fault-contained `copy_from_user` and `nanosleep` increment

## Outcome

The checked systems root now qualifies the input direction of Linux user
transfer. One affine scope tied to the exact process, address space, thread,
ABI, and syscall entry copies a native x86-64 `timespec` into private kernel
staging. A complete transfer exposes the validated zero-duration value; a
fault exposes only its eight-byte prefix count and no partial value.

The generated ring-3 probe invokes native Linux syscall 35 (`nanosleep`) twice
after the existing `getpid` and `uname` sequence. A complete zero-duration
request returns zero. A request beginning eight bytes before the unmapped
qualification guard page returns Linux `-EFAULT`. The probe then terminates
through `exit_group(0)` and restores its exact kernel continuation.

This is a systems-feature extension only. User candidates remain non-reference
machine words, destination staging is inaccessible to ordinary source, and no
core-language expression, reference, initialization, or exception rule
changed.

## Semantic and checked-source evidence

The sealed dispatcher owns the zero `tv_sec`, zero `tv_nsec`, and null
remainder expectations. It requires both `CopyInitialUnameToUser` and
`CopyInitialTimespecFromUser` effects. The reference protocol extends the
previous syscall stages with complete and faulting nanosleep entries and uses
the same monotonically refined transfer identity.

Completion requires all 16 bytes and a native layout satisfying nonnegative
seconds and nanoseconds below one billion; the qualification value is exactly
zero. The fault path requires candidate `0x601ff8`, prefix eight, and no decoded
value. Negative tests reject a non-null remainder candidate, missing complete
value, invalid signed value, partial-value exposure, wrong direction, extent,
identity, order, and result.

The modeled semantic trace now has 150 transitions. Its two input calls each
record syscall entry, transfer begin, copy complete or fault, transfer end, and
ABI return before terminal entry six.

## Initial x86-64 lowering

Provider and provider-object revision
`topal.provider.x86_64-qemu-pc-q35/18` add the generated supervisor-load loop
and private 16-byte kernel-stack staging. The complete path validates both
64-bit fields before publishing result zero. The fault path discards staging,
records only the copied prefix, and publishes `-EFAULT`.

The existing private vector-14 entry now selects recovery direction from
sealed per-context state. Output recovery still requires a non-present
supervisor write; input recovery requires a non-present supervisor read. Both
require the exact generated site and guard-page `CR2`, replace only the
registered private continuation, clear recovery authority, and return through
`IRETQ`. Every mismatch remains fatal.

The larger generated ring-3 probe also revealed that its publisher had used a
signed eight-bit displacement while accepting an unsigned byte offset. Probe
publication now uses checked 32-bit displacements and rejects a probe larger
than its sealed 4 KiB executable page. This preserves the existing abstraction
while allowing the new qualification sequence to be published correctly.

Root-object revision `topal.systems-root-object.x86_64/18` has 894 typed
relocations. Artifact revision
`topal.systems-artifact.x86_64-qemu-pc-q35/18` records 51 provider operations,
51 inspected placements, and the 150-transition semantic trace. The unchanged
boot-adapter revision remains
`topal.boot-adapter.linux-x86-protocol-2.15-q35/6` because vector 14 was already
installed and only its provider-private handler contract was refined.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU builds and executions
were byte-identical. Gate schema `topal-kernel-toolchain-gate-qemu/20` records
24 ordered markers, including `TOPAL_KERNEL_USER_COPY_FROM_OK`. The evidence
binds:

- linked kernel:
  `1a3d22cd00cfff4b7ae6722a85eb3d148c6a1348d8b2b0ac2b7a021b08df6e50`;
- boot image:
  `fb805b39d44c14788778e664e1ee789c4d27e756ad98de194d5404dc71fa87ee`;
- artifact provenance:
  `4f3537b00e1d1a61f536951e1ca46026e93f0a1ee17f514814fc366f7c1847e7`;
- boot provenance:
  `4a7008a30da671e1c912863d0d8dbf638d5be399f946137e1a9714d1329d1b50`;
- serial observation:
  `2b250964d15363a5e151aaca0d8e71e32ac452339c78309bd1316644cef3f853`;
  and
- complete evidence record:
  `8dc86e648ef9e4da71f8e8ef44ce8e5a02b813d07ed4c05dc8a7c44b9dedbf34`.

QEMU remained live in the final fatal disposition and its serial output stayed
quiescent after all markers.

## Validation and review

This is a high-risk privilege, initialization, and recoverable-fault boundary.
Review covered authority-order conformance, systems-only scope, affine
lifecycle negatives, absence of partial-value exposure, source/model
agreement, generated load and staging structure, direction-specific page-fault
classification, exact continuation validation, two physical executions, and
the complete repository test surface. Final validation uses:

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

The scope remains one qualification-only process, thread, address space,
layout, and zero-duration input. It does not add arbitrary byte input, dynamic
user mappings, concurrent mapping changes, nonzero sleep, remainder output,
signals, restart, file descriptors, ELF loading, process creation, SMP, or a
Linux userspace distribution. Those remain dependency-ordered increments
toward the complete pinned interface.
