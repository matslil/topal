# Bootstrap-region source and provider contract

## Decision

The initial kernel-owned-memory qualification uses the existing bounded
bootstrap pool through a general affine `BootstrapRegion` capability. Source
performs fallible allocation through the live bootstrap context, handles the
ordinary `Result` exhaustively, borrows a successful region for checked plain
byte access, and consumes it through explicit release.

This decision completes `TK-DEC-016`. It adds no pointer, address, source
assembly, section, register, or instruction vocabulary.

## Source contract

The admitted operations are:

- `context bootstrap allocate (...)`, with named static `byte-count`,
  `alignment-bytes`, and semantic `placement` fields, returns either `Ok
  region` or `Error problem`;
- `region byte store (...)` borrows the region and stores one `Nat` from 0
  through 255 at a checked zero-based byte offset;
- `region byte load (...)` borrows the region and returns one `Nat` from 0
  through 255 from a checked zero-based byte offset; and
- `context bootstrap release region` consumes the region and retains the
  pool's monotonic high-water mark.

The initial executable subset requires a static request and static offsets.
The checker proves the request fits the declared pool, proves every access is
in bounds, and requires exhaustive allocation and byte-comparison decisions.
Every reachable success branch releases exactly once before consuming the
bootstrap context. The allocation-error action never owns a region. Region
values cannot be returned, stored, captured, passed to ordinary functions,
sent, serialized, or moved to another entry.

Dynamic requests or offsets are not approximated. They remain unavailable
until a later contract defines the necessary proof or explicit failure result.

## Memory-domain boundary

Bootstrap-region load and store mean ordinary, kernel-owned, nonconcurrent
memory. They do not mean volatile access and carry no atomic, device, DMA,
firmware, or user-memory authority. Those domains retain their separate typed
protocols and ordering/fault obligations. A region exposes neither its model
offset nor any virtual or physical address.

The meaning is portable across x86-64, AArch64, and RISC-V: allocate bounded
owned bytes, preserve byte contents, check bounds, and enforce lifetime. Each
provider may choose its concrete storage, address calculation, and machine
instructions. No x86 addressing fact becomes source semantics.

## Initial x86-64 qualification

The x86-64 provider places the pool in the already generated aligned `NOBITS`
storage. For the first statically proven request it computes a private address
from that symbol, emits an actual byte store and byte load, compares the loaded
value, and transfers to the nonreturning fatal provider on mismatch. Release
has no machine instruction in this monotonic single-execution slice, but it
remains a checked source and model transition and closes the region lifetime.

The toolchain-gate root writes a sentinel, loads it back, and emits
`TOPAL_KERNEL_MEMORY_OK` only from the equality-success action. Pinned-QEMU
evidence contains that marker after the fault-resume marker and before the final
fatal halt. Artifact inspection proves that the root retains relocations to the
generated pool and fatal provider and that the byte load has not been
constant-folded away.

This contract was approved in the project discussion before implementation and
is qualified for the initial x86-64 slice by the committed toolchain-gate
evidence.
