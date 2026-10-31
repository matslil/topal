# Step 4 initial console `lseek`

## Outcome

The Topal kernel now implements Linux x86-64 `lseek` classification for every
live initial console description and alias. The console has stream semantics
and no file-position state. Accepted seek modes return `-ESPIPE`; an
unsupported mode on a live console returns `-EINVAL`; invalid or closed slots
return `-EBADF` before mode validation.

The signed offset is deliberately ignored only after descriptor and mode
classification establishes a live non-seekable object. No success path,
position update, per-slot offset, or shared-description offset exists.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-LSEEK-001` adds no core-language element. Language,
semantic, artifact, and provider-object tests cover x86-64 syscall 8, unsigned
seek modes through `SEEK_HOLE=4`, exact `EBADF`/`EINVAL`/`ESPIPE`, descriptor-
before-mode ordering, all initial description identities, and absence of
position state. The ring-3 image requires `ESPIPE` for `SEEK_CUR` with a
nonzero offset on input and output, `EINVAL` for live mode five, and `EBADF`
for mode five after slot zero closes. AArch64/RISC-V pressure testing retains
the portable stream contract while selecting syscall 62.

The checked root contains 144 fatal edges, a 319-event semantic trace, 3,144
relocations, and 2,760 UART calls. The provider plan retains 84 operations and
160 entropy bytes. Artifact `/107`, provider/provider-object `/99`, root `/98`,
boot adapter `/67`, and gate `/109` identify this contract.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 85
ordered observations with identical hashes. `TOPAL_KERNEL_USER_LSEEK_OK` is
reachable only after both live console directions return exact `ESPIPE`, live
mode five returns `EINVAL`, and a later closed-slot request returns `EBADF`.
Recorded hashes are:

- linked kernel: `d313be8af882fddf2c999869d5b711b1c6a551275a1b778728cd515d180d2ee8`;
- boot image: `6cf1243184e81a8ef6cf42103b9bdc4e8ce41a99bb447e8fe7a7967518458612`;
- boot resource: `278556a143fa2963a745a6ad287f2c77bccde55f51e860c886bd5dae0cf0f119`;
- generated image: `958a03cf480ec4ff23fc91506a66c527d0982960c5ee3b5075c29e9c7b192819`;
- supplied image: `479349dc4375ca3b5d652bb8e05de2de5e771c222adceecb2a9c26f787128c69`;
- artifact provenance: `dd5b6ccfd34d17f52d77150a83cd736c4173a5220e2504311c6a4d7bea3a74b5`;
- boot provenance: `2d3b452eddc7b23413a8ecc5adf853e29e888361045db2b26302aea303fb925b`;
- serial output: `786107cf285bcb660af8ad3e11e8e547a146dd5378b7dd0c5e5e385bf8b9060c`;
- evidence: `393af679bfcd2e9c57ec53ad53ee266c8a7e4c00f0c2fed9d0ffff3c1aec90c5`.

## Risk and remaining boundary

Risk is medium because errno classification and precedence are externally
observable, but this increment allocates no state, accesses no user pointer,
and mutates no object. Review covered Linux baseline ordering, unsigned mode
bounds, signed-offset independence, every initial description, alias behavior,
closed-slot precedence, structural dispatch, executable-page capacity, and
duplicate physical execution.

The profile still has no seekable object. It does not implement regular-file
offsets, shared-position synchronization, positioned I/O, directories, block
devices, pipes, sockets, or a general VFS seek operation.
