# Step 4 bounded static ELF entropy-generator lifecycle coexistence

## Outcome

The qualified x86-64 path now performs one complete entropy-generator
lifecycle while the complete static ELF process remains behind its acquisition
owner. It revalidates the retained process, replays the exact virtio-rng path,
creates and credits the 256-bit pool, initializes the generation-one ChaCha20
key, and extracts the exact generation-two output-one range.

Before returning, the transaction resets and verifies the transport,
overwrites and reads back the payload and ring, and overwrites and reads back
all sample, pool, generator-key, and output storage. It reverses bus mastering,
restores gateway state, returns the same affine owner, and publishes
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_OK` before nested rollback.
No output is delivered to the retained process.

## Validation boundary

Rule
`TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-GENERATOR-LIFECYCLE-COEXISTENCE-001`
adds no core-language element. Negative tests reject substituted owner,
generator, policy, and marker; the exact parser rejects every substituted
transport, quality, pool, conditioner, contribution, algorithm, or output
resource. Structural tests prove initialization after exact credit, extraction
after initialization, complete destruction, same-owner return, and later-
sequence preservation.

Artifact `/155`, provider and provider-object `/145`, root object `/144`, gate
`/157`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
147-rule systems profile admits 396 checked-source operations; the provider
plan contains 148; the root contains 989 fatal edges, a 457-event semantic
trace, 8,170 typed relocations, 5,805 UART calls, 110 required text symbols,
48 PCI-configuration reads, 23 masked PCI-configuration updates, 22 DMA-buffer
creation calls, ten opaque DMA-reference bindings, 126 DMA integer stores,
eighteen DMA preparations, eighteen DMA completions, 110 DMA integer loads,
five sample-capture calls, three pool-mixing calls, 24 PCI-region DMA-reference
publications, 288 control references, 33 payload references, twelve raw-sample
references, seven pool-state references, seven generator-key references, six
output references, 211 ordinary PCI-region reads, six bounded eventual reads,
192 PCI-region writes, 347 opaque-storage derivations, and 151 observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 151 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `bec3c8fe5ffb74a1e68a6e445495fd9dbafa1701644dc17cd8eaa55386f6b159`;
- boot image: `584f7162acf61fcdae5cc881971cc34247f5c0fce58a99846f0cc1cddfc66d49`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `c1021125663fee24557d452431de0075d66218b0af7f9fbfa50a50e197b1feb2`;
- boot provenance: `adf3436669e6efdb7fb4c0211b6fca3fea461082dcb5e0ba870c25f43eeb1e3f`;
- serial: `40d378c674c9d591a2cf2373681031dd488edad3eded7804c79f4adcc83ffaed`;
- evidence: `f3b8b873071c56ea6c7f9db0ccfbd5f6d99b4783bcaf0184316a93b6b22f88ca`.

## Risk and remaining boundary

Risk is high because this increment derives a key and output from credited
entropy beside a retained complete process. Review covered ordering, exact
request identities, provider-call ABI, opacity, verified destruction,
rollback, negative cases, and duplicate complete physical execution.

This increment destroys the output instead of delivering it. It does not
publish `AT_RANDOM`, initialize TLS, construct a user context, enter ring 3,
execute BusyBox, or add a syscall.
