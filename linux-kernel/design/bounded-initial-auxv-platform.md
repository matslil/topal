# Bounded initial `AT_PLATFORM` auxiliary-vector fact

This record resolves `TK-DEC-088` as a systems-profile refinement. It exposes
the pinned native userspace platform identity to libc through a bounded pointer
without conflating that identity with machine-provider selection.

## Decision

The x86-64 provider places one native `AT_PLATFORM` entry after `AT_CLKTCK` and
before `AT_NULL`. Its value points to a NUL-terminated `x86_64` string wholly
inside the initialized user-stack block. The platform string is distinct from
the executable-name string; `argv[0]` and `AT_EXECFN` continue to alias only the
latter at its fixed address.

Constructed code validates the ordered tag, exact pointer, complete bounded
platform string, moved terminator, unchanged executable-name alias, and exact
enlarged-block consumption before continuing.

An integer platform code was rejected because Linux publishes a string pointer.
Aliasing the executable name was rejected because the objects have different
meaning and lifetimes. Deriving target selection from the string was rejected
because the fact reports an already selected ABI; it does not authorize one.

The common contract is a provider-consistent bounded platform string. AArch64
and RISC-V providers would publish their qualified native strings and layouts.
This increment does not add hardware-capability masks, runtime architecture
selection, personality switching, compatibility ABIs, or a general auxiliary-
vector builder.
