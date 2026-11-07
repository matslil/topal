# Step 4 bounded virtio-PCI capability discovery

## Outcome

The Topal kernel now discovers the complete modern virtio-PCI layout of the
pinned Q35 entropy function at `0000:00:05.0`. Checked source performs 24
ordered PCI configuration dword observations: function identity, capability
head, MSI-X link, and the headers and payload fields of the PCI-access,
notification, device, ISR, and common virtio capabilities. It then emits
`TOPAL_KERNEL_VIRTIO_PCI_CAPABILITIES_OK`.

Capability walking and interpretation remain source/library policy. Every
observation uses the existing portable
`topal.systems.device.pci-configuration.read-dword/1` operation, while the
x86-64 provider remains one shared mechanism-1 transaction. No provider
operation is named for virtio and no assigned BAR address enters portable
source semantics.

## Validation boundary

Rule `TOPAL-SYSTEMS-VIRTIO-PCI-DISCOVERY-001` seals the 24-request order and
requires monotonically assigned transaction identities. Negative tests reject
omitted, duplicated, reordered, substituted, misaligned, and out-of-range
requests. Structural tests prove 24 calls to the one typed provider symbol,
90 fatal edges including one per read, and an exact 228-event semantic trace.

Artifact schema `/75`, unchanged provider and provider-object schemas `/71`,
artifact-root schema `/73`, boot-adapter schema `/35`, and gate schema `/77`
identify the contract. Focused compiler, language, and semantics suites, gate
fixtures, formatting, and the complete workspace suite pass.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The device keeps its `/dev/urandom` backend, 1024-byte quota, and
1000-ms period, but this increment performs configuration-space reads only and
does not consume entropy. It attaches no disk or network device.

Two independent pinned-QEMU runs were byte-identical and reached all 53
ordered observations. Hashes are:

- linked kernel: `501aa142cacc4027236903ede8217e51cd1db60fda664a207fb4d53a2c8ea840`;
- boot image: `cfee31705cdef0e5fbdaac18a66662545e8f7affd3421ce7c98bcabbc1e5d148`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `a831cac8ef516ef32b32b309d9e6be2f569f62115ac9eaeb057e1ac62a33a080`;
- boot provenance: `bd96b23242519a51940d6c4c175825d1869d44e00d3da7070f2503221efb37d5`;
- serial output: `d5382d7983a85959fcc43846444bd05d754dac8668dc268dfcdfa855d3632473`;
- evidence: `b623e9aa41f19fa231430c6d4fe12cde409f48197e429a3d2da37a62425ed9ed`.

## Risk and remaining boundary

Risk is medium because a wrong capability interpretation would direct later
privileged device access at the wrong region. Review covered Virtio 1.4 field
meaning, exact QEMU layout, link order, target-neutral PCI coordinates,
provider privacy, semantic ordering, structural call counts, duplicate
deterministic execution, and regression tests.

Opaque BAR resolution and mapping, PCI command enablement, virtio feature
negotiation, interrupts, virtqueues, DMA, entropy delivery, `AT_RANDOM`, and
`getrandom` remain unavailable.
