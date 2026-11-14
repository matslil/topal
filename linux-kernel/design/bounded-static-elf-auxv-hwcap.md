# Bounded static ELF hardware-capability facts

This record resolves `TK-DEC-152` as the native hardware-capability refinement
of the guarded static ELF stack. It is a systems-profile refinement only:
checked source composes the platform-qualified block with one sealed capability
identity, while providers own native masks, auxiliary-vector tags, placement,
and mutation representation.

## Decision

`static image construction add stack hardware capabilities` shall require the
exact live `PlatformStackEntryStaticImageConstruction` and nested backing from
`TK-DEC-151`. It shall accept only that owner, that backing identity, value
`native-userspace-hardware-capabilities`, and policy
`bounded-linux-auxv-before-null`. It shall produce one affine
`HardwareCapabilitiesStackEntryStaticImageConstruction` retaining every prior
fact while atomically adding provider-consistent `AT_HWCAP` and `AT_HWCAP2`
facts after `AT_PLATFORM` and before `AT_NULL`.

The facts shall describe capabilities that the selected native userspace may
safely use on the pinned machine profile. They shall not grant instruction,
register, feature-selection, or machine-discovery authority. Source shall
neither choose nor observe either mask, tag, or encoding. Before success, the
provider shall revalidate the complete platform-qualified block and every
inactive-translation obligation, clear the complete backing, reconstruct the
target-aligned block, perform target-required visibility operations, and read
back every word, pointer relation, string byte, capability mask, terminator,
and otherwise-zero byte. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_OK` observation.

Source shall then invoke
`static image construction remove stack hardware capabilities` with the exact
owner, backing, and policy
`bounded-stack-hardware-capabilities-qualification-rollback`. Removal shall
validate the complete extended block, reconstruct and completely validate the
exact platform-qualified block, and reproduce its affine owner. Source shall
emit exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_OK` observation
before platform removal.

Neither operation shall expose a mask, tag, stack address, word encoding,
offset, alignment, CPUID leaf, feature bit location, or write mechanism. It
shall not add random facts, initialize TLS, activate the hierarchy, construct a
user context, or enter the image.

## Exact x86-64 realization

For the pinned `qemu64-v1` profile, the provider shall encode native
`AT_HWCAP=(16,0x178bfbfd)` and `AT_HWCAP2=(26,0)` after the existing
`AT_PLATFORM` entry and before `AT_NULL`. The first mask is the exact
`CPUID.01H:EDX` capability set already admitted by the provider and observed in
the reference VM. The second mask is zero because the selected profile does
not provide the kernel-enabled FSGSBASE capability represented by x86-64
`HWCAP2`.

Addition and removal shall each clear all 512 backing qwords before
reconstruction, execute one store fence, and completely read back the result.
The entry pointer remains 16-byte aligned. The exact platform string, clock,
security, credential, executable-name, RW/NX stack leaf, guard, final image
leaves, inactive hierarchy, and bootstrap root remain unchanged. No active-
address invalidation is permitted.

## Affine lifecycle and rollback

Only the exact platform-qualified block may gain the capability group. While
live, platform or prior-fact removal, ordinary entry clearing, leaf revocation,
backing release, activation, teardown, duplicate addition, or recoverable
disposition is invalid. Only exact group removal may restore the platform-
qualified owner. Any content, identity, mask, ordering, translation, or
readback failure is fatal.

## Architecture pressure test

The portable fact is a provider-consistent pair of Linux native userspace
capability sets, not x86 CPUID or its bit numbering. AArch64 and RISC-V
providers derive their own `AT_HWCAP` and `AT_HWCAP2` meanings from enabled
architectural features. Capability systems gain no executable authority from
the integer representation. Source sees only the semantic identity and affine
lifecycle.

## Alternatives rejected

Omitting the facts was rejected because libc may select implementations from
them. Exposing CPUID or raw feature-bit construction to source was rejected
because discovery, enablement, and ABI projection are provider policy. Using
host features was rejected because qualification targets the pinned guest CPU.
Publishing advertised but kernel-disabled features was rejected because the
Linux contract describes facilities userspace may safely use.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one coherent
`AT_HWCAP`/`AT_HWCAP2` group. It does not retain the facts after qualification,
add random data or a vDSO, initialize TLS, construct a user context, enter user
mode, execute BusyBox, or add a syscall.
