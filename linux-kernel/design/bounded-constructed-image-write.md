# Bounded constructed-image `write` output

This record resolves `TK-DEC-069` as a systems-profile refinement. It requires
the finalized initial ELF to invoke one Linux-compatible `write(2)` operation
from ring 3 and makes that output observable on the kernel console. It changes
no core-language rule and does not expose a general descriptor, pointer,
length, syscall, console, or device interface.

## Decision

After the constructed continuation completes its existing `uname` output and
`nanosleep` input qualifications, it must:

1. reserve a bounded region in its existing writable, non-executable stack;
2. initialize the exact 26-byte `TOPAL_KERNEL_USER_WRITE_OK` payload;
3. invoke Linux `write` with descriptor one, that exact candidate, and length;
4. require the kernel to copy the complete candidate into private kernel
   staging before producing output;
5. publish the staged bytes through the provider's existing console boundary;
6. return the exact byte count, validate it in user mode, and release the
   temporary stack region; and
7. continue through `getpid` and terminal `exit_group(0)` as before.

The payload itself is the ordered physical witness. It occurs while both
finalized image mappings and the user context remain live, before the existing
post-return image witnesses.

```text
constructed continuation -> uname -> nanosleep
  -> reserve and initialize stack bytes -> write(1, candidate, 26)
  -> complete copy_from_user -> console bytes -> return 26
  -> release stack -> getpid -> exit_group(0) -> exact kernel return
```

Source owns dispatch policy but names neither syscall number, registers,
instruction encoding, serial port, user address, nor kernel staging address.
The closed systems disposition carries the descriptor and payload facts; the
provider binds them to the native Linux ABI and board console.

## Isolation and failure contract

No byte becomes externally visible until all 26 bytes have been copied into
private kernel memory. Wrong descriptor, candidate, length, return site,
payload byte, lifecycle state, partial or faulted transfer, short return,
unbalanced stack reservation, missing output, or return after terminal exit is
fatal. The kernel does not retain the user pointer. Descriptor one denotes only
the already-qualified console sink in this closed initial-process profile; no
descriptor table or filesystem object is implied.

## Cross-architecture pressure

The portable operation is a bounded Linux-compatible write from readable user
memory to the initial process's standard output, with all-or-nothing staging
for the qualified payload. It does not prescribe syscall instructions,
register conventions, stack direction, UART type, memory-mapped versus port
I/O, or console polling.

An x86-64 provider may use `rax=1`, `rdi=1`, `rsi=candidate`, and `rdx=26`,
then poll the Q35 serial transmitter. AArch64 would use `x8=64` and its
`x0`–`x2` arguments; RISC-V would use `a7=64` and `a0`–`a2`. A provider with an
MMIO UART or firmware console realizes the same checked console effect without
exposing that mechanism. Capability architectures must derive bounded readable
authority and cannot retain or widen it.

This increment does not admit arbitrary descriptors, buffers, lengths,
partial writes, console input, descriptor tables, filesystems, pipes, signals,
restart, asynchronous I/O, arbitrary syscall selection, or general process
loading.
