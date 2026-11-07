# Step 4 bounded static ELF entropy-acquisition authority

## Outcome

The qualified x86-64 path now transfers the complete entropy-pending static ELF
process into one opaque acquisition owner bound to the existing qualified first
output contract: output identity one at resulting generator generation two. It
publishes `TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_OK`, then
performs an exact qualification rollback, publishes
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_OK`, and returns the same
entropy-pending owner for the existing handoff rollback and teardown.

The checked source sees only opaque owner and policy identities. It cannot
observe the random-slot address, stack-backing identity, process resource tuple,
device state, allocator state, or entropy bytes. Authorization identifies the
only output contract a later acquisition may accept without falsely claiming
that acquisition or delivery has already occurred.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-ACQUISITION-AUTHORITY-001` adds no
core-language element. Negative tests reject substituted owner, output,
generation, policies, and markers. The affine semantic model permits exactly
one live pending or acquiring process owner and rejects handoff rollback while
acquisition authority remains live. Structural tests prove complete unchanged-
state readback, absence of device access, allocation, stores, fences,
invalidation, extraction, copy, or delivery, and exact nested rollback.

Artifact `/140`, provider and provider-object `/130`, root object `/129`, gate
`/142`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 366 operations; the provider plan contains 133; the
root contains 196 fatal edges, a 427-event semantic trace, 5,581 typed
relocations, 5,093 UART calls, 95 required text symbols, and 136 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
136 observations in order within the supported 60-second allowance and
produced identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `e802965008b148d1885a5e724ed525c54793b75e57b9e78910c1c1c886dc6f0b`;
- boot image: `b53fa3c61ceed9b6feb06f3ea3406e3b043f8073a7bb39559b5b21a4db7e6989`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `445daac1df600939c15d11d4bc1d8031422acd3f029b3ecbe75db005709df8ee`;
- boot provenance: `977c311ef5dc8290dea2e05a73166896967775bd66f517225d2c14d020a38a83`;
- serial: `b3573a71365d26e8b448693931778773af4ce48f8d564c77b1cf1e80347c8e38`;
- evidence: `3453812eac0343ad5259308ae501d946e899a26f04d1e9a2f40cee9198360fa6`.

## Risk and remaining boundary

Risk is high because an imprecise output contract, duplicated acquisition
owner, or premature handoff rollback could grant entropy mutation authority to
the wrong process. Review covered exact owner consumption and production,
sealed output identity and generation, complete process revalidation, private
backing preservation, structural machine code, negative cases, nested rollback,
and duplicate successful physical execution.

This increment establishes only acquisition authority. It does not retain the
owner across the device-to-output pipeline, acquire or consume entropy for the
static process, write or publish `AT_RANDOM`, initialize TLS, construct a user
context, enter ring 3, execute BusyBox from the static image, or add a syscall.
