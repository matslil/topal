# Step 4 bounded initial-process program break

## Outcome

The initial process now has one Linux-compatible byte-granular program break
inside its existing RW/NX data page. Image construction derives the initial
break from validated runtime ELF `p_memsz`, records both initial and current
values in private process state, and zeroes every byte from runtime `p_filesz`
through the page end before user entry.

The application queries the current value with `brk(0)`, grows by 16 bytes,
writes and reads the first newly admitted heap byte, shrinks to its initial
value, and verifies that a request beyond the reserve returns the unchanged
prior break. Data-role teardown clears both private values before mapping
revocation. No core-language rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-BRK-001` requires a live user context, executable
return continuation, runtime-derived initial value, complete future-heap
zeroing, one bounded current state, Linux prior-break failure results, and
teardown clearing. Structural tests prove runtime extent use, page-tail zero
looping, private initial/current stores, broad executable-return validation,
query/grow/shrink/rejection calls, the heap write/read, and both teardown
clears.

Artifact schema `/57`, provider `/54`, provider object `/54`, artifact root
`/55`, boot adapter `/17`, and gate `/59` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. Every constructed-image post-return
witness is reachable only after all four program-break calls and the admitted
heap-byte round trip succeed. The evidence records:

- linked kernel: `886e9ce81bbfd8eeda7f50a8edf62d59af34787f5601a194bf2e5b722ba6b4a6`;
- boot image: `8a213923c4dddac07776bf1d7eef921e690112a38a48cd04eeb79295c9759446`;
- boot resource: `485fd6ddf5c763c433eeae53f4081a69b78839b4fd5ebfbbaa1ba638f8cc196d`;
- generated default image: `299df950fecf5e793c0f8abc57f8a0443dbd1b8001580ecb3ed4d95aa913ba33`;
- supplied image: `2b4ae45bbf21088610d634f79d2d3802f62bbc4823b2df7a3c722fc2494f4abe`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `8cc786700b1422b9c1de8bc799ef94f3d068ceeb05baee8a91c15dd775c0be03`;
- boot provenance: `135ffc64470e7d3e31cc8d258acb581f9c13706c86949fe39f56ec5601bf7d41`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `cea8d70c94a4464088474a50b6350ce72f242eceaa175d1c3b4c91493a3ef1b7`.

## Risk and review

Risk is high because this adds mutable address-space state and extends which
user bytes are intended for allocation. Review covered runtime ELF authority,
overflow and page bounds, allocator-residue prevention, byte-granular state,
query/grow/shrink/failure returns, executable continuation validation, private
state lifetime, teardown order, W^X preservation, structural x86 checks, full
workspace validation, and two independent physical runs.

## Remaining boundary

The reserve is only the tail of one already mapped data page. This increment
has no additional page allocation, VMA collection, `mmap`, `mprotect`,
`munmap`, ASLR, resource limits, overcommit, locked memory, page-fault growth,
copy-on-write, multiple processes, concurrent threads, or general userspace
allocator. Those remain later memory-service work.
