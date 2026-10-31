# Step 4 bounded static ELF entropy-sample capture coexistence

## Outcome

The qualified x86-64 path now captures one real 32-byte virtio-rng result into
an opaque raw-sample object while the complete static ELF process remains
behind its acquisition owner. It revalidates the retained process, replays the
exact transport and DMA transaction through payload ownership return, performs
the existing exact zero-credit sample capture, and keeps the payload CPU-owned.

The source and retained process observe no sample byte, address, comparison, or
branch. Before returning, the transaction resets and verifies the transport,
overwrites and reads back the payload and complete ring, then overwrites and
reads back all provider-private raw-sample storage. It reverses bus mastering,
restores gateway state, returns the same affine owner, and publishes
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_OK` before nested rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-SAMPLE-CAPTURE-COEXISTENCE-001`
adds no core-language element. Negative tests reject substituted owner, sample,
policy, and marker; the exact parser also rejects substituted device, domain,
queue, or transport. Structural tests prove exact capture, zero credit,
preserved payload ownership, absence of assessment, complete transport,
payload, ring, and sample erasure, same-owner return, and later-sequence
preservation.

Artifact `/152`, provider and provider-object `/142`, root object `/141`, gate
`/154`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 390 operations; the provider plan contains 145; the
root contains 705 fatal edges, a 451-event semantic trace, 7,328 typed
relocations, 5,653 UART calls, 107 required text symbols, 48 PCI-configuration
reads, seventeen masked PCI-configuration updates, sixteen DMA-buffer creation
calls, seven opaque DMA-reference bindings, 75 DMA integer stores, twelve DMA
preparations, twelve DMA completions, 59 DMA integer loads, two sample-capture
calls, fifteen PCI-region DMA-reference publications, 168 control references,
21 payload references, four raw-sample references, 151 ordinary PCI-region
reads, three bounded eventual reads, 147 PCI-region writes, 216 opaque-storage
derivations, and 148 observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 148 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `597a6899a16383f30fa34a0fb6aea781d0e0584a3c0ce4d723134d5fecc65618`;
- boot image: `8f0ad783d679cb803bb235fda82de398c53e7f69e3f5ca3002e68c27618f9847`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `3c1d7a4ee75b405b0f66c57354e3fa397bc108653183de5fa8b04c84d7d73f1a`;
- boot provenance: `aac43a98aee466a96eeaa14564ff93e4436d91998aff227a91fcee988959f054`;
- serial: `17163420fba1f79b36e849ee228bec424c1ab8fe0064c1e5da40eff97d66d1b7`;
- evidence: `082107642c5aed76c8f082d2a08f531f0c127e89dade5337f9cd3783a2a50f06`.

## Risk and remaining boundary

Risk is high because this increment creates a real kernel-owned copy of random
device output beside a retained complete process. Review covered DMA ownership,
exact copy extent and zero credit, sample opacity, verified destruction of all
provider-owned raw-sample bytes, transport rollback, negative cases, and
duplicate complete physical execution.

This increment deliberately destroys the sample without assessing or crediting
it. It does not initialize a pool or generator, publish `AT_RANDOM`, initialize
TLS, construct a user context, enter ring 3, execute BusyBox, or add a syscall.
