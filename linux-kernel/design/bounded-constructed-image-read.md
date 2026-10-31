# Bounded constructed-image `read` input

This record resolves `TK-DEC-070` as a systems-profile refinement. It requires
the finalized initial ELF to receive one deterministic byte through a Linux-
compatible `read(2)` from standard input. It changes no core-language rule and
does not expose a general descriptor, pointer, terminal, interrupt, or device
interface.

## Decision

The QEMU qualification harness injects the byte `q` through the selected board
console after VM launch. After the constructed continuation completes its
bounded stdout write, it must:

1. reserve one bounded region in its existing RW/NX user stack;
2. invoke Linux `read` with descriptor zero, the exact candidate, and length
   one;
3. require the provider to receive exactly one byte through its checked console
   input effect and stage it in private kernel memory;
4. copy the staged byte to the exact user candidate under the existing fault-
   recovery protocol without retaining the pointer;
5. return one to the same continuation;
6. validate both the byte-count result and the byte `q`, then release the stack
   region; and
7. continue through the existing identity and terminal syscalls.

After exact kernel-continuation restoration, source emits
`TOPAL_KERNEL_USER_READ_OK` before the other constructed-image witnesses. That
observation is reachable only after the user-mode result and byte validations.

```text
harness injects q -> provider console input -> private kernel staging
  -> copy_to_user(candidate, 1) -> return 1 -> user validates q
  -> release stack -> getpid -> exit_group(0) -> exact kernel return
  -> TOPAL_KERNEL_USER_READ_OK
```

## Isolation and failure contract

Wrong descriptor, candidate, extent, return site, lifecycle state, input byte,
partial or faulted copy, short result, retained pointer, unbalanced stack,
missing or duplicate input, early witness, or terminal return is fatal. The
single-byte qualification may poll its provider input but cannot observe an
arbitrary input stream or create a terminal queue. Descriptor zero denotes only
the initial process's already-selected console source.

## Cross-architecture pressure

The portable operation is a bounded Linux-compatible read from the initial
process's standard input into writable user memory. It does not prescribe a
syscall instruction, register convention, polling versus interrupt delivery,
UART type, port versus MMIO, firmware transport, or QEMU control channel.

An x86-64 provider may use `rax=0`, `rdi=0`, `rsi=candidate`, and `rdx=1`, then
poll the Q35 serial receiver. AArch64 and RISC-V use Linux syscall number 63
with their native argument registers and entry instructions. Capability
architectures must derive exactly one byte of writable authority and cannot
retain it.

This increment does not admit arbitrary descriptors, buffers, lengths,
blocking policy, partial reads, line discipline, terminal ioctls, signals,
restart, readiness polling, VFS objects, filesystems, or a general prompt.
