# Step 4 bounded static ELF initial x86-64 FS base

## Outcome

The bounded static-process startup path now continues after its initial break
query and growth through one exact thread-pointer install. The clean-room
executable writes a known byte at `0x005f7400`, requests
`arch_prctl(ARCH_SET_FS, 0x005f7400)`, checks the zero result, reloads that byte
through FS-relative address zero, and only then requests `exit_group(0)`.

The x86-64 provider admits syscall 158 and operation `0x1002` only in the exact
post-growth state. It proves the selected byte belongs to the live readable
anonymous reserve, tolerates only hardware Accessed/Dirty status changes in the
heap leaf, retains the selection privately, installs `IA32_FS_BASE`, and
returns through `SYSRETQ`. Terminal handling restores the exact pre-entry FS
base, clears the retained selection and syscall scratch state, and then
completes the existing heap-leaf revocation and reserve erasure.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-X64-FS-BASE-001` refines the existing
target-qualified thread-pointer and static-process lifecycles. Source selects
only the startup profile; native syscall, operation, address, register, MSR,
and instruction details remain provider-owned. Checked source emits
`TOPAL_KERNEL_STATIC_ELF_INITIAL_X64_FS_BASE_OK` only after the completed owner
proves query/growth/install/access/terminal ordering. Negative tests reject a
different ABI, startup profile, terminal result, policy, query marker, growth
marker, FS-base marker, or exit marker.

Structural qualification proves the 108-byte clean-room sequence, native
selectors, exact candidate and result, live heap-leaf identity with normalized
hardware status bits, retained private state, physical `WRMSR` installation,
FS-relative load, terminal gating, saved-base restoration, scratch and
selection clearing, nineteen-qword inverse clearing, and unchanged heap
rollback.

Artifact schema `/160`, provider `/150`, provider object `/142`, root `/149`,
boot adapter `/69`, boot resource `/2`, and gate `/162` identify the changed
contract. The generated root has 407 operations, a 468-event semantic trace,
8,752 relocations, and 6,130 console-writer relocations. The formal systems
profile now contains 152 rules.

## Physical evidence

Two complete independent pinned `pc-q35-10.2`/`qemu64-v1` executions produced
identical canonical evidence with SHA-256
`b599223c95a36b422ecd7ba0f2fc570c96fd84858bbbae049df334209b676dc2`.
Both reached all 158 ordered observations, including initial-break query and
growth, FS-base installation and relative access, terminal return, complete
static-process rollback, and the unchanged later kernel/user probes. One
intervening run completed the static-process rollback but timed out later in
the pre-existing scheduling sequence; the gate rejected it and did not replace
evidence. The two complete runs record:

- linked kernel: `959cac1bc860fe326cec728fed3a5f0f71b333f140c7e24bed958abcc4aac128`;
- boot image: `9d2dbfeccc5f4a896e080f619826be64b2fbfdfd3c3ea3c201d27dca31869064`;
- boot resource: `f334c7f063cb5176828c1bb4ed7fcdbe65f4b83354db41fd01b2969f3d8799c9`;
- static clean-room image: `3cea49951d5c85d14397561bee0343c7eea993957e40e168a91813acca00728c`;
- artifact provenance: `3d6a6e3dade34970becb135c3c708b234aaaa5622f38465359f5dc46988f0748`;
- boot provenance: `ffc26026861a5782730473658c65464ab07561beb5a4324e55f6f4911a17e794`;
- source: `bb614071a7b2bc3a0da6d99d091fa53546909b2473c4e773cc9a06333b32a2f2`;
- serial output: `4eaf272563b16b18785846192111b7e5242aeea54242793cf6c7680f8e1a5431`.

## Risk and review

Risk is high because this changes architecture-visible thread state across a
resumable privilege boundary. Review covered exact ABI ordering, readable
mapping authority, hardware PTE status bits, saved syscall registers,
`SWAPGS`/`SYSRETQ` pairing, physical installation, relative addressing,
pre-entry MSR restoration, private and scratch-state clearing, terminal
ordering, heap rollback, fail-closed paths, structural inspection, full
workspace validation, and duplicate complete physical execution.

## Remaining boundary

This increment installs one exact initial x86-64 FS base. It does not add
`ARCH_GET_FS`, another architecture-control operation, dynamic TLS allocation
or initialization, multiple threads, cloning, scheduler TLS restoration,
signals, another architecture implementation, redistributed BusyBox bytes in
qualification, a prompt, or general application compatibility. The next
static-process increment can follow the pinned executable's observed
`set_tid_address` request from this proven thread-pointer boundary.
