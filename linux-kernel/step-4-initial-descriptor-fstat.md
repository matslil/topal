# Step 4 initial descriptor `fstat` metadata

## Outcome

The Topal kernel now implements Linux x86-64 `fstat` syscall 5 for each live
initial console description. It publishes the native 144-byte `struct stat`
for one immutable shared console object: device zero, inode one, link count
one, character-device mode `0600`, root ownership, `ttyS0` special-device
identity `(4, 64)`, zero size and allocated blocks, 4 KiB preferred block size,
zero timestamps, and zero padding and reserved words.

Descriptor lookup occurs before candidate access. Invalid or closed slots
therefore return `-EBADF` without inspecting the pointer. A live description
requires one overflow-free complete 144-byte extent inside a known writable
mapping; invalid, read-only, wrapping, cross-hole, or incomplete candidates
return `-EFAULT` without retaining the candidate. The provider constructs the
whole result in private staging and performs the bounded copy under page-fault
recovery.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-FSTAT-001` refines the approved systems profile and
adds no core-language element. Language, semantic, provider-planning,
artifact, and structural tests cover native syscall 5, all 18 output qwords,
the exact field offsets and constants, three ring-3 calls, descriptor-before-
pointer machine ordering, both complete writable ranges, overflow rejection,
private staging, the exact 144-byte copy limit, fault recovery, alias equality,
closed-slot `EBADF`, and absence of a retained user pointer. AArch64 and RISC-V
pressure testing selects their qualified `newfstatat` syscall and layout work
without changing portable object, range, or ownership meaning.

The checked root contains 144 fatal edges, an exact 316-event semantic trace,
3,063 relocations, and 2,679 UART calls. The sealed provider plan contains 84
operations and retains the 160-byte entropy-storage layout. Artifact schema
`/104`, provider and provider-object schemas `/96`, artifact-root schema `/98`,
boot-adapter schema `/64`, and gate schema `/106` identify this contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. `TOPAL_KERNEL_USER_FSTAT_OK` is reachable only after the original
description's complete layout is checked, an output alias returns byte-for-byte
identical metadata, and a closed alias returns `-EBADF` before pointer access.

Two independent pinned-QEMU runs reached all 82 ordered observations. Hashes
for the recorded run are:

- linked kernel: `2b4c2bebfbc71085ca632ca388da2ddb43cd2ffad3f7b3cb2be3535457d7c674`;
- boot image: `f0a5667bc6785549650ae474adcda1af7bfcc05f6af959a8bec6dd029c5dee38`;
- boot resource: `d40da81b60e7e018745a5434f4d4c324930fb256a237c2316845c9c3faedb6da`;
- generated image: `21d4bda75f6d9eb753ee7b65fb7fafba54bff0f6f224b0020cb6db45987da672`;
- supplied image: `ad0619fb66f547644f6001bb4f3b97c4e8f96f7e5123fcd91c2ecafb22d32dbd`;
- artifact provenance: `50ec8793651379863195c33353d6113e2fb54db09b806bdd9e0400034ff68b60`;
- boot provenance: `bdf4e7c03542c89ca5afe8c87084175d2074ef0cc520d2ba8bccb44a56b28a4c`;
- serial output: `6f07c9268f5485f8893c760a7180c83067a39efb1cb2d9f08c4dd495d1b60dd8`;
- evidence: `837ef95a6671f18b4488767a9da065eb1df4ec27c91a0056838afa90c7a2b9a1`.

## Risk and remaining boundary

Risk is high because the native aggregate layout, error precedence, complete-
range rule, alias identity, and fault behavior are externally observable ABI.
Review covered every byte and field, Linux x86-64 header values, descriptor
resolution order, overflow and mapping bounds, staging initialization, exact
copy termination, recovery-stack balance, alias comparison, exact errors,
architecture separation, structural instruction ordering, and duplicate
physical execution.

The profile remains one process and thread with three fixed slots and one
immutable console object. It does not provide dynamic inodes, filesystem
lookup, directories, regular files, mutable timestamps or sizes, mount state,
`statx`, `newfstatat`, legacy `stat`/`lstat`, device-node discovery, concurrent
metadata changes, or a general VFS.
