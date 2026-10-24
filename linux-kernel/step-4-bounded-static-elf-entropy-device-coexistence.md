# Step 4 bounded static ELF entropy-device coexistence

## Outcome

The qualified x86-64 path now performs one real entropy-side hardware
observation while the complete static ELF process remains behind its opaque
acquisition owner. It completely revalidates the retained process, reads pinned
PCI function `0000:00:05.0` register zero through the existing checked generic
provider boundary, requires device/vendor dword `0x1044_1af4`, returns the same
affine owner, and publishes `TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_OK` before
the existing nested rollback.

The checked source names only the acquisition owner, sealed device identity,
and policy. It cannot observe a PCI coordinate or value, port, stack address,
process resource, allocator state, or entropy byte. The provider performs no
configuration write, BAR access, DMA, negotiation, process mutation, or output
use.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-DEVICE-COEXISTENCE-001` adds no core-
language element. Negative tests reject substituted owner, device, policy, and
marker. The affine semantic model requires the exact live acquisition owner and
returns it unchanged. Structural tests prove complete process validation before
one additional call to the checked PCI read provider with exact selector and
identity, and prove the absence of writes, BAR/DMA operations, or process
stores.

Artifact `/141`, provider and provider-object `/131`, root object `/130`, gate
`/143`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 368 operations; the provider plan contains 134; the
root contains 198 fatal edges, a 429-event semantic trace, 5,626 typed
relocations, 5,134 UART calls, 96 required text symbols, 25 PCI-configuration
read calls, and 137 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
137 observations in order within the supported 60-second allowance and
produced identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `1c622511787c29c52506c64ac8200fc1db999268a49867e0f65ecfd46518053a`;
- boot image: `86a01d99418e6fed4399adaa8f7b2578e998cc143c0fa9d93667334513f4d0ca`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `b6d57c2b2d4c9dde972e2ef3c3597963b08bd4d22634065151db67231ce27e30`;
- boot provenance: `63e8cd1dbcfbeae0f2e2438c192b9ed7324c9d2ba83258d9e9de88866092e7a1`;
- serial: `2ac0573481e3db5358a118dc848632f37d4723ef2196297e8a983ba21f847174`;
- evidence: `51b75642e91b49817453981f0be35be192635d69332f882b0dad6ecdf4df226a`.

## Risk and remaining boundary

Risk is high because the observation crosses from prepared-process ownership
into real hardware access; a wrong selector, a hidden write, or process-resource
reuse would invalidate the isolation claim. Review covered exact affine owner
return, complete retained-state validation, the selector and expected identity,
generic provider reuse, structural call counts, negative cases, exact nested
rollback, and duplicate successful physical execution.

This increment performs only read-only device identity observation. It does not
negotiate or drive the device, retain the owner across DMA or extraction,
consume entropy for the static process, publish `AT_RANDOM`, initialize TLS,
construct a user context, enter ring 3, execute BusyBox, or add a syscall.
