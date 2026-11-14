# Step 4 bounded static ELF initial-random delivery coexistence

## Outcome

The qualified x86-64 path now turns the transient entropy-generator result
into one persistent Linux initial-process result. It revalidates the retained
static ELF process, replays the complete virtio-rng pipeline, publishes native
auxiliary tag 25 and a distinct aligned 16-byte random object, verifies the
copy without observing its value, and returns an affine randomized-process
owner only after every provider output, key, pool, sample, DMA, ring, device,
and gateway resource has been erased and verified.

The gate then exercises the qualification-only inverse: it validates and
erases the complete random object and auxiliary-vector pair, restores the
prior acquisition owner, and proves the existing nested rollback remains
exact. Production entry can consume the forward owner without that inverse.

## Validation boundary

Rule
`TOPAL-SYSTEMS-STATIC-ELF-INITIAL-RANDOM-DELIVERY-COEXISTENCE-001`
adds no core-language element. Negative tests reject substituted owner,
delivery, policy, and marker; exact parsing rejects every substituted entropy
resource, interface, or output. Structural tests bind both native object
qwords, the tag/value pair, provider-selected pointer, complete output wipe,
exact inverse, and typed provider dependencies.

Artifact `/156`, provider and provider-object `/146`, root object `/145`, gate
`/158`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
148-rule systems profile admits 400 checked-source operations; the provider
plan contains 150; the root contains 1,094 fatal edges, a 461-event semantic
trace, 8,520 typed relocations, 5,905 UART calls, 112 required text symbols,
48 PCI-configuration reads, 25 masked PCI-configuration updates, 24 DMA-buffer
creation calls, eleven opaque DMA-reference bindings, 143 DMA integer stores,
twenty DMA preparations, twenty DMA completions, 127 DMA integer loads, six
sample-capture calls, four pool-mixing calls, 27 PCI-region DMA-reference
publications, 328 control references, 37 payload references, fifteen raw-
sample references, ten pool-state references, ten generator-key references,
nine output references, 231 ordinary PCI-region reads, seven bounded eventual
reads, 207 PCI-region writes, 394 opaque-storage derivations, and 153 ordered
observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 153 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `06c924fd175ee8f9d011eb5e48deef2242b5405867ec22f86753c6212c8578e3`;
- boot image: `85814d948cea391640dc8e5ca6055549e17de81b3cbd011946a2b4bb2ee9ef01`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `cb123feb5683e062b4c45f2c1424b9608a0719cef471868c45228c8754de680b`;
- boot provenance: `fd4a4c3278c58f83e9fe3aa459de935914c2947dacff220938e55388c8744870`;
- serial: `73609808adcc38ed652b49542c5e5baa2f9b291efca971963978d3c038c70eac`;
- evidence: `bac4b8f8b71732a92fc685cadbab28750e660b12361a05f12c139ba7c851e177`.

## Risk and remaining boundary

Risk is high because this increment transfers secret material into a retained
process and changes its affine owner. Review covered native placement and non-
overlap, atomic tag/object publication, exact owner transitions, complete
provider-output and resource destruction, inverse erasure, rollback, negative
cases, and duplicate physical execution.

This increment does not construct a user context, enter the static process,
execute BusyBox, add a syscall, or provide `getrandom`.
