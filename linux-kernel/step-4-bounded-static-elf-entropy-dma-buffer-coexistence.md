# Step 4 bounded static ELF entropy DMA-buffer coexistence

## Outcome

The qualified x86-64 path now performs one reversible two-buffer DMA
qualification while the complete static ELF process remains behind its opaque
acquisition owner. It revalidates the retained process, replays negotiation and
queue discovery, enables bus mastering, qualifies the exact sealed control and
payload requests, then reverses bus mastering and restores device and gateway
state. No DMA identity escapes, the same affine owner returns, and
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_OK` precedes nested rollback.

The two provider-backed candidates are distinct 4,096-byte-aligned BSS objects
with exact control/payload roles. Checked source names only the opaque process,
device, translation domain, and policy; it cannot observe any physical,
virtual, bus, or I/O-virtual address.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-DMA-BUFFER-COEXISTENCE-001` adds no
core-language element. Negative tests reject substituted owner, device, domain,
policy, and marker. Structural tests prove complete process validation, exact
prerequisite replay, two exact DMA creation requests, distinct aligned backing,
complete rollback, and preservation of the later ordinary sequence.

Artifact `/146`, provider and provider-object `/136`, root object `/135`, gate
`/148`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 378 operations; the provider plan contains 139; the
root contains 319 fatal edges, a 439-event semantic trace, 6,089 typed
relocations, 5,355 UART calls, 101 required text symbols, 48 PCI-configuration
reads, five masked PCI-configuration updates, four DMA-buffer creation calls,
52 ordinary PCI-region reads, one bounded eventual read, 61 PCI-region writes,
and 142 observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 142 observations in order. One intervening attempt
reached the new marker and the existing scheduler terminal-retirement marker
but hit the gate timeout before the scheduler resumed; it was not counted as
duplicate evidence.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `d8f9fbb5f9d291b8473f4008b19b3b41808de00e42f9f4136c58547e211d9095`;
- boot image: `944dfd98dfb27d96914fc40a6c9f5f5740e559241220d130475848d4bbe8d8dd`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `0d4ebfcd7249f4898baeabd861911075b9bc124f0ed92fd12f63c13af2f06e05`;
- boot provenance: `be5150b9d15357fe25982525c30d5512d3fc676c7369ae5d78bd72daf814ce26`;
- serial: `7fcf5335bb506c023347117268586a7c472645d6b92430c9a10cd4a1979e9f9c`;
- evidence: `c0dd6338f0fe33ea0f47bf26f367feab9ffa24f041114750f7d723f724c41225`.

## Risk and remaining boundary

Risk is high because the increment combines real PCI writes with
device/domain-qualified DMA candidates while retaining an unrelated complete
process. Review covered owner return, complete retained-state validation,
candidate backing and request facts, bus-master and device rollback, later
sequence accounting, negative cases, the non-counted timeout, and duplicate
complete physical execution.

This increment returns no DMA owner. It does not construct or enable a queue,
grant device ownership, notify the device, consume entropy for the static
process, publish `AT_RANDOM`, initialize TLS, construct a user context, enter
ring 3, execute BusyBox, or add a syscall.
