# Step 4 bounded PCI configuration observation

## Outcome

The Topal kernel now proves the identity of the pinned Q35 entropy function at
`0000:00:05.0`. Checked source supplies the PCI coordinates, aligned register
offset zero, and expected `0x10441af4` device/vendor dword. The bootstrap
program continues only after one exact configuration read matches, then emits
`TOPAL_KERNEL_PCI_VIRTIO_RNG_OK`.

The portable operation carries PCI coordinates and a transaction-scoped device
effect. Its x86-64 provider constructs the mechanism-1 selector and emits one
32-bit `0xcf8` write followed by one `0xcfc` read. Port numbers, instructions,
and privilege remain private to the provider; enumeration and virtio matching
remain source/library policy.

## Validation boundary

Rule `TOPAL-SYSTEMS-PCI-CONFIG-001` validates segment/BDF ranges, dword
alignment, the sealed Q35 request, one observable read, and terminal mismatch.
Negative tests reject a wrong identity, misaligned offset, and out-of-range
device. Structural tests prove the exact provider instruction sequence, one
typed root dependency, and one fatal mismatch edge.

Artifact schema `/74`, provider `/71`, provider object `/71`, artifact root
`/72`, boot adapter `/34`, and gate `/76` identify the contract. Focused suites,
gate fixtures, formatting, and the complete workspace suite pass.

## Physical evidence

The gate attaches only the existing ISA serial console and the manifest-pinned
`virtio-rng-pci-non-transitional` device with `/dev/urandom` backend,
1024-byte quota, 1000-ms period, and PCI address `00:05.0`; it attaches no disk
or network device. Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` runs were
byte-identical and reached all 52 observations. The entropy data path was not
read. Hashes are:

- linked kernel: `518b37aab52c75b289da22b22a07ba583bc125d31638daf898e0e885f102396f`;
- boot image: `99b974be90d2751e100b74ea5329cd131d1d2004be4d630a7ea3186dc39d3f66`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `fecec658e7fac36312869cf8fd9657f4865437341447d31b1f1042c3c1fe37df`;
- boot provenance: `a222133699d76d5aa4c2fdb18db25dd7c55d57648af18e251f6c7078b6182e33`;
- serial output: `d1b80d6457a5a796b89bcace19cc13d5181d91afa83589ed5968a3c1863a2d92`;
- evidence: `4abaa7aa010080a12dcaeed46c14c57c24fea9fc09af539aaf4a70650593f2a8`.

## Risk and remaining boundary

Risk is medium because this adds privileged addressed I/O and a guest device.
Review covered cross-architecture meaning, range/alignment rejection, exact
selector construction, access width/order, provider privacy, structural ELF
inspection, duplicate deterministic execution, and full regression. General
PCI enumeration, capability walking, BAR mapping, virtio transport
negotiation, interrupts, DMA, entropy service, `AT_RANDOM`, and `getrandom`
remain unavailable.
