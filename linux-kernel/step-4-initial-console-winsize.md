# Step 4 initial console `TIOCGWINSZ`

## Outcome

The Topal kernel now implements Linux x86-64 `ioctl(TIOCGWINSZ)` for every
live initial console description. It publishes the native eight-byte
`struct winsize` with zero rows, columns, horizontal pixels, and vertical
pixels. Zero records unspecified geometry because the pinned serial transport
provides no authoritative dimensions; all descriptions and aliases observe
the same immutable console value.

Descriptor lookup precedes request decoding, which precedes pointer access.
Invalid or closed slots return `-EBADF`; unsupported live-console requests
retain `-ENOTTY`; invalid accepted-request candidates return `-EFAULT`. A
valid query requires one complete overflow-free writable extent, privately
stages all eight bytes, publishes them under page-fault recovery, and retains
no pointer.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-WINSIZE-001` adds no core-language element. Language,
semantic, artifact, and provider-object tests cover syscall 16, request
`0x5413`, the complete four-`u16` layout, input/output equality through
disjoint destinations, exact errors, three-stage precedence, mapping bounds,
private staging, exact copy length, fault recovery, and no retained pointer.
AArch64/RISC-V pressure testing keeps portable console identity, unspecified
geometry, and ordering while selecting syscall 29.

The checked root contains 144 fatal edges, a 318-event semantic trace, 3,118
relocations, and 2,734 UART calls. The provider plan retains 84 operations and
160 entropy bytes. Artifact `/106`, provider/provider-object `/98`, root `/98`,
boot adapter `/66`, and gate `/108` identify this contract.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 84
ordered observations with identical hashes. `TOPAL_KERNEL_USER_WINSIZE_OK` is
reachable only after input and output publish identical complete zero-valued
layouts and a later closed-slot query returns exact `EBADF`. Recorded hashes
are:

- linked kernel: `54d2e2e2b0dd7d3a93e0a9b3bd15a05ce4dff4b4914a036f042c5e707910aea6`;
- boot image: `bbba1d980d3aa6f1c7c626ae915cc5923758e2cecbb88fd6ecdb2bfcca1ec69a`;
- boot resource: `81118371dd5809fc01a85972f4ca318837556129d95b1d68242b30dab5d26325`;
- generated image: `61252c2d61fb0a9ef075aac7b11347a6afb093a4bbe11b7e1fe733fbc1a52c29`;
- supplied image: `0bb6a1317c2cffb4d145b62e6686fce4f14a8bcfda5cfc52d80c99b2473d8af1`;
- artifact provenance: `d1585a7010f03314edf43db90baa753bab682c90de95f7d5ee7df48cb2455ecf`;
- boot provenance: `1625f340c6e7ad04e6d6b0b65f69a445497d466d39055eb9b95b6d445cf3f9e6`;
- serial output: `1c0dd5ea1816eef669db693433a5eb3efecc3791bbcb2eac778c8880cf2783c4`;
- evidence: `9b5645dd9cfae2a1e3ace20c4009ec2b98cea96cc6823b8e9cbf50fa7cae4e30`.

## Risk and remaining boundary

Risk is high because ioctl classification, native aggregate layout, reported
geometry, error precedence, and fault handling are externally observable.
Review covered every byte, exact constants, lookup/request/pointer ordering,
bounds, staging, recovery balance, description equality, unknown and closed
requests, architecture separation, structural ordering, executable-page
capacity, and duplicate physical execution.

The profile remains one immutable raw serial terminal with unspecified
geometry. It does not implement `TIOCSWINSZ`, host geometry import,
`SIGWINCH`, sessions, process groups, job control, ptys, display devices, or a
general ioctl registry.
