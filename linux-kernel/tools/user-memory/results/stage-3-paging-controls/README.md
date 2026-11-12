# Stage 3: saved linked paging-control inventory

`TK-DEC-283`, `TOPAL-SYSTEMS-USER-EXTENT-001`.

The [reader](../../inventory_paging_controls.py) inventories selected privileged
control families in the unchanged linked kernel. Fresh LLVM **22.1.6** and GNU
objdump **2.46.1** listings cover every byte of its **1267581-byte** executable
section. They have **221365** and **221366** listing rows respectively. These are
linear interpretations, including padding and unvisited bytes, not complete
sets of reachable instructions. The kernel digest remains
`47b785aca3794d171d508a3fdbb616057a613016c1783ca9ec95236dbba2199f`.

The [inventory](inventory.json) retains the union of **847** selected controls:

| Family | Listed sites |
| --- | ---: |
| Control-register reads / writes | 297 / 10 |
| MSR reads / writes | 17 / 27 |
| Page invalidations | 29 |
| GS-base swaps | 218 |
| Interrupt disables / enables / returns | 26 / 4 / 6 |
| Syscall returns | 213 |

Both decoders agree on **846** sites. The unmatched LLVM `iretd` at
`0x2bbaa5d` remains explicitly unresolved. A preceding unknown instruction has
different decoded lengths, so GNU consumes that byte in another instruction.
The site is absent from the preceding process graph; that absence is not proof
that every alternate entry is impossible. The inventory reports
`full_decoder_control_agreement: false`. A successful inventory check therefore
does not mean complete interpretation agreement or security qualification.

LLVM retains **63 unknown rows**, **153 overlapping decoded bytes** at padding/
symbol boundaries and **three isolated LOCK-prefix rows**. GNU retains **60 bad
rows** and **one isolated REX-prefix row**. Every row's bytes must match the ELF;
the union must cover the entire section. Overlap is retained, not silently
clipped or treated as a unique instruction partition.

The reader binds **523 selected controls** to exact bytes in the preceding
process graph. That graph has three instruction starts absent from the whole
linear listing; its recovered interpretations remain separate. Seven of the
control-register writes belong to that process graph. Three more belong to the
linked static-image activation/restoration and bootstrap-translation functions.
The selected controls therefore extend beyond the local heap publisher and its
previously observed terminal handoff.

An all-byte scan retains **67** candidates for the three declared critical
opcode families: control-register writes, MSR writes and page invalidations.
One MSR-write opcode sequence is embedded inside another listed instruction.
Both listings retain its containing row. Embedded bytes are neither evidence
that an operation executes nor evidence that alternate entry is impossible.
This scan does not cover every x86 privileged instruction family.

[Qualification](qualification.json) passes the saved-code inventory and **nine
targeted reader tests**. Tests reject missing bytes, byte/operand disagreement,
duplicate starts and mislabelled control/MSR/invalidation operations, and retain
prefix fragments, padding overlaps and embedded/discordant interpretations.
Three source identities and four consumed artifact/listing identities are equal
before/after. Peak cgroup memory is **348233728 bytes**, with zero swap and zero
memory-event counters. No VM, compiler or guest application executes here.

Four earlier reader attempts remain separately failed: an unhandled separate
LOCK prefix, the initial rejection of overlapping padding, an isolated GNU REX
prefix and a strict decoder-agreement assertion. Their exact source snapshots
and failure summaries remain. Original complete stderr files were not saved;
the summaries are not represented as raw transcripts. The last diagnostic's
exact [listing difference](listing-differences.json) is retained. These are
reader failures, not reported AI safeguard actions or guest/kernel failures.

[Inputs](inputs.json) retain three source snapshots and two new compressed
listings. Kernel and process-graph artifacts are inherited by exact compressed/
raw identity from existing first-party evidence. No new ELF, initramfs or
third-party binary is added. [Replay](replay.json) restores these selected inputs,
reruns the nine tests and reproduces the exact inventory using Python only:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-paging-controls/replay.py linux-kernel/.cache/x86_64/paging-controls-replay-new
```

Use a new output directory. [Validation](validate-archive.py) checks selected
current sources, packed/raw evidence, exact report bindings, logs, failure
summaries, replay, links and the archive manifest. Hashes bind selected bytes;
they cannot authenticate observations or create kernel authority.

Risk remains high because an incomplete instruction inventory could hide a
mode change. Review is single-agent higher-authority/source/diff review plus
two installed decoders, raw opcode checks, nine counterexample tests and frozen
replay. These checks share reader code; no independent human review is claimed.

Kernel/protection behavior, core-language intent, host safeguards and production
selection remain unchanged. An all-entry/control-flow and complete mapping-store
inventory, runtime-generated/boot/AP code, native coverage, continuous mode/TLB
history, AP/DMA/asynchronous exclusion, stable lifetime, remaining caller/effect
contracts and canonical clean production qualification remain required. This
inventory closes none of those broader gates. Stage 3, initial Step 4 and full
Linux distribution/container/VM-host acceptance remain unfinished.
