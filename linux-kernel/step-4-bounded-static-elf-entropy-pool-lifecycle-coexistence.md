# Step 4 bounded static ELF entropy-pool lifecycle coexistence

## Outcome

The qualified x86-64 path now performs one complete entropy-pool lifecycle
while the complete static ELF process remains behind its acquisition owner. It
revalidates the retained process, replays the exact device path through an
assessed 32-byte sample, creates the exact empty 256-bit pool, conditions the
sample as the first BLAKE2s contribution, and transfers exact zero-to-256 pool
credit only after mixing succeeds.

Before returning, the transaction resets and verifies the transport,
overwrites and reads back the payload and ring, then overwrites and reads back
all sample and pool storage. It reverses bus mastering, restores gateway state,
returns the same affine owner, and publishes
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_OK` before nested rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-POOL-LIFECYCLE-COEXISTENCE-001`
adds no core-language element. Negative tests reject substituted owner, pool,
policy, and marker; the exact parser rejects every substituted transport,
quality, conditioner, or contribution resource. Structural tests prove exact
creation, conditioning, credit order and arithmetic, complete destruction,
same-owner return, and later-sequence preservation.

Artifact `/154`, provider and provider-object `/144`, root object `/143`, gate
`/156`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 394 operations; the provider plan contains 147; the
root contains 886 fatal edges, a 455-event semantic trace, 7,871 typed
relocations, 5,751 UART calls, 109 required text symbols, 48 PCI-configuration
reads, 21 masked PCI-configuration updates, twenty DMA-buffer creation calls,
nine opaque DMA-reference bindings, 109 DMA integer stores, sixteen DMA
preparations, sixteen DMA completions, 93 DMA integer loads, four sample-
capture calls, two pool-mixing calls, 21 PCI-region DMA-reference publications,
248 control references, 29 payload references, nine raw-sample references,
four pool-state references, 191 ordinary PCI-region reads, five bounded
eventual reads, 177 PCI-region writes, 300 opaque-storage derivations, and 150
observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 150 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `bbcdc9acd5cb637d1988a53bb13ce69dda1bac0e017d88fc9a364e4ed81ef230`;
- boot image: `58ed087d22b4e8c2c71a8caf6ad8800e3f458ca71d5faabbacd4a5450e409a7e`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `3f87ceb5eb93c425451926c29a1ed6368e77e346fac3d2f7e316e91c3c90e6d8`;
- boot provenance: `c85498bce0ec3a8b87e46bb3ec55828c1a2f84409ffb6e4376fe3120f2265212`;
- serial: `08b3d52327f795dd702fa8dd5886b135090060153b882c47d6f30a549f59bd8b`;
- evidence: `c4380cac39731292a9524ac624c1e941c84c2ff3c5a01e4d0bc200f0996ce5ec`.

## Risk and remaining boundary

Risk is high because this increment turns credited input into credited,
conditioned pool state beside a retained complete process. Review covered
mix-before-credit ordering, BLAKE2s identities, credit arithmetic, target-
independent policy, opacity, verified destruction, rollback, negative cases,
and duplicate complete physical execution.

This increment destroys the credited pool instead of initializing a generator.
It does not extract output, publish `AT_RANDOM`, initialize TLS, construct a
user context, enter ring 3, execute BusyBox, or add a syscall.
