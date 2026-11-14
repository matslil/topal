# Owned setup storage for processor startup

`TK-DEC-295` is a systems-only native prerequisite of the participant entry
contract. The initial x86 startup route needs retained storage below 1 MiB.
Derive it from an explicitly selected boot adapter's complete loaded setup
image. Source continues to request a provider-selected typed entry; it receives
no address or native storage geometry. Native participant initialization stays
unqualified while this prerequisite is developed.

The private adapter profile declares sixteen setup sectors including the boot
sector, for 8,192 bytes. Its second complete 4,096-byte page is startup storage,
loaded from the same freshly reconstructed adapter image. The first page keeps
the boot parameters and complete bounded E820 table. The adapter's real-mode
continuation and stack occupy separate declared offsets and transfer out before
the kernel claims the startup page. Existing adapter profiles retain their
original setup size and byte layout.

Both prepared inactive banks must also retain the startup page's supervisor
RW/NX identity view and supported direct-memory cache encoding. Check those
actual owned-bank bytes before the write probe and again before table retirement;
keep the preceding eleven-role journal unchanged and record this extra role
separately. Inactive reachability grants no execution permission or activation.

Linux defines setup size and protected-payload offset through `setup_sects`.
The pinned QEMU loader publishes that complete extent through fw_cfg; its
Linux option ROM copies the specified setup bytes before entering the setup
code. References: [Linux x86 boot protocol, sections 1.3 and 1.9](https://docs.kernel.org/arch/x86/boot.html),
[QEMU 10.2.2 loader](https://github.com/qemu/qemu/blob/v10.2.2/hw/i386/x86-common.c)
and [DMA option ROM](https://github.com/qemu/qemu/blob/v10.2.2/pc-bios/optionrom/linuxboot_dma.c).
This is the qualified lab route; it does not establish arbitrary boot-loader
support or a CPU-entry protocol.

The loaded setup extent and its disjoint first/startup page roles belong to one
boot-adapter instance. The provider derives an internal `BootStartupStorage`
owner from that handoff, complete artifact reconstruction and actual byte/range
checks. A reserved allocator prefix alone does not grant ownership. It prevents
the already owned page from being reassigned by ordinary frame allocation.
This resource does not fabricate a frame-allocation token or release boot
storage through a different issuer.

Before writing, native checks bind the actual aligned low setup base, exact
profile header, complete startup-page pattern, whole RAM coverage and absence
of conflicting firmware claims. Bound all E820 reads within the first page and
retain the complete relevant raw table for later comparison. Check exclusion
from BDA/EBDA/BIOS, command-line and extended setup inputs, high boot/kernel/
resource storage, and the adapter's continuation/stack. Unsupported inputs or
overlap fail fatally before the write probe.

Claiming is once per boot instance. The provider writes and reads back a
complete deterministic probe pattern, restores and verifies the loaded pattern,
then holds the page for kernel lifetime. Normal user/exec/root activity may run
without borrowing it. Before the inactive table scope retires, independently
revalidate its exact handoff/owner fields, complete firmware table and page
bytes under the current context. Initial register snapshots remain historical.
Failure retains/poisons boot-owned storage. No ordinary source operation may
free, alias or repurpose the page, and no frame-owner validator is widened.

The private journal is evidence, not an ownership capability manufactured by
numeric fields. Independent VM observation must bind the actual loaded file,
native before/write/restore/later states, complete bytes, reservations and
ordinary continuation. Altered saved data tests are model/replay evidence.
CPU startup, entry, asynchronous exclusion, memory-order/deadline qualification
and all-root protection remain separate obligations. This increment sends no
startup request and grants no participant arrival or protected-image witness.
