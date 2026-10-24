# Step 4 bounded static ELF entropy DMA-preparation coexistence

## Outcome

The qualified x86-64 path now performs one reversible DMA-preparation
transaction while the complete static ELF process remains behind its opaque
acquisition owner. It revalidates the retained process; replays negotiation,
queue discovery, bus mastering, DMA-buffer qualification, and split-ring
construction; then prepares the payload before the control buffer.

Because queue addresses are not published and the device cannot access either
buffer, the transaction cancels the unexposed control preparation before the
payload preparation. It then clears and reads back the ring, reverses bus
mastering, and restores device and configuration-gateway state. No DMA
identity escapes, the same affine owner returns, and
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_OK` precedes nested rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-DMA-PREPARATION-COEXISTENCE-001` adds no
core-language element. Negative tests reject substituted owner, device, domain,
queue, policy, and marker. Structural tests prove complete process validation,
exact prerequisite replay, payload-before-control preparation,
control-before-payload cancellation, complete ring erasure and readback,
device rollback, and preservation of the later ordinary sequence.

Artifact `/148`, provider and provider-object `/138`, root object `/137`, gate
`/150`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 382 operations; the provider plan contains 141; the
root contains 409 fatal edges, a 443-event semantic trace, 6,400 typed
relocations, 5,450 UART calls, 103 required text symbols, 48 PCI-configuration
reads, nine masked PCI-configuration updates, eight DMA-buffer creation calls,
three opaque DMA-reference bindings, 22 DMA integer stores, four DMA
preparations, four DMA completions, fourteen DMA integer loads, 46 control
references, eight payload references, 74 ordinary PCI-region reads, one
bounded eventual read, 85 PCI-region writes, 96 opaque-storage derivations,
and 144 observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 144 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `0cc4ca553e376b29dacc91e745b0c6bfb39cee9b2556f0f25eab12bd73e80117`;
- boot image: `9911397fe530ddfee66e4d85ce351218a63736324af3056db8f5bf0c43bdfe07`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `976480053b24e99e6ea2aea41ebb29d130cd028952c70307bd141923014c954a`;
- boot provenance: `aab23f7da67efc8590162c9f1db7a66b760364277ff46eee50608d5c4fcade07`;
- serial: `4196c557ce8471fd09c497a760e105a7471f4657e2161f6f1c3ddc8618af9c9c`;
- evidence: `16bcce09a391f4d507c29242ecd87280c6b4ebd42d72513cd849b548bed093a1`.

## Risk and remaining boundary

Risk is high because the increment temporarily changes two buffers from CPU
ownership to DMA-device preparation while retaining an unrelated complete
process. Review covered owner return, retained-state validation, preparation
and cancellation ordering, identity opacity, complete byte rollback with
physical readback, bus-master and device rollback, later-sequence accounting,
negative cases, and duplicate complete physical execution.

This increment returns no DMA owner and publishes no queue address. It does
not enable or notify the queue, grant device ownership, consume entropy for the
static process, publish `AT_RANDOM`, initialize TLS, construct a user context,
enter ring 3, execute BusyBox, or add a syscall.
