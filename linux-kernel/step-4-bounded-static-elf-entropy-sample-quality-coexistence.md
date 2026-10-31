# Step 4 bounded static ELF entropy-sample quality coexistence

## Outcome

The qualified x86-64 path now applies the sealed Q35 deployment evidence to one
opaque 32-byte virtio-rng sample while the complete static ELF process remains
behind its acquisition owner. It revalidates the retained process, replays the
exact device and DMA transaction through zero-credit capture, and performs the
target-independent 1024/1024 quality assessment that attributes 256 credited
bits without inspecting or changing sample bytes.

Before returning, the transaction resets and verifies the transport,
overwrites and reads back the payload, ring, and complete assessed-sample
storage, reverses bus mastering, and restores gateway state. No assessed
sample, credit authority, byte, or device identity escapes. The same affine
owner returns and `TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_OK` precedes
nested rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-SAMPLE-QUALITY-COEXISTENCE-001`
adds no core-language element. Negative tests reject substituted owner,
evidence, policy, and marker; the exact parser also rejects substituted
transport resources, sample, or source. Structural tests prove exact capture,
the target-independent full-quality transition, absence of provider-selected
trust, complete destruction, same-owner return, and later-sequence
preservation.

Artifact `/153`, provider and provider-object `/143`, root object `/142`, gate
`/155`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 392 operations; the provider plan contains 146; the
root contains 793 fatal edges, a 453-event semantic trace, 7,595 typed
relocations, 5,702 UART calls, 108 required text symbols, 48 PCI-configuration
reads, nineteen masked PCI-configuration updates, eighteen DMA-buffer creation
calls, eight opaque DMA-reference bindings, 92 DMA integer stores, fourteen DMA
preparations, fourteen DMA completions, 76 DMA integer loads, three sample-
capture calls, eighteen PCI-region DMA-reference publications, 208 control
references, 25 payload references, six raw-sample references, 171 ordinary
PCI-region reads, four bounded eventual reads, 162 PCI-region writes, 257
opaque-storage derivations, and 149 observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 149 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `ca79b68f47a2c943836c4eee5adbaf1d31bfd472ef180df3bd44c6aaf9ce4e56`;
- boot image: `6cc4fcba469b03103533b797861ea5962b5087471c36bcb8cb80931482145955`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `d8b7e0999bb9082b3441a8387953d002681aa2a7476742c7b891d9c4a5fc36fc`;
- boot provenance: `bb0efc260e1438493e67e7a40a809f83fb55e9879f36ded646cc9df28bc9d36b`;
- serial: `e2e0baca17a61300182256e6b8ddc997383a8b8b411516888c7358e34e68ab54`;
- evidence: `1012da914da9afc431b497b9cee301f6e33e9ac6477493625ff98cdabb31656a`.

## Risk and remaining boundary

Risk is high because this establishes credited entropy under a deployment trust
assumption beside a retained complete process. Review covered the evidence
identity, derived-credit arithmetic, separation of policy from target
mechanics, sample opacity, verified destruction, rollback, negative cases, and
duplicate physical execution.

This increment destroys the assessed sample without creating or mixing a pool.
It does not seed or extract a generator, publish `AT_RANDOM`, initialize TLS,
construct a user context, enter ring 3, execute BusyBox, or add a syscall.
