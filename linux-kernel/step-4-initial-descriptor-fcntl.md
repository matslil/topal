# Step 4 initial descriptor `fcntl` metadata

## Outcome

The Topal kernel now retains one private `FD_CLOEXEC` bit per initial
descriptor slot, separately from each shared open description. Linux x86-64
`fcntl` implements `F_GETFD`, masked `F_SETFD`, and `F_GETFL`. Input reports
`O_RDONLY`; output and error descriptions report `O_WRONLY` regardless of the
numeric slot containing their alias.

Descriptor lookup occurs before command decoding. Invalid or closed slots
therefore return `-EBADF`, while unsupported commands on live slots return
`-EINVAL` without mutation. Close clears its slot flag; `dup` and non-identity
`dup2` clear the destination flag while preserving the source. Terminal
dispatch verifies the remaining flags are clear and erases all flag storage.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-FCNTL-001` refines the approved systems profile and
adds no core-language element. Language, semantic, provider-planning,
artifact, and structural tests cover `F_GETFD=1`, `F_SETFD=2`, `F_GETFL=3`,
`FD_CLOEXEC=1`, `O_RDONLY=0`, `O_WRONLY=1`, `EINVAL=22`, native syscall 72,
nine ring-3 calls, lookup-before-command machine ordering, masked flag storage,
duplication clearing, independent source flags, exact errors, and terminal
erasure. AArch64/RISC-V pressure testing selects native syscall 25 without
changing portable command or ownership meaning.

The checked root contains 144 fatal edges, an exact 315-event semantic trace,
3,037 relocations, and 2,653 UART calls. The sealed provider plan contains 84
operations and retains the 160-byte entropy-storage layout. Artifact schema
`/103`, provider and provider-object schemas `/95`, artifact-root schema `/98`,
boot-adapter schema `/63`, and gate schema `/105` identify this contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. `TOPAL_KERNEL_USER_FCNTL_OK` is reachable only after all metadata,
error-precedence, alias, close, and duplication interactions return their exact
results and terminal dispatch erases the complete slot-flag state.

Two independent pinned-QEMU runs reached all 81 ordered observations. Hashes
for the recorded run are:

- linked kernel: `f06e0ee0634bc4905e47d30c41fc80add951dc21d6958665ddfa41afa22aae75`;
- boot image: `b0d66f3b1981ffca39d6421ea45f7d453b5ee3d56c6af2402a011d8fec9ef14e`;
- boot resource: `32fd32c5f2f6955ae8899e8109f963da7114fba194399890bf4ac9f2cda8ee5a`;
- generated image: `a21ec36385a25d99aba1c6b7190816a127393076de5d07c8c5828ad5f2bda351`;
- supplied image: `d4afe21551de9a2234ca71e1d5953230bfdde31629bb65bf87ee182bf37687fa`;
- artifact provenance: `025a890c525bfde8afb548f83370e0f9c03e9c0318ec494ae624a19638f4d0f3`;
- boot provenance: `7423d96336d9d5eecabf9dd4c522821ec08b765a0f1056592180d9b1cf4df133`;
- serial output: `658da1ce311032316f5ce5380701ae25853e481ee363aa9014ed2149ed34acf5`;
- evidence: `783f473b1d0f2ee85b73f1afc9761932b9d78bc8bae3e66a7833d1e6319d85eb`.

## Risk and remaining boundary

Risk is high because descriptor-local flags and shared open-description modes
must not be conflated, and Linux error precedence is externally observable.
Review covered ownership separation, flag masking, identity and non-identity
duplication, lookup order, close and terminal erasure, exact results,
architecture separation, structural instruction ordering, and duplicate
physical execution.

The profile remains one process and thread with three fixed slots and console
descriptions. It does not provide `F_DUPFD`, `F_DUPFD_CLOEXEC`, `F_SETFL`,
other status flags, locks, leases, ownership, asynchronous notification, seals,
pipe sizing, exec-time close, fork inheritance, dynamic open descriptions,
concurrent `fcntl`, or general VFS state.
