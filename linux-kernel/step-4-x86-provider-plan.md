# Step 4 x86-64 provider plan

## Outcome

This is the third implementation increment of Step 4. The compiler can now
derive one deterministic, sealed provider plan from the checked systems
program. The plan binds:

- target `x86_64-unknown-none`, board `topal-qemu-pc-q35-10.2`, and systems
  profile `topal.systems.x86_64-qemu-pc-q35-10.2/1`;
- provider revision `topal.provider.x86_64-qemu-pc-q35/1` and platform ABI
  `topal.systems.x86_64-bare/1`;
- the `qemu64-v1` machine CPU, `x86-64` code-generation baseline, exact LLVM
  data layout, ELF64 object format, static relocation model, and small code
  model;
- the checked bootstrap-storage capacity, alignment, and semantic placement;
  and
- every admitted initial semantic operation to one closed provider lowering.

The operation map selects static bootstrap storage, monotonic allocation,
region consumption, bootstrap completion, polled 16550 port I/O, breakpoint
vector 3, interrupt return, and an interrupts-disabled halt path. Source never
names these mechanisms and cannot construct a provider mapping.

## Portability boundary

Stable systems semantic identities remain common. This provider alone chooses
16550 port I/O and x86 exception/fatal mechanisms. AArch64 and RISC-V providers
may refine the same console, synchronous observation, resume, fatal, storage,
and placement meanings with different mechanisms; this plan gives them no
executable status.

The plan deliberately stops before LLVM IR, generated entry support, an object
file, a linked image, or a boot adapter. The target remains model-only and the
existing compiler continues to reject it before output. The next increment
must lower only this plan into a structurally inspected freestanding artifact.

## Validation and risk

Focused tests prove the complete eight-operation mapping, exact target and
artifact identities, deterministic derivation, absence of hosted platform
identities, and rejection of a program with the wrong board. All 181 compiler
library tests and all four coverage/traceability tests pass. The changed
compiler path passes strict Clippy after allowing only the compiler crate's
four unchanged baseline findings, and formatting checks are clean.

Risk is high because incorrect mapping could turn a portable semantic element
into an x86-specific source promise or grant machine authority to an
unqualified target. The plan is closed, derived only from a fully validated
systems program, carries exact target evidence, and is not accepted by the
physical publication path.
