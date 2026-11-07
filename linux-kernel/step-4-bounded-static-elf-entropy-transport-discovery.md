# Step 4 bounded static ELF entropy-transport discovery

## Outcome

The qualified x86-64 path now completes the read-only virtio-rng transport
discovery while the complete static ELF process remains behind its opaque
acquisition owner. After completely revalidating the retained process, it
performs the remaining 23 reads of the sealed 24-dword Q35 PCI configuration
sequence through the existing checked generic provider boundary. Together with
the preceding device-identity read, this proves the capability list, BAR roles,
offsets, lengths, and notification multiplier. The transition returns the same
affine owner and publishes `TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_OK` before
the existing nested rollback.

The checked source names only the acquisition owner, portable transport
identity, and policy. It cannot observe a PCI coordinate, capability field,
configuration value, port, process resource, allocator state, or entropy byte.
The provider performs no configuration write, BAR access, DMA, negotiation,
process mutation, or output use.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-TRANSPORT-DISCOVERY-001` adds no core-
language element. Negative tests reject substituted owner, transport, policy,
and marker. The affine semantic model requires the exact live acquisition owner
and returns it unchanged. Structural tests prove complete process validation,
one new typed provider dependency, the additional 23 calls to the checked PCI
read provider, and no configuration-write or BAR/DMA operation.

Artifact `/142`, provider and provider-object `/132`, root object `/131`, gate
`/144`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 370 operations; the provider plan contains 135; the
root contains 222 fatal edges, a 431-event semantic trace, 5,718 typed
relocations, 5,178 UART calls, 97 required text symbols, 48 PCI-configuration
read calls, and 138 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
138 observations in order within the supported allowance and produced
identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `894dadaa02c03f41f004a22704ae28220c0f165ebd371a6e9fc0f4d4169241a9`;
- boot image: `61ed8be86aeb31d9dddb6560504dbb1d5b254a96fd46b02690ef511c838bdee8`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `0e66dcb65cc25152f2e0dec2e32ce48a178d4dc99e90455f4a8f374bd197acc3`;
- boot provenance: `a4dcc7a10de6509763085e40e76d4b1745eaf2e99b16671489ca9088014afefc`;
- serial: `5eb0a4862c84efe8a7ed180d59ae15c906df613d0902e5ae0fc1bfe2dc9f18c1`;
- evidence: `caff96a333e7f20fd0a37e430fb3e5e524c7b8e67d5839cd3033577c749bd145`.

## Risk and remaining boundary

Risk is high because the increment duplicates a real hardware-discovery
sequence while preserving later qualification behavior; a wrong selector,
expected value, hidden write, or mutation of the original observation counter
would invalidate the claim. Review covered exact affine owner return, complete
retained-state validation, all sealed selectors and expected values, generic
provider reuse, unchanged later sequence accounting, structural call counts,
negative cases, exact nested rollback, and duplicate successful physical
execution.

This increment performs only read-only transport discovery. It does not
negotiate or drive the device, access a BAR, allocate DMA, retain the owner
across DMA or extraction, consume entropy for the static process, publish
`AT_RANDOM`, initialize TLS, construct a user context, enter ring 3, execute
BusyBox, or add a syscall.
