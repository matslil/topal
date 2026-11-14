# Step 4 constructed-image standard-input read

## Outcome

The supplied initial ELF now consumes one application-visible input byte using
Linux x86-64 `read(2)`. After its stdout write, the constructed continuation
reserves a bounded stack region, invokes `read(0, candidate, 1)`, requires an
exact one-byte result, verifies the byte `q`, restores the stack, and continues
through identity and terminal syscalls.

The harness owns a pseudo-terminal endpoint connected to the emulated Q35 ISA
serial device and injects `q` only after observing the boot-resource witness.
The provider polls the receiver, stages the byte privately, and performs a
fault-contained copy to the exact user candidate. No core-language rule
changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-READ-001` restricts dispatch to descriptor zero, one
provider-selected writable candidate, extent one, the exact return site, and
the declared input byte. It requires private staging, pointer disposal, exact
result and byte validation, balanced stack disposal, and the ordered
`TOPAL_KERNEL_USER_READ_OK` post-return witness.

Artifact schema `/54`, provider `/51`, provider object `/51`, artifact root
`/52`, boot adapter `/14`, and gate `/56` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. Evidence binds the injected byte and
its SHA-256 identity. The user write marker occurs during execution; the read
witness follows exact terminal return and precedes all other constructed-image
witnesses. The evidence records:

- linked kernel: `57b34971ddf186f6c6da073783135d2a5a0edf379b273db0c2e09c46440426f5`;
- boot image: `f45e3ce91172c7dc5b5f1244f96bac82a0e1070c2fe3578861980e0e0c9877ec`;
- boot resource: `605e482d6c03a69f22617d547aa6cdaef22402cba77b6ebc31e0b6a4b95e3000`;
- generated default image: `8779eb5f532abf434626b6ff48338beb5dc347e82bb7a0269996c0f6f0410f33`;
- supplied image: `de55df80b6263ae77afa3f7896772cf1ff5f35fba1a8db69800c49387bc55d11`;
- serial input: `8e35c2cd3bf6641bdb0e2050b76932cbb2e6034a0ddacc1d9bea82a6ba57f7cf`;
- artifact provenance: `ff0b2aa1f0b787d84eb324de1e0229d374115cbfd457b054f363e3f032982c55`;
- boot provenance: `ef2edbfae40b53335504a9c934079394a288305cb362412ff08bbd62b26b57ac`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `e4bbe33da1152044f1ab058fed9f94cc3de8288a60f401d9ae094bfe49f6ebaa`.

## Risk and review

Risk is high because the change introduces external input, a writable user
transfer, and a bidirectional physical-device test channel. Review covered
deterministic injection timing, pseudo-terminal lifetime, exact dispatch facts,
input polling and staging, page-fault recovery, result/byte validation, pointer
disposal, stack balance, ordered semantic state, structural x86 checks, full
workspace validation, and two independent physical runs.

## Remaining boundary

This is one predetermined byte, not a reusable or blocking `read`
implementation. There is no input stream, buffering, line discipline, terminal
ioctl, readiness polling, descriptor table, VFS, path lookup, filesystem,
dynamic loader, process creation, or shell. Those remain subsequent
userspace-gate work.
