# Bounded static ELF initial `getrandom`

## Decision

The static initial-random delivery may refine its 32-byte qualified output into
two affine results before destroying the generator: the first 16 bytes remain
the exact `AT_RANDOM` object, while the second 16 bytes become one private,
single-use process reservation governed by
`TOPAL-SYSTEMS-ENTROPY-RESERVE-001`. All provider output storage and every
other entropy resource are still wiped and destroyed before process entry.

After the optional `readlinkat` fallback, the pinned process requests Linux
`getrandom(0x005f5e68, 8, GRND_NONBLOCK)`. The provider validates the live
startup state, exact flags and length, sealed executable continuation, and
complete writable destination in the static image. It copies the requested
eight-byte prefix from the private reservation, verifies publication, discards
and wipes all remaining reserved bytes, marks the reservation consumed, and
returns eight. Terminal handling requires consumed zeroed state and clears the
private service marker before qualification restoration.

Source names an initial random-service reservation and delivery profile, not
bytes, addresses, syscall numbers, flags, registers, or copy mechanics. The
portable abstraction is an affine opaque output reservation bound to one
process; architecture providers own native ABI lowering and fault containment.

This systems-only refinement does not itself retain the generator, permit a
second nonempty call, accept another size or flag profile, reseed, expose
random bytes, redistribute BusyBox, or provide a prompt. The approved
[reusable-service refinement](bounded-static-elf-reusable-getrandom.md) carries
exclusive generator ownership without changing this first-call result.
