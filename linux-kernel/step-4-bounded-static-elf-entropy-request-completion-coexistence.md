# Step 4 bounded static ELF entropy request-completion coexistence

## Outcome

The qualified x86-64 path now completes one real entropy request while the
complete static ELF process remains behind its opaque acquisition owner. It
revalidates the retained process, replays the exact transport and DMA setup,
submits one 32-byte writable descriptor, notifies queue zero, observes bounded
completion, validates used-ring flags, index, identifier, and length, then
returns control and payload to CPU ownership in that order.

The transaction does not capture or credit the returned bytes. It resets the
device and proves status, queue enable, and all three logical 64-bit queue
addresses are zero, overwrites and reads back every payload byte, clears and
reads back all source- and device-written ring fields, reverses bus mastering,
and restores configuration-gateway state. No entropy or resolved reference
escapes, the same affine owner returns, and
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_OK` precedes nested
rollback.

## Validation boundary

Rule
`TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-REQUEST-COMPLETION-COEXISTENCE-001`
adds no core-language element. Negative tests reject substituted owner, policy,
and marker; the exact parser also rejects substituted device, domain, queue, or
transport. Structural tests prove complete process validation, exact request
availability, preparation, publication, activation, notification, bounded
completion, used-ring validation, control-before-payload ownership return,
absence of entropy capture, complete transport rollback, payload and ring
erasure, and later-sequence preservation.

Artifact `/151`, provider and provider-object `/141`, root object `/140`, gate
`/153`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 388 operations; the provider plan contains 144; the
root contains 617 fatal edges, a 449-event semantic trace, 7,061 typed
relocations, 5,604 UART calls, 106 required text symbols, 48 PCI-configuration
reads, fifteen masked PCI-configuration updates, fourteen DMA-buffer creation
calls, six opaque DMA-reference bindings, 58 DMA integer stores, ten DMA
preparations, ten DMA completions, 42 DMA integer loads, twelve PCI-region
DMA-reference publications, 128 control references, seventeen payload
references, 131 ordinary PCI-region reads, two bounded eventual reads, 132
PCI-region writes, 175 opaque-storage derivations, and 147 observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 147 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `18ffab62b16f71c814378554f527f89b4dd28b84cc29b50283de281c7cdc21c4`;
- boot image: `c772183ea30638f8864efbb4cde23b54be43adcbee63b427d0996cbd3fbfcacf`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `c6bd44f992e8d448c9ec2ecf1bf758ba91d837203470d36c7f9e35303656bafc`;
- boot provenance: `63c608cd110d6b5ee4aa242b57dbc66999f8a2307cd572052016517fbfc5da8e`;
- serial: `24f10b0415305cbe28c4bdab433f69a405e23fac03300db7c67342182526373d`;
- evidence: `0ef1e2f6cc9c335cd0c0c5f62d9912264177d44dd2f5c71eeb9be3ae131b3a6a`.

## Risk and remaining boundary

Risk is high because the increment submits a real writable DMA request while
retaining an unrelated complete process. Review covered notification and
completion ordering, used-ring validation, ownership transitions, reset
semantics, complete transport readback, discarded-byte confinement, payload
and ring erasure, reference opacity, bus-master rollback, negative cases, and
duplicate complete physical execution.

This increment intentionally discards the device result and cannot replay it:
later correctness must not depend on those bytes. It does not capture, assess,
mix, or credit entropy for the static process, publish `AT_RANDOM`, initialize
TLS, construct a user context, enter ring 3, execute BusyBox, or add a syscall.
