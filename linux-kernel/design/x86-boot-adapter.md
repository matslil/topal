# Initial x86-64 Linux boot adapter contract

## Decision

The initial Topal kernel uses the Linux x86 boot protocol 2.15 `bzImage`
container selected by `TK-DEC-001` and `TK-DEC-002`. This is an
architecture/board artifact adapter, not Topal language semantics. It wraps the
structurally validated linked kernel and is the artifact passed to QEMU's
`-kernel` option on the pinned `pc-q35-10.2` board.

The adapter is entirely compiler generated. Topal source does not contain
real-mode, protected-mode, long-mode, register, descriptor-table, page-table,
or instruction spelling.

## Image and handoff contract

The first adapter has this exact contract:

- four setup sectors follow the legacy boot sector, and the setup header
  advertises Linux x86 boot protocol 2.15;
- the setup entry is relocatable 16-bit code and enters the protected payload
  at physical address `0x0010_0000`;
- compiler-generated transition support occupies the range beginning at
  `0x0010_0000`; the validated kernel ELF `PT_LOAD` segments are materialized
  at their physical addresses beginning at `0x0020_0000` with at least 4 KiB
  alignment;
- the initial page tables identity-map the first 1 GiB with 2 MiB leaves;
- the bootstrap stack grows down from `0x0018_0000`, inside the reserved gap
  between transition support and kernel segments;
- the adapter generates the GDT, page tables, IDT, and descriptor pointers;
  IDT vector 3 names the linked generated debug-break entry;
- the adapter enters 64-bit mode with interrupts disabled, the selected data
  segments loaded, and the QEMU/bootloader `boot_params` physical address
  zero-extended in `RSI`; and
- the final transfer jumps to the linked bootstrap entry. The adapter does not
  call it using a process ABI and supplies no return path.

The fixed transition layout is:

| Physical range/address | Purpose |
| --- | --- |
| `0x0010_0000` | 32-bit and 64-bit transition code |
| `0x0010_1000` | PML4 |
| `0x0010_2000` | PDPT |
| `0x0010_3000` | page directory with 2 MiB identity leaves |
| `0x0010_4000` | GDT |
| `0x0010_5000` | 256-entry IDT |
| `0x0010_6000` | IDT descriptor and adapter data |
| `0x0018_0000` | initial stack top |
| `0x0020_0000` and above | materialized linked kernel segments |

Every occupied interval is bounds-checked before packaging. The adapter rejects
a kernel segment below `0x0020_0000`, beyond the initial 1 GiB mapping, with
overlapping file/memory ranges, or whose entry and debug-break symbols are not
inside executable load segments. Uninitialized segment tails are explicitly
zero-filled in the protected payload.

## Boot header and evidence

The generated setup header records a high-loaded, non-relocatable 64-bit
kernel, `code32_start = 0x0010_0000`, 4 KiB kernel alignment, the protected
payload size, and no embedded initramfs. QEMU remains responsible for the
command-line pointer and other bootloader-written fields. The adapter preserves
the boot-parameter address even though the current toolchain-gate root does not
yet consume the command line or firmware map.

Packaging publishes the `bzImage` and canonical adapter provenance together.
The record includes the exact protocol/header identity, transition layout,
linked kernel digest, boot image digest, entry addresses, QEMU machine/CPU, and
provider revisions. Structural inspection precedes publication; QEMU execution
is a separate physical-evidence stage.

## Alternatives and portability

An ELF parser inside the bootstrap payload was rejected for the initial gate:
materializing reviewed `PT_LOAD` segments during packaging is smaller and
removes a privileged runtime parser. UEFI/PE entry remains a possible later
adapter but would not exercise the Step 3 replacement seam. Requiring a
64-bit-aware external bootloader was rejected because QEMU's selected direct
Linux boot path starts at the setup entry.

The 16-bit transition, fixed physical layout, x86 descriptors, and page-table
format are x86 adapter facts. AArch64 and RISC-V adapters may use different
firmware handoffs, load layouts, and transition mechanisms without changing the
systems entry, storage, observation, or disposition semantics.

This contract was approved in the project discussion before implementation.
