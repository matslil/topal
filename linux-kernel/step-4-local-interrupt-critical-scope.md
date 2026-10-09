# Step 4 local-interrupt critical-scope increment

## Outcome

The checked kernel root now enters and restores the approved affine
`local-maskable-interrupts` critical scope. Entry consumes the current context,
returns a critical context carrying opaque exact-prior-state authority, and
admits only the scoped nonblocking marker in the initial source slice.
Restoration consumes that authority and returns the refined continuation
context before debug-break handling or bootstrap storage use can proceed.

The architecture-independent model assigns processor and nesting identities,
retains whether local maskable interrupts were enabled or disabled before
entry, restores that exact state, and models nested restoration in
last-in-first-out order. It rejects processor transfer, restoration without
authority, a live scope at completion, and potentially suspending work inside
the scope. The source checker additionally rejects a nonlocal domain, prior-
context use, a mismatched restore receiver, missing restoration, and nonfatal
entry failure.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/7` adds two typed
operations. Entry privately captures `RFLAGS` with `pushfq`/`pop`, executes
`cli`, and returns the flags as an opaque nonzero token to generated root code.
Restore validates the architectural fixed bit, tests only `RFLAGS.IF`, and
selects `sti` for a previously enabled state or `cli` for a previously disabled
state. It does not restore unrelated flags and never exposes the token,
register, bit position, or instructions to Topal source.

The root retains the token in compiler-selected state, branches to the fatal
provider if either typed provider path fails, consumes the token after restore,
and rejects completion while restoration authority remains live. The initial
x86 provider admits one non-nested scope after the active-translation edit has
returned all borrowed memory authority. The common semantic model nevertheless
defines nested LIFO behavior for later qualified providers.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/7` records 27
provider-plan identities, 24 linked semantic placements, and a 42-transition
trace. Gate schema `topal-kernel-toolchain-gate-qemu/8` binds those revisions
and the unchanged boot-adapter revision `/2`.

## Evidence

- all 71 semantic-library tests pass, including exact enabled/disabled
  restoration, nested LIFO restoration, processor identity, live-scope, and
  blocking-operation rejection;
- all 14 systems source tests pass, including domain, receiver, escape,
  restoration, and scoped-operation negative cases;
- all 188 compiler unit tests pass, including the 27-operation provider plan,
  exact generated enter/restore byte sequences, typed dependencies, 275 root
  relocations, 239 UART calls, 18 fatal branches, 24 placements, and ordered
  42-transition trace;
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced identical linked
  kernel `3a8e24425e807698c0417b72bb2c79e6ea8608a50365cbf4bd00821897ea0172`,
  boot image `cb02ae496f05c69e81ac96c0f02be0eb6ca545c47acb35fafe241eeec82d1982`,
  artifact provenance `fce9eeb92200b1b9f68377223eb5e3abedf3b38d76ad488aed114e202012e1db`,
  boot provenance `07dab859a285e652da24ffd935b746bc4054c1497fceaa66c57d32cc49c28f66`,
  and serial observation
  `6d5f8a285e13f4cc2f61af682a3304368ce27f2c350eb48a63a24b136f5cadd6`.

The serial stream emits `TOPAL_KERNEL_INTERRUPTS_MASKED` from inside the
critical continuation, then emits `TOPAL_KERNEL_MEMORY_DESCRIBED` only through
the restored context. The boot adapter enters this path with maskable
interrupts disabled, so this run exercises preservation of the disabled prior
state. Structural inspection covers both the disabled and conditional-enabled
restore paths; no runtime claim is made for an enabled-entry scenario yet.

## Remaining boundary

This increment does not add interrupt entry, acknowledgement, routing,
controller masking, preemption control, nested x86 scopes, multi-processor
exclusion, an interrupt-safe synchronization library, or enabled-entry runtime
qualification. Non-maskable events, other processors, devices, and DMA agents
remain outside the exclusion domain. AArch64 `DAIF` and RISC-V supervisor
interrupt state pressure-test the abstraction only; they remain model-only.
